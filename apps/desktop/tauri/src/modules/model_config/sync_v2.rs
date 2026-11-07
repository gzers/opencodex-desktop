//! Immutable configuration sync core. L0 registers this module and supplies the
//! real network/codec/local-transaction adapters; the existing pipeline is untouched.
#[path = "sync_v2_graph.rs"]
pub mod graph;
#[path = "sync_v2_store.rs"]
pub mod store;
#[cfg(test)]
#[path = "sync_v2_tests.rs"]
mod tests;
#[path = "sync_v2_types.rs"]
pub mod types;

use graph::{Graph, Network};
use std::{sync::Arc, time::Duration};
use store::{canonical, digest, execution_digest, plan_digest, ReceiveRecord, Store};
use tokio::sync::Semaphore;
use types::*;

// Windows CI timing evidence for remaining timeout failures. Only static phase
// names and elapsed milliseconds are emitted; no paths, payloads or credentials.
#[cfg(all(test, windows))]
pub(super) struct TestPhaseTiming {
    phase: &'static str,
    started: std::time::Instant,
}
#[cfg(all(test, windows))]
impl TestPhaseTiming {
    pub(super) fn start(phase: &'static str) -> Self {
        eprintln!("SYNC_TEST_PHASE begin {phase}");
        Self {
            phase,
            started: std::time::Instant::now(),
        }
    }
}
#[cfg(all(test, windows))]
impl Drop for TestPhaseTiming {
    fn drop(&mut self) {
        eprintln!(
            "SYNC_TEST_PHASE end {} elapsed_ms={}",
            self.phase,
            self.started.elapsed().as_millis()
        );
    }
}

