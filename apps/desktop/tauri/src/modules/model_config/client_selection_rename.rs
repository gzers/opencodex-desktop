//! Rename only model/review_model through Codex's native configuration owner.
//! The restart caller supplies the verified executable and client identity; this
//! module neither discovers Codex nor reads/migrates saved conversations.

use crate::{
    infrastructure::platform,
    modules::model_config::schema::{invalid, ModelError, ModelResult},
    state::ProcessContext,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(20);
const REPAIR_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_LINE: usize = 2 * 1024 * 1024;
const MAX_STDOUT: usize = 8 * 1024 * 1024;
const MAX_STDERR: usize = 256 * 1024;

/// Both outcomes confirm only the current-selection step. The caller should
/// acknowledge its pending mapping only after its entire restart flow succeeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSelectionRename {
    Unchanged,
    Updated,
}

struct Target {
    context: ProcessContext,
    executable: PathBuf,
    home: PathBuf,
    codex_home: PathBuf,
    cwd: PathBuf,
    profile: Option<String>,
}

/// Opaque, single-use intent. No config contents, paths or selectors are exposed
/// through Debug/Serialize. Keep this in memory across the official producer.
#[must_use]
pub struct ClientSelectionRenamePlan {
    target: Option<Target>,
    intent: Option<SelectionIntent>,
}

struct SelectionIntent {
    edits: Vec<SelectionEdit>,
}

struct SelectionEdit {
    before: Selection,
    value: String,
}

/// This closed set is intentional: no unknown/future configuration keys or
/// historical thread fields participate in current-reference migration.
#[derive(Clone, Copy)]
enum CurrentScalar {
    Model,
    ReviewModel,
}

impl CurrentScalar {
    const ALL: [Self; 2] = [Self::Model, Self::ReviewModel];

    fn key(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::ReviewModel => "review_model",
        }
    }
}

impl SelectionIntent {
    fn preflight(
        read: &ConfigRead,
        target: &Target,
        renames: &BTreeMap<String, String>,
    ) -> ModelResult<Option<Self>> {
        let mut edits = Vec::new();
        for key in CurrentScalar::ALL {
            // No owner origin (e.g. an unset/default field) is not permission
            // to invent a user-root write. None/missing-origin fields are noops.
            if !read.origins.contains_key(key.key()) {
                continue;
            }
            let Some((provider, suffix)) = read
                .config
                .value(key)
                .and_then(|value| value.split_once('/'))
            else {
                continue;
            };
            let Some(replacement) = renames.get(provider).filter(|new| new.as_str() != provider)
            else {
                continue;
            };
            edits.push(SelectionEdit {
                before: read.selection(target, key)?,
                value: format!("{replacement}/{suffix}"),
            });
        }
        let Some(first) = edits.first() else {
            return Ok(None);
        };
        // A single native batch is atomic only within one file/profile owner.
        // Refuse mixed owners before any write rather than splitting a plan.
        if edits.iter().any(|edit| {
            edit.before.file != first.before.file
                || edit.before.source != first.before.source
                || edit.before.version != first.before.version
        }) {
            return Err(failure("source"));
        }
        Ok(Some(Self { edits }))
    }

    fn file(&self) -> &Path {
        &self.edits[0].before.file
    }
}

