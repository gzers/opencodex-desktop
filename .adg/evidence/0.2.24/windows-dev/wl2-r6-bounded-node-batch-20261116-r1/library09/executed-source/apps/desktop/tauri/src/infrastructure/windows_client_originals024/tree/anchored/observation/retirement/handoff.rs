//! Own the original trees BEFORE detaching/closing native files. Archive and
//! retired objects move to history, never rebuilt from serialized locators.
use super::*;
use crate::infrastructure::windows_client_originals024::handle_release::NativeHandleBatch;
struct Retired {
    file: File,
    entry: Entry,
    path: PathBuf,
}
struct RootHistory {
    parent: PinnedDirectory,
    parent_security: SnapshotData,
    custody: PinnedDirectory,
    custody_security: SnapshotData,
    root: PathBuf,
    original: Vec<Entry>,
    archives: Vec<super::super::archive::FrameArchive>,
    retired: Vec<Retired>,
}
pub(in crate::infrastructure::windows_client_originals024) struct OriginalHandleHandoff {
    pending: Option<[AnchoredTree; 2]>,
    roots: Vec<RootHistory>,
    pub(in crate::infrastructure::windows_client_originals024) handles: NativeHandleBatch,
    poisoned: bool,
}
impl OriginalHandleHandoff {
    pub(in crate::infrastructure::windows_client_originals024) fn retain(
        trees: [AnchoredTree; 2],
        end: Instant,
    ) -> Self {
        Self {
            pending: Some(trees),
            roots: Vec::new(),
            handles: NativeHandleBatch::new(end),
            poisoned: false,
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn detach(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let trees = self.pending.as_mut().ok_or_else(invalid)?;
            // Driver just verified full native refs on this same owned pending
            // object. No client mutation occurs during Rust ownership transfer.
            if trees.iter().any(|t| t.poisoned) {
                return Err(invalid());
            }
            if trees.iter().any(|t| {
                t.observed.is_some()
                    || t.release_view.is_some()
                    || t.nodes.iter().any(|n| n.pending_work.is_some())
            }) {
                return Err(invalid());
            }
            // No fallible IO after taking the verified native trees. Every
            // actual File has exactly one retained destination below.
            let trees = self.pending.take().ok_or_else(invalid)?;
            for (r, tree) in trees.into_iter().enumerate() {
                let AnchoredTree {
                    parent,
                    parent_security,
                    root,
                    custody,
                    custody_security,
                    nodes,
                    retirement,
                    original_history,
                    snapshot_history,
                    observed,
                    release_view,
                    ..
                } = tree;
                let mut out = RootHistory {
                    parent,
                    parent_security,
                    custody,
                    custody_security,
                    root,
                    original: Vec::new(),
                    archives: Vec::new(),
                    retired: Vec::new(),
                };
                if observed.is_some() || release_view.is_some() {
                    self.handles.invalidate();
                }
                for (i, node) in nodes.into_iter().enumerate() {
                    self.handles.retain(
                        format!("tree/{r}/original/{i:04}"),
                        node.file,
                        node.original.id,
                        node.original.directory,
                        false,
                    );
                    if let Some((file, entry)) = node.work {
                        self.handles.retain(
                            format!("tree/{r}/initial-work/{i:04}"),
                            file,
                            entry.id,
                            false,
                            true,
                        );
                    }
                    if let Some(file) = node.pending_work {
                        self.handles.retain_observed(
                            format!("tree/{r}/pending-work/{i:04}"),
                            file,
                            false,
                            true,
                        );
                        self.handles.invalidate();
                    }
                    out.original.push(node.original);
                }
                if let Some(frame) = original_history {
                    for (i, n) in frame.nodes.into_iter().enumerate() {
                        self.handles.retain(
                            format!("tree/{r}/history-source/{i:04}"),
                            n.file,
                            n.entry.id,
                            n.entry.directory,
                            false,
                        );
                    }
                    if let Some(archive) = frame.archive {
                        out.archives.push(archive);
                    } else {
                        self.handles.invalidate();
                    }
                } else {
                    self.handles.invalidate();
                }
                if let Some(retirement) = retirement {
                    let targets = retirement
                        .items
                        .iter()
                        .map(|i| (i.frame_index, i.target.clone()))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    for (i, n) in retirement.frame.nodes.into_iter().enumerate() {
                        if let Some(path) = targets.get(&i) {
                            out.retired.push(Retired {
                                file: n.file,
                                entry: n.entry,
                                path: path.clone(),
                            });
                        } else {
                            self.handles.retain(
                                format!("tree/{r}/stopped-anchor/{i:04}"),
                                n.file,
                                n.entry.id,
                                n.entry.directory,
                                false,
                            );
                        }
                    }
                    if let Some(archive) = retirement.frame.archive {
                        out.archives.push(archive);
                    } else {
                        self.handles.invalidate();
                    }
                } else {
                    self.handles.invalidate();
                }
                for (s, snapshot) in snapshot_history.into_iter().enumerate() {
                    for (i, (file, id)) in snapshot.work_objects.into_iter().enumerate() {
                        self.handles.retain(
                            format!("tree/{r}/old-work/{s:04}/{i:04}"),
                            file,
                            id,
                            false,
                            true,
                        );
                    }
                    out.archives.push(snapshot.archive);
                }
                self.roots.push(out);
            }
            // Full live-original readback happens after durable intent and
            // immediately before native closes, not twice around a Rust move.
            self.handles.check_held(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            self.handles.invalidate();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn pending_history_references(
        &mut self,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        borrow_history(self.pending.as_mut().ok_or_else(invalid)?).history_references(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_originals(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        self.check_history(end)?;
        for root in &mut self.roots {
            let mut files = Vec::new();
            for n in &root.original {
                let kind = frame_kind(&root.root, &n.relative, n.directory);
                let path = root.root.join(&n.relative);
                let mut file = open_node(&path, kind, true)?;
                if stamp(&path, &mut file, &n.relative, kind, end)? != *n {
                    return Err(invalid());
                }
                files.push(file);
            }
            if root.root.file_name().and_then(|n| n.to_str()) == Some(".codex") {
                super::super::super::super::locks::prove(
                    &root.root,
                    files.first().ok_or_else(invalid)?,
                    &root.original,
                    end,
                )?;
            }
        }
        check(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_history(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        if self.poisoned || self.pending.is_some() || self.roots.len() != 2 {
            return Err(invalid());
        }
        for root in &mut self.roots {
            root.parent.recheck()?;
            root.custody.recheck()?;
            if parent_stamp(root.parent.path(), end)? != root.parent_security
                || parent_stamp(root.custody.path(), end)? != root.custody_security
            {
                return Err(invalid());
            }
            for archive in &mut root.archives {
                archive.check(end)?;
            }
            let mut expected = Vec::new();
            for n in &mut root.retired {
                if stamp(&n.path, &mut n.file, &n.entry.relative, Kind::File, end)? != n.entry {
                    return Err(invalid());
                }
                expected.push(
                    n.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(invalid)?
                        .to_string(),
                );
            }
            expected.sort();
            if names(root.custody.path())? != expected {
                return Err(invalid());
            }
        }
        check(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn poison(&mut self) {
        self.poisoned = true;
        self.handles.invalidate();
    }
    pub(in crate::infrastructure::windows_client_originals024) fn trace(
        &self,
    ) -> serde_json::Value {
        serde_json::json!({"treesPending":self.pending.is_some(),"historyRoots":self.roots.len(),"poisoned":self.poisoned,
            "handles":self.handles.trace(),"roots":self.roots.iter().map(|r|serde_json::json!({"root":r.root,"custody":r.custody.path(),
                "originals":r.original.iter().map(|n|serde_json::json!({"relative":n.relative,"volume":n.id.volume,"fileId":n.id.index.to_string(),
                    "directory":n.directory,"bytes":n.bytes.len(),"sha256":hash::sha256_hex(&n.bytes),"security":n.security,"children":n.children})).collect::<Vec<_>>(),
                "retired":r.retired.iter().map(|n|serde_json::json!({"path":n.path,"relative":n.entry.relative,"volume":n.entry.id.volume,
                    "fileId":n.entry.id.index.to_string(),"bytes":n.entry.bytes.len(),"sha256":hash::sha256_hex(&n.entry.bytes),"security":n.entry.security})).collect::<Vec<_>>(),
                "retainedNativeArchives":r.archives.len()})).collect::<Vec<_>>(),"nativeAuthority":false})
    }
}
