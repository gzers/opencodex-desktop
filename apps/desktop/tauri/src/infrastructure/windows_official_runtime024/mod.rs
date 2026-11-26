//! Windows facts for an unchanged official runtime. These process-local Rust
//! types are not an IPC or persisted schema.
//!
//! The shared caller owns official package admission, HMAC/ready verification,
//! serialization, recovery policy, and the original 30/5/10-second deadlines.
//! Failed Win32 observations remain errors; absence is never inferred from them.
#![cfg(windows)]

mod listeners;
mod paths;
mod process;
mod security;

#[allow(
    unused_imports,
    reason = "standalone native proof inspects listener families"
)]
pub use listeners::ListenerFamily;
pub use listeners::{listener_exit_readback, OwnedListeners};
#[allow(
    unused_imports,
    reason = "standalone native proof creates private fixture directories"
)]
pub use paths::create_private_directory;
pub use paths::{FileIdentity, PinnedDirectory, PrivateRecord};
pub use process::{FamilyState, LaunchSpec, OwnedRuntime, ProcessIdentity, ProcessState};
pub use security::{current_identity, TokenIdentity};
