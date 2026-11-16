//! Only original actual owned native Stop histories admit the final seal.
//! Caller DATA/flags cannot construct this borrowed guard or a native receipt.
use super::super::super::{
    content::{StartContentScope, StoppedContentScope},
    source::Source,
    v2::routes::RoutePin,
};
use super::*;
use crate::{
    infrastructure::{
        windows_diagnostic_roles024::ClosedExitHistory, windows_official_lifecycle024::Owner,
        windows_supervisor024::ClosedAbsentHistory,
    },
    modules::runtime::official_lifecycle024::StopAccepted,
};
pub(in crate::infrastructure::windows_client_originals024) struct CompletedStopGuard<'a> {
    source: &'a mut Source,
    owner: &'a mut Owner,
    exit: &'a ClosedExitHistory,
    absent: &'a ClosedAbsentHistory,
    official: &'a StopAccepted,
    routes: &'a mut RoutePin,
    start: &'a mut StartContentScope,
    stopped: &'a StoppedContentScope,
}
impl<'a> CompletedStopGuard<'a> {
    pub(in crate::infrastructure::windows_client_originals024) fn bind(
        source: &'a mut Source,
        owner: &'a mut Owner,
        exit: &'a ClosedExitHistory,
        absent: &'a ClosedAbsentHistory,
        official: &'a StopAccepted,
        routes: &'a mut RoutePin,
        start: &'a mut StartContentScope,
        stopped: &'a StoppedContentScope,
        journal: &mut Journal,
        end: Instant,
    ) -> io::Result<Self> {
        let mut out = Self {
            source,
            owner,
            exit,
            absent,
            official,
            routes,
            start,
            stopped,
        };
        out.check(journal, end)?;
        Ok(out)
    }
    fn check(&mut self, journal: &mut Journal, end: Instant) -> io::Result<()> {
        self.start.check_completed_stop(
            self.stopped,
            self.source,
            self.owner,
            self.exit,
            self.routes,
            end,
        )?;
        self.absent
            .check(self.owner, end)
            .map_err(io::Error::other)?;
        self.owner
            .originals_check_stopped_records(end)
            .map_err(io::Error::other)?;
        journal.check_source_owner(self.owner, end)?;
        if self.source.binding_raw != encoded(&journal.manifest.binding)?
            || self.stopped.deadline() != end
            || journal.progress.ready_generation.as_ref()
                != Some(&(
                    self.official.pid(),
                    self.official.port(),
                    self.official.record_sha256().into(),
                ))
        {
            return Err(invalid());
        }
        // Same actual content scope previously accepted by the independent
        // native assessment. No new scope/route/PID interval can be substituted.
        let content=journal.events.iter().filter_map(|r|match r.entry.as_ref(){Some(ChainEntry::Content(e))
            if e.operation==super::super::qualification::ContentOperation::ContentQualifiedVerified=>Some(&e.content),_=>None}).collect::<Vec<_>>();
        if content.len() != 1
            || content[0].scope_digest != self.stopped.digest()?
            || !journal
                .progress
                .content
                .as_ref()
                .is_some_and(|c| c.verified && content[0].digest().is_ok_and(|d| d == c.digest))
        {
            return Err(invalid());
        }
        journal.check(end)
    }
}
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024) fn complete_original_stop_history(
        &mut self,
        guard: &mut CompletedStopGuard<'_>,
        end: Instant,
    ) -> io::Result<()> {
        use super::super::closed::{ClosedData, ClosedOperation};
        let result = (|| {
            guard.check(self, end)?;
            let original = self.original_close_data(end)?;
            let intent = self
                .progress
                .closed_event(ClosedOperation::StopClosureIntent, original)?;
            self.append_entry_inner(ChainEntry::Closed(intent), end)?;
            guard.check(self, end)?;
            self.completed_stop_boundary(end)?;
            let data: ClosedData = match self.events.last().and_then(|r| r.entry.as_ref()) {
                Some(ChainEntry::Closed(e)) => {
                    serde_json::from_slice(&encoded(&e.closed)?).map_err(|_| invalid())?
                }
                _ => return Err(invalid()),
            };
            let verified = self
                .progress
                .closed_event(ClosedOperation::StopClosedVerified, data)?;
            self.append_entry_inner(ChainEntry::Closed(verified), end)?;
            guard.check(self, end)?;
            self.completed_stop_boundary(end)?;
            self.stop_completion = Some(super::completed::CompletedStop::capture(self, end)?);
            guard.check(self, end)?;
            before(end)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
}
