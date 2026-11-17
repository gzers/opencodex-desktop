//! Bounded nonrecursive node rows in ONE physical canonical phase. DATA only.
//! Every row passes the existing single-node validator on an isolated DATA
//! projection; neither parsing nor prefix counts create native qualifications.
use super::recovery::{RecoveryEvent, RecoveryOperation};
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::infrastructure::windows_client_originals024) enum BatchKind {
    OriginalMove,
    WorkCreate,
    CurrentRetire,
    OriginalRestore,
    AnchorSnapshot,
    AnchorRelease,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum BatchOperation {
    Intent,
    PrefixVerified,
    AnchorsVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum RowEvent {
    Original(Event),
    Recovery(RecoveryEvent),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct BatchRow {
    pub(super) ordinal: usize,
    pub(super) event: RowEvent,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct BatchEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) batch_id: String,
    pub(super) kind: BatchKind,
    pub(super) operation: BatchOperation,
    pub(super) plan_digest: String,
    pub(super) count: usize,
    pub(super) prefix: usize,
    pub(super) rows: Vec<BatchRow>,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) deadline_scope: String,
    pub(super) previous_event_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BatchProgress {
    batch_id: String,
    kind: BatchKind,
    plan_digest: String,
    rows: Vec<BatchRow>,
    prefix: usize,
}
impl BatchProgress {
    pub(super) fn before_resume_parts(&self) -> io::Result<(&str, BatchKind, usize, usize)> {
        if !matches!(self.kind, BatchKind::OriginalMove | BatchKind::WorkCreate)
            || self.prefix >= self.rows.len()
        {
            return Err(invalid());
        }
        Ok((&self.batch_id, self.kind, self.rows.len(), self.prefix))
    }
    pub(super) fn before_resume_rows(&self) -> io::Result<&[BatchRow]> {
        self.before_resume_parts()?;
        Ok(&self.rows)
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct BatchSummary {
    batch_id: String,
    kind: BatchKind,
    plan_digest: String,
    count: usize,
    prefix: usize,
}
impl BatchKind {
    fn ordinary(self) -> Option<(Operation, Operation)> {
        Some(match self {
            Self::OriginalMove => (Operation::NodeMoveIntent, Operation::NodeMovedVerified),
            Self::WorkCreate => (Operation::WorkCreateIntent, Operation::WorkCreatedVerified),
            Self::AnchorSnapshot => (Operation::SnapshotVerified, Operation::SnapshotVerified),
            _ => return None,
        })
    }
    fn recovery(self) -> Option<(RecoveryOperation, RecoveryOperation)> {
        Some(match self {
            Self::CurrentRetire => (
                RecoveryOperation::WorkRetireIntent,
                RecoveryOperation::WorkRetiredVerified,
            ),
            Self::OriginalRestore => (
                RecoveryOperation::OriginalRestoreIntent,
                RecoveryOperation::OriginalRestoredVerified,
            ),
            Self::AnchorRelease => (
                RecoveryOperation::ReleaseVerified,
                RecoveryOperation::ReleaseVerified,
            ),
            _ => return None,
        })
    }
    pub(super) fn anchor(self) -> bool {
        matches!(self, Self::AnchorSnapshot | Self::AnchorRelease)
    }
}
impl RowEvent {
    fn node(&self) -> io::Result<(&str, &str)> {
        match self {
            Self::Original(e) => Ok((&e.root_id, &e.node_id)),
            Self::Recovery(e) => Ok((
                e.root_id.as_deref().ok_or_else(invalid)?,
                e.node_id.as_deref().ok_or_else(invalid)?,
            )),
        }
    }
    fn id(&self) -> &str {
        match self {
            Self::Original(e) => &e.event_id,
            Self::Recovery(e) => &e.event_id,
        }
    }
    fn matches_phase(&self, g: &Progress, e: &BatchEvent) -> bool {
        match self {
            Self::Original(r) => {
                r.sequence == e.sequence
                    && r.previous_event_digest == e.previous_event_digest
                    && r.session_id == g.session
                    && r.lease_epoch == g.epoch
                    && r.claim_key == g.claim
                    && r.deadline_scope == g.deadline
            }
            Self::Recovery(r) => {
                r.sequence == e.sequence
                    && r.previous_event_digest == e.previous_event_digest
                    && r.session_id == g.session
                    && r.lease_epoch == g.epoch
                    && r.claim_key == g.claim
                    && r.deadline_scope == g.deadline
            }
        }
    }
}
impl Progress {
    pub(super) fn initial_anchor_rows_needed(&self, plan: &StoragePlan) -> io::Result<bool> {
        if self.batch.is_some() || self.recovery.is_some() || self.pending.is_some() {
            return Err(invalid());
        }
        let states = plan
            .roots
            .iter()
            .flat_map(|r| &r.nodes)
            .filter(|n| n.kind != Kind::CustodiedFile)
            .map(|n| self.nodes[&n.node_id].0)
            .collect::<Vec<_>>();
        if states.iter().all(|s| *s == 0) {
            Ok(true)
        } else if states.iter().all(|s| *s == 1) {
            Ok(false)
        } else {
            Err(invalid())
        }
    }
    pub(super) fn uses_node_batches(&self) -> bool {
        !self.seen_batches.is_empty()
    }
    fn data_projection(&self) -> io::Result<Self> {
        serde_json::from_slice(&encoded(self)?).map_err(|_| invalid())
    }
    pub(super) fn batch_expected_nodes(
        &self,
        plan: &StoragePlan,
        kind: BatchKind,
    ) -> io::Result<Vec<(String, String)>> {
        if matches!(kind, BatchKind::CurrentRetire | BatchKind::OriginalRestore) {
            return self.batch_recovery_nodes(kind == BatchKind::CurrentRetire);
        }
        Ok(plan
            .roots
            .iter()
            .flat_map(|r| {
                r.nodes
                    .iter()
                    .filter(|n| (n.kind != Kind::CustodiedFile) == kind.anchor())
                    .map(|n| (n.root_id.clone(), n.node_id.clone()))
            })
            .collect())
    }
    pub(super) fn batch_summary(&self) -> Option<BatchSummary> {
        self.batch.as_ref().map(|b| BatchSummary {
            batch_id: b.batch_id.clone(),
            kind: b.kind,
            plan_digest: b.plan_digest.clone(),
            count: b.rows.len(),
            prefix: b.prefix,
        })
    }
    pub(super) fn batch_intent_rows(&self) -> io::Result<serde_json::Value> {
        serde_json::to_value(&self.batch.as_ref().ok_or_else(invalid)?.rows).map_err(|_| invalid())
    }
    pub(super) fn batch_event(
        &self,
        kind: BatchKind,
        operation: BatchOperation,
        rows: Vec<BatchRow>,
    ) -> io::Result<BatchEvent> {
        let (batch_id, plan_digest, count, prefix) = if operation == BatchOperation::PrefixVerified
        {
            let b = self.batch.as_ref().ok_or_else(invalid)?;
            if kind != b.kind {
                return Err(invalid());
            }
            (
                b.batch_id.clone(),
                b.plan_digest.clone(),
                b.rows.len(),
                b.prefix.checked_add(rows.len()).ok_or_else(invalid)?,
            )
        } else {
            (
                uuid::Uuid::new_v4().to_string(),
                hash::sha256_hex(&encoded(&rows)?),
                rows.len(),
                if kind.anchor() { rows.len() } else { 0 },
            )
        };
        Ok(BatchEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "bounded-node-batch-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            batch_id,
            kind,
            operation,
            plan_digest,
            count,
            prefix,
            rows,
            lease_epoch: self.epoch.clone(),
            claim_key: self.claim.clone(),
            deadline_scope: self.deadline.clone(),
            previous_event_digest: self.tip.clone(),
            storage_plan_digest: self.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: self.manifest.clone(),
            binding_digest: self.binding.clone(),
        })
    }
    fn apply_batch_row(
        &mut self,
        plan: &StoragePlan,
        kind: BatchKind,
        row: &BatchRow,
        completion: bool,
        intent: Option<&BatchRow>,
    ) -> io::Result<()> {
        match &row.event {
            RowEvent::Original(event) => {
                let (before, after) = kind.ordinary().ok_or_else(invalid)?;
                if event.operation != if completion { after } else { before } {
                    return Err(invalid());
                }
                if completion && !kind.anchor() {
                    let RowEvent::Original(pending) = &intent.ok_or_else(invalid)?.event else {
                        return Err(invalid());
                    };
                    self.pending = Some(encoded(pending)?);
                    if kind == BatchKind::WorkCreate
                        && self
                            .nodes
                            .values()
                            .any(|(_, id)| id.as_ref() == event.native_id.as_ref() && id.is_some())
                    {
                        return Err(invalid());
                    }
                }
                if completion && kind == BatchKind::WorkCreate {
                    let id = event.native_id.as_ref().ok_or_else(invalid)?;
                    let node = plan
                        .roots
                        .iter()
                        .flat_map(|r| &r.nodes)
                        .find(|n| n.node_id == event.node_id)
                        .ok_or_else(invalid)?;
                    if plan.roots.iter().any(|r| {
                        r.custody_id == *id
                            || r.nodes
                                .iter()
                                .any(|n| n.file_id == *id || n.parent_id == *id)
                    }) || self.snapshot_objects.contains(&(node.volume, id.clone()))
                    {
                        return Err(invalid());
                    }
                }
                self.apply(plan, event)
            }
            RowEvent::Recovery(event) => {
                let (before, after) = kind.recovery().ok_or_else(invalid)?;
                if event.operation != if completion { after } else { before } {
                    return Err(invalid());
                }
                if completion && !kind.anchor() {
                    let RowEvent::Recovery(pending) = &intent.ok_or_else(invalid)?.event else {
                        return Err(invalid());
                    };
                    if pending.node_id != event.node_id
                        || pending.root_id != event.root_id
                        || pending.movement != event.movement
                        || pending.node_digest != event.node_digest
                        || pending.recovery_plan_digest != event.recovery_plan_digest
                    {
                        return Err(invalid());
                    }
                    self.set_batch_recovery_pending(
                        before,
                        event.node_id.clone().ok_or_else(invalid)?,
                    )?;
                }
                self.apply_recovery(plan, event)
            }
        }
    }
    pub(super) fn apply_batch(&mut self, plan: &StoragePlan, e: &BatchEvent) -> io::Result<()> {
        let raw = encoded(e)?;
        if raw.len()>1024*1024||e.rows.is_empty()||e.rows.len()>512||e.count==0||e.count>512
            ||e.schema!=crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3||e.document_kind!="bounded-node-batch-event"
            ||e.storage_strategy!=ANCHORED_STORAGE_STRATEGY||e.sequence!=self.sequence.checked_add(1).ok_or_else(invalid)?
            ||e.session_id!=self.session||!uuid(&e.event_id)||self.event_ids.contains(&e.event_id)||!uuid(&e.batch_id)||!digest(&e.plan_digest)
            ||e.previous_event_digest!=self.tip||e.storage_plan_digest!=self.plan||plan.digest()?!=self.plan||e.security_policy_digest!=Policy::fixed().digest()?
            ||e.manifest_digest!=self.manifest||e.binding_digest!=self.binding||e.lease_epoch!=self.epoch||e.claim_key!=self.claim||e.deadline_scope!=self.deadline
            ||self.pending.is_some()||self.snapshot_pending.is_some()||self.action_pending.is_some()||self.history.is_some() {return Err(invalid());}
        let mut seen = BTreeSet::from([e.event_id.as_str()]);
        for row in &e.rows {
            if !uuid(row.event.id())
                || !seen.insert(row.event.id())
                || self.event_ids.contains(row.event.id())
                || !row.event.matches_phase(self, e)
            {
                return Err(invalid());
            }
        }
        // Entire DATA phase is atomic: errors in any later row cannot mutate
        // the caller's Progress. No native objects are ever copied here.
        let mut projected = self.data_projection()?;
        let previous_batch = projected.batch.take();
        let (intents, start) = match e.operation {
            BatchOperation::Intent | BatchOperation::AnchorsVerified => {
                if previous_batch.is_some()
                    || self.seen_batches.contains(&e.batch_id)
                    || e.kind.anchor() != (e.operation == BatchOperation::AnchorsVerified)
                    || e.count != e.rows.len()
                    || e.prefix != if e.kind.anchor() { e.count } else { 0 }
                    || e.plan_digest != hash::sha256_hex(&encoded(&e.rows)?)
                {
                    return Err(invalid());
                }
                let expected = projected.batch_expected_nodes(plan, e.kind)?;
                if expected.len() != e.count {
                    return Err(invalid());
                }
                for (i, (row, (root, node))) in e.rows.iter().zip(expected).enumerate() {
                    if row.ordinal != i || row.event.node()? != (root.as_str(), node.as_str()) {
                        return Err(invalid());
                    }
                }
                (None, 0)
            }
            BatchOperation::PrefixVerified => {
                let batch = previous_batch.ok_or_else(invalid)?;
                if e.kind.anchor()
                    || batch.batch_id != e.batch_id
                    || batch.kind != e.kind
                    || batch.plan_digest != e.plan_digest
                    || batch.rows.len() != e.count
                    || e.prefix <= batch.prefix
                    || e.prefix > e.count
                    || e.rows.len() != e.prefix - batch.prefix
                {
                    return Err(invalid());
                }
                for (i, row) in e.rows.iter().enumerate() {
                    if row.ordinal != batch.prefix + i
                        || row.event.node()? != batch.rows[row.ordinal].event.node()?
                    {
                        return Err(invalid());
                    }
                }
                let start = batch.prefix;
                (Some(batch.rows), start)
            }
        };
        for (i, row) in e.rows.iter().enumerate() {
            projected.sequence = self.sequence;
            projected.tip = self.tip.clone();
            projected.clear_batch_projection_pending();
            projected.apply_batch_row(
                plan,
                e.kind,
                row,
                e.operation != BatchOperation::Intent,
                intents.as_ref().map(|r| &r[start + i]),
            )?;
        }
        projected.clear_batch_projection_pending();
        projected.batch = if e.operation == BatchOperation::Intent
            || e.operation == BatchOperation::PrefixVerified && e.prefix < e.count
        {
            let rows = if let Some(rows) = intents {
                rows
            } else {
                serde_json::from_slice(&encoded(&e.rows)?).map_err(|_| invalid())?
            };
            Some(BatchProgress {
                batch_id: e.batch_id.clone(),
                kind: e.kind,
                plan_digest: e.plan_digest.clone(),
                rows,
                prefix: e.prefix,
            })
        } else {
            None
        };
        projected.event_ids.insert(e.event_id.clone());
        projected.seen_batches.insert(e.batch_id.clone());
        projected.sequence = e.sequence;
        projected.tip = hash::sha256_hex(&raw);
        *self = projected;
        Ok(())
    }
}
#[cfg(test)]
#[path = "batch_data_tests.rs"]
mod tests;
