//! Exact read-only native namespaces. These are observations, not serialized
//! SupervisorAbsent capabilities; an original owner must still bind/recheck them.
use serde::Serialize;
use std::{io, path::Path, sync::mpsc, time::Instant};
use windows::{
    core::BSTR,
    Win32::System::{
        Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED,
        },
        TaskScheduler::{ITaskService, TaskScheduler},
        Variant::VARIANT,
    },
};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "state")]
pub(crate) enum Namespace {
    Absent { raw_code: u32 },
    Present,
    Unknown { stage: &'static str, raw_code: u32 },
}
#[derive(Debug, Serialize)]
pub(crate) struct NativeQueries {
    pub(crate) scm: Namespace,
    pub(crate) scheduler: Namespace,
}
fn exact_not_found(stage: &'static str, code: u32, expected: u32) -> Namespace {
    if code == expected {
        Namespace::Absent { raw_code: code }
    } else {
        Namespace::Unknown {
            stage,
            raw_code: code,
        }
    }
}
fn scm() -> Namespace {
    use std::ffi::c_void;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn OpenSCManagerW(machine: *const u16, database: *const u16, access: u32) -> *mut c_void;
        fn OpenServiceW(manager: *mut c_void, name: *const u16, access: u32) -> *mut c_void;
        fn CloseServiceHandle(handle: *mut c_void) -> i32;
    }
    struct Handle(*mut c_void);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseServiceHandle(self.0);
            }
        }
    }
    let raw = unsafe { OpenSCManagerW(std::ptr::null(), std::ptr::null(), 1) };
    if raw.is_null() {
        return Namespace::Unknown {
            stage: "OpenSCManagerW-local-CONNECT",
            raw_code: io::Error::last_os_error().raw_os_error().unwrap_or(-1) as u32,
        };
    }
    let _manager = Handle(raw);
    let name: Vec<u16> =
        crate::modules::runtime::windows_lifecycle024_contract::OFFICIAL_SCM_SERVICE
            .encode_utf16()
            .chain(Some(0))
            .collect();
    let service = unsafe { OpenServiceW(raw, name.as_ptr(), 1 | 4) };
    if service.is_null() {
        return exact_not_found(
            "OpenServiceW-exact-QUERY",
            io::Error::last_os_error().raw_os_error().unwrap_or(-1) as u32,
            1060,
        );
    }
    let _service = Handle(service);
    // Any actual registration, including stopped/pending-delete/foreign, vetoes
    // absence. No enumeration, control, change-config or delete access requested.
    Namespace::Present
}
fn scheduler() -> Namespace {
    let run = || -> windows::core::Result<Namespace> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe {
                    CoUninitialize();
                }
            }
        }
        let _apartment = Apartment;
        let service: ITaskService =
            unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)? };
        let empty = VARIANT::default();
        unsafe {
            service.Connect(&empty, &empty, &empty, &empty)?;
        }
        let folder = unsafe { service.GetFolder(&BSTR::from("\\"))? };
        let name =
            BSTR::from(crate::modules::runtime::windows_lifecycle024_contract::OFFICIAL_TASK);
        match unsafe { folder.GetTask(&name) } {
            Ok(_) => Ok(Namespace::Present),
            Err(e) => Ok(exact_not_found(
                "TaskScheduler-root-GetTask-exact",
                e.code().0 as u32,
                0x8007_0002,
            )),
        }
    };
    run().unwrap_or_else(|e| Namespace::Unknown {
        stage: "TaskScheduler-apartment-connect-root",
        raw_code: e.code().0 as u32,
    })
}
pub(crate) fn query(deadline: Instant) -> io::Result<NativeQueries> {
    let (tx, rx) = mpsc::sync_channel(1);
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    // Blocking read-only OS calls cannot mint late facts. Caller only accepts a
    // complete observation received before the inherited absolute deadline.
    std::thread::Builder::new()
        .name("windows024-exact-supervisor".into())
        .spawn(move || {
            let _ = tx.send(NativeQueries {
                scm: scm(),
                scheduler: scheduler(),
            });
        })?;
    let value = rx
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => io::Error::from(io::ErrorKind::TimedOut),
            mpsc::RecvTimeoutError::Disconnected => {
                io::Error::other("native supervisor worker disconnected")
            }
        })?;
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(value)
}
pub(crate) fn staged(home: &Path) -> Vec<(&'static str, Namespace)> {
    [
        "opencodex-service-task.xml",
        "opencodex-service.cmd",
        "opencodex-service-launcher.vbs",
        "winsw/opencodex-proxy-native.xml",
        "winsw/opencodex-proxy-native.exe",
    ]
    .into_iter()
    .map(|name| {
        (
            name,
            match std::fs::symlink_metadata(home.join(name)) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Namespace::Absent {
                    raw_code: e.raw_os_error().unwrap_or(2) as u32,
                },
                Err(e) => Namespace::Unknown {
                    stage: "exact-staged-artifact",
                    raw_code: e.raw_os_error().unwrap_or(-1) as u32,
                },
                Ok(_) => Namespace::Present,
            },
        )
    })
    .collect()
}

#[cfg(test)]
#[path = "windows_supervisor024_tests.rs"]
mod tests;
