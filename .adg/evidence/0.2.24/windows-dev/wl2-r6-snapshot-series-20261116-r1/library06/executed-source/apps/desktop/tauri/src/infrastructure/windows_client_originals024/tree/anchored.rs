//! r6 existing-root native component. Directories/coordinators never move.
//! Exact retained file objects move individually. No v2 adoption or production
//! Ready/Stop/Closed authority is constructed by this component.
use super::super::coordination::AnchoredClaim;
use super::*;
use crate::infrastructure::windows_official_runtime024 as native;
use crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_CODEX_STABLE_FILES;
use std::io::{Seek, SeekFrom, Write};
pub(super) mod capture;
pub(super) mod observation;
mod segments;
use observation::WorkMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Directory,
    Coordinator,
    File,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Original,
    Moved,
    Working,
    Retired,
    Restored,
}
struct Node {
    original: Entry,
    file: File,
    kind: Kind,
    state: State,
    work: Option<(File, Entry)>,
    pending_work: Option<File>,
    work_open: bool,
}
/// All fields are actual native objects or snapshots sealed from those objects.
/// No constructor from plan data, no Clone/serde, no mutable pathname admission.
pub(in super::super) struct AnchoredTree {
    parent: PinnedDirectory,
    parent_security: SnapshotData,
    root: PathBuf,
    custody: PinnedDirectory,
    custody_security: SnapshotData,
    nodes: Vec<Node>,
    poisoned: bool,
    work_mode: WorkMode,
    observed: Option<observation::WorkFrame>,
    release_view: Option<observation::WorkFrame>,
    snapshot_history: Vec<observation::RetainedSnapshot>,
    #[cfg(test)]
    create_fault: u8,
}
fn open_node(path: &Path, kind: Kind, observe: bool) -> io::Result<File> {
    validate_local_path(path)?;
    // Anchors are non-DELETE and deny directory substitution. Coordinator
    // sharing preserves official SQLite admission; actual exclusion is lease.
    OpenOptions::new()
        .access_mode(if kind == Kind::Directory {
            0x20080
        } else {
            0x80020000
                | if kind == Kind::File && !observe {
                    0x10000
                } else {
                    0
                }
        })
        .share_mode(if kind == Kind::Coordinator {
            1 | 2
        } else if observe {
            1 | 2 | 4
        } else if kind == Kind::Directory {
            1 | 2
        } else {
            1
        })
        .custom_flags(
            0x200000
                | if kind == Kind::Directory {
                    0x2000000
                } else {
                    0
                },
        )
        .open(path)
}
fn stamp(
    path: &Path,
    file: &mut File,
    relative: &str,
    kind: Kind,
    end: Instant,
) -> io::Result<Entry> {
    check(end)?;
    let directory = kind == Kind::Directory;
    let identity = id(file, directory)?;
    streams(path, directory)?;
    let security = native_stamp(file, end).map_err(failure)?;
    let children = if directory { names(path)? } else { Vec::new() };
    let mut bytes = Vec::new();
    if !directory {
        if file.metadata()?.len() > 2 * 1024 * 1024 {
            return Err(invalid());
        }
        file.seek(SeekFrom::Start(0))?;
        file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 2 * 1024 * 1024 || bytes.len() as u64 != file.metadata()?.len() {
            return Err(invalid());
        }
    }
    if id(file, directory)? != identity
        || native_stamp(file, end).map_err(failure)? != security
        || directory && names(path)? != children
    {
        return Err(invalid());
    }
    check(end)?;
    Ok(Entry {
        relative: relative.into(),
        directory,
        id: identity,
        security,
        bytes,
        children,
    })
}
fn collect(
    root: &Path,
    relative: &str,
    codex: bool,
    out: &mut Vec<Node>,
    total: &mut usize,
    end: Instant,
) -> io::Result<()> {
    check(end)?;
    if out.len() >= 512 {
        return Err(invalid());
    }
    let path = root.join(relative);
    let metadata = std::fs::symlink_metadata(&path)?;
    let kind = if metadata.is_dir() {
        Kind::Directory
    } else if codex && ANCHORED_CODEX_STABLE_FILES.contains(&relative) {
        Kind::Coordinator
    } else {
        Kind::File
    };
    // Unknown coordinator sidecars are never ordinary files eligible to move.
    if codex
        && ANCHORED_CODEX_STABLE_FILES
            .iter()
            .any(|s| relative.starts_with(&format!("{s}-")))
    {
        return Err(invalid());
    }
    let mut file = open_node(&path, kind, false)?;
    if kind == Kind::Directory {
        native::verify_private_directory(&file)?;
    } else {
        native::verify_private_file(&file)?;
    }
    let original = stamp(&path, &mut file, relative, kind, end)?;
    if kind == Kind::Coordinator {
        super::super::coordination::validate_stable_bytes(&original.bytes)?;
    }
    *total = total
        .checked_add(original.bytes.len())
        .ok_or_else(invalid)?;
    if *total > 32 * 1024 * 1024 {
        return Err(invalid());
    }
    let children = original.children.clone();
    out.push(Node {
        original,
        file,
        kind,
        state: State::Original,
        work: None,
        pending_work: None,
        work_open: false,
    });
    for name in children {
        let child = if relative.is_empty() {
            name
        } else {
            format!("{relative}/{name}")
        };
        collect(root, &child, codex, out, total, end)?;
    }
    Ok(())
}
impl AnchoredTree {
    fn node_security(
        &mut self,
        i: usize,
        working: bool,
        end: Instant,
    ) -> io::Result<(String, SnapshotData)> {
        let path = if working {
            self.work_location(i)
        } else {
            self.original_location(i)
        };
        let n = self.nodes.get_mut(i).ok_or_else(invalid)?;
        let observed = if working {
            let (file, entry) = n.work.as_mut().ok_or_else(invalid)?;
            let observed = stamp(&path, file, &entry.relative, Kind::File, end)?;
            if observed != *entry {
                return Err(invalid());
            }
            observed
        } else {
            let observed = stamp(&path, &mut n.file, &n.original.relative, n.kind, end)?;
            if observed != n.original {
                return Err(invalid());
            }
            observed
        };
        Ok((observed.id.index.to_string(), observed.security))
    }
    fn journal_snapshot_anchors(
        &mut self,
        journal: &mut super::super::v3::Journal,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let codex = self.parent.path().join(".codex");
            self.verify(claim, &codex, end)?;
            for i in 0..self.nodes.len() {
                if self.nodes[i].kind == Kind::File {
                    continue;
                }
                let plan = self.plan();
                let node = self.key(i);
                let native = self.node_security(i, false, end)?;
                journal.snapshot_anchor(&plan, &node, native, claim, end)?;
            }
            self.verify(claim, &codex, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn journal_step(
        &mut self,
        i: usize,
        action: super::super::v3::FileAction,
        journal: &mut super::super::v3::Journal,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        use super::super::v3::FileAction;
        let result = (|| {
            let codex = self.parent.path().join(".codex");
            self.verify(claim, &codex, end)?;
            let plan = self.plan();
            let node = self.key(i);
            let prior = match action {
                FileAction::CreateWork => None,
                FileAction::RetireWork => Some(self.node_security(i, true, end)?),
                _ => Some(self.node_security(i, false, end)?),
            };
            let state = match action {
                FileAction::MoveOriginal => State::Moved,
                FileAction::CreateWork => State::Working,
                FileAction::RetireWork => State::Retired,
                FileAction::RestoreOriginal => State::Restored,
            };
            journal.execute_node(&plan, &node, action, prior, claim, end, |claim| {
                self.step(i, state, claim, &codex, end)?;
                self.node_security(
                    i,
                    matches!(action, FileAction::CreateWork | FileAction::RetireWork),
                    end,
                )
            })?;
            self.verify(claim, &codex, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in super::super) fn capture(
        root: &Path,
        custody: &Path,
        claim: &mut AnchoredClaim<'_>,
        codex_root: &Path,
        end: Instant,
    ) -> io::Result<Self> {
        claim.check(codex_root, end)?;
        if ![Some(".codex"), Some(".grok")].contains(&root.file_name().and_then(|n| n.to_str())) {
            return Err(invalid());
        }
        let parent = PinnedDirectory::open(root.parent().ok_or_else(invalid)?)?;
        let parent_security = parent_stamp(parent.path(), end)?;
        let custody = PinnedDirectory::open_private(custody)?;
        let custody_security = parent_stamp(custody.path(), end)?;
        if custody.path().parent() != Some(parent.path())
            || custody.identity().volume != parent.identity().volume
            || !names(custody.path())?.is_empty()
        {
            return Err(invalid());
        }
        // Existing roots only. OriginalAbsent initialization/retirement is
        // intentionally unimplemented; never mkdir/adopt a missing client.
        let mut nodes = Vec::new();
        let codex = root.file_name().and_then(|n| n.to_str()) == Some(".codex");
        if codex && root != codex_root {
            return Err(invalid());
        }
        collect(root, "", codex, &mut nodes, &mut 0, end)?;
        nodes.sort_by(|a, b| a.original.relative.cmp(&b.original.relative));
        if codex
            && ANCHORED_CODEX_STABLE_FILES.iter().any(|s| {
                !nodes
                    .iter()
                    .any(|n| n.original.relative == *s && n.kind == Kind::Coordinator)
            })
        {
            return Err(invalid());
        }
        let mut out = Self {
            parent,
            parent_security,
            root: root.into(),
            custody,
            custody_security,
            nodes,
            poisoned: false,
            work_mode: WorkMode::Sealed,
            observed: None,
            release_view: None,
            snapshot_history: Vec::new(),
            #[cfg(test)]
            create_fault: 0,
        };
        out.verify(claim, codex_root, end)?;
        Ok(out)
    }
    fn key(&self, i: usize) -> String {
        hash::sha256_hex(
            format!(
                "{}/{}",
                self.root.file_name().unwrap().to_str().unwrap(),
                self.nodes[i].original.relative
            )
            .as_bytes(),
        )
    }
    fn original_location(&self, i: usize) -> PathBuf {
        let n = &self.nodes[i];
        if matches!(n.state, State::Moved | State::Working | State::Retired) {
            self.custody
                .path()
                .join(format!("original-{}", self.key(i)))
        } else {
            self.root.join(&n.original.relative)
        }
    }
    fn work_location(&self, i: usize) -> PathBuf {
        if matches!(self.nodes[i].state, State::Retired | State::Restored) {
            self.custody.path().join(format!("retired-{}", self.key(i)))
        } else {
            self.root.join(&self.nodes[i].original.relative)
        }
    }
    fn expected_children(&self, dir: &str) -> Vec<String> {
        let mut children = Vec::new();
        for n in &self.nodes {
            let p = Path::new(&n.original.relative);
            if !n.original.relative.is_empty()
                && p.parent() == Some(Path::new(dir))
                && (n.kind != Kind::File
                    || matches!(n.state, State::Original | State::Working | State::Restored))
            {
                children.push(p.file_name().unwrap().to_str().unwrap().into());
            }
        }
        children.sort();
        children
    }
    pub(in super::super) fn verify(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        codex_root: &Path,
        end: Instant,
    ) -> io::Result<()> {
        let result = self.verify_inner(claim, codex_root, end);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn verify_inner(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        codex_root: &Path,
        end: Instant,
    ) -> io::Result<()> {
        if self.poisoned {
            return Err(invalid());
        }
        claim.check(codex_root, end)?;
        self.verify_image(end)?;
        claim.check(codex_root, end)
    }
    fn verify_image(&mut self, end: Instant) -> io::Result<()> {
        if self.poisoned {
            return Err(invalid());
        }
        check(end)?;
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
        let mut custody_names = Vec::new();
        for i in 0..self.nodes.len() {
            let path = self.original_location(i);
            let expected_children = self.expected_children(&self.nodes[i].original.relative);
            let workpath = self.work_location(i);
            let n = &mut self.nodes[i];
            if n.pending_work.is_some() {
                return Err(invalid());
            }
            let observed = stamp(&path, &mut n.file, &n.original.relative, n.kind, end)?;
            let mut fresh = open_node(&path, n.kind, true)?;
            if id(&fresh, n.kind == Kind::Directory)? != n.original.id
                || native_stamp(&fresh, end).map_err(failure)? != n.original.security
            {
                return Err(invalid());
            }
            if n.kind == Kind::Directory {
                if observed.id != n.original.id
                    || observed.security != n.original.security
                    || observed.children != expected_children
                {
                    return Err(invalid());
                }
            } else if observed != n.original {
                return Err(invalid());
            }
            if path.parent() == Some(self.custody.path()) {
                custody_names.push(path.file_name().unwrap().to_str().unwrap().to_string());
            }
            if let Some((file, entry)) = &mut n.work {
                let mut read;
                let file = if n.work_open {
                    read = observation::reopen(file, 0x80020000, 1 | 2 | 4)?;
                    &mut read
                } else {
                    file
                };
                if stamp(&workpath, file, &entry.relative, Kind::File, end)? != *entry {
                    return Err(invalid());
                }
                fresh = open_node(&workpath, Kind::File, true)?;
                if id(&fresh, false)? != entry.id {
                    return Err(invalid());
                }
                if workpath.parent() == Some(self.custody.path()) {
                    custody_names.push(workpath.file_name().unwrap().to_str().unwrap().to_string());
                }
            }
        }
        custody_names.sort();
        if names(self.custody.path())? != custody_names {
            return Err(invalid());
        }
        check(end)
    }
    fn verify_after_release(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            if self
                .nodes
                .iter()
                .any(|n| n.kind == Kind::File && n.state != State::Restored)
            {
                return Err(invalid());
            }
            self.verify_image(end)?;
            if self.root.file_name().and_then(|n| n.to_str()) == Some(".codex") {
                let entries = self
                    .nodes
                    .iter_mut()
                    .map(|n| {
                        stamp(
                            &self.root.join(&n.original.relative),
                            &mut n.file,
                            &n.original.relative,
                            n.kind,
                            end,
                        )
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                super::locks::prove(&self.root, &self.nodes[0].file, &entries, end)?;
            }
            self.verify_image(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Native component event sink. The canonical v3 driver must persist an
    /// intent before entering here and a result after exact native readback.
    /// No production caller exists until that driver and its prerequisites pass.
    fn step(
        &mut self,
        i: usize,
        operation: State,
        claim: &mut AnchoredClaim<'_>,
        codex_root: &Path,
        end: Instant,
    ) -> io::Result<()> {
        if self.work_mode != WorkMode::Sealed {
            return Err(invalid());
        }
        self.verify(claim, codex_root, end)?;
        let n = self.nodes.get(i).ok_or_else(invalid)?;
        if n.kind != Kind::File
            || !matches!(
                (n.state, operation),
                (State::Original, State::Moved)
                    | (State::Moved, State::Working)
                    | (State::Working, State::Retired)
                    | (State::Retired, State::Restored)
            )
        {
            return Err(invalid());
        }
        let target = match operation {
            State::Moved => self
                .custody
                .path()
                .join(format!("original-{}", self.key(i))),
            State::Retired => self.custody.path().join(format!("retired-{}", self.key(i))),
            _ => self.root.join(&n.original.relative),
        };
        require_absent(&target)?;
        claim.check(codex_root, end)?;
        check(end)?;
        self.poisoned = true;
        let n = &mut self.nodes[i];
        match operation {
            State::Moved => rename_original_handle(&n.file, &target)?,
            State::Working => {
                // Retain the CREATE_NEW description before any fallible IO.
                // It remains owned even on empty/partial-write/seal failure.
                n.pending_work = Some(native::create_private_originals_work_file(&target)?);
                let writer = n.pending_work.as_mut().ok_or_else(invalid)?;
                let created = id(writer, false)?;
                native::verify_private_file(writer)?;
                #[cfg(test)]
                if self.create_fault == 1 {
                    return Err(io::Error::other(
                        "injected after actual CREATE_NEW, exact new handle retained",
                    ));
                }
                claim.check(codex_root, end)?;
                #[cfg(test)]
                if self.create_fault == 2 {
                    writer.write_all(&n.original.bytes[..1])?;
                    writer.sync_all()?;
                    return Err(io::Error::other(
                        "injected after real partial work write/sync, exact handle retained",
                    ));
                }
                writer.write_all(&n.original.bytes)?;
                writer.sync_all()?;
                if id(writer, false)? != created {
                    return Err(invalid());
                }
                let entry = stamp(&target, writer, &n.original.relative, Kind::File, end)?;
                if entry.bytes != n.original.bytes
                    || entry.id == n.original.id
                    || entry.id != created
                {
                    return Err(invalid());
                }
                #[cfg(test)]
                if self.create_fault == 3 {
                    return Err(io::Error::other(
                        "injected after complete work sync/readback, before native publication",
                    ));
                }
                n.work = Some((n.pending_work.take().ok_or_else(invalid)?, entry));
            }
            State::Retired => {
                rename_original_handle(&n.work.as_ref().ok_or_else(invalid)?.0, &target)?
            }
            State::Restored => rename_original_handle(&n.file, &target)?,
            _ => return Err(invalid()),
        }
        // Retain the exact native prefix immediately, before fallible IO.
        n.state = operation;
        self.poisoned = false;
        self.verify(claim, codex_root, end)
    }
    pub(in super::super) fn plan(&self) -> serde_json::Value {
        serde_json::json!({"root":self.root,"custody":self.custody.path(),"custodyVolume":self.custody.identity().volume,"custodyId":self.custody.identity().index.to_string(),"custodySecurity":self.custody_security,"nodes":self.nodes.iter().enumerate().map(|(i,n)|{
            let parent=Path::new(&n.original.relative).parent().and_then(|p|self.nodes.iter().find(|x|Path::new(&x.original.relative)==p));
            serde_json::json!({"nodeId":self.key(i),"rootId":self.root.file_name().unwrap().to_str().unwrap(),"relative":n.original.relative,"kind":match n.kind {Kind::Directory=>"DirectoryAnchor",Kind::Coordinator=>"CoordinatorAnchor",Kind::File=>"CustodiedFile"},"volume":n.original.id.volume,"fileId":n.original.id.index.to_string(),"parentId":parent.map(|p|p.original.id.index.to_string()).unwrap_or_else(||self.parent.identity().index.to_string()),"security":n.original.security,"children":n.original.children,"length":n.original.bytes.len(),"contentDigest":hash::sha256_hex(&n.original.bytes),"target":if n.kind==Kind::File{Some(format!("original-{}",self.key(i)))}else{None}})
        }).collect::<Vec<_>>()})
    }
}
#[cfg(test)]
#[path = "anchored_tests.rs"]
mod tests;
#[cfg(test)]
pub(in super::super) fn test_plan() -> (
    super::tests::Fixture,
    Vec<serde_json::Value>,
    PathBuf,
    FileIdentity,
) {
    use super::super::coordination::NativeLease;
    let f = super::tests::Fixture::new();
    let (root, custody, paths) = tests::prepare(&f);
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    let grok = root.with_file_name(".grok");
    drop(create_private_directory(&grok).unwrap());
    drop(PrivateRecord::create(&grok.join("settings.json"), b"synthetic grok", 1024).unwrap());
    let grok_custody = root.with_file_name("grok-custody");
    drop(create_private_directory(&grok_custody).unwrap());
    let profile = root.parent().unwrap().to_path_buf();
    let pin = PinnedDirectory::open(&profile).unwrap();
    let parent = pin.identity();
    let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
    let mut claim = lease.anchored_claim(&root, deadline).unwrap();
    let codex_tree = AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline).unwrap();
    let grok_tree =
        AnchoredTree::capture(&grok, &grok_custody, &mut claim, &root, deadline).unwrap();
    let data = vec![codex_tree.plan(), grok_tree.plan()];
    drop(claim);
    lease.release(deadline).unwrap();
    drop(codex_tree);
    drop(grok_tree);
    (f, data, profile, parent)
}
