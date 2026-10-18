//! Model configuration owns immutable template snapshots and precise adoption.
//! No runtime, credentials, discovery or provider calls occur in the local store.
pub mod agent;
pub mod broker;
pub mod compiler;
pub mod discovery;
pub mod feedback;
pub mod management;
pub mod matching;
pub mod measurements;
pub mod remote;
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
