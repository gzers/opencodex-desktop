//! Read-only scheduled-task observation on a fresh COM apartment.
//! No PowerShell startup, task registration, credentials or task mutation.
use std::{sync::mpsc, time::Duration};
use windows::{
    core::BSTR,
    Win32::System::{
        Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED,
        },
        TaskScheduler::{ITaskService, TaskScheduler, TASK_ENUM_HIDDEN},
        Variant::VARIANT,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryError {
    Timeout,
    Worker,
    Initialize,
    Connect,
    Folder,
    Enumerate,
    InvalidCount,
    Name,
}

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}

fn observe(name: &str) -> Result<bool, QueryError> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok() }
        .map_err(|_| QueryError::Initialize)?;
    let _apartment = Apartment;
    let service: ITaskService =
        unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER) }
            .map_err(|_| QueryError::Connect)?;
    let empty = VARIANT::default();
    unsafe { service.Connect(&empty, &empty, &empty, &empty) }.map_err(|_| QueryError::Connect)?;
    let folder = unsafe { service.GetFolder(&BSTR::from("\\")) }.map_err(|_| QueryError::Folder)?;
    let tasks =
        unsafe { folder.GetTasks(TASK_ENUM_HIDDEN.0) }.map_err(|_| QueryError::Enumerate)?;
    let count = unsafe { tasks.Count() }.map_err(|_| QueryError::Enumerate)?;
    scan_tasks(name, count, |index| {
        let task =
            unsafe { tasks.get_Item(&VARIANT::from(index)) }.map_err(|_| QueryError::Enumerate)?;
        let actual = unsafe { task.Name() }.map_err(|_| QueryError::Name)?;
        Ok(actual.to_string())
    })
}

fn scan_tasks(
    name: &str,
    count: i32,
    mut read: impl FnMut(i32) -> Result<String, QueryError>,
) -> Result<bool, QueryError> {
    if !(0..=100_000).contains(&count) {
        return Err(QueryError::InvalidCount);
    }
    for index in 1..=count {
        if read(index)?.eq_ignore_ascii_case(name) {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn opencodex_task_absent() -> Result<bool, QueryError> {
    query("opencodex-proxy", Duration::from_secs(8))
}

fn query(name: &'static str, budget: Duration) -> Result<bool, QueryError> {
    // Bound observation only. A late read-only worker has no authority to stop
    // a service or release a lifecycle lease; timeout never means absence.
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("task-scheduler-observation".into())
        .spawn(move || {
            let _ = sender.send(observe(name));
        })
        .map_err(|_| QueryError::Worker)?;
    receiver.recv_timeout(budget).map_err(|error| match error {
        mpsc::RecvTimeoutError::Timeout => QueryError::Timeout,
        mpsc::RecvTimeoutError::Disconnected => QueryError::Worker,
    })?
}

#[cfg(test)]
mod tests {
    #[test]
    fn registration_and_query_failure_never_count_as_absence() {
        assert_eq!(
            super::scan_tasks("opencodex-proxy", 2, |index| Ok(if index == 2 {
                "OpenCodex-Proxy"
            } else {
                "other"
            }
            .into())),
            Ok(false)
        );
        assert_eq!(
            super::scan_tasks("opencodex-proxy", 1, |_| Err(super::QueryError::Name)),
            Err(super::QueryError::Name)
        );
        assert_eq!(
            super::scan_tasks("opencodex-proxy", 100_001, |_| panic!(
                "must reject before reading"
            )),
            Err(super::QueryError::InvalidCount)
        );
        assert_eq!(
            super::scan_tasks("opencodex-proxy", -1, |_| panic!(
                "must reject before reading"
            )),
            Err(super::QueryError::InvalidCount)
        );
    }

    #[test]
    fn native_read_only_enumeration_has_a_bounded_result() {
        // A fresh test name; no task is ever created or removed. Access or COM
        // failure remains a test failure rather than fabricated absence.
        assert_eq!(
            super::query(
                "ocxd-native-identity-test-absent-91fb4d61",
                std::time::Duration::from_secs(8)
            ),
            Ok(true)
        );
    }
}
