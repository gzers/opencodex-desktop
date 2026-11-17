//! Complete read-only current view. No DELETE right, journal append, snapshot
//! directory, claim handoff or mutation permission is produced by this reader.
use super::*;
fn collect_read(
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
    // Ordinary open_node(observe=true) intentionally shares writes so official
    // atomic updates can continue. A complete read barrier instead needs its
    // OWN read-data, deny-write view; keep DELETE absent and coordinator sharing
    // unchanged. Conflicting live writer is a refusal, never a release/retry.
    validate_local_path(path)?;
    let mut file = OpenOptions::new()
        .access_mode(if kind == Kind::Directory {
            0x20080
        } else {
            0x80020000
        })
        .share_mode(if kind == Kind::File { 1 } else { 1 | 2 })
        .custom_flags(
            0x200000
                | if kind == Kind::Directory {
                    0x2000000
                } else {
                    0
                },
        )
        .open(path)?;
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
        collect_read(root, &path.join(child), &next, nodes, total, end)?;
    }
    Ok(())
}
fn artifact(root: &Path, entry: &Entry) -> bool {
    root.file_name().and_then(|n| n.to_str()) == Some(".codex")
        && [
            "opencodex-catalog.json",
            "models_cache.json",
            "opencodex-catalog-audit.jsonl",
        ]
        .contains(&entry.relative.as_str())
}
impl AnchoredTree {
    pub(in crate::infrastructure::windows_client_originals024) fn read_ready_tree(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            check(end)?;
            if self.poisoned
                || self.work_mode != WorkMode::Open
                || self.observed.is_some()
                || self.release_view.is_some()
                || self.ready_read.is_some()
            {
                return Err(invalid());
            }
            self.verify_custody_anchors(end)?;
            // Retain the partial view BEFORE walking. Any error keeps exact
            // observed file generations and original custody in the same tree.
            self.ready_read = Some(WorkFrame::empty(self, end)?);
            let frame = self.ready_read.as_mut().ok_or_else(invalid)?;
            collect_read(&frame.root, &frame.root, "", &mut frame.nodes, &mut 0, end)?;
            frame
                .nodes
                .sort_by(|a, b| a.entry.relative.cmp(&b.entry.relative));
            frame.check(end)?;
            if frame.root != self.root
                || frame.parent != self.parent.identity()
                || frame.parent_security != self.parent_security
            {
                return Err(invalid());
            }
            if frame
                .nodes
                .iter()
                .any(|n| artifact(&self.root, &n.entry) && n.entry.directory)
            {
                return Err(invalid());
            }
            let actual = frame
                .nodes
                .iter()
                .map(|n| &n.entry)
                .filter(|n| !artifact(&self.root, n))
                .collect::<Vec<_>>();
            let expected = self
                .nodes
                .iter()
                .map(|n| {
                    if n.kind == Kind::File {
                        n.work.as_ref().map(|w| &w.1).ok_or_else(invalid)
                    } else {
                        Ok(&n.original)
                    }
                })
                .collect::<io::Result<Vec<_>>>()?;
            let expected = expected
                .into_iter()
                .filter(|n| !artifact(&self.root, n))
                .collect::<Vec<_>>();
            if actual.len() != expected.len() {
                return Err(invalid());
            }
            for (a, b) in actual.into_iter().zip(expected) {
                if a.relative != b.relative
                    || a.directory != b.directory
                    || a.id != b.id
                    || a.security != b.security
                    || a.bytes != b.bytes
                {
                    return Err(invalid());
                }
                if a.relative.is_empty()
                    && self.root.file_name().and_then(|n| n.to_str()) == Some(".codex")
                {
                    let names = |v: &Vec<String>| {
                        v.iter()
                            .filter(|n| {
                                ![
                                    "opencodex-catalog.json",
                                    "models_cache.json",
                                    "opencodex-catalog-audit.jsonl",
                                ]
                                .contains(&n.as_str())
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    if names(&a.children) != names(&b.children) {
                        return Err(invalid());
                    }
                } else if a.children != b.children {
                    return Err(invalid());
                }
            }
            self.verify_custody_anchors(end)?;
            self.ready_read.as_mut().ok_or_else(invalid)?.check(end)?;
            check(end)?;
            // Keep this complete temporary view until the caller has BOTH
            // client roots held and final typed source/Ready checks completed.
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_ready_tree_read(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.poisoned || self.work_mode != WorkMode::Open {
                return Err(invalid());
            }
            self.verify_custody_anchors(end)?;
            self.ready_read.as_mut().ok_or_else(invalid)?.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_ready_tree_read(
        &mut self,
    ) {
        // Only the caller's already-completed BOTH-root read barrier releases
        // temporary observations. Original client/custody/lease state unchanged.
        self.ready_read = None;
    }
}
