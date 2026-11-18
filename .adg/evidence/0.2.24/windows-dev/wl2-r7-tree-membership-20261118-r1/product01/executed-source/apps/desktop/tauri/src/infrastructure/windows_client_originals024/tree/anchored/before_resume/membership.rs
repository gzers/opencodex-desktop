//! Actual retained original/work/pending resource membership. DATA only, never
//! complete Source/claim/Owner/return admission. Missing initial baselines stay
//! nullable and poison remains unchanged; no mutation is performed here.
use super::*;
use crate::infrastructure::windows_client_originals024::before_resume_record_v2::{
    projection_digest, Object, Role, Stage,
};
#[cfg(test)]
thread_local! {static AFTER_MEMBERSHIP:std::cell::Cell<Option<fn(&Path)>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]
fn after_for_test(hook: fn(&Path)) {
    AFTER_MEMBERSHIP.with(|slot| {
        assert!(slot.get().is_none());
        slot.set(Some(hook));
    });
}
fn node_object(
    session: uuid::Uuid,
    role: Role,
    slot: &str,
    relative: &str,
    locator: &Path,
    stage: Stage,
    observation: Option<&serde_json::Value>,
) -> io::Result<Object> {
    Ok(Object {
        object_id: projection_digest(&(session.to_string(), role, slot, relative))?,
        role,
        original_stage: stage,
        path_digest: projection_digest(&locator)?,
        identity_digest: observation
            .and_then(|data| data["identityDigest"].as_str().map(str::to_string)),
        security_digest: observation
            .filter(|data| data["originalBaselineVerified"] == true)
            .and_then(|data| data["observedSecurityDigest"].as_str().map(str::to_string)),
        content_digest: observation
            .and_then(|data| data["contentDigest"].as_str().map(str::to_string)),
    })
}
fn read_file(
    path: &Path,
    relative: &str,
    file: &mut File,
    kind: Kind,
    baseline: Option<&Entry>,
    expected_prefix: Option<&[u8]>,
    directory_children: Option<&[String]>,
    end: Instant,
) -> io::Result<serde_json::Value> {
    check(end)?;
    if kind == Kind::Directory {
        native::verify_private_directory(file)?;
    } else {
        native::verify_private_file(file)?;
    }
    let first = stamp(path, file, relative, kind, end)?;
    let fresh = open_node(path, kind, true)?;
    if id(&fresh, kind == Kind::Directory)? != first.id
        || native_stamp(&fresh, end).map_err(failure)? != first.security
    {
        return Err(invalid());
    }
    if let Some(old) = baseline {
        if first.id != old.id
            || first.security != old.security
            || kind != Kind::Directory && first.bytes != old.bytes
        {
            return Err(invalid());
        }
    }
    if let Some(expected) = directory_children {
        if first.children != expected {
            return Err(invalid());
        }
    }
    if let Some(expected) = expected_prefix {
        if !expected.starts_with(&first.bytes) {
            return Err(invalid());
        }
    }
    let second = stamp(path, file, relative, kind, end)?;
    if first != second
        || id(&fresh, kind == Kind::Directory)? != first.id
        || native_stamp(&fresh, end).map_err(failure)? != first.security
    {
        return Err(invalid());
    }
    check(end)?;
    Ok(
        serde_json::json!({"identityDigest":projection_digest(&(first.id.volume,first.id.index.to_string()))?,"observedSecurityDigest":projection_digest(&first.security)?,
        "contentDigest":if kind==Kind::Directory{projection_digest(&first.children)?}else{hash::sha256_hex(&first.bytes)},"bytes":first.bytes.len(),"originalBaselineVerified":baseline.is_some(),
        "writeState":if expected_prefix.is_some(){if first.bytes.is_empty(){"empty"}else if first.bytes.len()==expected_prefix.unwrap().len(){"completeBytes"}else{"partialBytes"}}else{"retainedSnapshot"}}),
    )
}
impl AnchoredTree {
    fn native_membership_pass(
        &mut self,
        session: uuid::Uuid,
        role: Role,
        end: Instant,
    ) -> io::Result<(Vec<Object>, Vec<serde_json::Value>)> {
        check(end)?;
        self.parent.recheck()?;
        self.custody.recheck()?;
        if parent_stamp(self.parent.path(), end)? != self.parent_security
            || parent_stamp(self.custody.path(), end)? != self.custody_security
        {
            return Err(invalid());
        }
        if self.pending_capture.is_some() && self.pending_capture_locator.is_none() {
            return Err(invalid());
        }
        let original_only = self.nodes.iter().all(|node| {
            node.state == State::Original && node.work.is_none() && node.pending_work.is_none()
        });
        let mut objects = Vec::new();
        let mut rows = Vec::new();
        let mut custody_members = Vec::new();
        for index in 0..self.nodes.len() {
            check(end)?;
            let original_path = self.original_location(index);
            let work_path = self.work_location(index);
            let logical = self.root.join(&self.nodes[index].original.relative);
            let children = if self.nodes[index].kind == Kind::Directory {
                Some(if original_only {
                    self.nodes[index].original.children.clone()
                } else {
                    let mut expected = self.expected_children(&self.nodes[index].original.relative);
                    // An actual pending creator has published its name even before
                    // State::Working. Its original retained handle must explain it.
                    for node in &self.nodes {
                        if node.pending_work.is_some()
                            && Path::new(&node.original.relative).parent()
                                == Some(Path::new(&self.nodes[index].original.relative))
                        {
                            expected.push(
                                Path::new(&node.original.relative)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .ok_or_else(invalid)?
                                    .to_string(),
                            );
                        }
                    }
                    expected.sort();
                    expected.dedup();
                    expected
                })
            } else {
                None
            };
            let node = &mut self.nodes[index];
            if node.work.is_some() && node.pending_work.is_some() {
                return Err(invalid());
            }
            let observed = read_file(
                &original_path,
                &node.original.relative,
                &mut node.file,
                node.kind,
                Some(&node.original),
                None,
                children.as_deref(),
                end,
            )?;
            objects.push(node_object(
                session,
                role,
                "original",
                &node.original.relative,
                &logical,
                Stage::HandleRetained,
                Some(&observed),
            )?);
            rows.push(serde_json::json!({"slot":"original","relativeDigest":projection_digest(&node.original.relative)?,"observation":observed}));
            if original_path.parent() == Some(self.custody.path()) {
                custody_members.push(
                    original_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(invalid)?
                        .to_string(),
                );
            }
            if let Some((work, snapshot)) = &mut node.work {
                let observed = read_file(
                    &work_path,
                    &node.original.relative,
                    work,
                    Kind::File,
                    Some(snapshot),
                    None,
                    None,
                    end,
                )?;
                objects.push(node_object(
                    session,
                    role,
                    "work",
                    &node.original.relative,
                    &logical,
                    Stage::HandleRetained,
                    Some(&observed),
                )?);
                rows.push(serde_json::json!({"slot":"work","relativeDigest":projection_digest(&node.original.relative)?,"observation":observed}));
                if work_path.parent() == Some(self.custody.path()) {
                    custody_members.push(
                        work_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .ok_or_else(invalid)?
                            .to_string(),
                    );
                }
            }
            if let Some(work) = &mut node.pending_work {
                let observed = read_file(
                    &work_path,
                    &node.original.relative,
                    work,
                    Kind::File,
                    None,
                    Some(&node.original.bytes),
                    None,
                    end,
                )?;
                objects.push(node_object(
                    session,
                    role,
                    "work",
                    &node.original.relative,
                    &logical,
                    Stage::HandleRetained,
                    Some(&observed),
                )?);
                rows.push(serde_json::json!({"slot":"pendingWork","relativeDigest":projection_digest(&node.original.relative)?,"observation":observed}));
            }
        }
        if let Some((relative, kind)) = &self.pending_capture_locator {
            let path = self.root.join(relative);
            let observed = if let Some(file) = &mut self.pending_capture {
                Some(read_file(
                    &path, relative, file, *kind, None, None, None, end,
                )?)
            } else {
                None
            };
            objects.push(node_object(
                session,
                role,
                "original",
                relative,
                &path,
                if observed.is_some() {
                    Stage::HandleRetained
                } else {
                    Stage::IntentPublished
                },
                observed.as_ref(),
            )?);
            rows.push(serde_json::json!({"slot":"pendingCapture","relativeDigest":projection_digest(relative)?,"observation":observed}));
        }
        custody_members.sort();
        if names(self.custody.path())? != custody_members {
            return Err(invalid());
        }
        for (slot, pin, sd) in [
            ("parent", &self.parent, &self.parent_security),
            ("custody", &self.custody, &self.custody_security),
        ] {
            let data = serde_json::json!({"identityDigest":projection_digest(&(pin.identity().volume,pin.identity().index.to_string()))?,"observedSecurityDigest":projection_digest(sd)?,"originalBaselineVerified":true,"contentDigest":null});
            objects.push(node_object(
                session,
                role,
                slot,
                "",
                pin.path(),
                Stage::HandleRetained,
                Some(&data),
            )?);
        }
        self.parent.recheck()?;
        self.custody.recheck()?;
        if parent_stamp(self.parent.path(), end)? != self.parent_security
            || parent_stamp(self.custody.path(), end)? != self.custody_security
        {
            return Err(invalid());
        }
        objects.sort_by(|a, b| a.object_id.cmp(&b.object_id));
        if objects.windows(2).any(|p| p[0].object_id == p[1].object_id) {
            return Err(invalid());
        }
        check(end)?;
        Ok((objects, rows))
    }
    pub(in crate::infrastructure::windows_client_originals024) fn v2_retained_object_data(
        &mut self,
        session: uuid::Uuid,
        role: Role,
        end: Instant,
    ) -> io::Result<(Vec<Object>, Vec<serde_json::Value>)> {
        let result = (|| {
            let (first, first_rows) = self.native_membership_pass(session, role, end)?;
            #[cfg(test)]
            AFTER_MEMBERSHIP.with(|slot| {
                if let Some(hook) = slot.take() {
                    hook(&self.root);
                }
            });
            let (second, second_rows) = self.native_membership_pass(session, role, end)?;
            if first != second || first_rows != second_rows {
                return Err(invalid());
            }
            check(end)?;
            // Baseline/pending diagnostics remain out of the error list; rows
            // never contain raw bytes, secrets or arbitrary path authority.
            Ok((second, Vec::new()))
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
