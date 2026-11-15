//! Original retained peer handles; PID values from requests never enter here.
use super::{
    process,
    security::{check, current_identity, denied},
    ProcessIdentity,
};
use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    ptr,
};
use windows_sys::Win32::System::{
    JobObjects::IsProcessInJob,
    Threading::{
        GetCurrentProcess, GetProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_SYNCHRONIZE,
    },
};

pub struct RetainedPeer {
    handle: OwnedHandle,
    identity: ProcessIdentity,
}
impl RetainedPeer {
    pub(super) fn from_original_handle(handle: OwnedHandle) -> io::Result<Self> {
        let identity = process::observe(handle.as_raw_handle())?;
        if identity.token.sid != current_identity()?.sid {
            return Err(denied("original peer SID differs"));
        }
        Ok(Self { handle, identity })
    }
    pub fn current() -> io::Result<Self> {
        Self::from_native_pid(unsafe { GetProcessId(GetCurrentProcess()) })
    }
    pub(super) fn from_native_pid(pid: u32) -> io::Result<Self> {
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
        let identity = process::observe(raw)?;
        if identity.token.sid != current_identity()?.sid {
            return Err(denied("native peer SID mismatch"));
        }
        Ok(Self { handle, identity })
    }
    pub fn identity(&self) -> &ProcessIdentity {
        &self.identity
    }
    pub fn exited(&self) -> io::Result<bool> {
        Ok(matches!(
            process::state(self.handle.as_raw_handle())?,
            super::ProcessState::Exited { .. }
        ))
    }
    pub fn verify(&self) -> io::Result<()> {
        if process::observe(self.handle.as_raw_handle())? != self.identity
            || process::state(self.handle.as_raw_handle())? != super::ProcessState::Running
        {
            return Err(denied(
                "retained peer generation/image/token exited or drifted",
            ));
        }
        Ok(())
    }
    pub fn verify_image(&self, image: &Path) -> io::Result<()> {
        self.verify()?;
        let f = std::fs::OpenOptions::new()
            .read(true)
            .share_mode_native()
            .open(image)?;
        if super::paths::identity(&f, false)? != self.identity.executable_id {
            return Err(denied("peer image differs from exact current"));
        }
        Ok(())
    }
    pub fn no_outer_job(&self) -> io::Result<()> {
        self.verify()?;
        let mut inside = 0;
        check(unsafe {
            IsProcessInJob(self.handle.as_raw_handle(), ptr::null_mut(), &mut inside)
        })?;
        if inside != 0 {
            return Err(denied("original process belongs to unproven outer Job"));
        }
        Ok(())
    }
}
trait NativeOpen {
    fn share_mode_native(&mut self) -> &mut Self;
}
impl NativeOpen for std::fs::OpenOptions {
    fn share_mode_native(&mut self) -> &mut Self {
        use std::os::windows::fs::OpenOptionsExt;
        self.share_mode(1).custom_flags(0x200000)
    }
}
