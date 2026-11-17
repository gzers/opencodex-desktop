#[path = "../../../../../../apps/desktop/tauri/src/infrastructure/windows_client_originals024/before_resume_record.rs"]
mod before_resume_record;
pub mod bounded;
pub mod collector;
mod coordination {include!(concat!(env!("OUT_DIR"),"/coordination_validation.rs"));}
mod partial_capture {mod native_collection {include!(concat!(env!("OUT_DIR"),"/native_partial_collect.rs"));}}
#[path = "../../../../../../apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/streams.rs"]
mod tree;
