//! Durable Windows startup anchors and explicit recovery of a fresh unwritable root.
//! The per-user registry value is a startup pointer, never an automatic data
//! fallback. Business stores are created only on the next admitted startup.
use crate::errors::{AppError, AppResult};
use std::path::{Component, Path, PathBuf};

/// No WebView command may run with half-initialized business state.
pub(crate) fn defer_main(
    config: &mut tauri::utils::config::Config,
) -> Option<tauri::utils::config::WindowConfig> {
    let main = config
        .app
        .windows
        .iter_mut()
        .find(|window| window.label == "main")?;
    let original = main.clone();
    main.create = false;
    Some(original)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Choice {
    schema: u32,
    anchor: PathBuf,
}

fn failure(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "recover startup data root".into(),
        detail: detail.into(),
    }
}

fn check_path(path: &Path) -> AppResult<()> {
    if !path.is_absolute()
        || path.parent().is_none()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        || path
            .as_os_str()
            .to_string_lossy()
            .chars()
            .any(char::is_control)
    {
        return Err(failure("an absolute non-root data directory is required"));
    }
    #[cfg(windows)]
    if !matches!(path.components().next(), Some(Component::Prefix(prefix))
        if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
    {
        return Err(failure("a local Windows data directory is required"));
    }
    Ok(())
}

fn encode_choice(path: &Path) -> AppResult<String> {
    check_path(path)?;
    let encoded = serde_json::to_string(&Choice {
        schema: 1,
        anchor: path.to_path_buf(),
    })
    .map_err(|_| failure("startup choice cannot be encoded"))?;
    if encoded.len() > 32768 {
        return Err(failure("startup choice exceeds the size limit"));
    }
    Ok(encoded)
}

fn decode_choice(value: &str) -> AppResult<PathBuf> {
    if value.len() > 32768 {
        return Err(failure("startup choice exceeds the size limit"));
    }
    let choice: Choice =
        serde_json::from_str(value).map_err(|_| failure("startup choice is unreadable"))?;
    if choice.schema != 1 {
        return Err(failure("startup choice is newer than supported"));
    }
    check_path(&choice.anchor)?;
    Ok(choice.anchor)
}

/// Only a fresh, unbound default may recover without inspecting an old
/// transaction or altering an existing configuration. Unknown/read errors stop.
fn fresh_anchor(anchor: &Path) -> AppResult<bool> {
    match std::fs::symlink_metadata(anchor) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(_) => Err(failure("the existing root could not be inspected")),
        Ok(metadata) if metadata.is_dir() && !is_link(&metadata) => {
            let mut entries = std::fs::read_dir(anchor)
                .map_err(|_| failure("the existing root is unreadable"))?;
            match entries.next() {
                None => Ok(true),
                Some(Ok(_)) => Ok(false),
                Some(Err(_)) => Err(failure("the existing root inventory is unreadable")),
            }
        }
        Ok(_) => Ok(false),
    }
}

fn is_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

fn existing_directory(path: &Path) -> AppResult<PathBuf> {
    check_path(path)?;
    for ancestor in path.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor)
            .map_err(|_| failure("the selected directory could not be inspected"))?;
        if is_link(&metadata) {
            return Err(failure(
                "directory aliases are not admitted for startup recovery",
            ));
        }
    }
    let selected = path
        .canonicalize()
        .map_err(|_| failure("the selected directory does not exist"))?;
    if !selected.is_dir() || selected.parent().is_none() {
        return Err(failure("select an existing dedicated data directory"));
    }
    Ok(selected)
}

