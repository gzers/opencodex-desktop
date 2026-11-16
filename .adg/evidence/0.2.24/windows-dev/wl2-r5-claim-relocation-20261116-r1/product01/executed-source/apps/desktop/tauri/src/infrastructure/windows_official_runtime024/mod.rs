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
