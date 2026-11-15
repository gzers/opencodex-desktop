//! First-instance, local-only, protected SID pipe with absolute bounded I/O.
//! Closing a connection closes only this pipe, never a Job or process.
use super::{
    peer::RetainedPeer,
    security::{check, current_identity, denied, verify_private, PrivateDescriptor},
};
use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::IO::OVERLAPPED,
};
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateNamedPipeW(
        name: *const u16,
        mode: u32,
        flags: u32,
        instances: u32,
        out: u32,
        input: u32,
        timeout: u32,
        sa: *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
    ) -> HANDLE;
    fn ConnectNamedPipe(pipe: HANDLE, overlapped: *mut OVERLAPPED) -> i32;
    fn DisconnectNamedPipe(pipe: HANDLE) -> i32;
    fn GetNamedPipeClientProcessId(pipe: HANDLE, pid: *mut u32) -> i32;
    fn GetNamedPipeServerProcessId(pipe: HANDLE, pid: *mut u32) -> i32;
    fn ReadFile(
        file: HANDLE,
        buffer: *mut u8,
        n: u32,
        read: *mut u32,
        overlapped: *mut OVERLAPPED,
    ) -> i32;
    fn WriteFile(
        file: HANDLE,
        buffer: *const u8,
        n: u32,
        written: *mut u32,
        overlapped: *mut OVERLAPPED,
    ) -> i32;
    fn CreateEventW(
        sa: *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
        manual: i32,
        initial: i32,
        name: *const u16,
    ) -> HANDLE;
    fn GetOverlappedResult(
        file: HANDLE,
        overlapped: *mut OVERLAPPED,
        count: *mut u32,
        wait: i32,
    ) -> i32;
    fn CancelIoEx(file: HANDLE, overlapped: *const OVERLAPPED) -> i32;
    fn WaitForSingleObject(handle: HANDLE, millis: u32) -> u32;
    fn PeekNamedPipe(
        pipe: HANDLE,
        buffer: *mut u8,
        size: u32,
        read: *mut u32,
        available: *mut u32,
        left: *mut u32,
    ) -> i32;
}
pub struct PrivatePipe {
    handle: std::sync::Arc<OwnedHandle>,
    server: bool,
    name: String,
}
impl PrivatePipe {
    pub fn create(instance: &str) -> io::Result<Self> {
        if instance.len() != 64
            || !instance
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(denied("pipe instance digest shape"));
        }
        let sid = current_identity()?.sid;
        let name = format!(
            "{s}{s}.{s}pipe{s}ocxd-owner024-{sid}-{instance}",
            s = char::from(92)
        );
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let sd = PrivateDescriptor::new(&sid, false)?;
        let sa = sd.attributes();
        let raw = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                3 | 0x40000000 | 0x80000,
                0x8,
                1,
                65540,
                65540,
                0,
                &sa,
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let handle = std::sync::Arc::new(unsafe { OwnedHandle::from_raw_handle(raw) });
        verify_private(raw, &sid, true)?;
        Ok(Self {
            handle,
            server: true,
            name,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn disconnect(&self) -> io::Result<()> {
        if !self.server {
            return Err(denied("only original server may disconnect"));
        }
        check(unsafe { DisconnectNamedPipe(self.handle.as_raw_handle()) })
    }
    pub fn connect_local(name: &str) -> io::Result<Self> {
        use std::os::windows::{fs::OpenOptionsExt, io::IntoRawHandle};
        let prefix = format!("{s}{s}.{s}pipe{s}ocxd-owner024-", s = char::from(92));
        if !name.starts_with(&prefix) || name.len() > 256 {
            return Err(denied("not local owner pipe"));
        }
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0x40000000 | 0x100000)
            .open(name)?;
        Ok(Self {
            handle: std::sync::Arc::new(unsafe {
                OwnedHandle::from_raw_handle(file.into_raw_handle())
            }),
            server: false,
            name: name.into(),
        })
    }
    /// A late kernel cancellation owns its private handle/event/buffers in the
    /// worker. The caller returns at the original deadline, and late results
    /// cannot enter an Owner operation or extend an action's budget.
    fn bounded<T: Send + 'static>(
        &self,
        deadline: Instant,
        operation: impl FnOnce(HANDLE) -> io::Result<T> + Send + 'static,
    ) -> io::Result<T> {
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let handle = self.handle.clone();
        std::thread::Builder::new()
            .name("owner024-private-io".into())
            .spawn(move || {
                let result = if Instant::now() >= deadline {
                    Err(io::ErrorKind::TimedOut.into())
                } else {
                    operation(handle.as_raw_handle())
                };
                let _ = tx.send(result);
            })?;
        let value = rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => {
                    io::Error::from(io::ErrorKind::TimedOut)
                }
                std::sync::mpsc::RecvTimeoutError::Disconnected => {
                    io::Error::other("private IO worker disconnected")
                }
            })??;
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(value)
    }
    pub fn accept(&self, deadline: Instant) -> io::Result<RetainedPeer> {
        if !self.server {
            return Err(denied("client cannot accept"));
        }
        self.bounded(deadline, move |h| {
            overlapped(h, deadline, |ov| {
                let ok = unsafe { ConnectNamedPipe(h, ov) };
                if ok == 0 && io::Error::last_os_error().raw_os_error() == Some(535) {
                    Ok(Some(0))
                } else {
                    pending(ok)
                }
            })
        })?;
        self.peer()
    }
    pub fn peer(&self) -> io::Result<RetainedPeer> {
        let mut pid = 0;
        check(unsafe {
            if self.server {
                GetNamedPipeClientProcessId(self.handle.as_raw_handle(), &mut pid)
            } else {
                GetNamedPipeServerProcessId(self.handle.as_raw_handle(), &mut pid)
            }
        })?;
        if pid == 0 {
            return Err(denied("pipe peer missing"));
        }
        RetainedPeer::from_native_pid(pid)
    }
    fn read_exact(&self, buffer: &mut [u8], deadline: Instant) -> io::Result<()> {
        let mut done = 0;
        let mut chunks = 0;
        while done < buffer.len() {
            chunks += 1;
            if chunks > 256 {
                return Err(denied("fragment count bound"));
            }
            let size = buffer.len() - done;
            let chunk = self.bounded(deadline, move |h| {
                let mut data = vec![0u8; size];
                let read = overlapped(h, deadline, |ov| {
                    let mut n = 0;
                    let ok = unsafe { ReadFile(h, data.as_mut_ptr(), size as u32, &mut n, ov) };
                    if ok != 0 {
                        Ok(Some(n))
                    } else {
                        pending(ok)
                    }
                })? as usize;
                if read == 0 || read > data.len() {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
                data.truncate(read);
                Ok(data)
            })?;
            buffer[done..done + chunk.len()].copy_from_slice(&chunk);
            done += chunk.len();
        }
        Ok(())
    }
    pub fn read_frame(&self, deadline: Instant) -> io::Result<Vec<u8>> {
        let mut length = [0u8; 4];
        self.read_exact(&mut length, deadline)?;
        let n = u32::from_le_bytes(length) as usize;
        if n == 0 || n > 65536 {
            return Err(denied("private pipe frame bound"));
        }
        let mut frame = vec![0u8; n + 4];
        frame[..4].copy_from_slice(&length);
        self.read_exact(&mut frame[4..], deadline)?;
        self.bounded(deadline, |h| {
            let mut available = 0;
            check(unsafe {
                PeekNamedPipe(
                    h,
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut available,
                    ptr::null_mut(),
                )
            })?;
            if available != 0 {
                return Err(denied("trailing private pipe frame/bytes"));
            }
            Ok(())
        })?;
        Ok(frame)
    }
    pub fn write_frame(&self, payload: &[u8], deadline: Instant) -> io::Result<()> {
        if payload.is_empty() || payload.len() > 65536 {
            return Err(denied("private pipe frame bound"));
        }
        let mut b = (payload.len() as u32).to_le_bytes().to_vec();
        b.extend_from_slice(payload);
        self.bounded(deadline, move |h| {
            let mut done = 0;
            let mut chunks = 0;
            while done < b.len() {
                chunks += 1;
                if chunks > 256 {
                    return Err(denied("write fragmentation bound"));
                }
                let n = overlapped(h, deadline, |ov| {
                    let mut n = 0;
                    let ok = unsafe {
                        WriteFile(h, b[done..].as_ptr(), (b.len() - done) as u32, &mut n, ov)
                    };
                    if ok != 0 {
                        Ok(Some(n))
                    } else {
                        pending(ok)
                    }
                })? as usize;
                if n == 0 || n > b.len() - done {
                    return Err(io::ErrorKind::WriteZero.into());
                }
                done += n;
            }
            Ok(())
        })
    }
}
fn overlapped(
    handle: HANDLE,
    deadline: Instant,
    operation: impl FnOnce(*mut OVERLAPPED) -> io::Result<Option<u32>>,
) -> io::Result<u32> {
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    let event = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
    if event.is_null() {
        return Err(io::Error::last_os_error());
    }
    let _event = unsafe { OwnedHandle::from_raw_handle(event) };
    let mut ov = OVERLAPPED::default();
    ov.hEvent = event;
    if let Some(n) = operation(&mut ov)? {
        return if Instant::now() < deadline {
            Ok(n)
        } else {
            Err(io::ErrorKind::TimedOut.into())
        };
    }
    let millis = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(u32::MAX as u128 - 1) as u32;
    let wait = unsafe { WaitForSingleObject(event, millis) };
    let mut n = 0;
    if wait != 0 {
        // Only this worker waits for cancellation completion; its original
        // event/OVERLAPPED/buffer remain valid. No owner/Job lives on this worker.
        unsafe {
            CancelIoEx(handle, &ov);
            GetOverlappedResult(handle, &mut ov, &mut n, 1);
        }
        return Err(if wait == 258 {
            io::ErrorKind::TimedOut.into()
        } else {
            io::Error::last_os_error()
        });
    }
    check(unsafe { GetOverlappedResult(handle, &mut ov, &mut n, 0) })?;
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(n)
}
fn pending(ok: i32) -> io::Result<Option<u32>> {
    if ok != 0 {
        Ok(Some(0))
    } else {
        let e = io::Error::last_os_error();
        if e.raw_os_error() == Some(997) {
            Ok(None)
        } else {
            Err(e)
        }
    }
}
