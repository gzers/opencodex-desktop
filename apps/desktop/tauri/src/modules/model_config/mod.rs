//! Model configuration owns immutable template snapshots and precise adoption.
//! No runtime, credentials, discovery or provider calls occur in the local store.
pub mod matching;
pub mod schema;
pub mod store;

pub use schema::*;
pub use store::ModelStore;

#[cfg(test)]
mod tests;
