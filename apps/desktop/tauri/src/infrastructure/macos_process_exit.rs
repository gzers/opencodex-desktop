//! Watch the exact live process generation before requesting official Stop.
//! Fork/exec after subscription invalidate the original single-process proof.
//! No signal, PID discovery, waitpid/reaping, or disappearance-as-exit-success.
use crate::errors::{AppError, AppResult};
use std::{
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    time::Instant,
};

fn fail(code: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: "未取得原服务进程的正常退出证明，未宣称停止完成；请核对诊断".into(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation {
    pub pid: u32,
    pub uid: u32,
    pub start_seconds: u64,
    pub start_micros: u64,
}
impl Generation {
    pub fn observe(pid: u32) -> AppResult<Self> {
        if pid == 0 || pid > i32::MAX as u32 {
            return Err(fail("lifecycle_exit_identity"));
        }
        let mut value = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
        let n = unsafe {
            libc::proc_pidinfo(
                pid as i32,
                libc::PROC_PIDTBSDINFO,
                0,
                value.as_mut_ptr().cast(),
                size,
            )
        };
        if n != size {
            return Err(fail("lifecycle_exit_identity"));
        }
        let value = unsafe { value.assume_init() };
        if value.pbi_pid != pid
            || value.pbi_uid != unsafe { libc::geteuid() }
            || value.pbi_status == 5
            || value.pbi_start_tvsec == 0
            || value.pbi_start_tvusec >= 1_000_000
        {
            return Err(fail("lifecycle_exit_identity"));
        }
        Ok(Self {
            pid,
            uid: value.pbi_uid,
            start_seconds: value.pbi_start_tvsec,
            start_micros: value.pbi_start_tvusec,
        })
    }
}
pub fn parent_pid(pid: u32) -> AppResult<u32> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(fail("lifecycle_exit_identity"));
    }
    let mut value = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
    let n = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            value.as_mut_ptr().cast(),
            size,
        )
    };
    if n != size {
        return Err(fail("lifecycle_exit_identity"));
    }
    let value = unsafe { value.assume_init() };
    if value.pbi_pid != pid || value.pbi_ppid == 0 {
        return Err(fail("lifecycle_exit_identity"));
    }
    Ok(value.pbi_ppid)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Exit {
    Normal(u8),
    Signal(u8),
}
fn decode_wait(status: i64) -> AppResult<Exit> {
    let raw = i32::try_from(status).map_err(|_| fail("lifecycle_exit_event_invalid"))?;
    if !(0..=65535).contains(&raw) {
        return Err(fail("lifecycle_exit_event_invalid"));
    }
    if libc::WIFEXITED(raw) {
        Ok(Exit::Normal(libc::WEXITSTATUS(raw) as u8))
    } else if libc::WIFSIGNALED(raw) {
        Ok(Exit::Signal(libc::WTERMSIG(raw) as u8))
    } else {
        Err(fail("lifecycle_exit_event_invalid"))
    }
}

