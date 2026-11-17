//! Strict r6 plan/progress DATA. Parsing never creates original handles, a
//! lease, a native session or Ready/Closed authority; no v2 conversion exists.
use super::{
    bounded::{Policy, SnapshotData},
    phase::{digest, invalid},
};
use crate::{
    infrastructure::{
        hash,
        windows_official_runtime024::{validate_local_path, FileIdentity},
    },
    modules::runtime::windows_lifecycle024_contract::{
        ANCHORED_CODEX_STABLE_FILES, ANCHORED_STORAGE_STRATEGY,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Component, Path},
};
mod actions;
mod capture;
mod closed;
pub(super) use capture::CaptureSlot;
pub(super) use capture::NativePredecessors;
pub(super) mod batches;
mod handle_rows;
mod handles;
mod history;
pub(super) mod journal;
mod observations;
mod predecessors;
mod qualification;
mod read_window;
mod recovery;
mod route_close;
mod segments;
mod snapshots;
pub(super) use journal::Journal;
use snapshots::SnapshotPurpose;
#[cfg(test)]
pub(in crate::infrastructure::windows_client_originals024) fn validate_route_close_component(
    value: serde_json::Value,
) -> io::Result<String> {
    let data: route_close::RouteCloseData = serde_json::from_value(value).map_err(|_| invalid())?;
    data.validate()?;
    data.digest()
}
#[derive(Clone, Copy, Debug, Serialize)]
pub(super) enum FileAction {
    MoveOriginal,
    CreateWork,
    RetireWork,
    RestoreOriginal,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Kind {
    DirectoryAnchor,
    CoordinatorAnchor,
    CustodiedFile,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Node {
    node_id: String,
    root_id: String,
    relative: String,
    kind: Kind,
    volume: u32,
    file_id: String,
    parent_id: String,
    security: SnapshotData,
    children: Vec<String>,
    length: usize,
    content_digest: String,
    target: Option<String>,
}
#[cfg(test)]
#[path = "v3_tests.rs"]
mod tests;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Root {
    root: String,
    custody: String,
    custody_volume: u32,
    custody_id: String,
    custody_security: SnapshotData,
    nodes: Vec<Node>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct StoragePlan {
    storage_strategy: String,
    roots: Vec<Root>,
    #[serde(skip)]
    data_digest: std::sync::OnceLock<String>,
}
fn decimal(s: &str) -> bool {
    s.parse::<u64>().is_ok_and(|n| n > 0 && n.to_string() == s)
}
fn relative(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(['\\', ':'])
        && Path::new(s)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && Path::new(s)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
            == s
        && validate_local_path(&Path::new("C:/").join(s)).is_ok()
}
impl StoragePlan {
    pub(super) fn from_native_data(
        roots: Vec<serde_json::Value>,
        profile: &Path,
        parent: FileIdentity,
    ) -> io::Result<Self> {
        let roots = roots
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()
            .map_err(|_| invalid())?;
        let out = Self {
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            roots,
            data_digest: std::sync::OnceLock::new(),
        };
        out.validate(profile, parent)?;
        Ok(out)
    }
    pub(super) fn digest(&self) -> io::Result<String> {
        if let Some(digest) = self.data_digest.get() {
            return Ok(digest.clone());
        }
        let digest = hash::sha256_hex(&serde_json::to_vec(self).map_err(|_| invalid())?);
        let _ = self.data_digest.set(digest.clone());
        Ok(digest)
    }
    pub(super) fn validate(&self, profile: &Path, parent: FileIdentity) -> io::Result<()> {
        if self.storage_strategy != ANCHORED_STORAGE_STRATEGY || self.roots.len() != 2 {
            return Err(invalid());
        }
        let mut seen_ids = BTreeSet::from([(parent.volume, parent.index.to_string())]);
        let mut seen_nodes = BTreeSet::new();
        let mut total = 0usize;
        let mut count = 0;
        let mut custody_paths = BTreeSet::new();
        for (root, leaf) in self.roots.iter().zip([".codex", ".grok"]) {
            if Path::new(&root.root) != profile.join(leaf)
                || Path::new(&root.custody).parent() != Some(profile)
                || [profile.join(".codex"), profile.join(".grok")]
                    .contains(&Path::new(&root.custody).to_path_buf())
                || !custody_paths.insert(root.custody.to_lowercase())
                || root.custody_volume != parent.volume
                || !decimal(&root.custody_id)
                || !root
                    .custody_security
                    .matches_id(root.custody_volume, &root.custody_id)
                || root.nodes.is_empty()
            {
                return Err(invalid());
            }
            validate_local_path(Path::new(&root.custody))?;
            root.custody_security.validate()?;
            if !seen_ids.insert((root.custody_volume, root.custody_id.clone())) {
                return Err(invalid());
            }
            let mut paths = BTreeMap::new();
            let mut case_paths = BTreeSet::new();
            for n in &root.nodes {
                count += 1;
                if count > 512 {
                    return Err(invalid());
                }
                if n.root_id != leaf
                    || !n.relative.is_empty() && !relative(&n.relative)
                    || !case_paths.insert(n.relative.to_lowercase())
                    || !decimal(&n.file_id)
                    || !decimal(&n.parent_id)
                    || n.volume != parent.volume
                    || !n.security.matches_id(n.volume, &n.file_id)
                    || !seen_ids.insert((n.volume, n.file_id.clone()))
                    || !seen_nodes.insert(n.node_id.clone())
                    || n.node_id != hash::sha256_hex(format!("{leaf}/{}", n.relative).as_bytes())
                    || !digest(&n.content_digest)
                {
                    return Err(invalid());
                }
                n.security.validate()?;
                paths.insert(n.relative.as_str(), n);
            }
            if root
                .nodes
                .windows(2)
                .any(|p| p[0].relative >= p[1].relative)
                || root.nodes[0].relative != ""
                || root.nodes[0].kind != Kind::DirectoryAnchor
            {
                return Err(invalid());
            }
            for n in &root.nodes {
                let expected_parent = if n.relative.is_empty() {
                    parent.index.to_string()
                } else {
                    let p = Path::new(&n.relative)
                        .parent()
                        .and_then(|p| p.to_str())
                        .and_then(|s| paths.get(s))
                        .ok_or_else(invalid)?;
                    if p.kind != Kind::DirectoryAnchor {
                        return Err(invalid());
                    }
                    p.file_id.clone()
                };
                if n.parent_id != expected_parent {
                    return Err(invalid());
                }
                let coordinator =
                    leaf == ".codex" && ANCHORED_CODEX_STABLE_FILES.contains(&n.relative.as_str());
                if coordinator != (n.kind == Kind::CoordinatorAnchor)
                    || n.relative.is_empty() && n.kind != Kind::DirectoryAnchor
                {
                    return Err(invalid());
                }
                if n.kind == Kind::DirectoryAnchor {
                    let expected = root
                        .nodes
                        .iter()
                        .filter(|x| {
                            !x.relative.is_empty()
                                && Path::new(&x.relative).parent() == Some(Path::new(&n.relative))
                        })
                        .map(|x| {
                            Path::new(&x.relative)
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string()
                        })
                        .collect::<Vec<_>>();
                    if n.target.is_some()
                        || n.children != expected
                        || n.length != 0
                        || n.content_digest != hash::sha256_hex(b"")
                    {
                        return Err(invalid());
                    }
                } else {
                    total = total.checked_add(n.length).ok_or_else(invalid)?;
                    if !n.children.is_empty()
                        || n.length > 2 * 1024 * 1024
                        || total > 32 * 1024 * 1024
                    {
                        return Err(invalid());
                    }
                    let target =
                        (n.kind == Kind::CustodiedFile).then(|| format!("original-{}", n.node_id));
                    if n.target != target {
                        return Err(invalid());
                    }
                }
            }
            if leaf == ".codex"
                && ANCHORED_CODEX_STABLE_FILES
                    .iter()
                    .any(|p| !paths.contains_key(p))
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Operation {
    SnapshotVerified,
    NodeMoveIntent,
    NodeMovedVerified,
    WorkCreateIntent,
    WorkCreatedVerified,
    WorkRetireIntent,
    WorkRetiredVerified,
    OriginalRestoreIntent,
    OriginalRestoredVerified,
    ReleaseVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Event {
    schema: String,
    document_kind: String,
    storage_strategy: String,
    event_id: String,
    sequence: u32,
    session_id: String,
    node_id: String,
    root_id: String,
    operation: Operation,
    from: String,
    to: String,
    native_id: Option<String>,
    native_security: Option<SnapshotData>,
    parent_id: String,
    expected_before: String,
    expected_after: String,
    lease_epoch: String,
    claim_key: String,
    deadline_scope: String,
    previous_event_digest: String,
    storage_plan_digest: String,
    security_policy_digest: String,
    manifest_digest: String,
    binding_digest: String,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Progress {
    session: String,
    plan: String,
    manifest: String,
    binding: String,
    sequence: u32,
    tip: String,
    nodes: BTreeMap<String, (u8, Option<String>)>,
    event_ids: BTreeSet<String>,
    pending: Option<Vec<u8>>,
    work_security: BTreeMap<String, String>,
    epoch: String,
    claim: String,
    deadline: String,
    lease_state: segments::LeaseState,
    lease_objects: Option<String>,
    seen_epochs: BTreeSet<String>,
    work_mode: observations::WorkMode,
    work_frames: Option<[String; 2]>,
    observed_frames: Option<[String; 2]>,
    snapshot_pending: Option<(SnapshotPurpose, u32)>,
    snapshot_links: BTreeMap<SnapshotPurpose, [String; 2]>,
    snapshot_stages: BTreeMap<SnapshotPurpose, (u32, String)>,
    snapshot_objects: BTreeSet<(u32, String)>,
    observed_sequence: u32,
    linked_frames: BTreeMap<u32, [String; 2]>,
    lifecycle: snapshots::LifecyclePhase,
    ready_generation: Option<(u32, u16, String)>,
    action_pending: Option<(String, actions::ActionKind)>,
    action_kind: actions::ActionKind,
    seen_actions: BTreeSet<String>,
    recovery: Option<recovery::RecoveryProgress>,
    history: Option<history::HistoryProgress>,
    handle_progress: Option<handles::HandleSummary>,
    handle_close: Option<(u32, String)>,
    batch: Option<batches::BatchProgress>,
    seen_batches: BTreeSet<String>,
    content: Option<qualification::ContentSummary>,
    closed: Option<closed::ClosedSummary>,
    route_close: Option<route_close::RouteCloseSummary>,
}
fn uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok_and(|u| !u.is_nil() && u.to_string() == s)
}
fn encoded<T: Serialize>(v: &T) -> io::Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| invalid())
}
impl Progress {
    pub(super) fn new(
        plan: &StoragePlan,
        session: &str,
        manifest: &str,
        binding: &str,
        epoch: &str,
        claim: &str,
        deadline: &str,
    ) -> io::Result<Self> {
        if !uuid(session)
            || !digest(manifest)
            || !digest(binding)
            || !uuid(epoch)
            || !digest(claim)
            || !uuid(deadline)
        {
            return Err(invalid());
        }
        Ok(Self {
            session: session.into(),
            plan: plan.digest()?,
            manifest: manifest.into(),
            binding: binding.into(),
            sequence: 0,
            tip: manifest.into(),
            nodes: plan
                .roots
                .iter()
                .flat_map(|r| r.nodes.iter().map(|n| (n.node_id.clone(), (0, None))))
                .collect(),
            event_ids: BTreeSet::new(),
            pending: None,
            work_security: BTreeMap::new(),
            epoch: epoch.into(),
            claim: claim.into(),
            deadline: deadline.into(),
            lease_state: segments::LeaseState::Active,
            lease_objects: None,
            seen_epochs: BTreeSet::from([epoch.to_string()]),
            work_mode: observations::WorkMode::Sealed,
            work_frames: None,
            observed_frames: None,
            snapshot_pending: None,
            snapshot_links: BTreeMap::new(),
            snapshot_stages: BTreeMap::new(),
            snapshot_objects: BTreeSet::new(),
            observed_sequence: 0,
            linked_frames: BTreeMap::new(),
            lifecycle: snapshots::LifecyclePhase::Preparing,
            ready_generation: None,
            action_pending: None,
            action_kind: actions::ActionKind::Start,
            seen_actions: BTreeSet::from([deadline.to_string()]),
            recovery: None,
            history: None,
            handle_progress: None,
            handle_close: None,
            batch: None,
            seen_batches: BTreeSet::new(),
            content: None,
            closed: None,
            route_close: None,
        })
    }
    pub(super) fn apply(&mut self, plan: &StoragePlan, e: &Event) -> io::Result<()> {
        if self.batch.is_some() {
            return Err(invalid());
        }
        if self.recovery.is_some() {
            return Err(invalid());
        }
        if self.lease_state!=segments::LeaseState::Active || self.work_mode!=observations::WorkMode::Sealed || self.snapshot_pending.is_some() || self.action_pending.is_some() || e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "node-event"
            || e.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || e.sequence != self.sequence.checked_add(1).ok_or_else(invalid)?
            || e.session_id != self.session
            || e.storage_plan_digest != self.plan
            || plan.digest()? != self.plan
            || e.security_policy_digest != Policy::fixed().digest()?
            || e.manifest_digest != self.manifest
            || e.binding_digest != self.binding
            || e.previous_event_digest != self.tip
            || !uuid(&e.event_id)
            || self.event_ids.contains(&e.event_id)
            || e.lease_epoch != self.epoch
            || e.claim_key != self.claim
            || e.deadline_scope != self.deadline
            || !digest(&e.expected_before)
            || !digest(&e.expected_after)
        {
            return Err(invalid());
        }
        let n = plan
            .roots
            .iter()
            .flat_map(|r| &r.nodes)
            .find(|n| n.node_id == e.node_id && n.root_id == e.root_id)
            .ok_or_else(invalid)?;
        let &(state, ref work) = self.nodes.get(&n.node_id).ok_or_else(invalid)?;
        let path = if n.relative.is_empty() {
            n.root_id.clone()
        } else {
            format!("{}/{}", n.root_id, n.relative)
        };
        let custody = format!("custody/{}/original-{}", n.root_id, n.node_id);
        let retired = format!("retired/{}/retired-{}", n.root_id, n.node_id);
        let (next, from, to, parent) = if n.kind != Kind::CustodiedFile {
            match (state, e.operation) {
                (0, Operation::SnapshotVerified) => {
                    (1, path.clone(), path.clone(), n.parent_id.clone())
                }
                (1, Operation::ReleaseVerified)
                    if plan
                        .roots
                        .iter()
                        .flat_map(|r| &r.nodes)
                        .filter(|n| n.kind == Kind::CustodiedFile)
                        .all(|n| self.nodes[&n.node_id].0 == 8) =>
                {
                    (2, path.clone(), path.clone(), n.parent_id.clone())
                }
                _ => return Err(invalid()),
            }
        } else {
            let root = plan
                .roots
                .iter()
                .find(|r| r.root.ends_with(&n.root_id))
                .ok_or_else(invalid)?;
            match (state, e.operation) {
                (0, Operation::NodeMoveIntent) => {
                    (1, path.clone(), custody.clone(), root.custody_id.clone())
                }
                (1, Operation::NodeMovedVerified) => {
                    (2, path.clone(), custody.clone(), root.custody_id.clone())
                }
                (2, Operation::WorkCreateIntent) => {
                    (3, "Absent".into(), path.clone(), n.parent_id.clone())
                }
                (3, Operation::WorkCreatedVerified) => {
                    (4, "Absent".into(), path.clone(), n.parent_id.clone())
                }
                (4, Operation::WorkRetireIntent) => {
                    (5, path.clone(), retired.clone(), root.custody_id.clone())
                }
                (5, Operation::WorkRetiredVerified) => {
                    (6, path.clone(), retired.clone(), root.custody_id.clone())
                }
                (6, Operation::OriginalRestoreIntent) => {
                    (7, custody.clone(), path.clone(), n.parent_id.clone())
                }
                (7, Operation::OriginalRestoredVerified) => {
                    (8, custody.clone(), path.clone(), n.parent_id.clone())
                }
                _ => return Err(invalid()),
            }
        };
        if e.from != from || e.to != to || e.parent_id != parent {
            return Err(invalid());
        }
        let mut new_work = work.clone();
        if matches!(e.operation, Operation::WorkCreatedVerified) {
            if !e.native_id.as_deref().is_some_and(decimal)
                || e.native_id.as_deref() == Some(&n.file_id)
            {
                return Err(invalid());
            }
            new_work = e.native_id.clone();
        } else if matches!(e.operation, Operation::WorkCreateIntent) {
            if e.native_id.is_some() {
                return Err(invalid());
            }
        } else if matches!(
            e.operation,
            Operation::WorkRetireIntent | Operation::WorkRetiredVerified
        ) {
            if e.native_id != *work {
                return Err(invalid());
            }
        } else if e.native_id.as_deref() != Some(&n.file_id) {
            return Err(invalid());
        }
        let mut new_security = None;
        if e.operation == Operation::WorkCreateIntent {
            if e.native_security.is_some() {
                return Err(invalid());
            }
        } else {
            let security = e.native_security.as_ref().ok_or_else(invalid)?;
            security.validate()?;
            if !security.matches_id(n.volume, e.native_id.as_deref().ok_or_else(invalid)?) {
                return Err(invalid());
            }
            let key = hash::sha256_hex(&encoded(security)?);
            if e.operation == Operation::WorkCreatedVerified {
                new_security = Some(key);
            } else if matches!(
                e.operation,
                Operation::WorkRetireIntent | Operation::WorkRetiredVerified
            ) {
                if self.work_security.get(&n.node_id) != Some(&key) {
                    return Err(invalid());
                }
            } else if *security != n.security {
                return Err(invalid());
            }
        }
        let bytes = encoded(e)?;
        let intent = matches!(
            e.operation,
            Operation::NodeMoveIntent
                | Operation::WorkCreateIntent
                | Operation::WorkRetireIntent
                | Operation::OriginalRestoreIntent
        );
        let completion = matches!(
            e.operation,
            Operation::NodeMovedVerified
                | Operation::WorkCreatedVerified
                | Operation::WorkRetiredVerified
                | Operation::OriginalRestoredVerified
        );
        if self.pending.is_some() && !completion {
            return Err(invalid());
        }
        if completion {
            let pending: Event = serde_json::from_slice(self.pending.as_ref().ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
            if pending.node_id != e.node_id
                || pending.root_id != e.root_id
                || pending.from != e.from
                || pending.to != e.to
                || pending.expected_before != e.expected_before
                || pending.expected_after != e.expected_after
            {
                return Err(invalid());
            }
        }
        self.pending = if intent { Some(bytes.clone()) } else { None };
        self.nodes.insert(n.node_id.clone(), (next, new_work));
        if let Some(key) = new_security {
            self.work_security.insert(n.node_id.clone(), key);
        }
        self.event_ids.insert(e.event_id.clone());
        self.sequence = e.sequence;
        self.tip = hash::sha256_hex(&bytes);
        Ok(())
    }
    /// Necessary DATA completeness only. Deliberately no method yields Ready,
    /// Restored or Closed; retained native/source/stop/release are independent.
    pub(super) fn complete_files(&self, plan: &StoragePlan) -> bool {
        self.batch.is_none()
            && self.pending.is_none()
            && (self.recovery.is_none() || self.lease_state == segments::LeaseState::Released)
            && plan.roots.iter().flat_map(|r| &r.nodes).all(|n| {
                self.nodes
                    .get(&n.node_id)
                    .is_some_and(|(s, _)| *s == if n.kind == Kind::CustodiedFile { 8 } else { 2 })
            })
    }
}
