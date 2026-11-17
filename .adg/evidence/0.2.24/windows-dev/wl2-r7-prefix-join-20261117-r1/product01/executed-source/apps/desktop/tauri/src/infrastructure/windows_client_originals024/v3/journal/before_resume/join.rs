//! DATA comparison only after actual native tree/canonical observations. Never
//! writes inferred completions or constructs claim/Owner/restore authority.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::batches::{BatchKind, RowEvent};
use std::collections::BTreeMap;
impl Journal {
    pub(super) fn match_before_resume_trees(
        &self,
        trees: &[serde_json::Value; 2],
        epoch: &str,
        claim: &str,
    ) -> io::Result<()> {
        if self.manifest.storage_plan.roots.len() != 2 {
            return Err(invalid());
        }
        let mut batch_native = BTreeMap::new();
        if let Some(batch) = &self.progress.batch {
            // This exact live native history, never serialized row counts,
            // establishes which intended initial operations actually happened.
            let (id, kind, count, prefix) = batch.before_resume_parts()?;
            let matches = self
                .native_batches
                .iter()
                .filter(|b| b.id == id && b.kind == kind)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(invalid());
            }
            let native = matches[0];
            if native.committed || native.values.len() < prefix || native.values.len() > count {
                return Err(invalid());
            }
            let rows = batch.before_resume_rows()?;
            for (ordinal, row) in rows.iter().enumerate() {
                let RowEvent::Original(e) = &row.event else {
                    return Err(invalid());
                };
                let value = native.values.get(ordinal);
                if let Some(value) = value {
                    if value.node() != e.node_id || value.root() != e.root_id {
                        return Err(invalid());
                    }
                }
                if batch_native
                    .insert(e.node_id.as_str(), (kind, value))
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for (root, view) in self.manifest.storage_plan.roots.iter().zip(trees) {
            if view["purpose"] != "before-resume-retained-tree-read-data"
                || view["runtimeAuthority"] != false
                || view["claimEpoch"] != epoch
                || view["claimKey"] != claim
                || view["plan"] != serde_json::to_value(root).map_err(|_| invalid())?
            {
                return Err(invalid());
            }
            let nodes = view["nodes"].as_array().ok_or_else(invalid)?;
            if nodes.len() != root.nodes.len() {
                return Err(invalid());
            }
            for (n, actual) in root.nodes.iter().zip(nodes) {
                if !seen.insert(n.node_id.as_str())
                    || actual["nodeId"] != n.node_id
                    || actual["originalVolume"] != n.volume
                    || actual["originalFileId"] != n.file_id
                    || actual["originalSecurity"]
                        != serde_json::to_value(&n.security).map_err(|_| invalid())?
                    || actual["originalBytes"] != n.length
                    || actual["originalSha256"] != n.content_digest
                {
                    return Err(invalid());
                }
                let (phase, work_id) = self.progress.nodes.get(&n.node_id).ok_or_else(invalid)?;
                let state = actual["state"].as_str().ok_or_else(invalid)?;
                let mut expected_work = None;
                let expected = if n.kind != Kind::CustodiedFile {
                    if *phase > 1 || !actual["work"].is_null() {
                        return Err(invalid());
                    }
                    "Original"
                } else if let Some((kind, value)) = batch_native.get(n.node_id.as_str()) {
                    match kind {
                        BatchKind::OriginalMove => {
                            if ![1, 2].contains(phase) {
                                return Err(invalid());
                            }
                            if let Some(v) = value {
                                let (id, sd) = v.parts()?;
                                if id != n.file_id || sd != n.security {
                                    return Err(invalid());
                                }
                                "Moved"
                            } else {
                                if *phase != 1 {
                                    return Err(invalid());
                                }
                                "Original"
                            }
                        }
                        BatchKind::WorkCreate => {
                            if ![3, 4].contains(phase) {
                                return Err(invalid());
                            }
                            if let Some(v) = value {
                                expected_work = Some(v.parts()?);
                                "Working"
                            } else {
                                if *phase != 3 {
                                    return Err(invalid());
                                }
                                "Moved"
                            }
                        }
                        _ => return Err(invalid()),
                    }
                } else {
                    match phase {
                        0 => "Original",
                        2 => "Moved",
                        4 => "Working",
                        // Durable single-node intent may precede or follow the
                        // actual same-object mutation. Keep it an intent; the
                        // read only identifies an exact retained native state.
                        1 | 3 if self.progress.pending.is_some() => {
                            let e: Event =
                                serde_json::from_slice(self.progress.pending.as_ref().unwrap())
                                    .map_err(|_| invalid())?;
                            let allowed = if *phase == 1 {
                                ["Original", "Moved"]
                            } else {
                                ["Moved", "Working"]
                            };
                            if e.node_id != n.node_id || !allowed.contains(&state) {
                                return Err(invalid());
                            }
                            state
                        }
                        _ => return Err(invalid()),
                    }
                };
                if state != expected {
                    return Err(invalid());
                }
                let location = if ["Moved", "Working"].contains(&state) {
                    PathBuf::from(&root.custody).join(n.target.as_ref().ok_or_else(invalid)?)
                } else {
                    PathBuf::from(&root.root).join(&n.relative)
                };
                if actual["originalLocation"]
                    != serde_json::to_value(location).map_err(|_| invalid())?
                {
                    return Err(invalid());
                }
                let work = &actual["work"];
                if state == "Working" {
                    let id = work["fileId"].as_str().ok_or_else(invalid)?;
                    let sd: SnapshotData =
                        serde_json::from_value(work["security"].clone()).map_err(|_| invalid())?;
                    sd.validate()?;
                    if !decimal(id)
                        || id == n.file_id
                        || work["volume"] != n.volume
                        || !sd.matches_id(n.volume, id)
                        || work["bytes"] != n.length
                        || work["sha256"] != n.content_digest
                        || work["location"]
                            != serde_json::to_value(PathBuf::from(&root.root).join(&n.relative))
                                .map_err(|_| invalid())?
                    {
                        return Err(invalid());
                    }
                    if let Some((expected_id, expected_sd)) = &expected_work {
                        if id != expected_id || &sd != expected_sd {
                            return Err(invalid());
                        }
                    }
                    if *phase == 4 {
                        if work_id.as_deref() != Some(id)
                            || self.progress.work_security.get(&n.node_id)
                                != Some(&hash::sha256_hex(&encoded(&sd)?))
                        {
                            return Err(invalid());
                        }
                    }
                } else if !work.is_null() {
                    return Err(invalid());
                }
            }
        }
        if seen.len() != self.progress.nodes.len() {
            return Err(invalid());
        }
        Ok(())
    }
}
