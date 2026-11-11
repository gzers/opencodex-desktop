//! Explicit official ocx entry. It is independent of the Desktop ocxd IPC client.
//! No installation/global npm, legacy config discovery or silent PATH takeover.
use super::{program_layout, program_selection};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, hash, locking::TargetFileLock, platform},
    modules::{backup::safety, data_root},
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

const RECORD: &str = "manager-state/terminal-cli.json";
const PENDING: &str = "manager-state/terminal-cli.pending.json";
const MAX: u64 = 16 * 1024 * 1024;
const MAX_PROFILE: usize = 256 * 1024;
const BEGIN: &str = "# >>> OpenCodeX Desktop ocx >>>";
const END: &str = "# <<< OpenCodeX Desktop ocx <<<";
const FLAG: &str = "--ocxd-runtime-cli";
fn fail(detail: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "terminal_cli_refused".into(),
        detail: detail.into(),
    }
}
fn io(_: std::io::Error) -> AppError {
    fail("终端入口文件操作失败，已有内容保留")
}
fn encode(value: &impl Serialize) -> AppResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| fail("终端入口记录无法编码"))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<T> {
    let raw = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes)
        .map_err(|_| fail("终端入口记录格式无效"))?;
    serde_json::from_value(raw).map_err(|_| fail("终端入口记录结构无效"))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Context {
    pub user_profile: PathBuf,
    pub desktop_root: PathBuf,
    pub opencodex_home: PathBuf,
    pub program_root: PathBuf,
    pub manager_executable: PathBuf,
}
impl Context {
    fn validate(&self) -> AppResult<()> {
        for path in [
            &self.user_profile,
            &self.desktop_root,
            &self.opencodex_home,
            &self.program_root,
            &self.manager_executable,
        ] {
            if !program_layout::clean_absolute(path) {
                return Err(fail("终端入口需要当前用户的明确路径"));
            }
        }
        super::bundled::roots(&self.program_root, &self.desktop_root)?;
        let (_, platform) = program_layout::host_target()?;
        let locations = program_layout::ProgramLocations::resolve(
            &self.manager_executable,
            &self.user_profile,
            platform,
        )?;
        if locations.program_root != self.program_root {
            return Err(fail("终端入口与当前应用程序位置不一致"));
        }
        let config = data_root::load_runtime_config(&self.desktop_root)?;
        let active = data_root::startup_v2::selected_config(
            &self.user_profile.join(".codex-desktop/desktop"),
        )?;
        if !data_root::runtime_binding_matches(&active, &self.desktop_root, &self.opencodex_home)
            || !data_root::runtime_binding_matches(
                &config,
                &self.desktop_root,
                &self.opencodex_home,
            )
        {
            return Err(fail("配置位置已变化，请重启管理器后核对终端入口"));
        }
        Ok(())
    }
    fn command(&self) -> PathBuf {
        self.user_profile.join(".local/bin/ocx")
    }
    fn bridge(&self) -> PathBuf {
        self.program_root.join("runtime/bin/ocx")
    }
    fn record(&self) -> PathBuf {
        self.desktop_root.join(RECORD)
    }
    fn pending(&self) -> PathBuf {
        self.desktop_root.join(PENDING)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Enable,
    Disable,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub action: Action,
    pub enabled: bool,
    pub pending: bool,
    pub version: Option<String>,
    pub command_path: PathBuf,
    pub runtime_entry: PathBuf,
    pub opencodex_home: PathBuf,
    pub profile_paths: Vec<PathBuf>,
    pub conflicts: Vec<PathBuf>,
    pub confirmation_digest: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub action: Action,
    pub enabled: bool,
    pub command_path: PathBuf,
    pub opencodex_home: PathBuf,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProfileEdit {
    path: PathBuf,
    block: String,
    was_absent: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Installed {
    schema_version: u32,
    context: Context,
    profiles: Vec<ProfileEdit>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Change {
    target: Target,
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "file",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Target {
    Command,
    Bridge,
    Profile(String),
    Record,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Transaction {
    schema_version: u32,
    context: Context,
    action: Action,
    changes: Vec<Change>,
}
fn quote(path: &Path) -> AppResult<String> {
    let path = path
        .to_str()
        .ok_or_else(|| fail("路径无法表示为终端文本"))?;
    if path.chars().any(char::is_control) {
        return Err(fail("路径包含终端控制字符"));
    }
    Ok(format!("'{}'", path.replace('\'', "'\\''")))
}
fn scripts(c: &Context) -> AppResult<(Vec<u8>, Vec<u8>)> {
    Ok((
        format!("#!/bin/sh\nexec {} \"$@\"\n", quote(&c.bridge())?).into_bytes(),
        format!(
            "#!/bin/sh\nexec {} {FLAG} \"$@\"\n",
            quote(&c.manager_executable)?
        )
        .into_bytes(),
    ))
}
fn path_block(c: &Context) -> AppResult<String> {
    let bin = quote(c.command().parent().unwrap())?;
    let own = quote(&c.command())?;
    let bridge = quote(&c.bridge())?;
    Ok(format!(
        r#"{BEGIN}
case "$(command -v ocx 2>/dev/null)" in
  ''|{own}|{bridge})
    case ":$PATH:" in
      *:{bin}:*) ;;
      *) export PATH={bin}:"$PATH" ;;
    esac ;;
  *) printf '%s\n' 'OpenCodeX Desktop: another ocx command is already active; PATH unchanged.' >&2 ;;
esac
{END}
"#
    ))
}
fn profile_paths(c: &Context, shell: &str) -> AppResult<Vec<PathBuf>> {
    match Path::new(shell).file_name().and_then(|s| s.to_str()) {
        Some("zsh") => Ok(vec![c.user_profile.join(".zshrc")]),
        Some("bash") => {
            let login = [".bash_profile", ".bash_login", ".profile"]
                .into_iter()
                .find(|name| c.user_profile.join(name).exists())
                .unwrap_or(".bash_profile");
            Ok(vec![
                c.user_profile.join(login),
                c.user_profile.join(".bashrc"),
            ])
        }
        _ => Err(fail(
            "当前终端Shell尚不支持自动配置PATH，可使用应用入口的完整路径",
        )),
    }
}
fn target(c: &Context, t: &Target) -> AppResult<(PathBuf, PathBuf, u32)> {
    Ok(match t {
        Target::Command => (c.user_profile.clone(), c.command(), 0o755),
        Target::Bridge => (c.program_root.clone(), c.bridge(), 0o755),
        Target::Record => (c.desktop_root.clone(), c.record(), 0o600),
        Target::Profile(name)
            if [
                ".zshrc",
                ".bashrc",
                ".bash_profile",
                ".bash_login",
                ".profile",
            ]
            .contains(&name.as_str()) =>
        {
            (c.user_profile.clone(), c.user_profile.join(name), 0o600)
        }
        _ => return Err(fail("记录中包含不属于本次操作的终端文件")),
    })
}
fn read(root: &Path, path: &Path) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    let file = match safety::open_file(path, false) {
        Ok(file) => file,
        Err(_) if matches!(fs::symlink_metadata(path),Err(e) if e.kind()==std::io::ErrorKind::NotFound) => {
            return Ok(None)
        }
        Err(e) => return Err(e),
    };
    safety::require_single_link(&file)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let m = file.metadata().map_err(io)?;
        if m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o022 != 0 {
            return Err(fail("终端文件归属或写入权限异常，未接管"));
        }
    }
    let mut bytes = Vec::new();
    file.take(MAX + 1).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() as u64 > MAX {
        return Err(fail("终端文件超过读取上限"));
    }
    Ok(Some(bytes))
}
fn conflicts(c: &Context, search: &[PathBuf]) -> AppResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for dir in search {
        if !dir.is_absolute() {
            return Err(fail("PATH含有相对目录，无法确定已有命令"));
        }
        let candidate = dir.join("ocx");
        if candidate == c.command() || candidate == c.bridge() {
            continue;
        }
        match fs::symlink_metadata(&candidate) {
            Ok(_) => {
                if !paths.contains(&candidate) {
                    paths.push(candidate)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(fail("无法核对PATH中的已有ocx命令")),
        }
    }
    Ok(paths)
}
fn installed(c: &Context) -> AppResult<Option<Installed>> {
    read(&c.desktop_root, &c.record())?
        .map(|bytes| {
            let value: Installed = decode(&bytes)?;
            validate_installed(c, &value)?;
            Ok(value)
        })
        .transpose()
}
fn validate_installed(c: &Context, value: &Installed) -> AppResult<()> {
    if value.schema_version != 1
        || value.context.user_profile != c.user_profile
        || value.context.desktop_root != c.desktop_root
        || value.context.program_root != c.program_root
        || value.profiles.is_empty()
        || value.profiles.len() > 2
    {
        return Err(fail("已有命令登记不属于当前应用，未覆盖"));
    }
    let expected_block = path_block(&value.context)?;
    let mut seen = std::collections::BTreeSet::new();
    for entry in &value.profiles {
        let name = entry
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| fail("终端配置路径无效"))?;
        let (_, expected, _) = target(c, &Target::Profile(name.into()))?;
        if entry.path != expected
            || !seen.insert(name.to_string())
            || !(entry.block == expected_block || entry.block == format!("\n{expected_block}"))
        {
            return Err(fail("终端配置登记无效"));
        }
    }
    Ok(())
}
fn remove_profile_block(text: &str, entry: &ProfileEdit) -> AppResult<Option<Vec<u8>>> {
    if text.matches(&entry.block).count() != 1 {
        return Err(fail("Shell中的原PATH登记被修改，未覆盖"));
    }
    let offset = text.find(&entry.block).unwrap();
    let suffix = &text[offset + entry.block.len()..];
    // Keep a separator if the user appended content after our block. Removing
    // it would join their next command onto an originally unterminated line.
    let separator = if entry.block.starts_with('\n') && !suffix.is_empty() {
        "\n"
    } else {
        ""
    };
    let remaining = format!("{}{}{}", &text[..offset], separator, suffix);
    Ok(if remaining.is_empty() && entry.was_absent {
        None
    } else {
        Some(remaining.into_bytes())
    })
}
fn make_plan(
    c: &Context,
    action: Action,
    shell: &str,
    search: &[PathBuf],
) -> AppResult<(Transaction, Vec<PathBuf>, Option<String>, bool)> {
    c.validate()?;
    let existing = installed(c)?;
    let enabled = existing.is_some();
    if action == Action::Disable && !enabled {
        return Err(fail("当前没有本应用已启用的终端入口"));
    }
    let (command, bridge) = scripts(c)?;
    let command_before = read(&c.user_profile, &c.command())?;
    let bridge_before = read(&c.program_root, &c.bridge())?;
    if let Some(installed) = &existing {
        let (original_command, original_bridge) = scripts(&installed.context)?;
        if command_before.as_deref() != Some(original_command.as_slice())
            || bridge_before.as_deref() != Some(original_bridge.as_slice())
        {
            return Err(fail("登记的命令入口已被修改，未覆盖或删除"));
        }
        if action == Action::Enable && installed.context != *c {
            return Err(fail("命令配置目录已变化，请先停用原入口再启用"));
        }
    } else if command_before.is_some() || bridge_before.is_some() {
        return Err(fail("目标位置已有ocx命令，未覆盖"));
    }
    let conflicts = if action == Action::Enable {
        conflicts(c, search)?
    } else {
        vec![]
    };
    let version = if action == Action::Enable {
        let (target, platform) = program_layout::host_target()?;
        Some(
            program_selection::read(&c.program_root, platform, target)?
                .ok_or_else(|| fail("运行程序尚未就绪"))?
                .selection
                .version,
        )
    } else {
        None
    };
    let profiles = match &existing {
        Some(i) => i.profiles.iter().map(|p| p.path.clone()).collect(),
        None => profile_paths(c, shell)?,
    };
    let mut changes = vec![
        Change {
            target: Target::Command,
            before: command_before,
            after: (action == Action::Enable).then_some(command),
        },
        Change {
            target: Target::Bridge,
            before: bridge_before,
            after: (action == Action::Enable).then_some(bridge),
        },
    ];
    let mut edits = vec![];
    for path in profiles {
        let before = read(&c.user_profile, &path)?;
        if before.as_ref().is_some_and(|b| b.len() > MAX_PROFILE) {
            return Err(fail("Shell配置超过修改上限，未修改"));
        }
        let text = std::str::from_utf8(before.as_deref().unwrap_or_default())
            .map_err(|_| fail("Shell配置不是UTF-8，未修改"))?;
        let block = if let Some(entry) = existing
            .as_ref()
            .and_then(|i| i.profiles.iter().find(|p| p.path == path))
        {
            if text.matches(&entry.block).count() != 1 {
                return Err(fail("Shell中的原PATH登记被修改，未覆盖"));
            }
            entry.block.clone()
        } else {
            if text.contains(BEGIN) || text.contains(END) {
                return Err(fail("Shell中存在未登记的同名PATH区域，未接管"));
            }
            format!(
                "{}{}",
                if text.is_empty() || text.ends_with('\n') {
                    ""
                } else {
                    "\n"
                },
                path_block(c)?
            )
        };
        let after = match action {
            Action::Enable => {
                if existing.is_some() {
                    before.clone()
                } else {
                    Some(format!("{text}{block}").into_bytes())
                }
            }
            Action::Disable => {
                if let Some(entry) = existing
                    .as_ref()
                    .and_then(|i| i.profiles.iter().find(|p| p.path == path))
                {
                    remove_profile_block(text, entry)?
                } else {
                    before.clone()
                }
            }
        };
        edits.push(ProfileEdit {
            path: path.clone(),
            block,
            was_absent: existing
                .as_ref()
                .and_then(|i| i.profiles.iter().find(|p| p.path == path))
                .map_or(before.is_none(), |p| p.was_absent),
        });
        changes.push(Change {
            target: Target::Profile(path.file_name().unwrap().to_string_lossy().into_owned()),
            before,
            after,
        });
    }
    let record_after = if action == Action::Enable {
        Some(encode(&Installed {
            schema_version: 1,
            context: c.clone(),
            profiles: edits,
        })?)
    } else {
        None
    };
    changes.push(Change {
        target: Target::Record,
        before: read(&c.desktop_root, &c.record())?,
        after: record_after,
    });
    Ok((
        Transaction {
            schema_version: 1,
            context: c.clone(),
            action,
            changes,
        },
        conflicts,
        version,
        enabled,
    ))
}
fn validate_transaction(c: &Context, t: &Transaction) -> AppResult<()> {
    if t.schema_version != 1
        || t.context != *c
        || !(4..=5).contains(&t.changes.len())
        || !matches!(t.changes[0].target, Target::Command)
        || !matches!(t.changes[1].target, Target::Bridge)
        || !matches!(t.changes.last().unwrap().target, Target::Record)
    {
        return Err(fail("未完成终端操作不属于当前配置"));
    }
    let record = t.changes.last().unwrap();
    let before = record
        .before
        .as_deref()
        .map(decode::<Installed>)
        .transpose()?;
    let after = record
        .after
        .as_deref()
        .map(decode::<Installed>)
        .transpose()?;
    for value in before.iter().chain(after.iter()) {
        validate_installed(c, value)?;
    }
    let owner = match t.action {
        Action::Enable => {
            let owner = after.as_ref().ok_or_else(|| fail("终端启用记录缺少后像"))?;
            if owner.context != *c {
                return Err(fail("终端启用记录绑定无效"));
            }
            if let Some(old) = &before {
                if encode(old)? != encode(owner)? {
                    return Err(fail("不能用继续操作替换既有命令登记"));
                }
            }
            owner
        }
        Action::Disable => {
            if after.is_some() {
                return Err(fail("停用事务不能新建登记"));
            }
            before.as_ref().ok_or_else(|| fail("停用事务缺少原登记"))?
        }
    };
    let (command, bridge) = scripts(&owner.context)?;
    for (change, script) in t.changes[..2].iter().zip([command, bridge]) {
        let expected_before = before.as_ref().map(|_| script.as_slice());
        let expected_after = (t.action == Action::Enable).then_some(script.as_slice());
        if change.before.as_deref() != expected_before || change.after.as_deref() != expected_after
        {
            return Err(fail("终端事务中的命令文本不是本应用登记内容"));
        }
    }
    if owner.profiles.len() + 3 != t.changes.len() {
        return Err(fail("终端事务目标数量与登记不符"));
    }
    for (change, entry) in t.changes[2..t.changes.len() - 1]
        .iter()
        .zip(&owner.profiles)
    {
        let (_, path, _) = target(c, &change.target)?;
        if !matches!(change.target, Target::Profile(_)) || path != entry.path {
            return Err(fail("终端事务包含错误的Shell目标"));
        }
        if change
            .before
            .as_ref()
            .is_some_and(|v| v.len() > MAX_PROFILE)
            || change
                .after
                .as_ref()
                .is_some_and(|v| v.len() > MAX_PROFILE + 4096)
        {
            return Err(fail("终端事务Shell内容超过上限"));
        }
        let text = std::str::from_utf8(change.before.as_deref().unwrap_or_default())
            .map_err(|_| fail("Shell原件无法解码"))?;
        let expected = if t.action == Action::Disable {
            remove_profile_block(text, entry)?
        } else if before.is_some() {
            if text.matches(&entry.block).count() != 1 {
                return Err(fail("已启用PATH区域无法核对"));
            }
            change.before.clone()
        } else {
            let block = format!(
                "{}{}",
                if text.is_empty() || text.ends_with('\n') {
                    ""
                } else {
                    "\n"
                },
                path_block(c)?
            );
            if text.contains(BEGIN)
                || text.contains(END)
                || entry.was_absent != change.before.is_none()
                || entry.block != block
            {
                return Err(fail("终端事务的PATH修改不属于原预览"));
            }
            Some(format!("{text}{block}").into_bytes())
        };
        if expected != change.after {
            return Err(fail("终端事务Shell后像不等于本次标记增删"));
        }
    }
    Ok(())
}
fn pending(c: &Context) -> AppResult<Option<Transaction>> {
    read(&c.desktop_root, &c.pending())?
        .map(|bytes| {
            let t: Transaction = decode(&bytes)?;
            validate_transaction(c, &t)?;
            Ok(t)
        })
        .transpose()
}
fn plan(
    c: &Context,
    action: Action,
    shell: &str,
    search: &[PathBuf],
) -> AppResult<(Transaction, Preview)> {
    if !cfg!(target_os = "macos") {
        return Err(fail("当前平台终端注册尚未接入"));
    }
    {
        c.validate()?;
        let (transaction, conflicts, version, enabled, has_pending) = if let Some(t) = pending(c)? {
            if t.action != action {
                return Err(fail("请先完成原终端操作，再执行其它动作"));
            }
            (
                t,
                if action == Action::Enable {
                    conflicts(c, search)?
                } else {
                    vec![]
                },
                None,
                false,
                true,
            )
        } else {
            let (t, conflicts, version, enabled) = make_plan(c, action, shell, search)?;
            (t, conflicts, version, enabled, false)
        };
        validate_transaction(c, &transaction)?;
        let mut observed = Vec::new();
        for change in &transaction.changes {
            let (root, path, _) = target(c, &change.target)?;
            let bytes = read(&root, &path)?;
            if bytes != change.before && bytes != change.after {
                return Err(fail("终端文件在原操作之后被修改，未覆盖"));
            }
            observed.push(bytes);
        }
        let selection = if action == Action::Enable {
            let (target, platform) = program_layout::host_target()?;
            Some(
                program_selection::read(&c.program_root, platform, target)?
                    .ok_or_else(|| fail("当前运行程序尚未就绪"))?
                    .selection,
            )
        } else {
            None
        };
        let version = selection.as_ref().map(|s| s.version.clone()).or(version);
        let confirmation_digest =
            hash::sha256_hex(&encode(&(&transaction, &observed, &conflicts, &selection))?);
        let profile_paths = transaction
            .changes
            .iter()
            .filter_map(|c| {
                if matches!(c.target, Target::Profile(_)) {
                    Some(target(&transaction.context, &c.target).map(|(_, p, _)| p))
                } else {
                    None
                }
            })
            .collect::<AppResult<Vec<_>>>()?;
        let preview = Preview {
            action,
            enabled,
            pending: has_pending,
            version,
            command_path: c.command(),
            runtime_entry: c.bridge(),
            opencodex_home: c.opencodex_home.clone(),
            profile_paths,
            conflicts,
            confirmation_digest,
        };
        Ok((transaction, preview))
    }
}
pub fn preview(c: &Context, action: Action, shell: &str, search: &[PathBuf]) -> AppResult<Preview> {
    plan(c, action, shell, search).map(|(_, p)| p)
}
pub fn inspect(c: &Context, shell: &str, search: &[PathBuf]) -> AppResult<Preview> {
    c.validate()?;
    let action = if let Some(t) = pending(c)? {
        t.action
    } else if installed(c)?.is_some() {
        Action::Disable
    } else {
        Action::Enable
    };
    preview(c, action, shell, search)
}
/// Read-only owned-terminal part of an uninstall review. No registration is
/// different from a foreign command at the reserved path; foreign bytes refuse.
pub fn removal_preview(c: &Context, shell: &str, search: &[PathBuf]) -> AppResult<Option<Preview>> {
    c.validate()?;
    if let Some(t) = pending(c)? {
        if t.action != Action::Disable {
            return Err(fail("请先完成原终端启用操作，再卸载程序"));
        }
        return preview(c, Action::Disable, shell, search).map(Some);
    }
    if installed(c)?.is_some() {
        return preview(c, Action::Disable, shell, search).map(Some);
    }
    if read(&c.user_profile, &c.command())?.is_some()
        || read(&c.program_root, &c.bridge())?.is_some()
    {
        return Err(fail("保留位置存在未登记的终端入口，未卸载程序"));
    }
    Ok(None)
}
pub fn revoke_for_removal(
    c: &Context,
    shell: &str,
    search: &[PathBuf],
    expected: Option<&str>,
) -> AppResult<()> {
    match removal_preview(c, shell, search)? {
        Some(preview) => {
            if Some(preview.confirmation_digest.as_str()) != expected {
                return Err(fail("终端撤销预览变化，请重新核对卸载范围"));
            }
            apply(
                c,
                Action::Disable,
                shell,
                search,
                &preview.confirmation_digest,
            )
            .map(|_| ())
        }
        None if expected.is_none() => Ok(()),
        None => Err(fail("原终端登记在卸载预览后变化，请重新核对")),
    }
}
/// The entry belongs to its explicit D/H. A root switch must not orphan an
/// enabled command or move a half-written shell transaction to another root.
pub(crate) fn ensure_binding_change_allowed(desktop_root: &Path) -> AppResult<()> {
    for name in [RECORD, PENDING] {
        let path = desktop_root.join(name);
        safety::check_path(desktop_root, &path, true)?;
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            _ => return Err(fail("请先停用本应用的终端 ocx 命令，再更改配置目录")),
        }
    }
    Ok(())
}
pub fn apply(
    c: &Context,
    action: Action,
    shell: &str,
    search: &[PathBuf],
    expected: &str,
) -> AppResult<Outcome> {
    let lock = c.desktop_root.join("manager-state/terminal-cli");
    safety::check_path(
        &c.desktop_root,
        &crate::infrastructure::locking::lock_path_for(&lock),
        true,
    )?;
    let _lock = TargetFileLock::lock(&lock)?;
    let (transaction, preview) = plan(c, action, shell, search)?;
    if preview.confirmation_digest != expected || !preview.conflicts.is_empty() {
        return Err(fail("终端入口预览已变化或已有同名命令，请重新核对"));
    }
    let bytes = encode(&transaction)?;
    if !preview.pending {
        let mut file = platform::create_new_private_file(&c.pending(), 0o600).map_err(io)?;
        file.write_all(&bytes).map_err(io)?;
        file.sync_all().map_err(io)?;
        safety::sync_directories(&c.desktop_root, c.pending().parent().unwrap())?;
    }
    for change in &transaction.changes {
        c.validate()?;
        if read(&c.desktop_root, &c.pending())?.as_deref() != Some(bytes.as_slice()) {
            return Err(fail("终端事务在执行期间变化"));
        }
        let (root, path, mode) = target(c, &change.target)?;
        let current = read(&root, &path)?;
        if current != change.before && current != change.after {
            return Err(fail("终端文件被外部修改，原操作保留"));
        }
        if current != change.after {
            if let Some(after) = &change.after {
                #[cfg(unix)]
                let mode = if matches!(change.target, Target::Profile(_)) && current.is_some() {
                    use std::os::unix::fs::PermissionsExt;
                    fs::metadata(&path).map_err(io)?.permissions().mode() & 0o777
                } else {
                    mode
                };
                atomic_write(&path, after, mode)?;
            } else {
                fs::remove_file(&path).map_err(io)?;
            }
        }
        if read(&root, &path)? != change.after {
            return Err(fail("终端入口写后读回不符"));
        }
        safety::sync_directories(&root, path.parent().unwrap())?;
    }
    for change in &transaction.changes {
        let (root, path, _) = target(c, &change.target)?;
        if read(&root, &path)? != change.after {
            return Err(fail("终端操作最终读回发生变化，记录保留"));
        }
    }
    if read(&c.desktop_root, &c.pending())?.as_deref() != Some(bytes.as_slice()) {
        return Err(fail("终端操作最终记录已变化"));
    }
    fs::remove_file(c.pending()).map_err(io)?;
    safety::sync_directories(&c.desktop_root, c.pending().parent().unwrap())?;
    Ok(Outcome {
        action,
        enabled: action == Action::Enable,
        command_path: c.command(),
        opencodex_home: c.opencodex_home.clone(),
    })
}

