//! Concrete original Owner callback on the single dedicated recovery worker.
//! No new timer, detached native work, phase reconstruction or public gate.
use super::*;
use crate::infrastructure::windows_client_originals024::recovery_worker::WorkerLocal;
pub(in crate::infrastructure::windows_official_lifecycle024) fn run(
    owner: &mut Owner,
    local: &mut WorkerLocal,
    window: &BeforeResumeRecoveryWindow,
) -> std::io::Result<()> {
    let result = (|| {
        owner.check_before_resume_process(window)?;
        let attempt = owner.before_resume_process.as_ref().unwrap();
        if attempt.restored.is_some() || attempt.attachment_directory.is_some() {
            return Err(recovery_required());
        }
        local.acquire(owner, window).map_err(native_failure)?;
        owner
            .before_resume_process
            .as_mut()
            .unwrap()
            .attachment_directory = Some(
            local
                .attachment_directory(owner, window)
                .map_err(native_failure)?,
        );
        local
            .canonical_claim_stage(owner, window)
            .map_err(native_failure)?;
        local.release(window.deadline()).map_err(native_failure)?;
        local
            .finish_released_stage(owner, window)
            .map_err(native_failure)?;
        owner.check_before_resume_quiescent(window)?;
        owner
            .before_resume_process
            .as_ref()
            .unwrap()
            .restored
            .as_ref()
            .unwrap()
            .read_data(window.deadline())?;
        budget(window.deadline())
    })();
    if result.is_err() {
        owner.fail_before_resume_process();
    }
    result.map_err(std::io::Error::other)
}
