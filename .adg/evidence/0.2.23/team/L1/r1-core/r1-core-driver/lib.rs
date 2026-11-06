#![allow(dead_code)]
mod errors { #[derive(Debug)] pub struct AppError; impl std::fmt::Display for AppError { fn fmt(&self,f:&mut std::fmt::Formatter)->std::fmt::Result { write!(f,"driver-only unrelated storage error") } } }
#[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/schema.rs"] pub mod schema;
#[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/schema_v2.rs"] pub mod schema_v2;
#[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/matching_v2.rs"] pub mod matching_v2;
#[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/source_v2.rs"] pub mod source_v2;
#[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/portable_v2.rs"] pub mod portable_v2;
#[cfg(test)] #[path = "/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/src/modules/model_config/v2_tests.rs"] mod v2_tests;
