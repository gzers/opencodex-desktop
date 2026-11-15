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
            path_namespace(pinned_path_observation(&home.join(name))),
        )
    })
    .collect()
}

/// Observe the exact leaf only while its nearest existing parent and every
/// ancestor are pinned as non-reparse directories. A missing leaf reached via
/// a junction is not evidence that the declared H namespace is absent.
struct PathObservationError {
    leaf: bool,
    cause: io::Error,
}
fn path_namespace(result: Result<std::fs::Metadata, PathObservationError>) -> Namespace {
    match result {
        Err(e) if e.leaf && e.cause.kind() == io::ErrorKind::NotFound => Namespace::Absent {
            raw_code: e.cause.raw_os_error().unwrap_or(2) as u32,
        },
        Err(e) => Namespace::Unknown {
            stage: if e.leaf {
                "exact-staged-leaf"
            } else {
                "exact-staged-parent-pin"
            },
            raw_code: e.cause.raw_os_error().unwrap_or(-1) as u32,
        },
        Ok(_) => Namespace::Present,
    }
}
fn pinned_path_observation(path: &Path) -> Result<std::fs::Metadata, PathObservationError> {
    let parent_error = |cause| PathObservationError { leaf: false, cause };
    let mut parent = None;
    for ancestor in path.ancestors().skip(1) {
        match std::fs::symlink_metadata(ancestor) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(parent_error(e)),
            Ok(_) => {
                parent = Some(
                    super::windows_official_runtime024::PinnedDirectory::open(ancestor)
                        .map_err(parent_error)?,
                );
                break;
            }
        }
    }
    let parent =
        parent.ok_or_else(|| parent_error(io::Error::other("absence parent unavailable")))?;
    let observed =
        std::fs::symlink_metadata(path).map_err(|cause| PathObservationError { leaf: true, cause });
    parent.recheck().map_err(parent_error)?;
    observed
}

#[cfg(test)]
#[path = "windows_supervisor024_tests.rs"]
mod tests;

