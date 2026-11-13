//! Windows branch of the same terminal command, registration and pending action.
//! Registry I/O is native in production; isolated tests supply memory facts.
use super::terminal_cli_path as policy;
use super::*;
use policy::{PathEdit, PathValue};
const SCHEMA: u32 = 2;
struct Admitted {
    program: program_selection::SelectedProgram,
    console_sha256: String,
}
impl Admitted {
    fn check(&self, c: &Context) -> AppResult<()> {
        program_selection::verify_current(&c.program_root, &self.program)?;
        safety::check_path(&c.program_root, &console(c), false)?;
        let file = safety::open_file(&console(c), false)?;
        safety::require_single_link(&file)?;
        if hash::sha256_file(&console(c)).map_err(io)? != self.console_sha256 {
            return Err(fail("控制台转交程序在操作期间变化"));
        }
        Ok(())
    }
}
const SCRIPT:&[u8]=b"@echo off\r\nsetlocal DisableDelayedExpansion\r\n\"%~dp0..\\..\\ocxd.exe\" --ocxd-runtime-cli %*\r\nexit /b %errorlevel%\r\n";
fn command(c: &Context) -> PathBuf {
    c.program_root.join("runtime/bin/ocx.cmd")
}
fn console(c: &Context) -> PathBuf {
    c.program_root.join("ocxd.exe")
}
fn directory(c: &Context) -> AppResult<String> {
    command(c)
        .parent()
        .and_then(Path::to_str)
        .map(str::to_owned)
        .ok_or_else(|| fail("命令目录无法表示为 PATH"))
}
/// Never-opened installation: observe absence without creating a data root,
/// registry key, lock or default binding. Existing/unknown objects refuse.
pub(super) fn pristine_installer(profile: &Path, program_root: &Path) -> AppResult<bool> {
    let anchor = profile.join(".codex-desktop/desktop");
    let command = program_root.join("runtime/bin/ocx.cmd");
    safety::check_path(profile, &anchor, true)?;
    safety::check_path(program_root, &command, true)?;
    let absent = |path: &Path| match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(io(e)),
        Ok(_) => Ok(false),
    };
    Ok(absent(&anchor)? && absent(&command)?)
}
trait Registry {
    fn read(&self) -> AppResult<Option<PathValue>>;
    fn replace(&self, before: &Option<PathValue>, after: &Option<PathValue>) -> AppResult<()>;
    fn notify(&self) -> AppResult<()>;
}
#[cfg(windows)]
struct Native;
#[cfg(windows)]
impl Registry for Native {
    fn read(&self) -> AppResult<Option<PathValue>> {
        crate::infrastructure::windows_cli024::read_user_path()
            .map(|v| {
                v.map(|v| PathValue {
                    value_type: v.value_type,
                    bytes: v.bytes,
                })
            })
            .map_err(|_| fail("无法核对当前用户 PATH"))
    }
    fn replace(&self, before: &Option<PathValue>, after: &Option<PathValue>) -> AppResult<()> {
        let native = |v: &Option<PathValue>| {
            v.as_ref()
                .map(|v| crate::infrastructure::windows_cli024::RegistryValue {
                    value_type: v.value_type,
                    bytes: v.bytes.clone(),
                })
        };
        crate::infrastructure::windows_cli024::replace_user_path(&native(before), &native(after))
            .map_err(|_| fail("PATH 写入或读回未确认，原操作保留；不会覆盖外部变化"))
    }
    fn notify(&self) -> AppResult<()> {
        crate::infrastructure::windows_cli024::notify_environment_changed()
            .map_err(|_| fail("环境通知未确认，请重新核对并继续原操作"))
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Installed {
    schema_version: u32,
    context: Context,
    path_edit: PathEdit,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Change {
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Transaction {
    schema_version: u32,
    context: Context,
    action: Action,
    command: Change,
    path_before: Option<PathValue>,
    path_after: Option<PathValue>,
    record: Change,
}
fn private_read(root: &Path, path: &Path) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    #[cfg(windows)]
    {
        use crate::infrastructure::windows_official_runtime024::PrivateRecord;
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io(e)),
            Ok(_) => {}
        }
        PrivateRecord::open(path, MAX as usize)
            .map_err(io)?
            .read()
            .map(Some)
            .map_err(io)
    }
    #[cfg(not(windows))]
    {
        read(root, path)
    }
}
fn validate_record(c: &Context, v: &Installed) -> AppResult<()> {
    if v.schema_version != SCHEMA || v.context != *c {
        return Err(fail("Windows 终端登记不属于当前应用与配置目录"));
    }
    policy::validate(&v.path_edit, &directory(c)?)
}
fn installed(c: &Context) -> AppResult<Option<Installed>> {
    private_read(&c.desktop_root, &c.record())?
        .map(|raw| {
            let v: Installed = decode(&raw)?;
            validate_record(c, &v)?;
            Ok(v)
        })
        .transpose()
}
fn validate_transaction(c: &Context, t: &Transaction) -> AppResult<()> {
    if t.schema_version != SCHEMA || t.context != *c {
        return Err(fail("Windows 终端事务来源变化"));
    }
    for v in [&t.path_before, &t.path_after].into_iter().flatten() {
        v.text()?;
    }
    let before = t
        .record
        .before
        .as_deref()
        .map(decode::<Installed>)
        .transpose()?;
    let after = t
        .record
        .after
        .as_deref()
        .map(decode::<Installed>)
        .transpose()?;
    for r in before.iter().chain(after.iter()) {
        validate_record(c, r)?;
    }
    if t.command.before != before.as_ref().map(|_| SCRIPT.to_vec())
        || t.command.after != (t.action == Action::Enable).then_some(SCRIPT.to_vec())
    {
        return Err(fail("终端事务命令文本不是本应用入口"));
    }
    match t.action {
        Action::Enable => {
            let after = after.ok_or_else(|| fail("终端启用缺少后像"))?;
            if let Some(before) = before {
                if encode(&before)? != encode(&after)? || t.path_before != t.path_after {
                    return Err(fail("已登记入口不能被重新绑定"));
                }
            } else {
                let expected = policy::enable(&t.path_before, &directory(c)?)?;
                if expected != after.path_edit || expected.after != t.path_after {
                    return Err(fail("PATH 启用后像不能从原件重建"));
                }
            }
        }
        Action::Disable => {
            let before = before.ok_or_else(|| fail("终端停用缺少原登记"))?;
            if after.is_some()
                || policy::disable(&before.path_edit, &t.path_before, &directory(c)?)?
                    != t.path_after
            {
                return Err(fail("PATH 停用不是原登记的精确撤销"));
            }
        }
    }
    Ok(())
}
fn pending(c: &Context) -> AppResult<Option<Transaction>> {
    private_read(&c.desktop_root, &c.pending())?
        .map(|raw| {
            let t: Transaction = decode(&raw)?;
            validate_transaction(c, &t)?;
            Ok(t)
        })
        .transpose()
}
fn validate_context(c: &Context) -> AppResult<()> {
    c.validate()?;
    #[cfg(windows)]
    {
        use crate::infrastructure::windows_official_runtime024::PinnedDirectory;
        PinnedDirectory::open_private(&c.desktop_root)
            .map_err(io)?
            .recheck()
            .map_err(io)?;
        PinnedDirectory::open_private(&c.opencodex_home)
            .map_err(io)?
            .recheck()
            .map_err(io)?;
        if c.manager_executable != c.program_root.join("opencodex-desktop.exe") {
            return Err(fail("Desktop 入口位置未确认"));
        }
    }
    Ok(())
}
fn expand(text: &str, c: &Context) -> AppResult<String> {
    let (mut result, mut tail) = (String::new(), text);
    while let Some(at) = tail.find('%') {
        result.push_str(&tail[..at]);
        tail = &tail[at + 1..];
        let end = tail.find('%').ok_or_else(|| fail("PATH 环境变量不完整"))?;
        let name = &tail[..end];
        let value =
            if name.eq_ignore_ascii_case("USERPROFILE") || name.eq_ignore_ascii_case("HOME") {
                c.user_profile.to_str().map(str::to_owned)
            } else {
                std::env::vars_os().find_map(|(k, v)| {
                    k.to_str()
                        .filter(|k| k.eq_ignore_ascii_case(name))
                        .and_then(|_| v.into_string().ok())
                })
            }
            .ok_or_else(|| fail("PATH 环境变量无法确定"))?;
        if value.contains(['%', ';', '"']) || value.chars().any(char::is_control) {
            return Err(fail("PATH 环境变量展开不明确"));
        }
        result.push_str(&value);
        tail = &tail[end + 1..];
    }
    result.push_str(tail);
    Ok(result)
}
fn conflicts(
    c: &Context,
    current: &Option<PathValue>,
    search: &[PathBuf],
) -> AppResult<Vec<PathBuf>> {
    let mut paths = search.to_vec();
    if let Some(v) = current {
        let text = v.text()?;
        for part in policy::parts(&text)? {
            let part = policy::unquote(part)?;
            if part.is_empty() {
                continue;
            }
            let text = if v.value_type == 2 {
                expand(part, c)?
            } else {
                part.into()
            };
            let path = PathBuf::from(text);
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    let mut result = vec![];
    for dir in paths {
        if !dir.is_absolute() {
            return Err(fail("PATH 含相对目录，无法确定同名命令"));
        }
        for name in ["ocx", "ocx.com", "ocx.exe", "ocx.cmd", "ocx.bat", "ocx.ps1"] {
            let path = dir.join(name);
            if path == command(c) {
                continue;
            }
            match fs::symlink_metadata(&path) {
                Ok(_) => {
                    if !result.contains(&path) {
                        result.push(path)
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(fail("无法核对 PATH 中的 ocx 命令")),
            }
        }
    }
    Ok(result)
}
fn check_current(c: &Context, t: &Transaction, r: &impl Registry) -> AppResult<()> {
    validate_context(c)?;
    for (root, path, change) in [
        (&c.program_root, command(c), &t.command),
        (&c.desktop_root, c.record(), &t.record),
    ] {
        let now = private_read(root, &path)?;
        if now != change.before && now != change.after {
            return Err(fail("终端文件已被外部修改，原操作保留"));
        }
    }
    let now = r.read()?;
    if now != t.path_before && now != t.path_after {
        return Err(fail("用户 PATH 已被外部修改，原操作保留"));
    }
    Ok(())
}
fn plan(
    c: &Context,
    action: Action,
    search: &[PathBuf],
    r: &impl Registry,
) -> AppResult<(Transaction, Preview, Option<Admitted>)> {
    validate_context(c)?;
    let current = r.read()?;
    let pending = pending(c)?;
    let is_pending = pending.is_some();
    let t = if let Some(t) = pending {
        if t.action != action {
            return Err(fail("请先继续原终端操作"));
        }
        t
    } else {
        let old = installed(c)?;
        let command_before = private_read(&c.program_root, &command(c))?;
        if old.is_some() && command_before.as_deref() != Some(SCRIPT)
            || old.is_none() && command_before.is_some()
        {
            return Err(fail("终端命令已修改或存在外部文件，未接管"));
        }
        let (path_after, record_after) = match action {
            Action::Enable => {
                let edit = match &old {
                    Some(v) => v.path_edit.clone(),
                    None => policy::enable(&current, &directory(c)?)?,
                };
                let after = if old.is_some() {
                    current.clone()
                } else {
                    edit.after.clone()
                };
                (
                    after,
                    Some(encode(&Installed {
                        schema_version: SCHEMA,
                        context: c.clone(),
                        path_edit: edit,
                    })?),
                )
            }
            Action::Disable => {
                let old = old.ok_or_else(|| fail("尚未启用本应用终端入口"))?;
                (
                    policy::disable(&old.path_edit, &current, &directory(c)?)?,
                    None,
                )
            }
        };
        Transaction {
            schema_version: SCHEMA,
            context: c.clone(),
            action,
            command: Change {
                before: command_before,
                after: (action == Action::Enable).then_some(SCRIPT.to_vec()),
            },
            path_before: current.clone(),
            path_after,
            record: Change {
                before: private_read(&c.desktop_root, &c.record())?,
                after: record_after,
            },
        }
    };
    validate_transaction(c, &t)?;
    check_current(c, &t, r)?;
    let (mut version, mut selection, mut helper) = (None, None, None);
    let mut admitted = None;
    let found = if action == Action::Enable {
        safety::check_path(&c.program_root, &console(c), false)?;
        let file = safety::open_file(&console(c), false)?;
        safety::require_single_link(&file)?;
        helper = Some(hash::sha256_file(&console(c)).map_err(io)?);
        let (target, platform) = program_layout::host_target()?;
        let selected = program_selection::read_complete(&c.program_root, platform, target)?
            .ok_or_else(|| fail("运行程序尚未就绪"))?;
        version = Some(selected.selection.version.clone());
        selection = Some(selected.selection.clone());
        admitted = Some(Admitted {
            program: selected,
            console_sha256: helper.clone().unwrap(),
        });
        conflicts(c, &current, search)?
    } else {
        vec![]
    };
    let observed = (
        private_read(&c.program_root, &command(c))?,
        private_read(&c.desktop_root, &c.record())?,
        r.read()?,
    );
    check_current(c, &t, r)?;
    let digest = hash::sha256_hex(&encode(&(&t, &observed, &found, &selection, &helper))?);
    let preview = Preview {
        action,
        enabled: t.record.before.is_some(),
        pending: is_pending,
        version,
        command_path: command(c),
        runtime_entry: console(c),
        opencodex_home: c.opencodex_home.clone(),
        profile_paths: vec![],
        conflicts: found,
        confirmation_digest: digest,
    };
    Ok((t, preview, admitted))
}
fn mutate(c: &Context, root: &Path, path: &Path, change: &Change) -> AppResult<()> {
    let now = private_read(root, path)?;
    if now == change.after {
        return Ok(());
    }
    if now != change.before {
        return Err(fail("终端文件在写入前变化，未覆盖"));
    }
    if let Some(after) = &change.after {
        atomic_write(path, after, 0o600)?;
    } else {
        fs::remove_file(path).map_err(io)?;
    }
    if private_read(root, path)? != change.after {
        return Err(fail("终端文件写后读回不符"));
    }
    safety::sync_directories(root, path.parent().unwrap())?;
    c.validate()
}
fn apply_with(
    c: &Context,
    action: Action,
    search: &[PathBuf],
    expected: &str,
    r: &impl Registry,
    point: &impl Fn(&str) -> AppResult<()>,
) -> AppResult<Outcome> {
    let lock = c.desktop_root.join("manager-state/terminal-cli");
    safety::check_path(
        &c.desktop_root,
        &crate::infrastructure::locking::lock_path_for(&lock),
        true,
    )?;
    let _lock = TargetFileLock::lock(&lock)?;
    let (t, p, admitted) = plan(c, action, search, r)?;
    if p.confirmation_digest != expected || !p.conflicts.is_empty() {
        return Err(fail("终端预览已变化或已有同名命令"));
    }
    let bytes = encode(&t)?;
    if !p.pending {
        let mut file = platform::create_new_private_file(&c.pending(), 0o600).map_err(io)?;
        file.write_all(&bytes).map_err(io)?;
        file.sync_all().map_err(io)?;
        safety::sync_directories(&c.desktop_root, c.pending().parent().unwrap())?;
    }
    let check = || {
        if let Some(source) = &admitted {
            source.check(c)?;
        }
        check_current(c, &t, r)?;
        if private_read(&c.desktop_root, &c.pending())?.as_deref() != Some(bytes.as_slice()) {
            return Err(fail("终端原事务在执行期间变化"));
        }
        Ok(())
    };
    let path = || -> AppResult<()> {
        let now = r.read()?;
        if now != t.path_after {
            r.replace(&t.path_before, &t.path_after)?;
        }
        Ok(())
    };
    check()?;
    point("pending")?;
    if action == Action::Disable {
        path()?;
        point("path")?;
        check()?;
    }
    mutate(c, &c.program_root, &command(c), &t.command)?;
    point("command")?;
    check()?;
    if action == Action::Enable {
        path()?;
        point("path")?;
        check()?;
    }
    mutate(c, &c.desktop_root, &c.record(), &t.record)?;
    point("record")?;
    check()?;
    r.notify()?;
    point("notified")?;
    if let Some(source) = &admitted {
        super::super::program_image::verify(&source.program)?;
    }
    point("image-readback")?;
    check()?;
    if private_read(&c.program_root, &command(c))? != t.command.after
        || private_read(&c.desktop_root, &c.record())? != t.record.after
        || r.read()? != t.path_after
    {
        return Err(fail("终端最终后像变化，原事务保留"));
    }
    fs::remove_file(c.pending()).map_err(io)?;
    safety::sync_directories(&c.desktop_root, c.pending().parent().unwrap())?;
    Ok(Outcome {
        action,
        enabled: action == Action::Enable,
        command_path: command(c),
        opencodex_home: c.opencodex_home.clone(),
    })
}
fn inspect_with(c: &Context, search: &[PathBuf], r: &impl Registry) -> AppResult<Preview> {
    let action = if let Some(t) = pending(c)? {
        t.action
    } else if installed(c)?.is_some() {
        Action::Disable
    } else {
        Action::Enable
    };
    plan(c, action, search, r).map(|(_, p, _)| p)
}
fn removal_with(c: &Context, search: &[PathBuf], r: &impl Registry) -> AppResult<Option<Preview>> {
    validate_context(c)?;
    if let Some(t) = pending(c)? {
        if t.action != Action::Disable {
            return Err(fail("请先完成原终端启用操作"));
        }
        return plan(c, Action::Disable, search, r).map(|(_, p, _)| Some(p));
    }
    if installed(c)?.is_some() {
        return plan(c, Action::Disable, search, r).map(|(_, p, _)| Some(p));
    }
    if private_read(&c.program_root, &command(c))?.is_some() {
        return Err(fail("未登记的终端命令不能在卸载中删除"));
    }
    Ok(None)
}
pub(super) fn require_forwarding(c: &Context) -> AppResult<()> {
    validate_context(c)?;
    if pending(c)?.is_some()
        || installed(c)?.is_none()
        || private_read(&c.program_root, &command(c))?.as_deref() != Some(SCRIPT)
    {
        return Err(fail("终端入口尚未完成登记或已变化"));
    }
    Ok(())
}
#[cfg(windows)]
pub(super) fn preview(c: &Context, a: Action, s: &[PathBuf]) -> AppResult<Preview> {
    plan(c, a, s, &Native).map(|(_, p, _)| p)
}
#[cfg(windows)]
pub(super) fn inspect(c: &Context, s: &[PathBuf]) -> AppResult<Preview> {
    inspect_with(c, s, &Native)
}
#[cfg(windows)]
pub(super) fn apply(c: &Context, a: Action, s: &[PathBuf], e: &str) -> AppResult<Outcome> {
    apply_with(c, a, s, e, &Native, &|_| Ok(()))
}
#[cfg(windows)]
pub(super) fn removal_preview(c: &Context, s: &[PathBuf]) -> AppResult<Option<Preview>> {
    removal_with(c, s, &Native)
}
#[cfg(windows)]
pub(super) fn revoke_for_removal(c: &Context, expected: Option<&str>) -> AppResult<()> {
    match removal_with(c, &[], &Native)? {
        Some(p) => {
            if Some(p.confirmation_digest.as_str()) != expected {
                return Err(fail("终端撤销预览变化"));
            }
            apply(c, Action::Disable, &[], &p.confirmation_digest).map(|_| ())
        }
        None if expected.is_none() => Ok(()),
        None => Err(fail("终端原登记已变化")),
    }
}
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "terminal_cli_windows_tests.rs"]
mod tests;
