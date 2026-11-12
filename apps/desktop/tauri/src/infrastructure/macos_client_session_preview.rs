//! Read-only projection inside the index observer's existing shared action
//! lock. No source/baseline learning, receipt adoption or mutation authority.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    NotRequired,
    Active,
    Pending,
    Restored,
    Unknown,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    NotRecorded,
    Verified,
    Unregistered,
    Unknown,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub status: Status,
    pub phase: Option<State>,
    pub revision: Option<u64>,
    pub attempt_id: Option<String>,
    pub original_evidence: Evidence,
    pub plan_evidence: Evidence,
    pub completion_evidence: Evidence,
    pub phase_count: usize,
    pub can_continue: bool,
    pub can_rollback: bool,
    pub can_finalize: bool,
    pub can_rearm: bool,
    pub rollback_digest: Option<String>,
    pub rearm_digest: Option<String>,
    pub confirmation_digest: Option<String>,
}
impl Preview {
    pub fn unsupported() -> Self {
        Self::unavailable(Status::Unsupported)
    }
    fn unavailable(status: Status) -> Self {
        Self {
            status,
            phase: None,
            revision: None,
            attempt_id: None,
            original_evidence: Evidence::Unknown,
            plan_evidence: Evidence::Unknown,
            completion_evidence: Evidence::Unknown,
            phase_count: 0,
            can_continue: false,
            can_rollback: false,
            can_finalize: false,
            can_rearm: false,
            rollback_digest: None,
            rearm_digest: None,
            confirmation_digest: None,
        }
    }
}
pub(super) fn project(
    context: &ProcessContext,
    data: &Path,
    head: Option<&Head>,
) -> io::Result<Preview> {
    let Some(head) = head else {
        return Ok(Preview {
            status: Status::NotRequired,
            phase: None,
            revision: None,
            attempt_id: None,
            original_evidence: Evidence::NotRecorded,
            plan_evidence: Evidence::NotRecorded,
            completion_evidence: Evidence::NotRecorded,
            phase_count: 0,
            can_continue: false,
            can_rollback: false,
            can_finalize: false,
            can_rearm: false,
            rollback_digest: None,
            rearm_digest: None,
            confirmation_digest: None,
        });
    };
    let mut preview = Preview {
        status: if head.state == State::Ready {
            Status::Active
        } else if matches!(head.state, State::Restored | State::Unchanged) {
            Status::Restored
        } else {
            Status::Pending
        },
        phase: Some(head.state),
        revision: Some(head.revision),
        attempt_id: None,
        original_evidence: Evidence::NotRecorded,
        plan_evidence: Evidence::NotRecorded,
        completion_evidence: Evidence::NotRecorded,
        phase_count: 0,
        can_continue: false,
        can_rollback: false,
        can_finalize: false,
        can_rearm: false,
        rollback_digest: None,
        rearm_digest: None,
        confirmation_digest: None,
    };
    if let Some(refs) = &head.references {
        let client = std::fs::canonicalize(context.working_directory.join(".codex"))?;
        let originals = Originals::open_for_profile(
            data,
            &client,
            &context.working_directory,
            &refs.attempt,
            &refs.original_sha256,
            &refs.phases,
        )?;
        session::verify_index_scope(&originals, refs, &context.opencodex_home)
            .map_err(|_| denied("recovery evidence scope/live changed"))?;
        preview.original_evidence = Evidence::Verified;
        preview.phase_count = refs.phases.len();
        preview.attempt_id = Some(refs.attempt.clone());
        let mut receipt_name = if head.state == State::Unchanged {
            format!(
                "unchanged-readback-{}.json",
                head.revision
                    .checked_sub(1)
                    .ok_or_else(|| denied("early revision invalid"))?
            )
        } else {
            "completion-readback.json".to_string()
        };
        if let Some(plan) = &head.restore_plan {
            // Immutable plan admission here. RestorePrepared gets its full
            // current-object review in finalization_fingerprint below. Do not
            // revalidate live artifacts after a registered Restored session:
            // legitimate later user edits are not interrupted recovery.
            let pending = restore::Pending::open(&originals, &plan.id, &plan.sha256)?;
            receipt_name = pending.readback_name(if head.state == State::RestoreRolledBack {
                restore::Decision::Rollback
            } else {
                restore::Decision::Complete
            });
            preview.plan_evidence = Evidence::Verified;
        }
        match capture_file(&originals.directory, &receipt_name, MAX_FRAME)? {
            Image::Absent => {}
            Image::File { protection, .. } => {
                require_private(&protection, false)?;
                preview.completion_evidence =
                    if matches!(head.state, State::Restored | State::RestoreRolledBack) {
                        completion::verify_registered(
                            data,
                            &client,
                            &context.working_directory,
                            head,
                        )?;
                        Evidence::Verified
                    } else if head.state == State::Unchanged {
                        unchanged::verify_registered(
                            data,
                            &client,
                            &context.working_directory,
                            head,
                        )?;
                        Evidence::Verified
                    } else {
                        Evidence::Unregistered
                    };
            }
        }
        if head.state == State::RestoreRolledBack {
            preview.rearm_digest = originals.rearm_fingerprint(head, false)?;
            preview.can_rearm = preview.rearm_digest.is_some();
        } else if let Some(fingerprint) = originals.finalization_fingerprint(head)? {
            preview.can_finalize = true;
            preview.confirmation_digest = Some(fingerprint);
        } else {
            let (continuation, rollback) = originals.recovery_fingerprints(head, false)?;
            preview.can_continue = continuation.is_some();
            preview.confirmation_digest = continuation;
            preview.can_rollback = rollback.is_some();
            preview.rollback_digest = rollback;
        }
    }
    // Persisted bytes or a readback preview never manufacture the original
    // kernel exit/watch or grant execution of an interrupted artifact plan.
    Ok(preview)
}
pub fn inspect_preview(context: &ProcessContext, data: &Path) -> Preview {
    observation::inspect_map(context, data, |head| project(context, data, head))
        .unwrap_or_else(|_| Preview::unavailable(Status::Unknown))
}
