//! Native provenance wrapper for incomplete preparation DATA. It is not a
//! PreparedAbort/claim/restoration proof. No serde/Clone or disk constructor.
use super::*;
use crate::infrastructure::windows_client_originals024::{
    before_resume_record_v2::{RecordV2, ReferenceV2},
    bounded::Policy,
};
use crate::{
    errors::AppResult, modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
struct OriginalPreparationData {
    session: uuid::Uuid,
    host: uuid::Uuid,
    owner_session: String,
    scope: String,
    action: String,
    failed_action: String,
    end: Instant,
    clock: [String; 3],
    reference: ReferenceV2,
}
impl PartialCaptureAnchor {
    pub(super) fn check_original_capture_metadata(
        &self,
        session: uuid::Uuid,
        host: uuid::Uuid,
        scope: &str,
        end: Instant,
    ) -> io::Result<()> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        if self.session_id != session
            || self.host_instance_id != host
            || self.owner_scope != scope
            || session.is_nil()
            || host.is_nil()
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn original_read_fence(
        &mut self,
        owner: &mut Owner,
        session: uuid::Uuid,
        window: &BeforeResumeRecoveryWindow,
        #[cfg(test)] ordinal: u8,
    ) -> AppResult<()> {
        owner.check_before_resume_process(window)?;
        self.check_original_capture_metadata(
            session,
            owner.original_host_instance(),
            &owner.scope()?,
            window.deadline(),
        )
        .map_err(refused)?;
        let (failed, _) = owner.before_resume_record_origin(window)?;
        if let Some(source) = &mut self.source {
            #[cfg(test)]
            source.check_preparation(owner, window.deadline(), ordinal, &failed)?;
            #[cfg(not(test))]
            source.check(owner, window.deadline())?;
        }
        if let Some(routes) = &mut self.routes {
            routes
                .declared(owner.home_path(), window.deadline())
                .map_err(refused)?;
        }
        if let Some(context) = &self.context {
            context.check_failed_start_read(owner, &failed, window.deadline())?;
        }
        owner.check_before_resume_process(window)
    }
    fn read_bound_original(
        &mut self,
        owner: &mut Owner,
        session: uuid::Uuid,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<OriginalPreparationData> {
        self.original_read_fence(
            owner,
            session,
            window,
            #[cfg(test)]
            1,
        )?;
        let action = owner.before_resume_action_id(window)?;
        let (failed_action, clock) = owner.before_resume_record_origin(window)?;
        let (first, _) = self.v2_reference_data(window.deadline()).map_err(refused)?;
        self.original_read_fence(
            owner,
            session,
            window,
            #[cfg(test)]
            2,
        )?;
        let (second, _) = self.v2_reference_data(window.deadline()).map_err(refused)?;
        if first != second
            || owner.before_resume_action_id(window)? != action
            || owner.before_resume_record_origin(window)? != (failed_action.clone(), clock.clone())
        {
            return Err(refused("原preparation/source/action/clock在读回中变化"));
        }
        self.original_read_fence(
            owner,
            session,
            window,
            #[cfg(test)]
            3,
        )?;
        Ok(OriginalPreparationData {
            session,
            host: self.host_instance_id,
            owner_session: owner.before_resume_owner_session(),
            scope: self.owner_scope.clone(),
            action,
            failed_action,
            end: window.deadline(),
            clock,
            reference: second,
        })
    }
}
impl OriginalPreparationData {
    fn initial_record(
        self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<RecordV2> {
        owner.check_before_resume_process(window)?;
        if self.end != window.deadline()
            || self.host != owner.original_host_instance()
            || self.owner_session != owner.before_resume_owner_session()
            || self.scope != owner.scope()?
            || self.action != owner.before_resume_action_id(window)?
            || owner.before_resume_record_origin(window)?
                != (self.failed_action.clone(), self.clock.clone())
        {
            return Err(refused("不是原Owner/window/failedStart绑定的V2data"));
        }
        let record = RecordV2::initial_preparation_data(
            [
                self.session.to_string(),
                self.failed_action,
                self.action,
                Policy::fixed().digest().map_err(refused)?,
            ],
            self.reference,
            self.clock,
        )
        .map_err(refused)?;
        owner.check_before_resume_process(window)?;
        Ok(record)
    }
}
impl CaptureSlot {
    pub(in crate::infrastructure::windows_client_originals024) fn recheck_preparation_reference(
        &mut self,
        owner: &mut Owner,
        expected: &RecordV2,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let anchor = self
            .partial
            .as_mut()
            .ok_or_else(|| refused("original preparation absent"))?;
        owner.check_before_resume_process(window)?;
        anchor
            .check_original_capture_metadata(
                uuid::Uuid::parse_str(&expected.session_id).map_err(refused)?,
                owner.original_host_instance(),
                &owner.scope()?,
                window.deadline(),
            )
            .map_err(refused)?;
        let (actual, _) = anchor
            .v2_reference_data(window.deadline())
            .map_err(refused)?;
        if actual != expected.reference {
            return Err(refused(
                "original preparation reference changed during attachment write",
            ));
        }
        owner.check_before_resume_process(window)
    }

    pub(in crate::infrastructure::windows_client_originals024) fn has_original_preparation(
        &self,
    ) -> bool {
        self.failed && self.partial.is_some() && self.retained.is_none()
    }

    pub(in crate::infrastructure::windows_client_originals024) fn read_bound_preparation_record(
        &mut self,
        owner: &mut Owner,
        session: uuid::Uuid,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<RecordV2> {
        let result = (|| {
            if self.retained.is_some() {
                return Err(refused("完整Store不能回退为preparation variant"));
            }
            self.partial
                .as_mut()
                .ok_or_else(|| refused("原partialanchor缺席"))?
                .read_bound_original(owner, session, window)?
                .initial_record(owner, window)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
}
