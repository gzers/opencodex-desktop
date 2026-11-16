//! Writable WORK-generation observation. Original custody/anchors/lock sharing
//! do not change. Observed changed artifact data never grants restore authority.
use super::*;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
mod archive;
pub(super) mod retirement;
pub(in crate::infrastructure::windows_client_originals024) use retirement::OriginalHandleHandoff;
pub(in crate::infrastructure::windows_client_originals024) use retirement::RecoveryWitness;
pub(super) mod series;
pub(super) mod snapshots;
pub(in crate::infrastructure::windows_client_originals024) use snapshots::{
    SnapshotLinkObservation, SnapshotWitness,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WorkMode {
    Sealed,
    Opening,
    Open,
    Observing,
    Observed,
}
pub(super) fn reopen(file: &File, access: u32, share: u32) -> io::Result<File> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReOpenFile(
            original: *mut std::ffi::c_void,
            access: u32,
            share: u32,
            flags: u32,
        ) -> *mut std::ffi::c_void;
    }
    let raw = unsafe { ReOpenFile(file.as_raw_handle(), access, share, 0x200000) };
    if raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_handle(raw) })
}
/// Observation ONLY for the already-registered old WORK description after an
/// external atomic replacement. Zero links is permitted only here; this never
/// validates a current path, original custody, a rename or recovery authority.
fn retained_work_identity(file: &File, expected: FileIdentity) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let actual = FileIdentity {
        volume: data.dwVolumeSerialNumber,
        index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
    };
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || data.dwFileAttributes & 0x410 != 0
        || data.nNumberOfLinks > 1
        || actual != expected
    {
        return Err(invalid());
    }
    Ok(actual)
}
struct FrameNode {
    file: File,
    entry: Entry,
}
pub(super) struct WorkFrame {
    root: PathBuf,
    parent: FileIdentity,
    parent_security: SnapshotData,
    nodes: Vec<FrameNode>,
    archive: Option<archive::FrameArchive>,
    archive_failed: bool,
    pending_history_directory: Option<PinnedDirectory>,
}
/// Immutable byte/history objects retained after current working handles are
/// handed back. Recheck archive objects, never compare a later live client tree
/// against a previous frame to mint a current-generation permission.
pub(super) struct RetainedSnapshot {
    archive: archive::FrameArchive,
    work_objects: Vec<(File, FileIdentity)>,
}
impl RetainedSnapshot {
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        for (file, expected) in &self.work_objects {
            retained_work_identity(file, *expected)?;
        }
        self.archive.check(end)
    }
}
fn frame_kind(root: &Path, relative: &str, directory: bool) -> Kind {
    if directory {
        Kind::Directory
    } else if root.file_name().and_then(|n| n.to_str()) == Some(".codex")
        && ANCHORED_CODEX_STABLE_FILES.contains(&relative)
    {
        Kind::Coordinator
    } else {
        Kind::File
    }
}
fn collect(
    root: &Path,
    path: &Path,
    relative: &str,
    nodes: &mut Vec<FrameNode>,
    total: &mut usize,
    end: Instant,
) -> io::Result<()> {
    check(end)?;
    if nodes.len() >= 512 {
        return Err(invalid());
    }
    let directory = std::fs::symlink_metadata(path)?.is_dir();
    let kind = frame_kind(root, relative, directory);
    let mut file = open_node(path, kind, false)?;
    native::verify_private_file(&file)?;
    let entry = stamp(path, &mut file, relative, kind, end)?;
    *total = total.checked_add(entry.bytes.len()).ok_or_else(invalid)?;
    if *total > 32 * 1024 * 1024 {
        return Err(invalid());
    }
    let children = entry.children.clone();
    nodes.push(FrameNode { file, entry });
    for child in children {
        let next = if relative.is_empty() {
            child.clone()
        } else {
            format!("{relative}/{child}")
        };
        collect(root, &path.join(child), &next, nodes, total, end)?;
    }
    Ok(())
}
impl WorkFrame {
    fn empty(tree: &AnchoredTree, end: Instant) -> io::Result<Self> {
        Ok(Self {
            root: tree.root.clone(),
            parent: tree.parent.identity(),
            parent_security: parent_stamp(tree.parent.path(), end)?,
            nodes: Vec::new(),
            archive: None,
            archive_failed: false,
            pending_history_directory: None,
        })
    }
    fn capture(&mut self, end: Instant) -> io::Result<()> {
        if !self.nodes.is_empty() {
            return Err(invalid());
        }
        collect(&self.root, &self.root, "", &mut self.nodes, &mut 0, end)?;
        self.nodes
            .sort_by(|a, b| a.entry.relative.cmp(&b.entry.relative));
        self.check(end)
    }
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        if self.archive_failed || self.pending_history_directory.is_some() {
            return Err(invalid());
        }
        let parent = PinnedDirectory::open(self.root.parent().ok_or_else(invalid)?)?;
        if parent.identity() != self.parent
            || parent_stamp(parent.path(), end)? != self.parent_security
        {
            return Err(invalid());
        }
        for n in &mut self.nodes {
            let path = self.root.join(&n.entry.relative);
            let kind = frame_kind(&self.root, &n.entry.relative, n.entry.directory);
            if stamp(&path, &mut n.file, &n.entry.relative, kind, end)? != n.entry {
                return Err(invalid());
            }
            let fresh = open_node(&path, kind, true)?;
            if id(&fresh, n.entry.directory)? != n.entry.id {
                return Err(invalid());
            }
        }
        if let Some(archive) = &mut self.archive {
            archive.check(end)?;
        }
        check(end)
    }
    pub(super) fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"root":self.root,"parentVolume":self.parent.volume,"parentId":self.parent.index.to_string(),"parentSecurity":self.parent_security,"nodes":self.nodes.iter().map(|n|serde_json::json!({"relative":n.entry.relative,"directory":n.entry.directory,"volume":n.entry.id.volume,"fileId":n.entry.id.index.to_string(),"security":n.entry.security,"bytes":n.entry.bytes.len(),"sha256":hash::sha256_hex(&n.entry.bytes),"children":n.entry.children})).collect::<Vec<_>>()})
    }
    pub(super) fn artifact_frame(&self) -> io::Result<super::super::ArtifactFrame> {
        // ArtifactFrame is a data view only; source/phase/content policy remain
        // independent. Copy the bounded entries rather than the native handles.
        let entries = self
            .nodes
            .iter()
            .map(|n| {
                Ok(Entry {
                    relative: n.entry.relative.clone(),
                    directory: n.entry.directory,
                    id: n.entry.id,
                    security: serde_json::from_value(
                        serde_json::to_value(&n.entry.security).map_err(|_| invalid())?,
                    )
                    .map_err(|_| invalid())?,
                    bytes: n.entry.bytes.clone(),
                    children: n.entry.children.clone(),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        super::super::ArtifactFrame::from_entries(&self.root, &entries)
    }
}
impl AnchoredTree {
    fn baseline_work_metadata(&self) -> io::Result<serde_json::Value> {
        let mut nodes = Vec::new();
        for n in &self.nodes {
            let e = if n.kind == Kind::File {
                &n.work.as_ref().ok_or_else(invalid)?.1
            } else {
                &n.original
            };
            nodes.push(serde_json::json!({"relative":e.relative,"directory":e.directory,"volume":e.id.volume,"fileId":e.id.index.to_string(),"security":e.security,"bytes":e.bytes.len(),"sha256":hash::sha256_hex(&e.bytes),"children":e.children}));
        }
        Ok(
            serde_json::json!({"root":self.root,"parentVolume":self.parent.identity().volume,"parentId":self.parent.identity().index.to_string(),"parentSecurity":self.parent_security,"nodes":nodes}),
        )
    }
    fn accept_unchanged_work(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let root = self.parent.path().join(".codex");
        claim.check(&root, end)?;
        self.verify_custody_anchors(end)?;
        if self.work_mode != WorkMode::Observed {
            return Err(invalid());
        }
        let expected = self.baseline_work_metadata()?;
        let frame = self.observed.as_mut().ok_or_else(invalid)?;
        frame.check(end)?;
        if frame.metadata() != expected {
            return Err(invalid());
        }
        let mut frame = self.observed.take().ok_or_else(invalid)?;
        if let Some(archive) = frame.archive.take() {
            self.snapshot_history.push(RetainedSnapshot {
                archive,
                work_objects: Vec::new(),
            });
        }
        for observed in frame.nodes {
            let n = self
                .nodes
                .iter_mut()
                .find(|n| n.original.relative == observed.entry.relative)
                .ok_or_else(invalid)?;
            if n.kind == Kind::File {
                n.work = Some((observed.file, observed.entry));
                n.work_open = false;
            }
        }
        self.work_mode = WorkMode::Sealed;
        self.verify(claim, &root, end)
    }
    fn verify_custody_anchors(&mut self, end: Instant) -> io::Result<()> {
        if self.poisoned {
            return Err(invalid());
        }
        self.parent.recheck()?;
        for snapshot in &mut self.snapshot_history {
            snapshot.check(end)?;
        }
        self.custody.recheck()?;
        if parent_stamp(self.parent.path(), end)? != self.parent_security
            || parent_stamp(self.custody.path(), end)? != self.custody_security
        {
            return Err(invalid());
        }
        let mut expected = Vec::new();
        for i in 0..self.nodes.len() {
            let path = self.original_location(i);
            let n = &mut self.nodes[i];
            if n.pending_work.is_some() || n.kind == Kind::File && n.state != State::Working {
                return Err(invalid());
            }
            let observed = stamp(&path, &mut n.file, &n.original.relative, n.kind, end)?;
            if n.kind == Kind::Directory {
                if observed.id != n.original.id || observed.security != n.original.security {
                    return Err(invalid());
                }
            } else if observed != n.original {
                return Err(invalid());
            }
            let fresh = open_node(&path, n.kind, true)?;
            if id(&fresh, n.kind == Kind::Directory)? != n.original.id {
                return Err(invalid());
            }
            if path.parent() == Some(self.custody.path()) {
                expected.push(path.file_name().unwrap().to_str().unwrap().to_string());
            }
        }
        expected.sort();
        if names(self.custody.path())? != expected {
            return Err(invalid());
        }
        check(end)
    }
    fn open_work_generation(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            self.verify(claim, &root, end)?;
            if self.work_mode != WorkMode::Sealed
                || self.observed.is_some()
                || self
                    .nodes
                    .iter()
                    .any(|n| n.kind == Kind::File && n.state != State::Working)
            {
                return Err(invalid());
            }
            self.work_mode = WorkMode::Opening;
            self.poisoned = true;
            for n in &mut self.nodes {
                if n.kind != Kind::File {
                    continue;
                }
                claim.check(&root, end)?;
                let (file, entry) = n.work.as_mut().ok_or_else(invalid)?;
                // ReOpenFile binds the exact existing creation object. Observer
                // takes no write/delete right, and shares all three so the old
                // creation handle and subsequent official atomic writer coexist.
                n.pending_work = Some(reopen(file, 0x20080, 1 | 2 | 4)?);
                let observer = n.pending_work.as_ref().ok_or_else(invalid)?;
                if id(observer, false)? != entry.id
                    || native_stamp(observer, end).map_err(failure)? != entry.security
                {
                    return Err(invalid());
                }
                let old = std::mem::replace(file, n.pending_work.take().ok_or_else(invalid)?);
                n.work_open = true;
                drop(old);
            }
            self.work_mode = WorkMode::Open;
            self.poisoned = false;
            self.verify(claim, &root, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn observe_work_generation(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            claim.check(&root, end)?;
            if self.work_mode != WorkMode::Open || self.observed.is_some() {
                return Err(invalid());
            }
            self.verify_custody_anchors(end)?;
            self.work_mode = WorkMode::Observing;
            self.observed = Some(WorkFrame::empty(self, end)?);
            self.observed.as_mut().ok_or_else(invalid)?.capture(end)?;
            // Working-object identity belongs to the complete observation
            // boundary, never custody-only pre-admission. Current hardlinks/ADS
            // first refuse in the retained frame scanner; the historical
            // observer is then checked independently of the current path.
            for n in &self.nodes {
                if let Some((file, entry)) = &n.work {
                    if n.work_open {
                        retained_work_identity(file, entry.id)?;
                    }
                }
            }
            let codex = self.root.file_name().and_then(|n| n.to_str()) == Some(".codex");
            let artifact = |path: &str| {
                codex
                    && [
                        "opencodex-catalog.json",
                        "models_cache.json",
                        "opencodex-catalog-audit.jsonl",
                    ]
                    .contains(&path)
            };
            let frame = self.observed.as_mut().ok_or_else(invalid)?;
            for observed in &frame.nodes {
                if artifact(&observed.entry.relative) {
                    if observed.entry.directory {
                        return Err(invalid());
                    }
                    continue;
                }
                let n = self
                    .nodes
                    .iter()
                    .find(|n| n.original.relative == observed.entry.relative)
                    .ok_or_else(invalid)?;
                let expected = if n.kind == Kind::File {
                    &n.work.as_ref().ok_or_else(invalid)?.1
                } else {
                    &n.original
                };
                let mut children = observed
                    .entry
                    .children
                    .iter()
                    .filter(|name| !(codex && observed.entry.relative.is_empty() && artifact(name)))
                    .cloned()
                    .collect::<Vec<_>>();
                let mut original = expected
                    .children
                    .iter()
                    .filter(|name| !(codex && observed.entry.relative.is_empty() && artifact(name)))
                    .cloned()
                    .collect::<Vec<_>>();
                children.sort();
                original.sort();
                if observed.entry.id != expected.id
                    || observed.entry.security != expected.security
                    || observed.entry.bytes != expected.bytes
                    || observed.entry.directory != expected.directory
                    || children != original
                {
                    return Err(invalid());
                }
            }
            for n in &self.nodes {
                if !artifact(&n.original.relative)
                    && !frame
                        .nodes
                        .iter()
                        .any(|v| v.entry.relative == n.original.relative)
                {
                    return Err(invalid());
                }
            }
            frame.check(end)?;
            self.work_mode = WorkMode::Observed;
            claim.check(&root, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
fn frame_digest(value: &serde_json::Value) -> io::Result<String> {
    Ok(hash::sha256_hex(
        &serde_json::to_vec(value).map_err(|_| invalid())?,
    ))
}
pub(super) fn open_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut super::super::super::v3::Journal,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let root = trees[0].parent.path().join(".codex");
        for t in trees.iter_mut() {
            t.verify(claim, &root, end)?;
        }
        let plans = trees.each_ref().map(|t| t.plan());
        let frames = [
            frame_digest(&trees[0].baseline_work_metadata()?)?,
            frame_digest(&trees[1].baseline_work_metadata()?)?,
        ];
        journal.work_transition(&plans, true, Some(frames.clone()), claim, end, |claim| {
            for t in trees.iter_mut() {
                t.open_work_generation(claim, end)?;
            }
            Ok([
                frame_digest(&trees[0].baseline_work_metadata()?)?,
                frame_digest(&trees[1].baseline_work_metadata()?)?,
            ])
        })
    })();
    if result.is_err() {
        journal.poison_segment();
        for t in trees {
            t.poisoned = true;
        }
    }
    result
}
pub(super) fn observe_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut super::super::super::v3::Journal,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let plans = trees.each_ref().map(|t| t.plan());
        journal.work_transition(&plans, false, None, claim, end, |claim| {
            for t in trees.iter_mut() {
                t.observe_work_generation(claim, end)?;
            }
            Ok([
                frame_digest(&trees[0].observed.as_ref().ok_or_else(invalid)?.metadata())?,
                frame_digest(&trees[1].observed.as_ref().ok_or_else(invalid)?.metadata())?,
            ])
        })
    })();
    if result.is_err() {
        journal.poison_segment();
        for t in trees {
            t.poisoned = true;
        }
    }
    result
}
pub(super) fn accept_unchanged_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut super::super::super::v3::Journal,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let root = trees[0].parent.path().join(".codex");
        for t in trees.iter_mut() {
            t.verify_custody_anchors(end)?;
            t.observed.as_mut().ok_or_else(invalid)?.check(end)?;
        }
        let plans = trees.each_ref().map(|t| t.plan());
        let frames = [
            frame_digest(&trees[0].observed.as_ref().ok_or_else(invalid)?.metadata())?,
            frame_digest(&trees[1].observed.as_ref().ok_or_else(invalid)?.metadata())?,
        ];
        journal.unchanged_work(&plans, frames, claim, end, |claim| {
            for t in trees.iter_mut() {
                t.accept_unchanged_work(claim, end)?;
            }
            for t in trees.iter_mut() {
                t.verify(claim, &root, end)?;
            }
            Ok(())
        })
    })();
    if result.is_err() {
        journal.poison_segment();
        for t in trees {
            t.poisoned = true;
        }
    }
    result
}
pub(super) fn reacquire_for_observation(
    trees: &mut [AnchoredTree; 2],
    journal: &mut super::super::super::v3::Journal,
    lease: &mut Option<super::super::super::coordination::NativeLease>,
    owner: Option<&mut crate::infrastructure::windows_official_lifecycle024::Owner>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        if lease.is_some() || trees.iter().any(|t| t.work_mode != WorkMode::Open) {
            return Err(invalid());
        }
        for t in trees.iter_mut() {
            t.verify_custody_anchors(end)?;
        }
        let plans = trees.each_ref().map(|t| t.plan());
        journal.prepare_lease_acquire(&plans, end)?;
        *lease = Some(journal.acquire_prepared_lease(owner, end)?);
        let root = trees[0].parent.path().join(".codex");
        let mut claim = lease
            .as_mut()
            .ok_or_else(invalid)?
            .anchored_claim(&root, end)?;
        for t in trees.iter_mut() {
            t.verify_custody_anchors(end)?;
        }
        journal.finish_lease_acquire(&plans, &mut claim, end)
    })();
    if result.is_err() {
        journal.poison_segment();
        for t in trees {
            t.poisoned = true;
        }
    }
    result
}
#[cfg(test)]
#[path = "observation_tests.rs"]
mod tests;
