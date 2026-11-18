#[cfg(test)]
mod before_resume_record {
    include!(concat!(env!("OUT_DIR"), "/before_resume_record_data.rs"));
}
pub mod bounded;
pub mod collector;
#[cfg(test)]
mod external_repair_steps;
mod coordination {include!(concat!(env!("OUT_DIR"),"/coordination_validation.rs"));}
mod partial_capture {mod native_collection {include!(concat!(env!("OUT_DIR"),"/native_partial_collect.rs"));}}
#[path = "../../../../../../apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/streams.rs"]
mod tree;