/// Read-only preflight before live reconciliation or injection. All client paths
/// and the optional profile-v2 name must come from verified official bindings.
/// Only a full provider segment matches; the model suffix is preserved verbatim.
pub async fn preflight_current_selection(
    context: &ProcessContext,
    codex_executable: &Path,
    home_directory: &Path,
    codex_home: &Path,
    working_directory: &Path,
    active_profile: Option<&str>,
    provider_renames: &BTreeMap<String, String>,
) -> ModelResult<ClientSelectionRenamePlan> {
    if provider_renames.is_empty() {
        return Ok(ClientSelectionRenamePlan {
            target: None,
            intent: None,
        });
    }
    if provider_renames
        .iter()
        .any(|(old, new)| !provider_segment(old) || !provider_segment(new))
    {
        return Err(failure("mapping"));
    }
    let target = Target {
        context: context.clone(),
        executable: codex_executable.into(),
        home: home_directory.into(),
        codex_home: codex_home.into(),
        cwd: working_directory.into(),
        profile: active_profile.map(str::to_owned),
    };
    target.validate()?;
    let mut owner = Owner::spawn(&target)?;
    let result = tokio::time::timeout(OPERATION_TIMEOUT, async {
        owner.initialize().await?;
        let read = owner.read(&target.cwd).await?;
        SelectionIntent::preflight(&read, &target, provider_renames)
    })
    .await
    .unwrap_or_else(|_| Err(failure("timeout")));
    let closed = owner.close().await;
    let intent = result?;
    if !closed {
        return Err(failure("cleanup"));
    }
    Ok(ClientSelectionRenamePlan {
        target: Some(target),
        intent,
    })
}

impl ClientSelectionRenamePlan {
    pub fn has_changes(&self) -> bool {
        self.intent
            .as_ref()
            .is_some_and(|intent| !intent.edits.is_empty())
    }
    /// Run after the producer. Injection may change unrelated fields and the
    /// file version, but a changed source/profile/model invalidates this intent.
    /// The write uses the latest observed owner version, not preflight's token.
    pub async fn apply(self) -> ModelResult<ClientSelectionRename> {
        self.apply_checked(None).await
    }
    /// Bind the CAS to the exact post-producer file whose routing owner the
    /// caller verified. Later unrelated changes must not silently rebind it.
    pub async fn apply_at_version(self, version: String) -> ModelResult<ClientSelectionRename> {
        if !version_token(&version) {
            return Err(failure("changed"));
        }
        self.apply_checked(Some(version)).await
    }
    async fn apply_checked(
        self,
        expected_version: Option<String>,
    ) -> ModelResult<ClientSelectionRename> {
        let Some(intent) = self.intent else {
            return Ok(ClientSelectionRename::Unchanged);
        };
        let target = self.target.ok_or_else(|| failure("paths"))?;
        target.validate()?;
        let mut owner = Owner::spawn(&target)?;
        let mut written_version: Option<String> = None;
        let result = tokio::time::timeout(OPERATION_TIMEOUT, async {
            owner.initialize().await?;
            let read = owner.read(&target.cwd).await?;
            let mut version = None;
            for edit in &intent.edits {
                let current = read.selection(&target, edit.before.key)?;
                if current.file != edit.before.file
                    || current.source != edit.before.source
                    || current.value != edit.before.value
                {
                    return Err(failure("changed"));
                }
                if version
                    .as_ref()
                    .is_some_and(|version| version != &current.version)
                {
                    return Err(failure("changed"));
                }
                version = Some(current.version);
            }
            let version = version.ok_or_else(|| failure("source"))?;
            if expected_version
                .as_ref()
                .is_some_and(|expected| expected != &version)
            {
                return Err(failure("changed"));
            }
            let receipt = owner.write(&version, &intent, false).await?;
            // A valid receipt is necessary for compensation. Never replace this
            // token with an external edit's version during rollback.
            if receipt.file_path != intent.file()
                || !version_token(&receipt.version)
                || receipt.version == version
            {
                return Err(failure("unknown"));
            }
            written_version = Some(receipt.version.clone());
            if receipt.status != "ok" || receipt.overridden_metadata.is_some() {
                return Err(failure("overridden"));
            }
            let after = owner.read(&target.cwd).await?;
            for edit in &intent.edits {
                let actual = after.selection(&target, edit.before.key)?;
                if actual.value != edit.value
                    || actual.file != edit.before.file
                    || actual.source != edit.before.source
                    || actual.version != receipt.version
                {
                    return Err(failure("verification"));
                }
            }
            Ok(ClientSelectionRename::Updated)
        })
        .await
        .unwrap_or_else(|_| Err(failure("timeout")));
        let closed = owner.close().await;
        if let Err(error) = result {
            if let Some(version) = written_version {
                // A fresh owner also handles EOF/timeouts in the first transport.
                let repaired =
                    tokio::time::timeout(REPAIR_TIMEOUT, compensate(&target, &intent, &version))
                        .await;
                if !matches!(repaired, Ok(Ok(()))) {
                    return Err(failure("repair_conflict"));
                }
            }
            return Err(error);
        }
        if !closed {
            return Err(failure("cleanup"));
        }
        result
    }
}

