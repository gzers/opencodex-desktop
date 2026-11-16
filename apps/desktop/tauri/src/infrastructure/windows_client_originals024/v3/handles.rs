//! Canonical exact own-handle close DATA; no native capability or typed Closed.
use super::super::handle_release::HandleRow;
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum HandleOperation {
    OriginalHandlesReleaseIntent,
    OriginalHandlesReleasedVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HandleEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: HandleOperation,
    pub(super) history_digest: String,
    pub(super) inventory_digest: String,
    pub(super) rows: Option<Vec<HandleRow>>,
    pub(super) count: usize,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) deadline_scope: String,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) previous_event_digest: String,
}
#[cfg(test)]
mod tests {
    use super::super::{history::tests::linked, tests::plan};
    use super::*;
    fn rows(p: &StoragePlan) -> Vec<HandleRow> {
        p.roots
            .iter()
            .enumerate()
            .flat_map(|(r, p)| {
                p.nodes.iter().enumerate().flat_map(move |(i, n)| {
                    ["original", "history-source"].map(move |role| HandleRow {
                        key: format!("tree/{r}/{role}/{i:04}"),
                        volume: n.volume,
                        file_id: n.file_id.clone(),
                        directory: n.kind == Kind::DirectoryAnchor,
                        historical_work: false,
                    })
                })
            })
            .collect()
    }
    #[test]
    fn wl2_r6_handles_data_exact_inventory_history_scope_skip_drift_and_text_closed_refuse() {
        let (mut f, p, _, _) = plan();
        let mut g = linked(&p);
        let good = serde_json::to_value(
            g.handle_event(
                HandleOperation::OriginalHandlesReleaseIntent,
                Some(rows(&p)),
            )
            .unwrap(),
        )
        .unwrap();
        for field in [
            "historyDigest",
            "inventoryDigest",
            "claimKey",
            "leaseEpoch",
            "deadlineScope",
            "manifestDigest",
            "bindingDigest",
            "previousEventDigest",
        ] {
            let mut bad = good.clone();
            bad[field] = "foreign".into();
            let tip = g.tip.clone();
            assert!(g
                .apply_handles(&p, &serde_json::from_value(bad).unwrap())
                .is_err());
            assert_eq!(g.tip, tip);
            assert!(g.handle_progress.is_none());
        }
        for mutate in 0..3 {
            let mut invalid_rows = rows(&p);
            match mutate {
                0 => {
                    invalid_rows.pop();
                }
                1 => {
                    invalid_rows[0].file_id = "999".into();
                }
                _ => {
                    invalid_rows.push(HandleRow {
                        key: invalid_rows[0].key.clone(),
                        volume: 0,
                        file_id: "99".into(),
                        directory: false,
                        historical_work: false,
                    });
                }
            }
            assert!(g
                .apply_handles(
                    &p,
                    &g.handle_event(
                        HandleOperation::OriginalHandlesReleaseIntent,
                        Some(invalid_rows)
                    )
                    .unwrap()
                )
                .is_err());
        }
        g.apply_entry(
            &p,
            &segments::ChainEntry::Handles(serde_json::from_value(good).unwrap()),
        )
        .unwrap();
        assert!(!g.handle_progress.as_ref().unwrap().released);
        assert!(g
            .apply_handles(
                &p,
                &g.handle_event(
                    HandleOperation::OriginalHandlesReleaseIntent,
                    Some(rows(&p))
                )
                .unwrap()
            )
            .is_err());
        g.apply_handles(
            &p,
            &g.handle_event(HandleOperation::OriginalHandlesReleasedVerified, None)
                .unwrap(),
        )
        .unwrap();
        assert!(g.handle_progress.as_ref().unwrap().released);
        assert!(g
            .apply_handles(
                &p,
                &g.handle_event(HandleOperation::OriginalHandlesReleasedVerified, None)
                    .unwrap()
            )
            .is_err());
        let mut unknown = serde_json::to_value(
            g.handle_event(HandleOperation::OriginalHandlesReleasedVerified, None)
                .unwrap(),
        )
        .unwrap();
        unknown["Closed"] = true.into();
        assert!(serde_json::from_value::<segments::ChainEntry>(unknown).is_err());
        println!("r6 exact-handle close DATA binds original/history-source complete inventory and original native/history/claim/deadline context; drift/missing/duplicate/wrong IDs/replay/textClosed refuse, no actual handles or native close proof minted");
        f.retain();
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HandleSummary {
    pub(super) inventory_digest: String,
    pub(super) count: usize,
    pub(super) released: bool,
}
impl Progress {
    pub(super) fn handle_event(
        &self,
        operation: HandleOperation,
        rows: Option<Vec<HandleRow>>,
    ) -> io::Result<HandleEvent> {
        let (inventory_digest, count) = if let Some(rows) = &rows {
            (hash::sha256_hex(&encoded(rows)?), rows.len())
        } else {
            let state = self.handle_progress.as_ref().ok_or_else(invalid)?;
            (state.inventory_digest.clone(), state.count)
        };
        Ok(HandleEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "original-handle-release-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            operation,
            history_digest: hash::sha256_hex(&encoded(
                &self.history_summary()?.ok_or_else(invalid)?,
            )?),
            inventory_digest,
            rows,
            count,
            manifest_digest: self.manifest.clone(),
            binding_digest: self.binding.clone(),
            storage_plan_digest: self.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            deadline_scope: self.deadline.clone(),
            lease_epoch: self.epoch.clone(),
            claim_key: self.claim.clone(),
            previous_event_digest: self.tip.clone(),
        })
    }
    pub(super) fn apply_handles(&mut self, plan: &StoragePlan, e: &HandleEvent) -> io::Result<()> {
        if self.batch.is_some() {
            return Err(invalid());
        }
        let summary = self.history_summary()?.ok_or_else(invalid)?;
        if !self.complete_files(plan)||!self.history_linked()||self.lease_state!=segments::LeaseState::Released
            ||e.schema!=crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            ||e.document_kind!="original-handle-release-event"||e.storage_strategy!=ANCHORED_STORAGE_STRATEGY||e.session_id!=self.session
            ||e.sequence!=self.sequence.checked_add(1).ok_or_else(invalid)?||!uuid(&e.event_id)||self.event_ids.contains(&e.event_id)
            ||e.previous_event_digest!=self.tip||e.manifest_digest!=self.manifest||e.binding_digest!=self.binding||e.storage_plan_digest!=self.plan
            ||e.security_policy_digest!=Policy::fixed().digest()?||e.deadline_scope!=self.deadline||e.lease_epoch!=self.epoch||e.claim_key!=self.claim
            ||e.history_digest!=hash::sha256_hex(&encoded(&summary)?)||!digest(&e.inventory_digest)||e.count==0||e.count>4096 {return Err(invalid());}
        match e.operation {
            HandleOperation::OriginalHandlesReleaseIntent => {
                let rows = e.rows.as_ref().ok_or_else(invalid)?;
                if self.handle_progress.is_some()
                    || rows.len() != e.count
                    || hash::sha256_hex(&encoded(rows)?) != e.inventory_digest
                {
                    return Err(invalid());
                }
                let mut keys = BTreeSet::new();
                for r in rows {
                    super::handle_rows::validate(r, plan, self)?;
                    if !keys.insert(&r.key)
                        || !decimal(&r.file_id)
                        || r.directory && r.historical_work
                        || !relative(&r.key)
                        || !r.key.starts_with("tree/") && !r.key.starts_with("lease/")
                    {
                        return Err(invalid());
                    }
                }
                for (root, p) in plan.roots.iter().enumerate() {
                    for (i, n) in p.nodes.iter().enumerate() {
                        for role in ["original", "history-source"] {
                            let key = format!("tree/{root}/{role}/{i:04}");
                            let row = rows.iter().find(|r| r.key == key).ok_or_else(invalid)?;
                            if row.volume != n.volume
                                || row.file_id != n.file_id
                                || row.directory != (n.kind == Kind::DirectoryAnchor)
                                || row.historical_work
                            {
                                return Err(invalid());
                            }
                        }
                    }
                }
                self.handle_progress = Some(HandleSummary {
                    inventory_digest: e.inventory_digest.clone(),
                    count: e.count,
                    released: false,
                });
            }
            HandleOperation::OriginalHandlesReleasedVerified => {
                let state = self.handle_progress.as_mut().ok_or_else(invalid)?;
                if state.released
                    || e.rows.is_some()
                    || state.inventory_digest != e.inventory_digest
                    || state.count != e.count
                {
                    return Err(invalid());
                }
                state.released = true;
            }
        }
        self.sequence = e.sequence;
        self.tip = hash::sha256_hex(&encoded(e)?);
        if e.operation == HandleOperation::OriginalHandlesReleasedVerified {
            self.handle_close = Some((e.sequence, self.tip.clone()));
        }
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
}
