use crate::{
    errors::AppResult,
    modules::{
        process::{LifecycleAction, LifecycleResult, ProcessCommand},
        runtime::official_lifecycle024::ActionWindow,
    },
    state::ProcessContext,
};
use std::path::Path;

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