use super::windows_official_lifecycle024::Owner;
use crate::{
    errors::{AppError, AppResult},
    modules::runtime::windows_lifecycle024_contract::{NativeSession, SupervisorAbsence},
};
fn unconfirmed(detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_supervisor_unknown".into(),
        detail: detail.into(),
    }
}
/// Nonserializable, non-Clone proof. A query response/receipt cannot construct it.
pub(crate) struct Absent {
    scope: String,
    deadline: Instant,
    observed: Instant,
    native: NativeQueries,
}
/// Retained old completed-Stop fact, never a current H absence claim after a
/// later native generation starts. Old completion deadline stays immutable.
pub(crate) struct ClosedAbsentHistory {
    original: Absent,
    completed_at: Instant,
}
impl ClosedAbsentHistory {
    pub(crate) fn check(&self, owner: &mut Owner, new_deadline: Instant) -> AppResult<()> {
        owner.check_current(new_deadline)?;
        if self.completed_at >= self.original.deadline
            || self.original.observed >= self.completed_at
            || self.original.scope != owner.scope()?
            || !queries_absent(&self.original.native)
        {
            return Err(unconfirmed("原Stop内监督缺席历史不成立"));
        }
        // Current machine registration is independent of H's newer runtime
        // records. Fresh Unknown/present is still a veto with its raw code.
        let fresh = query(new_deadline).map_err(|e| {
            unconfirmed(format!("历史监督注册重查 raw={:?}; {e}", e.raw_os_error()))
        })?;
        if !queries_absent(&fresh) {
            return Err(unconfirmed(format!("历史监督注册存在或未知 {fresh:?}")));
        }
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| unconfirmed(e.to_string()))?;
        if !family.complete_exit()
            || family.total_processes as usize != family.retained_members.len() + 1
        {
            return Err(unconfirmed("原历史监督家族代次未完整退出"));
        }
        owner.check_current(new_deadline)
    }
}
impl Absent {
    pub(crate) fn into_closed_history(
        self,
        owner: &mut Owner,
        deadline: Instant,
    ) -> AppResult<ClosedAbsentHistory> {
        WindowsSupervisor.recheck_absent(owner, &self, deadline)?;
        let completed_at = Instant::now();
        if completed_at >= deadline {
            return Err(unconfirmed("原监督缺席闭合窗口已过"));
        }
        Ok(ClosedAbsentHistory {
            original: self,
            completed_at,
        })
    }
    pub(crate) fn check(&self, owner: &mut Owner, deadline: Instant) -> AppResult<()> {
        owner.check_current(deadline)?;
        if self.deadline != deadline
            || Instant::now() >= deadline
            || self.observed >= deadline
            || self.scope != owner.scope()?
            || !queries_absent(&self.native)
        {
            return Err(unconfirmed("缺席证明不属于原动作/原创建代次"));
        }
        Ok(())
    }
}
fn queries_absent(q: &NativeQueries) -> bool {
    matches!(q.scm, Namespace::Absent { raw_code: 1060 })
        && matches!(
            q.scheduler,
            Namespace::Absent {
                raw_code: 0x80070002
            }
        )
}
#[cfg(test)]
mod r3_tests {
    use super::*;
    #[test]
    fn wl2_r3_typed_absence_requires_both_exact_native_namespaces() {
        let sc = Namespace::Absent { raw_code: 1060 };
        let task = Namespace::Absent {
            raw_code: 0x80070002,
        };
        assert!(queries_absent(&NativeQueries {
            scm: sc,
            scheduler: task
        }));
        for (a, b) in [
            (
                Namespace::Absent { raw_code: 1 },
                Namespace::Absent {
                    raw_code: 0x80070002,
                },
            ),
            (
                Namespace::Absent { raw_code: 1060 },
                Namespace::Absent { raw_code: 1 },
            ),
            (
                Namespace::Present,
                Namespace::Absent {
                    raw_code: 0x80070002,
                },
            ),
            (
                Namespace::Absent { raw_code: 1060 },
                Namespace::Unknown {
                    stage: "exact-GetTask",
                    raw_code: 0x80070005,
                },
            ),
            (
                Namespace::Unknown {
                    stage: "exact-OpenService",
                    raw_code: 1072,
                },
                Namespace::Absent {
                    raw_code: 0x80070002,
                },
            ),
        ] {
            assert!(!queries_absent(&NativeQueries {
                scm: a,
                scheduler: b
            }));
        }
        // This classification alone cannot construct Absent; only the Owner
        // adapter checks source/Job/staged/phase and original inherited deadline.
    }
}
pub(crate) struct WindowsSupervisor;
impl SupervisorAbsence for WindowsSupervisor {
    type Owner = Owner;
    type Absent = Absent;
    fn observe_absent(&mut self, owner: &mut Owner, stop_deadline: Instant) -> AppResult<Absent> {
        owner.check_current(stop_deadline)?;
        let scope = owner.scope()?;
        let facts = query(stop_deadline).map_err(|e| {
            unconfirmed(format!(
                "原生缺席查询未确认 raw={:?}; {e}",
                e.raw_os_error()
            ))
        })?;
        if !queries_absent(&facts) {
            return Err(unconfirmed(format!("注册存在或不可读；保留原码 {facts:?}")));
        }
        for (name, state) in staged(owner.home_path()) {
            if !matches!(state, Namespace::Absent { .. }) {
                return Err(unconfirmed(format!(
                    "准确H监督制品未缺席 {name}: {state:?}"
                )));
            }
        }
        // The official state relates runtime/current Profile/H. A present or
        // unreadable declaration needs full relation validation, not omission.
        // This adapter conservatively refuses it; no foreign registration write.
        require_state_absent(&owner.home_path().join("service-state.json"))?;
        // Official state.ts/serviceStatePaths includes the same Profile's
        // default mirror even with a custom H. It is an independent namespace,
        // not evidence that may be discarded because H's own record is absent.
        require_state_absent(&owner.profile_path().join(".opencodex/service-state.json"))?;
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| unconfirmed(e.to_string()))?;
        if !family.complete_exit()
            || family.total_processes as usize != family.retained_members.len() + 1
        {
            return Err(unconfirmed("原监督wrapper/后代未完整退出记账"));
        }
        owner.check_current(stop_deadline)?;
        if scope != owner.scope()? {
            return Err(unconfirmed("查询期间来源或原代次改变"));
        }
        Ok(Absent {
            scope,
            deadline: stop_deadline,
            observed: Instant::now(),
            native: facts,
        })
    }
    fn recheck_absent(
        &mut self,
        owner: &mut Owner,
        observed: &Absent,
        stop_deadline: Instant,
    ) -> AppResult<()> {
        observed.check(owner, stop_deadline)?;
        let fresh = self.observe_absent(owner, stop_deadline)?;
        if fresh.scope != observed.scope {
            return Err(unconfirmed("最终恢复边界缺席来源改变"));
        }
        observed.check(owner, stop_deadline)
    }
}
fn require_state_absent(path: &Path) -> AppResult<()> {
    match pinned_path_observation(path) {
        Err(e) if e.leaf && e.cause.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(unconfirmed(format!(
            "原service-state缺席路径未确认 parentFailure={}; raw={:?}; {}",
            !e.leaf,
            e.cause.raw_os_error(),
            e.cause
        ))),
        Ok(_) => Err(unconfirmed("原service-state或默认镜像仍存在；保留Unknown")),
    }
}
#[cfg(test)]
mod mirror_tests {
    use super::*;
    #[test]
    fn wl2_r3_default_state_mirror_is_independent_of_custom_home_absence() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("custom-H");
        let profile = root.path().join("Profile");
        std::fs::create_dir(&home).unwrap();
        std::fs::create_dir(&profile).unwrap();
        let original = profile.join(".opencodex");
        std::fs::create_dir(&original).unwrap();
        let mirror = original.join("service-state.json");
        std::fs::write(&mirror, b"synthetic foreign/stale declaration").unwrap();
        assert!(require_state_absent(&home.join("service-state.json")).is_ok());
        assert!(require_state_absent(&mirror).is_err());
        std::fs::remove_file(&mirror).unwrap();
        std::fs::create_dir(&mirror).unwrap();
        assert!(require_state_absent(&mirror).is_err());
    }
}
