//! Complete stopped-generation recovery DATA, in the existing phase chain.
//! Neither deserialization nor replay creates a native mutation capability.
use super::snapshots::{LifecyclePhase, SnapshotReference};
use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CurrentNode {
    relative: String,
    directory: bool,
    volume: u32,
    file_id: String,
    security: SnapshotData,
    bytes: usize,
    sha256: String,
    children: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CurrentTree {
    root: String,
    parent_volume: u32,
    parent_id: String,
    parent_security: SnapshotData,
    nodes: Vec<CurrentNode>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Retirement {
    relative: String,
    current_volume: u32,
    current_file_id: String,
    target: String,
    target_parent_id: String,
    source_parent_id: String,
    original_file_id: Option<String>,
    original_absent: bool,
    bytes: usize,
    sha256: String,
    security: SnapshotData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AbsentCurrent {
    relative: String,
    original_file_id: String,
    parent_id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecoveryRoot {
    schema: String,
    storage_strategy: String,
    original_tree: Root,
    current_tree: CurrentTree,
    current_snapshot: SnapshotReference,
    runtime_authority: bool,
    retirements: Vec<Retirement>,
    absent_current: Vec<AbsentCurrent>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecoveryPlan {
    roots: [RecoveryRoot; 2],
}
fn key(root: &str, relative: &str) -> String {
    hash::sha256_hex(format!("{root}/{relative}").as_bytes())
}
fn artifact(root: &str, relative: &str) -> bool {
    root == ".codex"
        && [
            "opencodex-catalog.json",
            "models_cache.json",
            "opencodex-catalog-audit.jsonl",
        ]
        .contains(&relative)
}
fn same<A: Serialize, B: Serialize>(a: &A, b: &B) -> io::Result<bool> {
    // Native frame digests use serde_json::Value's canonical key ordering.
    Ok(serde_json::to_value(a).map_err(|_| invalid())?
        == serde_json::to_value(b).map_err(|_| invalid())?)
}
impl RecoveryPlan {
    fn node_root(&self, id: &str) -> io::Result<String> {
        let matches = self
            .roots
            .iter()
            .filter(|r| {
                r.original_tree.nodes.iter().any(|n| n.node_id == id)
                    || r.retirements
                        .iter()
                        .any(|n| key(&r.original_tree.nodes[0].root_id, &n.relative) == id)
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(invalid());
        }
        Ok(matches[0].original_tree.nodes[0].root_id.clone())
    }
    fn anchor_release(&self, id: &str) -> io::Result<(String, RecoveryMove)> {
        let n = self
            .roots
            .iter()
            .flat_map(|r| &r.original_tree.nodes)
            .find(|n| n.node_id == id && n.kind != Kind::CustodiedFile)
            .ok_or_else(invalid)?;
        let path = if n.relative.is_empty() {
            n.root_id.clone()
        } else {
            format!("{}/{}", n.root_id, n.relative)
        };
        let expected = hash::sha256_hex(&encoded(&(self.digest()?, id, "OriginalAnchorReleased"))?);
        Ok((
            hash::sha256_hex(&encoded(n)?),
            RecoveryMove {
                from: path.clone(),
                to: path,
                native_id: n.file_id.clone(),
                volume: n.volume,
                parent_id: n.parent_id.clone(),
                expected_before: expected.clone(),
                expected_after: expected,
            },
        ))
    }
    fn movement(&self, id: &str, retire: bool) -> io::Result<(String, RecoveryMove)> {
        let plan = self.digest()?;
        let (row_digest, from, to, parent_id, native_id, volume, before, after) = if retire {
            let (r, n) = self
                .roots
                .iter()
                .flat_map(|r| r.retirements.iter().map(move |n| (r, n)))
                .find(|(r, n)| key(&r.original_tree.nodes[0].root_id, &n.relative) == id)
                .ok_or_else(invalid)?;
            let leaf = &r.original_tree.nodes[0].root_id;
            let target = Path::new(&n.target)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(invalid)?;
            (
                hash::sha256_hex(&encoded(n)?),
                format!("{leaf}/{}", n.relative),
                format!("custody/{leaf}/{target}"),
                n.target_parent_id.clone(),
                n.current_file_id.clone(),
                n.current_volume,
                "CurrentAtClient",
                "CurrentAtRetirement",
            )
        } else {
            let n = self
                .roots
                .iter()
                .flat_map(|r| &r.original_tree.nodes)
                .find(|n| n.node_id == id && n.kind == Kind::CustodiedFile)
                .ok_or_else(invalid)?;
            (
                hash::sha256_hex(&encoded(n)?),
                format!("custody/{}/original-{}", n.root_id, n.node_id),
                format!("{}/{}", n.root_id, n.relative),
                n.parent_id.clone(),
                n.file_id.clone(),
                n.volume,
                "OriginalAtCustody",
                "OriginalAtClient",
            )
        };
        Ok((
            row_digest,
            RecoveryMove {
                from,
                to,
                parent_id,
                native_id,
                volume,
                expected_before: hash::sha256_hex(&encoded(&(&plan, id, before))?),
                expected_after: hash::sha256_hex(&encoded(&(&plan, id, after))?),
            },
        ))
    }
    pub(super) fn from_native_data(roots: [serde_json::Value; 2]) -> io::Result<Self> {
        serde_json::from_value(serde_json::json!({"roots":roots})).map_err(|_| invalid())
    }
    pub(super) fn digest(&self) -> io::Result<String> {
        Ok(hash::sha256_hex(&encoded(self)?))
    }
    fn retirement_keys(&self) -> BTreeMap<String, u8> {
        self.roots
            .iter()
            .flat_map(|r| {
                r.retirements
                    .iter()
                    .map(|n| (key(&r.original_tree.nodes[0].root_id, &n.relative), 0))
            })
            .collect()
    }
    fn restore_keys(&self) -> BTreeMap<String, u8> {
        self.roots
            .iter()
            .flat_map(|r| {
                r.original_tree
                    .nodes
                    .iter()
                    .filter(|n| n.kind == Kind::CustodiedFile)
                    .map(|n| (n.node_id.clone(), 0))
            })
            .collect()
    }
    pub(super) fn validate(&self, plan: &StoragePlan, g: &Progress) -> io::Result<()> {
        self.validate_for_purpose(plan, g, SnapshotPurpose::Stopped)
    }
    pub(super) fn validate_for_purpose(
        &self,
        plan: &StoragePlan,
        g: &Progress,
        purpose: SnapshotPurpose,
    ) -> io::Result<()> {
        let frames = g.observed_frames.as_ref().ok_or_else(invalid)?;
        if g.linked_frames.get(&g.observed_sequence) != Some(frames) {
            return Err(invalid());
        }
        let mut identities = BTreeSet::new();
        for root in &plan.roots {
            identities.insert((root.nodes[0].volume, root.nodes[0].parent_id.clone()));
            identities.insert((root.custody_volume, root.custody_id.clone()));
            for n in &root.nodes {
                identities.insert((n.volume, n.file_id.clone()));
            }
        }
        let mut snapshot_hashes = Vec::new();
        let mut total_nodes = 0usize;
        let mut total_bytes = 0usize;
        for (index, r) in self.roots.iter().enumerate() {
            let original = &plan.roots[index];
            let leaf = &original.nodes[0].root_id;
            let frame = &r.current_tree;
            if r.schema != "windows-anchored-native-retirement-data-v3"
                || r.storage_strategy != ANCHORED_STORAGE_STRATEGY
                || r.runtime_authority
                || !same(&r.original_tree, original)?
                || frame.root != original.root
                || frame.parent_volume != original.nodes[0].volume
                || frame.parent_id != original.nodes[0].parent_id
                || !frame
                    .parent_security
                    .matches_id(frame.parent_volume, &frame.parent_id)
                || frame.nodes.is_empty()
                || frame.nodes[0].relative != ""
                || !frame.nodes[0].directory
                || frame
                    .nodes
                    .windows(2)
                    .any(|n| n[0].relative >= n[1].relative)
            {
                return Err(invalid());
            }
            frame.parent_security.validate()?;
            if self.roots.iter().any(|other| {
                !same(&other.current_tree.parent_security, &frame.parent_security).unwrap_or(false)
            }) {
                return Err(invalid());
            }
            let value = serde_json::to_value(frame).map_err(|_| invalid())?;
            if hash::sha256_hex(&encoded(&value)?) != frames[index] {
                return Err(invalid());
            }
            r.current_snapshot
                .validate(plan, index, g, purpose, g.observed_sequence, frames)?;
            // Archive objects may never alias an original, custody or another archive.
            for id in r.current_snapshot.object_ids() {
                if !identities.insert(id) {
                    return Err(invalid());
                }
            }
            snapshot_hashes.push(hash::sha256_hex(&encoded(&r.current_snapshot)?));
            let mut paths = BTreeMap::new();
            let mut case_paths = BTreeSet::new();
            let mut ordinary = Vec::new();
            for n in &frame.nodes {
                total_nodes += 1;
                total_bytes = total_bytes.checked_add(n.bytes).ok_or_else(invalid)?;
                if total_nodes > 512
                    || total_bytes > 32 * 1024 * 1024
                    || n.bytes > 2 * 1024 * 1024
                    || !n.relative.is_empty() && !relative(&n.relative)
                    || !case_paths.insert(n.relative.to_lowercase())
                    || !decimal(&n.file_id)
                    || n.volume != frame.parent_volume
                    || !n.security.matches_id(n.volume, &n.file_id)
                    || !digest(&n.sha256)
                {
                    return Err(invalid());
                }
                n.security.validate()?;
                let old = original.nodes.iter().find(|old| old.relative == n.relative);
                let anchor = old.is_some_and(|old| old.kind != Kind::CustodiedFile);
                if anchor {
                    let old = old.unwrap();
                    if n.directory != (old.kind == Kind::DirectoryAnchor)
                        || n.file_id != old.file_id
                        || n.security != old.security
                        || n.bytes != old.length
                        || n.sha256 != old.content_digest
                        || !n.directory && n.children != old.children
                    {
                        return Err(invalid());
                    }
                } else {
                    // Only already-observed exact three artifact leaves may be new.
                    // This is a data scope bound, never a content policy exception.
                    if n.directory
                        || old.is_none() && !artifact(leaf, &n.relative)
                        || !n.children.is_empty()
                        || !identities.insert((n.volume, n.file_id.clone()))
                    {
                        return Err(invalid());
                    }
                    if let Some(old) = old {
                        if !artifact(leaf, &n.relative)
                            && (g.nodes.get(&old.node_id).and_then(|n| n.1.as_ref())
                                != Some(&n.file_id)
                                || g.work_security.get(&old.node_id)
                                    != Some(&hash::sha256_hex(&encoded(&n.security)?))
                                || n.bytes != old.length
                                || n.sha256 != old.content_digest)
                        {
                            return Err(invalid());
                        }
                    }
                    if g.nodes.iter().any(|(id, (_, work))| {
                        work.as_ref() == Some(&n.file_id) && *id != key(leaf, &n.relative)
                    }) {
                        return Err(invalid());
                    }
                    ordinary.push(n);
                }
                paths.insert(n.relative.as_str(), n);
            }
            let absent = original
                .nodes
                .iter()
                .filter(|n| !paths.contains_key(n.relative.as_str()))
                .collect::<Vec<_>>();
            if absent.len() != r.absent_current.len() || ordinary.len() != r.retirements.len() {
                return Err(invalid());
            }
            for (n, a) in absent.iter().zip(&r.absent_current) {
                if n.kind != Kind::CustodiedFile
                    || !artifact(leaf, &n.relative)
                    || a.relative != n.relative
                    || a.original_file_id != n.file_id
                    || a.parent_id != n.parent_id
                {
                    return Err(invalid());
                }
            }
            for n in &frame.nodes {
                if !n.relative.is_empty() {
                    let parent = Path::new(&n.relative)
                        .parent()
                        .and_then(|p| p.to_str())
                        .and_then(|p| paths.get(p))
                        .ok_or_else(invalid)?;
                    if !parent.directory {
                        return Err(invalid());
                    }
                }
                if n.directory {
                    let children = frame
                        .nodes
                        .iter()
                        .filter(|c| {
                            !c.relative.is_empty()
                                && Path::new(&c.relative).parent() == Some(Path::new(&n.relative))
                        })
                        .map(|c| {
                            Path::new(&c.relative)
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string()
                        })
                        .collect::<Vec<_>>();
                    if n.children != children || n.bytes != 0 || n.sha256 != hash::sha256_hex(b"") {
                        return Err(invalid());
                    }
                }
            }
            for (n, row) in ordinary.iter().zip(&r.retirements) {
                let old = original.nodes.iter().find(|old| old.relative == n.relative);
                let parent = Path::new(&n.relative)
                    .parent()
                    .and_then(|p| p.to_str())
                    .and_then(|p| paths.get(p))
                    .ok_or_else(invalid)?;
                let target = Path::new(&original.custody).join(format!(
                    "retired-current-{}-{}",
                    key(leaf, &n.relative),
                    n.file_id
                ));
                if row.relative != n.relative
                    || row.current_volume != n.volume
                    || row.current_file_id != n.file_id
                    || Path::new(&row.target) != target
                    || row.target_parent_id != original.custody_id
                    || row.source_parent_id != parent.file_id
                    || row.original_file_id.as_ref() != old.map(|n| &n.file_id)
                    || row.original_absent != old.is_none()
                    || row.bytes != n.bytes
                    || row.sha256 != n.sha256
                    || row.security != n.security
                {
                    return Err(invalid());
                }
                validate_local_path(Path::new(&row.target))?;
            }
        }
        if g.snapshot_links.get(&purpose).map(|x| x.as_slice()) != Some(snapshot_hashes.as_slice())
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum RecoveryOperation {
    RecoveryPlanIntent,
    RecoveryPlanVerified,
    WorkRetireIntent,
    WorkRetiredVerified,
    OriginalRestoreIntent,
    OriginalRestoredVerified,
    WholeTreeRestoreIntent,
    WholeTreeRestoredVerified,
    ReleaseVerified,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecoveryMove {
    from: String,
    to: String,
    native_id: String,
    volume: u32,
    parent_id: String,
    expected_before: String,
    expected_after: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecoveryEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: RecoveryOperation,
    pub(super) plan: Option<RecoveryPlan>,
    pub(super) recovery_plan_digest: String,
    pub(super) node_id: Option<String>,
    pub(super) root_id: Option<String>,
    pub(super) node_digest: Option<String>,
    pub(super) movement: Option<RecoveryMove>,
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
pub(super) struct RecoveryProgress {
    plan: RecoveryPlan,
    verified: bool,
    retired: BTreeMap<String, u8>,
    restored: BTreeMap<String, u8>,
    pending: Option<(RecoveryOperation, String)>,
    whole_tree: u8,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecoverySummary {
    pub(super) plan_digest: String,
    progress_digest: String,
    plan_verified: bool,
    retired_count: usize,
    restored_count: usize,
    pending: Option<(RecoveryOperation, String)>,
    whole_tree: u8,
    anchor_released_count: usize,
}
impl Progress {
    pub(super) fn batch_recovery_nodes(&self, retire: bool) -> io::Result<Vec<(String, String)>> {
        let r = self.recovery.as_ref().ok_or_else(invalid)?;
        if r.pending.is_some() {
            return Err(invalid());
        }
        Ok(r.plan
            .roots
            .iter()
            .flat_map(|r| {
                let root = r.original_tree.nodes[0].root_id.clone();
                if retire {
                    r.retirements
                        .iter()
                        .map(|n| (root.clone(), key(&root, &n.relative)))
                        .collect::<Vec<_>>()
                } else {
                    r.original_tree
                        .nodes
                        .iter()
                        .filter(|n| n.kind == Kind::CustodiedFile)
                        .map(|n| (root.clone(), n.node_id.clone()))
                        .collect()
                }
            })
            .collect())
    }
    pub(super) fn clear_batch_projection_pending(&mut self) {
        self.pending = None;
        if let Some(r) = &mut self.recovery {
            r.pending = None;
        }
    }
    pub(super) fn set_batch_recovery_pending(
        &mut self,
        operation: RecoveryOperation,
        node: String,
    ) -> io::Result<()> {
        self.recovery.as_mut().ok_or_else(invalid)?.pending = Some((operation, node));
        Ok(())
    }
    pub(super) fn recovery_reserved_identities(&self) -> io::Result<BTreeSet<(u32, String)>> {
        let r = self.recovery.as_ref().ok_or_else(invalid)?;
        let mut ids = self.snapshot_objects.clone();
        for root in &r.plan.roots {
            ids.insert((
                root.current_tree.parent_volume,
                root.current_tree.parent_id.clone(),
            ));
            ids.insert((
                root.original_tree.custody_volume,
                root.original_tree.custody_id.clone(),
            ));
            for n in &root.original_tree.nodes {
                ids.insert((n.volume, n.file_id.clone()));
            }
            for n in &root.current_tree.nodes {
                ids.insert((n.volume, n.file_id.clone()));
            }
            for id in root.current_snapshot.object_ids() {
                ids.insert(id);
            }
        }
        for (id, (_, work)) in &self.nodes {
            if let Some(work) = work {
                let root = r
                    .plan
                    .roots
                    .iter()
                    .find(|r| r.original_tree.nodes.iter().any(|n| n.node_id == *id))
                    .ok_or_else(invalid)?;
                ids.insert((root.original_tree.custody_volume, work.clone()));
            }
        }
        Ok(ids)
    }
    pub(super) fn original_history_frames(
        &self,
        plan: &StoragePlan,
    ) -> io::Result<[serde_json::Value; 2]> {
        let r = self.recovery.as_ref().ok_or_else(invalid)?;
        if !self.complete_files(plan) || self.lease_state != segments::LeaseState::Released {
            return Err(invalid());
        }
        Ok(std::array::from_fn(|index| {
            let root = &plan.roots[index];
            let current = &r.plan.roots[index].current_tree;
            serde_json::json!({"root":root.root,"parentVolume":current.parent_volume,"parentId":current.parent_id,
                "parentSecurity":current.parent_security,"nodes":root.nodes.iter().map(|n|serde_json::json!({
                    "relative":n.relative,"directory":n.kind==Kind::DirectoryAnchor,"volume":n.volume,"fileId":n.file_id,
                    "security":n.security,"bytes":n.length,"sha256":n.content_digest,"children":n.children})).collect::<Vec<_>>()})
        }))
    }
    pub(super) fn retirement_history_digest(&self) -> io::Result<String> {
        self.recovery.as_ref().ok_or_else(invalid)?.plan.digest()
    }
    pub(super) fn recovery_event(
        &self,
        operation: RecoveryOperation,
        plan: Option<&RecoveryPlan>,
        node: Option<&str>,
    ) -> io::Result<RecoveryEvent> {
        let p = plan
            .or_else(|| self.recovery.as_ref().map(|r| &r.plan))
            .ok_or_else(invalid)?;
        let movement = node
            .map(|id| {
                if operation == RecoveryOperation::ReleaseVerified {
                    p.anchor_release(id)
                } else {
                    p.movement(
                        id,
                        matches!(
                            operation,
                            RecoveryOperation::WorkRetireIntent
                                | RecoveryOperation::WorkRetiredVerified
                        ),
                    )
                }
            })
            .transpose()?;
        let (node_digest, movement) = match movement {
            Some((d, m)) => (Some(d), Some(m)),
            None => (None, None),
        };
        Ok(RecoveryEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "stopped-recovery-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            operation,
            plan: plan
                .map(|p| serde_json::from_slice(&encoded(p)?).map_err(|_| invalid()))
                .transpose()?,
            recovery_plan_digest: p.digest()?,
            node_id: node.map(str::to_string),
            root_id: node.map(|id| p.node_root(id)).transpose()?,
            node_digest,
            movement,
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
    pub(super) fn recovery_summary(&self) -> io::Result<Option<RecoverySummary>> {
        self.recovery
            .as_ref()
            .map(|r| {
                let anchors = r
                    .plan
                    .roots
                    .iter()
                    .flat_map(|r| &r.original_tree.nodes)
                    .filter(|n| n.kind != Kind::CustodiedFile)
                    .map(|n| (n.node_id.clone(), self.nodes[&n.node_id].0))
                    .collect::<BTreeMap<_, _>>();
                Ok(RecoverySummary {
                    plan_digest: r.plan.digest()?,
                    progress_digest: hash::sha256_hex(&encoded(&(r, &anchors))?),
                    plan_verified: r.verified,
                    retired_count: r.retired.values().filter(|s| **s == 2).count(),
                    restored_count: r.restored.values().filter(|s| **s == 2).count(),
                    pending: r.pending.clone(),
                    whole_tree: r.whole_tree,
                    anchor_released_count: anchors.values().filter(|state| **state == 2).count(),
                })
            })
            .transpose()
    }
    pub(super) fn apply_recovery(
        &mut self,
        plan: &StoragePlan,
        e: &RecoveryEvent,
    ) -> io::Result<()> {
        if self.batch.is_some() {
            return Err(invalid());
        }
        use RecoveryOperation::*;
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "stopped-recovery-event"
            || e.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || e.sequence != self.sequence.checked_add(1).ok_or_else(invalid)?
            || e.session_id != self.session
            || !uuid(&e.event_id)
            || self.event_ids.contains(&e.event_id)
            || e.previous_event_digest != self.tip
            || e.storage_plan_digest != self.plan
            || plan.digest()? != self.plan
            || e.security_policy_digest != Policy::fixed().digest()?
            || e.manifest_digest != self.manifest
            || e.binding_digest != self.binding
            || e.lease_epoch != self.epoch
            || e.claim_key != self.claim
            || e.deadline_scope != self.deadline
            || !digest(&e.recovery_plan_digest)
            || self.lease_state
                != if e.operation == ReleaseVerified {
                    segments::LeaseState::ReleasePrepared
                } else {
                    segments::LeaseState::Active
                }
            || self.work_mode != observations::WorkMode::Observed
            || self.lifecycle != LifecyclePhase::Stopped
            || self.pending.is_some()
            || self.snapshot_pending.is_some()
            || self.action_pending.is_some()
            || !matches!(
                self.action_kind,
                actions::ActionKind::Stop | actions::ActionKind::Restart
            )
        {
            return Err(invalid());
        }
        let bytes = encoded(e)?;
        if let Some(id) = &e.node_id {
            if e.root_id.as_ref()
                != Some(
                    &self
                        .recovery
                        .as_ref()
                        .ok_or_else(invalid)?
                        .plan
                        .node_root(id)?,
                )
            {
                return Err(invalid());
            }
        } else if e.root_id.is_some() {
            return Err(invalid());
        }
        if e.operation == RecoveryPlanIntent {
            let p = e.plan.as_ref().ok_or_else(invalid)?;
            if self.recovery.is_some()
                || e.node_id.is_some()
                || e.node_digest.is_some()
                || e.movement.is_some()
                || e.recovery_plan_digest != p.digest()?
                || !plan.roots.iter().flat_map(|r| &r.nodes).all(|n| {
                    self.nodes[&n.node_id].0 == if n.kind == Kind::CustodiedFile { 4 } else { 1 }
                })
            {
                return Err(invalid());
            }
            p.validate(plan, self)?;
            // Independent owned DATA copy; no native objects or authority are reconstructed.
            let copy: RecoveryPlan = serde_json::from_slice(&encoded(p)?).map_err(|_| invalid())?;
            self.recovery = Some(RecoveryProgress {
                retired: copy.retirement_keys(),
                restored: copy.restore_keys(),
                plan: copy,
                verified: false,
                pending: None,
                whole_tree: 0,
            });
        } else {
            let r = self.recovery.as_mut().ok_or_else(invalid)?;
            if e.plan.is_some() || e.recovery_plan_digest != r.plan.digest()? {
                return Err(invalid());
            }
            if e.operation == RecoveryPlanVerified {
                if r.verified
                    || r.pending.is_some()
                    || e.node_id.is_some()
                    || e.node_digest.is_some()
                    || e.movement.is_some()
                {
                    return Err(invalid());
                }
                r.verified = true;
            } else if e.operation == ReleaseVerified {
                let id = e.node_id.as_ref().ok_or_else(invalid)?;
                let (expected, movement) = r.plan.anchor_release(id)?;
                if !r.verified
                    || r.whole_tree != 2
                    || r.pending.is_some()
                    || r.retired.values().any(|s| *s != 2)
                    || r.restored.values().any(|s| *s != 2)
                    || e.node_digest.as_ref() != Some(&expected)
                    || e.movement.as_ref() != Some(&movement)
                    || self.nodes.get(id).map(|n| n.0) != Some(1)
                {
                    return Err(invalid());
                }
                self.nodes.get_mut(id).ok_or_else(invalid)?.0 = 2;
            } else if matches!(
                e.operation,
                WholeTreeRestoreIntent | WholeTreeRestoredVerified
            ) {
                if !r.verified
                    || r.retired.values().any(|s| *s != 2)
                    || r.restored.values().any(|s| *s != 2)
                    || e.node_id.is_some()
                    || e.node_digest.is_some()
                    || e.movement.is_some()
                    || !plan.roots.iter().flat_map(|r| &r.nodes).all(|n| {
                        self.nodes[&n.node_id].0
                            == if n.kind == Kind::CustodiedFile { 8 } else { 1 }
                    })
                {
                    return Err(invalid());
                }
                if e.operation == WholeTreeRestoreIntent {
                    if r.whole_tree != 0 || r.pending.is_some() {
                        return Err(invalid());
                    }
                    r.whole_tree = 1;
                    r.pending = Some((WholeTreeRestoreIntent, e.recovery_plan_digest.clone()));
                } else {
                    if r.whole_tree != 1
                        || r.pending.as_ref()
                            != Some(&(WholeTreeRestoreIntent, e.recovery_plan_digest.clone()))
                    {
                        return Err(invalid());
                    }
                    r.whole_tree = 2;
                    r.pending = None;
                }
            } else {
                let id = e.node_id.as_ref().ok_or_else(invalid)?;
                if !r.verified || r.whole_tree != 0 {
                    return Err(invalid());
                }
                let (_, expected_move) = r.plan.movement(
                    id,
                    matches!(e.operation, WorkRetireIntent | WorkRetiredVerified),
                )?;
                if e.movement.as_ref() != Some(&expected_move) {
                    return Err(invalid());
                }
                let (map, intent, result, expected) = match e.operation {
                    WorkRetireIntent | WorkRetiredVerified => {
                        let row = r
                            .plan
                            .roots
                            .iter()
                            .flat_map(|r| r.retirements.iter().map(move |n| (r, n)))
                            .find(|(root, n)| {
                                key(&root.original_tree.nodes[0].root_id, &n.relative) == *id
                            })
                            .ok_or_else(invalid)?;
                        (
                            &mut r.retired,
                            WorkRetireIntent,
                            WorkRetiredVerified,
                            hash::sha256_hex(&encoded(row.1)?),
                        )
                    }
                    OriginalRestoreIntent | OriginalRestoredVerified => {
                        if r.retired.values().any(|s| *s != 2) {
                            return Err(invalid());
                        }
                        let node = plan
                            .roots
                            .iter()
                            .flat_map(|r| &r.nodes)
                            .find(|n| n.node_id == *id && n.kind == Kind::CustodiedFile)
                            .ok_or_else(invalid)?;
                        (
                            &mut r.restored,
                            OriginalRestoreIntent,
                            OriginalRestoredVerified,
                            hash::sha256_hex(&encoded(node)?),
                        )
                    }
                    _ => return Err(invalid()),
                };
                if e.node_digest.as_ref() != Some(&expected) {
                    return Err(invalid());
                }
                let state = map.get_mut(id).ok_or_else(invalid)?;
                if e.operation == intent {
                    if *state != 0 || r.pending.is_some() {
                        return Err(invalid());
                    }
                    *state = 1;
                    r.pending = Some((intent, id.clone()));
                } else if e.operation == result {
                    if *state != 1 || r.pending.as_ref() != Some(&(intent, id.clone())) {
                        return Err(invalid());
                    }
                    *state = 2;
                    r.pending = None;
                    if result == OriginalRestoredVerified {
                        self.nodes.get_mut(id).ok_or_else(invalid)?.0 = 8;
                    }
                } else {
                    return Err(invalid());
                }
            }
        }
        self.sequence = e.sequence;
        self.tip = hash::sha256_hex(&bytes);
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
    pub(super) fn recovery_tree_ready_for_release(&self, plan: &StoragePlan) -> bool {
        self.recovery.as_ref().is_some_and(|r| {
            r.verified
                && r.whole_tree == 2
                && r.pending.is_none()
                && r.retired.values().all(|s| *s == 2)
                && r.restored.values().all(|s| *s == 2)
        }) && self.lifecycle == LifecyclePhase::Stopped
            && self.work_mode == observations::WorkMode::Observed
            && self.pending.is_none()
            && self.action_pending.is_none()
            && self.snapshot_pending.is_none()
            && plan.roots.iter().flat_map(|r| &r.nodes).all(|n| {
                if n.kind == Kind::CustodiedFile {
                    self.nodes[&n.node_id].0 == 8
                } else {
                    matches!(self.nodes[&n.node_id].0, 1 | 2)
                }
            })
    }
}
#[cfg(test)]
#[path = "recovery_data_tests.rs"]
pub(super) mod tests;
