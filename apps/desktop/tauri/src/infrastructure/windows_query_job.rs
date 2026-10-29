//! A metadata query owns an anonymous, non-inheritable Windows job. Suspend the
//! new root before assigning it so it cannot create an unowned descendant.
use std::io;
use std::mem::size_of;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
    WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
    JobObjectAssociateCompletionPortInformation, JobObjectBasicAccountingInformation,
    JobObjectBasicProcessIdList, JobObjectExtendedLimitInformation, QueryInformationJobObject,
    SetInformationJobObject, TerminateJobObject, JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_BASIC_LIMIT_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::SystemServices::JOB_OBJECT_MSG_NEW_PROCESS;
use windows_sys::Win32::System::Threading::{
    GetProcessIdOfThread, OpenProcess, OpenThread, ResumeThread, WaitForSingleObject,
    CREATE_NO_WINDOW, CREATE_SUSPENDED, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    THREAD_QUERY_LIMITED_INFORMATION, THREAD_SUSPEND_RESUME,
};
use windows_sys::Win32::System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus};

pub(crate) struct OwnedQueryJob {
    job: OwnedHandle,
    port: OwnedHandle,
}

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
        let limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                LimitFlags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                ..Default::default()
            },
            ..Default::default()
        };
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
        let port =
            unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, std::ptr::null_mut(), 0, 1) };
        if port.is_null() {
            return Err(io::Error::last_os_error());
        }
        let port = unsafe { OwnedHandle::from_raw_handle(port) };
        let association = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
            CompletionKey: std::ptr::null_mut(),
            CompletionPort: port.as_raw_handle(),
        };
        let ok = unsafe {
            SetInformationJobObject(
                raw,
                JobObjectAssociateCompletionPortInformation,
                (&association as *const JOBOBJECT_ASSOCIATE_COMPLETION_PORT).cast(),
                size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { job: handle, port })
    }

    pub(crate) fn assign_and_resume(&self, child: &tokio::process::Child) -> io::Result<()> {
        let process = child
            .raw_handle()
            .ok_or_else(|| io::Error::other("query process exited"))?;
        let pid = child
            .id()
            .ok_or_else(|| io::Error::other("query process has no live ID"))?;
        if unsafe { AssignProcessToJobObject(self.job.as_raw_handle(), process) } == 0 {
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
        if unsafe { TerminateJobObject(self.job.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    /// Termination is asynchronous. Install activation must not race a probe's
    /// owned descendants that are still releasing package files/directories.
    pub(crate) async fn terminate_and_wait(&self) -> io::Result<()> {
        // Accounting can reach zero before the last process handle is signaled.
        // Retain real handles before termination so that this is an exit wait,
        // rather than just a wait for removal from the job's active list.
        let mut processes = Vec::new();
        self.retain_notified_processes(&mut processes)?;
        self.retain_active_processes(&mut processes)?;
        #[cfg(test)]
        eprintln!(
            "owned cleanup before terminate retained={:?}",
            processes.iter().map(|(pid, _)| *pid).collect::<Vec<_>>()
        );
        self.terminate()?;
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                self.retain_notified_processes(&mut processes)?;
                self.retain_active_processes(&mut processes)?;
                let mut account = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
                let ok = unsafe {
                    QueryInformationJobObject(
                        self.job.as_raw_handle(),
                        JobObjectBasicAccountingInformation,
                        (&mut account as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        std::ptr::null_mut(),
                    )
                };
                if ok == 0 {
                    return Err(io::Error::last_os_error());
                }
                let mut exited = true;
                for (_, process) in &processes {
                    match unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } {
                        WAIT_OBJECT_0 => {}
                        WAIT_TIMEOUT => exited = false,
                        _ => return Err(io::Error::last_os_error()),
                    }
                }
                if account.ActiveProcesses == 0 && exited {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "owned probe process family did not exit",
            )
        })?
    }

    fn retain_notified_processes(&self, retained: &mut Vec<(u32, OwnedHandle)>) -> io::Result<()> {
        // A library may already have started killing its helper before the root
        // exits. Such a helper has left the active list but its NEW_PROCESS
        // notification still identifies it until actual process teardown.
        for _ in 0..8192 {
            let mut message = 0;
            let mut key = 0;
            let mut identity = std::ptr::null_mut();
            let ok = unsafe {
                GetQueuedCompletionStatus(
                    self.port.as_raw_handle(),
                    &mut message,
                    &mut key,
                    &mut identity,
                    0,
                )
            };
            if ok == 0 {
                let error = io::Error::last_os_error();
                return if error.raw_os_error() == Some(WAIT_TIMEOUT as i32) {
                    Ok(())
                } else {
                    Err(error)
                };
            }
            if key != 0 {
                return Err(io::Error::other("unexpected owned job notification"));
            }
            if message == JOB_OBJECT_MSG_NEW_PROCESS {
                self.retain_process(identity as usize, retained)?;
            }
        }
        Err(io::Error::other("owned job notification limit exceeded"))
    }

    fn retain_active_processes(&self, retained: &mut Vec<(u32, OwnedHandle)>) -> io::Result<()> {
        // Native layout of JOBOBJECT_BASIC_PROCESS_ID_LIST with bounded trailing
        // storage. An oversized or incomplete snapshot fails closed.
        #[repr(C)]
        struct ProcessIds {
            assigned: u32,
            listed: u32,
            ids: [usize; 4096],
        }
        let mut list = Box::new(ProcessIds {
            assigned: 0,
            listed: 0,
            ids: [0; 4096],
        });
        let ok = unsafe {
            QueryInformationJobObject(
                self.job.as_raw_handle(),
                JobObjectBasicProcessIdList,
                (&mut *list as *mut ProcessIds).cast(),
                size_of::<ProcessIds>() as u32,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        if list.assigned != list.listed || list.listed as usize > list.ids.len() {
            return Err(io::Error::other("incomplete owned process snapshot"));
        }
        #[cfg(test)]
        eprintln!(
            "owned snapshot assigned={} listed={} ids={:?}",
            list.assigned,
            list.listed,
            &list.ids[..list.listed as usize]
        );
        for &id in &list.ids[..list.listed as usize] {
            self.retain_process(id, retained)?;
        }
        Ok(())
    }
    fn retain_process(&self, id: usize, retained: &mut Vec<(u32, OwnedHandle)>) -> io::Result<()> {
        let pid = u32::try_from(id).map_err(|_| io::Error::other("invalid owned process ID"))?;
        if retained.iter().any(|(existing, _)| *existing == pid) {
            return Ok(());
        }
        let raw = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid,
            )
        };
        if raw.is_null() {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_INVALID_PARAMETER as i32) {
                return Ok(()); // Already exited before the handle could be retained.
            }
            return Err(error);
        }
        let process = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut member = 0;
        if unsafe { IsProcessInJob(raw, self.job.as_raw_handle(), &mut member) } == 0 {
            return Err(io::Error::last_os_error());
        }
        #[cfg(test)]
        eprintln!("owned snapshot pid={pid} member={member} wait={}", unsafe {
            WaitForSingleObject(raw, 0)
        });
        // A reused PID must never make us wait on an unrelated process.
        if member != 0 {
            retained.push((pid, process));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    #[test]
    fn successful_root_does_not_return_before_its_owned_descendant_exits() {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE};
        let node = std::env::var_os("OPENCODEX_TEST_NODE")
            .map(std::path::PathBuf::from)
            .expect("explicit native Node required");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        tokio::runtime::Builder::new_current_thread()
            .enable_all().build().unwrap().block_on(async {
                let mut command = tokio::process::Command::new(node);
                // Keep the root alive until this fixture owns the helper handle.
                // Detached stdio prevents Node's own child-job teardown from
                // turning the intended live-descendant precondition into a race.
                command.args(["-e", "const c=require('child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{detached:true,stdio:'ignore'});console.log(c.pid);c.unref();process.stdin.once('data',()=>process.exit(0));"])
                    .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
                let job = OwnedQueryJob::new(&mut command).unwrap();
                let mut child = command.spawn().unwrap();
                job.assign_and_resume(&child).unwrap();
                let mut stdout = BufReader::new(child.stdout.take().unwrap());
                let mut line = String::new();
                tokio::time::timeout(std::time::Duration::from_secs(2), stdout.read_line(&mut line)).await.unwrap().unwrap();
                let pid = line.trim().parse::<u32>().unwrap();
                let raw = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
                assert!(!raw.is_null(), "retain helper handle before permitting root exit");
                let process = unsafe { OwnedHandle::from_raw_handle(raw) };
                assert_eq!(unsafe { WaitForSingleObject(process.as_raw_handle(), 0) }, WAIT_TIMEOUT);
                child.stdin.take().unwrap().write_all(b"exit").await.unwrap();
                let output = tokio::time::timeout(std::time::Duration::from_secs(2), child.wait_with_output()).await.unwrap().unwrap();
                assert!(output.status.success());
                assert_eq!(unsafe { WaitForSingleObject(process.as_raw_handle(), 0) }, WAIT_TIMEOUT, "fixture child must be alive before family cleanup");
                job.terminate_and_wait().await.unwrap();
                assert_eq!(unsafe { WaitForSingleObject(process.as_raw_handle(), 0) }, WAIT_OBJECT_0, "no owned child may retain a package directory at activation");
            });
    }
}