pub struct SyncContext<'a> {
    pub store: &'a Store,
    pub transport: &'a dyn ConfigSyncTransport,
    pub codec: &'a dyn PayloadCodec,
    pub local: &'a dyn LocalApplication,
    pub connection: ConnectionHandle,
    pub budgets: Budgets,
    /// Share this Arc across contexts using the same production adapter.
    pub network_slots: Arc<Semaphore>,
}
fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn unknown_relation() -> Relation {
    Relation {
        local: LocalRelation::NotCompared,
        remote: RemoteRelation::Unknown,
    }
}
impl<'a> SyncContext<'a> {
    pub fn new(
        store: &'a Store,
        transport: &'a dyn ConfigSyncTransport,
        codec: &'a dyn PayloadCodec,
        local: &'a dyn LocalApplication,
        connection: ConnectionHandle,
    ) -> SyncResult<Self> {
        require_digest(&connection.binding_id)?;
        require_digest(&connection.revision)?;
        Ok(Self {
            store,
            transport,
            codec,
            local,
            connection,
            budgets: Budgets::default(),
            network_slots: Arc::new(Semaphore::new(4)),
        })
    }
    /// One wall-clock budget for the complete command. In particular, multiple
    /// observe/readback stages cannot each reset the ninety-second contract.
    async fn bounded<T>(
        &self,
        id: Option<&str>,
        future: impl std::future::Future<Output = SyncResult<T>>,
    ) -> SyncResult<T> {
        self.budgets.validate()?;
        match tokio::time::timeout(Duration::from_millis(self.budgets.observation_ms), future).await
        {
            Ok(value) => value.map_err(|mut error| {
                if let Some(id) = id {
                    error = error.for_operation(id);
                } else {
                    error.recovery_actions.retain(|a| {
                        !matches!(
                            a.kind,
                            RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
                        )
                    });
                }
                error
            }),
            Err(_) => {
                let mut error = failure(ErrorCode::SyncNetwork, Step::Observe);
                error.cause = Some(Cause::Timeout);
                if let Some(id) = id {
                    require_uuid(id)?;
                    if let Some(_guard) = self.store.try_lock(&format!("op-{id}"))? {
                        if let Ok(mut j) = self.store.journal(id) {
                            if j.fact.last_write_step != WriteStep::None {
                                error = failure(ErrorCode::SyncOperationUnknown, Step::Verify);
                                error.cause = Some(Cause::Timeout);
                            }
                            return self
                                .save_error(&mut j, error)
                                .map(|_| unreachable!("save_error always returns Err"));
                        }
                        if let Ok(mut record) = self.store.receive(id) {
                            if let Some(fact) = record.fact.as_mut() {
                                fact.failure = Some(error.clone());
                                fact.recovery_actions = error.recovery_actions.clone();
                            }
                            if let Some(receipt) = record.receipt.as_mut() {
                                receipt.failure = Some(error.clone());
                                receipt.recovery_actions = error.recovery_actions.clone();
                            }
                            self.store.save_receive(id, &record)?;
                        }
                    }
                }
                if id.is_none() {
                    error.recovery_actions.retain(|a| {
                        !matches!(
                            a.kind,
                            RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
                        )
                    });
                }
                Err(error)
            }
        }
    }
    fn net(&self) -> Network<'_> {
        Network {
            transport: self.transport,
            connection: &self.connection,
            limits: &self.budgets,
            slots: &self.network_slots,
        }
    }
    fn connection_id(&self, id: &str) -> SyncResult<()> {
        if id != self.connection.id {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
        }
        self.budgets.validate()
    }
    fn binding(&self, id: &str) -> SyncResult<()> {
        if id != self.connection.binding_id {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::Verify));
        }
        Ok(())
    }
    async fn observe(&self) -> SyncResult<Graph> {
        graph::observe(&self.net(), self.store).await
    }
    pub async fn observe_remote(&self, r: ObserveRequest) -> SyncResult<RemoteObservation> {
        self.connection_id(&r.connection_id)?;
        self.bounded(None, async { Ok(self.observe().await?.observation) })
            .await
    }
    fn source(&self, expected: &SourceGuard) -> SyncResult<()> {
        require_guard(expected)?;
        let current = self.local.current_source()?;
        self.validate_source(expected, &current)
    }
    fn validate_source(&self, expected: &SourceGuard, current: &SourceGuard) -> SyncResult<()> {
        require_guard(expected)?;
        require_guard(current)?;
        if current.scope_digest != expected.scope_digest
            || current.mcp_sources_digest != expected.mcp_sources_digest
        {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        if current.saved_source_digest != expected.saved_source_digest {
            return Err(failure(ErrorCode::SyncSourceChanged, Step::SourceGuard));
        }
        Ok(())
    }
    fn fresh(&self, prepared: &str, expires: &str) -> SyncResult<()> {
        let p = chrono::DateTime::parse_from_rfc3339(prepared)
            .map_err(|_| failure(ErrorCode::SyncPlanChanged, Step::SourceGuard))?;
        let e = chrono::DateTime::parse_from_rfc3339(expires)
            .map_err(|_| failure(ErrorCode::SyncPlanChanged, Step::SourceGuard))?;
        if !prepared.ends_with('Z')
            || !expires.ends_with('Z')
            || e.signed_duration_since(p).num_milliseconds() != self.budgets.preview_ms
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let n = chrono::Utc::now();
        if n < p || n > e {
            return Err(failure(ErrorCode::SyncPlanExpired, Step::SourceGuard));
        }
        Ok(())
    }
    fn check_schema(&self, s: &PayloadSchemaRef) -> SyncResult<()> {
        if !self.codec.supported_schemas().contains(s) {
            return Err(failure(
                ErrorCode::SyncPayloadSchemaUnsupported,
                Step::Decode,
            ));
        }
        Ok(())
    }
    fn check_payload(&self, b: &[u8], s: &PayloadSchemaRef) -> SyncResult<()> {
        self.check_schema(s)?;
        if b.is_empty() || b.len() > self.budgets.payload_bytes {
            return Err(failure(ErrorCode::SyncLimit, Step::Decode));
        }
        self.codec.inspect_and_validate(b, s)
    }
    async fn fetch(&self, g: &GenerationRef, h: &CommitRef, graph: &Graph) -> SyncResult<Vec<u8>> {
        let n = graph.node(g, h)?;
        self.check_schema(&n.commit.payload_schema)?;
        let b = self
            .net()
            .get_retry(
                &SyncRelativePath::data(&g.generation_id, &h.operation_id)?,
                self.budgets.payload_bytes,
                Step::PayloadGet,
            )
            .await?;
        if digest(&b) != n.commit.payload_digest || b.len() as u64 != n.commit.payload_bytes {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::PayloadGet));
        }
        self.check_payload(&b, &n.commit.payload_schema)?;
        Ok(b)
    }
    pub async fn prepare_upload(&self, r: PrepareUploadRequest) -> SyncResult<UploadPlan> {
        self.bounded(None, self.prepare(r, None)).await
    }
    pub async fn prepare_resolution(&self, r: PrepareResolutionRequest) -> SyncResult<UploadPlan> {
        self.bounded(None, self.prepare_resolution_inner(r)).await
    }
    async fn prepare_resolution_inner(
        &self,
        r: PrepareResolutionRequest,
    ) -> SyncResult<UploadPlan> {
        self.connection_id(&r.connection_id)?;
        require_digest(&r.expected_view_digest)?;
        let graph = self.observe().await?;
        graph.ordinary(true)?;
        if r.resolution.scope_digest != r.source_guard.scope_digest {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        if graph.observation.state != RemoteState::Conflict
            || graph.observation.generation.as_ref() != Some(&r.generation)
            || graph.observation.heads != r.heads
            || r.heads.len() < 2
        {
            return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
        }
        self.prepare(
            PrepareUploadRequest {
                connection_id: r.connection_id,
                expected_view_digest: r.expected_view_digest,
                source_guard: r.source_guard,
                initialize_on_confirm: false,
            },
            Some(r.resolution),
        )
        .await
    }
    async fn prepare(
        &self,
        r: PrepareUploadRequest,
        resolution: Option<Resolution>,
    ) -> SyncResult<UploadPlan> {
        self.connection_id(&r.connection_id)?;
        require_digest(&r.expected_view_digest)?;
        self.source(&r.source_guard)?;
        let graph = self.observe().await?;
        graph.ordinary(resolution.is_some())?;
        if graph.observation.view_digest != r.expected_view_digest {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        if graph.observation.state == RemoteState::Empty && !r.initialize_on_confirm {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Initialize));
        }
        if resolution.is_some() && graph.observation.state != RemoteState::Conflict {
            return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
        }
        // A Head resolution only grants the codec the explicitly selected
        // payload. All observed heads STILL become immutable commit parents.
        // Generic Merged codecs retain the full set; the production host opts
        // out of automatic merging separately, without changing this format.
        let payload_heads: Vec<_> = match resolution.as_ref() {
            Some(Resolution {
                selection_mode: SelectionMode::Head,
                selected_head: Some(head),
                ..
            }) => {
                if !graph.observation.heads.contains(head) {
                    return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
                }
                vec![head]
            }
            Some(Resolution {
                selection_mode: SelectionMode::Head,
                ..
            }) => return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard)),
            _ => graph.observation.heads.iter().collect(),
        };
        let mut inputs = Vec::new();
        let mut total_bytes = 0usize;
        if let Some(g) = &graph.observation.generation {
            for h in payload_heads {
                let bytes = self.fetch(g, h, &graph).await?;
                total_bytes = total_bytes.saturating_add(bytes.len());
                if total_bytes > self.budgets.payload_bytes {
                    return Err(failure(ErrorCode::SyncLimit, Step::Decode));
                }
                inputs.push(PayloadInput {
                    head: h.clone(),
                    bytes,
                });
            }
        }
        let frozen = self
            .codec
            .freeze_selected(&r.source_guard, &inputs, resolution.as_ref())?;
        self.check_payload(&frozen.bytes, &frozen.schema)?;
        self.source(&r.source_guard)?;
        let id = new_id();
        let handle = new_id();
        let prepared_at = now();
        // Derive expiry from the frozen timestamp, rather than two clock reads.
        let expires_at = self.expiry(&prepared_at)?;
        let mut plan = UploadPlan {
            dto_version: 1,
            operation_id: id.clone(),
            plan_digest: String::new(),
            binding_id: self.connection.binding_id.clone(),
            initialization: if graph.observation.state == RemoteState::Empty {
                Initialization::OnConfirm
            } else {
                Initialization::None
            },
            generation: graph.observation.generation.clone(),
            parents: graph.observation.heads.clone(),
            intent: if resolution.is_some() {
                Intent::Resolve
            } else {
                Intent::Update
            },
            resolution,
            payload_digest: digest(&frozen.bytes),
            payload_bytes: frozen.bytes.len() as u64,
            payload_schema: frozen.schema,
            protection_mode: frozen.protection_mode,
            source_guard: r.source_guard,
            expected_view_digest: r.expected_view_digest,
            prepared_at,
            expires_at,
        };
        if plan.intent == Intent::Resolve {
            let c = Commit {
                format: "OCXD_CONFIG_COMMIT".into(),
                format_version: 1,
                generation_id: plan.generation.as_ref().unwrap().generation_id.clone(),
                operation_id: id.clone(),
                parents: plan.parents.clone(),
                payload_digest: plan.payload_digest.clone(),
                payload_bytes: plan.payload_bytes,
                payload_schema: plan.payload_schema.clone(),
                intent: plan.intent,
                resolution: plan.resolution.clone(),
            };
            graph::validate_commit(&c, &c.generation_id, &id, &self.budgets)?;
        }
        plan.plan_digest = plan_digest(&plan)?;
        let fact = OperationFact {
            dto_version: 1,
            operation_id: id.clone(),
            kind: OperationKind::Upload,
            phase: Phase::Prepared,
            last_write_step: WriteStep::None,
            storage_status: StorageStatus::NotAttempted,
            local_apply_status: LocalApplyStatus::NotApplicable,
            relation: unknown_relation(),
            generation: plan.generation.clone(),
            payload_digest: plan.payload_digest.clone(),
            commit: None,
            stored_at: None,
            observation: Some(graph.observation.clone()),
            failure: None,
            recovery_actions: vec![],
        };
        let generation_bytes_utf8 = plan
            .generation
            .as_ref()
            .and_then(|g| graph.generation_bytes.get(&g.generation_id))
            .map(|b| String::from_utf8(b.clone()).unwrap());
        let journal = Journal {
            journal_version: 1,
            operation_id: id,
            plan: plan.clone(),
            fact,
            root_revision: self.local.root_revision()?,
            connection_revision: self.connection.revision.clone(),
            payload_handle: handle.clone(),
            generation_bytes_utf8,
            commit_bytes_utf8: None,
            execution_digest: None,
            previous_failures: vec![],
        };
        self.store.freeze(&handle, &frozen.bytes)?;
        self.store.save_journal(&journal)?;
        Ok(plan)
    }
    fn context_guard(&self, j: &Journal) -> SyncResult<()> {
        self.binding(&j.plan.binding_id)?;
        if j.connection_revision != self.connection.revision {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
        }
        #[cfg(all(test, windows))]
        let _timing = TestPhaseTiming::start("fresh-local-source-context");
        let (revision, current) = self.local.source_context()?;
        require_digest(&revision)?;
        if j.root_revision != revision {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        self.validate_source(&j.plan.source_guard, &current)
    }
    fn save_error(&self, j: &mut Journal, mut e: SyncFailure) -> SyncResult<OperationFact> {
        e = e.for_operation(&j.operation_id);
        j.fact.failure = Some(e.clone());
        j.fact.recovery_actions = e.recovery_actions.clone();
        if j.fact.storage_status != StorageStatus::Stored
            && matches!(
                e.code,
                ErrorCode::SyncOperationCollision | ErrorCode::SyncRemoteInvalid
            )
        {
            j.fact.storage_status = StorageStatus::Invalid;
        }
        j.previous_failures.push(e.clone());
        if j.previous_failures.len() > 64 {
            j.previous_failures.remove(1);
        }
        self.store.save_journal(j)?;
        Err(e)
    }
    pub async fn publish_upload(&self, r: ConfirmUploadRequest) -> SyncResult<OperationFact> {
        let id = r.operation_id.clone();
        self.bounded(Some(&id), self.publish_upload_inner(r)).await
    }
    async fn publish_upload_inner(&self, r: ConfirmUploadRequest) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        let _binding = self
            .store
            .lock(&format!("publish-{}", &self.connection.binding_id[7..]))
            .await?;
        let _op = self.store.lock(&format!("op-{}", r.operation_id)).await?;
        let mut j = self.store.journal(&r.operation_id)?;
        self.binding(&j.plan.binding_id)?;
        if j.plan.plan_digest != r.plan_digest {
            return self.save_error(
                &mut j,
                failure(ErrorCode::SyncPlanChanged, Step::SourceGuard),
            );
        }
        // A repeated confirmation after any possible send is ALWAYS read-only.
        if j.fact.last_write_step != WriteStep::None {
            return self.verify_locked(&mut j).await;
        }
        let result = self.run_upload(&mut j, None).await;
        match result {
            Ok(v) => Ok(v),
            Err(e) => self.save_error(&mut j, e),
        }
    }
    fn cancelled(&self, j: &mut Journal) -> SyncResult<()> {
        if self.store.is_cancelled(&j.operation_id)? {
            if j.fact.last_write_step == WriteStep::None {
                j.fact.phase = Phase::CancelledBeforeWrite;
                self.store.save_journal(j)?;
                return Err(failure(ErrorCode::SyncCancelled, Step::SourceGuard));
            }
            return Err(failure(ErrorCode::SyncOperationUnknown, Step::Verify));
        }
        Ok(())
    }
    fn prepare_metadata(&self, j: &mut Journal) -> SyncResult<()> {
        if j.fact.generation.is_none() {
            let g = Generation {
                format: "OCXD_CONFIG_GENERATION".into(),
                format_version: 1,
                generation_id: new_id(),
            };
            let bytes = canonical(&g)?;
            j.fact.generation = Some(GenerationRef {
                generation_id: g.generation_id,
                digest: digest(&bytes),
            });
            j.generation_bytes_utf8 = Some(String::from_utf8(bytes).unwrap());
        }
        if j.commit_bytes_utf8.is_none() {
            let c = Commit {
                format: "OCXD_CONFIG_COMMIT".into(),
                format_version: 1,
                generation_id: j.fact.generation.as_ref().unwrap().generation_id.clone(),
                operation_id: j.operation_id.clone(),
                parents: j.plan.parents.clone(),
                payload_digest: j.plan.payload_digest.clone(),
                payload_bytes: j.plan.payload_bytes,
                payload_schema: j.plan.payload_schema.clone(),
                intent: j.plan.intent,
                resolution: j.plan.resolution.clone(),
            };
            graph::validate_commit(&c, &c.generation_id, &c.operation_id, &self.budgets)?;
            let bytes = canonical(&c)?;
            j.fact.commit = Some(CommitRef {
                operation_id: j.operation_id.clone(),
                digest: digest(&bytes),
            });
            j.commit_bytes_utf8 = Some(String::from_utf8(bytes).unwrap());
            j.execution_digest = Some(execution_digest(j, &c)?);
        }
        self.store.save_journal(j)
    }
    async fn guard(&self, j: &mut Journal, require_parents: bool) -> SyncResult<Graph> {
        self.cancelled(j)?;
        self.context_guard(j)?;
        let graph = self.observe().await?;
        graph.ordinary(j.plan.intent == Intent::Resolve)?;
        if graph.observation.generation.as_ref() != j.fact.generation.as_ref() {
            return Err(failure(ErrorCode::SyncGenerationChanged, Step::SourceGuard));
        }
        if require_parents && graph.observation.heads != j.plan.parents {
            return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
        }
        Ok(graph)
    }
    async fn run_upload(
        &self,
        j: &mut Journal,
        resume_view: Option<&str>,
    ) -> SyncResult<OperationFact> {
        self.cancelled(j)?;
        self.context_guard(j)?;
        if resume_view.is_none() {
            self.fresh(&j.plan.prepared_at, &j.plan.expires_at)?;
        }
        let graph = self.observe().await?;
        graph.ordinary(j.plan.intent == Intent::Resolve)?;
        let expected = resume_view
            .unwrap_or(&j.plan.expected_view_digest)
            .to_owned();
        if graph.observation.view_digest != expected {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let needs_init = graph.observation.state == RemoteState::Empty;
        if needs_init {
            if j.plan.initialization != Initialization::OnConfirm
                || !matches!(
                    j.fact.phase,
                    Phase::Prepared | Phase::InitializationMayBeWritten
                )
            {
                return Err(failure(ErrorCode::SyncGenerationChanged, Step::Initialize));
            }
        } else {
            let expected_g = j.fact.generation.as_ref().or(j.plan.generation.as_ref());
            if graph.observation.generation.as_ref() != expected_g {
                return Err(failure(ErrorCode::SyncGenerationChanged, Step::Initialize));
            }
            if graph.observation.heads != j.plan.parents {
                return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
            }
        }
        let bytes = self
            .store
            .payload(&j.payload_handle, &j.plan.payload_digest)?;
        if bytes.len() as u64 != j.plan.payload_bytes {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        self.prepare_metadata(j)?;
        let g = j.fact.generation.as_ref().unwrap().clone();
        let id = j.operation_id.clone();
        if needs_init {
            self.cancelled(j)?;
            self.context_guard(j)?;
            let current = self.observe().await?;
            current.ordinary(false)?;
            if current.observation.state != RemoteState::Empty
                || current.observation.view_digest != expected
            {
                return Err(failure(ErrorCode::SyncGenerationChanged, Step::Initialize));
            }
            j.fact.phase = Phase::InitializationMayBeWritten;
            j.fact.last_write_step = std::cmp::max(j.fact.last_write_step, WriteStep::Generation);
            self.store.save_journal(j)?;
            let gb = j.generation_bytes_utf8.as_ref().unwrap().as_bytes();
            self.net()
                .put(
                    &SyncRelativePath::generation(&g.generation_id)?,
                    gb,
                    Step::Initialize,
                )
                .await?;
            let got = self
                .net()
                .get_retry(
                    &SyncRelativePath::generation(&g.generation_id)?,
                    self.budgets.metadata_bytes,
                    Step::Initialize,
                )
                .await?;
            if got != gb {
                return Err(failure(ErrorCode::SyncOperationCollision, Step::Initialize));
            }
        }
        self.guard(j, true).await?;
        j.fact.phase = Phase::GenerationVerified;
        self.store.save_journal(j)?;
        // Never overwrite a pre-existing ID with other bytes. The absence check
        // is not CAS; globally unique IDs + frozen bytes are the writer contract.
        match self
            .net()
            .get(
                &SyncRelativePath::data(&g.generation_id, &id)?,
                self.budgets.payload_bytes,
                Step::PayloadGet,
            )
            .await
        {
            Ok(old) if old != bytes => {
                return Err(failure(ErrorCode::SyncOperationCollision, Step::PayloadGet))
            }
            Ok(_) => (),
            Err(e) if e.code == ErrorCode::SyncListingIncomplete => {
                self.guard(j, true).await?;
                j.fact.phase = Phase::PayloadMayBeWritten;
                j.fact.last_write_step = std::cmp::max(j.fact.last_write_step, WriteStep::Payload);
                j.fact.storage_status = StorageStatus::Unknown;
                self.store.save_journal(j)?;
                self.net()
                    .put(
                        &SyncRelativePath::data(&g.generation_id, &id)?,
                        &bytes,
                        Step::PayloadPut,
                    )
                    .await?;
            }
            Err(e) => return Err(e),
        }
        let got = self
            .net()
            .get_retry(
                &SyncRelativePath::data(&g.generation_id, &id)?,
                self.budgets.payload_bytes,
                Step::PayloadGet,
            )
            .await?;
        if digest(&got) != j.plan.payload_digest || got.len() as u64 != j.plan.payload_bytes {
            return Err(failure(ErrorCode::SyncOperationCollision, Step::PayloadGet));
        }
        j.fact.phase = Phase::PayloadVerified;
        self.store.save_journal(j)?;
        self.guard(j, true).await?;
        let commit_bytes = j.commit_bytes_utf8.clone().unwrap().into_bytes();
        match self
            .net()
            .get(
                &SyncRelativePath::commit(&g.generation_id, &id)?,
                self.budgets.metadata_bytes,
                Step::CommitGet,
            )
            .await
        {
            Ok(old) if old != commit_bytes => {
                return Err(failure(ErrorCode::SyncOperationCollision, Step::CommitGet))
            }
            Ok(_) => (),
            Err(e) if e.code == ErrorCode::SyncListingIncomplete => {
                self.guard(j, true).await?;
                j.fact.phase = Phase::CommitMayBeWritten;
                j.fact.last_write_step = std::cmp::max(j.fact.last_write_step, WriteStep::Commit);
                j.fact.storage_status = StorageStatus::Unknown;
                self.store.save_journal(j)?;
                self.net()
                    .put(
                        &SyncRelativePath::commit(&g.generation_id, &id)?,
                        &commit_bytes,
                        Step::CommitPut,
                    )
                    .await?;
            }
            Err(e) => return Err(e),
        }
        self.verify_locked(j).await
    }
    async fn verify_inner(&self, j: &mut Journal) -> SyncResult<OperationFact> {
        #[cfg(all(test, windows))]
        let _timing = TestPhaseTiming::start("verify-original-upload");
        self.binding(&j.plan.binding_id)?;
        j.fact.relation.local = LocalRelation::NotCompared;
        let graph = self.observe().await;
        let mut observation_error = None;
        match &graph {
            Ok(g) => {
                if let Some(original) = &j.fact.generation {
                    j.fact.relation.remote = g.remote_relation(original, &j.operation_id);
                    if !g.observation.generations.contains(original) {
                        return Err(failure(ErrorCode::SyncGenerationChanged, Step::Verify));
                    }
                }
                j.fact.observation = Some(g.observation.clone());
                if matches!(
                    g.observation.state,
                    RemoteState::Invalid | RemoteState::Incomplete
                ) {
                    observation_error = g.observation.problems.first().cloned();
                }
            }
            Err(e) => {
                j.fact.relation.remote = RemoteRelation::Unknown;
                observation_error = Some(e.clone());
            }
        }
        let g = j
            .fact
            .generation
            .clone()
            .ok_or_else(|| failure(ErrorCode::SyncOperationUnknown, Step::Verify))?;
        let expected_g = j
            .generation_bytes_utf8
            .as_ref()
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        let got = self
            .net()
            .get_retry(
                &SyncRelativePath::generation(&g.generation_id)?,
                self.budgets.metadata_bytes,
                Step::Verify,
            )
            .await?;
        if got != expected_g.as_bytes() {
            return Err(failure(ErrorCode::SyncGenerationChanged, Step::Verify));
        }
        let expected = j
            .commit_bytes_utf8
            .as_ref()
            .ok_or_else(|| failure(ErrorCode::SyncOperationUnknown, Step::Verify))?;
        let got = self
            .net()
            .get_retry(
                &SyncRelativePath::commit(&g.generation_id, &j.operation_id)?,
                self.budgets.metadata_bytes,
                Step::CommitGet,
            )
            .await?;
        if got != expected.as_bytes() {
            return Err(failure(ErrorCode::SyncOperationCollision, Step::CommitGet));
        }
        let bytes = self
            .net()
            .get_retry(
                &SyncRelativePath::data(&g.generation_id, &j.operation_id)?,
                self.budgets.payload_bytes,
                Step::PayloadGet,
            )
            .await?;
        if digest(&bytes) != j.plan.payload_digest || bytes.len() as u64 != j.plan.payload_bytes {
            return Err(failure(ErrorCode::SyncOperationCollision, Step::PayloadGet));
        }
        j.fact.storage_status = StorageStatus::Stored;
        j.fact.phase = Phase::Settled;
        if j.fact.stored_at.is_none() {
            j.fact.stored_at = Some(now());
        }
        j.fact.failure = observation_error;
        j.fact.recovery_actions = j
            .fact
            .failure
            .as_ref()
            .map(|e| e.clone().for_operation(&j.operation_id).recovery_actions)
            .unwrap_or_default();
        if j.fact.relation.remote == RemoteRelation::Conflict {
            j.fact
                .recovery_actions
                .push(action(RecoveryKind::ChooseConflictSource));
        }
        self.store.save_journal(j)?;
        Ok(j.fact.clone())
    }
    async fn verify_locked(&self, j: &mut Journal) -> SyncResult<OperationFact> {
        let result = tokio::time::timeout(
            Duration::from_millis(self.budgets.observation_ms),
            self.verify_inner(j),
        )
        .await;
        match result {
            Ok(Ok(f)) => Ok(f),
            r => {
                let mut e = match r {
                    Ok(Err(e)) => e,
                    _ => failure(ErrorCode::SyncOperationUnknown, Step::Verify),
                };
                if e.code == ErrorCode::SyncListingIncomplete {
                    e.code = ErrorCode::SyncOperationUnknown;
                    e.retryability = Retryability::SameOperationReadOnly;
                    e.recovery_actions = vec![
                        action(RecoveryKind::VerifyOriginalUpload),
                        action(RecoveryKind::RetryLater),
                    ];
                }
                self.save_error(j, e)
            }
        }
    }
    pub async fn verify_original_upload(&self, r: OperationRequest) -> SyncResult<OperationFact> {
        let id = r.operation_id.clone();
        self.bounded(Some(&id), self.verify_original_upload_inner(r))
            .await
    }
    async fn verify_original_upload_inner(&self, r: OperationRequest) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        let _op = self.store.lock(&format!("op-{}", r.operation_id)).await?;
        let mut j = self.store.journal(&r.operation_id)?;
        self.verify_locked(&mut j).await
    }
    pub async fn resume_original_upload(
        &self,
        r: ResumeUploadRequest,
    ) -> SyncResult<OperationFact> {
        let id = r.operation_id.clone();
        self.bounded(Some(&id), self.resume_original_upload_inner(r))
            .await
    }
    async fn resume_original_upload_inner(
        &self,
        r: ResumeUploadRequest,
    ) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        let _binding = self
            .store
            .lock(&format!("publish-{}", &self.connection.binding_id[7..]))
            .await?;
        let _op = self.store.lock(&format!("op-{}", r.operation_id)).await?;
        let mut j = self.store.journal(&r.operation_id)?;
        self.binding(&j.plan.binding_id)?;
        if j.plan.plan_digest != r.plan_digest {
            return self.save_error(
                &mut j,
                failure(ErrorCode::SyncPlanChanged, Step::SourceGuard),
            );
        }
        let verified = self.verify_locked(&mut j).await;
        if j.fact.storage_status == StorageStatus::Stored {
            return verified;
        }
        if let Err(e) = verified {
            if !matches!(e.code, ErrorCode::SyncOperationUnknown) {
                return Err(e);
            }
        }
        self.store.clear_cancelled(&r.operation_id)?;
        let result = self.run_upload(&mut j, Some(&r.expected_view_digest)).await;
        match result {
            Ok(v) => Ok(v),
            Err(e) => self.save_error(&mut j, e),
        }
    }
    pub fn cancel_upload(&self, r: OperationRequest) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        self.store.set_cancelled(&r.operation_id)?;
        let Some(_lock) = self.store.try_lock(&format!("op-{}", r.operation_id))? else {
            // An active sender might be between the last cancellation check and
            // the socket send. Never report a definitive pre-write cancellation.
            let mut f = self.store.journal(&r.operation_id)?.fact;
            f.failure = Some(
                failure(ErrorCode::SyncOperationUnknown, Step::Verify)
                    .for_operation(&r.operation_id),
            );
            f.recovery_actions = f
                .failure
                .as_ref()
                .map(|e| e.recovery_actions.clone())
                .unwrap_or_default();
            return Ok(f);
        };
        let mut j = self.store.journal(&r.operation_id)?;
        self.binding(&j.plan.binding_id)?;
        if j.fact.storage_status == StorageStatus::Stored {
            return Ok(j.fact);
        }
        if j.fact.last_write_step == WriteStep::None {
            j.fact.phase = Phase::CancelledBeforeWrite;
            j.fact.failure = Some(failure(ErrorCode::SyncCancelled, Step::SourceGuard));
        } else {
            j.fact.failure = Some(
                failure(ErrorCode::SyncOperationUnknown, Step::Verify)
                    .for_operation(&r.operation_id),
            );
        }
        j.fact.recovery_actions = j.fact.failure.as_ref().unwrap().recovery_actions.clone();
        self.store.save_journal(&j)?;
        Ok(j.fact)
    }
    pub fn assess_current_relation(&self, r: OperationRequest) -> SyncResult<Relation> {
        let j = self.store.journal(&r.operation_id)?;
        self.binding(&j.plan.binding_id)?;
        let mut relation = j.fact.relation;
        relation.local = match self.local.current_source() {
            Ok(g) if g == j.plan.source_guard => LocalRelation::ObservedEqual,
            Ok(_) => LocalRelation::Changed,
            Err(_) => LocalRelation::Unknown,
        };
        Ok(relation)
    }
    pub fn read_transfer_result(&self, r: OperationRequest) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        match self.store.journal(&r.operation_id) {
            Ok(j) => Ok(j.fact),
            Err(upload_error) => match self.store.receive(&r.operation_id) {
                Ok(r) => r.fact.ok_or(upload_error),
                Err(_) => Err(upload_error),
            },
        }
    }
    pub async fn prepare_download(&self, r: PrepareDownloadRequest) -> SyncResult<DownloadPlan> {
        self.bounded(None, self.prepare_download_inner(r)).await
    }
    async fn prepare_download_inner(&self, r: PrepareDownloadRequest) -> SyncResult<DownloadPlan> {
        self.connection_id(&r.connection_id)?;
        require_digest(&r.scope_digest)?;
        let graph = self.observe().await?;
        graph.ordinary(false)?;
        if graph.observation.state != RemoteState::Ready
            || graph.observation.generation.as_ref() != Some(&r.generation)
            || graph.observation.heads != vec![r.head.clone()]
            || graph.observation.view_digest != r.expected_view_digest
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        let bytes = self.fetch(&r.generation, &r.head, &graph).await?;
        let schema = graph
            .node(&r.generation, &r.head)?
            .commit
            .payload_schema
            .clone();
        let after = self.observe().await?;
        if after.observation.view_digest != r.expected_view_digest {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        let target_digest = self.local.target_digest(&r.scope_digest)?;
        require_digest(&target_digest)?;
        let operation_id = new_id();
        let handle = new_id();
        let prepared_at = now();
        let expires_at = self.expiry(&prepared_at)?;
        let mut plan = DownloadPlan {
            dto_version: 1,
            operation_id: operation_id.clone(),
            plan_digest: String::new(),
            binding_id: self.connection.binding_id.clone(),
            generation: r.generation.clone(),
            head: r.head.clone(),
            payload_digest: digest(&bytes),
            scope_digest: r.scope_digest,
            target_digest,
            expected_view_digest: r.expected_view_digest,
            backup_before_update: r.backup_before_update,
            prepared_at,
            expires_at,
        };
        plan.plan_digest = plan_digest(&plan)?;
        let fact = OperationFact {
            dto_version: 1,
            operation_id: operation_id.clone(),
            kind: OperationKind::Download,
            phase: Phase::Prepared,
            last_write_step: WriteStep::None,
            storage_status: StorageStatus::Stored,
            local_apply_status: LocalApplyStatus::NotAttempted,
            relation: unknown_relation(),
            generation: Some(r.generation),
            payload_digest: plan.payload_digest.clone(),
            commit: Some(r.head),
            stored_at: Some(now()),
            observation: Some(after.observation),
            failure: None,
            recovery_actions: vec![],
        };
        self.store.freeze(&handle, &bytes)?;
        self.store.save_receive(
            &operation_id,
            &ReceiveRecord {
                download: Some(plan.clone()),
                recovery: None,
                payload_handle: handle,
                payload_schema: schema,
                root_revision: self.local.root_revision()?,
                connection_revision: self.connection.revision.clone(),
                fact: Some(fact),
                receipt: None,
            },
        )?;
        Ok(plan)
    }
    fn expiry(&self, prepared: &str) -> SyncResult<String> {
        let p = chrono::DateTime::parse_from_rfc3339(prepared)
            .map_err(|_| failure(ErrorCode::SyncPlanChanged, Step::SourceGuard))?;
        Ok(
            (p + chrono::Duration::milliseconds(self.budgets.preview_ms))
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        )
    }
    fn receive_context(&self, r: &ReceiveRecord, binding: &str) -> SyncResult<()> {
        self.binding(binding)?;
        if r.root_revision != self.local.root_revision()?
            || r.connection_revision != self.connection.revision
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        Ok(())
    }
    pub async fn apply_download(&self, r: ConfirmDownloadRequest) -> SyncResult<OperationFact> {
        let id = r.operation_id.clone();
        self.bounded(Some(&id), self.apply_download_inner(r)).await
    }
    async fn apply_download_inner(&self, r: ConfirmDownloadRequest) -> SyncResult<OperationFact> {
        require_uuid(&r.operation_id)?;
        let _lock = self.store.lock(&format!("op-{}", r.operation_id)).await?;
        let mut record = self.store.receive(&r.operation_id)?;
        let p = record
            .download
            .clone()
            .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard))?;
        if p.operation_id != r.operation_id
            || p.plan_digest != r.plan_digest
            || plan_digest(&p)? != r.plan_digest
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        self.receive_context(&record, &p.binding_id)?;
        if record
            .fact
            .as_ref()
            .is_some_and(|f| f.local_apply_status == LocalApplyStatus::Applied)
        {
            return Ok(record.fact.unwrap());
        }
        self.fresh(&p.prepared_at, &p.expires_at)?;
        let graph = self.observe().await?;
        graph.ordinary(false)?;
        if graph.observation.generation.as_ref() != Some(&p.generation)
            || graph.observation.heads != vec![p.head.clone()]
            || graph.observation.view_digest != p.expected_view_digest
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        let remote = self.fetch(&p.generation, &p.head, &graph).await?;
        if digest(&remote) != p.payload_digest {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::PayloadGet));
        }
        let after = self.observe().await?;
        if after.observation.view_digest != p.expected_view_digest {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard));
        }
        let bytes = self
            .store
            .payload(&record.payload_handle, &p.payload_digest)?;
        let fact = record
            .fact
            .as_mut()
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        fact.local_apply_status = LocalApplyStatus::Unknown;
        self.store.save_receive(&r.operation_id, &record)?;
        // The host performs expected-target + backup + once-only receipt atomically.
        let applied = self.local.apply_once(
            &r.operation_id,
            &bytes,
            &record.payload_schema,
            &p.scope_digest,
            &p.target_digest,
            p.backup_before_update,
        );
        let fact = record.fact.as_mut().unwrap();
        fact.phase = Phase::Settled;
        match applied {
            Ok(()) => {
                fact.local_apply_status = LocalApplyStatus::Applied;
                fact.failure = None;
                fact.recovery_actions = vec![];
                fact.observation = Some(after.observation);
                fact.relation = Relation {
                    local: LocalRelation::ObservedEqual,
                    remote: RemoteRelation::ObservedEqual,
                };
            }
            Err(e) => {
                fact.failure = Some(e.clone());
                fact.recovery_actions = e.recovery_actions.clone();
                self.store.save_receive(&r.operation_id, &record)?;
                return Err(e);
            }
        }
        self.store.save_receive(&r.operation_id, &record)?;
        Ok(record.fact.unwrap())
    }
    pub async fn prepare_recovery(&self, r: PrepareRecoveryRequest) -> SyncResult<RecoveryPlan> {
        self.bounded(None, self.prepare_recovery_inner(r)).await
    }
    async fn prepare_recovery_inner(&self, r: PrepareRecoveryRequest) -> SyncResult<RecoveryPlan> {
        self.connection_id(&r.connection_id)?;
        self.binding(&r.source_binding_id)?;
        require_digest(&r.scope_digest)?;
        let graph = self.observe().await?;
        if graph.observation.state != RemoteState::InitializationAmbiguous
            || graph.observation.view_digest != r.expected_view_digest
            || !graph
                .observation
                .recovery_candidates
                .iter()
                .any(|c| c.generation == r.generation && c.head == r.head)
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        let bytes = self.fetch(&r.generation, &r.head, &graph).await?;
        let schema = graph
            .node(&r.generation, &r.head)?
            .commit
            .payload_schema
            .clone();
        if self.observe().await?.observation.view_digest != r.expected_view_digest {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        let id = new_id();
        let handle = new_id();
        let prepared_at = now();
        let expires_at = self.expiry(&prepared_at)?;
        let target_digest = if r.mode == RecoveryMode::Download {
            Some(self.local.target_digest(&r.scope_digest)?)
        } else {
            None
        };
        let mut plan = RecoveryPlan {
            dto_version: 1,
            recovery_plan_id: id.clone(),
            plan_digest: String::new(),
            kind: if r.mode == RecoveryMode::Download {
                RecoveryPlanKind::RecoveryDownload
            } else {
                RecoveryPlanKind::RecoveryExport
            },
            source_binding_id: r.source_binding_id,
            generation: r.generation,
            head: r.head,
            payload_digest: digest(&bytes),
            payload_schema: schema.clone(),
            scope_digest: r.scope_digest,
            target_digest,
            expected_view_digest: r.expected_view_digest,
            backup_before_update: r.backup_before_update,
            prepared_at,
            expires_at,
        };
        plan.plan_digest = plan_digest(&plan)?;
        self.store.freeze(&handle, &bytes)?;
        self.store.save_receive(
            &id,
            &ReceiveRecord {
                download: None,
                recovery: Some(plan.clone()),
                payload_handle: handle,
                payload_schema: schema,
                root_revision: self.local.root_revision()?,
                connection_revision: self.connection.revision.clone(),
                fact: None,
                receipt: None,
            },
        )?;
        Ok(plan)
    }
    pub async fn apply_recovery(&self, r: ConfirmRecoveryRequest) -> SyncResult<RecoveryReceipt> {
        let id = r.recovery_plan_id.clone();
        self.bounded(Some(&id), self.apply_recovery_inner(r)).await
    }
    async fn apply_recovery_inner(&self, r: ConfirmRecoveryRequest) -> SyncResult<RecoveryReceipt> {
        require_uuid(&r.recovery_plan_id)?;
        let _lock = self
            .store
            .lock(&format!("op-{}", r.recovery_plan_id))
            .await?;
        let mut record = self.store.receive(&r.recovery_plan_id)?;
        let p = record
            .recovery
            .clone()
            .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::Recovery))?;
        if p.recovery_plan_id != r.recovery_plan_id
            || p.plan_digest != r.plan_digest
            || plan_digest(&p)? != r.plan_digest
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        self.receive_context(&record, &p.source_binding_id)?;
        if let Some(receipt) = &record.receipt {
            if matches!(
                receipt.outcome,
                RecoveryOutcome::Applied | RecoveryOutcome::Exported
            ) {
                return Ok(receipt.clone());
            }
        }
        self.fresh(&p.prepared_at, &p.expires_at)?;
        let graph = self.observe().await?;
        if graph.observation.view_digest != p.expected_view_digest
            || !graph
                .observation
                .recovery_candidates
                .iter()
                .any(|c| c.generation == p.generation && c.head == p.head)
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        let bytes = self.fetch(&p.generation, &p.head, &graph).await?;
        if digest(&bytes) != p.payload_digest
            || self.observe().await?.observation.view_digest != p.expected_view_digest
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        self.store
            .payload(&record.payload_handle, &p.payload_digest)?;
        record.receipt = Some(RecoveryReceipt {
            dto_version: 1,
            recovery_plan_id: p.recovery_plan_id.clone(),
            kind: p.kind,
            outcome: RecoveryOutcome::Unknown,
            source_binding_id: p.source_binding_id.clone(),
            generation: p.generation.clone(),
            head: p.head.clone(),
            payload_digest: p.payload_digest.clone(),
            artifact_id: None,
            finished_at: now(),
            failure: None,
            recovery_actions: vec![],
        });
        self.store.save_receive(&r.recovery_plan_id, &record)?;
        let applied = if p.kind == RecoveryPlanKind::RecoveryDownload {
            self.local
                .apply_once(
                    &p.recovery_plan_id,
                    &bytes,
                    &p.payload_schema,
                    &p.scope_digest,
                    p.target_digest
                        .as_ref()
                        .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Recovery))?,
                    p.backup_before_update,
                )
                .map(|()| None)
        } else {
            self.local
                .export_once(
                    &p.recovery_plan_id,
                    &bytes,
                    &p.payload_schema,
                    &p.scope_digest,
                )
                .and_then(|id| {
                    require_uuid(&id)?;
                    Ok(Some(id))
                })
        };
        let receipt = record.receipt.as_mut().unwrap();
        receipt.finished_at = now();
        match applied {
            Ok(artifact) => {
                receipt.outcome = if p.kind == RecoveryPlanKind::RecoveryDownload {
                    RecoveryOutcome::Applied
                } else {
                    RecoveryOutcome::Exported
                };
                receipt.artifact_id = artifact;
                receipt.recovery_actions = vec![action(RecoveryKind::SelectEmptyDirectory)];
            }
            Err(e) => {
                receipt.failure = Some(e.clone());
                receipt.recovery_actions = e.recovery_actions.clone();
                self.store.save_receive(&r.recovery_plan_id, &record)?;
                return Err(e);
            }
        }
        self.store.save_receive(&r.recovery_plan_id, &record)?;
        Ok(record.receipt.unwrap())
    }
}
