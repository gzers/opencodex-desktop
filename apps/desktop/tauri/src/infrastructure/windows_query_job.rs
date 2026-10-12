//! A metadata query owns an anonymous, non-inheritable Windows job. Suspend the
//! new root before assigning it so it cannot create an unowned descendant.
use std::io;
use std::mem::size_of;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{
    GetProcessIdOfThread, OpenThread, ResumeThread, WaitForSingleObject, CREATE_NO_WINDOW,
    CREATE_SUSPENDED, THREAD_QUERY_LIMITED_INFORMATION, THREAD_SUSPEND_RESUME,
};

pub(crate) struct OwnedQueryJob(OwnedHandle);

impl OwnedQueryJob {
    pub(crate) fn new(command: &mut tokio::process::Command) -> io::Result<Self> {
        // Preserve background CLI behavior while preventing the assignment race.
        command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: this is a newly created, uniquely owned handle. Default
        // SECURITY_ATTRIBUTES make it non-inheritable, including by the query.
        let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let ok = unsafe {
            SetInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    pub(crate) fn assign_and_resume(&self, child: &tokio::process::Child) -> io::Result<()> {
        let process = child
            .raw_handle()
            .ok_or_else(|| io::Error::other("query process exited"))?;
        let pid = child
            .id()
            .ok_or_else(|| io::Error::other("query process has no live ID"))?;
        if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), process) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // There is exactly one primary thread before CREATE_SUSPENDED is
        // released. Never choose an arbitrary thread or resume an unassigned root.
        let thread = suspended_primary_thread(process, pid)?;
        let previous = unsafe { ResumeThread(thread.as_raw_handle()) };
        if previous == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        if previous != 1 {
            return Err(io::Error::other("unexpected query thread suspend count"));
        }
        Ok(())
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    // OwnedHandle drops the only job handle. KILL_ON_JOB_CLOSE also covers
    // normal result return, output-reader failure, unwinding and cancellation.
}

fn suspended_primary_thread(process: HANDLE, pid: u32) -> io::Result<OwnedHandle> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot) };
    let mut entry = THREADENTRY32 {
        dwSize: size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };
    if unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut selected = None;
    loop {
        if entry.dwSize as usize != size_of::<THREADENTRY32>() {
            return Err(io::Error::other("incomplete query thread snapshot"));
        }
        if entry.th32OwnerProcessID == pid && selected.replace(entry.th32ThreadID).is_some() {
            return Err(io::Error::other(
                "query root has multiple suspended threads",
            ));
        }
        entry.dwSize = size_of::<THREADENTRY32>() as u32;
        if unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) } == 0 {
            let error = unsafe { GetLastError() };
            if error != ERROR_NO_MORE_FILES {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
            break;
        }
    }
    let id = selected.ok_or_else(|| io::Error::other("query primary thread is absent"))?;
    let raw = unsafe {
        OpenThread(
            THREAD_SUSPEND_RESUME | THREAD_QUERY_LIMITED_INFORMATION,
            0,
            id,
        )
    };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let thread = unsafe { OwnedHandle::from_raw_handle(raw) };
    // Retained thread and process handles prevent later ID reuse from changing
    // the target. Re-check ownership and root liveness before resuming.
    if unsafe { GetProcessIdOfThread(raw) } != pid
        || unsafe { WaitForSingleObject(process, 0) } != WAIT_TIMEOUT
    {
        return Err(io::Error::other("query process/thread identity changed"));
    }
    Ok(thread)
}
