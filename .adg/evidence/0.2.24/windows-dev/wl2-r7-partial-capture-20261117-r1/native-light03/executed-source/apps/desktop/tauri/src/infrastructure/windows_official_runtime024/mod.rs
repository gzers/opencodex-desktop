//! Windows facts for an unchanged official runtime. W0 is not registered in the
//! product yet. These process-local Rust types are not an IPC or persisted schema.
//!
//! The shared caller owns official package admission, HMAC/ready verification,
//! serialization, recovery policy, and the original 30/5/10-second deadlines.
//! Failed Win32 observations remain errors; absence is never inferred from them.
#![cfg(windows)]

mod file_move;
pub(crate) use file_move::rename_handle as rename_original_handle;
pub use file_move::RetainedFileMove;
mod listeners;
mod paths;
mod peer;
mod pipe;
mod process;
mod security;
pub use peer::RetainedPeer;
pub use pipe::PrivatePipe;

pub use listeners::{
    listener_exit_readback, tcp_listeners, Listener, ListenerFamily, OwnedListeners,
};
pub use paths::{
    create_private_directory, validate_local_path, FileIdentity, PinnedDirectory, PrivateRecord,
};
pub use process::SuspendedOwnerHost;
pub use process::{
    CommandEvidence, FamilyState, ForcedCleanup, LaunchSpec, OwnedRuntime, ProcessIdentity,
    ProcessState, RootBinding,
    PreparedLauncherAbort, QuiescentLauncher,
};
pub use security::{current_identity, TokenIdentity};
/// Read-only private TokenUser/SYSTEM check on this exact retained file handle.
pub(crate) fn verify_private_file(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    security::verify_private(file.as_raw_handle(), &current_identity()?.sid, true)
}
pub(crate) fn verify_private_directory(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    security::verify_private(file.as_raw_handle(), &current_identity()?.sid, true)
}
/// CREATE_NEW private work object, returned immediately as the exact original
/// creation handle. The caller must retain it BEFORE any fallible post-create
/// query/write. No path reopen, truncation or ACL repair of existing objects.
pub(crate) fn create_private_originals_work_file(
    path: &std::path::Path,
) -> std::io::Result<std::fs::File> {
    use std::{
        os::windows::{ffi::OsStrExt, io::FromRawHandle},
        ptr,
    };
    validate_local_path(path)?;
    let token = current_identity()?;
    if token.elevated || token.elevation_type == 2 {
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }
    let parent =
        PinnedDirectory::open_private(path.parent().ok_or(std::io::ErrorKind::InvalidInput)?)?;
    let descriptor = security::PrivateDescriptor::new(&token.sid, false)?;
    let attributes = descriptor.attributes();
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<u16>>();
    parent.recheck()?;
    let raw = unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateFileW(
            wide.as_ptr(),
            0xc0030000,
            1,
            &attributes,
            1,
            0x200000,
            ptr::null_mut(),
        )
    };
    if raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { std::fs::File::from_raw_handle(raw) })
}
