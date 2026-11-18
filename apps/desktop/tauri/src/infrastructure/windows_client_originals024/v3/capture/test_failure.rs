use super::*;
use crate::infrastructure::{hash, windows_owner_host024::qpc};
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

pub(crate) const INJECTED_RAW: i32 = 31;

#[derive(Default)]
pub(crate) struct CaptureFailureArm(Arc<Mutex<Option<Value>>>);

impl CaptureFailureArm {
    pub(crate) fn new() -> (Self, Arc<Mutex<Option<Value>>>) {
        let arm = Self::default();
        let evidence = arm.0.clone();
        (arm, evidence)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Binding {
    owner: [uuid::Uuid; 3],
    scope: String,
    capture_session: uuid::Uuid,
    thread: ThreadId,
    end: Instant,
}

impl Binding {
    fn read(owner: &Owner, capture_session: uuid::Uuid, end: Instant) -> AppResult<Self> {
        let (owner, scope) = owner.test_capture_binding()?;
        Ok(Self {
            owner,
            scope,
            capture_session,
            thread: thread::current().id(),
            end,
        })
    }
}

pub(super) struct BoundCaptureFailure {
    arm: CaptureFailureArm,
    binding: Binding,
}

impl StartRanges {
    pub(crate) fn arm_capture_failure(
        &mut self,
        arm: CaptureFailureArm,
        owner: &Owner,
        capture_session: uuid::Uuid,
        end: Instant,
    ) -> AppResult<()> {
        if self.failure.is_some() || !self.empty() || Instant::now() >= end {
            return Err(refused(
                "test failure arm requires original unstarted worker",
            ));
        }
        self.failure = Some(BoundCaptureFailure {
            arm,
            binding: Binding::read(owner, capture_session, end)?,
        });
        Ok(())
    }
}

impl BoundCaptureFailure {
    pub(super) fn fail(
        self,
        owner: &mut Owner,
        anchor: &PartialCaptureAnchor,
        directory: &PinnedDirectory,
        claim: &mut super::super::super::coordination::AnchoredClaim<'_>,
        root: &Path,
        end: Instant,
    ) -> io::Result<()> {
        let observed = Binding::read(owner, anchor.session_id, end).map_err(io::Error::other)?;
        if observed != self.binding
            || anchor.owner_scope != observed.scope
            || anchor.host_instance_id != observed.owner[0]
            || anchor.journal.is_some()
            || anchor.trees.iter().any(Option::is_none)
            || anchor.source.is_none()
        {
            return Err(invalid());
        }
        let tree_plans = anchor
            .trees
            .iter()
            .flatten()
            .map(AnchoredTree::plan)
            .collect::<Vec<_>>();
        let intent_prefix = serde_json::to_vec(&anchor.intents).map_err(io::Error::other)?;
        directory.recheck()?;
        for name in ["manifest.json", "current.json"] {
            let path = directory.path().join(name);
            if anchor.intents.contains(&path) {
                return Err(invalid());
            }
            match std::fs::symlink_metadata(path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
                Ok(_) => return Err(invalid()),
            }
        }
        directory.recheck()?;
        let ranges = claim.facts(root, end)?;
        owner.test_suspended_creator(end)?;
        let mut evidence = self.arm.0.lock().map_err(|_| invalid())?;
        let (counter, frequency) = qpc()?;
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        *evidence = Some(json!({
            "stage": "captured-both-trees-before-first-journal-intent",
            "raw": INJECTED_RAW, "injected": true,
            "firstErrorQpc": counter.to_string(), "qpcFrequency": frequency.to_string(),
            "hostInstanceId": observed.owner[0], "ownerSessionId": observed.owner[1],
            "failedActionId": observed.owner[2], "scopeDigest": observed.scope,
            "captureSessionId": observed.capture_session, "workerThread": format!("{:?}", observed.thread),
            "sourceBindingDigest": hash::sha256_hex(&anchor.source.as_ref().unwrap().binding_raw),
            "originalTrees": tree_plans, "intentPrefixDigest": hash::sha256_hex(&intent_prefix),
            "intentCount": anchor.intents.len(), "planPath": directory.path(),
            "manifestIntentAndFileAbsent": true, "currentIntentAndFileAbsent": true,
            "originalClaimAndNlkVerified": ranges, "originalCreatorNeverResumed": true,
            "nativeAuthority": false
        }));
        Err(io::Error::from_raw_os_error(INJECTED_RAW))
    }
}
