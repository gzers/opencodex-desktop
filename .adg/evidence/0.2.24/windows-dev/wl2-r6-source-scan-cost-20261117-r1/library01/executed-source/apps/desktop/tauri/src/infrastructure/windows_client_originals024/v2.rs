//! New r4 format only. Deserialization locates pending work, never creates
//! BoundedAccessV1/native custody or any ClosedEligibility. v1 is not migrated.
use super::tree::{SnapshotPhase, WorkingReference};
use super::{
    binding::Binding,
    bounded::{Policy, SnapshotData},
    phase::{digest, invalid},
};
use crate::{
    infrastructure::{
        hash,
        windows_official_runtime024::{current_identity, PinnedDirectory, PrivateRecord},
    },
    modules::runtime::windows_lifecycle024_contract::{
        CURRENT_REFERENCE_SCHEMA_V2, ORIGINALS_RECORD_SCHEMA_V2,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io,
    path::{Component, Path},
};
mod history;
pub(super) mod routes;
use history::{check_predecessors, ClosedReference};
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Identity {
    volume: u32,
    file_id: String,
}
impl Identity {
    fn valid(&self) -> bool {
        self.file_id
            .parse::<u64>()
            .is_ok_and(|n| n > 0 && n.to_string() == self.file_id)
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
enum Kind {
    Absent,
    File,
    Directory,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Node {
    relative: String,
    kind: Kind,
    identity: Option<Identity>,
    parent_identity: Identity,
    security_projection_snapshot: Option<SnapshotData>,
    parent_security_projection_snapshot: SnapshotData,
    length: Option<u64>,
    content_digest: Option<String>,
    children: Vec<String>,
    custody_identity: Option<Identity>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Custody {
    path: String,
    identity: Identity,
    security_projection_snapshot: SnapshotData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    schema: String,
    document_kind: String,
    session_id: String,
    binding: Binding,
    security_policy: Policy,
    security_policy_digest: String,
    custody: Custody,
    nodes: Vec<Node>,
    predecessors: Vec<ClosedReference>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
enum Phase {
    Captured,
    Ready,
    PreparedRestore,
    OfficialTeardown,
    Restored,
    Closed,
    Aborted,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Stage {
    schema: String,
    document_kind: String,
    session_id: String,
    sequence: u32,
    previous_digest: String,
    previous_record_id: Option<String>,
    binding: Binding,
    phase: Phase,
    tree_record: TreeRecord,
    working_record: WorkingRecord,
    history_record: WorkingRecord,
    originals_manifest_digest: String,
    security_policy_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum WorkingRecord {
    None,
    Snapshot(WorkingReference),
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Current {
    schema: String,
    session_id: String,
    sequence: u32,
    phase_digest: String,
    previous_phase_digest: String,
    binding_digest: String,
    originals_manifest_digest: String,
    security_policy_digest: String,
}
fn json<T: serde::de::DeserializeOwned>(b: &[u8], max: usize) -> io::Result<T> {
    if b.is_empty() || b.len() > max {
        return Err(invalid());
    }
    serde_json::from_slice(b).map_err(|_| invalid())
}
fn session(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|s| !s.is_nil() && s.to_string() == id)
}
fn relative(path: &str) -> bool {
    let p = Path::new(path);
    !path.contains('\\')
        && !p.is_absolute()
        && !path.contains(':')
        && p.components().all(|c| matches!(c, Component::Normal(_)))
        && p.components().count() > 0
        && p.components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
            == path
        && crate::infrastructure::windows_official_runtime024::validate_local_path(
            &Path::new("C:/").join(p),
        )
        .is_ok()
}
fn check_manifest(m: &Manifest, sid: &str) -> io::Result<()> {
    if m.schema != ORIGINALS_RECORD_SCHEMA_V2
        || m.document_kind != "manifest"
        || !session(&m.session_id)
        || m.nodes.is_empty()
        || m.nodes.len() > 512
    {
        return Err(invalid());
    }
    m.binding.validate(Some(sid))?;
    m.security_policy.validate()?;
    check_predecessors(
        &m.predecessors,
        &m.session_id,
        &m.binding,
        &m.security_policy_digest,
    )?;
    if m.security_policy.digest()? != m.security_policy_digest {
        return Err(invalid());
    }
    let profile = m.binding.paths()[0];
    let custody = Path::new(&m.custody.path);
    crate::infrastructure::windows_official_runtime024::validate_local_path(custody)?;
    let (profile_volume, profile_id) = m.binding.profile_identity();
    if custody.parent() != Some(profile)
        || custody.file_name().and_then(|s| s.to_str())
            != Some(&format!(".ocxd024-originals-{}", m.session_id))
        || !m.custody.identity.valid()
        || m.custody.identity.volume != profile_volume
        || m.custody.identity.file_id == profile_id
        || !m
            .custody
            .security_projection_snapshot
            .matches_id(m.custody.identity.volume, &m.custody.identity.file_id)
    {
        return Err(invalid());
    }
    m.custody.security_projection_snapshot.validate()?;
    let mut names = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for n in &m.nodes {
        if !relative(&n.relative)
            || !(n.relative == ".codex"
                || n.relative == ".grok"
                || n.relative.starts_with(".codex/")
                || n.relative.starts_with(".grok/"))
            || !names.insert(n.relative.clone())
            || !keys.insert(n.relative.to_lowercase())
            || !n.parent_identity.valid()
        {
            return Err(invalid());
        }
        n.parent_security_projection_snapshot.validate()?;
        if !n.relative.contains('/')
            && (n.parent_identity.volume != profile_volume
                || n.parent_identity.file_id != profile_id)
        {
            return Err(invalid());
        }
        if !n
            .parent_security_projection_snapshot
            .matches_id(n.parent_identity.volume, &n.parent_identity.file_id)
        {
            return Err(invalid());
        }
        if n.kind == Kind::Absent {
            if n.identity.is_some()
                || n.security_projection_snapshot.is_some()
                || n.custody_identity.is_some()
                || n.length.is_some()
                || n.content_digest.is_some()
                || !n.children.is_empty()
            {
                return Err(invalid());
            }
        } else {
            let id = n.identity.as_ref().ok_or_else(invalid)?;
            let custody = n.custody_identity.as_ref().ok_or_else(invalid)?;
            if !id.valid() || id != custody {
                return Err(invalid());
            }
            let security = n
                .security_projection_snapshot
                .as_ref()
                .ok_or_else(invalid)?;
            security.validate()?;
            if !security.matches_id(id.volume, &id.file_id) {
                return Err(invalid());
            }
            match n.kind {
                Kind::File
                    if n.length.is_some_and(|len| len <= 2 * 1024 * 1024)
                        && n.content_digest.as_ref().is_some_and(|s| digest(s))
                        && n.children.is_empty() => {}
                Kind::Directory if n.length.is_none() && n.content_digest.is_none() => {
                    let mut previous = None;
                    for child in &n.children {
                        if !relative(child)
                            || child.contains('/')
                            || previous.is_some_and(|v: &str| v >= child.as_str())
                        {
                            return Err(invalid());
                        }
                        previous = Some(child);
                    }
                }
                _ => return Err(invalid()),
            }
        }
    }
    if !names.contains(".codex") || !names.contains(".grok") {
        return Err(invalid());
    }
    for n in &m.nodes {
        if let Some((parent, leaf)) = n.relative.rsplit_once('/') {
            let p = m
                .nodes
                .iter()
                .find(|p| p.relative == parent)
                .ok_or_else(invalid)?;
            if p.kind != Kind::Directory
                || !p.children.iter().any(|s| s == leaf)
                || p.identity.as_ref() != Some(&n.parent_identity)
                || p.security_projection_snapshot.as_ref()
                    != Some(&n.parent_security_projection_snapshot)
            {
                return Err(invalid());
            }
        }
        for child in &n.children {
            if !names.contains(&format!("{}/{child}", n.relative)) {
                return Err(invalid());
            }
        }
    }
    let codex = m
        .nodes
        .iter()
        .find(|n| n.relative == ".codex")
        .ok_or_else(invalid)?;
    let grok = m
        .nodes
        .iter()
        .find(|n| n.relative == ".grok")
        .ok_or_else(invalid)?;
    if codex.parent_security_projection_snapshot != grok.parent_security_projection_snapshot {
        return Err(invalid());
    }
    Ok(())
}
fn transition(a: &Phase, b: &Phase) -> bool {
    matches!(
        (a, b),
        (Phase::Captured, Phase::Ready | Phase::Aborted)
            | (Phase::Ready, Phase::PreparedRestore | Phase::Aborted)
            | (Phase::PreparedRestore, Phase::OfficialTeardown)
            | (Phase::OfficialTeardown, Phase::Restored)
            | (Phase::Restored, Phase::Closed)
    )
}
struct Tip {
    session: String,
    sequence: u32,
    phase_digest: String,
    previous_digest: String,
    binding_digest: String,
    manifest_digest: String,
    policy_digest: String,
    terminal_record_only: bool,
    phase: Phase,
    tree_record: TreeRecord,
}
mod progress;
use progress::{TreeAction, TreeProgress, TreeRecord, TreeRoot};
fn check_prefix(manifest: &[u8], records: &[Vec<u8>], sid: &str) -> io::Result<Tip> {
    let m: Manifest = json(manifest, 1024 * 1024)?;
    check_manifest(&m, sid)?;
    if records.is_empty() || records.len() > 512 {
        return Err(invalid());
    }
    let manifest_digest = hash::sha256_hex(manifest);
    let binding_digest = hash::sha256_hex(&serde_json::to_vec(&m.binding).map_err(|_| invalid())?);
    let mut previous: Option<Stage> = None;
    let mut last = String::new();
    let mut progress = TreeProgress::default();
    for (i, raw) in records.iter().enumerate() {
        let stage: Stage = json(raw, 65536)?;
        if stage.schema != ORIGINALS_RECORD_SCHEMA_V2
            || stage.document_kind != "phase"
            || stage.session_id != m.session_id
            || stage.sequence as usize != i + 1
            || stage.previous_digest != last
            || stage.binding != m.binding
            || stage.originals_manifest_digest != manifest_digest
            || stage.security_policy_digest != m.security_policy_digest
        {
            return Err(invalid());
        }
        match (&stage.working_record, stage.phase) {
            (WorkingRecord::Snapshot(reference), Phase::Ready)
                if stage.tree_record == TreeRecord::None =>
            {
                reference.validate(
                    SnapshotPhase::Ready,
                    &m.session_id,
                    &binding_digest,
                    &m.security_policy_digest,
                    crate::infrastructure::windows_official_runtime024::FileIdentity {
                        volume: m.custody.identity.volume,
                        index: m.custody.identity.file_id.parse().map_err(|_| invalid())?,
                    },
                )?;
            }
            (WorkingRecord::Snapshot(reference), Phase::PreparedRestore)
                if stage.tree_record == TreeRecord::None =>
            {
                reference.validate(
                    SnapshotPhase::PreStop,
                    &m.session_id,
                    &binding_digest,
                    &m.security_policy_digest,
                    crate::infrastructure::windows_official_runtime024::FileIdentity {
                        volume: m.custody.identity.volume,
                        index: m.custody.identity.file_id.parse().map_err(|_| invalid())?,
                    },
                )?;
            }
            (WorkingRecord::None, phase)
                if !matches!(phase, Phase::Ready | Phase::PreparedRestore) => {}
            _ => return Err(invalid()),
        }
        match (&stage.history_record, stage.phase) {
            (WorkingRecord::Snapshot(reference), Phase::Closed)
                if stage.tree_record == TreeRecord::None =>
            {
                reference.validate(
                    SnapshotPhase::OriginalHistory,
                    &m.session_id,
                    &binding_digest,
                    &m.security_policy_digest,
                    crate::infrastructure::windows_official_runtime024::FileIdentity {
                        volume: m.custody.identity.volume,
                        index: m.custody.identity.file_id.parse().map_err(|_| invalid())?,
                    },
                )?;
            }
            (WorkingRecord::None, phase) if phase != Phase::Closed => {}
            _ => return Err(invalid()),
        }
        match &previous {
            None if stage.phase != Phase::Captured
                || stage.previous_record_id.is_some()
                || stage.tree_record != TreeRecord::None =>
            {
                return Err(invalid())
            }
            Some(p)
                if stage.previous_record_id.as_deref()
                    != Some(&format!("phase-{:08}.json", p.sequence))
                    || if stage.tree_record == TreeRecord::None {
                        !transition(&p.phase, &stage.phase)
                    } else {
                        p.phase != stage.phase
                    } =>
            {
                return Err(invalid())
            }
            _ => {}
        }
        progress.observe(stage.phase, stage.tree_record)?;
        last = hash::sha256_hex(raw);
        previous = Some(stage);
    }
    let tail = previous.ok_or_else(invalid)?;
    Ok(Tip {
        session: m.session_id,
        sequence: tail.sequence,
        phase_digest: last,
        previous_digest: tail.previous_digest,
        binding_digest,
        manifest_digest,
        policy_digest: m.security_policy_digest,
        terminal_record_only: tail.phase == Phase::Closed,
        phase: tail.phase,
        tree_record: tail.tree_record,
    })
}
fn current_for(t: &Tip) -> Current {
    Current {
        schema: CURRENT_REFERENCE_SCHEMA_V2.into(),
        session_id: t.session.clone(),
        sequence: t.sequence,
        phase_digest: t.phase_digest.clone(),
        previous_phase_digest: t.previous_digest.clone(),
        binding_digest: t.binding_digest.clone(),
        originals_manifest_digest: t.manifest_digest.clone(),
        security_policy_digest: t.policy_digest.clone(),
    }
}
/// Only native private record handles and pending data. There is deliberately no
/// cast/constructor to v1 Full, PartialDiagnostic, OriginalSession or Closed.
mod current;
pub(super) use current::BoundedCurrentV2;
mod capture;
pub(super) use capture::MoveIntent;
mod adapter;
pub(crate) use adapter::{LivePhase, WindowsOriginals};
pub(super) struct PendingV2 {
    directory: PinnedDirectory,
    records: Vec<PrivateRecord>,
    tip: Tip,
    manifest_raw: Vec<u8>,
    phase_raw: Vec<Vec<u8>>,
    pub(super) terminal_record_only: bool,
    pub(super) current_matches: bool,
}
pub(super) fn read_pending(directory: &Path) -> io::Result<PendingV2> {
    read_pending_impl(directory, true)
}
fn read_pending_impl(directory: &Path, check_current: bool) -> io::Result<PendingV2> {
    let pin = PinnedDirectory::open_private(directory)?;
    let mut names = std::fs::read_dir(directory)?
        .map(|e| e.map(|x| x.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    let mut expected = vec![std::ffi::OsString::from("manifest.json")];
    let mut stage_names = Vec::new();
    for name in &names {
        if name.to_str().is_some_and(|s| s.starts_with("phase-")) {
            stage_names.push(name.clone());
        }
    }
    if stage_names.is_empty() || stage_names.len() > 512 {
        return Err(invalid());
    }
    let mut records = Vec::new();
    let mut raw = Vec::new();
    let mut manifest = PrivateRecord::open(&directory.join("manifest.json"), 1024 * 1024)?;
    let mbytes = manifest.read()?;
    for (i, name) in stage_names.iter().enumerate() {
        if name.to_str() != Some(&format!("phase-{:08}.json", i + 1)) {
            return Err(invalid());
        }
        expected.push(name.clone());
        let mut p = PrivateRecord::open(&directory.join(name), 65536)?;
        raw.push(p.read()?);
        records.push(p);
    }
    let sid = current_identity()?.sid;
    let tip = check_prefix(&mbytes, &raw, &sid)?;
    if directory.file_name().and_then(|s| s.to_str()) != Some(&tip.session) {
        return Err(invalid());
    }
    let has_current = names.iter().any(|s| s == "current.json");
    if has_current {
        expected.push("current.json".into());
    }
    expected.sort();
    if expected != names {
        return Err(invalid());
    }
    let matches = if has_current && check_current {
        let mut current = PrivateRecord::open(&directory.join("current.json"), 4096)?;
        let value: Current = json(&current.read()?, 4096)?;
        let good = value == current_for(&tip);
        drop(current);
        good
    } else {
        false
    };
    records.push(manifest);
    pin.recheck()?;
    Ok(PendingV2 {
        directory: pin,
        records,
        terminal_record_only: tip.terminal_record_only,
        tip,
        manifest_raw: mbytes,
        phase_raw: raw,
        current_matches: matches,
    })
}
#[cfg(test)]
mod tests;
