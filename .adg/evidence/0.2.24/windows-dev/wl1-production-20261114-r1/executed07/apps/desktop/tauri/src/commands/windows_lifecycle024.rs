use crate::{
    errors::AppResult,
    modules::{
        process::{LifecycleAction, LifecycleResult, ProcessCommand},
        runtime::official_lifecycle024::ActionWindow,
    },
    state::ProcessContext,
};
use std::path::Path;

/// A proven running owner still permits explicit ordinary Stop, even when
/// durable client recovery cannot yet grant Restart or a successor Start.
pub(crate) fn retained_result(
    mut value: crate::types::process_action::ProcessActionResult,
    result: &LifecycleResult,
) -> crate::types::process_action::ProcessActionResult {
    value.can_start = false;
    value.can_restart = false;
    value.client_recovery_required = true;
    value.can_stop = matches!(result, LifecycleResult::Started);
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proven_running_allows_stop_but_never_recovery_successor() {
        use crate::{
            modules::process::ProcessLifecycleState, types::process_action::ProcessActionResult,
        };
        let running = retained_result(
            ProcessActionResult::from_state(ProcessLifecycleState::Running)
                .with_result(LifecycleResult::Started),
            &LifecycleResult::Started,
        );
        assert!(
            running.can_stop
                && running.client_recovery_required
                && !running.can_start
                && !running.can_restart
        );
        assert_eq!(running.result, Some(LifecycleResult::Started));
        let stopped = retained_result(
            ProcessActionResult::from_state(ProcessLifecycleState::Stopped),
            &LifecycleResult::Stopped,
        );
        assert!(
            stopped.client_recovery_required
                && !stopped.can_stop
                && !stopped.can_start
                && !stopped.can_restart
        );
    }
}

/// Invoked by the same public command path used by GUI/tray/IPC. No legacy
/// recovery-owner scan or old desktop_lifetime fallback is used for official024.
pub(crate) fn execute(
    command: &ProcessCommand,
    context: &ProcessContext,
    data: &Path,
    window: ActionWindow,
) -> AppResult<LifecycleResult> {
    let mut command = command.clone();
    command.environment.home = Some(context.working_directory.clone().into_os_string());
    if command.action == LifecycleAction::Start {
        // Existing persisted Auto/fixed port selection remains the authority.
        let selected = crate::modules::runtime::port::prepare_start(data, &context.opencodex_home)?;
        command.requested_port = selected.port;
    }
    crate::infrastructure::windows_official_lifecycle024::execute(
        &context.runtime,
        &command,
        data,
        window,
    )
}
