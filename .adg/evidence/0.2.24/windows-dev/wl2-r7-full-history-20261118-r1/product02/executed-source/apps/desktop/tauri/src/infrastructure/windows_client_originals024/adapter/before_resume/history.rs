//! Full historical native read only from original completed objects. No old
//! Start/recovery action renewal, range acquisition, record writes or admission.
use super::attachment::BoundAttachment;
use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::ReleasedLease, source::Source, tree::AfterReturnGuard, v2::routes::RoutePin,
    v3::Journal,
};
pub(in crate::infrastructure::windows_client_originals024) struct OriginalHistoricalScope<'a> {
    routes: &'a mut RoutePin,
    source: &'a mut Source,
    owner: &'a mut Owner,
    prefix: &'a PreparedAbortPrefix,
    attachment: &'a mut BoundAttachment,
    window: &'a BeforeResumeRecoveryWindow,
    read_end: Instant,
}
impl<'a> OriginalHistoricalScope<'a> {
    pub(in crate::infrastructure::windows_client_originals024) fn bind(
        routes: &'a mut RoutePin,
        source: &'a mut Source,
        owner: &'a mut Owner,
        prefix: &'a PreparedAbortPrefix,
        proof: &QuiescentOriginals,
        attachment: &'a mut BoundAttachment,
        window: &'a BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<Self> {
        proof.check_tag(prefix)?;
        prefix.check_history_owner(owner, window, read_end)?;
        attachment.check_history(owner, prefix, window, read_end)?;
        Ok(Self {
            routes,
            source,
            owner,
            prefix,
            attachment,
            window,
            read_end,
        })
    }
}
impl AfterReturnGuard for OriginalHistoricalScope<'_> {
    fn check(
        &mut self,
        journal: &mut Journal,
        released: &mut ReleasedLease,
        end: Instant,
    ) -> std::io::Result<()> {
        let result = (|| {
            if end != self.read_end {
                return Err(refused("原历史read截止不符"));
            }
            self.prefix
                .check_history_owner(self.owner, self.window, end)?;
            self.routes
                .declared(self.owner.home_path(), end)
                .map_err(refused)?;
            self.attachment
                .check_history(self.owner, self.prefix, self.window, end)?;
            let actual = journal
                .read_before_resume_released_history(
                    self.source,
                    self.owner,
                    released,
                    &self.prefix.epoch,
                    &self.prefix.claim,
                    self.window,
                    end,
                )
                .map_err(refused)?;
            if actual != self.prefix.observed["canonical"] {
                return Err(refused("原历史canonical变化"));
            }
            self.owner.check_before_resume_history(self.window, end)
        })();
        if result.is_err() {
            self.owner.fail_before_resume_process();
            self.attachment.failed_for_history();
        }
        result.map_err(std::io::Error::other)
    }
    fn record_verified(&mut self, _: u32, _: u32, _: Instant) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "historical native read cannot write completion records",
        ))
    }
}
impl WindowsOriginals {
    pub(crate) fn read_retained_history(
        &mut self,
        owner: &mut Owner,
        released: &mut ReleasedLease,
        window: &BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            let prefix = self
                .abort_prefix
                .as_ref()
                .ok_or_else(|| refused("原历史prefix缺席"))?;
            let proof = self
                .quiescent_for_release
                .as_ref()
                .ok_or_else(|| refused("原历史静止tag缺席"))?;
            proof.check_tag(prefix)?;
            if self.session != Some(prefix.session)
                || self.owner_scope.as_ref() != Some(&owner.scope()?)
            {
                return Err(refused("不是原历史slot"));
            }
            let attachment = self
                .abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原历史attachment缺席"))?;
            let data = self.slot.read_before_resume_history(
                owner, prefix, proof, attachment, released, window, read_end,
            )?;
            Ok(
                serde_json::json!({"sessionId":prefix.session.to_string(),"recoveryActionId":prefix.action,"restored":data,"normalStop":false,"closed":false,"publicStartOpen":false}),
            )
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
}
