//! Model configuration owns immutable template snapshots and precise adoption.
//! No runtime, credentials, discovery or provider calls occur in the local store.
pub mod agent;
pub mod broker;
pub(crate) mod channel_actions;
pub mod client_state;
pub mod compiler;
pub mod config_remote;
pub mod discovery;
pub mod feedback;
pub mod file_objects;
pub mod management;
pub mod matching;
pub mod measurements;
mod operations;
mod protocol_groups;
pub mod provider_auth;
pub mod recovery;
mod remote;
pub mod runtime;
pub mod schema;
pub mod store;
pub mod tasks;
pub mod transfer;
pub mod upstream;

pub use schema::*;
pub use store::ModelStore;

#[cfg(test)]
mod tests;
