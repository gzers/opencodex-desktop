use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

#[derive(Default)]
struct Remote {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
    writes: AtomicUsize,
    reads: AtomicUsize,
    lose_commit_response: std::sync::atomic::AtomicBool,
    lose_generation_response: std::sync::atomic::AtomicBool,
    hide: Mutex<BTreeSet<String>>,
    fail_put: Mutex<Option<TransportKind>>,
    before_put: Mutex<Option<(String, Vec<u8>)>>,
    commit_barrier: Mutex<Option<Arc<tokio::sync::Barrier>>>,
    generation_barrier: Mutex<Option<Arc<tokio::sync::Barrier>>>,
    lose_payload_response: std::sync::atomic::AtomicBool,
    fail_list: Mutex<Option<TransportKind>>,
    list_delay_ms: AtomicUsize,
    get_delay_ms: AtomicUsize,
    vanish_unknown: std::sync::atomic::AtomicBool,
}
fn terr(kind: TransportKind) -> SyncTransportError {
    SyncTransportError {
        kind,
        http_status: None,
        delivery: Delivery::MayHaveBeenSent,
    }
}
#[async_trait::async_trait]
impl ConfigSyncTransport for Remote {
    async fn list_children(
        &self,
        _: &ConnectionHandle,
        _: ListLimits,
    ) -> Result<ChildListing, SyncTransportError> {
        if let Some(kind) = self.fail_list.lock().unwrap().take() {
            return Err(terr(kind));
        }
        tokio::time::sleep(Duration::from_millis(
            self.list_delay_ms.load(Ordering::SeqCst) as u64,
        ))
        .await;
        let mut files = self.files.lock().unwrap();
        if self.vanish_unknown.swap(false, Ordering::SeqCst) {
            files.remove("unknown-business.json");
            return Ok(ChildListing {
                entries: vec![ChildEntry {
                    name: "unknown-business.json".into(),
                    is_collection: false,
                    content_length: Some(4),
                }],
                http_status: 207,
                response_bytes: 100,
            });
        }
        let hide = self.hide.lock().unwrap();
        Ok(ChildListing {
            entries: files
                .iter()
                .filter(|(k, _)| !hide.contains(*k))
                .map(|(name, b)| ChildEntry {
                    name: name.clone(),
                    is_collection: false,
                    content_length: Some(b.len() as u64),
                })
                .collect(),
            http_status: 207,
            response_bytes: 100,
        })
    }
    async fn get_file(
        &self,
        _: &ConnectionHandle,
        path: &SyncRelativePath,
        _: ReadLimits,
    ) -> Result<FileRead, SyncTransportError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(
            self.get_delay_ms.load(Ordering::SeqCst) as u64,
        ))
        .await;
        self.files
            .lock()
            .unwrap()
            .get(path.as_str())
            .cloned()
            .map(|body| FileRead {
                body,
                http_status: 200,
            })
            .ok_or_else(|| terr(TransportKind::NotFound))
    }
    async fn put_file(
        &self,
        _: &ConnectionHandle,
        path: &SyncRelativePath,
        body: &[u8],
        _: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        let barrier = if path.as_str().contains("-commit-") {
            self.commit_barrier.lock().unwrap().clone()
        } else if path.as_str().contains("-generation-") {
            self.generation_barrier.lock().unwrap().clone()
        } else {
            None
        };
        if let Some(barrier) = barrier {
            barrier.wait().await;
        }
        if let Some(kind) = self.fail_put.lock().unwrap().take() {
            return Err(terr(kind));
        }
        if let Some((name, bytes)) = self.before_put.lock().unwrap().take() {
            self.files.lock().unwrap().insert(name, bytes);
        }
        self.files
            .lock()
            .unwrap()
            .insert(path.as_str().into(), body.to_vec());
        if path.as_str().contains("-data-")
            && self.lose_payload_response.swap(false, Ordering::SeqCst)
        {
            return Err(terr(TransportKind::Timeout));
        }
        if path.as_str().contains("-commit-")
            && self.lose_commit_response.swap(false, Ordering::SeqCst)
        {
            return Err(terr(TransportKind::Timeout));
        }
        if path.as_str().contains("-generation-")
            && self.lose_generation_response.swap(false, Ordering::SeqCst)
        {
            return Err(terr(TransportKind::Timeout));
        }
        Ok(PutResponse { http_status: 201 })
    }
}
fn schema() -> PayloadSchemaRef {
    PayloadSchemaRef {
        envelope: SchemaRef {
            format: "OCXDCONF".into(),
            version: 3,
        },
        document: SchemaRef {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 2,
        },
        model: ModelSchemaRef {
            format: "OCXD_MODEL_PACKAGE".into(),
            format_version: 6,
            schema_version: 2,
        },
    }
}
struct Codec {
    bytes: Mutex<Vec<u8>>,
}
impl Codec {
    fn new() -> Self {
        Self {
            bytes: Mutex::new(b"synthetic frozen config".to_vec()),
        }
    }
}
impl PayloadCodec for Codec {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef> {
        vec![schema()]
    }
    fn freeze_selected(
        &self,
        _: &SourceGuard,
        base: &[PayloadInput],
        resolution: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload> {
        let bytes = if let Some(r) = resolution {
            if let Some(head) = &r.selected_head {
                base.iter().find(|b| b.head == *head).unwrap().bytes.clone()
            } else {
                self.bytes.lock().unwrap().clone()
            }
        } else {
            self.bytes.lock().unwrap().clone()
        };
        Ok(FrozenPayload {
            bytes,
            schema: schema(),
            protection_mode: ProtectionMode::Plain,
        })
    }
    fn inspect_and_validate(&self, bytes: &[u8], _: &PayloadSchemaRef) -> SyncResult<()> {
        if bytes.is_empty() {
            Err(failure(ErrorCode::SyncPayloadInvalid, Step::Decode))
        } else {
            Ok(())
        }
    }
}
struct Local {
    source: Mutex<SourceGuard>,
    target: Mutex<Digest>,
    applied: Mutex<BTreeSet<String>>,
    apply_count: AtomicUsize,
    exports: AtomicUsize,
    backup_fails: std::sync::atomic::AtomicBool,
}
impl Local {
    fn new() -> Self {
        Self {
            source: Mutex::new(SourceGuard {
                saved_source_digest: digest(b"source"),
                scope_digest: digest(b"scope"),
                mcp_sources_digest: digest(b"MCP client A"),
            }),
            target: Mutex::new(digest(b"target")),
            applied: Mutex::new(BTreeSet::new()),
            apply_count: AtomicUsize::new(0),
            exports: AtomicUsize::new(0),
            backup_fails: std::sync::atomic::AtomicBool::new(false),
        }
    }
}
impl LocalApplication for Local {
    fn root_revision(&self) -> SyncResult<Digest> {
        Ok(digest(b"root"))
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        Ok(self.source.lock().unwrap().clone())
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        Ok(self.target.lock().unwrap().clone())
    }
    fn apply_once(
        &self,
        id: &str,
        bytes: &[u8],
        _: &PayloadSchemaRef,
        _: &str,
        expected: &str,
        backup: bool,
    ) -> SyncResult<()> {
        let mut applied = self.applied.lock().unwrap();
        if applied.contains(id) {
            return Ok(());
        }
        if self.target.lock().unwrap().as_str() != expected {
            return Err(failure(ErrorCode::SyncLocalChanged, Step::ApplyLocal));
        }
        if backup && self.backup_fails.load(Ordering::SeqCst) {
            return Err(failure(ErrorCode::SyncBackupFailed, Step::ApplyLocal));
        }
        *self.target.lock().unwrap() = digest(bytes);
        applied.insert(id.into());
        self.apply_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn export_once(&self, id: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<Uuid> {
        let mut applied = self.applied.lock().unwrap();
        if applied.insert(id.into()) {
            self.exports.fetch_add(1, Ordering::SeqCst);
        }
        Ok(id.into())
    }
}
fn connection() -> ConnectionHandle {
    ConnectionHandle {
        id: "synthetic".into(),
        binding_id: digest(b"binding"),
        revision: digest(b"revision"),
    }
}
struct Env {
    dir: tempfile::TempDir,
    store: Store,
    remote: Remote,
    codec: Codec,
    local: Local,
}
impl Env {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        Self {
            dir,
            store,
            remote: Remote::default(),
            codec: Codec::new(),
            local: Local::new(),
        }
    }
    fn cx(&self) -> SyncContext<'_> {
        let mut cx = SyncContext::new(
            &self.store,
            &self.remote,
            &self.codec,
            &self.local,
            connection(),
        )
        .unwrap();
        cx.budgets.get_attempts = 1;
        cx
    }
    async fn plan(&self) -> UploadPlan {
        let cx = self.cx();
        let o = cx
            .observe_remote(ObserveRequest {
                connection_id: "synthetic".into(),
            })
            .await
            .unwrap();
        cx.prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o.view_digest,
            source_guard: self.local.current_source().unwrap(),
            initialize_on_confirm: true,
        })
        .await
        .unwrap()
    }
    async fn upload(&self) -> OperationFact {
        let p = self.plan().await;
        self.cx()
            .publish_upload(ConfirmUploadRequest {
                operation_id: p.operation_id,
                plan_digest: p.plan_digest,
            })
            .await
            .unwrap()
    }
}
fn seed_generation(remote: &Remote) -> GenerationRef {
    let g = Generation {
        format: "OCXD_CONFIG_GENERATION".into(),
        format_version: 1,
        generation_id: new_id(),
    };
    let b = canonical(&g).unwrap();
    remote.files.lock().unwrap().insert(
        SyncRelativePath::generation(&g.generation_id)
            .unwrap()
            .as_str()
            .into(),
        b.clone(),
    );
    GenerationRef {
        generation_id: g.generation_id,
        digest: digest(&b),
    }
}
fn seed_commit(
    remote: &Remote,
    g: &GenerationRef,
    parents: Vec<CommitRef>,
    bytes: &[u8],
) -> CommitRef {
    let id = new_id();
    let c = Commit {
        format: "OCXD_CONFIG_COMMIT".into(),
        format_version: 1,
        generation_id: g.generation_id.clone(),
        operation_id: id.clone(),
        parents,
        payload_digest: digest(bytes),
        payload_bytes: bytes.len() as u64,
        payload_schema: schema(),
        intent: Intent::Update,
        resolution: None,
    };
    let b = canonical(&c).unwrap();
    let mut files = remote.files.lock().unwrap();
    files.insert(
        SyncRelativePath::commit(&g.generation_id, &id)
            .unwrap()
            .as_str()
            .into(),
        b.clone(),
    );
    files.insert(
        SyncRelativePath::data(&g.generation_id, &id)
            .unwrap()
            .as_str()
            .into(),
        bytes.to_vec(),
    );
    CommitRef {
        operation_id: id,
        digest: digest(&b),
    }
}
#[test]
fn strict_wire_rejects_duplicate_null_unknown_float_and_invalid_paths() {
    for bytes in [br#"{"format":"OCXD_CONFIG_GENERATION","format":"other","formatVersion":1,"generationId":"x"}"#.as_slice(),br#"{"format":"x","formatVersion":1.0,"generationId":"x"}"#,br#"{"format":"x","formatVersion":null,"generationId":"x"}"#,br#"{"format":"x","formatVersion":1,"generationId":"x","etag":"x"}"#] {assert!(store::decode::<Generation>(bytes).is_err());}
    assert!(SyncRelativePath::generation("../secrets").is_err());
    assert!(graph::file_key(&format!("config-v2-commit-{}.json", "界".repeat(73))).is_err());
}
#[tokio::test]
async fn first_prepare_read_only_then_confirmation_initializes_and_stores() {
    let e = Env::new();
    let p = e.plan().await;
    assert_eq!(p.initialization, Initialization::OnConfirm);
    assert!(p.generation.is_none());
    assert!(e.remote.files.lock().unwrap().is_empty());
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
    let f = e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone(),
        })
        .await
        .unwrap();
    assert_eq!(f.storage_status, StorageStatus::Stored);
    assert_eq!(f.relation.remote, RemoteRelation::ObservedEqual);
    assert_eq!(e.remote.files.lock().unwrap().len(), 3);
    let count = e.remote.writes.load(Ordering::SeqCst);
    e.cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(count, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn lost_commit_response_reopen_verify_ignores_new_source_scope_and_evicted_payload() {
    let e = Env::new();
    let p = e.plan().await;
    e.remote.lose_commit_response.store(true, Ordering::SeqCst);
    assert!(e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    let j = e.store.journal(&p.operation_id).unwrap();
    assert_eq!(j.fact.phase, Phase::CommitMayBeWritten);
    std::fs::remove_file(e.dir.path().join(format!("{}.payload", j.payload_handle))).unwrap();
    *e.local.source.lock().unwrap() = SourceGuard {
        saved_source_digest: digest(b"later"),
        scope_digest: digest(b"different selection"),
        mcp_sources_digest: digest(b"client B"),
    };
    let before = e.remote.writes.load(Ordering::SeqCst);
    let reopened = Store::open(e.dir.path()).unwrap();
    let mut cx = SyncContext::new(&reopened, &e.remote, &e.codec, &e.local, connection()).unwrap();
    cx.budgets.get_attempts = 1;
    let f = cx
        .verify_original_upload(OperationRequest {
            operation_id: p.operation_id.clone(),
        })
        .await
        .unwrap();
    assert_eq!(f.storage_status, StorageStatus::Stored);
    assert_eq!(f.relation.local, LocalRelation::NotCompared);
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
    assert_eq!(
        cx.assess_current_relation(OperationRequest {
            operation_id: p.operation_id
        })
        .unwrap()
        .local,
        LocalRelation::Changed
    );
}
#[tokio::test]
async fn mcp_source_change_blocks_publish_but_not_original_verification() {
    let e = Env::new();
    let p = e.plan().await;
    e.local.source.lock().unwrap().mcp_sources_digest = digest(b"client B");
    let err = e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SyncScopeChanged);
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn graph_retains_concurrent_roots_and_resolution_keeps_original_files() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let a = seed_commit(&e.remote, &g, vec![], b"a");
    let b = seed_commit(&e.remote, &g, vec![], b"b");
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Conflict);
    assert_eq!(o.heads.len(), 2);
    let original = e.remote.files.lock().unwrap().clone();
    let p = e
        .cx()
        .prepare_resolution(PrepareResolutionRequest {
            connection_id: "synthetic".into(),
            generation: g,
            heads: o.heads,
            expected_view_digest: o.view_digest,
            source_guard: e.local.current_source().unwrap(),
            resolution: Resolution {
                selection_mode: SelectionMode::Head,
                selected_head: Some(a.clone()),
                scope_digest: digest(b"scope"),
                preview_digest: digest(b"explicit all-head preview"),
            },
        })
        .await
        .unwrap();
    let f = e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(f.relation.remote, RemoteRelation::ObservedEqual);
    for (name, bytes) in original {
        assert_eq!(e.remote.files.lock().unwrap().get(&name), Some(&bytes));
    }
    let obs = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(obs.heads.len(), 1);
    assert_ne!(obs.heads[0], a);
    assert_ne!(obs.heads[0], b);
}
#[tokio::test]
async fn delayed_listing_does_not_forget_observed_head() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let a = seed_commit(&e.remote, &g, vec![], b"a");
    let b = seed_commit(&e.remote, &g, vec![], b"b");
    let cx = e.cx();
    let first = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    e.remote.hide.lock().unwrap().insert(
        SyncRelativePath::commit(&g.generation_id, &b.operation_id)
            .unwrap()
            .as_str()
            .into(),
    );
    let second = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(second.state, RemoteState::Conflict);
    assert_eq!(first.view_digest, second.view_digest);
    assert!(second.heads.contains(&a));
    assert!(second.heads.contains(&b));
}
#[tokio::test]
async fn missing_parent_or_corrupt_json_is_never_empty() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let bogus = CommitRef {
        operation_id: new_id(),
        digest: digest(b"missing"),
    };
    seed_commit(&e.remote, &g, vec![bogus], b"child");
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Incomplete);
    {
        let mut files = e.remote.files.lock().unwrap();
        let name = files
            .keys()
            .find(|n| n.contains("-commit-"))
            .unwrap()
            .clone();
        files.insert(name, b"{".to_vec());
    }
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Invalid);
}
#[tokio::test]
async fn ambiguous_generations_only_recovery_exports_and_never_remote_writes() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let a = seed_commit(&e.remote, &g, vec![], b"retained config");
    seed_generation(&e.remote);
    let cx = e.cx();
    let o = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::InitializationAmbiguous);
    assert!(cx
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o.view_digest.clone(),
            source_guard: e.local.current_source().unwrap(),
            initialize_on_confirm: true
        })
        .await
        .is_err());
    let p = cx
        .prepare_recovery(PrepareRecoveryRequest {
            connection_id: "synthetic".into(),
            source_binding_id: connection().binding_id,
            generation: g,
            head: a,
            expected_view_digest: o.view_digest,
            scope_digest: digest(b"scope"),
            mode: RecoveryMode::Export,
            backup_before_update: true,
        })
        .await
        .unwrap();
    let result = cx
        .apply_recovery(ConfirmRecoveryRequest {
            recovery_plan_id: p.recovery_plan_id.clone(),
            plan_digest: p.plan_digest.clone(),
        })
        .await
        .unwrap();
    assert_eq!(result.outcome, RecoveryOutcome::Exported);
    cx.apply_recovery(ConfirmRecoveryRequest {
        recovery_plan_id: p.recovery_plan_id,
        plan_digest: p.plan_digest,
    })
    .await
    .unwrap();
    assert_eq!(e.local.exports.load(Ordering::SeqCst), 1);
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn cancellation_and_result_reads_do_not_write_or_contact_remote() {
    let e = Env::new();
    let p = e.plan().await;
    let cx = e.cx();
    let f = cx
        .cancel_upload(OperationRequest {
            operation_id: p.operation_id.clone(),
        })
        .unwrap();
    assert_eq!(f.phase, Phase::CancelledBeforeWrite);
    let reads = e.remote.reads.load(Ordering::SeqCst);
    cx.read_transfer_result(OperationRequest {
        operation_id: p.operation_id.clone(),
    })
    .unwrap();
    assert_eq!(reads, e.remote.reads.load(Ordering::SeqCst));
    assert!(cx
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn cancel_after_uncertain_send_retains_same_operation_and_verify_is_read_only() {
    let e = Env::new();
    let p = e.plan().await;
    e.remote.lose_commit_response.store(true, Ordering::SeqCst);
    let cx = e.cx();
    assert!(cx
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    let f = cx
        .cancel_upload(OperationRequest {
            operation_id: p.operation_id.clone(),
        })
        .unwrap();
    assert_ne!(f.phase, Phase::CancelledBeforeWrite);
    let before = e.remote.writes.load(Ordering::SeqCst);
    assert_eq!(
        cx.verify_original_upload(OperationRequest {
            operation_id: p.operation_id
        })
        .await
        .unwrap()
        .storage_status,
        StorageStatus::Stored
    );
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn generation_reset_rejects_old_plan_and_ignores_late_old_data() {
    let e = Env::new();
    e.upload().await;
    let p = e.plan().await;
    let old = p.generation.clone().unwrap();
    e.remote.files.lock().unwrap().clear();
    let current = seed_generation(&e.remote);
    e.remote.files.lock().unwrap().insert(
        SyncRelativePath::data(&old.generation_id, &new_id())
            .unwrap()
            .as_str()
            .into(),
        b"late".to_vec(),
    );
    let before = e.remote.writes.load(Ordering::SeqCst);
    assert!(e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.generation, Some(current));
    assert_eq!(o.state, RemoteState::GenerationEmpty);
    assert_eq!(o.stale_count, 1);
}
#[tokio::test]
async fn initialization_lost_response_resumes_without_new_generation() {
    let e = Env::new();
    let p = e.plan().await;
    e.remote
        .lose_generation_response
        .store(true, Ordering::SeqCst);
    let cx = e.cx();
    assert!(cx
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone()
        })
        .await
        .is_err());
    let g = e
        .store
        .journal(&p.operation_id)
        .unwrap()
        .fact
        .generation
        .unwrap();
    let o = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let f = cx
        .resume_original_upload(ResumeUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest,
            expected_view_digest: o.view_digest,
        })
        .await
        .unwrap();
    assert_eq!(f.generation, Some(g));
    assert_eq!(f.storage_status, StorageStatus::Stored);
    assert_eq!(
        e.remote
            .files
            .lock()
            .unwrap()
            .keys()
            .filter(|k| k.contains("-generation-"))
            .count(),
        1
    );
}
#[tokio::test]
async fn download_protects_local_target_backup_and_idempotent_reapply() {
    let e = Env::new();
    e.upload().await;
    let cx = e.cx();
    let o = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let p = cx
        .prepare_download(PrepareDownloadRequest {
            connection_id: "synthetic".into(),
            generation: o.generation.unwrap(),
            head: o.heads[0].clone(),
            expected_view_digest: o.view_digest,
            scope_digest: digest(b"scope"),
            backup_before_update: true,
        })
        .await
        .unwrap();
    let before = e.remote.writes.load(Ordering::SeqCst);
    e.local.backup_fails.store(true, Ordering::SeqCst);
    assert_eq!(
        cx.apply_download(ConfirmDownloadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone()
        })
        .await
        .unwrap_err()
        .code,
        ErrorCode::SyncBackupFailed
    );
    assert_eq!(e.local.apply_count.load(Ordering::SeqCst), 0);
    e.local.backup_fails.store(false, Ordering::SeqCst);
    let f = cx
        .apply_download(ConfirmDownloadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone(),
        })
        .await
        .unwrap();
    assert_eq!(f.local_apply_status, LocalApplyStatus::Applied);
    cx.apply_download(ConfirmDownloadRequest {
        operation_id: p.operation_id,
        plan_digest: p.plan_digest,
    })
    .await
    .unwrap();
    assert_eq!(e.local.apply_count.load(Ordering::SeqCst), 1);
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn capacity_and_unsupported_schema_fail_without_silent_truncation() {
    let e = Env::new();
    seed_generation(&e.remote);
    let mut cx = e.cx();
    cx.budgets.listing_entries = 1;
    assert_eq!(
        cx.observe_remote(ObserveRequest {
            connection_id: "synthetic".into()
        })
        .await
        .unwrap_err()
        .code,
        ErrorCode::SyncLimit
    );
    let e = Env::new();
    let p = e.plan().await;
    *e.remote.fail_put.lock().unwrap() = Some(TransportKind::InsufficientStorage);
    let err = e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SyncRemoteFull);
}
#[test]
fn corrupt_journal_and_symlink_do_not_become_fresh_state() {
    let e = Env::new();
    let id = new_id();
    std::fs::write(e.dir.path().join(format!("{id}.upload.json")), b"{}").unwrap();
    assert_eq!(
        e.store.journal(&id).unwrap_err().code,
        ErrorCode::SyncJournalInvalid
    );
    #[cfg(unix)]
    {
        let target = e.dir.path().join("outside");
        std::fs::write(&target, b"sentinel").unwrap();
        let id = new_id();
        std::os::unix::fs::symlink(&target, e.dir.path().join(format!("{id}.payload"))).unwrap();
        assert!(e.store.freeze(&id, b"replacement").is_err());
        assert_eq!(std::fs::read(target).unwrap(), b"sentinel");
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn simultaneous_commits_from_independent_clients_retain_both_stored_branches() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    seed_commit(&e.remote, &g, vec![], b"common base");
    let other = tempfile::tempdir().unwrap();
    let store2 = Store::open(other.path()).unwrap();
    let local2 = Local::new();
    let codec2 = Codec::new();
    *codec2.bytes.lock().unwrap() = b"client two".to_vec();
    let first = e.cx();
    let mut second = SyncContext::new(&store2, &e.remote, &codec2, &local2, connection()).unwrap();
    second.budgets.get_attempts = 1;
    let o1 = first
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let o2 = second
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let p1 = first
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o1.view_digest,
            source_guard: e.local.current_source().unwrap(),
            initialize_on_confirm: false,
        })
        .await
        .unwrap();
    let p2 = second
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o2.view_digest,
            source_guard: local2.current_source().unwrap(),
            initialize_on_confirm: false,
        })
        .await
        .unwrap();
    *e.remote.commit_barrier.lock().unwrap() = Some(Arc::new(tokio::sync::Barrier::new(2)));
    let (a, b) = tokio::join!(
        first.publish_upload(ConfirmUploadRequest {
            operation_id: p1.operation_id.clone(),
            plan_digest: p1.plan_digest
        }),
        second.publish_upload(ConfirmUploadRequest {
            operation_id: p2.operation_id.clone(),
            plan_digest: p2.plan_digest
        })
    );
    assert_eq!(a.unwrap().storage_status, StorageStatus::Stored);
    assert_eq!(b.unwrap().storage_status, StorageStatus::Stored);
    let o = first
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Conflict);
    assert_eq!(o.heads.len(), 2);
    let ids: BTreeSet<_> = o.heads.iter().map(|h| h.operation_id.clone()).collect();
    assert!(ids.contains(&p1.operation_id) && ids.contains(&p2.operation_id));
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn simultaneous_first_initializations_preserve_markers_and_stop_before_data() {
    let e = Env::new();
    let other = tempfile::tempdir().unwrap();
    let store2 = Store::open(other.path()).unwrap();
    let local2 = Local::new();
    let codec2 = Codec::new();
    let first = e.cx();
    let mut second = SyncContext::new(&store2, &e.remote, &codec2, &local2, connection()).unwrap();
    second.budgets.get_attempts = 1;
    let p1 = e.plan().await;
    let o = second
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let p2 = second
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o.view_digest,
            source_guard: local2.current_source().unwrap(),
            initialize_on_confirm: true,
        })
        .await
        .unwrap();
    *e.remote.generation_barrier.lock().unwrap() = Some(Arc::new(tokio::sync::Barrier::new(2)));
    let (a, b) = tokio::join!(
        first.publish_upload(ConfirmUploadRequest {
            operation_id: p1.operation_id,
            plan_digest: p1.plan_digest
        }),
        second.publish_upload(ConfirmUploadRequest {
            operation_id: p2.operation_id,
            plan_digest: p2.plan_digest
        })
    );
    assert!(a.is_err() && b.is_err());
    let files = e.remote.files.lock().unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.keys().all(|n| n.contains("-generation-")));
}
#[tokio::test]
async fn unsupported_payload_schema_and_changed_download_target_are_rejected() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let h = seed_commit(&e.remote, &g, vec![], b"payload");
    let path = SyncRelativePath::commit(&g.generation_id, &h.operation_id).unwrap();
    let mut c: Commit =
        store::decode(e.remote.files.lock().unwrap().get(path.as_str()).unwrap()).unwrap();
    c.payload_schema.model.format_version = 7;
    e.remote
        .files
        .lock()
        .unwrap()
        .insert(path.as_str().into(), canonical(&c).unwrap());
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let err = e
        .cx()
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: o.view_digest,
            source_guard: e.local.current_source().unwrap(),
            initialize_on_confirm: false,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SyncPayloadSchemaUnsupported);
    let e = Env::new();
    e.upload().await;
    let cx = e.cx();
    let o = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let p = cx
        .prepare_download(PrepareDownloadRequest {
            connection_id: "synthetic".into(),
            generation: o.generation.unwrap(),
            head: o.heads[0].clone(),
            expected_view_digest: o.view_digest,
            scope_digest: digest(b"scope"),
            backup_before_update: true,
        })
        .await
        .unwrap();
    *e.local.target.lock().unwrap() = digest(b"external later save");
    assert_eq!(
        cx.apply_download(ConfirmDownloadRequest {
            operation_id: p.operation_id,
            plan_digest: p.plan_digest
        })
        .await
        .unwrap_err()
        .code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(e.local.apply_count.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn changed_payload_is_not_confirmed_and_original_id_is_never_overwritten() {
    let e = Env::new();
    let f = e.upload().await;
    let g = f.generation.unwrap();
    let path = SyncRelativePath::data(&g.generation_id, &f.operation_id).unwrap();
    e.remote
        .files
        .lock()
        .unwrap()
        .insert(path.as_str().into(), b"corrupt".to_vec());
    let before = e.remote.writes.load(Ordering::SeqCst);
    assert_eq!(
        e.cx()
            .verify_original_upload(OperationRequest {
                operation_id: f.operation_id
            })
            .await
            .unwrap_err()
            .code,
        ErrorCode::SyncOperationCollision
    );
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn whole_publish_deadline_is_not_reset_by_repeated_observations() {
    let e = Env::new();
    let p = e.plan().await;
    e.remote.list_delay_ms.store(12, Ordering::SeqCst);
    e.remote.get_delay_ms.store(3, Ordering::SeqCst);
    let mut cx = e.cx();
    cx.budgets.observation_ms = 85;
    assert!(cx
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    let j = e.store.journal(&p.operation_id).unwrap();
    assert!(j.fact.failure.is_some());
    assert_ne!(j.fact.storage_status, StorageStatus::Stored);
    // The failure, rather than a completed publication, proves later observe
    // calls did not each obtain a fresh full command deadline.
}
#[tokio::test]
async fn whole_prepare_download_deadline_is_shared_between_read_and_recheck() {
    let e = Env::new();
    e.upload().await;
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    e.remote.list_delay_ms.store(30, Ordering::SeqCst);
    e.remote.get_delay_ms.store(15, Ordering::SeqCst);
    let mut cx = e.cx();
    cx.budgets.observation_ms = 210;
    let err = cx
        .prepare_download(PrepareDownloadRequest {
            connection_id: "synthetic".into(),
            generation: o.generation.unwrap(),
            head: o.heads[0].clone(),
            expected_view_digest: o.view_digest,
            scope_digest: digest(b"scope"),
            backup_before_update: true,
        })
        .await
        .unwrap_err();
    assert_eq!(err.cause, Some(Cause::Timeout));
}
#[tokio::test]
async fn whole_verify_deadline_preserves_historical_stored_evidence() {
    let e = Env::new();
    let f = e.upload().await;
    let stored_at = f.stored_at.clone();
    let before = e.remote.writes.load(Ordering::SeqCst);
    e.remote.list_delay_ms.store(15, Ordering::SeqCst);
    e.remote.get_delay_ms.store(15, Ordering::SeqCst);
    let mut cx = e.cx();
    cx.budgets.observation_ms = 90;
    assert!(cx
        .verify_original_upload(OperationRequest {
            operation_id: f.operation_id.clone()
        })
        .await
        .is_err());
    let original = e.store.journal(&f.operation_id).unwrap();
    assert_eq!(original.fact.storage_status, StorageStatus::Stored);
    assert_eq!(original.fact.stored_at, stored_at);
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), before);
}
#[tokio::test]
async fn unknown_business_name_seen_in_listing_union_is_never_empty() {
    let e = Env::new();
    e.remote
        .files
        .lock()
        .unwrap()
        .insert("unknown-business.json".into(), b"data".to_vec());
    e.remote.vanish_unknown.store(true, Ordering::SeqCst);
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Invalid);
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn seen_root_commit_identity_cannot_be_silently_replaced() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let head = seed_commit(&e.remote, &g, vec![], b"original");
    e.cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let path = SyncRelativePath::commit(&g.generation_id, &head.operation_id).unwrap();
    let mut c: Commit =
        store::decode(e.remote.files.lock().unwrap().get(path.as_str()).unwrap()).unwrap();
    c.payload_digest = digest(b"external replacement");
    e.remote
        .files
        .lock()
        .unwrap()
        .insert(path.as_str().into(), canonical(&c).unwrap());
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Invalid);
    assert!(o
        .problems
        .iter()
        .any(|p| p.code == ErrorCode::SyncOperationCollision));
}
#[tokio::test]
async fn resume_cannot_write_through_changed_connection_or_root_revision() {
    struct DifferentRoot<'a>(&'a Local);
    impl LocalApplication for DifferentRoot<'_> {
        fn root_revision(&self) -> SyncResult<Digest> {
            Ok(digest(b"new root"))
        }
        fn current_source(&self) -> SyncResult<SourceGuard> {
            self.0.current_source()
        }
        fn target_digest(&self, s: &str) -> SyncResult<Digest> {
            self.0.target_digest(s)
        }
        fn apply_once(
            &self,
            _: &str,
            _: &[u8],
            _: &PayloadSchemaRef,
            _: &str,
            _: &str,
            _: bool,
        ) -> SyncResult<()> {
            unreachable!()
        }
        fn export_once(
            &self,
            _: &str,
            _: &[u8],
            _: &PayloadSchemaRef,
            _: &str,
        ) -> SyncResult<Uuid> {
            unreachable!()
        }
    }
    let e = Env::new();
    let p = e.plan().await;
    e.remote
        .lose_generation_response
        .store(true, Ordering::SeqCst);
    assert!(e
        .cx()
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone()
        })
        .await
        .is_err());
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let before = e.remote.writes.load(Ordering::SeqCst);
    let mut changed = connection();
    changed.revision = digest(b"changed credential configuration");
    let mut cx = SyncContext::new(&e.store, &e.remote, &e.codec, &e.local, changed).unwrap();
    cx.budgets.get_attempts = 1;
    let req = ResumeUploadRequest {
        operation_id: p.operation_id.clone(),
        plan_digest: p.plan_digest.clone(),
        expected_view_digest: o.view_digest,
    };
    assert_eq!(
        cx.resume_original_upload(req.clone())
            .await
            .unwrap_err()
            .code,
        ErrorCode::SyncEndpointChanged
    );
    let root = DifferentRoot(&e.local);
    let mut cx = SyncContext::new(&e.store, &e.remote, &e.codec, &root, connection()).unwrap();
    cx.budgets.get_attempts = 1;
    assert_eq!(
        cx.resume_original_upload(req).await.unwrap_err().code,
        ErrorCode::SyncPlanChanged
    );
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn corrupt_aggregate_state_does_not_erase_previous_receipt_or_accept_new_baseline() {
    let e = Env::new();
    let f = e.upload().await;
    let binding = connection().binding_id;
    std::fs::write(
        e.dir.path().join(format!("{}.seen.json", &binding[7..])),
        b"{malformed",
    )
    .unwrap();
    let before = e.remote.writes.load(Ordering::SeqCst);
    assert_eq!(
        e.cx()
            .observe_remote(ObserveRequest {
                connection_id: "synthetic".into()
            })
            .await
            .unwrap_err()
            .code,
        ErrorCode::SyncJournalInvalid
    );
    let verified = e
        .cx()
        .verify_original_upload(OperationRequest {
            operation_id: f.operation_id,
        })
        .await
        .unwrap();
    assert_eq!(verified.storage_status, StorageStatus::Stored);
    assert!(verified.failure.is_some());
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn lost_payload_response_resumes_same_id_without_reuploading_frozen_bytes() {
    let e = Env::new();
    let p = e.plan().await;
    e.remote.lose_payload_response.store(true, Ordering::SeqCst);
    let cx = e.cx();
    assert!(cx
        .publish_upload(ConfirmUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest.clone()
        })
        .await
        .is_err());
    assert_eq!(
        e.store
            .journal(&p.operation_id)
            .unwrap()
            .fact
            .last_write_step,
        WriteStep::Payload
    );
    let before = e.remote.writes.load(Ordering::SeqCst);
    let o = cx
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let result = cx
        .resume_original_upload(ResumeUploadRequest {
            operation_id: p.operation_id.clone(),
            plan_digest: p.plan_digest,
            expected_view_digest: o.view_digest,
        })
        .await
        .unwrap();
    assert_eq!(result.operation_id, p.operation_id);
    assert_eq!(result.storage_status, StorageStatus::Stored);
    assert_eq!(before + 1, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn original_verification_distinguishes_descendant_and_local_later_save() {
    let e = Env::new();
    let original = e.upload().await;
    let g = original.generation.clone().unwrap();
    seed_commit(
        &e.remote,
        &g,
        vec![original.commit.clone().unwrap()],
        b"successor",
    );
    e.local.source.lock().unwrap().saved_source_digest = digest(b"new local save");
    let before = e.remote.writes.load(Ordering::SeqCst);
    let cx = e.cx();
    let verified = cx
        .verify_original_upload(OperationRequest {
            operation_id: original.operation_id.clone(),
        })
        .await
        .unwrap();
    assert_eq!(verified.storage_status, StorageStatus::Stored);
    assert_eq!(verified.relation.remote, RemoteRelation::Descendant);
    assert_eq!(verified.relation.local, LocalRelation::NotCompared);
    let relation = cx
        .assess_current_relation(OperationRequest {
            operation_id: original.operation_id,
        })
        .unwrap();
    assert_eq!(relation.local, LocalRelation::Changed);
    assert_eq!(relation.remote, RemoteRelation::Descendant);
    assert_eq!(before, e.remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn authentication_failure_is_not_empty_or_an_unbound_original_verify_action() {
    let e = Env::new();
    *e.remote.fail_list.lock().unwrap() = Some(TransportKind::Unauthorized);
    let err = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SyncAuthRequired);
    assert_eq!(err.cause, Some(Cause::Unauthorized));
    *e.remote.fail_list.lock().unwrap() = Some(TransportKind::Timeout);
    let err = e
        .cx()
        .prepare_upload(PrepareUploadRequest {
            connection_id: "synthetic".into(),
            expected_view_digest: digest(b"unknown"),
            source_guard: e.local.current_source().unwrap(),
            initialize_on_confirm: true,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SyncNetwork);
    assert!(!err.recovery_actions.iter().any(|a| matches!(
        a.kind,
        RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
    )));
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn corrupted_metadata_with_duplicate_fields_never_admits_a_root() {
    let e = Env::new();
    let g = new_id();
    let json = format!(
        r#"{{"format":"OCXD_CONFIG_GENERATION","formatVersion":1,"generationId":"{g}","generationId":"{g}"}}"#
    );
    e.remote.files.lock().unwrap().insert(
        SyncRelativePath::generation(&g).unwrap().as_str().into(),
        json.into_bytes(),
    );
    let o = e
        .cx()
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Invalid);
    assert_eq!(e.remote.writes.load(Ordering::SeqCst), 0);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_conflict_resolutions_form_two_new_heads_and_keep_original_branches() {
    let e = Env::new();
    let g = seed_generation(&e.remote);
    let a = seed_commit(&e.remote, &g, vec![], b"choice A");
    let b = seed_commit(&e.remote, &g, vec![], b"choice B");
    let original = e.remote.files.lock().unwrap().clone();
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path()).unwrap();
    let codec = Codec::new();
    let local = Local::new();
    let first = e.cx();
    let mut second = SyncContext::new(&store, &e.remote, &codec, &local, connection()).unwrap();
    second.budgets.get_attempts = 1;
    let o = first
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    let make = |head| PrepareResolutionRequest {
        connection_id: "synthetic".into(),
        generation: g.clone(),
        heads: o.heads.clone(),
        expected_view_digest: o.view_digest.clone(),
        source_guard: e.local.current_source().unwrap(),
        resolution: Resolution {
            selection_mode: SelectionMode::Head,
            selected_head: Some(head),
            scope_digest: digest(b"scope"),
            preview_digest: digest(b"preview both"),
        },
    };
    let p1 = first.prepare_resolution(make(a.clone())).await.unwrap();
    let p2 = second.prepare_resolution(make(b.clone())).await.unwrap();
    *e.remote.commit_barrier.lock().unwrap() = Some(Arc::new(tokio::sync::Barrier::new(2)));
    let (left, right) = tokio::join!(
        first.publish_upload(ConfirmUploadRequest {
            operation_id: p1.operation_id.clone(),
            plan_digest: p1.plan_digest
        }),
        second.publish_upload(ConfirmUploadRequest {
            operation_id: p2.operation_id.clone(),
            plan_digest: p2.plan_digest
        })
    );
    assert_eq!(left.unwrap().storage_status, StorageStatus::Stored);
    assert_eq!(right.unwrap().storage_status, StorageStatus::Stored);
    let o = first
        .observe_remote(ObserveRequest {
            connection_id: "synthetic".into(),
        })
        .await
        .unwrap();
    assert_eq!(o.state, RemoteState::Conflict);
    assert_eq!(o.heads.len(), 2);
    assert!(!o.heads.contains(&a) && !o.heads.contains(&b));
    for (name, bytes) in original {
        assert_eq!(e.remote.files.lock().unwrap().get(&name), Some(&bytes));
    }
}
