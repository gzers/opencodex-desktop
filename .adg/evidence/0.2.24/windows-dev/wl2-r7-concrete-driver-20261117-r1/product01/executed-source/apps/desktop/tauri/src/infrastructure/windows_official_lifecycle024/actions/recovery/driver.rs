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
        {
            let lease = local.lease().map_err(native_failure)?;
            lease
                .check_owner(owner, window.deadline())
                .map_err(native_failure)?;
            let root = owner.profile_path().join(".codex");
            let mut claim = lease
                .anchored_claim(&root, window.deadline())
                .map_err(native_failure)?;
            let prefix = owner.with_originals(|originals, owner| {
                originals.prepare_abort_prefix(owner, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                let directory = owner
                    .before_resume_process
                    .as_mut()
                    .unwrap()
                    .attachment_directory
                    .take()
                    .ok_or_else(recovery_required)?;
                originals.initialize_abort_attachment(owner, &prefix, &mut claim, directory, window)
            })?;
            let quiescent = owner.with_originals(|originals, owner| {
                originals.quiesce_abort_prefix(owner, prefix, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                originals.return_quiescent_originals(owner, &quiescent, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                originals.retain_quiescent_for_release(owner, quiescent, window)
            })?;
            claim
                .check(&root, window.deadline())
                .map_err(native_failure)?;
        }
        local.release(window.deadline()).map_err(native_failure)?;
        let (released, probe) = local.released_and_probe().map_err(native_failure)?;
        let restored = owner.with_originals(|originals, owner| {
            originals.finish_retained_quiescent(owner, released, probe, window)
        })?;
        owner.before_resume_process.as_mut().unwrap().restored = Some(restored);
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
