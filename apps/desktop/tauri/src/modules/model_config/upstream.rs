//! Official CLI adapter. Only explicit public verbs are admitted; no shell,
//! arbitrary dot-path writes, admin-token forwarding or private module imports.
use super::schema::*;
use crate::state::ProcessContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

const MAX_OUTPUT: u64 = 16 * 1024 * 1024;
#[cfg(unix)]
struct CommandGroup(Option<u32>);
#[cfg(unix)]
impl Drop for CommandGroup {
    fn drop(&mut self) {
        if let Some(pid) = self.0 {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ReadOperation {
    Version,
    ProviderSnapshot,
    LiveModels,
    ModelOrder,
    Catalog,
    Status,
    DisabledModels,
}
impl ReadOperation {
    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Version => &["--version"],
            Self::ProviderSnapshot => &["provider", "snapshot", "--json"],
            Self::LiveModels => &["models", "live", "--json"],
            Self::ModelOrder => &["models", "order", "status", "--json"],
            Self::Catalog => &["inspect", "catalog", "--json"],
            Self::Status => &["status", "--json"],
            // The official get verb rejects an absent optional key. A successful
            // redacted show proves absence without treating command failure as [].
            Self::DisabledModels => &["config", "show", "--json"],
        }
    }
}
pub async fn read(context: &ProcessContext, operation: ReadOperation) -> ModelResult<Value> {
    let executable = context.executable()?;
    if !executable.is_absolute()
        || !context.opencodex_home.is_absolute()
        || !context.working_directory.is_dir()
    {
        return Err(invalid("upstream_binding", "官方运行来源或数据根尚未绑定"));
    }
    read_command(
        &executable,
        &context.working_directory,
        &context.opencodex_home,
        operation,
    )
    .await
}
pub async fn read_command(
    executable: &Path,
    working: &Path,
    home: &Path,
    operation: ReadOperation,
) -> ModelResult<Value> {
    let output_limit = if matches!(operation, ReadOperation::Catalog) {
        256 * 1024 * 1024
    } else {
        MAX_OUTPUT
    };
    let mut command = tokio::process::Command::new(executable);
    command
        .args(operation.args())
        .current_dir(working)
        .env_clear()
        .env("OPENCODEX_HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    crate::infrastructure::platform::apply_user_environment(
        command.as_std_mut(),
        crate::infrastructure::platform::home_dir()
            .as_deref()
            .map(Path::as_os_str),
    );
    for key in [
        "PATH",
        "LANG",
        "LC_ALL",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "NO_PROXY",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    #[cfg(windows)]
    let job = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command)
        .map_err(|_| invalid("upstream_job", "无法建立只读查询进程边界"))?;
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .spawn()
        .map_err(|_| invalid("upstream_unavailable", "无法执行已绑定的官方命令"))?;
    #[cfg(unix)]
    let mut group = CommandGroup(child.id());
    #[cfg(windows)]
    job.assign_and_resume(&child)
        .map_err(|_| invalid("upstream_job", "只读查询进程未安全准入"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("upstream_output", "官方命令未提供输出"))?;
    let result = tokio::time::timeout(Duration::from_secs(20), async {
        let mut bytes = vec![];
        stdout
            .take(output_limit + 1)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| invalid("upstream_output", "官方输出读取失败"))?;
        if bytes.len() as u64 > output_limit {
            return Err(invalid("upstream_limit", "官方输出超出读取上限"));
        }
        let status = child
            .wait()
            .await
            .map_err(|_| invalid("upstream_unavailable", "官方命令结果待核对"))?;
        #[cfg(unix)]
        {
            group.0 = None;
        }
        if !status.success() {
            return Err(invalid(
                "upstream_rejected",
                "官方接口拒绝或尚未就绪；请检查运行来源和管理服务",
            ));
        }
        if matches!(operation, ReadOperation::Version) {
            let text =
                String::from_utf8(bytes).map_err(|_| invalid("upstream_output", "版本输出无效"))?;
            Ok(Value::String(text.trim().into()))
        } else if matches!(operation, ReadOperation::DisabledModels) {
            let config: Value = serde_json::from_slice(&bytes)?;
            if !config.is_object() {
                return Err(invalid("upstream_output", "官方配置只读投影结构无效"));
            }
            Ok(config
                .get("disabledModels")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([])))
        } else {
            Ok(serde_json::from_slice(&bytes)?)
        }
    })
    .await;
    match result {
        Ok(value) => value,
        Err(_) => Err(invalid("upstream_timeout", "官方只读请求超时，未写配置")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamSnapshot {
    pub version: String,
    pub providers: Value,
    pub models: Value,
    pub order: Value,
    pub catalog: Value,
}
pub async fn snapshot(context: &ProcessContext) -> ModelResult<UpstreamSnapshot> {
    let executable = context.executable()?;
    let frozen = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
            executable.clone(),
        ),
        working_directory: context.working_directory.clone(),
        opencodex_home: context.opencodex_home.clone(),
    };
    let version = read(&frozen, ReadOperation::Version)
        .await?
        .as_str()
        .unwrap_or_default()
        .to_string();
    // Capability failures are explicit, not silently replaced by static model ids.
    let providers = read(&frozen, ReadOperation::ProviderSnapshot).await?;
    if !providers.get("providers").is_some_and(Value::is_object) {
        return Err(invalid("upstream_shape", "官方渠道快照格式不受支持"));
    }
    let models = read(&frozen, ReadOperation::LiveModels).await?;
    if !models.is_array() {
        return Err(invalid("upstream_shape", "官方模型清单格式不受支持"));
    }
    let order = read(&frozen, ReadOperation::ModelOrder).await?;
    let catalog = read(&frozen, ReadOperation::Catalog).await?;
    if context.executable()? != executable {
        return Err(invalid(
            "stale_upstream",
            "运行来源在读取中改变，请重新核对",
        ));
    }
    reject_secrets(&providers)?;
    reject_secrets(&models)?;
    reject_secrets(&order)?;
    reject_secrets(&catalog)?;
    Ok(UpstreamSnapshot {
        version,
        providers,
        models,
        order,
        catalog,
    })
}

#[derive(Debug)]
pub struct OfficialWrite {
    pub success: bool,
    pub receipt: Value,
}
/// Only the adapter's private verbs can write. JSON operands use stdin, never shell text.
pub enum WriteOperation<'a> {
    Providers { baseline: &'a Path, next: &'a Value },
    Order(&'a [String]),
    Sync,
    EnableModel(&'a str),
    DisableModel(&'a str),
    DisabledModels(&'a [String]),
}
pub async fn write(
    context: &ProcessContext,
    operation: WriteOperation<'_>,
) -> ModelResult<OfficialWrite> {
    let management = match &operation {
        WriteOperation::Order(values) => Some((
            "/api/subagent-models",
            serde_json::json!({"pickerOrder":values,"pickerOrderMode":null}),
        )),
        WriteOperation::DisabledModels(values) => {
            Some(("/api/disabled-models", serde_json::json!({"models":values})))
        }
        _ => None,
    };
    if let Some((path, body)) = management {
        let management = super::management::Management::bind(context).await?;
        let receipt = management
            .request(reqwest::Method::PUT, path, Some(body))
            .await?;
        return Ok(OfficialWrite {
            success: true,
            receipt,
        });
    }
    let executable = context.executable()?;
    if !executable.is_absolute()
        || !context.opencodex_home.is_absolute()
        || !context.working_directory.is_dir()
    {
        return Err(invalid(
            "upstream_binding",
            "官方保存的运行来源或数据根尚未绑定",
        ));
    }
    let (args, input): (Vec<String>, Option<Vec<u8>>) = match operation {
        WriteOperation::Providers { baseline, next } => (
            vec![
                "provider".into(),
                "apply".into(),
                "--baseline".into(),
                baseline
                    .to_str()
                    .ok_or_else(|| invalid("plan_path", "计划路径编码不受支持"))?
                    .into(),
                "--file".into(),
                "-".into(),
                "--json".into(),
                "--yes".into(),
            ],
            Some(serde_json::to_vec(next)?),
        ),
        WriteOperation::Order(_) => {
            unreachable!("picker writes use the authenticated live management API")
        }
        WriteOperation::Sync => (vec!["sync".into()], None),
        WriteOperation::EnableModel(selector) => (
            vec![
                "models".into(),
                "enable".into(),
                selector.into(),
                "--json".into(),
            ],
            None,
        ),
        WriteOperation::DisableModel(selector) => (
            vec![
                "models".into(),
                "disable".into(),
                selector.into(),
                "--json".into(),
            ],
            None,
        ),
        WriteOperation::DisabledModels(_) => {
            unreachable!("visibility restore uses the authenticated live management API")
        }
    };
    let mut command = tokio::process::Command::new(executable);
    command
        .args(args)
        .current_dir(&context.working_directory)
        .env_clear()
        .env("OPENCODEX_HOME", &context.opencodex_home)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    crate::infrastructure::platform::apply_user_environment(
        command.as_std_mut(),
        crate::infrastructure::platform::home_dir()
            .as_deref()
            .map(Path::as_os_str),
    );
    for key in [
        "PATH",
        "LANG",
        "LC_ALL",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "NO_PROXY",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    #[cfg(windows)]
    let job = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command)
        .map_err(|_| invalid("upstream_job", "无法建立官方保存进程边界"))?;
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .spawn()
        .map_err(|_| invalid("upstream_write", "无法执行官方保存请求，运行配置尚未确认"))?;
    #[cfg(unix)]
    let mut group = CommandGroup(child.id());
    #[cfg(windows)]
    job.assign_and_resume(&child)
        .map_err(|_| invalid("upstream_job", "官方保存进程未安全准入"))?;
    let pipe = child.stdin.take();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("upstream_write", "官方保存输出不可用"))?;
    let result = tokio::time::timeout(Duration::from_secs(45), async {
        let send = async move {
            if let Some(input) = input {
                let mut pipe =
                    pipe.ok_or_else(|| invalid("upstream_write", "官方输入管道不可用"))?;
                pipe.write_all(&input).await.map_err(|_| {
                    invalid("upstream_write_unknown", "保存输入未完整传送，须核对原计划")
                })?;
                pipe.shutdown()
                    .await
                    .map_err(|_| invalid("upstream_write_unknown", "保存输入结束状态待核对"))?;
            }
            Ok::<_, ModelError>(())
        };
        let receive = async move {
            let mut bytes = vec![];
            stdout
                .take(MAX_OUTPUT + 1)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| invalid("upstream_write", "保存输出读取失败，须核对运行态"))?;
            if bytes.len() as u64 > MAX_OUTPUT {
                return Err(invalid("upstream_limit", "保存输出过大，须核对运行态"));
            }
            Ok::<_, ModelError>(bytes)
        };
        let (_, bytes) = tokio::try_join!(send, receive)?;
        let exit = child
            .wait()
            .await
            .map_err(|_| invalid("upstream_write", "保存结果待核对"))?;
        #[cfg(unix)]
        {
            group.0 = None;
        }
        Ok(OfficialWrite {
            success: exit.success(),
            receipt: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        })
    })
    .await;
    result.map_err(|_| {
        invalid(
            "upstream_write_unknown",
            "官方保存超时，可能已写入；请查询原计划，不要重复提交",
        )
    })?
}
pub(super) fn reject_secrets(value: &Value) -> ModelResult<()> {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                if [
                    "apiKey",
                    "headers",
                    "password",
                    "accessToken",
                    "refreshToken",
                    "idToken",
                    "token",
                    "clientSecret",
                    "proxy",
                ]
                .contains(&key.as_str())
                {
                    return Err(invalid(
                        "upstream_redaction",
                        "官方快照含不可回显字段，已拒绝传入界面",
                    ));
                }
                reject_secrets(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_secrets(value)?;
            }
        }
        Value::String(text) if text.starts_with("http://") || text.starts_with("https://") => {
            if let Ok(url) = reqwest::Url::parse(text) {
                if !url.username().is_empty() || url.password().is_some() {
                    return Err(invalid(
                        "upstream_redaction",
                        "官方快照地址含认证信息，已拒绝回显",
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[tokio::test]
    async fn bounded_official_read_uses_exact_verb_and_explicit_home() {
        let root = tempfile::tempdir().unwrap();
        let script = root.path().join("ocx");
        std::fs::write(
            &script,
            br#"#!/bin/sh
if [ "$1 $2 $3" != "provider snapshot --json" ]; then exit 3; fi
printf '{"defaultProvider":"test","providers":{},"home":"%s"}' "$OPENCODEX_HOME"
"#,
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let value = read_command(
            &script,
            root.path(),
            &root.path().join("home"),
            ReadOperation::ProviderSnapshot,
        )
        .await
        .unwrap();
        assert_eq!(value["defaultProvider"], "test");
        assert_eq!(
            value["home"],
            root.path().join("home").to_string_lossy().as_ref()
        );
    }
    #[test]
    fn snapshots_containing_credentials_never_reach_the_webview() {
        assert!(
            reject_secrets(&serde_json::json!({"providers":{"test":{"apiKey":"secret"}}})).is_err()
        );
        assert!(reject_secrets(
            &serde_json::json!({"baseUrl":"https://user:password@example.test"})
        )
        .is_err());
        assert!(
            reject_secrets(&serde_json::json!({"hasApiKey":true,"apiKeyTransport":"bearer"}))
                .is_ok()
        );
    }
}