fn prospective_location(path: &Path) -> AppResult<PathBuf> {
    check_path(path)?;
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| failure("invalid root path"))?,
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| failure("invalid root ancestor"))?;
            }
            Err(_) => return Err(failure("the original root could not be inspected")),
        }
    }
    let mut resolved = ancestor
        .canonicalize()
        .map_err(|_| failure("invalid root ancestor"))?;
    for part in missing.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn validate_contents(selected: &Path, allow_empty: bool) -> AppResult<()> {
    let mut entries =
        std::fs::read_dir(selected).map_err(|_| failure("the selected directory is unreadable"))?;
    if entries
        .next()
        .transpose()
        .map_err(|_| failure("directory inventory failed"))?
        .is_none()
    {
        return if allow_empty {
            Ok(())
        } else {
            Err(failure("the saved data root is empty"))
        };
    }
    // Only admit bounded, current metadata. Never repair a malformed/future
    // root or follow a partition/metadata alias while selecting a new anchor.
    for relative in crate::modules::data_root::PARTITIONS
        .iter()
        .map(|(_, name)| *name)
        .chain(["manager-state/data-root.json", "manager-state/app.lock"])
    {
        match std::fs::symlink_metadata(selected.join(relative)) {
            Ok(metadata) if is_link(&metadata) => {
                return Err(failure("data root aliases are not admitted"))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(failure("data root entries could not be inspected")),
        }
    }
    crate::modules::data_root::load_runtime_config(selected)?;
    Ok(())
}

fn validate_selection(path: &Path, failed_anchor: &Path) -> AppResult<PathBuf> {
    let selected = existing_directory(path)?;
    let failed_anchor = prospective_location(failed_anchor)?;
    // Do not initialize a containing source or retry the same failed directory.
    if selected.starts_with(&failed_anchor) || failed_anchor.starts_with(&selected) {
        return Err(failure("select a separate dedicated data directory"));
    }
    validate_contents(&selected, true)?;
    Ok(selected)
}

fn validate_saved(path: &Path) -> AppResult<PathBuf> {
    let selected = existing_directory(path)?;
    validate_contents(&selected, false)?;
    Ok(selected)
}

fn recovery_allowed(error: &AppError, configured: bool, fresh: bool) -> bool {
    !configured && fresh && matches!(error, AppError::DataRootWriteDenied { .. })
}

enum InitialAdmission {
    Ready(crate::modules::data_root::bootstrap::StartupResolution),
    Choose(PathBuf),
}

/// The exact production admission path is testable without native dialogs or
/// the real HKCU pointer. Invalid saved choices cannot initialize a default.
fn admit_initial(
    platform_root: &Path,
    executable: &Path,
    configured: Option<&Path>,
) -> AppResult<InitialAdmission> {
    use crate::modules::data_root::bootstrap::{self, StartupResolution};
    let anchor = match configured {
        Some(path) => validate_saved(path)?,
        None => bootstrap::default_anchor(platform_root, executable, true)?,
    };
    let fresh = configured.is_none() && fresh_anchor(&anchor)?;
    match bootstrap::resolve_locked_with_boundary(&anchor, None) {
        Ok((runtime_config, locks)) => Ok(InitialAdmission::Ready(StartupResolution {
            anchor,
            sandbox_boundary: None,
            runtime_config,
            locks,
        })),
        Err(error) if recovery_allowed(&error, configured.is_some(), fresh) => {
            Ok(InitialAdmission::Choose(anchor))
        }
        Err(error) => Err(error),
    }
}

/// Remember the first admitted anchor before business services can start.
/// The executable may move during a later installation; it must not silently
/// choose a new empty data directory. A malformed/unreadable pointer stops
/// before admission, and an existing pointer is never rewritten here.
fn admit_registered(
    platform_root: &Path,
    executable: &Path,
    read: impl FnOnce() -> AppResult<Option<PathBuf>>,
    persist: impl FnOnce(&Path) -> AppResult<()>,
) -> AppResult<InitialAdmission> {
    let configured = read()?;
    let admission = admit_initial(platform_root, executable, configured.as_deref())?;
    if let (None, InitialAdmission::Ready(resolution)) = (configured.as_ref(), &admission) {
        // The anchor is already initialized and exclusively locked. Store
        // its canonical identity so later launches do not depend on the
        // old spelling or executable directory. The active root may be a
        // different binding; that binding still belongs to this anchor.
        let anchor = resolution
            .anchor
            .canonicalize()
            .map_err(|_| failure("the admitted startup anchor could not be resolved"))?;
        persist(&anchor)?;
    }
    Ok(admission)
}

/// Keep admission/selection/commit ordering independently testable without
/// invoking native UI, modifying the registry, or changing process privileges.
fn commit_selected<T>(
    selected: Option<PathBuf>,
    failed_anchor: &Path,
    admit: impl FnOnce(&Path) -> AppResult<T>,
    persist: impl FnOnce(&Path) -> AppResult<()>,
) -> AppResult<Option<T>> {
    let Some(path) = selected else {
        return Ok(None);
    };
    let path = validate_selection(&path, failed_anchor)?;
    let retained = admit(&path)?;
    persist(&path)?;
    Ok(Some(retained))
}

#[cfg(windows)]
mod native {
    use super::*;
    use crate::modules::data_root::bootstrap::{self, StartupResolution};
    use sha2::{Digest, Sha256};
    use tauri::{Manager, Runtime};
    use tauri_plugin_dialog::{DialogExt, FilePath, MessageDialogButtons, MessageDialogKind};
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS, HANDLE,
            WAIT_OBJECT_0,
        },
        Security::{GetLengthSid, GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER},
        System::{
            Registry::{
                RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegFlushKey, RegGetValueW,
                RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE,
                REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ,
            },
            Threading::{
                CreateSemaphoreW, GetCurrentProcess, OpenProcessToken, ReleaseSemaphore,
                WaitForSingleObject,
            },
        },
    };

    const KEY: &str = "Software\\com.gzers.opencodex.desktop";
    const VALUE: &str = "StartupDataRootV1";
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    // A semaphore lease may be released on a native-dialog callback thread.
    // Failed competing launches close their handles immediately; no waiter is
    // kept alive after a crashed chooser. The last close destroys the object.
    struct SelectionLease(usize);
    impl Drop for SelectionLease {
        fn drop(&mut self) {
            // SAFETY: this lease owns one successful wait and one live handle.
            unsafe {
                ReleaseSemaphore(self.0 as HANDLE, 1, std::ptr::null_mut());
                CloseHandle(self.0 as HANDLE);
            }
        }
    }

    fn lease() -> AppResult<SelectionLease> {
        let mut token = std::ptr::null_mut();
        // SAFETY: current-process token is queried and closed locally.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(failure("current user identity could not be verified"));
        }
        let result = (|| {
            let mut needed = 0;
            // SAFETY: first call obtains the bounded required output size.
            unsafe {
                GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
            }
            if needed == 0 || needed > 65536 {
                return Err(failure("current user identity size is invalid"));
            }
            // usize storage keeps TOKEN_USER naturally aligned.
            let mut buffer = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
            // SAFETY: buffer covers needed bytes and is correctly aligned.
            if unsafe {
                GetTokenInformation(
                    token,
                    TokenUser,
                    buffer.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            } == 0
            {
                return Err(failure("current user identity is unavailable"));
            }
            // SAFETY: successful TokenUser output contains the SID in this buffer.
            let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
            let length = unsafe { GetLengthSid(user.User.Sid) } as usize;
            if !(8..=256).contains(&length) {
                return Err(failure("current user SID size is invalid"));
            }
            // SAFETY: Windows returned a valid SID for the current retained token.
            let bytes = unsafe { std::slice::from_raw_parts(user.User.Sid.cast::<u8>(), length) };
            let name = wide(&format!(
                "Global\\OpenCodex-StartupRoot-{:x}",
                Sha256::digest(bytes)
            ));
            // SAFETY: terminated UTF-16 name, no inherited custom ACL or handle.
            let handle = unsafe { CreateSemaphoreW(std::ptr::null(), 1, 1, name.as_ptr()) };
            if handle.is_null() {
                return Err(failure("startup selection lock is unavailable"));
            }
            // SAFETY: zero-time wait never steals or terminates another chooser.
            if unsafe { WaitForSingleObject(handle, 0) } != WAIT_OBJECT_0 {
                unsafe {
                    CloseHandle(handle);
                }
                return Err(AppError::InstanceLockConflict);
            }
            Ok(SelectionLease(handle as usize))
        })();
        // SAFETY: token is owned by this function.
        unsafe {
            CloseHandle(token);
        }
        result
    }

    fn read_choice() -> AppResult<Option<PathBuf>> {
        let key = wide(KEY);
        let value = wide(VALUE);
        let mut size = 0;
        // SAFETY: read-only query of this application's current-user string.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS || !(2..=65536).contains(&size) || !size.is_multiple_of(2) {
            return Err(failure("the saved startup root choice could not be read"));
        }
        let mut buffer = vec![0u16; size as usize / 2];
        // SAFETY: buffer is allocated for the queried byte length; growth fails.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if status != ERROR_SUCCESS
            || size < 2
            || !size.is_multiple_of(2)
            || size as usize > buffer.len() * 2
        {
            return Err(failure("the saved startup choice changed while being read"));
        }
        let count = size as usize / 2;
        if buffer[count - 1] != 0 || buffer[..count - 1].contains(&0) {
            return Err(failure(
                "the saved startup choice has invalid string framing",
            ));
        }
        let text = String::from_utf16(&buffer[..count - 1])
            .map_err(|_| failure("invalid startup choice encoding"))?;
        decode_choice(&text).map(Some)
    }

    fn persist_choice(path: &Path) -> AppResult<()> {
        // The per-user selection lease serializes the application's startup
        // writers. Refuse external changes rather than overwriting a choice.
        if read_choice()?.is_some() {
            return Err(failure("a startup choice was saved by another operation"));
        }
        let text = encode_choice(path)?;
        let data = wide(&text);
        if data.len() > 32768 {
            return Err(failure("startup choice is too large"));
        }
        let key_name = wide(KEY);
        let value = wide(VALUE);
        let mut key = std::ptr::null_mut();
        // SAFETY: create/open only this application's HKCU key.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                key_name.as_ptr(),
                0,
                std::ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                std::ptr::null(),
                &mut key,
                std::ptr::null_mut(),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(failure("startup choice could not be saved"));
        }
        // SAFETY: bounded terminated UTF-16 data and the locally owned HKEY.
        let status = unsafe {
            RegSetValueExW(
                key,
                value.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        };
        // This runs for a first admitted anchor or an explicit recovery choice,
        // never every startup.
        // SAFETY: key remains open with the query right required by RegFlushKey.
        let flush_status = if status == ERROR_SUCCESS {
            unsafe { RegFlushKey(key) }
        } else {
            status
        };
        unsafe {
            RegCloseKey(key);
        }
        if status != ERROR_SUCCESS
            || flush_status != ERROR_SUCCESS
            || read_choice()?.as_deref() != Some(path)
        {
            return Err(failure(
                "startup choice persistence requires a restart readback",
            ));
        }
        Ok(())
    }

    /// Called only by ocxd's explicit installer command. Retain the same
    /// per-user startup lease until all data and the exact pointer are removed.
    pub(crate) fn remove_for_uninstall() -> AppResult<()> {
        if crate::modules::test_sandbox::enabled() {
            return Err(failure("installer cleanup is not a sandbox command"));
        }
        let _lease = lease()?;
        let choice = read_choice()?;
        let executable = std::env::current_exe()
            .map_err(|_| failure("installed cleanup executable cannot be resolved"))?;
        let install = executable
            .parent()
            .ok_or_else(|| failure("installation directory is unavailable"))?;
        let platform = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| failure("application data directory is unavailable"))?
            .join("com.gzers.opencodex.desktop");
        let anchor = match &choice {
            Some(path) => path.clone(),
            None => bootstrap::default_anchor(&platform, &executable, true)?,
        };
        // An app which was never opened has no initialized data to remove.
        // A saved but missing root likewise needs only pointer removal.
        match std::fs::symlink_metadata(&anchor) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(failure("saved data root cannot be inspected")),
            Ok(_) => {
                let plan =
                    crate::modules::data_root::uninstall::RemovalPlan::prepare(&anchor, install)?;
                if read_choice()? != choice {
                    return Err(failure("startup choice changed before removal"));
                }
                plan.remove(&anchor, install)?;
            }
        }
        if read_choice()? != choice {
            return Err(failure("startup choice changed during removal"));
        }
        if choice.is_some() {
            let key_name = wide(KEY);
            let value = wide(VALUE);
            let mut key = std::ptr::null_mut();
            // SAFETY: open only the existing application-owned HKCU key.
            let status = unsafe {
                RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    key_name.as_ptr(),
                    0,
                    KEY_QUERY_VALUE | KEY_SET_VALUE,
                    &mut key,
                )
            };
            if status != ERROR_SUCCESS {
                return Err(failure("startup choice cannot be cleared"));
            }
            // SAFETY: remove only this app's saved startup value; the handle
            // and UTF-16 value are locally owned and always released.
            let status = unsafe { RegDeleteValueW(key, value.as_ptr()) };
            let flush = unsafe { RegFlushKey(key) };
            unsafe { RegCloseKey(key) };
            if status != ERROR_SUCCESS || flush != ERROR_SUCCESS || read_choice()?.is_some() {
                return Err(failure("startup choice removal requires verification"));
            }
        }
        Ok(())
    }

    fn explain<R: Runtime>(app: &tauri::AppHandle<R>, error: &AppError) {
        let message = match error {
            AppError::InstanceLockConflict => "已有管理器或目录选择正在运行。请先关闭原实例后再试；本次未切换数据目录。",
            AppError::DataRootWriteDenied { .. } => "已有数据目录无法写入。请检查该目录的访问权限后重启；现有绑定和数据已保留，本次未切换到其他目录。",
            _ => "数据目录未能安全载入。请检查目录结构、访问权限和未完成的迁移后重启；本次未自动改用其他目录。",
        };
        if let Some(window) = app.get_window("main") {
            let _ = window.hide();
        }
        let handle = app.clone();
        app.dialog()
            .message(message)
            .title("数据目录暂不可用")
            .kind(MessageDialogKind::Error)
            .show(move |_| handle.exit(1));
    }

    pub(crate) fn prepare<R: Runtime>(
        app: &tauri::AppHandle<R>,
        platform_root: &Path,
        executable: &Path,
    ) -> AppResult<Option<StartupResolution>> {
        let lease = match lease() {
            Ok(value) => value,
            Err(error) => {
                explain(app, &error);
                return Ok(None);
            }
        };
        match admit_registered(platform_root, executable, read_choice, persist_choice) {
            Ok(InitialAdmission::Ready(resolution)) => Ok(Some(resolution)),
            Ok(InitialAdmission::Choose(anchor)) => {
                if let Some(window) = app.get_window("main") {
                    let _ = window.hide();
                }
                let handle = app.clone();
                app.dialog().message(
                    "安装目录下的数据目录无法写入。请选择一个可写的空目录或已有合法管理器数据目录。选择验证通过后将保存并重启；取消将退出，不自动改用 AppData，也不移动或删除原目录。"
                ).title("选择数据目录")
                    .buttons(MessageDialogButtons::OkCancelCustom("选择目录".into(), "退出".into()))
                    .show(move |accepted| {
                        if !accepted { handle.exit(0); return; }
                        let after_pick = handle.clone();
                        handle.dialog().file().set_title("选择管理器数据目录").pick_folder(move |picked| {
                            let selected = match picked {
                                Some(FilePath::Path(path)) => Some(path),
                                None => { after_pick.exit(0); return; }
                                _ => { explain(&after_pick, &failure("a local directory is required")); return; }
                            };
                            std::thread::spawn(move || {
                                let selection_lease = lease;
                                let result = commit_selected(selected, &anchor,
                                    |path| bootstrap::resolve_locked_with_boundary(path, None),
                                    persist_choice);
                                match result {
                                    Ok(Some(retained_root_locks)) => {
                                        let handoff = crate::infrastructure::windows_restart::prepare_current_restart()
                                            .and_then(|handoff| handoff.commit());
                                        drop(retained_root_locks);
                                        drop(selection_lease);
                                        match handoff {
                                            Ok(()) => after_pick.exit(0),
                                            Err(_) => {
                                                let exit_handle = after_pick.clone();
                                                after_pick.dialog().message(
                                                    "数据目录已保存，但未能准备自动重启。请退出后重新打开应用；本次未移动或删除数据。"
                                                ).title("请重新打开应用")
                                                    .kind(MessageDialogKind::Error)
                                                    .show(move |_| exit_handle.exit(1));
                                            }
                                        }
                                    }
                                    Ok(None) => after_pick.exit(0),
                                    Err(error) => explain(&after_pick, &error),
                                }
                            });
                        });
                    });
                Ok(None)
            }
            Err(error) => {
                explain(app, &error);
                Ok(None)
            }
        }
    }
}