fn provider_segment(value: &str) -> bool {
    !value.is_empty() && !value.contains('/') && !value.chars().any(char::is_control)
}

fn version_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}

fn failure(reason: &str) -> ModelError {
    let message = match reason {
        "source" => "当前 Codex 当前模型引用来源不支持安全更新；未盲写用户根配置",
        "executable" => "当前 Codex 运行入口不支持受控配置更新；请绑定官方原生可执行文件后重试",
        "changed" => "Codex 当前模型引用或来源已改变；未覆盖新选择，请重新检查后重试",
        "overridden" => "Codex 当前模型引用写入被其它配置层覆盖，未确认生效",
        "repair_conflict" => {
            "Codex 当前模型引用更新未确认，补偿遇到版本变化或无法核验；未覆盖后续编辑，请检查后重试"
        }
        "unknown" | "timeout" => "Codex 当前模型引用更新结果未确认，请检查后重试",
        _ => "Codex 当前模型引用引用更新未完成，请检查后重试",
    };
    invalid(&format!("client_selection_{reason}"), message)
}

impl Target {
    fn validate(&self) -> ModelResult<()> {
        if !self.executable.is_absolute()
            || !self.executable.is_file()
            || [
                &self.home,
                &self.codex_home,
                &self.cwd,
                &self.context.working_directory,
                &self.context.opencodex_home,
            ]
            .iter()
            .any(|path| !path.is_absolute() || !path.is_dir())
        {
            return Err(failure("paths"));
        }
        validate_executable_kind(&self.executable, cfg!(windows))?;
        if self.profile.as_deref().is_some_and(|profile| {
            profile.is_empty()
                || profile == "."
                || profile == ".."
                || profile.contains(['/', '\\'])
                || profile.chars().any(char::is_control)
        }) {
            return Err(failure("source"));
        }
        Ok(())
    }

    fn file(&self, profile: Option<&str>) -> ModelResult<PathBuf> {
        let name = match profile {
            Some(profile) if self.profile.as_deref() == Some(profile) => {
                format!("{profile}.config.toml")
            }
            None => "config.toml".into(),
            _ => return Err(failure("source")),
        };
        let home = dunce::canonicalize(&self.codex_home).map_err(|_| failure("source"))?;
        let file = dunce::canonicalize(home.join(&name)).map_err(|_| failure("source"))?;
        // A custom CODEX_HOME is supported; a config symlink escaping that
        // explicit owner boundary is not an authorization to write elsewhere.
        if file.parent() != Some(home.as_path())
            || file.file_name() != Some(std::ffi::OsStr::new(&name))
        {
            return Err(failure("source"));
        }
        Ok(file)
    }
}