/// Only a live registered native event may construct this proof. It covers one
/// process, not its descendants, ports, HOME, client restore or overall budget.
#[derive(Clone, Debug)]
pub struct NormalExitProof {
    generation: Generation,
}
impl NormalExitProof {
    pub fn generation(&self) -> Generation {
        self.generation
    }
}
pub struct ExitWatch {
    queue: OwnedFd,
    generation: Generation,
    cookie: u64,
    observed: Option<Exit>,
    invalidated: Option<&'static str>,
}
impl ExitWatch {
    pub fn capture(expected: Generation) -> AppResult<Self> {
        if expected.pid == std::process::id()
            || expected.uid != unsafe { libc::geteuid() }
            || Generation::observe(expected.pid)? != expected
        {
            return Err(fail("lifecycle_exit_identity"));
        }
        let fd = unsafe { libc::kqueue() };
        if fd < 0 {
            return Err(fail("lifecycle_exit_subscribe"));
        }
        let queue = unsafe { OwnedFd::from_raw_fd(fd) };
        if unsafe { libc::fcntl(queue.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } != 0 {
            return Err(fail("lifecycle_exit_subscribe"));
        }
        let cookie = loop {
            let candidate = rand::random::<u64>();
            if candidate != 0 {
                break candidate;
            }
        };
        let registration = libc::kevent64_s {
            ident: expected.pid.into(),
            filter: libc::EVFILT_PROC,
            flags: libc::EV_ADD | libc::EV_ENABLE | libc::EV_CLEAR | libc::EV_RECEIPT,
            fflags: libc::NOTE_EXIT | libc::NOTE_EXITSTATUS | libc::NOTE_FORK | libc::NOTE_EXEC,
            data: 0,
            udata: cookie,
            ext: [0, 0],
        };
        let mut receipt = std::mem::MaybeUninit::<libc::kevent64_s>::uninit();
        let zero = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let n = unsafe {
            libc::kevent64(
                queue.as_raw_fd(),
                &registration,
                1,
                receipt.as_mut_ptr(),
                1,
                0,
                &zero,
            )
        };
        if n != 1 {
            return Err(fail("lifecycle_exit_subscribe"));
        }
        let receipt = unsafe { receipt.assume_init() };
        if receipt.ident != expected.pid as u64
            || receipt.filter != libc::EVFILT_PROC
            || receipt.udata != cookie
            || receipt.flags & libc::EV_ERROR == 0
            || receipt.data != 0
        {
            return Err(fail("lifecycle_exit_subscribe"));
        }
        // A PID reuse or an exit racing registration is uncertainty. Never open
        // a new generation or infer normal exit because this PID vanished.
        if Generation::observe(expected.pid)? != expected {
            return Err(fail("lifecycle_exit_identity"));
        }
        Ok(Self {
            queue,
            generation: expected,
            cookie,
            observed: None,
            invalidated: None,
        })
    }
    pub fn normal_exit(&mut self, deadline: Instant) -> AppResult<Option<NormalExitProof>> {
        if Instant::now() >= deadline {
            return Err(fail("lifecycle_timeout"));
        }
        if let Some(code) = self.invalidated {
            return Err(fail(code));
        }
        if self.observed.is_none() {
            let mut event = std::mem::MaybeUninit::<libc::kevent64_s>::uninit();
            let zero = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            let n = unsafe {
                libc::kevent64(
                    self.queue.as_raw_fd(),
                    std::ptr::null(),
                    0,
                    event.as_mut_ptr(),
                    1,
                    0,
                    &zero,
                )
            };
            if n < 0 {
                return Err(fail("lifecycle_exit_observation"));
            }
            if n == 0 {
                return Ok(None);
            }
            if n != 1 {
                return Err(fail("lifecycle_exit_event_invalid"));
            }
            let event = unsafe { event.assume_init() };
            if event.ident != self.generation.pid as u64
                || event.udata != self.cookie
                || event.filter != libc::EVFILT_PROC
                || event.flags & libc::EV_ERROR != 0
            {
                return Err(fail("lifecycle_exit_event_invalid"));
            }
            // These flags can coalesce with NOTE_EXIT. Check them before the
            // exit status, and retain refusal after the event has been read.
            // Darwin cannot follow child PIDs via NOTE_TRACK/NOTE_CHILD; a fork
            // is uncertainty, never an invitation to invent a complete family.
            let changed = if event.fflags & libc::NOTE_FORK != 0 {
                Some("lifecycle_exit_membership_changed")
            } else if event.fflags & libc::NOTE_EXEC != 0 {
                Some("lifecycle_exit_image_changed")
            } else {
                None
            };
            if let Some(code) = changed {
                self.invalidated = Some(code);
                return Err(fail(code));
            }
            if event.flags & libc::EV_EOF == 0
                || event.fflags & (libc::NOTE_EXIT | libc::NOTE_EXITSTATUS)
                    != (libc::NOTE_EXIT | libc::NOTE_EXITSTATUS)
            {
                return Err(fail("lifecycle_exit_event_invalid"));
            }
            self.observed = Some(decode_wait(event.data)?);
        }
        if Instant::now() >= deadline {
            return Err(fail("lifecycle_timeout"));
        }
        match self.observed {
            Some(Exit::Normal(0)) => Ok(Some(NormalExitProof {
                generation: self.generation,
            })),
            Some(Exit::Normal(_)) => Err(fail("lifecycle_exit_nonzero")),
            Some(Exit::Signal(_)) => Err(fail("lifecycle_exit_signalled")),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[path = "macos_process_exit_tests.rs"]
mod tests;
