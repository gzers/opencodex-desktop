//! Same-chain route handle close DATA; no original Files/actual Stop factory.
use super::super::handle_release::HandleRow;
use super::*;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RouteCloseData {
    home: std::path::PathBuf,
    config_security: SnapshotData,
    config_bytes: usize,
    config_sha256: String,
    route_digest: String,
    inventory_digest: String,
    handle_count: usize,
    rows: Vec<HandleRow>,
}
impl RouteCloseData {
    pub(super) fn digest(&self) -> io::Result<String> {
        Ok(hash::sha256_hex(&encoded(self)?))
    }
    pub(super) fn validate(&self) -> io::Result<()> {
        validate_local_path(&self.home)?;
        self.config_security.validate()?;
        let depth = self.home.ancestors().count();
        if self.config_bytes == 0
            || self.config_bytes > 2 * 1024 * 1024
            || !digest(&self.config_sha256)
            || !digest(&self.route_digest)
            || self.handle_count != self.rows.len()
            || self.handle_count != 2 + 2 * depth
            || self.handle_count > 4096
            || self.inventory_digest
                != hash::sha256_hex(&encoded(
                    &serde_json::to_value(&self.rows).map_err(|_| invalid())?,
                )?)
        {
            return Err(invalid());
        }
        let first = self.rows.first().ok_or_else(invalid)?;
        let last = self.rows.last().ok_or_else(invalid)?;
        if first.key != "route/security/file"
            || last.key != "route/config"
            || first.directory
            || first.historical_work
            || last.directory
            || last.historical_work
            || first.volume != last.volume
            || first.file_id != last.file_id
            || !self
                .config_security
                .matches_id(first.volume, &first.file_id)
        {
            return Err(invalid());
        }
        for (i, row) in self.rows[1..self.rows.len() - 1].iter().enumerate() {
            let key = if i < depth {
                format!("route/security/parent/{i}")
            } else {
                format!("route/home/{}", i - depth)
            };
            if row.key != key || !row.directory || row.historical_work || !decimal(&row.file_id) {
                return Err(invalid());
            }
            if i >= depth {
                let other = &self.rows[1 + i - depth];
                if row.volume != other.volume || row.file_id != other.file_id {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum RouteCloseOperation {
    RouteHandlesCloseIntent,
    RouteHandlesClosedVerified,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RouteCloseSummary {
    pub(super) digest: String,
    pub(super) verified: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RouteCloseEvent {
    schema: String,
    document_kind: String,
    storage_strategy: String,
    event_id: String,
    sequence: u32,
    session_id: String,
    operation: RouteCloseOperation,
    route: RouteCloseData,
    closure_digest: String,
    manifest_digest: String,
    binding_digest: String,
    storage_plan_digest: String,
    security_policy_digest: String,
    deadline_scope: String,
    lease_epoch: String,
    claim_key: String,
    previous_event_digest: String,
}
impl Progress {
    pub(super) fn route_close_event(
        &self,
        operation: RouteCloseOperation,
        route: RouteCloseData,
    ) -> io::Result<RouteCloseEvent> {
        Ok(RouteCloseEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "route-handle-close-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            operation,
            route,
            closure_digest: self.closed.as_ref().ok_or_else(invalid)?.digest.clone(),
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
    pub(super) fn apply_route_close(
        &mut self,
        plan: &StoragePlan,
        e: &RouteCloseEvent,
    ) -> io::Result<()> {
        let closed = self.closed.as_ref().ok_or_else(invalid)?;
        if closed.verified || closed.digest!=e.closure_digest || self.batch.is_some() || self.pending.is_some() || self.snapshot_pending.is_some() || self.action_pending.is_some()
            || !self.complete_files(plan) || !self.history_linked() || !self.handle_progress.as_ref().is_some_and(|h|h.released)
            || !self.content.as_ref().is_some_and(|c|c.verified) || self.lifecycle!=snapshots::LifecyclePhase::Stopped || self.lease_state!=segments::LeaseState::Released
            || e.schema!=crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3 || e.document_kind!="route-handle-close-event" || e.storage_strategy!=ANCHORED_STORAGE_STRATEGY
            || !uuid(&e.event_id) || self.event_ids.contains(&e.event_id) || e.sequence!=self.sequence.checked_add(1).ok_or_else(invalid)? || e.session_id!=self.session
            || e.previous_event_digest!=self.tip || e.manifest_digest!=self.manifest || e.binding_digest!=self.binding || e.storage_plan_digest!=self.plan || plan.digest()?!=self.plan
            || e.security_policy_digest!=Policy::fixed().digest()? || e.deadline_scope!=self.deadline || e.lease_epoch!=self.epoch || e.claim_key!=self.claim {return Err(invalid());}
        e.route.validate()?;
        let value = e.route.digest()?;
        if value != closed.route_close_digest {
            return Err(invalid());
        }
        match e.operation {
            RouteCloseOperation::RouteHandlesCloseIntent if self.route_close.is_none() => {}
            RouteCloseOperation::RouteHandlesClosedVerified
                if self.route_close.as_ref()
                    == Some(&RouteCloseSummary {
                        digest: value.clone(),
                        verified: false,
                    }) => {}
            _ => return Err(invalid()),
        }
        let tip = hash::sha256_hex(&encoded(e)?);
        self.route_close = Some(RouteCloseSummary {
            digest: value,
            verified: e.operation == RouteCloseOperation::RouteHandlesClosedVerified,
        });
        self.tip = tip;
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
}
#[cfg(test)]
#[path = "route_close_data_tests.rs"]
pub(super) mod tests;
