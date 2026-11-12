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
        }
    }
}
fn project(context: &ProcessContext, data: &Path, head: Option<&Head>) -> io::Result<Preview> {
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
        });
    };
    let mut preview = Preview {
        status: if head.state == State::Ready {
            Status::Active
        } else if head.state == State::Restored {
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
        if let Some(plan) = &head.restore_plan {
            restore::Pending::open(&originals, &plan.id, &plan.sha256)?;
            preview.plan_evidence = Evidence::Verified;
        }
        match capture_file(&originals.directory, "completion-readback.json", MAX_FRAME)? {
            Image::Absent => {}
            Image::File { protection, .. } => {
                require_private(&protection, false)?;
                preview.completion_evidence = if head.state == State::Restored {
                    completion::verify_registered(data, &client, &context.working_directory, head)?;
                    Evidence::Verified
                } else {
                    Evidence::Unregistered
                };
            }
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
