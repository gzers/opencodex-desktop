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
pub mod content_versions;
pub mod discovery;
pub mod discovery_config;
pub mod execution_directory;
pub mod execution_directory_v2;
pub mod feedback;
pub mod file_exports;
pub mod file_objects;
pub mod file_objects_v2;
pub mod file_transfer_v2;
pub mod objects_overlay_v2;
pub mod legacy_migration;
pub mod management;
pub mod matching;
pub mod schema_v2;
pub mod matching_v2;
pub mod source_v2;
pub mod credential_references_v2;
pub mod saved_objects_capture_v2;
pub mod portable_v2;
pub mod compiler_v2;
pub mod store_v2;
pub mod template_ops_v2;
pub mod agent_proposal_v2;
pub mod agent_runtime_v2;
pub mod model_transaction_v2;
#[cfg(test)]
mod v2_tests;
#[cfg(test)]
mod consumer_v2_tests;
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
pub mod sync_state;
pub mod sync_v2;
pub mod sync_host_v2;
pub mod sync_service_v2;
pub mod sync_identity_v2;
pub mod sync_local_v2;
pub mod receive_models_v2;
pub mod tasks;
pub mod transfer;
pub mod transfer_backup;
pub mod transfer_choices;
pub mod transfer_protection;
pub mod transfer_results;
pub mod upstream;
pub mod usage;

pub use schema::*;
pub use store::ModelStore;

#[cfg(test)]
mod tests;
