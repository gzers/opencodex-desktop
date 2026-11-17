//! Current v3 typed adapter. Tags locate this original retained native slot.
//! No v2 fallback/migration, serialized-phase factory or PID/source adoption.
use super::v3::{CaptureSlot, NativePredecessors};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        windows_diagnostic_roles024::NormalExit, windows_official_lifecycle024::Owner,
        windows_supervisor024::Absent,
    },
    modules::runtime::{
        official_lifecycle024::{ActionWindow, ServiceObservation, StopAccepted},
        windows_lifecycle024_contract::{ClientOriginals, NativeSession},
    },
};
use std::time::{Duration, Instant};
fn refused(reason: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_v3_adapter_required".into(),
        detail: format!("原v3完整代次或阶段未知，保留原生现场：{reason}"),
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Original,
    Ready,
    Prepared,
    Closed,
}
pub(crate) struct OriginalSession {
    session: uuid::Uuid,
}
pub(crate) struct ReadySession {
    session: uuid::Uuid,
}
pub(crate) struct PreparedRestore {
    session: uuid::Uuid,
}
pub(crate) struct ClosedEligibility {
    session: uuid::Uuid,
}
pub(crate) enum LivePhase {
    Original(OriginalSession),
    Ready(ReadySession),
    Prepared(PreparedRestore),
    Closed(ClosedEligibility),
}
struct ClosedGeneration {
    closed: ClosedEligibility,
    scope: String,
    // Retain the SAME actual observation taken from the old native Owner.
    _ready: ServiceObservation,
}
struct SuccessorTicket {
    previous_session: uuid::Uuid,
    previous_scope: String,
    predecessors: NativePredecessors,
    digest: String,
}
#[derive(Default)]
pub(crate) struct WindowsOriginals {
    slot: CaptureSlot,
    closed: Vec<CaptureSlot>,
    closed_generations: Vec<ClosedGeneration>,
    tickets: Vec<SuccessorTicket>,
    awaiting_successor: bool,
    session: Option<uuid::Uuid>,
    owner_scope: Option<String>,
    stage: Option<Stage>,
    prepared_ready: Option<ServiceObservation>,
    failed: bool,
}
impl WindowsOriginals {
    fn check_closed_owners(&self, owner: &mut Owner) -> AppResult<()> {
        let previous = owner.originals_previous_owners();
        if self.closed.len() != previous.len()
            || self.closed.len() != self.closed_generations.len()
            || self.closed.len() != self.tickets.len()
            || self.closed.len() > 512
        {
            return Err(refused("原closed槽/Owner/typed观察/票据集合不一致"));
        }
        for (generation, old) in self.closed_generations.iter().zip(previous) {
            if !old.originals_is_stopped() || old.scope()? != generation.scope {
                return Err(refused("保留原Owner不是原closed代次"));
            }
        }
        Ok(())
    }
    fn check_histories_for_action(
        &mut self,
        owner: &mut Owner,
        window: &ActionWindow,
        stopping: bool,
    ) -> AppResult<()> {
        self.check_closed_owners(owner)?;
        self.slot
            .check_predecessors_for_action(&mut self.closed, owner, window, stopping)
    }
    fn check_histories_at(
        &mut self,
        owner: &mut Owner,
        when: Instant,
        end: Instant,
    ) -> AppResult<()> {
        self.check_closed_owners(owner)?;
        self.slot
            .check_predecessors_at(&mut self.closed, owner, when, end)
    }
    fn matches_tag(&self, id: uuid::Uuid, stage: Stage, scope: &str) -> bool {
        !self.failed
            && self.session == Some(id)
            && self.stage == Some(stage)
            && !scope.is_empty()
            && self.owner_scope.as_deref() == Some(scope)
    }
    fn admit(
        &mut self,
        id: uuid::Uuid,
        stage: Stage,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        let scope = owner.scope();
        let matched = scope
            .as_ref()
            .is_ok_and(|scope| self.matches_tag(id, stage, scope));
        self.failed = true;
        if !matched {
            return Err(refused("tag/原Owner/scope/阶段不匹配"));
        }
        owner.check_current(end)
    }
    fn current_id(&self) -> AppResult<uuid::Uuid> {
        self.session.ok_or_else(|| refused("原session缺席"))
    }
    pub(crate) fn before_resume(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        self.admit(self.current_id()?, Stage::Original, owner, end)?;
        let window = owner.originals_action_window(false, end)?;
        self.check_histories_for_action(owner, &window, false)?;
        self.slot.before_resume_existing(owner, end)?;
        self.check_histories_for_action(owner, &window, false)?;
        self.failed = false;
        Ok(())
    }
    pub(crate) fn check_ready(
        &mut self,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> AppResult<()> {
        self.admit(self.current_id()?, Stage::Ready, owner, end)?;
        let when = end
            .checked_sub(Duration::from_secs(5))
            .ok_or_else(|| refused("原Status截止不可定位"))?;
        self.check_histories_at(owner, when, end)?;
        self.slot.check_ready_existing(owner, ready, end)?;
        self.check_histories_at(owner, when, end)?;
        self.failed = false;
        Ok(())
    }
    pub(crate) fn check_ready_for_action(
        &mut self,
        owner: &mut Owner,
        ready: &ServiceObservation,
        window: &ActionWindow,
        stopping: bool,
    ) -> AppResult<()> {
        let end = if stopping {
            window.stop_deadline()
        } else {
            window.deadline()
        };
        self.admit(self.current_id()?, Stage::Ready, owner, end)?;
        self.check_histories_for_action(owner, window, stopping)?;
        self.slot.check_ready_existing(owner, ready, end)?;
        self.check_histories_for_action(owner, window, stopping)?;
        self.failed = false;
        Ok(())
    }
    pub(crate) fn check_closed(
        &mut self,
        closed: &ClosedEligibility,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        // Frozen native status entry receives the original end only. Derive
        // deterministic earliest5s start; never new now+5 or deadline renewal.
        let when = end
            .checked_sub(Duration::from_secs(5))
            .ok_or_else(|| refused("原Status截止不可定位"))?;
        self.check_closed_at(closed, owner, when, end)
    }
    pub(crate) fn check_closed_at(
        &mut self,
        closed: &ClosedEligibility,
        owner: &mut Owner,
        when: Instant,
        end: Instant,
    ) -> AppResult<()> {
        self.admit(closed.session, Stage::Closed, owner, end)?;
        self.check_histories_at(owner, when, end)?;
        self.slot.completed_stop_at(owner, when, end)?;
        self.check_histories_at(owner, when, end)?;
        self.failed = false;
        Ok(())
    }
    pub(crate) fn check_closed_original(
        &mut self,
        closed: &ClosedEligibility,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        self.admit(closed.session, Stage::Closed, owner, end)?;
        self.check_closed_owners(owner)?;
        self.slot
            .check_predecessors_original(&mut self.closed, owner, end)?;
        self.slot.completed_stop_original(owner, end)?;
        self.slot
            .check_predecessors_original(&mut self.closed, owner, end)?;
        self.failed = false;
        Ok(())
    }
    pub(crate) fn check_closed_for_action(
        &mut self,
        closed: &ClosedEligibility,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        window.remaining()?;
        self.admit(closed.session, Stage::Closed, owner, window.deadline())?;
        self.check_histories_for_action(owner, window, false)?;
        self.slot.completed_stop_for_action(owner, window)?;
        self.check_histories_for_action(owner, window, false)?;
        self.failed = false;
        Ok(())
    }
}
impl ClientOriginals for WindowsOriginals {
    type Owner = Owner;
    type OriginalSession = OriginalSession;
    type ReadySession = ReadySession;
    type PreparedRestore = PreparedRestore;
    type NativeNormalExit = NormalExit;
    type SupervisorAbsent = Absent;
    type ClosedEligibility = ClosedEligibility;
    fn capture(&mut self, owner: &mut Owner, window: &ActionWindow) -> AppResult<OriginalSession> {
        window.remaining()?;
        if self.failed || self.session.is_some() || self.stage.is_some() || self.slot.occupied() {
            return Err(refused("已有原v3会话或失败；不替换/追赶"));
        }
        self.failed = true;
        self.check_closed_owners(owner)?;
        let predecessors = NativePredecessors::collect_previous(&mut self.closed, owner, window)?;
        if self.closed.is_empty() {
            if self.awaiting_successor || !self.tickets.is_empty() {
                return Err(refused("首代不得携未消费后继票据"));
            }
        } else {
            let ticket = self
                .tickets
                .last()
                .ok_or_else(|| refused("原后继票据缺席"))?;
            let previous = self
                .closed_generations
                .last()
                .ok_or_else(|| refused("原closed代次缺席"))?;
            if !self.awaiting_successor
                || ticket.previous_session != previous.closed.session
                || ticket.previous_scope != previous.scope
                || ticket.digest != predecessors.digest().map_err(refused)?
                || !ticket.predecessors.same(&predecessors).map_err(refused)?
            {
                return Err(refused("后继票据不是原有序前驱/来源/完整动作截止"));
            }
        }
        let session = uuid::Uuid::new_v4();
        self.session = Some(session);
        self.owner_scope = Some(owner.scope()?);
        self.slot
            .capture_existing_with_predecessors(owner, window, &predecessors)?;
        self.check_histories_for_action(owner, window, false)?;
        self.awaiting_successor = false;
        self.stage = Some(Stage::Original);
        self.failed = false;
        Ok(OriginalSession { session })
    }
    fn record_ready(
        &mut self,
        original: OriginalSession,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> AppResult<ReadySession> {
        self.admit(original.session, Stage::Original, owner, end)?;
        let window = owner.originals_action_window(false, end)?;
        self.check_histories_for_action(owner, &window, false)?;
        self.slot.record_ready_existing(owner, ready, end)?;
        self.check_histories_for_action(owner, &window, false)?;
        self.stage = Some(Stage::Ready);
        self.failed = false;
        Ok(ReadySession {
            session: original.session,
        })
    }
    fn pre_stop(
        &mut self,
        ready: ReadySession,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<PreparedRestore> {
        self.admit(ready.session, Stage::Ready, owner, end)?;
        let window = owner.originals_action_window(true, end)?;
        self.check_histories_for_action(owner, &window, true)?;
        if self.prepared_ready.is_some() {
            return Err(refused("重复原typedReady"));
        }
        // Store the same actual observed object before fallible native capture;
        // a failure retains it beside the poisoned slot, never reconstructs it.
        self.prepared_ready = Some(owner.take_original_ready_for_stop(end)?);
        let kind = owner.original_stop_action(end)?.kind();
        self.slot.prepare_stop_existing(
            owner,
            self.prepared_ready
                .as_ref()
                .ok_or_else(|| refused("原typedReady缺席"))?,
            kind,
            end,
        )?;
        self.check_histories_for_action(owner, &window, true)?;
        self.stage = Some(Stage::Prepared);
        self.failed = false;
        Ok(PreparedRestore {
            session: ready.session,
        })
    }
    fn complete_stop(
        &mut self,
        prepared: PreparedRestore,
        owner: &mut Owner,
        normal: NormalExit,
        absent: Absent,
        official: &StopAccepted,
        end: Instant,
    ) -> AppResult<ClosedEligibility> {
        self.admit(prepared.session, Stage::Prepared, owner, end)?;
        let window = owner.originals_action_window(true, end)?;
        self.check_histories_for_action(owner, &window, true)?;
        let ready = self
            .prepared_ready
            .as_ref()
            .ok_or_else(|| refused("原prepared typedReady缺席"))?;
        if (ready.pid(), ready.port(), ready.record_sha256())
            != (official.pid(), official.port(), official.record_sha256())
        {
            return Err(refused("typed Stop接受不是原prepared代次"));
        }
        self.slot
            .restore_stopped_existing(owner, normal, absent, official, end)?;
        self.slot.completed_stop_original(owner, end)?;
        self.check_histories_for_action(owner, &window, true)?;
        self.stage = Some(Stage::Closed);
        self.failed = false;
        Ok(ClosedEligibility {
            session: prepared.session,
        })
    }
    fn authorize_successor(
        &mut self,
        closed: ClosedEligibility,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        self.check_closed_for_action(&closed, owner, window)?;
        self.failed = true;
        if self.awaiting_successor || !owner.originals_is_stopped() || self.closed.len() >= 512 {
            return Err(refused("后继重复/原Stop未知/完整前驱预算耗尽"));
        }
        let predecessors =
            NativePredecessors::with_current(&mut self.closed, &mut self.slot, owner, window)?;
        let scope = owner.scope()?;
        let ticket = SuccessorTicket {
            previous_session: closed.session,
            previous_scope: scope.clone(),
            digest: predecessors.digest().map_err(refused)?,
            predecessors,
        };
        // All fallible native admission precedes this move. Every old slot,
        // original typed observation and consumed Closed locator stays retained.
        let ready = self
            .prepared_ready
            .take()
            .ok_or_else(|| refused("原Stop观察缺席"))?;
        self.closed.push(std::mem::take(&mut self.slot));
        self.closed_generations.push(ClosedGeneration {
            closed,
            scope,
            _ready: ready,
        });
        self.tickets.push(ticket);
        self.awaiting_successor = true;
        self.session = None;
        self.owner_scope = None;
        self.stage = None;
        self.failed = false;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_r6_typed_adapter_data_locator_never_accepts_missing_stale_foreign_or_failed_stage() {
        let id = uuid::Uuid::new_v4();
        let scope = "synthetic locator DATA, no native authority";
        let mut adapter = WindowsOriginals::default();
        assert!(adapter.current_id().is_err());
        assert!(!adapter.matches_tag(id, Stage::Original, scope));
        adapter.session = Some(id);
        adapter.owner_scope = Some(scope.into());
        for stage in [
            Stage::Original,
            Stage::Ready,
            Stage::Prepared,
            Stage::Closed,
        ] {
            adapter.stage = Some(stage);
            assert!(adapter.matches_tag(id, stage, scope));
            assert!(!adapter.matches_tag(uuid::Uuid::new_v4(), stage, scope));
            assert!(!adapter.matches_tag(id, stage, "foreign"));
            assert!(!adapter.matches_tag(id, stage, ""));
            for wrong in [
                Stage::Original,
                Stage::Ready,
                Stage::Prepared,
                Stage::Closed,
            ] {
                if stage != wrong {
                    assert!(!adapter.matches_tag(id, wrong, scope));
                }
            }
            adapter.failed = true;
            assert!(!adapter.matches_tag(id, stage, scope));
            adapter.failed = false;
            assert!(
                !adapter.slot.occupied(),
                "DATA-stage comparisons never create native slot/proof or any typed phase tag"
            );
        }
        println!("r6 typed adapter DATA locator permutations reject missing/stale/foreign/failed/wrong stages; no phase tag, Owner, ServiceObservation, original capture or ClosedEligibility constructed from this test");
    }
    #[test]
    fn wl2_r6_typed_adapter_retained_v3_slot_and_private_observation_types_are_send() {
        fn send<T: Send>() {}
        send::<WindowsOriginals>();
        send::<ServiceObservation>();
        assert!(WindowsOriginals::default().current_id().is_err());
        println!("r6 compiler retains exact original typed observation+v3 slot within Send production adapter; no serde/Clone/native reconstruction or unsafeSend added");
    }
}