/// Rust may implicitly launch cmd.exe for a batch path. Until a verified native
/// runtime binding is available, refuse wrappers before spawning anything. RPC
/// JSON is always written to stdin, never interpolated into a shell command.
fn validate_executable_kind(executable: &Path, windows: bool) -> ModelResult<()> {
    let extension = executable
        .extension()
        .and_then(|part| part.to_str())
        .unwrap_or("");
    if ["cmd", "bat"]
        .iter()
        .any(|kind| extension.eq_ignore_ascii_case(kind))
        || (windows && !extension.eq_ignore_ascii_case("exe"))
    {
        return Err(failure("executable"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct ConfigRead {
    config: CurrentConfig,
    origins: BTreeMap<String, Metadata>,
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
struct CurrentConfig {
    model: Option<String>,
    review_model: Option<String>,
    profile: Option<String>,
}

impl CurrentConfig {
    fn value(&self, key: CurrentScalar) -> Option<&str> {
        match key {
            CurrentScalar::Model => self.model.as_deref(),
            CurrentScalar::ReviewModel => self.review_model.as_deref(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct Source {
    #[serde(rename = "type")]
    kind: String,
    file: Option<PathBuf>,
    profile: Option<String>,
}

#[derive(Deserialize)]
struct Metadata {
    name: Source,
    version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layer {
    name: Source,
    version: String,
    config: CurrentConfig,
    disabled_reason: Option<String>,
}

struct Selection {
    key: CurrentScalar,
    value: String,
    source: Source,
    file: PathBuf,
    version: String,
}

impl ConfigRead {
    fn selection(&self, target: &Target, key: CurrentScalar) -> ModelResult<Selection> {
        let origin = self
            .origins
            .get(key.key())
            .ok_or_else(|| failure("source"))?;
        if origin.name.kind != "user"
            || !version_token(&origin.version)
            || self
                .config
                .profile
                .as_deref()
                .is_some_and(|profile| target.profile.as_deref() != Some(profile))
        {
            return Err(failure("source"));
        }
        let file = target.file(origin.name.profile.as_deref())?;
        if origin.name.file.as_ref() != Some(&file) {
            return Err(failure("source"));
        }
        let value = self.config.value(key).ok_or_else(|| failure("source"))?;
        let layer = self
            .layers
            .iter()
            .find(|layer| layer.name == origin.name)
            .ok_or_else(|| failure("source"))?;
        if layer.disabled_reason.is_some()
            || layer.version != origin.version
            || layer.config.value(key) != Some(value)
        {
            return Err(failure("source"));
        }
        if let Some(profile) = target.profile.as_deref() {
            let profile_file = target.file(Some(profile))?;
            if !self.layers.iter().any(|layer| {
                layer.name.kind == "user"
                    && layer.name.profile.as_deref() == Some(profile)
                    && layer.name.file.as_ref() == Some(&profile_file)
                    && layer.disabled_reason.is_none()
            }) {
                return Err(failure("source"));
            }
        }
        Ok(Selection {
            key,
            value: value.into(),
            source: origin.name.clone(),
            file,
            version: origin.version.clone(),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteReceipt {
    file_path: PathBuf,
    status: String,
    version: String,
    overridden_metadata: Option<Value>,
}

async fn compensate(target: &Target, intent: &SelectionIntent, version: &str) -> ModelResult<()> {
    let mut owner = Owner::spawn(target)?;
    let result = async {
        owner.initialize().await?;
        let receipt = owner.write(version, intent, true).await?;
        if !matches!(receipt.status.as_str(), "ok" | "okOverridden")
            || receipt.file_path != intent.file()
            || !version_token(&receipt.version)
        {
            return Err(failure("repair_conflict"));
        }
        let read = owner.read(&target.cwd).await?;
        // Compensation restores both original scalars in one CAS, even if a
        // different layer now wins. Never use a newer external version token.
        for edit in &intent.edits {
            if !read.layers.iter().any(|layer| {
                layer.name == edit.before.source
                    && layer.version == receipt.version
                    && layer.config.value(edit.before.key) == Some(edit.before.value.as_str())
            }) {
                return Err(failure("repair_conflict"));
            }
        }
        Ok(())
    }
    .await;
    if !owner.close().await {
        return Err(failure("cleanup"));
    }
    result
}

// Deliberately only the four native methods needed for these two scalars, not a
// general RPC client. Stdout, stderr, wall time and process lifetime are bounded.
struct Owner {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    stderr: tokio::task::JoinHandle<ModelResult<()>>,
    next_id: u64,
    remaining: usize,
    boundary: ProcessBoundary,
    _scratch: tempfile::TempDir,
}

impl Owner {
    fn spawn(target: &Target) -> ModelResult<Self> {
        let scratch = tempfile::Builder::new()
            .prefix(".client-selection-")
            .tempdir_in(&target.context.opencodex_home)
            .map_err(|_| failure("spawn"))?;
        let mut command = Command::new(&target.executable);
        if let Some(profile) = target.profile.as_deref() {
            command.args(["--profile", profile]);
        }
        command
            .args(["app-server", "--stdio"])
            .current_dir(&target.cwd)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        platform::apply_user_environment(command.as_std_mut(), Some(target.home.as_os_str()));
        command
            .env("HOME", &target.home)
            .env("CODEX_HOME", &target.codex_home)
            .env("CODEX_SQLITE_HOME", scratch.path())
            .env("TMPDIR", scratch.path())
            .env("TMP", scratch.path())
            .env("TEMP", scratch.path())
            .env(
                "PATH",
                platform::controlled_path(std::iter::empty::<PathBuf>(), None),
            )
            .env("LANG", "C.UTF-8");
        #[cfg(windows)]
        command
            .env("APPDATA", target.home.join("AppData/Roaming"))
            .env("LOCALAPPDATA", target.home.join("AppData/Local"));
        let mut boundary = ProcessBoundary::prepare(&mut command)?;
        let mut child = command.spawn().map_err(|_| failure("spawn"))?;
        boundary.attach(&child)?;
        let input = child.stdin.take().ok_or_else(|| failure("spawn"))?;
        let output = BufReader::new(child.stdout.take().ok_or_else(|| failure("spawn"))?);
        let mut stderr = child.stderr.take().ok_or_else(|| failure("spawn"))?;
        let stderr = tokio::spawn(async move {
            let mut count = 0;
            let mut buffer = [0u8; 4096];
            loop {
                let size = stderr
                    .read(&mut buffer)
                    .await
                    .map_err(|_| failure("transport"))?;
                if size == 0 {
                    return Ok(());
                }
                count += size;
                if count > MAX_STDERR {
                    return Err(failure("output_limit"));
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            stderr,
            next_id: 0,
            remaining: MAX_STDOUT,
            boundary,
            _scratch: scratch,
        })
    }

    async fn initialize(&mut self) -> ModelResult<()> {
        self.call("initialize", json!({
            "clientInfo": {"name": "opencodex-desktop-current-selection", "version": env!("CARGO_PKG_VERSION")},
            "capabilities": {"experimentalApi": true}
        })).await?;
        self.send(json!({"method": "initialized"})).await
    }

    async fn read(&mut self, cwd: &Path) -> ModelResult<ConfigRead> {
        serde_json::from_value(
            self.call("config/read", json!({"cwd": cwd, "includeLayers": true}))
                .await?,
        )
        .map_err(|_| failure("response"))
    }

    async fn write(
        &mut self,
        version: &str,
        intent: &SelectionIntent,
        restore: bool,
    ) -> ModelResult<WriteReceipt> {
        let edits: Vec<_> = intent
            .edits
            .iter()
            .map(|edit| {
                json!({
                    "keyPath": edit.before.key.key(), "mergeStrategy": "replace",
                    "value": if restore { &edit.before.value } else { &edit.value }
                })
            })
            .collect();
        let (method, params) = if edits.len() == 1 {
            let mut params = edits.into_iter().next().expect("one scalar");
            params["filePath"] = json!(intent.file());
            params["expectedVersion"] = json!(version);
            ("config/value/write", params)
        } else {
            (
                "config/batchWrite",
                json!({"filePath": intent.file(), "expectedVersion": version,
                "edits": edits, "reloadUserConfig": false}),
            )
        };
        serde_json::from_value(self.call(method, params).await?).map_err(|_| failure("unknown"))
    }

    async fn send(&mut self, value: Value) -> ModelResult<()> {
        let mut bytes = serde_json::to_vec(&value).map_err(|_| failure("transport"))?;
        bytes.push(b'\n');
        self.input
            .write_all(&bytes)
            .await
            .map_err(|_| failure("transport"))?;
        self.input.flush().await.map_err(|_| failure("transport"))
    }

    async fn call(&mut self, method: &str, params: Value) -> ModelResult<Value> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({"id": id, "method": method, "params": params}))
            .await?;
        loop {
            let read = async {
                let mut line = Vec::new();
                loop {
                    let available = self
                        .output
                        .fill_buf()
                        .await
                        .map_err(|_| failure("transport"))?;
                    if available.is_empty() {
                        return Err(failure("transport"));
                    }
                    let length = available
                        .iter()
                        .position(|byte| *byte == b'\n')
                        .map_or(available.len(), |pos| pos + 1);
                    if line.len() + length > MAX_LINE || length > self.remaining {
                        return Err(failure("output_limit"));
                    }
                    line.extend_from_slice(&available[..length]);
                    self.output.consume(length);
                    self.remaining -= length;
                    if line.last() == Some(&b'\n') {
                        break;
                    }
                }
                serde_json::from_slice::<Value>(&line).map_err(|_| failure("response"))
            };
            let row = tokio::select! {
                value = read => value?,
                closed = &mut self.stderr => return Err(closed.ok().and_then(Result::err).unwrap_or_else(|| failure("transport"))),
            };
            if row.get("id").and_then(Value::as_u64) == Some(id) {
                if row.get("error").is_some() {
                    return Err(failure("write"));
                }
                return row
                    .get("result")
                    .cloned()
                    .ok_or_else(|| failure("response"));
            }
            // Notifications are ignored, never logged. Do not service requests
            // that could initiate tools, approvals, auth or network activity.
            if row.get("id").is_some() {
                return Err(failure("response"));
            }
        }
    }

    async fn close(&mut self) -> bool {
        self.boundary.terminate();
        let _ = self.child.start_kill();
        let waited = matches!(
            tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await,
            Ok(Ok(_))
        );
        self.stderr.abort();
        #[cfg(windows)]
        let waited = self.boundary.job.terminate_and_wait().await.is_ok() && waited;
        waited
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        self.boundary.terminate();
        let _ = self.child.start_kill();
        self.stderr.abort();
    }
}

struct ProcessBoundary {
    #[cfg(unix)]
    pid: Option<libc::pid_t>,
    #[cfg(windows)]
    job: crate::infrastructure::windows_query_job::OwnedQueryJob,
}

impl ProcessBoundary {
    fn prepare(command: &mut Command) -> ModelResult<Self> {
        #[cfg(unix)]
        {
            command.process_group(0);
            Ok(Self { pid: None })
        }
        #[cfg(windows)]
        {
            Ok(Self {
                job: crate::infrastructure::windows_query_job::OwnedQueryJob::new(command)
                    .map_err(|_| failure("spawn"))?,
            })
        }
    }

    fn attach(&mut self, child: &Child) -> ModelResult<()> {
        #[cfg(unix)]
        {
            self.pid = Some(child.id().ok_or_else(|| failure("spawn"))? as libc::pid_t);
            Ok(())
        }
        #[cfg(windows)]
        {
            self.job
                .assign_and_resume(child)
                .map_err(|_| failure("spawn"))
        }
    }

    fn terminate(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.pid.take() {
            // SAFETY: only the process group created for our own child is used.
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }
        }
        #[cfg(windows)]
        {
            let _ = self.job.terminate();
        }
    }
}

impl Drop for ProcessBoundary {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_policy_rejects_windows_wrappers_before_spawn() {
        for name in ["codex.cmd", "codex.CMD", "codex.bat", "codex.BAT"] {
            for windows in [false, true] {
                assert_eq!(
                    validate_executable_kind(Path::new(name), windows)
                        .unwrap_err()
                        .code,
                    "client_selection_executable"
                );
            }
        }
        assert!(validate_executable_kind(Path::new("codex.exe"), true).is_ok());
        assert!(validate_executable_kind(Path::new("codex.EXE"), true).is_ok());
        assert!(validate_executable_kind(Path::new("codex"), true).is_err());
        assert!(validate_executable_kind(Path::new("codex"), false).is_ok());
    }
}
