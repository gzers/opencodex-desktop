//! Native witnesses join complete durable row plans to actual ordered steps.
//! No result is synthesized from counts/JSON; every completion has a retained
//! native readback value and exact unchanged single-node DATA validation.
use super::super::super::{
    coordination::AnchoredClaim,
    tree::{InitialBatchWitness, RecoveryWitness, VerifiedBatchValue},
};
use super::super::batches::{BatchKind, BatchOperation, BatchRow, RowEvent};
use super::super::recovery::RecoveryOperation;
use super::*;
pub(super) struct RetainedNativeBatch {
    pub(super) id: String,
    pub(super) kind: BatchKind,
    pub(super) values: Vec<VerifiedBatchValue>,
    pub(super) committed: bool,
}
impl Journal {
    fn batch_claim(&mut self, claim: &mut AnchoredClaim<'_>, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.poisoned || end > self.deadline {
            return Err(invalid());
        }
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.lease_state != LeaseState::Active
            || claim.epoch() != self.progress.epoch
            || claim.key() != self.progress.claim
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn batch_original_event(
        &self,
        node: &str,
        kind: BatchKind,
        completion: bool,
        native: Option<(String, SnapshotData)>,
    ) -> io::Result<Event> {
        let (operation, from, to) = match (kind, completion) {
            (BatchKind::OriginalMove, false) => (
                Operation::NodeMoveIntent,
                "OriginalAtClient",
                "OriginalAtCustody",
            ),
            (BatchKind::OriginalMove, true) => (
                Operation::NodeMovedVerified,
                "OriginalAtClient",
                "OriginalAtCustody",
            ),
            (BatchKind::WorkCreate, false) => (
                Operation::WorkCreateIntent,
                "OriginalAtCustody",
                "OriginalWithWorkAtClient",
            ),
            (BatchKind::WorkCreate, true) => (
                Operation::WorkCreatedVerified,
                "OriginalAtCustody",
                "OriginalWithWorkAtClient",
            ),
            (BatchKind::AnchorSnapshot, _) => (
                Operation::SnapshotVerified,
                "AnchorUnchanged",
                "AnchorUnchanged",
            ),
            _ => return Err(invalid()),
        };
        let expected_before =
            hash::sha256_hex(&encoded(&(self.progress.plan.as_str(), node, from))?);
        let expected_after = hash::sha256_hex(&encoded(&(self.progress.plan.as_str(), node, to))?);
        self.event(node, operation, native, expected_before, expected_after)
    }
    fn expected_original_identity(&self, node: &str) -> io::Result<(String, SnapshotData)> {
        let n = self
            .manifest
            .storage_plan
            .roots
            .iter()
            .flat_map(|r| &r.nodes)
            .find(|n| n.node_id == node)
            .ok_or_else(invalid)?;
        Ok((
            n.file_id.clone(),
            serde_json::from_value(serde_json::to_value(&n.security).map_err(|_| invalid())?)
                .map_err(|_| invalid())?,
        ))
    }
    fn retain_batch(&mut self, id: String, kind: BatchKind) -> io::Result<usize> {
        if self.native_batches.len() >= 4096 {
            return Err(invalid());
        }
        let index = self.native_batches.len();
        self.native_batches.push(RetainedNativeBatch {
            id,
            kind,
            values: Vec::new(),
            committed: false,
        });
        Ok(index)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_initial_batches(
        &mut self,
        mut witness: InitialBatchWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.batch_claim(claim, end)?;
            let expected = self
                .manifest
                .storage_plan
                .roots
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| invalid())?;
            if witness.plans(claim, end)?.as_slice() != expected.as_slice() {
                return Err(invalid());
            }
            let values = witness.anchors(claim, end)?;
            let record_anchors = self
                .progress
                .initial_anchor_rows_needed(&self.manifest.storage_plan)?;
            let expected_nodes = self
                .progress
                .batch_expected_nodes(&self.manifest.storage_plan, BatchKind::AnchorSnapshot)?;
            if values.len() != expected_nodes.len() {
                return Err(invalid());
            }
            let mut rows = Vec::new();
            for (ordinal, (value, (root, node))) in values.iter().zip(expected_nodes).enumerate() {
                if value.node() != node || value.root() != root {
                    return Err(invalid());
                }
                rows.push(BatchRow {
                    ordinal,
                    event: RowEvent::Original(self.batch_original_event(
                        &node,
                        BatchKind::AnchorSnapshot,
                        true,
                        Some(value.parts()?),
                    )?),
                });
            }
            if record_anchors {
                let event = self.progress.batch_event(
                    BatchKind::AnchorSnapshot,
                    BatchOperation::AnchorsVerified,
                    rows,
                )?;
                let index = self.retain_batch(event.batch_id.clone(), BatchKind::AnchorSnapshot)?;
                self.native_batches[index].values = values;
                self.append_entry_inner(ChainEntry::Batch(event), end)?;
                witness.check(claim, end)?;
                self.native_batches[index].committed = true;
            } else {
                // Same live new session may already have all exact individual
                // anchor observations. Reverify actual objects; do not convert,
                // adopt or resume any prior directory/session records.
                self.check(end)?;
                witness.check(claim, end)?;
            }
            for kind in [BatchKind::OriginalMove, BatchKind::WorkCreate] {
                let expected_nodes = self
                    .progress
                    .batch_expected_nodes(&self.manifest.storage_plan, kind)?;
                if expected_nodes.is_empty() {
                    continue;
                }
                witness.check(claim, end)?;
                let mut rows = Vec::new();
                for (ordinal, (_, node)) in expected_nodes.iter().enumerate() {
                    let prior = if kind == BatchKind::WorkCreate {
                        None
                    } else {
                        Some(self.expected_original_identity(node)?)
                    };
                    rows.push(BatchRow {
                        ordinal,
                        event: RowEvent::Original(
                            self.batch_original_event(node, kind, false, prior)?,
                        ),
                    });
                }
                let event = self
                    .progress
                    .batch_event(kind, BatchOperation::Intent, rows)?;
                let index = self.retain_batch(event.batch_id.clone(), kind)?;
                self.append_entry_inner(ChainEntry::Batch(event), end)?;
                self.batch_claim(claim, end)?;
                #[cfg(test)]
                let fault_base = if kind == BatchKind::OriginalMove {
                    100
                } else {
                    140
                };
                #[cfg(test)]
                if self.fault == fault_base {
                    return Err(io::Error::other(
                        "injected after full initial batch intent/current before first native node",
                    ));
                }
                for (ordinal, (root, node)) in expected_nodes.iter().enumerate() {
                    self.batch_claim(claim, end)?;
                    let value = witness.execute(node, kind, claim, end)?;
                    self.native_batches[index].values.push(value);
                    let actual = self.native_batches[index]
                        .values
                        .last()
                        .ok_or_else(invalid)?;
                    if actual.node() != node || actual.root() != root {
                        return Err(invalid());
                    }
                    #[cfg(test)]
                    if self.fault == fault_base + 12 && ordinal == 0 {
                        let rows = vec![BatchRow {
                            ordinal: 0,
                            event: RowEvent::Original(self.batch_original_event(
                                node,
                                kind,
                                true,
                                Some(actual.parts()?),
                            )?),
                        }];
                        let event = self.progress.batch_event(
                            kind,
                            BatchOperation::PrefixVerified,
                            rows,
                        )?;
                        self.append_entry_inner(ChainEntry::Batch(event), end)?;
                        witness.check(claim, end)?;
                        return Err(io::Error::other("injected after one actual initial node and durable prefix result/current before next native node"));
                    }
                    #[cfg(test)]
                    if self.fault == fault_base + 1 + ordinal as u8 {
                        return Err(io::Error::other(
                            "injected after exact initial batch native prefix before result phase",
                        ));
                    }
                }
                self.batch_claim(claim, end)?;
                witness.check(claim, end)?;
                let mut rows = Vec::new();
                for (ordinal, value) in self.native_batches[index].values.iter().enumerate() {
                    rows.push(BatchRow {
                        ordinal,
                        event: RowEvent::Original(self.batch_original_event(
                            value.node(),
                            kind,
                            true,
                            Some(value.parts()?),
                        )?),
                    });
                }
                #[cfg(test)]
                {
                    if self.fault == fault_base + 10 {
                        self.fault = 1;
                    } else if self.fault == fault_base + 11 {
                        self.fault = 2;
                    }
                }
                let event =
                    self.progress
                        .batch_event(kind, BatchOperation::PrefixVerified, rows)?;
                self.append_entry_inner(ChainEntry::Batch(event), end)?;
                witness.check(claim, end)?;
                self.native_batches[index].committed = true;
            }
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    pub(super) fn recover_native_batch(
        &mut self,
        mut witness: RecoveryWitness<'_>,
        retire: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.batch_claim(claim, end)?;
            self.recovery_native_check(&mut witness, claim, end)?;
            let kind = if retire {
                BatchKind::CurrentRetire
            } else {
                BatchKind::OriginalRestore
            };
            let nodes = self
                .progress
                .batch_expected_nodes(&self.manifest.storage_plan, kind)?;
            if nodes.is_empty() {
                return Ok(());
            }
            let (intent, completion) = if retire {
                (
                    RecoveryOperation::WorkRetireIntent,
                    RecoveryOperation::WorkRetiredVerified,
                )
            } else {
                (
                    RecoveryOperation::OriginalRestoreIntent,
                    RecoveryOperation::OriginalRestoredVerified,
                )
            };
            let rows = nodes
                .iter()
                .enumerate()
                .map(|(ordinal, (_, node))| {
                    Ok(BatchRow {
                        ordinal,
                        event: RowEvent::Recovery(self.progress.recovery_event(
                            intent,
                            None,
                            Some(node),
                        )?),
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            let event = self
                .progress
                .batch_event(kind, BatchOperation::Intent, rows)?;
            let index = self.retain_batch(event.batch_id.clone(), kind)?;
            self.append_entry_inner(ChainEntry::Batch(event), end)?;
            self.batch_claim(claim, end)?;
            #[cfg(test)]
            if self.fault == 120 {
                return Err(io::Error::other(
                    "injected after full recovery batch intent/current before first actual node",
                ));
            }
            for (ordinal, (root, node)) in nodes.iter().enumerate() {
                self.batch_claim(claim, end)?;
                let value = witness.execute_batch_node(node, retire, claim, end)?;
                self.native_batches[index].values.push(value);
                let actual = self.native_batches[index]
                    .values
                    .last()
                    .ok_or_else(invalid)?;
                if actual.node() != node || actual.root() != root {
                    return Err(invalid());
                }
                self.progress
                    .verify_batch_native_value(node, retire, actual)?;
                #[cfg(test)]
                if self.fault == 132 && ordinal == 0 {
                    let rows = vec![BatchRow {
                        ordinal: 0,
                        event: RowEvent::Recovery(self.progress.recovery_event(
                            completion,
                            None,
                            Some(node),
                        )?),
                    }];
                    let event =
                        self.progress
                            .batch_event(kind, BatchOperation::PrefixVerified, rows)?;
                    self.append_entry_inner(ChainEntry::Batch(event), end)?;
                    witness.check(claim, end)?;
                    return Err(io::Error::other("injected after one exact recovery node and durable prefix/current before next native node"));
                }
                #[cfg(test)]
                if self.fault == 121 + ordinal as u8 {
                    return Err(io::Error::other(
                        "injected after exact actual recovery batch prefix before result phase",
                    ));
                }
            }
            self.batch_claim(claim, end)?;
            witness.check(claim, end)?;
            let mut rows = Vec::new();
            for (ordinal, value) in self.native_batches[index].values.iter().enumerate() {
                self.progress
                    .verify_batch_native_value(value.node(), retire, value)?;
                rows.push(BatchRow {
                    ordinal,
                    event: RowEvent::Recovery(self.progress.recovery_event(
                        completion,
                        None,
                        Some(value.node()),
                    )?),
                });
            }
            #[cfg(test)]
            {
                if self.fault == 130 {
                    self.fault = 1;
                } else if self.fault == 131 {
                    self.fault = 2;
                }
            }
            let event = self
                .progress
                .batch_event(kind, BatchOperation::PrefixVerified, rows)?;
            self.append_entry_inner(ChainEntry::Batch(event), end)?;
            witness.check(claim, end)?;
            self.native_batches[index].committed = true;
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    pub(super) fn publish_released_anchor_batch(
        &mut self,
        witness: &mut RecoveryWitness<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        let values = witness.released_anchors(end)?;
        let expected = self
            .progress
            .batch_expected_nodes(&self.manifest.storage_plan, BatchKind::AnchorRelease)?;
        if values.len() != expected.len() {
            return Err(invalid());
        }
        let mut rows = Vec::new();
        for (ordinal, (value, (root, node))) in values.iter().zip(expected).enumerate() {
            if value.node() != node
                || value.root() != root
                || value.parts()? != self.expected_original_identity(&node)?
            {
                return Err(invalid());
            }
            rows.push(BatchRow {
                ordinal,
                event: RowEvent::Recovery(self.progress.recovery_event(
                    RecoveryOperation::ReleaseVerified,
                    None,
                    Some(&node),
                )?),
            });
        }
        let event = self.progress.batch_event(
            BatchKind::AnchorRelease,
            BatchOperation::AnchorsVerified,
            rows,
        )?;
        let index = self.retain_batch(event.batch_id.clone(), BatchKind::AnchorRelease)?;
        self.native_batches[index].values = values;
        #[cfg(test)]
        {
            if self.fault == 59 {
                self.fault = 1;
            } else if self.fault == 60 {
                self.fault = 2;
            }
        }
        self.append_entry_inner(ChainEntry::Batch(event), end)?;
        witness.post_release(end)?;
        self.native_batches[index].committed = true;
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_recover_batch_for_test(
        &mut self,
        witness: RecoveryWitness<'_>,
        retire: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        self.recover_native_batch(witness, retire, claim, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_batch_trace_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        let mut events = Vec::new();
        for r in &self.events {
            if let ChainEntry::Batch(e) =
                serde_json::from_slice::<ChainEntry>(&r.raw).map_err(|_| invalid())?
            {
                events.push(serde_json::to_value(e).map_err(|_| invalid())?);
            }
        }
        Ok(
            serde_json::json!({"sequence":self.progress.sequence,"pending":self.progress.batch_summary(),"poisoned":self.poisoned,"directory":self.directory.path(),
            "immutableBatchPhases":events,"retainedNativeBatches":self.native_batches.iter().map(|b|serde_json::json!({"batchId":b.id,"kind":b.kind,
                "committed":b.committed,"actualVerifiedPrefix":b.values.len(),"nativeValues":b.values.iter().map(|v|v.data()).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "physicalEventFiles":self.events.len(),"runtimeAuthority":false}),
        )
    }
}
