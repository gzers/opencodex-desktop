//! Tauri Doctor 命令层。只锁共享来源并投影模块结果，不访问文件系统或进程细节。

use std::sync::{Arc, Mutex};

use crate::errors::{AppError, AppResult};
use crate::modules::doctor::{DoctorReport, DoctorSource};
use crate::state::SharedDoctorSource;
use crate::types::doctor::DoctorDto;

#[tauri::command]
pub async fn run_doctor(source: tauri::State<'_, SharedDoctorSource>) -> AppResult<DoctorDto> {
    let source = Arc::clone(&source);
    crate::commands::run_readonly("run doctor", move || run_doctor_with_source(&source)).await
}

pub fn run_doctor_with_source<S: DoctorSource + ?Sized>(
    source: &Arc<Mutex<S>>,
) -> AppResult<DoctorDto> {
    let guard = source.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    Ok(DoctorReport::run(&*guard)?.into())
}
