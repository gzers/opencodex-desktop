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
    let saved_port_home =
        (command.action == LifecycleAction::Start).then_some(context.opencodex_home.as_path());
    crate::infrastructure::windows_official_lifecycle024::execute_with_port_selection(
        &context.runtime,
        &command,
        data,
        window,
        saved_port_home,
    )
}
