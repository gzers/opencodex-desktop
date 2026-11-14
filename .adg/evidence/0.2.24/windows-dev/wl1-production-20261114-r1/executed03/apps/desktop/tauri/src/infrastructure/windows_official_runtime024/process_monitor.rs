//! Capture new original-Job members before resume. Completion messages alone
//! never prove exit: retain handles and reconcile total accounting or refuse.
use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    System::{
        JobObjects::{
            JobObjectAssociateCompletionPortInformation, SetInformationJobObject,
            JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
        },
        IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, PostQueuedCompletionStatus},
    },
};
pub(super) struct Monitor {
    port: Arc<OwnedHandle>,
    job: Option<OwnedHandle>,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<io::Result<(OwnedHandle, ProcessIdentity)>>,
    sender: Option<mpsc::Sender<io::Result<(OwnedHandle, ProcessIdentity)>>>,
    fault: Option<String>,
}
impl Monitor {
    pub(super) fn new(job: &OwnedHandle) -> io::Result<Self> {
        let raw = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, ptr::null_mut(), 0, 1) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let port = Arc::new(unsafe { OwnedHandle::from_raw_handle(raw) });
        let info = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
            CompletionKey: 1usize as *mut _,
            CompletionPort: raw,
        };
        check(unsafe {
            SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectAssociateCompletionPortInformation,
                (&info as *const JOBOBJECT_ASSOCIATE_COMPLETION_PORT).cast(),
                size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32,
            )
        })?;
        let mut duplicate = ptr::null_mut();
        check(unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                job.as_raw_handle(),
                GetCurrentProcess(),
                &mut duplicate,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        })?;
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            port,
            job: Some(unsafe { OwnedHandle::from_raw_handle(duplicate) }),
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
            receiver: rx,
            sender: Some(tx),
            fault: None,
        })
    }
    pub(super) fn start(&mut self, root: ProcessIdentity) -> io::Result<()> {
        if self.worker.is_some() {
            return Err(denied("Job monitor already started"));
        }
        let job = self
            .job
            .take()
            .ok_or_else(|| denied("monitor Job absent"))?;
        let port = self.port.clone();
        let stop = self.stop.clone();
        let tx = self.sender.take().unwrap();
        let (ready_tx, ready_rx) = mpsc::sync_channel(0);
        self.worker = Some(std::thread::spawn(move || {
            let _ = ready_tx.send(());
            let mut count = 0usize;
            while !stop.load(Ordering::SeqCst) {
                let (mut message, mut key, mut process) = (0u32, 0usize, ptr::null_mut());
                let result = unsafe {
                    GetQueuedCompletionStatus(
                        port.as_raw_handle(),
                        &mut message,
                        &mut key,
                        &mut process,
                        50,
                    )
                };
                if result == 0 {
                    let e = io::Error::last_os_error();
                    if e.raw_os_error() == Some(258) {
                        continue;
                    }
                    let _ = tx.send(Err(e));
                    break;
                }
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                if key != 1 {
                    let _ = tx.send(Err(denied("unknown Job completion key")));
                    break;
                }
                if message != 6 {
                    continue;
                }
                let pid = match u32::try_from(process as usize) {
                    Ok(p) if p != 0 => p,
                    _ => {
                        let _ = tx.send(Err(denied("invalid Job new-member PID")));
                        break;
                    }
                };
                if pid == root.pid {
                    continue;
                }
                count += 1;
                if count > 4096 {
                    let _ = tx.send(Err(denied("retained Job history exceeds bound")));
                    break;
                }
                let captured = (|| {
                    let raw = unsafe {
                        OpenProcess(
                            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                            0,
                            pid,
                        )
                    };
                    if raw.is_null() {
                        return Err(io::Error::last_os_error());
                    }
                    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
                    let mut member = 0;
                    check(unsafe { IsProcessInJob(raw, job.as_raw_handle(), &mut member) })?;
                    if member == 0 {
                        return Err(denied("new member is no longer in original Job"));
                    }
                    let identity = observe(raw)?;
                    if identity.token.sid != root.token.sid
                        || identity.creation_filetime < root.creation_filetime
                    {
                        return Err(denied("Job new-member identity differs"));
                    }
                    Ok((handle, identity))
                })();
                if tx.send(captured).is_err() {
                    break;
                }
            }
        }));
        ready_rx
            .recv()
            .map_err(|_| denied("Job monitor did not start"))
    }
    pub(super) fn drain(
        &mut self,
        members: &mut HashMap<u32, (OwnedHandle, ProcessIdentity)>,
    ) -> io::Result<()> {
        while let Ok(item) = self.receiver.try_recv() {
            match item {
                Ok((h, f)) => {
                    if let Some((_, old)) = members.get(&f.pid) {
                        if old != &f {
                            self.fault = Some("PID generation differs".into())
                        }
                    } else {
                        members.insert(f.pid, (h, f));
                    }
                }
                Err(e) => self.fault = Some(e.to_string()),
            }
        }
        if let Some(reason) = &self.fault {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Job completion member capture is incomplete: {reason}"),
            ))
        } else {
            Ok(())
        }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        unsafe { PostQueuedCompletionStatus(self.port.as_raw_handle(), 0, 1, ptr::null()) };
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
