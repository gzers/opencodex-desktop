//! Saved-only channel identity owner. Preview is an in-memory capability, never
//! a deserialized authority. This owner never writes a business or mapping file.
use super::{
    config_file::v2::{self as config, receive::IdentityMappingWriteV2},
    content_versions,
    file_objects_v2::{self, CapturedObjects, SavedObjects, SecretValues},
    objects_overlay_v2::UsageV2,
    portable_v2,
    receive_models_v2::{self, ReceiveBindingModeV2, ReceiveBindingV2},
    saved_objects_capture_v2::{self as capture, CaptureCredentialsV2, CaptureRequestV2},
    schema::{invalid, ModelError, ModelResult},
    schema_v2::{self, LibraryV2, PolicyV2},
    sync_host_v2::{
        DirectionV2, LocalTargetBindingV2, PayloadAuthorizationV2, TargetExpectationV2,
        TargetSlotV2, TrustedMappingOwnerV2,
    },
    sync_v2::types::SourceGuard,
};
use crate::modules::{
    backup::safety,
    extensions::{self, discovery, projection, ClientTarget, CLIENT_IDS},
    sync::{portable_accounts, portable_assets},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

const MAPPING_PATH: &str = "manager-state/sync-identity-mappings-v2.json";
const MAX_MAP_BYTES: u64 = 8 * 1024 * 1024;
const MAX_BINDINGS: usize = 20000;
const MAX_PREVIEWS: usize = 64;

fn err(code: &str) -> ModelError {
    invalid(code, "同步渠道身份事实无法核对，请重新预览或明确选择映射")
}
fn model(e: schema_v2::MatchErrorV2) -> ModelError {
    err(&e.code)
}
fn hash(value: &impl Serialize) -> ModelResult<String> {
    Ok(schema_v2::digest(
        &schema_v2::canonical_bytes(value).map_err(model)?,
    ))
}
fn request_copy(r: &CaptureRequestV2) -> CaptureRequestV2 {
    CaptureRequestV2 {
        selection: r.selection.clone(),
        mcp_clients: r.mcp_clients.clone(),
        assets_reviewed: r.assets_reviewed,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MappingDocument {
    schema_version: u32,
    bindings: Vec<MappingRow>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MappingRow {
    endpoint_binding_id: String,
    portable_id: String,
    local_id: String,
    adopted_operation_id: String,
}
fn document(bytes: Option<&[u8]>) -> ModelResult<MappingDocument> {
    let Some(bytes) = bytes else {
        return Ok(MappingDocument {
            schema_version: 2,
            bindings: vec![],
        });
    };
    if bytes.len() as u64 > MAX_MAP_BYTES {
        return Err(err("sync_identity_limit"));
    }
    let tree = schema_v2::parse_strict_json_v2(bytes).map_err(model)?;
    let doc: MappingDocument =
        serde_json::from_value(tree).map_err(|_| err("sync_identity_schema"))?;
    if doc.schema_version != 2 {
        return Err(err("model_schema_unsupported"));
    }
    if doc.bindings.len() > MAX_BINDINGS {
        return Err(err("sync_identity_limit"));
    }
    let mut portable = BTreeSet::new();
    let mut local = BTreeSet::new();
    for row in &doc.bindings {
        if !schema_v2::valid_digest(&row.endpoint_binding_id)
            || !schema_v2::valid_id(&row.portable_id)
            || !schema_v2::valid_id(&row.local_id)
            || !schema_v2::valid_id(&row.adopted_operation_id)
            || !portable.insert((&row.endpoint_binding_id, &row.portable_id))
            || !local.insert((&row.endpoint_binding_id, &row.local_id))
        {
            return Err(err("sync_identity_conflict"));
        }
    }
    Ok(doc)
}

/// Root is the explicit trust anchor, as in backup::safety (OS ancestors may
/// contain aliases). All existing descendants, including dangling links, fail
/// closed. NotFound is absence only after the complete owned path was checked.
fn mapping_bytes(root: &Path) -> ModelResult<Option<Vec<u8>>> {
    let path = root.join(MAPPING_PATH);
    safety::check_path(root, &path, true).map_err(|_| err("sync_identity_read"))?;
    match std::fs::symlink_metadata(&path) {
        Ok(_) => safety::read_file(&path, MAX_MAP_BYTES)
            .map(Some)
            .map_err(|_| err("sync_identity_read")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(err("sync_identity_read")),
    }
}
fn map_revision(bytes: Option<&[u8]>) -> String {
    schema_v2::digest(bytes.unwrap_or_default())
}

fn present_path(anchor: &Path, path: &Path, directory: bool) -> ModelResult<bool> {
    safety::check_path(anchor, path, true).map_err(|_| err("capture_source_invalid"))?;
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(err("capture_source_invalid")),
        Ok(_) => {
            let metadata = safety::inspect(path).map_err(|_| err("capture_source_invalid"))?;
            if metadata.is_dir() != directory {
                return Err(err("capture_source_invalid"));
            }
            Ok(true)
        }
    }
}

struct ObservedLocal {
    library: LibraryV2,
    policy: PolicyV2,
    providers: BTreeSet<String>,
    roster_digest: String,
    source_digest: String,
    root_revision: String,
}
impl ObservedLocal {
    fn same_source(&self, other: &Self) -> bool {
        self.roster_digest == other.roster_digest
            && self.source_digest == other.source_digest
            && self.root_revision == other.root_revision
            && self.library == other.library
            && self.policy == other.policy
    }
}
struct FrozenFacts {
    binding: LocalTargetBindingV2,
    request: CaptureRequestV2,
    receive: Vec<ReceiveBindingV2>,
    map_before: Option<Vec<u8>>,
    root_revision: String,
    roster_digest: String,
    source_digest: String,
    source_guard: SourceGuard,
    adoption: Mutex<Option<FrozenAdoption>>,
}
/// Read-only mapping evidence, never a model receipt or write admission.
struct FrozenAdoption {
    operation_id: String,
    after: Vec<u8>,
    revision: String,
}

/// D/H and endpoint binding digest come from the trusted composition owner, not
/// the UI. Credentials are the actual saved-capture credential owner. No Debug,
/// Serialize or Deserialize is implemented for this owner or its preview.
pub struct SyncIdentityOwnerV2 {
    root: PathBuf,
    home: PathBuf,
    endpoint_binding_id: String,
    credentials: Arc<dyn CaptureCredentialsV2 + Send + Sync>,
    issued: Mutex<BTreeMap<String, Weak<FrozenFacts>>>,
}
pub struct MappingPreviewV2 {
    owner: Arc<SyncIdentityOwnerV2>,
    facts: Arc<FrozenFacts>,
}
impl MappingPreviewV2 {
    pub fn binding(&self) -> &LocalTargetBindingV2 {
        &self.facts.binding
    }
    pub fn receive_bindings(&self) -> &[ReceiveBindingV2] {
        &self.facts.receive
    }
    pub fn local_capture_request(&self) -> CaptureRequestV2 {
        request_copy(&self.facts.request)
    }
    /// The actual local saved-capture guard, including original MCP guards.
    pub fn source_guard(&self) -> &SourceGuard {
        &self.facts.source_guard
    }
    pub fn root_revision(&self) -> &str {
        &self.facts.root_revision
    }
    /// Prepare exact before/after images for ReceiveFileHostV2's fixed eighth
    /// transaction target. Calling or dropping this never activates a pair.
    pub fn after_adoption(&self, operation_id: &str) -> ModelResult<IdentityMappingWriteV2> {
        if !schema_v2::valid_id(operation_id) {
            return Err(err("sync_identity_operation"));
        }
        self.owner.verify(self.binding())?;
        self.freeze_adoption(operation_id)?;
        let adoption = self
            .facts
            .adoption
            .lock()
            .map_err(|_| err("sync_identity_read"))?;
        let adoption = adoption
            .as_ref()
            .ok_or_else(|| err("sync_identity_adoption_unfrozen"))?;
        Ok(IdentityMappingWriteV2 {
            before: self.facts.map_before.clone(),
            after: adoption.after.clone(),
        })
    }
    /// Freeze before apply, or read the already frozen revision after apply.
    /// One issued preview can bind only ONE operation. A digest returned here
    /// does not prove that any model participant or transaction was applied.
    pub fn expected_adoption_revision(&self, operation_id: &str) -> ModelResult<String> {
        self.require_issued()?;
        self.freeze_adoption(operation_id)?;
        let adoption = self
            .facts
            .adoption
            .lock()
            .map_err(|_| err("sync_identity_read"))?;
        Ok(adoption
            .as_ref()
            .ok_or_else(|| err("sync_identity_adoption_unfrozen"))?
            .revision
            .clone())
    }
    /// MAPPING BYTES ONLY. The Service must independently verify this operation's
    /// actual ModelApply receipt before using this result to finish non-model
    /// participants. Never use this method to admit a new write. In particular,
    /// an existing-map no-op may match even BEFORE any model write occurred.
    /// This deliberately does not recapture or recheck the original Absent fact.
    pub fn verify_adopted(&self, operation_id: &str) -> ModelResult<()> {
        self.require_issued()?;
        let adoption = self
            .facts
            .adoption
            .lock()
            .map_err(|_| err("sync_identity_read"))?;
        let adoption = adoption
            .as_ref()
            .ok_or_else(|| err("sync_identity_adoption_unfrozen"))?;
        if adoption.operation_id != operation_id {
            return Err(err("sync_identity_adoption_operation"));
        }
        let current = mapping_bytes(&self.owner.root)?;
        document(current.as_deref())?;
        if current.as_deref() != Some(adoption.after.as_slice())
            || map_revision(current.as_deref()) != adoption.revision
        {
            return Err(err("sync_identity_adoption_changed"));
        }
        Ok(())
    }
    fn require_issued(&self) -> ModelResult<()> {
        let issued = self
            .owner
            .issued
            .lock()
            .map_err(|_| err("sync_identity_read"))?;
        if !issued
            .get(&self.binding().digest)
            .and_then(Weak::upgrade)
            .is_some_and(|facts| Arc::ptr_eq(&facts, &self.facts))
        {
            return Err(err("sync_identity_untrusted_preview"));
        }
        Ok(())
    }
    fn freeze_adoption(&self, operation_id: &str) -> ModelResult<()> {
        if !schema_v2::valid_id(operation_id) {
            return Err(err("sync_identity_operation"));
        }
        // Serialize first issue; verify() never takes the adoption mutex.
        let mut frozen = self
            .facts
            .adoption
            .lock()
            .map_err(|_| err("sync_identity_read"))?;
        if let Some(adoption) = frozen.as_ref() {
            return if adoption.operation_id == operation_id {
                Ok(())
            } else {
                Err(err("sync_identity_adoption_operation"))
            };
        }
        // A post-apply call cannot manufacture a missing pre-apply commitment.
        self.owner.verify(self.binding())?;
        let mut doc = document(self.facts.map_before.as_deref())?;
        let mut changed = false;
        for slot in &self.facts.binding.slots {
            let known = doc.bindings.iter().find(|row| {
                row.endpoint_binding_id == self.owner.endpoint_binding_id
                    && row.portable_id == slot.portable_id
            });
            if let Some(row) = known {
                if row.local_id != slot.local_id {
                    return Err(err("sync_identity_conflict"));
                }
            } else {
                changed = true;
                doc.bindings.push(MappingRow {
                    endpoint_binding_id: self.owner.endpoint_binding_id.clone(),
                    portable_id: slot.portable_id.clone(),
                    local_id: slot.local_id.clone(),
                    adopted_operation_id: operation_id.to_owned(),
                });
            }
        }
        // Preserve the EXACT original bytes for a known-map no-op, including
        // whitespace and original adoptedOperationIds. No canonical rewrite.
        let after = match (&self.facts.map_before, changed) {
            (Some(before), false) => before.clone(),
            _ => schema_v2::canonical_bytes(&doc).map_err(model)?,
        };
        document(Some(&after))?;
        let revision = schema_v2::digest(&after);
        *frozen = Some(FrozenAdoption {
            operation_id: operation_id.to_owned(),
            after,
            revision,
        });
        Ok(())
    }
}
impl SyncIdentityOwnerV2 {
    pub fn new(
        root: PathBuf,
        home: PathBuf,
        endpoint_binding_id: String,
        credentials: Arc<dyn CaptureCredentialsV2 + Send + Sync>,
    ) -> ModelResult<Self> {
        if !schema_v2::valid_digest(&endpoint_binding_id) {
            return Err(err("sync_identity_endpoint"));
        }
        for path in [&root, &home] {
            if !path.is_absolute()
                || !safety::inspect(path)
                    .map_err(|_| err("capture_root_invalid"))?
                    .is_dir()
            {
                return Err(err("capture_root_invalid"));
            }
        }
        Ok(Self {
            root,
            home,
            endpoint_binding_id,
            credentials,
            issued: Mutex::new(BTreeMap::new()),
        })
    }
    fn observe(&self) -> ModelResult<ObservedLocal> {
        safety::check_path(&self.home, &self.home.join("config.json"), true)
            .map_err(|_| err("sync_identity_read"))?;
        // The callback returns public facts only. All config/Store locks have
        // been released before capture() or a credential callback is invoked.
        config::with_saved_config(&self.root, &self.home, |library, policy, mappings, raw, source, root| {
            let providers: BTreeSet<String> = raw.get("providers")
                .map(|v| v.as_object().ok_or_else(|| schema_v2::error("model_source_invalid")))
                .transpose()?.into_iter().flat_map(|p| p.keys().cloned()).collect();
            let roster_digest = schema_v2::digest(&schema_v2::canonical_bytes(&json!({
                "providers": providers,
                "channels": policy.channels.iter().map(|c| (&c.id, &c.connection.provider_id)).collect::<Vec<_>>(),
                "runtimeMappings": mappings,
            }))?);
            Ok(ObservedLocal { library: library.clone(), policy: policy.clone(), providers,
                roster_digest, source_digest: source.to_owned(), root_revision: root.to_owned() })
        }).map_err(model)
    }
    fn capture(&self, request: &CaptureRequestV2) -> ModelResult<capture::CapturedSnapshotV2> {
        capture::capture(&self.root, &self.home, request, self.credentials.as_ref())
    }
    /// Only narrow the LOCAL saved-source request. Remote choices remain in
    /// binding.remote_scope. Missing non-model targets belong to LocalApplication's
    /// target CAS, not to channel identity allocation or a fabricated capture.
    fn local_nonmodel_sources(&self, request: &mut CaptureRequestV2) -> ModelResult<()> {
        let s = &mut request.selection;
        if s.manager.preferences {
            s.manager.preferences = present_path(
                &self.root,
                &self
                    .root
                    .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
                false,
            )?;
        }
        let extension_path = self.root.join(projection::CONFIG_RELATIVE_PATH);
        let needs_extension_config = s.manager.extensions || !s.skill_names.is_empty();
        let extension_exists =
            needs_extension_config && present_path(&self.root, &extension_path, false)?;
        s.manager.extensions &= extension_exists;
        if !s.manager.account_ids.is_empty() {
            // Read the existing metadata owner only. catalog() would allocate
            // account IDs, scan OAuth sources and write a business registry.
            let accounts = portable_accounts::imported(&self.root)?;
            s.manager.account_ids.retain(|id| {
                accounts
                    .iter()
                    .any(|a| a.id == *id && portable_accounts::information_supported(a))
            });
        }
        if !s.skill_names.is_empty() {
            let config = if extension_exists {
                let bytes = safety::read_file(&extension_path, projection::CONFIG_MAX_BYTES)
                    .map_err(|_| err("capture_source_invalid"))?;
                let value = schema_v2::parse_strict_json_v2(&bytes).map_err(model)?;
                let p: projection::ExtensionProjection =
                    serde_json::from_value(value).map_err(|_| err("asset_config"))?;
                if p.fingerprint.validate().is_err()
                    || p.config.fingerprint() != p.fingerprint.fingerprint
                {
                    return Err(err("asset_config"));
                }
                p.config
            } else {
                extensions::ExtensionConfig::default()
            };
            let source = extensions::source_dir::configured_source_dir(&config, &self.home);
            let custom = config.source_store.is_some();
            if custom {
                let targets =
                    CLIENT_IDS.map(|id| ClientTarget::user_target(id, &self.home, true, true));
                extensions::source_dir::validate_source_dir(
                    &source, &self.root, &self.home, &targets,
                )
                .map_err(|_| err("asset_source"))?;
            }
            let mut skills = Vec::new();
            for name in &s.skill_names {
                if !discovery::is_safe_skill_name(name) {
                    return Err(err("asset_identity"));
                }
                let (anchor, path) = if present_path(
                    if custom { &source } else { &self.home },
                    &source.join(name),
                    true,
                )? {
                    (if custom { &source } else { &self.home }, source.join(name))
                } else {
                    (
                        &self.root,
                        discovery::skills_store_dir(&self.root).join(name),
                    )
                };
                if !present_path(anchor, &path, true)? {
                    continue;
                }
                let credential = s.extension_credentials.skill_names.contains(name);
                let actual = portable_assets::snapshot_skill(
                    &path,
                    name,
                    credential,
                    request.assets_reviewed,
                )?;
                if credential && !actual.contains_secrets {
                    s.extension_credentials.skill_names.retain(|n| n != name);
                }
                skills.push(name.clone());
            }
            s.skill_names = skills;
        }
        let mut mcps = Vec::new();
        for selected in &s.mcp_sources {
            let chosen = request.mcp_clients.get(&selected.name).map(String::as_str);
            if chosen.is_some_and(|c| {
                c != "managed"
                    && !CLIENT_IDS
                        .iter()
                        .any(|id| portable_assets::client_key(*id) == c)
            }) {
                return Err(err("capture_scope_invalid"));
            }
            // With a validated choice, mcp_source means the exact selected
            // source has no such object. I/O, parse and provenance errors keep
            // their own errors and must never become absence.
            let detail = match portable_assets::detail(
                &self.root,
                &self.home,
                "mcps",
                &selected.name,
                chosen,
            ) {
                Ok(detail) => detail,
                Err(e) if e.code == "mcp_source" => continue,
                Err(e) => return Err(e),
            };
            if chosen.is_none() && detail.clients.len() != 1 {
                return Err(err("capture_mcp_source_required"));
            }
            let sources = chosen
                .map(|c| BTreeMap::from([(selected.name.clone(), c.to_owned())]))
                .unwrap_or_else(|| {
                    BTreeMap::from([(selected.name.clone(), detail.clients[0].clone())])
                });
            let (actual, conflicts) = portable_assets::export_with_credential_scope(
                &self.root,
                &self.home,
                &[],
                &[selected.name.clone()],
                &sources,
                false,
                request.assets_reviewed,
                Some(&[]),
            )?;
            if !conflicts.is_empty() || actual.mcps.len() != 1 {
                return Err(err("capture_mcp_source_changed"));
            }
            let mut local = selected.clone();
            local.definition_digest =
                super::objects_overlay_v2::mcp_source_digest(&actual.mcps[0])?;
            if actual.mcps[0].secret_fields.is_empty() {
                s.extension_credentials
                    .mcp_names
                    .retain(|n| n != &local.name);
            }
            mcps.push(local);
        }
        s.mcp_sources = mcps;
        s.extension_credentials
            .skill_names
            .retain(|n| s.skill_names.contains(n));
        s.extension_credentials
            .mcp_names
            .retain(|n| s.mcp_sources.iter().any(|m| &m.name == n));
        request
            .mcp_clients
            .retain(|n, _| s.mcp_sources.iter().any(|m| &m.name == n));
        Ok(())
    }
    /// For Download, request.selection uses remote portable IDs and cloud is a
    /// fully validated wrapper. For Upload, request.selection uses actual saved
    /// local IDs; the returned remote scope projects them through known pairs.
    /// MCP clients and reviewed choices always come from this explicit request.
    pub fn freeze_preview(
        self: &Arc<Self>,
        direction: DirectionV2,
        mut request: CaptureRequestV2,
        cloud: Option<&SavedObjects>,
    ) -> ModelResult<MappingPreviewV2> {
        if direction == DirectionV2::NoChange
            || request.selection.usage != UsageV2::RemoteSync
            || !request.selection.manager.webdav_ids.is_empty()
            || !request.selection.manager.webdav_password_ids.is_empty()
            || request.selection.channels.len() > 4096
            || request.selection.skill_names.len() > 500
            || request.selection.mcp_sources.len() > 1000
            || request.selection.manager.account_ids.len() > 1000
        {
            return Err(err("capture_scope_invalid"));
        }
        let before = mapping_bytes(&self.root)?;
        let doc = document(before.as_deref())?;
        let observed = self.observe()?;
        let rows: Vec<_> = doc
            .bindings
            .iter()
            .filter(|r| r.endpoint_binding_id == self.endpoint_binding_id)
            .collect();
        let mut remote = request.selection.clone();
        let mut receive = vec![];
        let mut local_channels = vec![];
        let mut identities = BTreeSet::new();
        for (selection, remote_selection) in
            request.selection.channels.iter().zip(&mut remote.channels)
        {
            let supplied = &selection.portable_channel_id;
            schema_v2::check_id(supplied).map_err(model)?;
            if !identities.insert(supplied.clone()) {
                return Err(err("sync_identity_conflict"));
            }
            let (portable, local) = match direction {
                DirectionV2::Download => {
                    let known = rows.iter().find(|r| r.portable_id == *supplied);
                    let local = known.map_or(supplied.as_str(), |r| r.local_id.as_str());
                    if known.is_none()
                        && (observed.policy.channels.iter().any(|c| c.id == local)
                            || rows.iter().any(|r| r.local_id == local))
                    {
                        return Err(err("sync_identity_explicit_mapping_required"));
                    }
                    (supplied.clone(), local.to_owned())
                }
                DirectionV2::Upload => {
                    if !observed.policy.channels.iter().any(|c| c.id == *supplied) {
                        return Err(err("capture_selection_missing"));
                    }
                    let portable = rows
                        .iter()
                        .find(|r| r.local_id == *supplied)
                        .map_or(supplied.as_str(), |r| r.portable_id.as_str());
                    if rows
                        .iter()
                        .any(|r| r.portable_id == portable && r.local_id != *supplied)
                    {
                        return Err(err("sync_identity_conflict"));
                    }
                    (portable.to_owned(), supplied.clone())
                }
                DirectionV2::NoChange => return Err(err("capture_scope_invalid")),
            };
            remote_selection.portable_channel_id = portable.clone();
            let actual = observed.policy.channels.iter().find(|c| c.id == local);
            let provider_id = if let Some(actual) = actual {
                let mut local_selection = selection.clone();
                local_selection.portable_channel_id = local.clone();
                local_channels.push(local_selection);
                actual.connection.provider_id.clone()
            } else {
                let candidate = format!("sync-v2-{local}");
                if observed
                    .providers
                    .iter()
                    .any(|p| p.eq_ignore_ascii_case(&candidate))
                    || observed
                        .policy
                        .channels
                        .iter()
                        .any(|c| c.connection.provider_id.eq_ignore_ascii_case(&candidate))
                {
                    return Err(err("sync_identity_provider_conflict"));
                }
                candidate
            };
            receive.push(ReceiveBindingV2 {
                portable_channel_id: portable,
                local_channel_id: local,
                provider_id,
                mode: if actual.is_some() {
                    ReceiveBindingModeV2::Existing
                } else {
                    ReceiveBindingModeV2::New
                },
            });
        }
        request.selection.channels = local_channels;
        if direction == DirectionV2::Download {
            let cloud = cloud.ok_or_else(|| err("sync_identity_cloud_required"))?;
            cloud.validate_remote_sync_scope()?;
            let _ = file_objects_v2::encode_document(cloud, Some(&PayloadAuthorizationV2))?;
            receive_models_v2::plan(
                &observed.library,
                &observed.policy,
                cloud.models(),
                &remote,
                &receive,
            )
            .map_err(model)?;
            // New remote revisions are not local capture sources. Keep only
            // the exact revisions positively observed in the saved library.
            request
                .selection
                .template_refs
                .retain(|r| schema_v2::revision(&observed.library, r).is_ok());
            request
                .selection
                .adopt_template_defaults
                .retain(|r| schema_v2::revision(&observed.library, r).is_ok());
            self.local_nonmodel_sources(&mut request)?;
        }
        let captured = self.capture(&request)?;
        let atoms = file_objects_v2::save_state(&captured.objects)?;
        let slots = receive
            .iter()
            .map(|b| {
                let expected = if b.mode == ReceiveBindingModeV2::Existing {
                    TargetExpectationV2::Present(channel_atom(
                        &captured.objects,
                        &atoms,
                        &b.local_channel_id,
                    )?)
                } else {
                    TargetExpectationV2::ExpectedAbsent(absence(
                        &observed,
                        &self.endpoint_binding_id,
                        b,
                    )?)
                };
                Ok(TargetSlotV2 {
                    kind: "channel".into(),
                    portable_id: b.portable_channel_id.clone(),
                    local_id: b.local_channel_id.clone(),
                    expected,
                })
            })
            .collect::<ModelResult<Vec<_>>>()?;
        let binding = LocalTargetBindingV2::freeze(
            map_revision(before.as_deref()),
            request.selection.clone(),
            remote,
            direction,
            slots,
        )
        .map_err(|_| err("capture_scope_invalid"))?;
        if captured.root_revision != observed.root_revision
            || !observed.same_source(&self.observe()?)
            || mapping_bytes(&self.root)? != before
        {
            return Err(err("capture_stale"));
        }
        let facts = Arc::new(FrozenFacts {
            binding,
            request,
            receive,
            map_before: before,
            root_revision: observed.root_revision,
            roster_digest: observed.roster_digest,
            source_digest: observed.source_digest,
            source_guard: captured.source_guard,
            adoption: Mutex::new(None),
        });
        let mut issued = self.issued.lock().map_err(|_| err("sync_identity_read"))?;
        issued.retain(|_, weak| weak.strong_count() > 0);
        if issued.len() >= MAX_PREVIEWS {
            return Err(err("sync_identity_limit"));
        }
        // Equal bindings can have distinct explicit MCP/reviewed requests;
        // never replace an existing live authority with different source facts.
        if let Some(old) = issued.get(&facts.binding.digest).and_then(Weak::upgrade) {
            if old.source_guard != facts.source_guard {
                return Err(err("capture_stale"));
            }
            return Ok(MappingPreviewV2 {
                owner: self.clone(),
                facts: old,
            });
        }
        issued.insert(facts.binding.digest.clone(), Arc::downgrade(&facts));
        Ok(MappingPreviewV2 {
            owner: self.clone(),
            facts,
        })
    }
}
fn channel_atom(
    objects: &SavedObjects,
    atoms: &content_versions::SaveState,
    id: &str,
) -> ModelResult<String> {
    let channel = objects
        .models()
        .payload
        .policy
        .as_ref()
        .and_then(|p| p.channels.iter().find(|c| c.id == id))
        .ok_or_else(|| err("capture_selection_missing"))?;
    let key = content_versions::key(&json!([
        "v2-channel",
        channel.id,
        channel.model_subset,
        channel.family_scope
    ]))?;
    atoms
        .get(&key)
        .cloned()
        .ok_or_else(|| err("capture_selection_missing"))
}
fn absence(observed: &ObservedLocal, namespace: &str, b: &ReceiveBindingV2) -> ModelResult<String> {
    hash(
        &json!({ "hashDomain": "OCXD_LOCAL_CHANNEL_ABSENCE_V2", "rootRevision": observed.root_revision,
        "localId": b.local_channel_id, "providerCandidate": b.provider_id, "namespace": namespace,
        "observedProviderRosterDigest": observed.roster_digest }),
    )
}
impl TrustedMappingOwnerV2 for SyncIdentityOwnerV2 {
    fn verify(&self, binding: &LocalTargetBindingV2) -> ModelResult<()> {
        let facts = self
            .issued
            .lock()
            .map_err(|_| err("sync_identity_read"))?
            .get(&binding.digest)
            .and_then(Weak::upgrade)
            .ok_or_else(|| err("sync_identity_untrusted_preview"))?;
        if facts.binding != *binding {
            return Err(err("sync_identity_untrusted_preview"));
        }
        let before = mapping_bytes(&self.root)?;
        document(before.as_deref())?;
        let observed = self.observe()?;
        if before != facts.map_before
            || map_revision(before.as_deref()) != binding.mapping_revision
            || observed.root_revision != facts.root_revision
            || observed.roster_digest != facts.roster_digest
            || observed.source_digest != facts.source_digest
        {
            return Err(err("capture_stale"));
        }
        let captured = self.capture(&facts.request)?;
        if captured.root_revision != facts.root_revision
            || captured.source_guard != facts.source_guard
        {
            return Err(err("capture_stale"));
        }
        let atoms = file_objects_v2::save_state(&captured.objects)?;
        for (slot, b) in binding.slots.iter().zip(&facts.receive) {
            let actual = observed
                .policy
                .channels
                .iter()
                .find(|c| c.id == slot.local_id);
            let same = match &slot.expected {
                TargetExpectationV2::Present(d) => {
                    actual.is_some_and(|c| c.connection.provider_id == b.provider_id)
                        && channel_atom(&captured.objects, &atoms, &slot.local_id)? == *d
                }
                TargetExpectationV2::ExpectedAbsent(d) => {
                    actual.is_none()
                        && !observed
                            .providers
                            .iter()
                            .any(|p| p.eq_ignore_ascii_case(&b.provider_id))
                        && absence(&observed, &self.endpoint_binding_id, b)? == *d
                }
            };
            if !same {
                return Err(err("capture_stale"));
            }
        }
        if !observed.same_source(&self.observe()?) || mapping_bytes(&self.root)? != before {
            return Err(err("capture_stale"));
        }
        Ok(())
    }
    fn project_saved_to_remote(
        &self,
        objects: SavedObjects,
        binding: &LocalTargetBindingV2,
    ) -> ModelResult<SavedObjects> {
        self.verify(binding)?;
        let facts = self
            .issued
            .lock()
            .map_err(|_| err("sync_identity_read"))?
            .get(&binding.digest)
            .and_then(Weak::upgrade)
            .ok_or_else(|| err("sync_identity_untrusted_preview"))?;
        let actual = self.capture(&facts.request)?;
        if file_objects_v2::semantic_digest(&objects)? != actual.semantic_digest
            || file_objects_v2::save_state(&objects)? != actual.save_state
        {
            return Err(err("sync_identity_projection_source"));
        }
        let CapturedObjects {
            mut models,
            manager,
            assets,
            channel_keys,
            channel_authorizations,
            webdav_passwords,
            content_versions,
        } = objects.into_captured();
        let pairs: BTreeMap<_, _> = binding
            .slots
            .iter()
            .filter(|s| matches!(s.expected, TargetExpectationV2::Present(_)))
            .map(|s| (s.local_id.as_str(), s.portable_id.as_str()))
            .collect();
        let rename = |id: &str| {
            pairs
                .get(id)
                .map(|v| (*v).to_owned())
                .ok_or_else(|| err("sync_identity_projection_scope"))
        };
        if let Some(policy) = &mut models.payload.policy {
            for channel in &mut policy.channels {
                // Namespace adoption is intentionally unsupported by this
                // minimal owner, matching the controlled receive rule.
                if channel.namespace.is_some() {
                    return Err(err("model_receive_namespace_adoption_required"));
                }
                channel.id = rename(&channel.id)?;
            }
            for id in &mut policy.channel_order {
                *id = rename(id)?;
            }
            if let Some(id) = &mut policy.default_channel_id {
                *id = rename(id)?;
            }
        }
        let mut keys = SecretValues::new();
        for (id, value) in channel_keys.into_entries() {
            keys.insert(rename(&id)?, value)?;
        }
        let mut auth = BTreeMap::new();
        for (id, value) in channel_authorizations {
            if auth.insert(rename(&id)?, value).is_some() {
                return Err(err("sync_identity_conflict"));
            }
        }
        models.digest = hash(&models.payload)?;
        portable_v2::validate_portable_models_v2(&models).map_err(model)?;
        let projected = SavedObjects::capture(
            CapturedObjects {
                models,
                manager,
                assets,
                channel_keys: keys,
                channel_authorizations: auth,
                webdav_passwords,
                content_versions,
            },
            Some(&PayloadAuthorizationV2),
        )?;
        projected.validate_remote_sync_scope()?;
        self.verify(binding)?;
        Ok(projected)
    }
}

#[cfg(test)]
#[path = "sync_identity_v2_tests.rs"]
mod tests;
