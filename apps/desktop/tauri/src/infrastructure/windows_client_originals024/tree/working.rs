//! Same-volume immutable working observations. These are data, never mutation,
//! official-writer exclusion, ReadySession or ClosedEligibility authority.
use super::*;
use crate::infrastructure::windows_client_originals024::bounded::BoundedAccessV1;
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "windows-client-working-snapshot-v2";
const PART: usize = 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum SnapshotPhase {
    Ready,
    PreStop,
    OriginalHistory,
}
impl SnapshotPhase {
    pub(super) fn leaf(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::PreStop => "pre-stop",
            Self::OriginalHistory => "original-history",
        }
    }
}
#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ObjectData {
    volume: u32,
    file_id: String,
    security: SnapshotData,
}
impl ObjectData {
    fn capture(id: FileIdentity, security: &BoundedAccessV1) -> io::Result<Self> {
        let out = Self {
            volume: id.volume,
            file_id: id.index.to_string(),
            security: security.snapshot_data()?,
        };
        out.validate()?;
        Ok(out)
    }
    fn validate(&self) -> io::Result<()> {
        if !self
            .file_id
            .parse::<u64>()
            .is_ok_and(|n| n > 0 && n.to_string() == self.file_id)
            || !self.security.matches_id(self.volume, &self.file_id)
        {
            return Err(invalid());
        }
        self.security.validate()
    }
}
#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RootReference {
    root: String,
    directory: ObjectData,
    metadata: ObjectData,
    metadata_sha256: String,
}
/// Strict locator data. It is intentionally serde-capable, unlike the archive
/// below. Decoding it cannot reopen native phase/session/restore authority.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct WorkingReference {
    schema: String,
    phase: SnapshotPhase,
    session_id: String,
    binding_digest: String,
    security_policy_digest: String,
    directory: ObjectData,
    roots: [RootReference; 2],
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn archive_names(path: &Path, limit: usize) -> io::Result<Vec<String>> {
    let mut values = std::fs::read_dir(path)?
        .take(limit + 1)
        .map(|row| row.and_then(|row| row.file_name().into_string().map_err(|_| invalid())))
        .collect::<io::Result<Vec<_>>>()?;
    if values.len() > limit {
        return Err(invalid());
    }
    values.sort();
    Ok(values)
}
impl WorkingReference {
    pub(crate) fn validate(
        &self,
        phase: SnapshotPhase,
        session: &str,
        binding: &str,
        policy: &str,
        custody: FileIdentity,
    ) -> io::Result<()> {
        if self.schema != SCHEMA
            || self.phase != phase
            || self.session_id != session
            || self.binding_digest != binding
            || self.security_policy_digest != policy
            || !digest(binding)
            || !digest(policy)
            || self.directory.volume != custody.volume
        {
            return Err(invalid());
        }
        self.directory.validate()?;
        let mut identities = std::collections::BTreeSet::new();
        identities.insert((custody.volume, custody.index.to_string()));
        if !identities.insert((self.directory.volume, self.directory.file_id.clone())) {
            return Err(invalid());
        }
        for (root, expected) in self.roots.iter().zip([".codex", ".grok"]) {
            if root.root != expected || !digest(&root.metadata_sha256) {
                return Err(invalid());
            }
            root.directory.validate()?;
            root.metadata.validate()?;
            if !identities.insert((root.directory.volume, root.directory.file_id.clone()))
                || !identities.insert((root.metadata.volume, root.metadata.file_id.clone()))
            {
                return Err(invalid());
            }
            if root.directory.volume != self.directory.volume
                || root.metadata.volume != self.directory.volume
                || root.directory.file_id == self.directory.file_id
                || root.metadata.file_id == root.directory.file_id
            {
                return Err(invalid());
            }
        }
        if self.roots[0].directory.file_id == self.roots[1].directory.file_id
            || self.roots[0].metadata.file_id == self.roots[1].metadata.file_id
        {
            return Err(invalid());
        }
        Ok(())
    }
}
/// Actual native observation owned only by its root/session. Bytes never enter
/// the phase/current locator on D and are not exposed by Debug or serde.
pub(super) struct WorkingImage {
    path: PathBuf,
    parent: FileIdentity,
    parent_security: SnapshotData,
    entries: Vec<Entry>,
}
impl WorkingImage {
    pub(super) fn artifact_frame(&self) -> io::Result<super::ArtifactFrame> {
        super::ArtifactFrame::from_entries(&self.path, &self.entries)
    }
}
impl RootTree {
    fn observe_restored_original(&mut self, end: Instant) -> io::Result<WorkingImage> {
        self.verify_restored(end)?;
        let entries = if let Some(file) = self.root.as_mut() {
            // Use the original retained DELETE-capable root description. A new
            // deny-delete reader would conflict with that actual owner; sharing
            // is never relaxed merely to make history capture pass.
            let observed = image(&self.original, file, end)?;
            if observed != self.entries {
                return Err(invalid());
            }
            observed
        } else {
            require_absent(&self.original)?;
            Vec::new()
        };
        self.verify_restored(end)?;
        Ok(WorkingImage {
            path: self.original.clone(),
            parent: self.parent.identity(),
            parent_security: self.parent_security_data(end)?,
            entries,
        })
    }
    fn parent_security_data(&self, end: Instant) -> io::Result<SnapshotData> {
        parent_stamp(self.parent.path(), end)
    }
    pub(super) fn observe_working(&mut self, end: Instant) -> io::Result<WorkingImage> {
        self.check_original(end)?;
        if self.custody.is_none() || self.working.is_none() || self.retired_work.is_some() {
            return Err(invalid());
        }
        let mut file = open(&self.original, true, false)?;
        let entries = image(&self.original, &mut file, end)?;
        if image(&self.original, &mut file, end)? != entries {
            return Err(invalid());
        }
        self.check_original(end)?;
        Ok(WorkingImage {
            path: self.original.clone(),
            parent: self.parent.identity(),
            parent_security: parent_stamp(self.parent.path(), end)?,
            entries,
        })
    }
    fn matches_image(&mut self, expected: &WorkingImage, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        if expected.path != self.original
            || expected.parent != self.parent.identity()
            || expected.parent_security != self.parent_security
        {
            return Err(invalid());
        }
        let mut file = open(&self.original, true, false)?;
        if image(&self.original, &mut file, end)? != expected.entries {
            return Err(invalid());
        }
        self.check_original(end)
    }
}
struct Chunk {
    node: usize,
    begin: usize,
    end: usize,
    name: String,
    record: PrivateRecord,
    security: BoundedAccessV1,
}
struct StoredRoot {
    image: WorkingImage,
    directory: PinnedDirectory,
    security: BoundedAccessV1,
    chunks: Vec<Chunk>,
    metadata: Option<(PrivateRecord, BoundedAccessV1, Vec<u8>)>,
    failed: bool,
    #[cfg(test)]
    fail_after_first_part: bool,
}
impl StoredRoot {
    fn write(
        &mut self,
        phase: SnapshotPhase,
        session: &str,
        binding: &str,
        policy: &str,
        end: Instant,
    ) -> io::Result<()> {
        for (node, entry) in self.image.entries.iter().enumerate() {
            if entry.directory {
                continue;
            }
            let count = entry.bytes.len().max(1).div_ceil(PART);
            for part in 0..count {
                check(end)?;
                let begin = part * PART;
                let stop = (begin + PART).min(entry.bytes.len());
                let name = format!("data-{node:04}-{part:04}.bin");
                let path = self.directory.path().join(&name);
                let record = PrivateRecord::create(&path, &entry.bytes[begin..stop], PART)?;
                let security = BoundedAccessV1::capture(&path, false, end).map_err(failure)?;
                self.chunks.push(Chunk {
                    node,
                    begin,
                    end: stop,
                    name,
                    record,
                    security,
                });
                #[cfg(test)]
                if self.fail_after_first_part {
                    return Err(io::Error::other(
                        "injected after actual first sealed working data part",
                    ));
                }
            }
        }
        let mut nodes = Vec::new();
        for (index, entry) in self.image.entries.iter().enumerate() {
            let chunks=self.chunks.iter().filter(|chunk|chunk.node==index).map(|chunk| {
                let id=chunk.record.identity();
                Ok(serde_json::json!({"name":chunk.name,"volume":id.volume,"fileId":id.index.to_string(),
                    "bytes":chunk.end-chunk.begin,"sha256":hash::sha256_hex(&entry.bytes[chunk.begin..chunk.end])}))
            }).collect::<io::Result<Vec<_>>>()?;
            nodes.push(serde_json::json!({"relative":entry.relative,"directory":entry.directory,
                "sourceVolume":entry.id.volume,"sourceFileId":entry.id.index.to_string(),
                "securityProjectionSnapshot":entry.security,"children":entry.children,
                "bytes":if entry.directory{None}else{Some(entry.bytes.len())},
                "sha256":if entry.directory{None}else{Some(hash::sha256_hex(&entry.bytes))},"chunks":chunks}));
        }
        let metadata = serde_json::json!({"schema":SCHEMA,"phase":phase,"sessionId":session,
            "bindingDigest":binding,"securityPolicyDigest":policy,"sourceRoot":self.image.path,
            "sourceParent":{"volume":self.image.parent.volume,"fileId":self.image.parent.index.to_string(),"securityProjectionSnapshot":self.image.parent_security},
            "sourceAbsent":self.image.entries.is_empty(),"nodes":nodes,"auditSaclAssurance":"excluded-unknown","fullSecurityProven":false});
        let raw = serde_json::to_vec(&metadata).map_err(|_| invalid())?;
        let path = self.directory.path().join("snapshot.json");
        check(end)?;
        let record = PrivateRecord::create(&path, &raw, PART)?;
        let security = BoundedAccessV1::capture(&path, false, end).map_err(failure)?;
        self.metadata = Some((record, security, raw));
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    fn recheck(&mut self, end: Instant) -> io::Result<()> {
        check(end)?;
        if self.failed {
            return Err(invalid());
        }
        self.directory.recheck()?;
        streams(self.directory.path(), true)?;
        self.security.recheck(end).map_err(failure)?;
        for chunk in &mut self.chunks {
            streams(&self.directory.path().join(&chunk.name), false)?;
            if chunk.record.read()? != self.image.entries[chunk.node].bytes[chunk.begin..chunk.end]
            {
                return Err(invalid());
            }
            chunk.security.recheck(end).map_err(failure)?;
        }
        let (record, security, raw) = self.metadata.as_mut().ok_or_else(invalid)?;
        streams(&self.directory.path().join("snapshot.json"), false)?;
        if record.read()? != *raw {
            return Err(invalid());
        }
        security.recheck(end).map_err(failure)?;
        let mut expected = self
            .chunks
            .iter()
            .map(|part| part.name.clone())
            .collect::<Vec<_>>();
        expected.push("snapshot.json".into());
        expected.sort();
        if archive_names(self.directory.path(), expected.len())? != expected {
            return Err(invalid());
        }
        self.directory.recheck()?;
        check(end)
    }
    fn reference(&self, root: &str) -> io::Result<RootReference> {
        let (metadata, security, raw) = self.metadata.as_ref().ok_or_else(invalid)?;
        Ok(RootReference {
            root: root.into(),
            directory: ObjectData::capture(self.directory.identity(), &self.security)?,
            metadata: ObjectData::capture(metadata.identity(), security)?,
            metadata_sha256: hash::sha256_hex(raw),
        })
    }
}
/// Retained sealed copies and their exact private native objects. No Clone,
/// serde, public fields or constructor from references; source proof is separate.
pub(crate) struct WorkingArchive {
    custody: FileIdentity,
    directory: PinnedDirectory,
    security: BoundedAccessV1,
    phase: SnapshotPhase,
    session: String,
    binding: String,
    policy: String,
    roots: Vec<StoredRoot>,
    failed: bool,
    #[cfg(test)]
    fail_after_first_part: bool,
}
impl WorkingArchive {
    /// Rechecked sealed archive data, never live client permission. Generation
    /// comes from its captured original working entries, not archive-copy IDs.
    pub(crate) fn artifact_frame(&mut self, end: Instant) -> io::Result<super::ArtifactFrame> {
        self.recheck(end)?;
        let image = &self.roots.first().ok_or_else(invalid)?.image;
        super::ArtifactFrame::from_entries(&image.path, &image.entries)
    }
    pub(crate) fn create(
        custody: &PinnedDirectory,
        phase: SnapshotPhase,
        session: &str,
        binding: &str,
        policy: &str,
        end: Instant,
    ) -> io::Result<Self> {
        check(end)?;
        custody.recheck()?;
        if !digest(binding)
            || !digest(policy)
            || !uuid::Uuid::parse_str(session)
                .is_ok_and(|v| !v.is_nil() && v.to_string() == session)
        {
            return Err(invalid());
        }
        let directory = create_private_directory(&custody.path().join(phase.leaf()))?;
        if directory.identity().volume != custody.identity().volume {
            return Err(invalid());
        }
        let security = BoundedAccessV1::capture(directory.path(), true, end).map_err(failure)?;
        Ok(Self {
            custody: custody.identity(),
            directory,
            security,
            phase,
            session: session.into(),
            binding: binding.into(),
            policy: policy.into(),
            roots: Vec::new(),
            failed: true,
            #[cfg(test)]
            fail_after_first_part: false,
        })
    }
    pub(crate) fn capture(&mut self, trees: &mut [RootTree; 2], end: Instant) -> io::Result<()> {
        if !self.failed || !self.roots.is_empty() {
            return Err(invalid());
        }
        for (tree, name) in trees.iter_mut().zip([".codex", ".grok"]) {
            let image = if self.phase == SnapshotPhase::OriginalHistory {
                tree.observe_restored_original(end)?
            } else {
                tree.observe_working(end)?
            };
            if image.path.file_name().and_then(|s| s.to_str()) != Some(name) {
                return Err(invalid());
            }
            let directory = create_private_directory(&self.directory.path().join(name))?;
            let security =
                BoundedAccessV1::capture(directory.path(), true, end).map_err(failure)?;
            self.roots.push(StoredRoot {
                image,
                directory,
                security,
                chunks: Vec::new(),
                metadata: None,
                failed: true,
                #[cfg(test)]
                fail_after_first_part: self.fail_after_first_part,
            });
            self.roots.last_mut().unwrap().write(
                self.phase,
                &self.session,
                &self.binding,
                &self.policy,
                end,
            )?;
        }
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        self.matches_live(trees, end)
    }
    #[cfg(test)]
    pub(crate) fn inject_first_part_failure(&mut self) {
        self.fail_after_first_part = true;
    }
    pub(crate) fn matches_live(
        &mut self,
        trees: &mut [RootTree; 2],
        end: Instant,
    ) -> io::Result<()> {
        self.recheck(end)?;
        for (tree, root) in trees.iter_mut().zip(&self.roots) {
            if self.phase == SnapshotPhase::OriginalHistory {
                tree.verify_restored(end)?;
                Self::original_matches_image(tree, &root.image)?;
            } else {
                tree.matches_image(&root.image, end)?;
            }
        }
        check(end)
    }
    fn original_matches_image(tree: &RootTree, source: &WorkingImage) -> io::Result<()> {
        if tree.original != source.path
            || tree.parent.identity() != source.parent
            || tree.parent_security != source.parent_security
            || tree.entries != source.entries
            || !tree.original_handle_released && tree.root.is_none() != source.entries.is_empty()
        {
            return Err(invalid());
        }
        Ok(())
    }
    /// Native archive/readback only, deliberately independent of future live
    /// client paths. Historical data is still tied to the retained original
    /// capture entries and cannot construct a new original root or permission.
    pub(crate) fn check_original_history(
        &mut self,
        trees: &mut [RootTree; 2],
        end: Instant,
    ) -> io::Result<()> {
        if self.phase != SnapshotPhase::OriginalHistory {
            return Err(invalid());
        }
        self.recheck(end)?;
        for (tree, root) in trees.iter_mut().zip(&self.roots) {
            tree.check_closed_retention(end)?;
            Self::original_matches_image(tree, &root.image)?;
        }
        check(end)
    }
    pub(crate) fn recheck(&mut self, end: Instant) -> io::Result<()> {
        let result = self.recheck_inner(end);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn recheck_inner(&mut self, end: Instant) -> io::Result<()> {
        check(end)?;
        if self.failed || self.roots.len() != 2 {
            return Err(invalid());
        }
        self.directory.recheck()?;
        streams(self.directory.path(), true)?;
        self.security.recheck(end).map_err(failure)?;
        if names(self.directory.path())? != [".codex".to_string(), ".grok".to_string()] {
            return Err(invalid());
        }
        for root in &mut self.roots {
            root.recheck(end)?;
        }
        self.directory.recheck()?;
        check(end)
    }
    pub(crate) fn reference(&mut self, end: Instant) -> io::Result<WorkingReference> {
        self.recheck(end)?;
        let out = WorkingReference {
            schema: SCHEMA.into(),
            phase: self.phase,
            session_id: self.session.clone(),
            binding_digest: self.binding.clone(),
            security_policy_digest: self.policy.clone(),
            directory: ObjectData::capture(self.directory.identity(), &self.security)?,
            roots: [
                self.roots[0].reference(".codex")?,
                self.roots[1].reference(".grok")?,
            ],
        };
        out.validate(
            self.phase,
            &self.session,
            &self.binding,
            &self.policy,
            self.custody,
        )?;
        Ok(out)
    }
}
