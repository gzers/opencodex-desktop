//! Model configuration owns immutable template snapshots and precise adoption.
//! No runtime, credentials, discovery or provider calls occur in the local store.
pub mod agent;
pub mod agent_conversation;
pub mod broker;
pub mod cache_refresh;
pub(crate) mod channel_actions;
pub mod compiler;
pub mod config_file;
mod config_file_mapping;
pub mod config_remote;
pub mod discovery;
pub mod discovery_config;
pub mod execution_directory;
pub mod feedback;
pub mod file_objects;
pub mod legacy_migration;
pub mod management;
pub mod matching;
pub mod measurements;
mod operations;
pub mod portable_channels;
mod protocol_groups;
pub mod provider_auth;
pub mod provider_fields;
pub mod provider_import;
pub mod recovery;
mod remote;
mod research_receipts;
pub mod runtime;
pub mod runtime_history;
pub mod schema;
pub mod store;
pub mod tasks;
pub mod transfer;
pub mod upstream;
pub mod usage;

pub use schema::*;
pub use store::ModelStore;

#[cfg(test)]
mod tests;
