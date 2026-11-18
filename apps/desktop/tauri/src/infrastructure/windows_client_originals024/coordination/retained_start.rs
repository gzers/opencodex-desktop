use super::*;
use crate::infrastructure::windows_official_lifecycle024::BeforeResumeLeaseBinding;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
use std::thread;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StartLeaseState {
    Other,
    Start((Uuid, Uuid)),
    Failed((Uuid, Uuid)),
    Recovering,
    Spent,
}

pub(in crate::infrastructure::windows_client_originals024) struct RecoveryPermit {
    epoch: String,
    thread: thread::ThreadId,
    origin: BeforeResumeLeaseBinding,
    pub(super) end: Instant,
    _not_send: PhantomData<*mut ()>,
}

impl NativeLease {
    pub(in crate::infrastructure::windows_client_originals024) fn inspect_failed_start(
        &mut self,
        owner: &mut Owner,
        origin: &BeforeResumeLeaseBinding,
        end: Instant,
    ) -> io::Result<super::segments::LeaseFacts> {
        before(end)?;
        if self.active
            || self.start_state != StartLeaseState::Failed((origin.owner, origin.failed_start))
            || origin.failed_start == origin.recovery
            || origin.start_deadline != self.deadline
        {
            return Err(refused());
        }
        self.check_bound_owner(owner, end)?;
        self.verify_retained_objects(end)?;
        let facts = self.facts_inner()?;
        before(end)?;
        Ok(facts)
    }

    pub(in crate::infrastructure::windows_client_originals024) fn retain_failed_start(&mut self) {
        self.active = false;
        self.start_state = match self.start_state {
            StartLeaseState::Start(binding) => StartLeaseState::Failed(binding),
            _ => StartLeaseState::Spent,
        };
    }

    pub(in crate::infrastructure::windows_client_originals024) fn authorize_retained_recovery(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<RecoveryPermit> {
        let result = (|| {
            let origin = owner
                .before_resume_lease_binding(window)
                .map_err(io_error)?;
            let permit = self.begin_recovery(origin, window.deadline())?;
            self.check_recovery_owner(&permit, owner, window)?;
            Ok(permit)
        })();
        if result.is_err() {
            self.revoke_retained();
        }
        result
    }

    pub(super) fn begin_recovery(
        &mut self,
        origin: BeforeResumeLeaseBinding,
        end: Instant,
    ) -> io::Result<RecoveryPermit> {
        let state = self.start_state;
        if self.active || !matches!(state, StartLeaseState::Failed(_)) {
            return Err(refused());
        }
        // Consume the attempt before the first fallible inspection.
        self.start_state = StartLeaseState::Spent;
        if state != StartLeaseState::Failed((origin.owner, origin.failed_start))
            || origin.failed_start == origin.recovery
            || origin.start_deadline != self.deadline
        {
            return Err(refused());
        }
        before(end)?;
        self.verify_retained_objects(end)?;
        self.start_state = StartLeaseState::Recovering;
        Ok(RecoveryPermit {
            epoch: self.epoch.clone(),
            thread: self.acquisition_thread,
            origin,
            end,
            _not_send: PhantomData,
        })
    }

    pub(super) fn check_recovery(
        &mut self,
        permit: &RecoveryPermit,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.active
                || self.start_state != StartLeaseState::Recovering
                || permit.epoch != self.epoch
                || permit.thread != thread::current().id()
                || permit.origin.start_deadline != self.deadline
                || end > permit.end
            {
                return Err(refused());
            }
            before(end)?;
            self.verify_retained_objects(end)
        })();
        if result.is_err() {
            self.revoke_retained();
        }
        result
    }

    pub(in crate::infrastructure::windows_client_originals024) fn check_recovery_owner(
        &mut self,
        permit: &RecoveryPermit,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        let result = (|| {
            let end = window.deadline();
            self.check_recovery(permit, end)?;
            if end != permit.end
                || owner
                    .before_resume_lease_binding(window)
                    .map_err(io_error)?
                    != permit.origin
            {
                return Err(refused());
            }
            self.check_bound_owner(owner, end)?;
            self.check_recovery(permit, end)
        })();
        if result.is_err() {
            self.revoke_retained();
        }
        result
    }

    pub(in crate::infrastructure::windows_client_originals024) fn recovery_claim<'a>(
        &'a mut self,
        permit: &'a RecoveryPermit,
        root: &Path,
        end: Instant,
    ) -> io::Result<AnchoredClaim<'a>> {
        self.check_recovery(permit, end)?;
        self.borrow_claim(root, end, Some(permit))
    }
}
