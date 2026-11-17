//! Original typed producer of v1 attachment DATA. Native capabilities remain
//! solely in original Owner/slot/claim; reading an attachment cannot create one.
use super::*;
use crate::infrastructure::{
    windows_client_originals024::before_resume_record::{AttachmentStorage, Event, Record},
    windows_official_runtime024::PinnedDirectory,
};
pub(in crate::infrastructure::windows_client_originals024) struct BoundAttachment {
    storage: AttachmentStorage,
    pending_directory: Option<PinnedDirectory>,
    last: Option<Record>,
    last_raw: Option<Vec<u8>>,
    prefix_digest: String,
    attempted: bool,
    failed: bool,
}
pub(crate) struct OriginalReturnScope<'a> {
    routes: &'a mut crate::infrastructure::windows_client_originals024::v2::routes::RoutePin,
    source: &'a mut crate::infrastructure::windows_client_originals024::source::Source,
    owner: &'a mut Owner,
    prefix: &'a PreparedAbortPrefix,
    attachment: &'a mut BoundAttachment,
    window: &'a BeforeResumeRecoveryWindow,
}
pub(crate) struct OriginalReleasedScope<'a> {
    routes: &'a mut crate::infrastructure::windows_client_originals024::v2::routes::RoutePin,
    source: &'a mut crate::infrastructure::windows_client_originals024::source::Source,
    owner: &'a mut Owner,
    prefix: &'a PreparedAbortPrefix,
    attachment: &'a mut BoundAttachment,
    window: &'a BeforeResumeRecoveryWindow,
}
impl<'a> OriginalReleasedScope<'a> {
    pub(in crate::infrastructure::windows_client_originals024) fn bind(
        routes: &'a mut crate::infrastructure::windows_client_originals024::v2::routes::RoutePin,
        source: &'a mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &'a mut Owner,
        prefix: &'a PreparedAbortPrefix,
        proof: &QuiescentOriginals,
        attachment: &'a mut BoundAttachment,
        window: &'a BeforeResumeRecoveryWindow,
    ) -> AppResult<Self> {
        proof.check_tag(prefix)?;
        owner.check_before_resume_quiescent(window)?;
        attachment.check(owner, prefix, window)?;
        Ok(Self {
            routes,
            source,
            owner,
            prefix,
            attachment,
            window,
        })
    }
}
impl crate::infrastructure::windows_client_originals024::tree::AfterReturnGuard
    for OriginalReleasedScope<'_>
{
    fn check(
        &mut self,
        journal: &mut crate::infrastructure::windows_client_originals024::v3::Journal,
        released:&mut crate::infrastructure::windows_client_originals024::coordination::ReleasedLease,
        end: Instant,
    ) -> std::io::Result<()> {
        let result = (|| {
            if end != self.window.deadline() {
                return Err(refused("原post-release截止不同"));
            }
            self.owner.check_before_resume_quiescent(self.window)?;
            self.routes
                .declared(self.owner.home_path(), end)
                .map_err(refused)?;
            self.attachment
                .check(self.owner, self.prefix, self.window)?;
            let canonical = journal
                .read_before_resume_released(
                    self.source,
                    self.owner,
                    released,
                    &self.prefix.epoch,
                    &self.prefix.claim,
                    self.window,
                )
                .map_err(refused)?;
            if canonical != self.prefix.observed["canonical"] {
                return Err(refused("原canonical变化"));
            }
            Ok(())
        })();
        if result.is_err() {
            self.owner.fail_before_resume_process();
            self.attachment.failed = true;
        }
        result.map_err(std::io::Error::other)
    }
    fn record_verified(
        &mut self,
        nodes: u32,
        coordination: u32,
        end: Instant,
    ) -> std::io::Result<()> {
        let result = (|| {
            if end != self.window.deadline() {
                return Err(refused("原finalrecord截止不符"));
            }
            self.attachment
                .check(self.owner, self.prefix, self.window)?;
            let record = self
                .attachment
                .last
                .as_ref()
                .unwrap()
                .next_data(
                    self.attachment.last_raw.as_ref().unwrap(),
                    Event::AbortRestoreVerified {
                        original_nodes: nodes,
                        coordination_objects: coordination,
                        normal_stop: false,
                        closed: false,
                        public_start_open: false,
                    },
                )
                .map_err(refused)?;
            self.attachment.last_raw = Some(record.encode_data().map_err(refused)?);
            self.attachment.last = Some(record);
            self.attachment
                .storage
                .append_data(self.attachment.last_raw.as_ref().unwrap(), end)
                .map_err(refused)?;
            self.attachment.check(self.owner, self.prefix, self.window)
        })();
        if result.is_err() {
            self.owner.fail_before_resume_process();
            self.attachment.failed = true;
        }
        result.map_err(std::io::Error::other)
    }
}
impl<'a> OriginalReturnScope<'a> {
    pub(in crate::infrastructure::windows_client_originals024) fn bind(
        routes: &'a mut crate::infrastructure::windows_client_originals024::v2::routes::RoutePin,
        source: &'a mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &'a mut Owner,
        prefix: &'a PreparedAbortPrefix,
        proof: &QuiescentOriginals,
        attachment: &'a mut BoundAttachment,
        window: &'a BeforeResumeRecoveryWindow,
    ) -> AppResult<Self> {
        proof.check_tag(prefix)?;
        owner.check_before_resume_quiescent(window)?;
        attachment.check(owner, prefix, window)?;
        Ok(Self {
            routes,
            source,
            owner,
            prefix,
            attachment,
            window,
        })
    }
}
impl crate::infrastructure::windows_client_originals024::tree::ReturnGuard
    for OriginalReturnScope<'_>
{
    fn check(
        &mut self,
        journal: &mut crate::infrastructure::windows_client_originals024::v3::Journal,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> std::io::Result<()> {
        let result = (|| {
            if end != self.window.deadline()
                || claim.epoch() != self.prefix.epoch
                || claim.key() != self.prefix.claim
            {
                return Err(refused("原return截止/claim不符"));
            }
            self.owner.check_before_resume_quiescent(self.window)?;
            self.routes
                .declared(self.owner.home_path(), end)
                .map_err(refused)?;
            self.attachment
                .check(self.owner, self.prefix, self.window)?;
            let canonical = journal
                .read_before_resume_prefix(self.source, self.owner, claim, self.window)
                .map_err(refused)?;
            if canonical != self.prefix.observed["canonical"] {
                return Err(refused("原canonical变化"));
            }
            Ok(())
        })();
        if result.is_err() {
            self.attachment.failed = true;
            self.owner.fail_before_resume_process();
        }
        result.map_err(std::io::Error::other)
    }
    fn record(
        &mut self,
        step: &crate::infrastructure::windows_client_originals024::tree::ReturnStep,
        verified: bool,
        end: Instant,
    ) -> std::io::Result<()> {
        let result = (|| {
            if end != self.window.deadline() {
                return Err(refused("原return附件截止不符"));
            }
            self.attachment
                .check(self.owner, self.prefix, self.window)?;
            let record = self
                .attachment
                .last
                .as_ref()
                .unwrap()
                .next_data(
                    self.attachment.last_raw.as_ref().unwrap(),
                    step.event(verified),
                )
                .map_err(refused)?;
            self.attachment.last_raw = Some(record.encode_data().map_err(refused)?);
            self.attachment.last = Some(record);
            self.attachment
                .storage
                .append_data(self.attachment.last_raw.as_ref().unwrap(), end)
                .map_err(refused)?;
            self.attachment.check(self.owner, self.prefix, self.window)
        })();
        if result.is_err() {
            self.attachment.failed = true;
            self.owner.fail_before_resume_process();
        }
        result.map_err(std::io::Error::other)
    }
}
fn field(value: &serde_json::Value, name: &str) -> AppResult<String> {
    value[name]
        .as_str()
        .map(String::from)
        .ok_or_else(|| refused(format!("实际原canonical缺{name}")))
}
fn initial_data(
    prefix: &PreparedAbortPrefix,
    failed: String,
    clock: [String; 3],
) -> AppResult<Record> {
    let canonical = &prefix.observed["canonical"];
    Record::initial_data(
        [
            field(canonical, "sessionId")?,
            failed,
            field(canonical, "manifestDigest")?,
            field(canonical, "bindingDigest")?,
            field(canonical, "storagePlanDigest")?,
            field(canonical, "securityPolicyDigest")?,
            prefix.digest.clone(),
            prefix.action.clone(),
        ],
        clock,
    )
    .map_err(refused)
}
impl BoundAttachment {
    fn pending(prefix: &PreparedAbortPrefix, directory: PinnedDirectory) -> Self {
        Self {
            storage: AttachmentStorage::default(),
            pending_directory: Some(directory),
            last: None,
            last_raw: None,
            prefix_digest: prefix.digest.clone(),
            attempted: false,
            failed: true,
        }
    }
    fn initialize(
        &mut self,
        owner: &mut Owner,
        prefix: &PreparedAbortPrefix,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        if self.attempted {
            return Err(refused("原attachment不能重试"));
        }
        self.attempted = true;
        let result = (|| {
            prefix.check_owner(owner, window)?;
            let (failed, clock) = owner.before_resume_record_origin(window)?;
            let record = initial_data(prefix, failed, clock)?;
            // Native storage owns original directory before fallible IO. The
            // producer remains reachable on original WindowsOriginals already.
            self.storage
                .initialize(
                    self.pending_directory
                        .take()
                        .ok_or_else(|| refused("原附件目录pin缺席"))?,
                    window.deadline(),
                )
                .map_err(refused)?;
            let raw = record.encode_data().map_err(refused)?;
            self.last = Some(record);
            self.last_raw = Some(raw);
            self.storage
                .append_data(self.last_raw.as_ref().unwrap(), window.deadline())
                .map_err(refused)?;
            self.failed = false;
            self.check(owner, prefix, window)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(super) fn check(
        &mut self,
        owner: &mut Owner,
        prefix: &PreparedAbortPrefix,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed || self.prefix_digest != prefix.digest {
                return Err(refused("原attachment failed/prefix不同"));
            }
            prefix.check_owner(owner, window)?;
            let (failed, clock) = owner.before_resume_record_origin(window)?;
            let expected = initial_data(prefix, failed, clock)?
                .encode_data()
                .map_err(refused)?;
            let records = self.storage.read_data(window.deadline()).map_err(refused)?;
            if records.first() != Some(&expected) {
                return Err(refused("首条附件不是实际原Owner/canonical/clock绑定"));
            }
            let raw = self
                .last_raw
                .as_ref()
                .ok_or_else(|| refused("原attachment last缺席"))?;
            if records.last() != Some(raw)
                || self
                    .last
                    .as_ref()
                    .ok_or_else(|| refused("原attachment typed缺席"))?
                    .encode_data()
                    .map_err(refused)?
                    != *raw
            {
                return Err(refused("原attachment字节与本地事件不符"));
            }
            prefix.check_owner(owner, window)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(super) fn record_quiescent(
        &mut self,
        owner: &mut Owner,
        prefix: &PreparedAbortPrefix,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let result = (|| {
            self.check(owner, prefix, window)?;
            let (pid, creation, raw_exit) = owner.before_resume_quiescent_record(window)?;
            let event = Event::LauncherQuiesced {
                pid,
                creation_filetime: creation.to_string(),
                forced: true,
                raw_exit,
                job_total: 1,
                job_active: 0,
            };
            let record = self
                .last
                .as_ref()
                .unwrap()
                .next_data(self.last_raw.as_ref().unwrap(), event)
                .map_err(refused)?;
            self.last_raw = Some(record.encode_data().map_err(refused)?);
            self.last = Some(record);
            self.storage
                .append_data(self.last_raw.as_ref().unwrap(), window.deadline())
                .map_err(refused)?;
            self.check(owner, prefix, window)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
impl WindowsOriginals {
    pub(crate) fn initialize_abort_attachment(
        &mut self,
        owner: &mut Owner,
        prefix: &PreparedAbortPrefix,
        claim: &mut AnchoredClaim<'_>,
        directory: PinnedDirectory,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        if self.abort_attachment.is_some() || self.abort_prefix.is_some() {
            return Err(refused("原恢复attachment已尝试"));
        }
        self.abort_attachment = Some(BoundAttachment::pending(prefix, directory));
        let result = (|| {
            let actual = self.read_before_resume_prefix(owner, claim, window)?;
            prefix.check_read(
                self.session,
                claim.epoch(),
                &claim.key(),
                window.deadline(),
                &actual,
            )?;
            let expected = owner
                .data_path()
                .join("before-resume-recovery")
                .join(&prefix.action);
            if self
                .abort_attachment
                .as_ref()
                .unwrap()
                .pending_directory
                .as_ref()
                .unwrap()
                .path()
                != expected
            {
                return Err(refused("必须是原D/恢复action独占附件目录"));
            }
            self.abort_attachment
                .as_mut()
                .unwrap()
                .initialize(owner, prefix, window)?;
            let after = self.read_before_resume_prefix(owner, claim, window)?;
            prefix.check_read(
                self.session,
                claim.epoch(),
                &claim.key(),
                window.deadline(),
                &after,
            )
        })();
        if result.is_err() {
            self.failed = true;
            if let Some(a) = &mut self.abort_attachment {
                a.failed = true;
            }
            owner.fail_before_resume_process();
        }
        result
    }
}
