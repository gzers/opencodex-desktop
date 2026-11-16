//! The live claim and exact native plan join the durable data chain only for
//! one bounded node callback. No callback/lease/mutation authority escapes it.
use super::super::super::coordination::AnchoredClaim;
use super::*;
impl Journal {
    fn check_tree(
        &mut self,
        plan: &serde_json::Value,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        if self.progress.lease_state != LeaseState::Active {
            return Err(invalid());
        }
        let codex = self.pins[0].path().join(".codex");
        claim.check(&codex, end)?;
        if claim.epoch() != self.progress.epoch || claim.key() != self.progress.claim {
            return Err(invalid());
        }
        if !self
            .manifest
            .storage_plan
            .roots
            .iter()
            .any(|root| serde_json::to_value(root).is_ok_and(|v| v == *plan))
        {
            return Err(invalid());
        }
        before(end)
    }
    fn event(
        &self,
        node: &str,
        op: Operation,
        native: Option<(String, SnapshotData)>,
        expected_before: String,
        expected_after: String,
    ) -> io::Result<Event> {
        let (root, n) = self
            .manifest
            .storage_plan
            .roots
            .iter()
            .find_map(|root| {
                root.nodes
                    .iter()
                    .find(|n| n.node_id == node)
                    .map(|n| (root, n))
            })
            .ok_or_else(invalid)?;
        let path = if n.relative.is_empty() {
            n.root_id.clone()
        } else {
            format!("{}/{}", n.root_id, n.relative)
        };
        let custody = format!("custody/{}/original-{}", n.root_id, n.node_id);
        let retired = format!("retired/{}/retired-{}", n.root_id, n.node_id);
        let (from, to, parent) = match op {
            Operation::NodeMoveIntent | Operation::NodeMovedVerified => {
                (path, custody, root.custody_id.clone())
            }
            Operation::WorkCreateIntent | Operation::WorkCreatedVerified => {
                ("Absent".into(), path, n.parent_id.clone())
            }
            Operation::WorkRetireIntent | Operation::WorkRetiredVerified => {
                (path, retired, root.custody_id.clone())
            }
            Operation::OriginalRestoreIntent | Operation::OriginalRestoredVerified => {
                (custody, path, n.parent_id.clone())
            }
            Operation::SnapshotVerified | Operation::ReleaseVerified => {
                (path.clone(), path, n.parent_id.clone())
            }
        };
        let (id, security) = match native {
            None => (None, None),
            Some((id, security)) => (Some(id), Some(security)),
        };
        Ok(Event {
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "node-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.progress.session.clone(),
            node_id: node.into(),
            root_id: n.root_id.clone(),
            operation: op,
            from,
            to,
            native_id: id,
            native_security: security,
            parent_id: parent,
            expected_before,
            expected_after,
            lease_epoch: self.progress.epoch.clone(),
            claim_key: self.progress.claim.clone(),
            deadline_scope: self.progress.deadline.clone(),
            previous_event_digest: self.progress.tip.clone(),
            storage_plan_digest: self.progress.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: self.progress.manifest.clone(),
            binding_digest: self.progress.binding.clone(),
        })
    }
    pub(in crate::infrastructure::windows_client_originals024) fn snapshot_anchor(
        &mut self,
        plan: &serde_json::Value,
        node: &str,
        native: (String, SnapshotData),
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_tree(plan, claim, end)?;
            let expectation = hash::sha256_hex(&encoded(&(
                self.progress.plan.as_str(),
                node,
                "AnchorUnchanged",
            ))?);
            let event = self.event(
                node,
                Operation::SnapshotVerified,
                Some(native),
                expectation.clone(),
                expectation,
            )?;
            self.append(event, end)?;
            self.check_tree(plan, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn execute_node<F>(
        &mut self,
        plan: &serde_json::Value,
        node: &str,
        action: FileAction,
        native_before: Option<(String, SnapshotData)>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        native: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut AnchoredClaim<'_>) -> io::Result<(String, SnapshotData)>,
    {
        let result = self.execute_inner(plan, node, action, native_before, claim, end, native);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn execute_inner<F>(
        &mut self,
        plan: &serde_json::Value,
        node: &str,
        action: FileAction,
        native_before: Option<(String, SnapshotData)>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        native: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut AnchoredClaim<'_>) -> io::Result<(String, SnapshotData)>,
    {
        self.check_tree(plan, claim, end)?;
        let (intent, completion, from, to) = match action {
            FileAction::MoveOriginal => (
                Operation::NodeMoveIntent,
                Operation::NodeMovedVerified,
                "OriginalAtClient",
                "OriginalAtCustody",
            ),
            FileAction::CreateWork => (
                Operation::WorkCreateIntent,
                Operation::WorkCreatedVerified,
                "OriginalAtCustody",
                "OriginalWithWorkAtClient",
            ),
            FileAction::RetireWork => (
                Operation::WorkRetireIntent,
                Operation::WorkRetiredVerified,
                "OriginalWithWorkAtClient",
                "OriginalWithRetiredWork",
            ),
            FileAction::RestoreOriginal => (
                Operation::OriginalRestoreIntent,
                Operation::OriginalRestoredVerified,
                "OriginalWithRetiredWork",
                "OriginalAtClientWithRetiredWork",
            ),
        };
        // These are exact expected position/content/generation-policy digests
        // bound to the full immutable native plan. New work identity/security
        // cannot be predicted: completion records its actual native all8, and
        // later retirement must match that same generation and snapshot.
        let before_digest = hash::sha256_hex(&encoded(&(self.progress.plan.as_str(), node, from))?);
        let after_digest = hash::sha256_hex(&encoded(&(self.progress.plan.as_str(), node, to))?);
        let event = self.event(
            node,
            intent,
            native_before,
            before_digest.clone(),
            after_digest.clone(),
        )?;
        self.append(event, end)?;
        self.check_tree(plan, claim, end)?;
        #[cfg(test)]
        if self.fault == 4 {
            return Err(io::Error::other(
                "injected after durable node intent/current, before native mutation",
            ));
        }
        let observed = native(claim)?;
        self.check_tree(plan, claim, end)?;
        #[cfg(test)]
        if self.fault == 3 {
            return Err(io::Error::other(
                "injected after actual native mutation before result publication",
            ));
        }
        #[cfg(test)]
        {
            if self.fault == 5 {
                self.fault = 1;
            } else if self.fault == 6 {
                self.fault = 2;
            }
        }
        let result = self.event(
            node,
            completion,
            Some(observed),
            before_digest,
            after_digest,
        )?;
        self.append(result, end)?;
        self.check_tree(plan, claim, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn inject_driver_fault(
        &mut self,
        fault: u8,
    ) {
        self.fault = fault;
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn native_progress_for_test(
        &self,
    ) -> (u32, bool) {
        (self.progress.sequence, self.poisoned)
    }
}
