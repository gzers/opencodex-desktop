//! Backend-owned synchronization service. Commands are thin delegates; private
//! sessions, original endpoint records, selected scopes and exact graph material
//! never accept UI paths, URLs, public-provider objects or credential references.
use super::{
    file_objects_v2::{self, PublicSummary, SavedObjects},
    objects_overlay_v2::{ChannelSelectionV2, McpSourceSelectionV2, SelectionV2},
    schema::{invalid, ModelResult},
    sync_host_v2::{
        self as host, CaptureProviderV2, DirectionV2, EndpointCredentialsV2, EndpointResolverV2,
        HostPayloadCodecV2, HostPreparationV2, HostSessionV2, OriginalUploadOwnerV2,
        SessionSecretsV2, SystemEndpointCredentialsV2,
    },
    sync_v2::{
        graph,
        store::{self, Store},
        types::*,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{Mutex as AsyncMutex, Semaphore};
use zeroize::Zeroizing;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceDirectionV2 {
    Upload,
    Download,
}
impl ServiceDirectionV2 {
    fn host(&self) -> DirectionV2 {
        match self {
            Self::Upload => DirectionV2::Upload,
            Self::Download => DirectionV2::Download,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviewRequestV2 {
    pub selection: SelectionV2,
    pub mcp_clients: BTreeMap<String, String>,
    pub assets_reviewed: bool,
    pub selected_head: Option<CommitRef>,
    pub forced_direction: Option<ServiceDirectionV2>,
    pub backup_before_update: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmRequestV2 {
    pub session_id: String,
    pub plan_digest: Digest,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeRequestV2 {
    pub selection: SelectionV2,
    pub mcp_clients: BTreeMap<String, String>,
    pub assets_reviewed: bool,
}
/// Invoke may accept only this operation-specific secret argument, converted to
/// Zeroizing before entering the owner. It has no Serialize/Debug and is never
/// put in a scope/session/journal record.
#[derive(Default)]
pub struct ServiceSecretsV2 {
    pub read_passphrase: Option<Zeroizing<String>>,
    pub write_passphrase: Option<Zeroizing<String>>,
}
fn copy_secret(s: &Option<Zeroizing<String>>) -> Option<Zeroizing<String>> {
    s.as_ref().map(|v| Zeroizing::new(v.to_string()))
}
impl ServiceSecretsV2 {
    fn host(&self) -> SessionSecretsV2 {
        SessionSecretsV2 {
            read_passphrase: copy_secret(&self.read_passphrase),
            write_passphrase: copy_secret(&self.write_passphrase),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKindV2 {
    Channel,
    Template,
    Preferences,
    Extensions,
    Account,
    Skill,
    Mcp,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObjectEntryV2 {
    pub kind: ObjectKindV2,
    pub id: String,
    pub label: String,
    pub available_local: bool,
    pub available_cloud: bool,
    pub model_ids: Vec<String>,
    pub models: Vec<ModelInventoryV2>,
    pub family_ids: Vec<String>,
    pub revisions: Vec<super::schema_v2::RefV2>,
    pub default_revision: Option<super::schema_v2::RefV2>,
    pub secret_available: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelInventoryV2 {
    pub raw_model_id: String,
    pub label: String,
    pub observed: bool,
    pub selected: bool,
    pub available_local: bool,
    pub available_cloud: bool,
    pub family_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpSourceOptionV2 {
    pub source: String,
    pub definition_digest: Digest,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogRequestV2 {
    pub read_passphrase: Option<String>,
}
impl Drop for CatalogRequestV2 {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        if let Some(s) = &mut self.read_passphrase {
            s.zeroize();
        }
    }
}
impl CatalogRequestV2 {
    pub fn take_secret(&mut self) -> Option<Zeroizing<String>> {
        self.read_passphrase.take().map(Zeroizing::new)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpSourceEntryV2 {
    pub name: String,
    pub sources: Vec<McpSourceOptionV2>,
    pub differing_fields: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogV2 {
    pub connection_state: String,
    pub entries: Vec<ObjectEntryV2>,
    pub mcp_sources: Vec<McpSourceEntryV2>,
    pub selected: ScopeRequestV2,
    pub initialized: bool,
    pub cloud_state: RemoteState,
    pub heads: Vec<CommitRef>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub observation: RemoteObservation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatusV2 {
    Add,
    Update,
    Unchanged,
    Unavailable,
    SourceRequired,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiffEntryV2 {
    pub kind: ObjectKindV2,
    pub id: String,
    pub status: DiffStatusV2,
    pub before_digest: Option<Digest>,
    pub after_digest: Option<Digest>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "plan", rename_all = "snake_case")]
pub enum ServicePlanV2 {
    NoChange,
    Upload(Box<UploadPlan>),
    Download(Box<DownloadPlan>),
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewV2 {
    pub session_id: String,
    pub plan_digest: Digest,
    pub chosen_direction: DirectionV2,
    pub automatic_reason: host::AutomaticReasonV2,
    pub local_saved_time: Option<String>,
    pub cloud_saved_time: Option<String>,
    pub no_change: bool,
    pub backup_before_update: bool,
    pub entries: Vec<DiffEntryV2>,
    pub local_summary: PublicSummary,
    pub cloud_summary: Option<PublicSummary>,
    pub plan: ServicePlanV2,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumePreflightV2 {
    pub request: ResumeUploadRequest,
    pub can_resume: bool,
    pub reason: String,
    pub observation: RemoteObservation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryRequestV2 {
    pub selection: SelectionV2,
    pub mcp_clients: BTreeMap<String, String>,
    pub assets_reviewed: bool,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub mode: RecoveryMode,
    pub backup_before_update: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryPreviewV2 {
    pub session_id: String,
    pub plan: RecoveryPlan,
    pub entries: Vec<DiffEntryV2>,
}
/// Capture inventory and per-plan composition belong to the SAME saved/identity/
/// receive owner. The production adapter below is added only against L0's fixed
/// committed modules. Injection is also used for isolated deterministic failures.
pub struct InventoryV2 {
    pub objects: SavedObjects,
    pub mcp_sources: Vec<McpSourceEntryV2>,
    pub key_channel_ids: BTreeSet<String>,
    pub authorization_channel_ids: BTreeSet<String>,
    pub credential_skill_names: BTreeSet<String>,
    pub credential_mcp_names: BTreeSet<String>,
}
pub struct CompositionV2 {
    pub provider: Arc<dyn CaptureProviderV2>,
    pub local: Arc<dyn LocalApplication>,
    pub remote_scope: SelectionV2,
    pub diff: Vec<DiffEntryV2>,
}
pub trait ServiceBackendV2: Send + Sync {
    fn inventory(&self, connection: &ConnectionHandle) -> ModelResult<InventoryV2>;
    fn compose(
        &self,
        request: &PreviewRequestV2,
        connection: &ConnectionHandle,
        cloud: Option<&SavedObjects>,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> ModelResult<CompositionV2>;
    /// Id is the one produced by core.prepare_download. The production bridge
    /// must call MappingPreview.after_adoption(id) lazily, never preallocate a
    /// second ID or rebuild mappings. Upload/no-change need no receive plan.
    fn prepared_operation(
        &self,
        _operation_id: &str,
        _composition: &CompositionV2,
    ) -> ModelResult<()> {
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScopeRecordV2 {
    version: u32,
    root_digest: Digest,
    initialized: bool,
    request: ScopeRequestV2,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OriginalRecordV2 {
    version: u32,
    root_digest: Digest,
    operation_id: String,
    endpoint: crate::modules::sync::config::SyncEndpointConfig,
    request: ScopeRequestV2,
    handle: StoredHandleV2,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredHandleV2 {
    id: String,
    binding_id: Digest,
    revision: Digest,
}
impl From<&ConnectionHandle> for StoredHandleV2 {
    fn from(h: &ConnectionHandle) -> Self {
        Self {
            id: h.id.clone(),
            binding_id: h.binding_id.clone(),
            revision: h.revision.clone(),
        }
    }
}
impl StoredHandleV2 {
    fn handle(&self) -> ConnectionHandle {
        ConnectionHandle {
            id: self.id.clone(),
            binding_id: self.binding_id.clone(),
            revision: self.revision.clone(),
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedRecordV2<T> {
    body: T,
    digest: Digest,
}
struct ServiceFilesV2 {
    directory: PathBuf,
    root_digest: Digest,
}
impl ServiceFilesV2 {
    fn new(root: &Path) -> SyncResult<Self> {
        if !root.is_absolute() || !root.is_dir() {
            return Err(failure(ErrorCode::SyncLocalStateIo, Step::Persist));
        }
        let directory = root.join("manager-state/sync-service-v2");
        for path in [root.join("manager-state"), directory.clone()] {
            if let Ok(m) = std::fs::symlink_metadata(&path) {
                if m.file_type().is_symlink() || !m.is_dir() {
                    return Err(failure(ErrorCode::SyncLocalStateIo, Step::Persist));
                }
            }
        }
        std::fs::create_dir_all(&directory)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
        let root_digest = store::digest(
            root.canonicalize()
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?
                .to_string_lossy()
                .as_bytes(),
        );
        Ok(Self {
            directory,
            root_digest,
        })
    }
    fn path(&self, name: &str) -> SyncResult<PathBuf> {
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return Err(failure(ErrorCode::SyncLocalStateIo, Step::Persist));
        }
        let p = self.directory.join(name);
        if let Ok(m) = std::fs::symlink_metadata(&p) {
            if !m.is_file() || m.file_type().is_symlink() {
                return Err(failure(ErrorCode::SyncLocalStateIo, Step::Persist));
            }
        }
        Ok(p)
    }
    fn read<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        name: &str,
    ) -> SyncResult<Option<T>> {
        let p = self.path(name)?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let file = match options.open(p) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(failure(ErrorCode::SyncLocalStateIo, Step::Persist)),
        };
        use std::io::Read;
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(failure(ErrorCode::SyncLimit, Step::Persist));
        }
        let value = super::schema_v2::parse_strict_json_v2(&bytes)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        let r: CheckedRecordV2<T> = serde_json::from_value(value)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        if store::digest(&store::canonical(&r.body)?) != r.digest {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        Ok(Some(r.body))
    }
    fn write<T: Serialize>(&self, name: &str, body: T) -> SyncResult<()> {
        let bytes = store::canonical(&CheckedRecordV2 {
            digest: store::digest(&store::canonical(&body)?),
            body,
        })?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(failure(ErrorCode::SyncLimit, Step::Persist));
        }
        let target = self.path(name)?;
        let mut f = tempfile::NamedTempFile::new_in(&self.directory)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
        use std::io::Write;
        f.write_all(&bytes)
            .and_then(|_| f.as_file().sync_all())
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
        f.persist(target)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
        Ok(())
    }
}
struct SessionV2 {
    host: Arc<HostSessionV2>,
    local: Arc<dyn LocalApplication>,
    plan: ServicePlanV2,
    digest: Digest,
    operation_id: Option<String>,
    gate: AsyncMutex<()>,
    cancelled: AtomicBool,
    connection: ConnectionHandle,
}
struct RecoverySessionV2 {
    plan: RecoveryPlan,
    connection: ConnectionHandle,
    local: Arc<dyn LocalApplication>,
    passphrase: Option<Zeroizing<String>>,
    gate: AsyncMutex<()>,
}
struct LimitedTransportV2 {
    inner: Arc<dyn ConfigSyncTransport>,
    slots: Semaphore,
}
#[async_trait::async_trait]
impl ConfigSyncTransport for LimitedTransportV2 {
    async fn list_children(
        &self,
        c: &ConnectionHandle,
        l: ListLimits,
    ) -> Result<ChildListing, SyncTransportError> {
        let _permit = self.slots.acquire().await.map_err(|_| SyncTransportError {
            kind: TransportKind::Protocol,
            http_status: None,
            delivery: Delivery::NotSent,
        })?;
        self.inner.list_children(c, l).await
    }
    async fn get_file(
        &self,
        c: &ConnectionHandle,
        p: &SyncRelativePath,
        l: ReadLimits,
    ) -> Result<FileRead, SyncTransportError> {
        let _permit = self.slots.acquire().await.map_err(|_| SyncTransportError {
            kind: TransportKind::Protocol,
            http_status: None,
            delivery: Delivery::NotSent,
        })?;
        self.inner.get_file(c, p, l).await
    }
    async fn put_file(
        &self,
        c: &ConnectionHandle,
        p: &SyncRelativePath,
        b: &[u8],
        l: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError> {
        let _permit = self.slots.acquire().await.map_err(|_| SyncTransportError {
            kind: TransportKind::Protocol,
            http_status: None,
            delivery: Delivery::NotSent,
        })?;
        self.inner.put_file(c, p, b, l).await
    }
}
pub struct SyncServiceV2 {
    root: PathBuf,
    backend: Arc<dyn ServiceBackendV2>,
    resolver: Arc<EndpointResolverV2>,
    transport: Arc<dyn ConfigSyncTransport>,
    file_transport: Option<Arc<crate::infrastructure::webdav_config_v2::FileWebDavTransport>>,
    store: Arc<Store>,
    files: Arc<ServiceFilesV2>,
    sessions: Mutex<BTreeMap<String, Arc<SessionV2>>>,
    recoveries: Mutex<BTreeMap<String, Arc<RecoverySessionV2>>>,
    slots: Arc<Semaphore>,
}
fn err(code: ErrorCode, step: Step) -> SyncFailure {
    let mut e = failure(code, step);
    e.recovery_actions.retain(|a| {
        !matches!(
            a.kind,
            RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
        )
    });
    e
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> SyncResult<T> + Send + 'static,
) -> SyncResult<T> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| err(ErrorCode::SyncLocalStateIo, Step::Persist))?
}
impl SyncServiceV2 {
    pub fn new(
        root: PathBuf,
        backend: Arc<dyn ServiceBackendV2>,
        transport: Arc<dyn ConfigSyncTransport>,
        credentials: Arc<dyn EndpointCredentialsV2>,
    ) -> SyncResult<Self> {
        let resolver = Arc::new(EndpointResolverV2::new(root.clone(), credentials)?);
        let store = Arc::new(Store::open(
            &root.join("manager-state/sync-v2-transactions"),
        )?);
        let files = Arc::new(ServiceFilesV2::new(&root)?);
        Ok(Self {
            root,
            backend,
            resolver,
            transport: Arc::new(LimitedTransportV2 {
                inner: transport,
                slots: Semaphore::new(4),
            }),
            file_transport: None,
            store,
            files,
            sessions: Default::default(),
            recoveries: Default::default(),
            slots: Arc::new(Semaphore::new(4)),
        })
    }
    pub fn with_file_transport(
        root: PathBuf,
        backend: Arc<dyn ServiceBackendV2>,
        transport: Arc<crate::infrastructure::webdav_config_v2::FileWebDavTransport>,
        credentials: Arc<dyn EndpointCredentialsV2>,
    ) -> SyncResult<Self> {
        let mut service = Self::new(root, backend, transport.clone(), credentials)?;
        service.file_transport = Some(transport);
        Ok(service)
    }
    /// Construct once for the active explicit D/H under L0 storage admission.
    /// The service never reads process HOME or configures the global vault root.
    pub fn production(root: PathBuf, home: PathBuf) -> SyncResult<Self> {
        let backend = Arc::new(
            ProductionServiceBackendV2::new(
                root.clone(),
                home,
                Arc::new(super::saved_objects_capture_v2::SystemCaptureCredentialsV2),
            )
            .map_err(host::map_host_error)?,
        );
        let transport = Arc::new(
            crate::infrastructure::webdav_config_v2::FileWebDavTransport::new()
                .map_err(|e| transport_failure(e, Step::Observe))?,
        );
        Self::with_file_transport(
            root,
            backend,
            transport,
            Arc::new(SystemEndpointCredentialsV2),
        )
    }
    pub fn production_credentials() -> Arc<dyn EndpointCredentialsV2> {
        Arc::new(SystemEndpointCredentialsV2)
    }
    fn session(&self, id: &str) -> SyncResult<Arc<SessionV2>> {
        require_uuid(id)?;
        self.sessions
            .lock()
            .map_err(|_| err(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .get(id)
            .cloned()
            .ok_or_else(|| err(ErrorCode::SyncPlanExpired, Step::SourceGuard))
    }
    async fn active(
        &self,
    ) -> SyncResult<(
        ConnectionHandle,
        crate::modules::sync::config::SyncEndpointConfig,
    )> {
        let resolver = self.resolver.clone();
        let root = self.root.clone();
        let file = self.file_transport.clone();
        blocking(move || {
            let bound = resolver.resolve_active()?;
            if let Some(file) = file {
                bound.bind_file_transport(&file)?;
            }
            let config = crate::modules::sync::config::SyncConfigStore::new(&root)
                .load()
                .map_err(|_| err(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
            let endpoint = config
                .active()
                .cloned()
                .ok_or_else(|| err(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
            Ok((bound.handle().clone(), endpoint))
        })
        .await
    }
    async fn graph(&self, connection: &ConnectionHandle) -> SyncResult<graph::Graph> {
        let transport = self.transport.clone();
        let connection = connection.clone();
        let slots = self.slots.clone();
        let store = self.store.clone();
        let runtime = tokio::runtime::Handle::current();
        blocking(move || {
            runtime.block_on(graph::observe(
                &graph::Network {
                    transport: transport.as_ref(),
                    connection: &connection,
                    limits: &Budgets::default(),
                    slots: &slots,
                },
                &store,
            ))
        })
        .await
    }
    async fn cloud(
        &self,
        g: &graph::Graph,
        selected: Option<&CommitRef>,
        connection: &ConnectionHandle,
    ) -> SyncResult<Option<PayloadInput>> {
        let h = match g.observation.state {
            RemoteState::Empty | RemoteState::GenerationEmpty => {
                if selected.is_some() {
                    return Err(err(ErrorCode::SyncPlanChanged, Step::Observe));
                }
                return Ok(None);
            }
            RemoteState::Ready => {
                let h = g
                    .observation
                    .heads
                    .first()
                    .ok_or_else(|| err(ErrorCode::SyncRemoteInvalid, Step::Observe))?;
                if selected.is_some_and(|s| s != h) {
                    return Err(err(ErrorCode::SyncPlanChanged, Step::Observe));
                }
                h
            }
            RemoteState::Conflict => selected
                .filter(|h| g.observation.heads.contains(h))
                .ok_or_else(|| err(ErrorCode::SyncRemoteConflict, Step::Observe))?,
            RemoteState::InitializationAmbiguous => {
                return Err(err(ErrorCode::SyncGenerationAmbiguous, Step::Observe))
            }
            RemoteState::Incomplete => {
                return Err(g
                    .observation
                    .problems
                    .first()
                    .cloned()
                    .unwrap_or_else(|| err(ErrorCode::SyncListingIncomplete, Step::Observe)))
            }
            RemoteState::Invalid => return Err(err(ErrorCode::SyncRemoteInvalid, Step::Observe)),
        };
        let generation = g
            .observation
            .generation
            .as_ref()
            .ok_or_else(|| err(ErrorCode::SyncRemoteInvalid, Step::Observe))?;
        let node = g.node(generation, h)?;
        let budgets = Budgets::default();
        let bytes = graph::Network {
            transport: self.transport.as_ref(),
            connection,
            limits: &budgets,
            slots: &self.slots,
        }
        .get_retry(
            &SyncRelativePath::data(&generation.generation_id, &h.operation_id)?,
            budgets.payload_bytes,
            Step::PayloadGet,
        )
        .await?;
        if store::digest(&bytes) != node.commit.payload_digest
            || bytes.len() as u64 != node.commit.payload_bytes
        {
            return Err(err(ErrorCode::SyncRemoteInvalid, Step::PayloadGet));
        }
        Ok(Some(PayloadInput {
            head: h.clone(),
            bytes,
        }))
    }
    /// Catalog with an encrypted cloud may require only the current read
    /// passphrase. Failure leaves initialized/scope untouched and never writes.
    pub async fn catalog(
        &self,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<CatalogV2> {
        let (connection, _) = self.active().await?;
        let backend = self.backend.clone();
        let inventory_connection = connection.clone();
        let inventory = blocking(move || {
            backend
                .inventory(&inventory_connection)
                .map_err(host::map_host_error)
        })
        .await?;
        let graph = self.graph(&connection).await?;
        let payload = if graph.observation.state == RemoteState::Conflict
            || graph.observation.state == RemoteState::InitializationAmbiguous
        {
            None
        } else {
            self.cloud(&graph, None, &connection).await?
        };
        let cloud = match payload {
            Some(p) => Some(
                blocking(move || {
                    file_objects_v2::decode(
                        &p.bytes,
                        read_passphrase.as_ref().map(|s| s.as_str()),
                        &file_objects_v2::supported_schema(),
                        Some(&host::PayloadAuthorizationV2),
                    )
                    .map(|d| d.objects)
                    .map_err(host::map_host_error)
                })
                .await?,
            ),
            None => None,
        };
        let mut entries = entries_from(&inventory.objects, true);
        for row in &mut entries {
            if row.kind == ObjectKindV2::Channel {
                row.secret_available |= inventory.key_channel_ids.contains(&row.id)
                    || inventory.authorization_channel_ids.contains(&row.id);
            }
        }
        for source in &inventory.mcp_sources {
            if !entries
                .iter()
                .any(|e| e.kind == ObjectKindV2::Mcp && e.id == source.name)
            {
                entries.push(ObjectEntryV2 {
                    kind: ObjectKindV2::Mcp,
                    id: source.name.clone(),
                    label: source.name.clone(),
                    available_local: true,
                    available_cloud: false,
                    model_ids: vec![],
                    models: vec![],
                    family_ids: vec![],
                    revisions: vec![],
                    default_revision: None,
                    secret_available: inventory.credential_mcp_names.contains(&source.name),
                });
            }
        }
        for name in &inventory.credential_skill_names {
            if !entries
                .iter()
                .any(|e| e.kind == ObjectKindV2::Skill && e.id == *name)
            {
                entries.push(ObjectEntryV2 {
                    kind: ObjectKindV2::Skill,
                    id: name.clone(),
                    label: name.clone(),
                    available_local: true,
                    available_cloud: false,
                    model_ids: vec![],
                    models: vec![],
                    family_ids: vec![],
                    revisions: vec![],
                    default_revision: None,
                    secret_available: true,
                });
            }
        }

        if let Some(cloud) = &cloud {
            for remote in entries_from(cloud, false) {
                if let Some(old) = entries
                    .iter_mut()
                    .find(|e| e.kind == remote.kind && e.id == remote.id)
                {
                    old.available_cloud = true;
                    old.secret_available |= remote.secret_available;
                    for model in remote.models {
                        if let Some(existing) = old
                            .models
                            .iter_mut()
                            .find(|m| m.raw_model_id == model.raw_model_id)
                        {
                            existing.available_cloud = true;
                        } else {
                            old.models.push(model);
                        }
                    }
                    old.model_ids = old.models.iter().map(|m| m.raw_model_id.clone()).collect();
                    old.family_ids.extend(remote.family_ids);
                    old.family_ids.sort();
                    old.family_ids.dedup();
                    for reference in remote.revisions {
                        if let Some(old_ref) = old
                            .revisions
                            .iter()
                            .find(|r| r.revision_id == reference.revision_id)
                        {
                            if old_ref.digest != reference.digest {
                                return Err(err(ErrorCode::SyncPayloadInvalid, Step::Decode));
                            }
                        } else {
                            old.revisions.push(reference);
                        }
                    }
                    if old.default_revision.is_none() {
                        old.default_revision = remote.default_revision;
                    }
                } else {
                    entries.push(remote);
                }
            }
        }
        let saved = self.read_scope().await?;
        let initialized = saved.is_some();
        let selected = match saved {
            Some(s) => s,
            None => {
                let mut request = initial_scope(&inventory.objects, cloud.as_ref());
                for channel in &mut request.selection.channels {
                    if !channel.include_key && !channel.include_authorization {
                        if inventory
                            .authorization_channel_ids
                            .contains(&channel.portable_channel_id)
                        {
                            channel.include_authorization = true;
                        } else if inventory
                            .key_channel_ids
                            .contains(&channel.portable_channel_id)
                        {
                            channel.include_key = true;
                        }
                    }
                }
                for name in &inventory.credential_skill_names {
                    if !request.selection.skill_names.contains(name) {
                        request.selection.skill_names.push(name.clone());
                    }
                    if !request
                        .selection
                        .extension_credentials
                        .skill_names
                        .contains(name)
                    {
                        request
                            .selection
                            .extension_credentials
                            .skill_names
                            .push(name.clone());
                    }
                }
                for name in &inventory.credential_mcp_names {
                    if request
                        .selection
                        .mcp_sources
                        .iter()
                        .any(|m| m.name == *name)
                        && !request
                            .selection
                            .extension_credentials
                            .mcp_names
                            .contains(name)
                    {
                        request
                            .selection
                            .extension_credentials
                            .mcp_names
                            .push(name.clone());
                    }
                }
                for mcp in &inventory.mcp_sources {
                    if mcp.differing_fields.is_empty() {
                        if let Some(source) = mcp.sources.first() {
                            if request
                                .selection
                                .mcp_sources
                                .iter()
                                .any(|m| m.name == mcp.name)
                            {
                                request
                                    .mcp_clients
                                    .insert(mcp.name.clone(), source.source.clone());
                            }
                        }
                    }
                }

                // A conflict/ambiguous initialization is not a successful initial
                // directory read and cannot persist a truncated initial selection.
                if !matches!(
                    graph.observation.state,
                    RemoteState::Conflict | RemoteState::InitializationAmbiguous
                ) {
                    self.save_scope(request.clone()).await?;
                }
                request
            }
        };
        Ok(CatalogV2 {
            connection_state: "ready".into(),
            entries,
            mcp_sources: inventory.mcp_sources,
            selected,
            initialized: initialized
                || !matches!(
                    graph.observation.state,
                    RemoteState::Conflict | RemoteState::InitializationAmbiguous
                ),
            cloud_state: graph.observation.state,
            heads: graph.observation.heads.clone(),
            observation: graph.observation.clone(),
            recovery_actions: graph.observation.recovery_actions,
        })
    }
    pub async fn read_scope(&self) -> SyncResult<Option<ScopeRequestV2>> {
        let files = self.files.clone();
        blocking(move || {
            let record: Option<ScopeRecordV2> = files.read("scope.json")?;
            match record {
                Some(r)
                    if r.version == 2 && r.root_digest == files.root_digest && r.initialized =>
                {
                    Ok(Some(r.request))
                }
                Some(_) => Err(err(ErrorCode::SyncJournalInvalid, Step::Persist)),
                None => Ok(None),
            }
        })
        .await
    }
    pub async fn save_scope(&self, request: ScopeRequestV2) -> SyncResult<()> {
        if request.selection.usage != super::objects_overlay_v2::UsageV2::RemoteSync {
            return Err(err(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        let files = self.files.clone();
        blocking(move || {
            files.write(
                "scope.json",
                ScopeRecordV2 {
                    version: 2,
                    root_digest: files.root_digest.clone(),
                    initialized: true,
                    request,
                },
            )
        })
        .await
    }
    pub async fn preview(
        &self,
        request: PreviewRequestV2,
        secrets: ServiceSecretsV2,
    ) -> SyncResult<PreviewV2> {
        tokio::time::timeout(
            Duration::from_secs(90),
            self.preview_inner(request, secrets),
        )
        .await
        .map_err(|_| err(ErrorCode::SyncNetwork, Step::Observe))?
    }
    async fn preview_inner(
        &self,
        request: PreviewRequestV2,
        secrets: ServiceSecretsV2,
    ) -> SyncResult<PreviewV2> {
        let (connection, endpoint) = self.active().await?;
        let graph = self.graph(&connection).await?;
        let cloud = self
            .cloud(&graph, request.selected_head.as_ref(), &connection)
            .await?;
        let backend = self.backend.clone();
        let request_copy = request.clone();
        let conn = connection.clone();
        let read = copy_secret(&secrets.read_passphrase);
        let cloud_bytes = cloud.as_ref().map(|p| Zeroizing::new(p.bytes.clone()));
        let composition = blocking(move || {
            let decoded = cloud_bytes
                .map(|b| {
                    file_objects_v2::decode(
                        &b,
                        read.as_ref().map(|s| s.as_str()),
                        &file_objects_v2::supported_schema(),
                        Some(&host::PayloadAuthorizationV2),
                    )
                    .map(|d| d.objects)
                    .map_err(host::map_host_error)
                })
                .transpose()?;
            backend
                .compose(&request_copy, &conn, decoded.as_ref(), read)
                .map_err(host::map_host_error)
        })
        .await?;
        let provider = composition.provider.clone();
        let selection = composition.remote_scope.clone();
        let host_secrets = secrets.host();
        let forced = request.forced_direction.clone().map(|d| d.host());
        let cloud_copy = cloud.as_ref().map(|p| PayloadInput {
            head: p.head.clone(),
            bytes: p.bytes.clone(),
        });
        let preview = blocking(move || {
            host::preview_semantics(
                provider.as_ref(),
                selection,
                cloud_copy
                    .as_ref()
                    .map(host::CloudMaterialV2::Valid)
                    .unwrap_or(host::CloudMaterialV2::Empty),
                &host_secrets,
                forced,
            )
        })
        .await?;
        let public = preview.decision.clone();
        let codec = HostPayloadCodecV2::new(composition.provider.clone(), preview, secrets.host());
        let bound = {
            let resolver = self.resolver.clone();
            let h = connection.clone();
            blocking(move || resolver.resolve_original(&h)).await?
        };
        let host = Arc::new(HostSessionV2::new(codec, &bound, graph.observation)?);
        let host_prepare = host.clone();
        let resolver = self.resolver.clone();
        let store = self.store.clone();
        let transport = self.transport.clone();
        let local = composition.local.clone();
        let runtime = tokio::runtime::Handle::current();
        let backup = request.backup_before_update;
        let prepared = blocking(move || {
            runtime.block_on(host_prepare.prepare_with_backup(
                &resolver,
                &store,
                transport.as_ref(),
                local.as_ref(),
                backup,
            ))
        })
        .await?;
        let (plan, operation_id, plan_digest) = match prepared {
            HostPreparationV2::NoChange(_) => (
                ServicePlanV2::NoChange,
                None,
                store::digest(&store::canonical(&request)?),
            ),
            HostPreparationV2::Upload(p) => {
                let id = p.operation_id.clone();
                let d = p.plan_digest.clone();
                (ServicePlanV2::Upload(Box::new(p)), Some(id), d)
            }
            HostPreparationV2::Download(p) => {
                let id = p.operation_id.clone();
                let d = p.plan_digest.clone();
                (ServicePlanV2::Download(Box::new(p)), Some(id), d)
            }
        };
        if let Some(id) = &operation_id {
            let backend = self.backend.clone();
            let comp = CompositionV2 {
                provider: composition.provider.clone(),
                local: composition.local.clone(),
                remote_scope: composition.remote_scope.clone(),
                diff: vec![],
            };
            let id = id.clone();
            blocking(move || {
                backend
                    .prepared_operation(&id, &comp)
                    .map_err(host::map_host_error)
            })
            .await?;
            let record = OriginalRecordV2 {
                version: 2,
                root_digest: self.files.root_digest.clone(),
                operation_id: id_for_record(&operation_id)?,
                endpoint,
                request: ScopeRequestV2 {
                    selection: request.selection.clone(),
                    mcp_clients: request.mcp_clients.clone(),
                    assets_reviewed: request.assets_reviewed,
                },
                handle: StoredHandleV2::from(&connection),
            };
            let files = self.files.clone();
            let id = id_for_record(&operation_id)?;
            blocking(move || files.write(&format!("{id}.original.json"), record)).await?;
        }
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(SessionV2 {
            host,
            local: composition.local,
            plan: plan.clone(),
            digest: plan_digest.clone(),
            operation_id,
            gate: AsyncMutex::new(()),
            cancelled: AtomicBool::new(false),
            connection,
        });
        self.sessions
            .lock()
            .map_err(|_| err(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .insert(session_id.clone(), session);
        Ok(PreviewV2 {
            session_id,
            plan_digest,
            chosen_direction: public.chosen_direction,
            automatic_reason: public.automatic_reason,
            local_saved_time: public.local_saved_time,
            cloud_saved_time: public.cloud_saved_time,
            no_change: public.no_change,
            backup_before_update: request.backup_before_update,
            entries: composition.diff,
            local_summary: public.local_summary,
            cloud_summary: public.cloud_summary,
            plan,
        })
    }
    pub async fn confirm(&self, request: ConfirmRequestV2) -> SyncResult<OperationFact> {
        let session = self.session(&request.session_id)?;
        let _guard = session.gate.lock().await;
        if session.digest != request.plan_digest || session.cancelled.load(Ordering::SeqCst) {
            return Err(err(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let resolver = self.resolver.clone();
        let store = self.store.clone();
        let transport = self.transport.clone();
        let runtime = tokio::runtime::Handle::current();
        let owned = session.clone();
        blocking(move || {
            runtime.block_on(async {
                match &owned.plan {
                    ServicePlanV2::Upload(p) => {
                        owned
                            .host
                            .publish_upload(
                                ConfirmUploadRequest {
                                    operation_id: p.operation_id.clone(),
                                    plan_digest: p.plan_digest.clone(),
                                },
                                &resolver,
                                &store,
                                transport.as_ref(),
                                owned.local.as_ref(),
                            )
                            .await
                    }
                    ServicePlanV2::Download(p) => {
                        owned
                            .host
                            .apply_download(
                                ConfirmDownloadRequest {
                                    operation_id: p.operation_id.clone(),
                                    plan_digest: p.plan_digest.clone(),
                                },
                                &resolver,
                                &store,
                                transport.as_ref(),
                                owned.local.as_ref(),
                            )
                            .await
                    }
                    ServicePlanV2::NoChange => {
                        Err(err(ErrorCode::SyncPlanChanged, Step::SourceGuard))
                    }
                }
            })
        })
        .await
    }
    pub async fn result(&self, operation_id: String) -> SyncResult<OperationFact> {
        require_uuid(&operation_id)?;
        let store = self.store.clone();
        blocking(move || match store.journal(&operation_id) {
            Ok(j) => Ok(j.fact),
            Err(e) => store.receive(&operation_id)?.fact.ok_or(e),
        })
        .await
    }
    pub async fn verify(
        &self,
        operation_id: String,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<OperationFact> {
        require_uuid(&operation_id)?;
        let files = self.files.clone();
        let resolver = self.resolver.clone();
        let file = self.file_transport.clone();
        let id = operation_id.clone();
        let owner = blocking(move || {
            let r: OriginalRecordV2 = files
                .read(&format!("{id}.original.json"))?
                .ok_or_else(|| err(ErrorCode::SyncEndpointChanged, Step::Verify))?;
            if r.version != 2 || r.root_digest != files.root_digest || r.operation_id != id {
                return Err(err(ErrorCode::SyncJournalInvalid, Step::Verify));
            }
            let bound = resolver.restore_original(r.endpoint, &r.handle.handle())?;
            if let Some(file) = file {
                bound.bind_file_transport(&file)?;
            }
            Ok(OriginalUploadOwnerV2::new(&bound, read_passphrase))
        })
        .await?;
        let resolver = self.resolver.clone();
        let store = self.store.clone();
        let transport = self.transport.clone();
        let runtime = tokio::runtime::Handle::current();
        blocking(move || {
            runtime.block_on(owner.verify(
                OperationRequest { operation_id },
                &resolver,
                &store,
                transport.as_ref(),
                &HistoricalOnlyLocal,
            ))
        })
        .await
    }
    pub async fn resume_preflight(
        &self,
        operation_id: String,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<ResumePreflightV2> {
        require_uuid(&operation_id)?;
        let store = self.store.clone();
        let id = operation_id.clone();
        let j = blocking(move || store.journal(&id)).await?;
        let files = self.files.clone();
        let id = operation_id.clone();
        let r: OriginalRecordV2 = blocking(move || {
            files
                .read(&format!("{id}.original.json"))?
                .ok_or_else(|| err(ErrorCode::SyncEndpointChanged, Step::Verify))
        })
        .await?;
        if r.root_digest != self.files.root_digest
            || r.operation_id != operation_id
            || r.version != 2
        {
            return Err(err(ErrorCode::SyncJournalInvalid, Step::Verify));
        }
        let handle = r.handle.handle();
        let expected = handle.clone();
        let resolver = self.resolver.clone();
        let file = self.file_transport.clone();
        blocking(move || {
            let bound = resolver.restore_original(r.endpoint, &expected)?;
            if let Some(file) = file {
                bound.bind_file_transport(&file)?;
            }
            Ok(())
        })
        .await?;
        let graph = self.graph(&handle).await?;
        let request = ResumeUploadRequest {
            operation_id: operation_id.clone(),
            plan_digest: j.plan.plan_digest.clone(),
            expected_view_digest: graph.observation.view_digest.clone(),
        };
        let mut reason = String::new();
        if j.fact.storage_status == StorageStatus::Stored {
            reason = "already_stored".into();
        } else if graph.observation.state == RemoteState::InitializationAmbiguous {
            reason = "initialization_ambiguous".into();
        } else if graph.observation.generation.as_ref() != j.fact.generation.as_ref() {
            reason = "generation_changed".into();
        } else if graph.observation.heads != j.plan.parents {
            reason = "remote_changed".into();
        } else if self
            .store
            .payload(&j.payload_handle, &j.plan.payload_digest)
            .is_err()
        {
            reason = "original_material_missing".into();
        }
        if reason.is_empty() {
            let parent = j
                .plan
                .resolution
                .as_ref()
                .and_then(|r| r.selected_head.as_ref())
                .or(j.plan.parents.first());
            let bytes = if let Some(parent) = parent {
                let g = j
                    .fact
                    .generation
                    .as_ref()
                    .ok_or_else(|| err(ErrorCode::SyncGenerationChanged, Step::Verify))?;
                let node = graph.node(g, parent)?;
                let limits = Budgets::default();
                let bytes = graph::Network {
                    transport: self.transport.as_ref(),
                    connection: &handle,
                    limits: &limits,
                    slots: &self.slots,
                }
                .get_retry(
                    &SyncRelativePath::data(&g.generation_id, &parent.operation_id)?,
                    limits.payload_bytes,
                    Step::PayloadGet,
                )
                .await?;
                if store::digest(&bytes) != node.commit.payload_digest
                    || bytes.len() as u64 != node.commit.payload_bytes
                {
                    return Err(err(ErrorCode::SyncRemoteInvalid, Step::PayloadGet));
                }
                Some(Zeroizing::new(bytes))
            } else {
                None
            };
            let backend = self.backend.clone();
            let conn = handle.clone();
            let old = r.request;
            let trusted = blocking(move || {
                let cloud = bytes
                    .map(|b| {
                        file_objects_v2::decode(
                            &b,
                            read_passphrase.as_ref().map(|s| s.as_str()),
                            &file_objects_v2::supported_schema(),
                            Some(&host::PayloadAuthorizationV2),
                        )
                        .map(|d| d.objects)
                        .map_err(host::map_host_error)
                    })
                    .transpose()?;
                backend
                    .compose(
                        &PreviewRequestV2 {
                            selection: old.selection,
                            mcp_clients: old.mcp_clients,
                            assets_reviewed: old.assets_reviewed,
                            selected_head: None,
                            forced_direction: Some(ServiceDirectionV2::Upload),
                            backup_before_update: true,
                        },
                        &conn,
                        cloud.as_ref(),
                        read_passphrase,
                    )
                    .map_err(host::map_host_error)
            })
            .await;
            match trusted {
                Ok(c) => {
                    if c.local.root_revision()? != j.root_revision
                        || c.local.current_source()? != j.plan.source_guard
                    {
                        reason = "source_changed".into();
                    }
                }
                Err(_) => reason = "source_changed".into(),
            }
        }
        let can_resume = reason.is_empty();
        if can_resume {
            reason = "ready".into();
        }
        Ok(ResumePreflightV2 {
            request,
            can_resume,
            reason,
            observation: graph.observation,
        })
    }
    /// Only the initialization-ambiguous graph's verified candidates are legal;
    /// no automatic winner/latest and no remote clear/move is performed.
    pub async fn recovery_preview(
        &self,
        request: RecoveryRequestV2,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<RecoveryPreviewV2> {
        let (connection, _) = self.active().await?;
        let graph = self.graph(&connection).await?;
        if graph.observation.state != RemoteState::InitializationAmbiguous
            || !graph
                .observation
                .recovery_candidates
                .iter()
                .any(|c| c.generation == request.generation && c.head == request.head)
        {
            return Err(err(ErrorCode::SyncGenerationAmbiguous, Step::Recovery));
        }
        let node = graph.node(&request.generation, &request.head)?;
        let limits = Budgets::default();
        let bytes = graph::Network {
            transport: self.transport.as_ref(),
            connection: &connection,
            limits: &limits,
            slots: &self.slots,
        }
        .get_retry(
            &SyncRelativePath::data(
                &request.generation.generation_id,
                &request.head.operation_id,
            )?,
            limits.payload_bytes,
            Step::PayloadGet,
        )
        .await?;
        if store::digest(&bytes) != node.commit.payload_digest
            || bytes.len() as u64 != node.commit.payload_bytes
        {
            return Err(err(ErrorCode::SyncRemoteInvalid, Step::Recovery));
        }
        let backend = self.backend.clone();
        let r = request.clone();
        let conn = connection.clone();
        let pass = copy_secret(&read_passphrase);
        let composition = blocking(move || {
            let cloud = file_objects_v2::decode(
                &bytes,
                pass.as_ref().map(|s| s.as_str()),
                &file_objects_v2::supported_schema(),
                Some(&host::PayloadAuthorizationV2),
            )
            .map_err(host::map_host_error)?;
            backend
                .compose(
                    &PreviewRequestV2 {
                        selection: r.selection,
                        mcp_clients: r.mcp_clients,
                        assets_reviewed: r.assets_reviewed,
                        selected_head: Some(r.head),
                        forced_direction: Some(ServiceDirectionV2::Download),
                        backup_before_update: r.backup_before_update,
                    },
                    &conn,
                    Some(&cloud.objects),
                    pass,
                )
                .map_err(host::map_host_error)
        })
        .await?;
        let current = composition.local.current_source()?;
        let codec = ReadOnlyCodecV2 {
            read_passphrase: copy_secret(&read_passphrase),
        };
        let plan = super::sync_v2::SyncContext::new(
            &self.store,
            self.transport.as_ref(),
            &codec,
            composition.local.as_ref(),
            connection.clone(),
        )?
        .prepare_recovery(PrepareRecoveryRequest {
            connection_id: connection.id.clone(),
            source_binding_id: connection.binding_id.clone(),
            generation: request.generation,
            head: request.head,
            expected_view_digest: graph.observation.view_digest,
            scope_digest: current.scope_digest,
            mode: request.mode,
            backup_before_update: request.backup_before_update,
        })
        .await?;
        let backend = self.backend.clone();
        let id = plan.recovery_plan_id.clone();
        let prepared = CompositionV2 {
            provider: composition.provider.clone(),
            local: composition.local.clone(),
            remote_scope: composition.remote_scope.clone(),
            diff: vec![],
        };
        blocking(move || {
            backend
                .prepared_operation(&id, &prepared)
                .map_err(host::map_host_error)
        })
        .await?;
        let session_id = uuid::Uuid::new_v4().to_string();
        self.recoveries
            .lock()
            .map_err(|_| err(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .insert(
                session_id.clone(),
                Arc::new(RecoverySessionV2 {
                    plan: plan.clone(),
                    connection,
                    local: composition.local,
                    passphrase: read_passphrase,
                    gate: AsyncMutex::new(()),
                }),
            );
        Ok(RecoveryPreviewV2 {
            session_id,
            plan,
            entries: composition.diff,
        })
    }
    pub async fn recovery_confirm(&self, request: ConfirmRequestV2) -> SyncResult<RecoveryReceipt> {
        require_uuid(&request.session_id)?;
        let recovery = self
            .recoveries
            .lock()
            .map_err(|_| err(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .get(&request.session_id)
            .cloned()
            .ok_or_else(|| err(ErrorCode::SyncPlanExpired, Step::Recovery))?;
        let _gate = recovery.gate.lock().await;
        if recovery.plan.plan_digest != request.plan_digest {
            return Err(err(ErrorCode::SyncPlanChanged, Step::Recovery));
        }
        let resolver = self.resolver.clone();
        let store = self.store.clone();
        let transport = self.transport.clone();
        let runtime = tokio::runtime::Handle::current();
        let original = recovery.clone();
        blocking(move || {
            runtime.block_on(async {
                resolver.resolve_original(&original.connection)?;
                let codec = ReadOnlyCodecV2 {
                    read_passphrase: copy_secret(&original.passphrase),
                };
                super::sync_v2::SyncContext::new(
                    &store,
                    transport.as_ref(),
                    &codec,
                    original.local.as_ref(),
                    original.connection.clone(),
                )?
                .apply_recovery(ConfirmRecoveryRequest {
                    recovery_plan_id: original.plan.recovery_plan_id.clone(),
                    plan_digest: original.plan.plan_digest.clone(),
                })
                .await
            })
        })
        .await
    }
    pub async fn recovery_result(&self, recovery_plan_id: String) -> SyncResult<RecoveryReceipt> {
        require_uuid(&recovery_plan_id)?;
        let store = self.store.clone();
        blocking(move || {
            store
                .receive(&recovery_plan_id)?
                .receipt
                .ok_or_else(|| err(ErrorCode::SyncJournalInvalid, Step::Recovery))
        })
        .await
    }
    /// Public cross-restart recovery keyed by the ORIGINAL operation, never a
    /// UI-invented session. Verify stored results first without today's source;
    /// only a genuinely unfinished same-O write reconstructs trusted guards.
    pub async fn resume_original(
        &self,
        request: ResumeUploadRequest,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<OperationFact> {
        require_uuid(&request.operation_id)?;
        require_digest(&request.plan_digest)?;
        require_digest(&request.expected_view_digest)?;
        let verified = self
            .verify(request.operation_id.clone(), copy_secret(&read_passphrase))
            .await;
        let store = self.store.clone();
        let id = request.operation_id.clone();
        let journal = blocking(move || store.journal(&id)).await?;
        if journal.fact.storage_status == StorageStatus::Stored {
            return verified;
        }
        match verified {
            Err(e) if e.code != ErrorCode::SyncOperationUnknown => return Err(e),
            _ => (),
        }
        if journal.plan.plan_digest != request.plan_digest {
            return Err(err(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let files = self.files.clone();
        let id = request.operation_id.clone();
        let original: OriginalRecordV2 = blocking(move || {
            files
                .read(&format!("{id}.original.json"))?
                .ok_or_else(|| err(ErrorCode::SyncEndpointChanged, Step::Verify))
        })
        .await?;
        if original.root_digest != self.files.root_digest
            || original.operation_id != request.operation_id
        {
            return Err(err(ErrorCode::SyncJournalInvalid, Step::Verify));
        }
        let handle = original.handle.handle();
        let resolver = self.resolver.clone();
        let saved = original.endpoint.clone();
        let expected = handle.clone();
        let file = self.file_transport.clone();
        blocking(move || {
            let bound = resolver.restore_original(saved, &expected)?;
            if let Some(file) = file {
                bound.bind_file_transport(&file)?;
            }
            Ok(())
        })
        .await?;
        let graph = self.graph(&handle).await?;
        graph.ordinary(journal.plan.intent == Intent::Resolve)?;
        let parent = journal
            .plan
            .resolution
            .as_ref()
            .and_then(|r| r.selected_head.as_ref())
            .or(journal.plan.parents.first());
        let baseline = if let Some(parent) = parent {
            let generation = journal
                .fact
                .generation
                .as_ref()
                .ok_or_else(|| err(ErrorCode::SyncGenerationChanged, Step::Verify))?;
            let node = graph.node(generation, parent)?;
            let budgets = Budgets::default();
            let bytes = graph::Network {
                transport: self.transport.as_ref(),
                connection: &handle,
                limits: &budgets,
                slots: &self.slots,
            }
            .get_retry(
                &SyncRelativePath::data(&generation.generation_id, &parent.operation_id)?,
                budgets.payload_bytes,
                Step::PayloadGet,
            )
            .await?;
            if store::digest(&bytes) != node.commit.payload_digest
                || bytes.len() as u64 != node.commit.payload_bytes
            {
                return Err(err(ErrorCode::SyncRemoteInvalid, Step::PayloadGet));
            }
            Some(Zeroizing::new(bytes))
        } else {
            None
        };
        let backend = self.backend.clone();
        let old = original.request;
        let conn = handle.clone();
        let pass = copy_secret(&read_passphrase);
        let composition = blocking(move || {
            let cloud = baseline
                .map(|bytes| {
                    file_objects_v2::decode(
                        &bytes,
                        pass.as_ref().map(|s| s.as_str()),
                        &file_objects_v2::supported_schema(),
                        Some(&host::PayloadAuthorizationV2),
                    )
                    .map(|d| d.objects)
                    .map_err(host::map_host_error)
                })
                .transpose()?;
            backend
                .compose(
                    &PreviewRequestV2 {
                        selection: old.selection,
                        mcp_clients: old.mcp_clients,
                        assets_reviewed: old.assets_reviewed,
                        selected_head: None,
                        forced_direction: Some(ServiceDirectionV2::Upload),
                        backup_before_update: true,
                    },
                    &conn,
                    cloud.as_ref(),
                    pass,
                )
                .map_err(host::map_host_error)
        })
        .await
        .map_err(|e| {
            if e.code == ErrorCode::SyncEndpointChanged {
                e
            } else {
                err(ErrorCode::SyncSourceChanged, Step::SourceGuard)
            }
        })?;
        let root = composition.local.root_revision()?;
        let source = composition.local.current_source()?;
        if root != journal.root_revision || source != journal.plan.source_guard {
            return Err(err(ErrorCode::SyncSourceChanged, Step::SourceGuard));
        }
        let resolver = self.resolver.clone();
        let store = self.store.clone();
        let transport = self.transport.clone();
        let runtime = tokio::runtime::Handle::current();
        blocking(move || {
            runtime.block_on(async {
                resolver.resolve_original(&handle)?;
                let codec = ReadOnlyCodecV2 { read_passphrase };
                super::sync_v2::SyncContext::new(
                    &store,
                    transport.as_ref(),
                    &codec,
                    composition.local.as_ref(),
                    handle,
                )?
                .resume_original_upload(request)
                .await
            })
        })
        .await
    }
    pub async fn resume(
        &self,
        request: ConfirmRequestV2,
        expected_view_digest: Digest,
    ) -> SyncResult<OperationFact> {
        let session = self.session(&request.session_id)?;
        let _guard = session.gate.lock().await;
        if session.digest != request.plan_digest {
            return Err(err(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let id = session
            .operation_id
            .clone()
            .ok_or_else(|| err(ErrorCode::SyncPlanChanged, Step::Verify))?;
        session
            .host
            .resume_original_upload(
                ResumeUploadRequest {
                    operation_id: id,
                    plan_digest: session.digest.clone(),
                    expected_view_digest,
                },
                &self.resolver,
                &self.store,
                self.transport.as_ref(),
                session.local.as_ref(),
            )
            .await
    }
    pub async fn cancel(&self, session_id: String) -> SyncResult<Option<OperationFact>> {
        let session = self.session(&session_id)?;
        session.cancelled.store(true, Ordering::SeqCst);
        if !matches!(session.plan, ServicePlanV2::Upload(_)) {
            return Ok(None);
        }
        let store = self.store.clone();
        let transport = self.transport.clone();
        let local = session.local.clone();
        let connection = session.connection.clone();
        let id = session
            .operation_id
            .clone()
            .ok_or_else(|| err(ErrorCode::SyncPlanChanged, Step::Verify))?;
        // Cancel is local and must not queue behind an in-flight network await.
        let resolver = self.resolver.clone();
        let _ = resolver;
        blocking(move || {
            let codec = CancelCodec;
            super::sync_v2::SyncContext::new(
                &store,
                transport.as_ref(),
                &codec,
                local.as_ref(),
                connection,
            )?
            .cancel_upload(OperationRequest { operation_id: id })
            .map(Some)
        })
        .await
    }
}
fn entries_from(objects: &SavedObjects, local: bool) -> Vec<ObjectEntryV2> {
    let p = objects.partitions();
    let mut out = Vec::new();
    let mut add = |kind, id: String, label: String, model_ids: Vec<String>, secret_available| {
        out.push(ObjectEntryV2 {
            kind,
            id,
            label,
            model_ids,
            models: vec![],
            family_ids: vec![],
            revisions: vec![],
            default_revision: None,
            secret_available,
            available_local: local,
            available_cloud: !local,
        })
    };
    for c in p.models.payload.policy.iter().flat_map(|p| &p.channels) {
        add(
            ObjectKindV2::Channel,
            c.id.clone(),
            c.label.clone(),
            c.models.iter().map(|m| m.raw_model_id.clone()).collect(),
            p.channel_keys.get(&c.id).is_some() || p.channel_authorizations.contains_key(&c.id),
        );
    }
    for t in &p.models.payload.templates {
        add(
            ObjectKindV2::Template,
            t.id.clone(),
            t.label.clone(),
            vec![],
            false,
        );
    }
    if p.manager.preferences.is_some() {
        add(
            ObjectKindV2::Preferences,
            "preferences".into(),
            "偏好设置".into(),
            vec![],
            false,
        );
    }
    if p.manager.extensions.is_some() {
        add(
            ObjectKindV2::Extensions,
            "extensions".into(),
            "扩展配置".into(),
            vec![],
            false,
        );
    }
    for a in &p.manager.accounts {
        add(
            ObjectKindV2::Account,
            a.id.clone(),
            a.label.clone(),
            vec![],
            false,
        );
    }
    for s in &p.assets.skills {
        add(
            ObjectKindV2::Skill,
            s.name.clone(),
            s.name.clone(),
            vec![],
            s.contains_secrets,
        );
    }
    for m in &p.assets.mcps {
        add(
            ObjectKindV2::Mcp,
            m.name.clone(),
            m.name.clone(),
            vec![],
            !m.secret_fields.is_empty(),
        );
    }
    for row in &mut out {
        match row.kind {
            ObjectKindV2::Channel => {
                if let Some(c) = p
                    .models
                    .payload
                    .policy
                    .iter()
                    .flat_map(|p| &p.channels)
                    .find(|c| c.id == row.id)
                {
                    row.models = c
                        .models
                        .iter()
                        .map(|m| ModelInventoryV2 {
                            raw_model_id: m.raw_model_id.clone(),
                            label: m.label.clone(),
                            observed: m.observed,
                            selected: match &c.selection {
                                super::schema::ModelSelection::All => true,
                                super::schema::ModelSelection::None => false,
                                super::schema::ModelSelection::Whitelist(ids) => {
                                    ids.contains(&m.raw_model_id)
                                }
                            },
                            available_local: local,
                            available_cloud: !local,
                            family_id: m
                                .binding
                                .as_ref()
                                .map(|b| b.reference().template_id.clone()),
                        })
                        .collect();
                    row.family_ids = c
                        .family_scope
                        .iter()
                        .flatten()
                        .cloned()
                        .chain(c.family_overrides.keys().cloned())
                        .chain(c.models.iter().filter_map(|m| {
                            m.binding
                                .as_ref()
                                .map(|b| b.reference().template_id.clone())
                        }))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                }
            }
            ObjectKindV2::Template => {
                if let Some(t) = p.models.payload.templates.iter().find(|t| t.id == row.id) {
                    row.revisions = t
                        .revisions
                        .iter()
                        .map(|r| super::schema_v2::RefV2 {
                            template_id: t.id.clone(),
                            revision_id: r.id.clone(),
                            digest: r.digest.clone(),
                        })
                        .collect();
                    row.default_revision = t
                        .default_revision
                        .as_ref()
                        .and_then(|id| row.revisions.iter().find(|r| r.revision_id == *id))
                        .cloned();
                }
            }
            _ => (),
        }
    }
    out
}
fn initial_scope(local: &SavedObjects, cloud: Option<&SavedObjects>) -> ScopeRequestV2 {
    let mut selection = SelectionV2::default();
    let mut seen = BTreeSet::new();
    // Prefer cloud portable identity for a new device. The mapping owner, not
    // a label match, later decides whether a local identity is the same object.
    for o in cloud.into_iter().chain(std::iter::once(local)) {
        let p = o.partitions();
        for c in p.models.payload.policy.iter().flat_map(|p| &p.channels) {
            if seen.insert(("channel", c.id.clone())) {
                selection.channels.push(ChannelSelectionV2 {
                    portable_channel_id: c.id.clone(),
                    model_ids: c.model_subset.clone(),
                    family_ids: c.family_scope.clone(),
                    include_connection: c.connection.is_some(),
                    include_key: p.channel_keys.get(&c.id).is_some(),
                    include_authorization: p.channel_authorizations.contains_key(&c.id),
                });
            }
        }
        for t in &p.models.payload.templates {
            for r in &t.revisions {
                if seen.insert(("revision", r.id.clone())) {
                    selection.template_refs.push(super::schema_v2::RefV2 {
                        template_id: t.id.clone(),
                        revision_id: r.id.clone(),
                        digest: r.digest.clone(),
                    });
                }
            }
        }
        selection.manager.preferences |= p.manager.preferences.is_some();
        selection.manager.extensions |= p.manager.extensions.is_some();
        for a in &p.manager.accounts {
            if seen.insert(("account", a.id.clone())) {
                selection.manager.account_ids.push(a.id.clone());
            }
        }
        for s in &p.assets.skills {
            if seen.insert(("skill", s.name.clone())) {
                selection.skill_names.push(s.name.clone());
                if s.contains_secrets {
                    selection
                        .extension_credentials
                        .skill_names
                        .push(s.name.clone());
                }
            }
        }
        for m in &p.assets.mcps {
            if seen.insert(("mcp", m.name.clone())) {
                if let Ok(definition_digest) = super::objects_overlay_v2::mcp_source_digest(m) {
                    selection.mcp_sources.push(McpSourceSelectionV2 {
                        name: m.name.clone(),
                        definition_digest,
                    });
                    if !m.secrets.0.is_empty() {
                        selection
                            .extension_credentials
                            .mcp_names
                            .push(m.name.clone());
                    }
                }
            }
        }
    }
    ScopeRequestV2 {
        selection,
        mcp_clients: Default::default(),
        assets_reviewed: false,
    }
}
fn id_for_record(id: &Option<String>) -> SyncResult<String> {
    id.clone()
        .ok_or_else(|| err(ErrorCode::SyncPlanChanged, Step::Persist))
}
struct CancelCodec;
impl PayloadCodec for CancelCodec {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef> {
        vec![file_objects_v2::supported_schema()]
    }
    fn freeze_selected(
        &self,
        _: &SourceGuard,
        _: &[PayloadInput],
        _: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload> {
        Err(err(ErrorCode::SyncPlanChanged, Step::Verify))
    }
    fn inspect_and_validate(&self, _: &[u8], _: &PayloadSchemaRef) -> SyncResult<()> {
        Err(err(ErrorCode::SyncPlanChanged, Step::Verify))
    }
}
struct ReadOnlyCodecV2 {
    read_passphrase: Option<Zeroizing<String>>,
}
impl PayloadCodec for ReadOnlyCodecV2 {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef> {
        vec![file_objects_v2::supported_schema()]
    }
    fn freeze_selected(
        &self,
        _: &SourceGuard,
        _: &[PayloadInput],
        _: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload> {
        Err(err(ErrorCode::SyncPlanChanged, Step::Verify))
    }
    fn inspect_and_validate(&self, bytes: &[u8], schema: &PayloadSchemaRef) -> SyncResult<()> {
        file_objects_v2::decode(
            bytes,
            self.read_passphrase.as_ref().map(|p| p.as_str()),
            schema,
            Some(&host::PayloadAuthorizationV2),
        )
        .map(|_| ())
        .map_err(host::map_host_error)
    }
}
struct HistoricalOnlyLocal;
impl LocalApplication for HistoricalOnlyLocal {
    fn root_revision(&self) -> SyncResult<Digest> {
        Err(err(ErrorCode::SyncSourceChanged, Step::Verify))
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        Err(err(ErrorCode::SyncSourceChanged, Step::Verify))
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        Err(err(ErrorCode::SyncLocalChanged, Step::Verify))
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
        Err(err(ErrorCode::SyncPlanChanged, Step::Verify))
    }
    fn export_once(&self, _: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<String> {
        Err(err(ErrorCode::SyncPlanChanged, Step::Verify))
    }
}
#[cfg(test)]
#[path = "sync_service_v2_tests.rs"]
mod tests;

/// Production saved-object composer. Every mutable file/model/credential rule
/// stays with its existing owner. This adapter only supplies immutable context
/// and turns the exact core operation ID into the receiver's participant ID.
pub struct ProductionServiceBackendV2 {
    root: PathBuf,
    home: PathBuf,
    credentials: Arc<dyn super::saved_objects_capture_v2::CaptureCredentialsV2 + Send + Sync>,
}
impl ProductionServiceBackendV2 {
    pub fn new(
        root: PathBuf,
        home: PathBuf,
        credentials: Arc<dyn super::saved_objects_capture_v2::CaptureCredentialsV2 + Send + Sync>,
    ) -> ModelResult<Self> {
        if !root.is_absolute() || !root.is_dir() || !home.is_absolute() || !home.is_dir() {
            return Err(invalid("capture_root_invalid", "同步尚未绑定合法用户根"));
        }
        Ok(Self {
            root,
            home,
            credentials,
        })
    }
}
/// Keep the actual issued mapping facts alive for upload sessions too. The
/// mapper intentionally uses Weak issued records; a binding hash alone is not
/// permission and must never be reconstructed as one.
struct OwnedMappingCaptureV2 {
    inner: host::MappedCaptureProviderV2,
    _preview: Arc<super::sync_identity_v2::MappingPreviewV2>,
}
impl CaptureProviderV2 for OwnedMappingCaptureV2 {
    fn capture_saved(&self, s: &SelectionV2) -> ModelResult<host::HostCapturedV2> {
        self.inner.capture_saved(s)
    }
    fn target_binding(&self, s: &SelectionV2) -> ModelResult<Option<host::LocalTargetBindingV2>> {
        self.inner.target_binding(s)
    }
    fn verify_target(&self, b: &host::LocalTargetBindingV2) -> ModelResult<()> {
        self.inner.verify_target(b)
    }
}
/// Private reconstruction inputs issued by THIS service. A checksum here is
/// not a model adoption receipt or mapping authority: owner proof getters must
/// validate these exact descriptors against their own journals/transactions.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DownloadContextV2 {
    version: u32,
    operation_id: String,
    root_digest: Digest,
    home_digest: Digest,
    original_binding_bytes: Vec<u8>,
    receive_bindings: Vec<StoredReceiveBindingV2>,
    mapping_before: Option<Vec<u8>>,
    mapping_after: Vec<u8>,
    model_handle: String,
    local_capture: ScopeRequestV2,
    remote_scope: SelectionV2,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReceiveBindingV2 {
    portable_channel_id: String,
    local_channel_id: String,
    provider_id: String,
    mode: String,
}
impl DownloadContextV2 {
    fn receiver(
        &self,
        root: &Path,
        home: &Path,
    ) -> ModelResult<super::config_file::v2::receive::ReceiveFileHostV2> {
        use super::{
            config_file::v2::receive::{IdentityMappingWriteV2, ReceiveFileHostV2},
            receive_models_v2::{ReceiveBindingModeV2, ReceiveBindingV2},
        };
        require_uuid(&self.operation_id)
            .map_err(|_| invalid("sync_identity_operation", "原操作身份无效"))?;
        let files = ServiceFilesV2::new(root)
            .map_err(|_| invalid("sync_identity_read", "原接收根无法读取"))?;
        let home_digest = store::digest(
            home.canonicalize()
                .map_err(|_| invalid("sync_identity_read", "原配置根无法读取"))?
                .to_string_lossy()
                .as_bytes(),
        );
        if self.version != 2
            || self.root_digest != files.root_digest
            || self.home_digest != home_digest
        {
            return Err(invalid("host_mapping_changed", "原接收根绑定不一致"));
        }
        let mut sources = BTreeSet::new();
        let mut targets = BTreeSet::new();
        let mut bindings = Vec::new();
        for b in &self.receive_bindings {
            super::schema_v2::check_id(&b.portable_channel_id)
                .map_err(|_| invalid("sync_identity_read", "原身份材料损坏"))?;
            super::schema_v2::check_id(&b.local_channel_id)
                .map_err(|_| invalid("sync_identity_read", "原身份材料损坏"))?;
            if b.provider_id.is_empty()
                || b.provider_id.len() > 512
                || !sources.insert(&b.portable_channel_id)
                || !targets.insert(&b.local_channel_id)
            {
                return Err(invalid("sync_identity_read", "原接收身份冲突"));
            }
            let mode = match b.mode.as_str() {
                "new" => ReceiveBindingModeV2::New,
                "existing" => ReceiveBindingModeV2::Existing,
                _ => return Err(invalid("sync_identity_read", "原接收模式无效")),
            };
            bindings.push(ReceiveBindingV2 {
                portable_channel_id: b.portable_channel_id.clone(),
                local_channel_id: b.local_channel_id.clone(),
                provider_id: b.provider_id.clone(),
                mode,
            });
        }
        Ok(ReceiveFileHostV2 {
            root: root.into(),
            home: home.into(),
            bindings,
            trusted_mapping_write: Some(IdentityMappingWriteV2 {
                before: self.mapping_before.clone(),
                after: self.mapping_after.clone(),
            }),
        })
    }
}
struct ReceiveFactoryV2 {
    root: PathBuf,
    home: PathBuf,
    preview: Arc<super::sync_identity_v2::MappingPreviewV2>,
    hosts: Mutex<BTreeMap<String, Arc<super::config_file::v2::receive::ReceiveFileHostV2>>>,
    handles: Mutex<BTreeMap<String, super::sync_local_v2::ModelApplyPlanHandle>>,
}
impl ReceiveFactoryV2 {
    fn receiver(
        &self,
        id: &str,
    ) -> ModelResult<Arc<super::config_file::v2::receive::ReceiveFileHostV2>> {
        use super::config_file::v2::receive::ReceiveFileHostV2;
        super::schema_v2::check_id(id)
            .map_err(|_| invalid("sync_identity_operation", "操作身份无效"))?;
        let mut hosts = self
            .hosts
            .lock()
            .map_err(|_| invalid("sync_identity_read", "接收操作状态不可用"))?;
        if let Some(host) = hosts.get(id) {
            return Ok(host.clone());
        }
        // This is the SAME ID generated by prepare_download. No second UUID,
        // moved head, reissued mapping preview or ad-hoc identity write exists.
        let host = Arc::new(ReceiveFileHostV2 {
            root: self.root.clone(),
            home: self.home.clone(),
            bindings: self.preview.receive_bindings().to_vec(),
            trusted_mapping_write: Some(self.preview.after_adoption(id)?),
        });
        hosts.insert(id.into(), host.clone());
        Ok(host)
    }
    fn verify_applied(&self, id: &str, binding: &host::LocalTargetBindingV2) -> ModelResult<()> {
        use super::sync_local_v2::{ModelApplyStatus, ModelApplyV2};
        if *binding != *self.preview.binding() {
            return Err(invalid("host_mapping_changed", "原接收目标绑定已改变"));
        }
        let handle = self
            .handles
            .lock()
            .map_err(|_| invalid("sync_identity_read", "原接收状态不可用"))?
            .get(id)
            .cloned()
            .ok_or_else(|| invalid("host_mapping_changed", "原模型采用回执缺失"))?;
        let receiver = self
            .hosts
            .lock()
            .map_err(|_| invalid("sync_identity_read", "原接收状态不可用"))?
            .get(id)
            .cloned()
            .ok_or_else(|| invalid("host_mapping_changed", "原接收操作缺失"))?;
        if !matches!(receiver.status(id, &handle)?, ModelApplyStatus::Applied(_)) {
            return Err(invalid(
                "host_mapping_changed",
                "原模型尚未采用，不能读取后继映射作为原证明",
            ));
        }
        self.preview.verify_adopted(id)
    }
}
impl super::sync_local_v2::ModelApplyV2 for ReceiveFactoryV2 {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        scope: &SelectionV2,
        expected_target: &str,
    ) -> ModelResult<super::sync_local_v2::ModelApplyPlanHandle> {
        let receiver = self.receiver(id)?;
        let handle = receiver.prepare(id, objects, scope, expected_target)?;
        let mapping = receiver
            .trusted_mapping_write
            .as_ref()
            .ok_or_else(|| invalid("sync_identity_read", "原接收映射材料缺失"))?;
        let files = ServiceFilesV2::new(&self.root)
            .map_err(|_| invalid("sync_identity_read", "原接收上下文无法保存"))?;
        let local = self.preview.local_capture_request();
        let context = DownloadContextV2 {
            version: 2,
            operation_id: id.into(),
            root_digest: files.root_digest.clone(),
            home_digest: store::digest(
                self.home
                    .canonicalize()
                    .map_err(|_| invalid("sync_identity_read", "原配置根无法读取"))?
                    .to_string_lossy()
                    .as_bytes(),
            ),
            original_binding_bytes: store::canonical(self.preview.binding())
                .map_err(|_| invalid("sync_identity_read", "原目标绑定无法冻结"))?,
            receive_bindings: receiver
                .bindings
                .iter()
                .map(|b| StoredReceiveBindingV2 {
                    portable_channel_id: b.portable_channel_id.clone(),
                    local_channel_id: b.local_channel_id.clone(),
                    provider_id: b.provider_id.clone(),
                    mode: match b.mode {
                        super::receive_models_v2::ReceiveBindingModeV2::New => "new",
                        super::receive_models_v2::ReceiveBindingModeV2::Existing => "existing",
                    }
                    .into(),
                })
                .collect(),
            mapping_before: mapping.before.clone(),
            mapping_after: mapping.after.clone(),
            model_handle: handle.token().into(),
            local_capture: ScopeRequestV2 {
                selection: local.selection,
                mcp_clients: local.mcp_clients,
                assets_reviewed: local.assets_reviewed,
            },
            remote_scope: self.preview.binding().remote_scope.clone(),
        };
        // Save before protect/apply. On re-open this record can reconstruct the
        // same receiver, but only its own Applied proof may authorize recovery.
        files
            .write(&format!("{id}.download-context.json"), context)
            .map_err(|_| invalid("sync_identity_read", "原接收上下文未持久保存"))?;
        let roundtrip: DownloadContextV2 = files
            .read(&format!("{id}.download-context.json"))
            .map_err(|_| invalid("sync_identity_read", "原接收上下文读回失败"))?
            .ok_or_else(|| invalid("sync_identity_read", "原接收上下文缺失"))?;
        let exact = roundtrip.receiver(&self.root, &self.home)?;
        if roundtrip.operation_id != id
            || roundtrip.model_handle != handle.token()
            || exact.bindings != receiver.bindings
        {
            return Err(invalid("sync_identity_read", "原接收上下文读回不一致"));
        }
        self.handles
            .lock()
            .map_err(|_| invalid("sync_identity_read", "接收操作状态不可用"))?
            .insert(id.into(), handle.clone());
        Ok(handle)
    }
    fn protect(
        &self,
        id: &str,
        handle: &super::sync_local_v2::ModelApplyPlanHandle,
        backup: bool,
    ) -> ModelResult<()> {
        self.receiver(id)?.protect(id, handle, backup)
    }
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        handle: &super::sync_local_v2::ModelApplyPlanHandle,
        backup: bool,
    ) -> ModelResult<super::sync_local_v2::ModelApplyReceipt> {
        self.receiver(id)?.apply_once(id, objects, handle, backup)
    }
    fn status(
        &self,
        id: &str,
        handle: &super::sync_local_v2::ModelApplyPlanHandle,
    ) -> ModelResult<super::sync_local_v2::ModelApplyStatus> {
        self.receiver(id)?.status(id, handle)
    }
}
struct MappedSourceBridgeV2 {
    provider: Arc<dyn CaptureProviderV2>,
    scope: SelectionV2,
    binding: host::LocalTargetBindingV2,
    receiver: Arc<ReceiveFactoryV2>,
}
impl super::sync_local_v2::SourceSnapshotProviderV2 for MappedSourceBridgeV2 {
    fn current(
        &self,
        _: &Path,
        _: &Path,
        _: &super::saved_objects_capture_v2::CaptureRequestV2,
    ) -> ModelResult<SourceGuard> {
        self.provider
            .capture_saved(&self.scope)
            .map(|c| c.source_guard)
    }
    fn root(
        &self,
        d: &Path,
        h: &Path,
        _: &super::saved_objects_capture_v2::CaptureRequestV2,
    ) -> ModelResult<String> {
        super::config_file::v2::with_saved_config(d, h, |_, _, _, _, _, revision| {
            Ok(revision.to_owned())
        })
        .map_err(|e| invalid(&e.code, "根绑定无法核对"))
    }
    fn target(
        &self,
        _: &Path,
        _: &Path,
        _: &super::saved_objects_capture_v2::CaptureRequestV2,
    ) -> ModelResult<String> {
        self.provider.verify_target(&self.binding)?;
        Ok(self.binding.digest.clone())
    }
    fn verify_applied_mapping(
        &self,
        id: &str,
        binding: &host::LocalTargetBindingV2,
    ) -> ModelResult<()> {
        self.receiver.verify_applied(id, binding)
    }
}
impl ProductionServiceBackendV2 {
    fn inventory_request(
        &self,
    ) -> ModelResult<(
        super::saved_objects_capture_v2::CaptureRequestV2,
        Vec<McpSourceEntryV2>,
    )> {
        use crate::modules::sync::{portable_accounts, portable_assets};
        let mut selection = super::config_file::v2::with_saved_config(
            &self.root,
            &self.home,
            |library, policy, _, _, _, _| {
                let channels = policy
                    .channels
                    .iter()
                    .filter(|c| {
                        !matches!(
                            c.connection.provider_id.to_ascii_lowercase().as_str(),
                            "openai" | "chatgpt" | "openai-multi" | "combo"
                        )
                    })
                    .map(|c| ChannelSelectionV2 {
                        portable_channel_id: c.id.clone(),
                        model_ids: None,
                        family_ids: None,
                        include_connection: true,
                        include_key: false,
                        include_authorization: false,
                    })
                    .collect();
                let template_refs = library
                    .templates
                    .iter()
                    .flat_map(|t| {
                        t.revisions.iter().map(move |r| super::schema_v2::RefV2 {
                            template_id: t.id.clone(),
                            revision_id: r.id.clone(),
                            digest: r.digest.clone(),
                        })
                    })
                    .collect();
                Ok(SelectionV2 {
                    channels,
                    template_refs,
                    ..Default::default()
                })
            },
        )
        .map_err(|e| invalid(&e.code, "已保存模型目录无法读取"))?;
        fn exists(root: &Path, relative: &str) -> ModelResult<bool> {
            let p = root.join(relative);
            match std::fs::symlink_metadata(&p) {
                Ok(m) if m.file_type().is_symlink() || !m.is_file() => {
                    Err(invalid("capture_source_invalid", "目录来源不是普通文件"))
                }
                Ok(_) => Ok(true),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(_) => Err(invalid("capture_source_invalid", "目录来源无法读取")),
            }
        }
        selection.manager.preferences = exists(
            &self.root,
            crate::modules::preferences::PREFERENCES_RELATIVE_PATH,
        )?;
        selection.manager.extensions = exists(
            &self.root,
            crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
        )?;
        selection.manager.account_ids = portable_accounts::catalog(&self.root, &self.home)?
            .into_iter()
            .filter(|a| a.information_supported)
            .map(|a| a.account.id)
            .collect();
        let catalog = portable_assets::catalog(&self.root, &self.home)?;
        // Non-secret Skills can be captured for public inventory without a vault
        // read. Sensitive SKILL.md remains listed by the asset catalog below;
        // adopting its content requires an explicit credential scope.
        let mut sources = Vec::new();
        let mut clients = BTreeMap::new();
        let names: BTreeSet<_> = catalog
            .mcps
            .into_iter()
            .chain(portable_assets::stored_names(&self.root)?)
            .collect();
        for name in names {
            let details = portable_assets::detail(&self.root, &self.home, "mcps", &name, None)?;
            let mut options = Vec::new();
            for source in &details.clients {
                let (assets, conflicts) = portable_assets::export_with_credential_scope(
                    &self.root,
                    &self.home,
                    &[],
                    std::slice::from_ref(&name),
                    &BTreeMap::from([(name.clone(), source.clone())]),
                    false,
                    false,
                    Some(&[]),
                )?;
                if !conflicts.is_empty() || assets.mcps.len() != 1 {
                    return Err(invalid("mcp_source", "目录来源需要重新确认"));
                }
                options.push(McpSourceOptionV2 {
                    source: source.clone(),
                    definition_digest: super::objects_overlay_v2::mcp_source_digest(
                        &assets.mcps[0],
                    )?,
                });
            }
            if details.conflicts.is_empty() {
                if let Some(chosen) = options.first() {
                    selection.mcp_sources.push(McpSourceSelectionV2 {
                        name: name.clone(),
                        definition_digest: chosen.definition_digest.clone(),
                    });
                    clients.insert(name.clone(), chosen.source.clone());
                }
            }
            sources.push(McpSourceEntryV2 {
                name,
                sources: options,
                differing_fields: details.conflicts,
            });
        }
        for name in catalog.skills {
            match portable_assets::export_with_credential_scope(
                &self.root,
                &self.home,
                std::slice::from_ref(&name),
                &[],
                &Default::default(),
                false,
                false,
                Some(&[]),
            ) {
                Ok(_) => selection.skill_names.push(name),
                Err(e) if e.code == "asset_secret" => {}
                Err(e) => return Err(e),
            }
        }
        Ok((
            super::saved_objects_capture_v2::CaptureRequestV2 {
                selection,
                mcp_clients: clients,
                assets_reviewed: false,
            },
            sources,
        ))
    }
}
impl ServiceBackendV2 for ProductionServiceBackendV2 {
    fn inventory(&self, connection: &ConnectionHandle) -> ModelResult<InventoryV2> {
        let (request, mcp_sources) = self.inventory_request()?;
        let identity = Arc::new(super::sync_identity_v2::SyncIdentityOwnerV2::new(
            self.root.clone(),
            self.home.clone(),
            connection.binding_id.clone(),
            self.credentials.clone(),
        )?);
        let mapping = identity.freeze_preview(DirectionV2::Upload, request, None)?;
        let actual = super::saved_objects_capture_v2::capture(
            &self.root,
            &self.home,
            &mapping.local_capture_request(),
            self.credentials.as_ref(),
        )?;
        let objects = host::TrustedMappingOwnerV2::project_saved_to_remote(
            identity.as_ref(),
            actual.objects,
            mapping.binding(),
        )?;
        let (local_keys, local_auth) = super::config_file::v2::with_saved_config(
            &self.root,
            &self.home,
            |_, policy, _, raw, _, _| {
                let mut keys = BTreeSet::new();
                let mut auth = BTreeSet::new();
                for channel in &policy.channels {
                    if let Some(provider) = raw
                        .get("providers")
                        .and_then(|v| v.get(&channel.connection.provider_id))
                    {
                        let private = [
                            "apiKeyPool",
                            "headers",
                            "queryParams",
                            "proxy",
                            "mcpServers",
                            "desktopExecutor",
                        ]
                        .iter()
                        .any(|field| {
                            provider.get(*field).is_some_and(|v| match v {
                                serde_json::Value::Null => false,
                                serde_json::Value::String(s) => !s.is_empty(),
                                serde_json::Value::Array(a) => !a.is_empty(),
                                serde_json::Value::Object(m) => !m.is_empty(),
                                _ => true,
                            })
                        });
                        if private {
                            auth.insert(channel.id.clone());
                        } else if provider
                            .get("apiKey")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|s| !s.is_empty())
                        {
                            keys.insert(channel.id.clone());
                        }
                    }
                }
                Ok((keys, auth))
            },
        )
        .map_err(|e| invalid(&e.code, "凭据范围目录无法读取"))?;
        let projected = |ids: BTreeSet<String>| -> BTreeSet<String> {
            ids.into_iter()
                .filter_map(|id| {
                    mapping
                        .binding()
                        .slots
                        .iter()
                        .find(|s| s.local_id == id)
                        .map(|s| s.portable_id.clone())
                })
                .collect()
        };
        let asset_catalog = crate::modules::sync::portable_assets::catalog(&self.root, &self.home)?;
        Ok(InventoryV2 {
            objects,
            mcp_sources,
            key_channel_ids: projected(local_keys),
            authorization_channel_ids: projected(local_auth),
            credential_skill_names: asset_catalog.credential_skills.into_iter().collect(),
            credential_mcp_names: asset_catalog.credential_mcps.into_iter().collect(),
        })
    }
    fn compose(
        &self,
        request: &PreviewRequestV2,
        connection: &ConnectionHandle,
        cloud: Option<&SavedObjects>,
        read_passphrase: Option<Zeroizing<String>>,
    ) -> ModelResult<CompositionV2> {
        use super::{sync_identity_v2::SyncIdentityOwnerV2, sync_local_v2::LocalApplicationV2};
        let identity = Arc::new(SyncIdentityOwnerV2::new(
            self.root.clone(),
            self.home.clone(),
            connection.binding_id.clone(),
            self.credentials.clone(),
        )?);
        let input_request = super::saved_objects_capture_v2::CaptureRequestV2 {
            selection: request.selection.clone(),
            mcp_clients: request.mcp_clients.clone(),
            assets_reviewed: request.assets_reviewed,
        };
        // A genuinely empty remote has no receive authority to resolve first.
        // Its explicit local upload selection goes straight through Upload.
        // With valid cloud material the owner translates portable -> actual
        // local identities before direction selection; no label inference.
        let (direction, mapping) = if cloud.is_none() {
            (
                DirectionV2::Upload,
                Arc::new(identity.freeze_preview(DirectionV2::Upload, input_request, None)?),
            )
        } else {
            let receive_mapping =
                Arc::new(identity.freeze_preview(DirectionV2::Download, input_request, cloud)?);
            let actual_local = super::saved_objects_capture_v2::capture(
                &self.root,
                &self.home,
                &receive_mapping.local_capture_request(),
                self.credentials.as_ref(),
            )?;
            let projected_local = host::TrustedMappingOwnerV2::project_saved_to_remote(
                identity.as_ref(),
                actual_local.objects,
                receive_mapping.binding(),
            )?;
            let direction = match &request.forced_direction {
                Some(d) => d.host(),
                None => host::automatic_direction_from_saved(
                    &projected_local,
                    cloud,
                    &request.selection,
                )
                .map_err(|_| invalid("capture_source_invalid", "保存事实无法判断方向"))?,
            };
            let mapping = if direction == DirectionV2::Upload {
                if receive_mapping
                    .binding()
                    .slots
                    .iter()
                    .any(|s| matches!(s.expected, host::TargetExpectationV2::ExpectedAbsent(_)))
                {
                    return Err(invalid(
                        "capture_selection_missing",
                        "上传来源缺少明确已保存对象",
                    ));
                }
                Arc::new(identity.freeze_preview(
                    DirectionV2::Upload,
                    receive_mapping.local_capture_request(),
                    None,
                )?)
            } else {
                receive_mapping
            };
            (direction, mapping)
        };
        let local_capture = mapping.local_capture_request();
        let actual = Arc::new(
            host::SavedCaptureProviderV2::new(
                self.root.clone(),
                self.home.clone(),
                local_capture_for_copy(&local_capture)?,
                self.credentials.clone(),
            )
            .map_err(|_| invalid("capture_scope_invalid", "本机捕获选区无效"))?,
        );
        let mapped = Arc::new(OwnedMappingCaptureV2 {
            inner: host::MappedCaptureProviderV2::new(
                actual,
                identity.clone(),
                mapping.binding().clone(),
            )
            .map_err(|_| invalid("host_mapping_changed", "映射选区未通过核验"))?,
            _preview: mapping.clone(),
        });
        let remote_scope = mapping.binding().remote_scope.clone();
        let source = mapped.capture_saved(&remote_scope)?;
        let receiver = Arc::new(ReceiveFactoryV2 {
            root: self.root.clone(),
            home: self.home.clone(),
            preview: mapping.clone(),
            hosts: Default::default(),
            handles: Default::default(),
        });
        let source_bridge = Arc::new(MappedSourceBridgeV2 {
            provider: mapped.clone(),
            scope: remote_scope.clone(),
            binding: mapping.binding().clone(),
            receiver: receiver.clone(),
        });
        let local: Arc<dyn LocalApplication> = if direction == DirectionV2::Download {
            Arc::new(
                LocalApplicationV2::new_mapped(
                    self.root.clone(),
                    self.home.clone(),
                    remote_scope.clone(),
                    local_capture,
                    source.source_guard.scope_digest.clone(),
                    source_bridge,
                    receiver,
                    read_passphrase,
                    mapping.binding().clone(),
                    identity,
                )
                .map_err(|_| invalid("host_mapping_changed", "实际接收目标无法绑定"))?,
            )
        } else {
            Arc::new(UploadSourceLocalV2 {
                provider: host::CurrentSourceProviderV2::new(
                    mapped.clone(),
                    remote_scope.clone(),
                    source.root_revision.clone(),
                )
                .map_err(|_| invalid("capture_scope_invalid", "上传来源无法绑定"))?,
            })
        };
        let diff = semantic_diff(&remote_scope, &source.objects, cloud, direction)?;
        Ok(CompositionV2 {
            provider: mapped,
            local,
            remote_scope,
            diff,
        })
    }
}
fn local_capture_for_copy(
    r: &super::saved_objects_capture_v2::CaptureRequestV2,
) -> ModelResult<super::saved_objects_capture_v2::CaptureRequestV2> {
    Ok(super::saved_objects_capture_v2::CaptureRequestV2 {
        selection: r.selection.clone(),
        mcp_clients: r.mcp_clients.clone(),
        assets_reviewed: r.assets_reviewed,
    })
}
struct UploadSourceLocalV2 {
    provider: host::CurrentSourceProviderV2,
}
impl LocalApplication for UploadSourceLocalV2 {
    fn root_revision(&self) -> SyncResult<Digest> {
        self.provider.root_revision()
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        self.provider.current_source()
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        Err(err(ErrorCode::SyncPlanChanged, Step::DownloadGuard))
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
        Err(err(ErrorCode::SyncPlanChanged, Step::ApplyLocal))
    }
    fn export_once(&self, _: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<String> {
        Err(err(ErrorCode::SyncPlanChanged, Step::ApplyLocal))
    }
}
fn semantic_diff(
    selection: &SelectionV2,
    local: &SavedObjects,
    cloud: Option<&SavedObjects>,
    direction: DirectionV2,
) -> ModelResult<Vec<DiffEntryV2>> {
    let left = file_objects_v2::save_state(local)?;
    let right = cloud
        .map(file_objects_v2::save_state)
        .transpose()?
        .unwrap_or_default();
    let mut rows = Vec::new();
    let mut add = |kind: ObjectKindV2, id: String, key: serde_json::Value| -> ModelResult<()> {
        let key = super::content_versions::key(&key)?;
        let (before, after) = if direction == DirectionV2::Download {
            (left.get(&key), right.get(&key))
        } else {
            (right.get(&key), left.get(&key))
        };
        let status = match (before, after) {
            (_, None) => DiffStatusV2::Unavailable,
            (None, Some(_)) => DiffStatusV2::Add,
            (Some(a), Some(b)) if a == b => DiffStatusV2::Unchanged,
            _ => DiffStatusV2::Update,
        };
        rows.push(DiffEntryV2 {
            kind,
            id,
            status,
            before_digest: None,
            after_digest: None,
        });
        Ok(())
    };
    for c in &selection.channels {
        add(
            ObjectKindV2::Channel,
            c.portable_channel_id.clone(),
            serde_json::json!([
                "v2-channel",
                c.portable_channel_id,
                c.model_ids,
                c.family_ids
            ]),
        )?;
    }
    for r in &selection.template_refs {
        add(
            ObjectKindV2::Template,
            r.template_id.clone(),
            serde_json::json!(["v2-template", r.template_id, r.revision_id]),
        )?;
    }
    if selection.manager.preferences {
        add(
            ObjectKindV2::Preferences,
            "preferences".into(),
            serde_json::json!(["v2-preferences"]),
        )?;
    }
    if selection.manager.extensions {
        add(
            ObjectKindV2::Extensions,
            "extensions".into(),
            serde_json::json!(["v2-extensions"]),
        )?;
    }
    for reference in &selection.adopt_template_defaults {
        add(
            ObjectKindV2::Template,
            reference.template_id.clone(),
            serde_json::json!(["v2-template-meta", reference.template_id]),
        )?;
    }
    for id in &selection.manager.account_ids {
        let account = local
            .partitions()
            .manager
            .accounts
            .iter()
            .find(|a| a.id == *id)
            .or_else(|| {
                cloud.and_then(|c| c.partitions().manager.accounts.iter().find(|a| a.id == *id))
            })
            .ok_or_else(|| invalid("overlay_selection_missing", "所选账户信息不存在"))?;
        add(
            ObjectKindV2::Account,
            id.clone(),
            serde_json::json!([
                "v2-accounts",
                super::objects_overlay_v2::account_source_identity(account)?
            ]),
        )?;
    }
    for name in &selection.skill_names {
        add(
            ObjectKindV2::Skill,
            name.clone(),
            serde_json::json!(["v2-skills", name]),
        )?;
    }
    for m in &selection.mcp_sources {
        add(
            ObjectKindV2::Mcp,
            m.name.clone(),
            serde_json::json!(["v2-mcps", m.name]),
        )?;
    }
    Ok(rows)
}
