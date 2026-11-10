//! Windows facts for an unchanged official runtime. W0 is not registered in the
//! product yet. These process-local Rust types are not an IPC or persisted schema.
//!
//! The shared caller owns official package admission, HMAC/ready verification,
//! serialization, recovery policy, and the original 30/5/10-second deadlines.
//! Failed Win32 observations remain errors; absence is never inferred from them.
#![cfg(windows)]

mod listeners;
mod paths;
mod process;
mod security;

pub use listeners::{
    listener_exit_readback, tcp_listeners, Listener, ListenerFamily, OwnedListeners,
};
pub use paths::{create_private_directory, FileIdentity, PinnedDirectory, PrivateRecord};
pub use process::{
    FamilyState, ForcedCleanup, LaunchSpec, OwnedRuntime, ProcessIdentity, ProcessState,
    RootBinding,
};
pub use security::{current_identity, TokenIdentity};
