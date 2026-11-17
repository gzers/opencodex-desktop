//! Original-slot full history reads. No deadline is created here. The original
//! successful validation and all returned native objects stay in this control.
use super::*;
fn caller_read_end(end: Instant) -> io::Result<()> {
    let now = Instant::now();
    if now >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    if end.duration_since(now) > std::time::Duration::from_secs(5) {
        return Err(invalid());
    }
    Ok(())
}
impl<T: Original> WorkerControl<T> {
    fn read_returned_history<B, F>(
        &self,
        end: Instant,
        mut binding: B,
        read: F,
    ) -> io::Result<serde_json::Value>
    where
        B: FnMut(&mut T, &BeforeResumeRecoveryWindow, Instant) -> io::Result<TransferBinding>,
        F: FnOnce(
            &mut T,
            &mut ReleasedLease,
            &BeforeResumeRecoveryWindow,
            Instant,
        ) -> io::Result<serde_json::Value>,
    {
        caller_read_end(end)?;
        let mut retained = self.validation.try_lock().map_err(|_| invalid())?;
        if !retained.attempted
            || !retained.complete
            || self.state.try_lock().map_err(|_| invalid())?.phase != Phase::ReturnedValidated
        {
            return Err(invalid());
        }
        let completed = retained.completed_at.ok_or_else(invalid)?;
        let value = retained.original.as_mut().ok_or_else(invalid)?;
        let result = (|| {
            if completed >= value.window.deadline() || completed > Instant::now() {
                return Err(invalid());
            }
            let before = binding(&mut value.original, &value.window, end)?;
            if before != self.state.try_lock().map_err(|_| invalid())?.binding {
                return Err(invalid());
            }
            value.released.readback_history(end)?;
            let data = read(&mut value.original, &mut value.released, &value.window, end)?;
            value.released.readback_history(end)?;
            let after = binding(&mut value.original, &value.window, end)?;
            if before != after || after != self.state.try_lock().map_err(|_| invalid())?.binding {
                return Err(invalid());
            }
            caller_read_end(end)?;
            Ok(data)
        })();
        if let Err(error) = &result {
            retained.complete = false;
            retained.original.as_mut().unwrap().original.revoke();
            if let Ok(mut state) = self.state.lock() {
                state.phase = Phase::HistoryReadFailedRetained;
                state.raw = error.raw_os_error();
                state.kind = Some(format!("{:?}", error.kind()));
            }
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024::recovery_worker) fn read_returned_history_for_test<
        B,
        F,
    >(
        &self,
        end: Instant,
        binding: B,
        read: F,
    ) -> io::Result<serde_json::Value>
    where
        B: FnMut(&mut T, &BeforeResumeRecoveryWindow, Instant) -> io::Result<TransferBinding>,
        F: FnOnce(
            &mut T,
            &mut ReleasedLease,
            &BeforeResumeRecoveryWindow,
            Instant,
        ) -> io::Result<serde_json::Value>,
    {
        self.read_returned_history(end, binding, read)
    }
}
impl WorkerControl<Owner> {
    pub(crate) fn read_original_owner_history<F>(
        &self,
        end: Instant,
        slot_check: F,
    ) -> io::Result<serde_json::Value>
    where
        F: FnOnce(&Owner) -> crate::errors::AppResult<()>,
    {
        self.read_returned_history(end,|owner,window,read_end| {
            let [session,action,scope]=owner.before_resume_history_binding(window,read_end).map_err(io::Error::other)?;
            Ok(TransferBinding{session,action,scope,end:window.deadline()})
        },|owner,released,window,read_end| {
            slot_check(owner).map_err(io::Error::other)?;
            let expected=owner.before_resume_restored_history_data(window,read_end).map_err(io::Error::other)?;
            let actual=owner.with_originals(|originals,owner|originals.read_retained_history(owner,released,window,read_end)).map_err(io::Error::other)?;
            if actual!=expected || owner.before_resume_restored_history_data(window,read_end).map_err(io::Error::other)? != expected {return Err(invalid());}
            Ok(serde_json::json!({"schema":"windows-before-resume-native-history-v1","history":actual,"readOnly":true,"recoveryRequired":true,"normalStop":false,"closed":false,"publicStartOpen":false,"runtimeAuthority":false}))
        })
    }
}
