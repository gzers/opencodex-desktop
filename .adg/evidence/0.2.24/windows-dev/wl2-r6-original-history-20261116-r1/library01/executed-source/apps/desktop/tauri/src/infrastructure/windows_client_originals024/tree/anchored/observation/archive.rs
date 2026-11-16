//! Same-volume private full observation bytes, not a second journal/state
//! machine or a factory for restoration permissions. Canonical phase linkage
//! and production Source/Stop authority remain separate.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::journal::HistoryContext;
use crate::infrastructure::windows_client_originals024::v3::journal::ObservationContext;
const PART: usize = 1024 * 1024;
struct Part {
    name: String,
    file: PrivateRecord,
    security: Option<SnapshotData>,
    bytes: Vec<u8>,
}
pub(super) struct FrameArchive {
    kind: ArchiveKind,
    directory: PinnedDirectory,
    security: SnapshotData,
    source: serde_json::Value,
    context: serde_json::Value,
    parts: Vec<Part>,
    metadata: Option<Part>,
    complete: bool,
}
enum ArchiveKind {
    Observation,
    OriginalHistory,
}
impl ArchiveKind {
    fn document(&self) -> &'static str {
        match self {
            Self::Observation => "windows-anchored-work-observation-v3",
            Self::OriginalHistory => "windows-anchored-original-history-v3",
        }
    }
    fn reference(&self) -> &'static str {
        match self {
            Self::Observation => "windows-anchored-work-observation-reference-v3",
            Self::OriginalHistory => "windows-anchored-original-history-reference-v3",
        }
    }
}
fn native_record_security(path: &Path, end: Instant) -> io::Result<SnapshotData> {
    let file = OpenOptions::new()
        .access_mode(0x20080)
        .share_mode(1)
        .custom_flags(0x200000)
        .open(path)?;
    native::verify_private_file(&file)?;
    native_stamp(&file, end).map_err(failure)
}
impl FrameArchive {
    pub(super) fn matches_metadata(
        &mut self,
        metadata: &serde_json::Value,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        if self.source != *metadata {
            return Err(invalid());
        }
        Ok(())
    }
    pub(super) fn observation_sequence(&self) -> io::Result<u32> {
        self.context["observationSequence"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(invalid)
    }
    pub(super) fn matches_series_context(
        &self,
        expected: &serde_json::Value,
        sequence: u32,
        frame: &str,
    ) -> io::Result<()> {
        for key in [
            "sessionId",
            "storagePlanDigest",
            "bindingDigest",
            "manifestDigest",
            "deadlineScope",
        ] {
            if self.context[key] != expected[key] {
                return Err(invalid());
            }
        }
        if self.observation_sequence()? != sequence || frame_digest(&self.source)? != frame {
            return Err(invalid());
        }
        Ok(())
    }
    /// Native archive DATA: source generations are distinct from archive chunk
    /// IDs. Every immutable chunk and metadata object was rechecked first.
    pub(super) fn artifact_data(&self) -> io::Result<super::super::super::ArtifactFrame> {
        // Called only inside the complete TWO-tree before/after native barrier
        // in series::assess_tree_series. Reconstruct data from already verified
        // immutable bytes; this function issues no permission or fresh proof.
        if !self.complete {
            return Err(invalid());
        }
        let path = PathBuf::from(self.source["root"].as_str().ok_or_else(invalid)?);
        let rows = self.source["nodes"].as_array().ok_or_else(invalid)?;
        if rows.len() > 512 {
            return Err(invalid());
        }
        let mut entries = Vec::new();
        let mut consumed = 0usize;
        for (index, row) in rows.iter().enumerate() {
            let directory = row["directory"].as_bool().ok_or_else(invalid)?;
            let length = row["bytes"].as_u64().ok_or_else(invalid)?;
            if length > 2 * 1024 * 1024 {
                return Err(invalid());
            }
            let mut bytes = Vec::new();
            if !directory {
                let count = (length as usize).max(1).div_ceil(PART);
                for part in 0..count {
                    let expected = format!("data-{index:04}-{part:04}.bin");
                    let p = self
                        .parts
                        .iter()
                        .find(|p| p.name == expected)
                        .ok_or_else(invalid)?;
                    bytes.extend_from_slice(&p.bytes);
                    consumed += 1;
                }
            }
            if bytes.len() != length as usize
                || row["sha256"].as_str() != Some(&hash::sha256_hex(&bytes))
            {
                return Err(invalid());
            }
            let security: SnapshotData =
                serde_json::from_value(row["security"].clone()).map_err(|_| invalid())?;
            security.validate()?;
            let volume = u32::try_from(row["volume"].as_u64().ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
            let file_id = row["fileId"]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(invalid)?;
            if !security.matches_id(volume, &file_id.to_string()) {
                return Err(invalid());
            }
            let children =
                serde_json::from_value(row["children"].clone()).map_err(|_| invalid())?;
            entries.push(Entry {
                relative: row["relative"].as_str().ok_or_else(invalid)?.into(),
                directory,
                id: FileIdentity {
                    volume,
                    index: file_id,
                },
                security,
                bytes,
                children,
            });
        }
        if consumed != self.parts.len() {
            return Err(invalid());
        }
        super::super::super::ArtifactFrame::from_entries(&path, &entries)
    }
    fn begin(
        parent: &Path,
        source: serde_json::Value,
        context: &ObservationContext,
        end: Instant,
    ) -> io::Result<Self> {
        check(end)?;
        let parent = PinnedDirectory::open_private(parent)?;
        let root = source["root"].as_str().ok_or_else(invalid)?;
        if !context.matches(Path::new(root), &frame_digest(&source)?) {
            return Err(invalid());
        }
        if Path::new(root).parent() != Some(parent.path())
            || source["parentVolume"].as_u64() != Some(parent.identity().volume as u64)
            || source["parentId"].as_str() != Some(&parent.identity().index.to_string())
        {
            return Err(invalid());
        }
        let name = format!(
            ".ocxd024-observed-{}-{}-{}",
            context.session(),
            context.purpose_leaf(),
            Path::new(root)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(invalid)?
                .trim_start_matches('.')
        );
        let directory = create_private_directory(&parent.path().join(name))?;
        if directory.identity().volume != parent.identity().volume {
            return Err(invalid());
        }
        let security = parent_stamp(directory.path(), end)?;
        Ok(Self {
            kind: ArchiveKind::Observation,
            directory,
            security,
            source,
            context: context.metadata(),
            parts: Vec::new(),
            metadata: None,
            complete: false,
        })
    }
    fn create_part(directory: &PinnedDirectory, name: String, bytes: Vec<u8>) -> io::Result<Part> {
        let file = PrivateRecord::create(&directory.path().join(&name), &bytes, PART)?;
        Ok(Part {
            name,
            file,
            security: None,
            bytes,
        })
    }
    fn seal(&mut self, frame: &WorkFrame, fail_after_part: bool, end: Instant) -> io::Result<()> {
        if self.complete
            || !self.parts.is_empty()
            || self.metadata.is_some()
            || frame.metadata() != self.source
        {
            return Err(invalid());
        }
        let mut parts = Vec::new();
        for (index, n) in frame.nodes.iter().enumerate() {
            if n.entry.directory {
                continue;
            }
            for part in 0..n.entry.bytes.len().max(1).div_ceil(PART) {
                check(end)?;
                let start = part * PART;
                let stop = (start + PART).min(n.entry.bytes.len());
                let name = format!("data-{index:04}-{part:04}.bin");
                let bytes = n.entry.bytes[start..stop].to_vec();
                let record = Self::create_part(&self.directory, name, bytes)?;
                self.parts.push(record);
                let p = self.parts.last_mut().ok_or_else(invalid)?;
                p.security = Some(native_record_security(
                    &self.directory.path().join(&p.name),
                    end,
                )?);
                if p.file.read()? != p.bytes {
                    return Err(invalid());
                }
                parts.push(serde_json::json!({"node":index,"begin":start,"end":stop,"name":p.name,"volume":p.file.identity().volume,"fileId":p.file.identity().index.to_string(),"security":p.security,"bytes":p.bytes.len(),"sha256":hash::sha256_hex(&p.bytes)}));
                if fail_after_part && !p.bytes.is_empty() {
                    return Err(io::Error::other(
                        "injected after actual nonempty immutable observation chunk sync/readback",
                    ));
                }
            }
        }
        let metadata = serde_json::json!({"schema":self.kind.document(),"context":self.context,"source":self.source,"parts":parts,"runtimeAuthority":false,"auditSaclAssurance":"excluded-unknown","fullSecurityProven":false});
        let bytes = serde_json::to_vec(&metadata).map_err(|_| invalid())?;
        self.metadata = Some(Self::create_part(
            &self.directory,
            "snapshot.json".into(),
            bytes,
        )?);
        let record = self.metadata.as_mut().ok_or_else(invalid)?;
        record.security = Some(native_record_security(
            &self.directory.path().join("snapshot.json"),
            end,
        )?);
        self.complete = true;
        self.check(end)
    }
    #[cfg(test)]
    pub(super) fn complete_for_test(&self) -> bool {
        self.complete
    }
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        let result = self.check_inner(end);
        if result.is_err() {
            self.complete = false;
        }
        result
    }
    fn check_inner(&mut self, end: Instant) -> io::Result<()> {
        if !self.complete {
            return Err(invalid());
        }
        check(end)?;
        self.directory.recheck()?;
        if parent_stamp(self.directory.path(), end)? != self.security {
            return Err(invalid());
        }
        let mut expected = Vec::new();
        for p in self.parts.iter_mut().chain(self.metadata.iter_mut()) {
            streams(&self.directory.path().join(&p.name), false)?;
            let fresh = OpenOptions::new()
                .access_mode(0x20080)
                .share_mode(1)
                .custom_flags(0x200000)
                .open(self.directory.path().join(&p.name))?;
            if id(&fresh, false)? != p.file.identity()
                || native_stamp(&fresh, end).map_err(failure)?
                    != *p.security.as_ref().ok_or_else(invalid)?
                || p.file.read()? != p.bytes
            {
                return Err(invalid());
            }
            expected.push(p.name.clone());
        }
        expected.sort();
        if names(self.directory.path())? != expected {
            return Err(invalid());
        }
        check(end)
    }
    pub(super) fn reference(&mut self, end: Instant) -> io::Result<serde_json::Value> {
        self.check(end)?;
        let metadata = self.metadata.as_ref().ok_or_else(invalid)?;
        let mut value = serde_json::json!({"schema":self.kind.reference(),"directory":self.directory.path(),"volume":self.directory.identity().volume,"fileId":self.directory.identity().index.to_string(),"security":self.security,"metadataId":metadata.file.identity().index.to_string(),"metadataSecurity":metadata.security,"metadataSha256":hash::sha256_hex(&metadata.bytes),"frameDigest":frame_digest(&self.source)?,"context":self.context,"runtimeAuthority":false});
        if matches!(self.kind, ArchiveKind::OriginalHistory) {
            value["rootId"] = Path::new(self.source["root"].as_str().ok_or_else(invalid)?)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(invalid)?
                .into();
        }
        Ok(value)
    }
}
impl WorkFrame {
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn archive_original_history(
        &mut self,
        parent: &Path,
        context: &HistoryContext,
        fail_after_part: bool,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.check(end)?;
        if self.archive.is_some() || self.archive_failed {
            return Err(invalid());
        }
        let parent = PinnedDirectory::open(parent)?;
        let source = self.metadata();
        context.matches(&self.root, &frame_digest(&source)?)?;
        if self.root.parent() != Some(parent.path())
            || parent.identity() != self.parent
            || parent_stamp(parent.path(), end)? != self.parent_security
        {
            return Err(invalid());
        }
        let leaf = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(invalid)?
            .trim_start_matches('.');
        let directory = create_private_directory(
            &parent
                .path()
                .join(format!(".ocxd024-history-{}-{leaf}", context.session())),
        )?;
        if directory.identity().volume != parent.identity().volume {
            return Err(invalid());
        }
        let security = parent_stamp(directory.path(), end)?;
        let mut archive = FrameArchive {
            kind: ArchiveKind::OriginalHistory,
            directory,
            security,
            source,
            context: context.metadata(),
            parts: Vec::new(),
            metadata: None,
            complete: false,
        };
        let result = archive.seal(self, fail_after_part, end);
        self.archive = Some(archive);
        if let Err(error) = result {
            self.archive_failed = true;
            return Err(error);
        }
        self.check(end)?;
        self.archive.as_mut().ok_or_else(invalid)?.reference(end)
    }
    pub(super) fn archive(
        &mut self,
        parent: &Path,
        context: &ObservationContext,
        fail_after_part: bool,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.check(end)?;
        if self.archive.is_some() {
            return Err(invalid());
        }
        let mut archive = FrameArchive::begin(parent, self.metadata(), context, end)?;
        // Store all partial original records even if creation/snapshot IO fails.
        let result = archive.seal(self, fail_after_part, end);
        self.archive = Some(archive);
        if let Err(error) = result {
            self.archive_failed = true;
            return Err(error);
        }
        self.check(end)?;
        self.archive.as_mut().ok_or_else(invalid)?.reference(end)
    }
}