/// Called before Tauri initializes, so running ocx never opens a second manager.
pub fn dispatch() -> bool {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(FLAG)) {
        return false;
    }
    let result = forward(args.collect());
    if let Err(error) = result {
        eprintln!("无法运行本应用的 ocx：{error}");
        std::process::exit(1);
    }
    true
}
fn forward(args: Vec<OsString>) -> AppResult<()> {
    let profile = platform::home_dir().ok_or_else(|| fail("当前用户目录不可确认"))?;
    let executable = std::env::current_exe().map_err(io)?;
    let config = data_root::startup_v2::selected_config(&profile.join(".codex-desktop/desktop"))?;
    let (_, platform) = program_layout::host_target()?;
    let c = Context {
        user_profile: profile.clone(),
        desktop_root: config.active_data_root.clone(),
        opencodex_home: data_root::resolve_opencodex_home(&config),
        program_root: program_layout::ProgramLocations::resolve(&executable, &profile, platform)?
            .program_root,
        manager_executable: executable,
    };
    let mut process = forwarding_command(&c, args)?;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(io(process.exec()))
    }
    #[cfg(not(unix))]
    {
        let status = process.status().map_err(io)?;
        std::process::exit(status.code().unwrap_or(1))
    }
}

fn forwarding_command(c: &Context, args: Vec<OsString>) -> AppResult<Command> {
    c.validate()?;
    if pending(c)?.is_some() {
        return Err(fail("终端入口操作尚未完成，请在应用中重新核对"));
    }
    let record = installed(c)?.ok_or_else(|| fail("终端入口尚未启用"))?;
    if record.context != *c {
        return Err(fail("终端入口绑定已变化，请在应用中重新启用"));
    }
    let (command, bridge) = scripts(c)?;
    if read(&c.user_profile, &c.command())?.as_deref() != Some(command.as_slice())
        || read(&c.program_root, &c.bridge())?.as_deref() != Some(bridge.as_slice())
    {
        return Err(fail("终端入口与登记不一致"));
    }
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read(&c.program_root, platform, target)?
        .ok_or_else(|| fail("当前运行程序未就绪"))?;
    let mut process = Command::new(&selected.paths.node);
    process.arg(&selected.script).args(args);
    process
        .env("OPENCODEX_HOME", &c.opencodex_home)
        .env("OCX_NO_DESKTOP_HANDOFF", "1");
    // HOME/USERPROFILE retain the calling OS user's profile. This launcher only
    // selects the explicit OpenCodex data home and version; no secret logging.
    Ok(process)
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "terminal_cli_tests.rs"]
pub(crate) mod tests;
