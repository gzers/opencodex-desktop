//! Exact windows passed from the existing shared public boundary. No nested
//! ActionWindow::begin, replacement now+budget, serde or text receipt factory.
use super::*;
use crate::modules::runtime::windows_lifecycle024_contract::NativeSession;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalActionKind {
    Start,
    Stop,
    Restart,
}
pub(super) struct ActionOrigin {
    id: uuid::Uuid,
    kind: OriginalActionKind,
    end: Instant,
    stop_end: Instant,
    admitted_at: Instant,
}
impl ActionOrigin {
    pub(super) fn admit(
        kind: OriginalActionKind,
        window: &shared::ActionWindow,
    ) -> AppResult<Self> {
        window.remaining()?;
        let admitted_at = Instant::now();
        if window.stop_deadline() > window.deadline() || admitted_at >= window.deadline() {
            return Err(AppError::Timeout);
        }
        let out = Self {
            id: uuid::Uuid::new_v4(),
            kind,
            end: window.deadline(),
            stop_end: window.stop_deadline(),
            admitted_at,
        };
        out.check(out.relevant_end())?;
        Ok(out)
    }
    fn relevant_end(&self) -> Instant {
        if self.kind == OriginalActionKind::Start {
            self.end
        } else {
            self.stop_end
        }
    }
    fn check(&self, end: Instant) -> AppResult<()> {
        if end != self.relevant_end() || self.admitted_at >= end {
            return Err(recovery_required());
        }
        budget(end)
    }
}
/// Fresh private nonserde receipt bound to actual retained Owner scope/token and
/// original action object. Its data trace never creates journal/native authority.
pub(crate) struct NativeAction {
    scope: String,
    owner: uuid::Uuid,
    origin: uuid::Uuid,
    kind: OriginalActionKind,
    end: Instant,
    outer_end: Instant,
    stop_end: Instant,
    token: native::TokenIdentity,
}
impl Owner {
    pub(super) fn bind_stop_action(
        &mut self,
        kind: LifecycleAction,
        window: &shared::ActionWindow,
    ) -> AppResult<()> {
        self.source(window.stop_deadline())?;
        let kind = match kind {
            LifecycleAction::Stop => OriginalActionKind::Stop,
            LifecycleAction::Restart => OriginalActionKind::Restart,
            _ => return Err(recovery_required()),
        };
        install_stop_origin(
            &mut self.stop_action,
            self.stop_started.is_some(),
            kind,
            window,
        )
    }
    pub(crate) fn original_action(
        &mut self,
        kind: OriginalActionKind,
        end: Instant,
    ) -> AppResult<NativeAction> {
        self.check_current(end)?;
        let origin = if kind == OriginalActionKind::Start {
            &self.start_action
        } else {
            self.stop_action.as_ref().ok_or_else(recovery_required)?
        };
        if origin.kind != kind {
            return Err(recovery_required());
        }
        origin.check(end)?;
        let out = NativeAction {
            scope: self.scope()?,
            owner: self.session,
            origin: origin.id,
            kind,
            end,
            outer_end: origin.end,
            stop_end: origin.stop_end,
            token: native::current_identity().map_err(unknown)?,
        };
        out.check(self, end)?;
        Ok(out)
    }
}
fn install_stop_origin(
    slot: &mut Option<ActionOrigin>,
    already_started: bool,
    kind: OriginalActionKind,
    window: &shared::ActionWindow,
) -> AppResult<()> {
    if slot.is_some() || already_started || kind == OriginalActionKind::Start {
        return Err(recovery_required());
    }
    *slot = Some(ActionOrigin::admit(kind, window)?);
    Ok(())
}
impl NativeAction {
    pub(crate) fn same_original_owner(&self, other: &Self) -> bool {
        self.owner == other.owner && self.scope == other.scope && self.token == other.token
    }
    pub(crate) fn check(&self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        if self.end != end
            || self.owner != owner.session
            || self.scope != owner.scope()?
            || native::current_identity().map_err(unknown)? != self.token
        {
            return Err(recovery_required());
        }
        let origin = if self.kind == OriginalActionKind::Start {
            &owner.start_action
        } else {
            owner.stop_action.as_ref().ok_or_else(recovery_required)?
        };
        if origin.id != self.origin
            || origin.end != self.outer_end
            || origin.stop_end != self.stop_end
            || origin.kind != self.kind
        {
            return Err(recovery_required());
        }
        origin.check(end)?;
        owner.check_current(end)
    }
    pub(crate) fn deadline(&self) -> Instant {
        self.end
    }
    pub(crate) fn scope_id(&self) -> String {
        self.origin.to_string()
    }
    pub(crate) fn kind(&self) -> OriginalActionKind {
        self.kind
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_r6_original_actions_repeated_or_partially_started_stop_cannot_replace_origin() {
        let now = Instant::now();
        let w =
            shared::ActionWindow::begin(LifecycleAction::Stop, now, now + Duration::from_secs(10))
                .unwrap();
        let mut slot = None;
        install_stop_origin(&mut slot, false, OriginalActionKind::Stop, &w).unwrap();
        let id = slot.as_ref().unwrap().id;
        let later = shared::ActionWindow::begin(
            LifecycleAction::Stop,
            now + Duration::from_millis(1),
            now + Duration::from_secs(11),
        )
        .unwrap();
        assert!(install_stop_origin(&mut slot, false, OriginalActionKind::Stop, &later).is_err());
        assert_eq!(slot.unwrap().id, id);
        let mut empty = None;
        assert!(install_stop_origin(&mut empty, true, OriginalActionKind::Stop, &w).is_err());
        assert!(empty.is_none());
        assert!(install_stop_origin(&mut empty, false, OriginalActionKind::Start, &w).is_err());
        assert!(empty.is_none());
        println!("r6 repeated/new deadline or partially started Stop cannot reset original receipt; no Origin minted for invalid kind");
    }
    fn window(kind: LifecycleAction, started: Instant, parent: Instant) -> shared::ActionWindow {
        shared::ActionWindow::begin(kind, started, parent).unwrap()
    }
    #[test]
    fn wl2_r6_original_actions_exact_start_stop_and_restart_subwindow_never_reset() {
        let now = Instant::now();
        let parent = now + Duration::from_secs(29);
        let start = window(LifecycleAction::Start, now, parent);
        let original = ActionOrigin::admit(OriginalActionKind::Start, &start).unwrap();
        assert_eq!(original.end, parent);
        assert_eq!(original.stop_end, now + Duration::from_secs(10));
        assert!(original.check(parent).is_ok());
        assert!(original.check(parent + Duration::from_nanos(1)).is_err());
        assert!(original.check(now + Duration::from_secs(28)).is_err());
        let restart = window(LifecycleAction::Restart, now, parent);
        let original = ActionOrigin::admit(OriginalActionKind::Restart, &restart).unwrap();
        assert_eq!(original.end, parent);
        assert_eq!(original.relevant_end(), now + Duration::from_secs(10));
        assert!(
            original.check(restart.deadline()).is_err(),
            "restart stop cannot borrow outer29 seconds"
        );
        assert!(original.check(restart.stop_deadline()).is_ok());
        let short = window(LifecycleAction::Stop, now, now + Duration::from_secs(3));
        let original = ActionOrigin::admit(OriginalActionKind::Stop, &short).unwrap();
        assert_eq!(original.relevant_end(), now + Duration::from_secs(3));
        assert!(original.check(now + Duration::from_secs(10)).is_err());
        println!("r6 actual shared ActionWindow original parent/end/stop sub10 retained exactly; no nested new budget or different deadline admitted, source-owner receipt factory remains actual native only");
    }
    #[test]
    fn wl2_r6_original_actions_late_stop_subwindow_and_expired_origin_refuse() {
        let now = Instant::now();
        let restart = window(
            LifecycleAction::Restart,
            now - Duration::from_secs(11),
            now + Duration::from_secs(15),
        );
        assert!(
            ActionOrigin::admit(OriginalActionKind::Restart, &restart).is_err(),
            "original stop10 already elapsed even if outer remains"
        );
        let mut original = ActionOrigin::admit(
            OriginalActionKind::Start,
            &window(LifecycleAction::Start, now, now + Duration::from_secs(2)),
        )
        .unwrap();
        original.end = now - Duration::from_millis(1);
        assert!(original.check(original.end).is_err());
        println!("r6 late restart admission does not reset elapsed original stop sub10; expired private origin no permission");
    }
}
