//! Original creator/primary-thread/Job half of explicit r7 failure recovery.
//! It cannot restore clients, create Stop/Closed or adopt an external process.
use super::*;
use windows_sys::Win32::System::Threading::{GetProcessIdOfThread, GetThreadId};
#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryThreadCycleTime(
        thread: windows_sys::Win32::Foundation::HANDLE,
        cycles: *mut u64,
    ) -> i32;
}
pub(super) fn thread_cycles(thread: windows_sys::Win32::Foundation::HANDLE) -> io::Result<u64> {
    let mut cycles = 0;
    check(unsafe { QueryThreadCycleTime(thread, &mut cycles) })?;
    Ok(cycles)
}

/// Same original creation and exact primary thread, fixed fresh deadline.
/// Private/nonClone/nonserde: disk/IPC fields cannot construct this proof.
pub struct PreparedLauncherAbort {
    binding: RootBinding,
    thread_id: u32,
    deadline: Instant,
}
/// Exact original Job emptied by its original creator. Forced/raw242 is kept
/// distinct from NativeNormalExit. No client or successor eligibility exists.
pub struct QuiescentLauncher {
    binding: RootBinding,
    deadline: Instant,
    raw_exit: u32,
    total: u32,
}
fn within(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
impl OwnedRuntime {
    fn original_primary_unexecuted(&self, end: Instant) -> io::Result<u32> {
        within(end)?;
        if self.resumed || self.forced_cleanup_requested {
            return Err(denied(
                "original Resume/forced history disqualifies beforeResume branch",
            ));
        }
        let thread = self
            .thread
            .as_ref()
            .ok_or_else(|| denied("original primary creation handle missing"))?;
        let id = unsafe { GetThreadId(thread.as_raw_handle()) };
        let pid = unsafe { GetProcessIdOfThread(thread.as_raw_handle()) };
        if id == 0 || pid == 0 {
            return Err(io::Error::last_os_error());
        }
        if pid != self.binding.process.pid
            || unsafe { WaitForSingleObject(thread.as_raw_handle(), 0) } != WAIT_TIMEOUT
        {
            return Err(denied(
                "exact primary thread identity/execution state differs",
            ));
        }
        // Query the same retained creation thread. Preserve its original
        // CREATE_SUSPENDED baseline, including OS initialization cycles. Any
        // later accounted execution is a refusal; never Suspend/Resume to probe.
        let cycles = thread_cycles(thread.as_raw_handle())?;
        if cycles != self.primary_cycles_at_creation {
            return Err(denied(
                "original primary cycles changed since original suspended creation; never-resumed proof unknown",
            ));
        }
        within(end)?;
        Ok(id)
    }
    /// The caller supplies a fresh explicit recovery window, not an expired
    /// Start origin. Revocation is monotonic even if any subsequent check fails.
    pub fn prepare_before_first_resume_abort(
        &mut self,
        started: Instant,
        parent_end: Instant,
    ) -> io::Result<PreparedLauncherAbort> {
        if self.abort_blocked || self.abort_consumed {
            return Err(denied(
                "original abort already attempted; no automatic new window",
            ));
        }
        self.abort_blocked = true;
        let now = Instant::now();
        let deadline = parent_end.min(
            started
                .checked_add(Duration::from_secs(10))
                .ok_or_else(|| denied("abort deadline overflow"))?,
        );
        if started > now || now >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        self.check_creator_not_resumed(deadline)?;
        let thread_id = self.original_primary_unexecuted(deadline)?;
        self.verify_binding(&self.binding)?;
        within(deadline)?;
        Ok(PreparedLauncherAbort {
            binding: self.binding.clone(),
            thread_id,
            deadline,
        })
    }
    /// One attempt, only through the actual original Job. A mismatched proof
    /// revokes this owner and stops; it never becomes an external PID fallback.
    pub fn quiesce_before_first_resume(
        &mut self,
        prepared: PreparedLauncherAbort,
    ) -> io::Result<QuiescentLauncher> {
        if self.abort_consumed {
            return Err(denied("original abort quiescence already attempted"));
        }
        self.abort_consumed = true;
        self.abort_blocked = true;
        within(prepared.deadline)?;
        if prepared.binding != self.binding {
            return Err(denied("abort proof is not this original creation/Job"));
        }
        self.check_creator_not_resumed(prepared.deadline)?;
        if self.original_primary_unexecuted(prepared.deadline)? != prepared.thread_id {
            return Err(denied("abort original primary thread changed"));
        }
        let forced = self.force_owned_cleanup(prepared.deadline)?;
        if !forced.forced
            || forced.final_state.total_processes != 1
            || forced.final_state.active_processes != 0
            || !forced.final_state.active_members.is_empty()
            || !forced.final_state.retained_members.is_empty()
            || forced.final_state.root != (ProcessState::Exited { code: 242 })
        {
            return Err(denied("original abort Job/root raw exit/history differs"));
        }
        let proof = QuiescentLauncher {
            binding: self.binding.clone(),
            deadline: prepared.deadline,
            raw_exit: 242,
            total: 1,
        };
        self.verify_before_resume_quiescent(&proof)?;
        Ok(proof)
    }
    /// Read the same retained preparation without consuming it, changing its
    /// cutoff or authorizing any force/client mutation. This permits the
    /// original host to validate its retained partial store before quiescence.
    pub fn verify_prepared_before_first_resume(
        &mut self,
        proof: &PreparedLauncherAbort,
    ) -> io::Result<()> {
        within(proof.deadline)?;
        if !self.abort_blocked || self.abort_consumed || proof.binding != self.binding {
            return Err(denied(
                "prepared abort is not this retained pending creation",
            ));
        }
        self.check_creator_not_resumed(proof.deadline)?;
        if self.original_primary_unexecuted(proof.deadline)? != proof.thread_id {
            return Err(denied("prepared abort original primary thread differs"));
        }
        within(proof.deadline)
    }
    pub fn verify_before_resume_quiescent(&mut self, proof: &QuiescentLauncher) -> io::Result<()> {
        within(proof.deadline)?;
        if !self.abort_blocked
            || !self.abort_consumed
            || self.resumed
            || !self.forced_cleanup_requested
            || self.binding != proof.binding
            || proof.raw_exit != 242
            || proof.total != 1
        {
            return Err(denied("quiescent proof is not this original failed abort"));
        }
        self.verify_binding(&proof.binding)?;
        let family = self.family_state()?;
        if family.total_processes != 1
            || family.active_processes != 0
            || !family.active_members.is_empty()
            || !family.retained_members.is_empty()
            || family.root != (ProcessState::Exited { code: 242 })
        {
            return Err(denied("original Job no longer exactly quiescent"));
        }
        within(proof.deadline)
    }
    pub fn resume_revoked_by_abort(&self) -> bool {
        self.abort_blocked
    }
}
impl PreparedLauncherAbort {
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}
impl QuiescentLauncher {
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn forced_raw_exit(&self) -> u32 {
        self.raw_exit
    }
}
