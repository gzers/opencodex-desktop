//! One original-slot validation, after actual released resources return.
//! All originals are retained here before a fallible check. No native release
//! probe/range is acquired on the receiving thread or across a new deadline.
use super::*;
mod history;
fn publish_validation(state: &Arc<Mutex<State>>, phase: Phase, error: Option<&io::Error>) {
    if let Ok(mut state) = state.lock() {
        state.phase = phase;
        state.validator_thread = Some(format!("{:?}", thread::current().id()));
        state.raw = error.and_then(|e| e.raw_os_error());
        state.kind = error.map(|e| format!("{:?}", e.kind()));
    }
}
pub(super) struct Validation<T> {
    pub(super) original: Option<Returned<T>>,
    pub(super) attempted: bool,
    pub(super) complete: bool,
    pub(super) completed_at: Option<Instant>,
}
impl<T> Validation<T> {
    pub(super) fn new() -> Self {
        Self {
            original: None,
            attempted: false,
            complete: false,
            completed_at: None,
        }
    }
}
impl<T: Original> WorkerControl<T> {
    #[cfg(test)]
    pub(super) fn validate_returned_for_test<F>(
        &self,
        end: Instant,
        check: F,
    ) -> io::Result<Option<serde_json::Value>>
    where
        F: FnOnce(
            &mut T,
            &mut ReleasedLease,
            &BeforeResumeRecoveryWindow,
        ) -> io::Result<serde_json::Value>,
    {
        self.validate_returned(end, check)
    }
    fn validate_returned<F>(&self, end: Instant, check: F) -> io::Result<Option<serde_json::Value>>
    where
        F: FnOnce(
            &mut T,
            &mut ReleasedLease,
            &BeforeResumeRecoveryWindow,
        ) -> io::Result<serde_json::Value>,
    {
        let mut retained = self.validation.try_lock().map_err(|_| invalid())?;
        if retained.attempted {
            return Err(invalid());
        }
        {
            let state = self.state.try_lock().map_err(|_| invalid())?;
            if state.phase != Phase::ReturnedForValidation {
                return Ok(None);
            }
        }
        if retained.original.is_none() {
            match self.returned.try_lock().map_err(|_| invalid())?.try_recv() {
                Ok(original) => retained.original = Some(original),
                Err(mpsc::TryRecvError::Empty) => return Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(io::ErrorKind::BrokenPipe.into())
                }
            }
        }
        retained.attempted = true;
        let value = retained.original.as_mut().unwrap();
        let result = (|| {
            if end != value.window.deadline() {
                return Err(invalid());
            }
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let before = value.original.binding(&value.window)?;
            if self.state.try_lock().map_err(|_| invalid())?.binding.as_ref() != Some(&before) {
                return Err(invalid());
            }
            value.released.check(end)?;
            let data = check(&mut value.original, &mut value.released, &value.window)?;
            value.released.check(end)?;
            let after = value.original.binding(&value.window)?;
            if before != after || self.state.try_lock().map_err(|_| invalid())?.binding.as_ref() != Some(&after) {
                return Err(invalid());
            }
            let completed = Instant::now();
            if completed >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            Ok((data, completed))
        })();
        match result {
            Ok((data, completed)) => {
                retained.complete = true;
                retained.completed_at = Some(completed);
                publish_validation(&self.state, Phase::ReturnedValidated, None);
                Ok(Some(data))
            }
            Err(error) => {
                retained.original.as_mut().unwrap().original.revoke();
                publish_validation(
                    &self.state,
                    Phase::ReturnValidationFailedRetained,
                    Some(&error),
                );
                Err(error)
            }
        }
    }
}
impl WorkerControl<Owner> {
    pub(crate) fn validate_original_owner_return<F>(
        &self,
        end: Instant,
        slot_check: F,
    ) -> io::Result<Option<serde_json::Value>>
    where
        F: FnOnce(&Owner) -> crate::errors::AppResult<()>,
    {
        self.validate_returned(end, |owner, released, window| {
            slot_check(owner).map_err(io::Error::other)?;
            let expected = owner
                .before_resume_restored_data(window)
                .map_err(io::Error::other)?;
            let actual = owner
                .with_originals(|originals, owner| {
                    originals.recheck_retained_quiescent(owner, released, window)
                })
                .map_err(io::Error::other)?;
            if actual != expected
                || owner
                    .before_resume_restored_data(window)
                    .map_err(io::Error::other)?
                    != expected
            {
                return Err(invalid());
            }
            Ok(actual)
        })
    }
}