#[cfg(windows)]
pub(crate) use native::prepare;
#[cfg(windows)]
pub(crate) use native::remove_for_uninstall;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let selected = base.join("selected");
        std::fs::create_dir(&selected).unwrap();
        (temp, selected, base.join("install/data"))
    }

    #[test]
    fn real_initial_admission_uses_install_data_then_keeps_its_instance_lock() {
        let (temp, _target, anchor) = fixture();
        let base = temp.path().canonicalize().unwrap();
        let legacy = base.join("legacy");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        let first = match admit_initial(&legacy, &executable, None).unwrap() {
            InitialAdmission::Ready(value) => value,
            InitialAdmission::Choose(path) => {
                panic!("writable default requested recovery: {path:?}")
            }
        };
        assert_eq!(first.anchor, anchor);
        assert_eq!(
            first.runtime_config.active_data_root,
            anchor.canonicalize().unwrap()
        );
        assert!(first.sandbox_boundary.is_none());
        assert_eq!(first.locks.len(), 1);
        assert!(matches!(
            admit_initial(&legacy, &executable, None),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(!legacy.exists());
        drop(first);
        assert!(matches!(
            admit_initial(&legacy, &executable, None),
            Ok(InitialAdmission::Ready(_))
        ));
    }

    #[test]
    fn installed_location_change_reuses_registered_anchor_and_its_reference_binding() {
        let (temp, target, anchor) = fixture();
        let base = temp.path().canonicalize().unwrap();
        let platform = base.join("legacy");
        let first_executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        let moved_executable = base.join("new-install/opencodex-desktop.exe");
        let pointer = base.join("fixture-startup-pointer.json");
        let read_pointer = || {
            if pointer.exists() {
                decode_choice(&std::fs::read_to_string(&pointer).unwrap()).map(Some)
            } else {
                Ok(None)
            }
        };
        let first = match admit_registered(&platform, &first_executable, read_pointer, |path| {
            assert!(matches!(
                crate::modules::instance::AppInstanceLock::acquire(path),
                Err(AppError::InstanceLockConflict)
            ));
            std::fs::write(&pointer, encode_choice(path)?).unwrap();
            Ok(())
        })
        .unwrap()
        {
            InitialAdmission::Ready(value) => value,
            InitialAdmission::Choose(_) => panic!("writable default must be admitted"),
        };
        assert_eq!(
            decode_choice(&std::fs::read_to_string(&pointer).unwrap()).unwrap(),
            anchor.canonicalize().unwrap()
        );
        drop(first);
        // Settings can bind this durable anchor to a custom active root. A
        // later executable location must keep both this binding and its data.
        crate::modules::data_root::initialize(&target).unwrap();
        std::fs::write(target.join("manager-state/keep"), b"retained state").unwrap();
        crate::modules::data_root::switch_reference(
            &anchor,
            &target,
            crate::modules::data_root::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        let before =
            std::fs::read(anchor.join(crate::modules::data_root::METADATA_RELATIVE_PATH)).unwrap();
        let resumed = match admit_registered(&platform, &moved_executable, read_pointer, |_| {
            panic!("a registered pointer must not be rewritten")
        })
        .unwrap()
        {
            InitialAdmission::Ready(value) => value,
            InitialAdmission::Choose(_) => panic!("registered root must not request a new choice"),
        };
        assert_eq!(resumed.anchor, anchor.canonicalize().unwrap());
        assert_eq!(
            resumed.runtime_config.active_data_root,
            target.canonicalize().unwrap()
        );
        assert_eq!(resumed.locks.len(), 2);
        assert_eq!(
            std::fs::read(target.join("manager-state/keep")).unwrap(),
            b"retained state"
        );
        assert_eq!(
            std::fs::read(anchor.join(crate::modules::data_root::METADATA_RELATIVE_PATH)).unwrap(),
            before
        );
        assert!(!base.join("new-install/data").exists());
        assert!(!platform.exists());
    }

    #[test]
    fn first_anchor_registration_failure_releases_locks_and_refuses_business_startup() {
        let (temp, _target, anchor) = fixture();
        let platform = temp.path().canonicalize().unwrap().join("legacy");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        let result = admit_registered(
            &platform,
            &executable,
            || Ok(None),
            |path| {
                assert!(matches!(
                    crate::modules::instance::AppInstanceLock::acquire(path),
                    Err(AppError::InstanceLockConflict)
                ));
                Err(failure("startup pointer persistence is unavailable"))
            },
        );
        assert!(matches!(result, Err(AppError::FileSystem { .. })));
        assert!(crate::modules::instance::AppInstanceLock::acquire(&anchor).is_ok());
        assert!(!platform.exists());
    }

    #[test]
    fn unreadable_or_future_pointer_stops_before_creating_installation_data() {
        let (temp, _target, anchor) = fixture();
        let platform = temp.path().canonicalize().unwrap().join("legacy");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        for payload in ["corrupt", r#"{"schema":2,"anchor":"invalid"}"#] {
            let result = admit_registered(
                &platform,
                &executable,
                || decode_choice(payload).map(Some),
                |_| panic!("an unreadable pointer must not be overwritten"),
            );
            assert!(result.is_err());
            assert!(!anchor.exists());
            assert!(!platform.exists());
        }
    }

    #[test]
    fn uncertain_pointer_commit_is_reconciled_without_resetting_a_moved_install() {
        let (temp, _target, anchor) = fixture();
        let base = temp.path().canonicalize().unwrap();
        let platform = base.join("legacy");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        let moved = base.join("new-install/opencodex-desktop.exe");
        let pointer = base.join("fixture-pointer.json");
        let first = admit_registered(
            &platform,
            &executable,
            || Ok(None),
            |path| {
                std::fs::write(&pointer, encode_choice(path)?).unwrap();
                Err(failure("pointer flush/readback result is uncertain"))
            },
        );
        assert!(first.is_err());
        let restarted = admit_registered(
            &platform,
            &moved,
            || decode_choice(&std::fs::read_to_string(&pointer).unwrap()).map(Some),
            |_| panic!("an uncertain committed pointer must not be reset"),
        )
        .unwrap();
        match restarted {
            InitialAdmission::Ready(resolution) => {
                assert_eq!(resolution.anchor, anchor.canonicalize().unwrap());
            }
            InitialAdmission::Choose(_) => panic!("committed anchor must be reconciled"),
        }
        assert!(!base.join("new-install/data").exists());
        assert!(!platform.exists());
    }

    #[test]
    fn legacy_anchor_is_registered_without_changing_its_binding_or_external_home() {
        let (temp, target, anchor) = fixture();
        let base = temp.path().canonicalize().unwrap();
        let legacy = base.join("legacy");
        let home = base.join("external-home");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        crate::modules::data_root::initialize(&legacy).unwrap();
        crate::modules::data_root::initialize(&target).unwrap();
        crate::modules::data_root::set_opencodex_home(
            &legacy,
            crate::modules::data_root::OpenCodexHomeMode::External,
            Some(&home),
        )
        .unwrap();
        crate::modules::data_root::switch_reference(
            &legacy,
            &target,
            crate::modules::data_root::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        let pointer = RefCell::new(None);
        let result = match admit_registered(
            &legacy,
            &executable,
            || Ok(None),
            |path| {
                *pointer.borrow_mut() = Some(encode_choice(path)?);
                Ok(())
            },
        )
        .unwrap()
        {
            InitialAdmission::Ready(value) => value,
            InitialAdmission::Choose(_) => panic!("legacy binding must remain admitted"),
        };
        assert_eq!(
            decode_choice(pointer.borrow().as_ref().unwrap()).unwrap(),
            legacy.canonicalize().unwrap()
        );
        assert_eq!(
            result.runtime_config.active_data_root,
            target.canonicalize().unwrap()
        );
        assert_eq!(
            crate::modules::data_root::resolve_opencodex_home(&result.runtime_config),
            home
        );
        assert!(!anchor.exists());
    }

    #[test]
    fn saved_choice_precedes_legacy_and_invalid_choice_never_falls_back() {
        let (temp, target, anchor) = fixture();
        let legacy = temp.path().canonicalize().unwrap().join("legacy");
        let executable = anchor.parent().unwrap().join("opencodex-desktop.exe");
        std::fs::create_dir(&legacy).unwrap();
        std::fs::write(legacy.join("keep"), b"old legacy").unwrap();
        crate::modules::data_root::initialize(&target).unwrap();
        let selected = match admit_initial(&legacy, &executable, Some(&target)).unwrap() {
            InitialAdmission::Ready(value) => value,
            InitialAdmission::Choose(path) => {
                panic!("valid selected root requested recovery: {path:?}")
            }
        };
        assert_eq!(selected.anchor, target.canonicalize().unwrap());
        assert!(!anchor.exists());
        drop(selected);
        let metadata = target.join(crate::modules::data_root::METADATA_RELATIVE_PATH);
        std::fs::write(&metadata, b"corrupt").unwrap();
        assert!(admit_initial(&legacy, &executable, Some(&target)).is_err());
        assert_eq!(std::fs::read(metadata).unwrap(), b"corrupt");
        assert_eq!(std::fs::read(legacy.join("keep")).unwrap(), b"old legacy");
        assert!(!legacy.join("manager-state").exists());
        assert!(!anchor.exists());
    }

    #[test]
    fn only_fresh_unconfigured_native_write_denial_offers_selection() {
        let denied = AppError::DataRootWriteDenied {
            operation: "open instance lock".into(),
        };
        assert!(recovery_allowed(&denied, false, true));
        assert!(!recovery_allowed(&denied, true, true));
        assert!(!recovery_allowed(&denied, false, false));
        for error in [
            AppError::InstanceLockConflict,
            AppError::NotConfigured,
            failure("corrupt metadata"),
            AppError::AtomicWrite {
                reason: "failed".into(),
            },
        ] {
            assert!(!recovery_allowed(&error, false, true));
        }
    }

    #[test]
    fn recovery_defers_only_main_and_preserves_its_window_configuration() {
        use tauri::utils::config::{Config, WindowConfig};
        let mut config = Config::default();
        config.app.windows = vec![
            WindowConfig {
                label: "other".into(),
                ..Default::default()
            },
            WindowConfig {
                label: "main".into(),
                width: 1180.0,
                ..Default::default()
            },
        ];
        let retained = defer_main(&mut config).unwrap();
        assert_eq!(retained.width, 1180.0);
        assert!(retained.create);
        assert!(config.app.windows[0].create);
        assert!(!config.app.windows[1].create);
    }

    #[test]
    fn cancellation_does_not_admit_a_root_or_write_a_choice() {
        let (_temp, _target, anchor) = fixture();
        assert!(commit_selected::<()>(
            None,
            &anchor,
            |_| panic!("cancel must not initialize"),
            |_| panic!("cancel must not persist")
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn persistence_occurs_only_after_successful_admission_while_locks_are_retained() {
        let (_temp, target, anchor) = fixture();
        let order = RefCell::new(Vec::new());
        let retained = commit_selected(
            Some(target),
            &anchor,
            |_| {
                order.borrow_mut().push("admit");
                Ok(42)
            },
            |_| {
                order.borrow_mut().push("persist");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(retained, Some(42));
        assert_eq!(*order.borrow(), ["admit", "persist"]);
    }

    #[test]
    fn admitted_selection_retains_the_real_root_lock_through_choice_commit() {
        let (_temp, target, anchor) = fixture();
        let saved = RefCell::new(None);
        let retained = commit_selected(
            Some(target.clone()),
            &anchor,
            |path| crate::modules::data_root::bootstrap::resolve_locked_with_boundary(path, None),
            |path| {
                assert!(matches!(
                    crate::modules::instance::AppInstanceLock::acquire(path),
                    Err(AppError::InstanceLockConflict)
                ));
                *saved.borrow_mut() = Some(encode_choice(path)?);
                Ok(())
            },
        )
        .unwrap()
        .unwrap();
        let chosen = decode_choice(saved.borrow().as_ref().unwrap()).unwrap();
        assert_eq!(retained.0.active_data_root, chosen);
        assert!(!anchor.exists());
        drop(retained);
        assert!(validate_saved(&chosen).is_ok());
        assert!(crate::modules::instance::AppInstanceLock::acquire(&chosen).is_ok());
    }

    #[test]
    fn rejected_or_busy_target_never_overwrites_saved_choice() {
        let (_temp, target, anchor) = fixture();
        let writes = Cell::new(0);
        let result = commit_selected::<()>(
            Some(target),
            &anchor,
            |_| Err(AppError::InstanceLockConflict),
            |_| {
                writes.set(writes.get() + 1);
                Ok(())
            },
        );
        assert!(matches!(result, Err(AppError::InstanceLockConflict)));
        assert_eq!(writes.get(), 0);
    }

    #[test]
    fn persistence_failure_drops_locks_without_claiming_startup_success() {
        let (_temp, target, anchor) = fixture();
        let result = commit_selected(
            Some(target.clone()),
            &anchor,
            |path| crate::modules::data_root::bootstrap::resolve_locked_with_boundary(path, None),
            |_| Err(failure("choice store unavailable")),
        );
        assert!(result.is_err());
        // Original directory/binding are not written by this selection.
        assert!(!anchor.exists());
        assert!(crate::modules::instance::AppInstanceLock::acquire(&target).is_ok());
    }

    #[test]
    fn nonempty_unowned_root_is_rejected_without_modification() {
        let (_temp, target, anchor) = fixture();
        let payload = target.join("unrelated.json");
        std::fs::write(&payload, b"original").unwrap();
        assert!(validate_selection(&target, &anchor).is_err());
        assert_eq!(std::fs::read(&payload).unwrap(), b"original");
        assert!(!target.join("manager-state").exists());
    }

    #[test]
    fn corrupt_future_and_unpublished_roots_never_commit_a_choice() {
        let (_temp, target, anchor) = fixture();
        std::fs::create_dir(target.join("manager-state")).unwrap();
        let metadata = target.join(crate::modules::data_root::METADATA_RELATIVE_PATH);
        for payload in [b"invalid".as_slice(), br#"{"structure_version":"2"}"#] {
            std::fs::write(&metadata, payload).unwrap();
            assert!(commit_selected::<()>(
                Some(target.clone()),
                &anchor,
                |_| panic!("invalid root must not be admitted"),
                |_| panic!("must not persist")
            )
            .is_err());
            assert_eq!(std::fs::read(&metadata).unwrap(), payload);
        }
        std::fs::write(&metadata, br#"{"structure_version":"1"}"#).unwrap();
        std::fs::write(
            target.join(crate::modules::data_root::migration::UNPUBLISHED_MARKER),
            b"pending",
        )
        .unwrap();
        assert!(commit_selected(
            Some(target.clone()),
            &anchor,
            |path| crate::modules::data_root::bootstrap::resolve_locked_with_boundary(path, None),
            |_| panic!("unpublished root must not persist")
        )
        .is_err());
        assert_eq!(
            std::fs::read(&metadata).unwrap(),
            br#"{"structure_version":"1"}"#
        );
    }

    #[test]
    fn saved_root_disappearance_emptying_or_corruption_stops_without_reinitializing() {
        let (_temp, target, _anchor) = fixture();
        assert!(validate_saved(&target).is_err());
        crate::modules::data_root::initialize(&target).unwrap();
        assert_eq!(
            validate_saved(&target).unwrap(),
            target.canonicalize().unwrap()
        );
        let metadata = target.join(crate::modules::data_root::METADATA_RELATIVE_PATH);
        std::fs::write(&metadata, b"corrupt").unwrap();
        assert!(validate_saved(&target).is_err());
        assert_eq!(std::fs::read(&metadata).unwrap(), b"corrupt");
        let missing = target.join("no-longer-present");
        assert!(validate_saved(&missing).is_err());
        assert!(!missing.exists());
    }

    #[test]
    fn fresh_anchor_does_not_admit_existing_partitions_or_migration_markers() {
        let (_temp, target, anchor) = fixture();
        assert!(fresh_anchor(&anchor).unwrap());
        assert!(fresh_anchor(&target).unwrap());
        std::fs::write(target.join(".data-root-migration-incomplete"), b"pending").unwrap();
        assert!(!fresh_anchor(&target).unwrap());
    }

    #[test]
    fn selected_directory_and_saved_pointer_roundtrip_without_default_fallback() {
        let (_temp, target, anchor) = fixture();
        let selected = validate_selection(&target, &anchor).unwrap();
        assert_eq!(
            decode_choice(&encode_choice(&selected).unwrap()).unwrap(),
            selected
        );
        for bad in [
            r#"{"schema":2,"anchor":"/tmp/data"}"#,
            "{}",
            "invalid",
            r#"{"schema":1,"anchor":"relative"}"#,
        ] {
            assert!(decode_choice(bad).is_err());
        }
    }

    #[test]
    fn drive_root_parent_traversal_and_source_nesting_are_rejected() {
        let (_temp, target, anchor) = fixture();
        assert!(validate_selection(target.parent().unwrap(), &anchor).is_err());
        // PathBuf::push normalizes '..' for Windows verbatim prefixes. Keep
        // the raw input so this actually exercises a traversal supplied by a
        // pointer or chooser, rather than its already-normalized destination.
        let mut traversal = target.as_os_str().to_os_string();
        traversal.push(std::path::MAIN_SEPARATOR_STR);
        traversal.push("..");
        traversal.push(std::path::MAIN_SEPARATOR_STR);
        traversal.push("selected");
        let traversal = Path::new(&traversal);
        assert!(traversal
            .components()
            .any(|part| part == Component::ParentDir));
        assert!(validate_selection(traversal, &anchor).is_err());
        assert!(check_path(target.ancestors().last().unwrap()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_directory_is_not_initialized_as_a_selected_root() {
        let (temp, target, anchor) = fixture();
        let alias = temp.path().canonicalize().unwrap().join("alias");
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        assert!(validate_selection(&alias, &anchor).is_err());
        assert!(!target.join("manager-state").exists());
    }

    #[cfg(unix)]
    #[test]
    fn linked_partition_is_rejected_before_selected_root_repair() {
        let (temp, target, anchor) = fixture();
        let outside = temp.path().canonicalize().unwrap().join("outside");
        std::fs::create_dir(&outside).unwrap();
        crate::modules::data_root::initialize(&target).unwrap();
        std::fs::remove_dir(target.join("cache")).unwrap();
        std::os::unix::fs::symlink(&outside, target.join("cache")).unwrap();
        assert!(validate_selection(&target, &anchor).is_err());
        assert!(validate_saved(&target).is_err());
        assert!(std::fs::read_dir(&outside).unwrap().next().is_none());
    }
}
