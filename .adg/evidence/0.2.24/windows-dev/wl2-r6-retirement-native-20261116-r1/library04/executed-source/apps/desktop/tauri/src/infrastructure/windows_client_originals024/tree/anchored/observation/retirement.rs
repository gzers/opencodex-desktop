//! Complete current WORK retirement, original identities kept separate. This
//! native component grants no permission. Execution stays private until the
//! actual typed Stop/content + canonical node intent/result driver is joined.
use super::*;
struct Item {
    frame_index: usize,
    retired: bool,
    target: PathBuf,
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) struct NativeRetirement {
    frame: WorkFrame,
    items: Vec<Item>,
    plan: serde_json::Value,
}
fn ordinary(tree: &AnchoredTree, entry: &Entry) -> bool {
    !entry.directory
        && !(tree.root.file_name().and_then(|s| s.to_str()) == Some(".codex")
            && ANCHORED_CODEX_STABLE_FILES.contains(&entry.relative.as_str()))
}
impl AnchoredTree {
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn capture_retirement(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            claim.check(&root, end)?;
            if self.work_mode != WorkMode::Observed
                || self.retirement.is_some()
                || self.release_view.is_some()
            {
                return Err(invalid());
            }
            self.verify_custody_anchors(end)?;
            let frame = self.observed.as_mut().ok_or_else(invalid)?;
            frame.check(end)?;
            let reference = frame.archive.as_mut().ok_or_else(invalid)?.reference(end)?;
            let mut plan = serde_json::json!({"schema":"windows-anchored-native-retirement-data-v3","storageStrategy":"windows-anchored-directories-v1","originalTree":self.plan(),"currentTree":self.observed.as_ref().ok_or_else(invalid)?.metadata(),"currentSnapshot":reference,"runtimeAuthority":false});
            let frame = self.observed.take().ok_or_else(invalid)?;
            let items = frame
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| ordinary(self, &n.entry))
                .map(|(index, n)| {
                    let key = hash::sha256_hex(
                        format!(
                            "{}/{}",
                            self.root.file_name().unwrap().to_str().unwrap(),
                            n.entry.relative
                        )
                        .as_bytes(),
                    );
                    Item {
                        frame_index: index,
                        retired: false,
                        target: self
                            .custody
                            .path()
                            .join(format!("retired-current-{key}-{}", n.entry.id.index)),
                    }
                })
                .collect::<Vec<_>>();
            plan["retirements"]=serde_json::json!(items.iter().map(|item|{
                let entry=&frame.nodes[item.frame_index].entry;
                serde_json::json!({"relative":entry.relative,"currentVolume":entry.id.volume,"currentFileId":entry.id.index.to_string(),"target":item.target,"targetParentId":self.custody.identity().index.to_string(),"sourceParentId":self.nodes.iter().find(|n|n.kind==Kind::Directory&&Path::new(&entry.relative).parent()==Some(Path::new(&n.original.relative))).map(|n|n.original.id.index.to_string()),"originalFileId":self.nodes.iter().find(|n|n.kind==Kind::File&&n.original.relative==entry.relative).map(|n|n.original.id.index.to_string()),"originalAbsent":!self.nodes.iter().any(|n|n.original.relative==entry.relative),"bytes":entry.bytes.len(),"sha256":hash::sha256_hex(&entry.bytes),"security":entry.security})
            }).collect::<Vec<_>>());
            self.retirement = Some(NativeRetirement {
                frame,
                items,
                plan: plan.clone(),
            });
            self.verify_retirement_prefix(claim, end)?;
            Ok(plan)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn verify_retirement_prefix(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            claim.check(&root, end)?;
            self.check_retirement_objects(end)?;
            claim.check(&root, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn check_retirement_objects(&mut self, end: Instant) -> io::Result<()> {
        if self.poisoned {
            return Err(invalid());
        }
        check(end)?;
        self.parent.recheck()?;
        self.custody.recheck()?;
        if parent_stamp(self.parent.path(), end)? != self.parent_security
            || parent_stamp(self.custody.path(), end)? != self.custody_security
        {
            return Err(invalid());
        }
        for old in &mut self.snapshot_history {
            old.check(end)?;
        }
        let frozen = self.retirement.as_mut().ok_or_else(invalid)?;
        if frozen.frame.archive_failed
            || frozen.frame.root != self.root
            || frozen.frame.parent != self.parent.identity()
            || frozen.frame.parent_security != self.parent_security
        {
            return Err(invalid());
        }
        frozen
            .frame
            .archive
            .as_mut()
            .ok_or_else(invalid)?
            .check(end)?;
        let mut custody_names = Vec::<String>::new();
        let mut live = Vec::<String>::new();
        for item in &mut frozen.items {
            let n = frozen
                .frame
                .nodes
                .get_mut(item.frame_index)
                .ok_or_else(invalid)?;
            let path = if item.retired {
                item.target.clone()
            } else {
                self.root.join(&n.entry.relative)
            };
            if stamp(&path, &mut n.file, &n.entry.relative, Kind::File, end)? != n.entry {
                return Err(invalid());
            }
            let fresh = open_node(&path, Kind::File, true)?;
            if id(&fresh, false)? != n.entry.id {
                return Err(invalid());
            }
            if item.retired {
                custody_names.push(
                    item.target
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(invalid)?
                        .into(),
                );
            } else {
                require_absent(&item.target)?;
                live.push(n.entry.relative.clone());
            }
        }
        for i in 0..self.nodes.len() {
            let path = self.original_location(i);
            let n = &mut self.nodes[i];
            if n.pending_work.is_some()
                || !matches!(n.state, State::Working | State::Restored | State::Original)
            {
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
                custody_names.push(
                    path.file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(invalid)?
                        .into(),
                );
            }
            if n.kind != Kind::File || matches!(n.state, State::Restored | State::Original) {
                live.push(n.original.relative.clone());
            }
            if let Some((file, entry)) = &n.work {
                if n.work_open {
                    retained_work_identity(file, entry.id)?;
                }
            }
        }
        live.sort();
        if live.windows(2).any(|p| p[0].eq_ignore_ascii_case(&p[1])) {
            return Err(invalid());
        }
        for n in &self.nodes {
            if n.kind != Kind::Directory {
                continue;
            }
            let mut expected = live
                .iter()
                .filter(|p| {
                    !p.is_empty() && Path::new(p).parent() == Some(Path::new(&n.original.relative))
                })
                .map(|p| {
                    Path::new(p)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
                .collect::<Vec<_>>();
            expected.sort();
            if names(&self.root.join(&n.original.relative))? != expected {
                return Err(invalid());
            }
        }
        custody_names.sort();
        if names(self.custody.path())? != custody_names {
            return Err(invalid());
        }
        check(end)
    }
    fn readback_retirement_after_release(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            self.check_retirement_objects(end)?;
            if self
                .retirement
                .as_ref()
                .ok_or_else(invalid)?
                .items
                .iter()
                .any(|i| !i.retired)
                || self
                    .nodes
                    .iter()
                    .any(|n| n.kind == Kind::File && n.state != State::Restored)
            {
                return Err(invalid());
            }
            for n in &self.nodes {
                if n.kind == Kind::Directory
                    && names(&self.root.join(&n.original.relative))? != n.original.children
                {
                    return Err(invalid());
                }
            }
            if self.root.file_name().and_then(|s| s.to_str()) == Some(".codex") {
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
                super::super::super::locks::prove(&self.root, &self.nodes[0].file, &entries, end)?;
            }
            self.check_retirement_objects(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn retire_current_file(
        &mut self,
        index: usize,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.verify_retirement_prefix(claim, end)?;
            let root = self.parent.path().join(".codex");
            let frozen = self.retirement.as_mut().ok_or_else(invalid)?;
            let item = frozen.items.get_mut(index).ok_or_else(invalid)?;
            if item.retired {
                return Err(invalid());
            }
            require_absent(&item.target)?;
            claim.check(&root, end)?;
            check(end)?;
            self.poisoned = true;
            rename_original_handle(&frozen.frame.nodes[item.frame_index].file, &item.target)?;
            item.retired = true;
            self.poisoned = false;
            self.verify_retirement_prefix(claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn restore_original_file(
        &mut self,
        index: usize,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.verify_retirement_prefix(claim, end)?;
            if self
                .retirement
                .as_ref()
                .ok_or_else(invalid)?
                .items
                .iter()
                .any(|i| !i.retired)
            {
                return Err(invalid());
            }
            let root = self.parent.path().join(".codex");
            let n = self.nodes.get_mut(index).ok_or_else(invalid)?;
            if n.kind != Kind::File || n.state != State::Working {
                return Err(invalid());
            }
            let target = self.root.join(&n.original.relative);
            require_absent(&target)?;
            claim.check(&root, end)?;
            check(end)?;
            self.poisoned = true;
            rename_original_handle(&n.file, &target)?;
            n.state = State::Restored;
            self.poisoned = false;
            self.verify_retirement_prefix(claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn verify_retirement_restored(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.verify_retirement_prefix(claim, end)?;
            if self
                .retirement
                .as_ref()
                .ok_or_else(invalid)?
                .items
                .iter()
                .any(|i| !i.retired)
                || self
                    .nodes
                    .iter()
                    .any(|n| n.kind == Kind::File && n.state != State::Restored)
            {
                return Err(invalid());
            }
            for n in &self.nodes {
                if n.kind == Kind::Directory
                    && names(&self.root.join(&n.original.relative))? != n.original.children
                {
                    return Err(invalid());
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
#[path = "retirement_tests.rs"]
mod tests;
