//! Trusted saved-source capture. D/H are supplied by L0; never discover roots.
//! Two equal observations freeze an optimistic snapshot, not a global atomic
//! transaction with external clients. Secret-bearing results are not IPC DTOs.
use super::{
    content_versions::{self, SaveState, TimeKind},
    file_objects_v2::{
        self, AuthorizationValidator, CapturedObjects, PublicSummary, SavedObjects, SecretValues,
    },
    objects_overlay_v2::{self, SelectionV2, UsageV2},
    portable_v2::{self, ModelExportScopeV2, PortableChannelV2},
    protocol_groups::MappingFileV2,
    provider_auth::{self, PortableAuthorization},
    schema::{invalid, ModelError, ModelResult},
    schema_v2::{self, ChannelV2, LibraryV2},
    sync_v2::types::SourceGuard,
};
use crate::modules::{
    backup::safety,
    sync::{portable_assets, portable_manager},
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureRequestV2 {
    #[serde(deserialize_with = "explicit_selection")]
    pub selection: SelectionV2,
    pub mcp_clients: BTreeMap<String, String>,
    pub assets_reviewed: bool,
}
// Overlay defaults must not turn an omitted capture choice into authority.
fn explicit_selection<'de, D: Deserializer<'de>>(d: D) -> Result<SelectionV2, D::Error> {
    let value = Value::deserialize(d)?;
    let required = [
        "channels",
        "templateRefs",
        "adoptTemplateDefaults",
        "includeGlobalTemplateRefs",
        "manager",
        "skillNames",
        "mcpSources",
        "extensionCredentials",
        "usage",
    ];
    let explicit = |v: &Value, keys: &[&str]| keys.iter().all(|k| v.get(k).is_some());
    if !explicit(&value, &required)
        || !explicit(
            &value["manager"],
            &[
                "preferences",
                "extensions",
                "webdavIds",
                "webdavPasswordIds",
                "accountIds",
            ],
        )
        || !explicit(&value["extensionCredentials"], &["skillNames", "mcpNames"])
        || value["channels"].as_array().is_some_and(|cs| {
            cs.iter().any(|c| {
                !explicit(
                    c,
                    &[
                        "portableChannelId",
                        "modelIds",
                        "familyIds",
                        "includeConnection",
                        "includeKey",
                        "includeAuthorization",
                    ],
                )
            })
        })
    {
        return Err(serde::de::Error::custom(
            "explicit capture selection required",
        ));
    }
    serde_json::from_value(value).map_err(|_| serde::de::Error::custom("invalid capture selection"))
}

/// Host-only frozen objects. Never derive Debug or Serialize.
pub struct CapturedSnapshotV2 {
    pub objects: SavedObjects,
    pub source_guard: SourceGuard,
    pub root_revision: String,
    pub semantic_digest: String,
    pub save_state: SaveState,
    pub summary: PublicSummary,
}
/// Trusted selected Key evidence, constructed only from L0's actual provider
/// row while its config/Store locks are held. No metadata credential_ref and no
/// draft value enters this boundary. Never Debug/Serialize or return over IPC.
pub struct SelectedKeySourceV2<'a> {
    provider: &'a str,
    value: &'a str,
    scoped_public: &'a Value,
    actual_source_public: &'a Value,
    raw_digest: &'a str,
}
impl SelectedKeySourceV2<'_> {
    pub fn provider(&self) -> &str {
        self.provider
    }
    pub fn value(&self) -> &str {
        self.value
    }
    pub fn scoped_public(&self) -> &Value {
        self.scoped_public
    }
    pub fn actual_source_public(&self) -> &Value {
        self.actual_source_public
    }
    pub fn raw_digest(&self) -> &str {
        self.raw_digest
    }
}
/// Called only for selected credentials, using actual saved identities.
pub trait CaptureCredentialsV2 {
    fn channel_key(
        &self,
        home: &Path,
        source: &SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        if let Some(account) = source.value.strip_prefix("keychain:") {
            return self.keychain_key(home, source, account);
        }
        if let Some(name) = environment_name(source.value)? {
            return self.environment_key(home, source, name);
        }
        Ok(Zeroizing::new(source.value.to_owned()))
    }
    /// Invoked only after the original reference's owner and source direction
    /// have been checked. Implementations must not fall back to manager vaults.
    fn keychain_key(
        &self,
        _home: &Path,
        _source: &SelectedKeySourceV2<'_>,
        _account: &str,
    ) -> ModelResult<Zeroizing<String>> {
        Err(err("capture_credential_missing"))
    }
    fn environment_key(
        &self,
        _home: &Path,
        _source: &SelectedKeySourceV2<'_>,
        _name: &str,
    ) -> ModelResult<Zeroizing<String>> {
        Err(err("capture_credential_missing"))
    }
    fn authorization(
        &self,
        _home: &Path,
        _provider: &str,
        _scoped_public: &Value,
        _actual_source_public: &Value,
    ) -> ModelResult<PortableAuthorization> {
        Err(err("capture_credential_missing"))
    }
    fn webdav_password(&self, _root: &Path, _endpoint_id: &str) -> ModelResult<Zeroizing<String>> {
        Err(err("capture_credential_missing"))
    }
}
pub struct SystemCaptureCredentialsV2;
impl CaptureCredentialsV2 for SystemCaptureCredentialsV2 {
    fn keychain_key(
        &self,
        _home: &Path,
        source: &SelectedKeySourceV2<'_>,
        account: &str,
    ) -> ModelResult<Zeroizing<String>> {
        if source.value.strip_prefix("keychain:") != Some(account) {
            return Err(err("provider_auth_reference"));
        }
        owned_key_reference(source.value, source.provider)?;
        keyring::Entry::new("opencodex.provider-api-key.v1", account)
            .map_err(|_| err("provider_auth_reference"))?
            .get_password()
            .map(Zeroizing::new)
            .map_err(|_| err("provider_auth_reference"))
    }
    fn environment_key(
        &self,
        _home: &Path,
        source: &SelectedKeySourceV2<'_>,
        name: &str,
    ) -> ModelResult<Zeroizing<String>> {
        if environment_name(source.value)? != Some(name) {
            return Err(err("provider_auth_reference"));
        }
        std::env::var(name)
            .map(Zeroizing::new)
            .map_err(|_| err("provider_auth_reference"))
    }
    fn authorization(
        &self,
        home: &Path,
        provider: &str,
        scoped: &Value,
        source: &Value,
    ) -> ModelResult<PortableAuthorization> {
        provider_auth::export_portable_scoped(home, provider, scoped, source)
    }
    fn webdav_password(&self, root: &Path, endpoint_id: &str) -> ModelResult<Zeroizing<String>> {
        use crate::{
            infrastructure::{credentials, keychain::WEBDAV_CREDENTIAL_PURPOSE},
            modules::sync::config::SyncConfigStore,
        };
        let config = SyncConfigStore::new(root)
            .load()
            .map_err(|_| err("capture_source_invalid"))?;
        let endpoint = config
            .endpoints
            .iter()
            .find(|e| e.endpoint_id == endpoint_id)
            .ok_or_else(|| err("capture_selection_missing"))?;
        // Endpoint identity is not a credential reference. Resolve only the
        // actual saved endpoint's ref in this injected D; no global root/cache.
        credentials::load_optional_at(
            root,
            WEBDAV_CREDENTIAL_PURPOSE,
            &endpoint.credential_ref.ref_id,
        )
        .map_err(|_| err("capture_credential_source_invalid"))?
        .ok_or_else(|| err("capture_credential_missing"))
    }
}
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.trim() == key && key.len() <= 8192 && !key.contains(['\r', '\n'])
}
fn environment_name(value: &str) -> ModelResult<Option<&str>> {
    let Some(name) = value.strip_prefix('$') else {
        return Ok(None);
    };
    let name = if let Some(braced) = name.strip_prefix('{') {
        braced
            .strip_suffix('}')
            .ok_or_else(|| err("provider_auth_reference"))?
    } else {
        name
    };
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(err("provider_auth_reference"));
    }
    Ok(Some(name))
}
fn owned_key_reference(value: &str, provider: &str) -> ModelResult<()> {
    if let Some(account) = value.strip_prefix("keychain:") {
        if account != provider
            && account
                .strip_prefix(provider)
                .and_then(|s| s.strip_prefix('/'))
                .is_none_or(|s| s.is_empty())
        {
            return Err(err("provider_auth_reference"));
        }
    }
    environment_name(value)?;
    Ok(())
}
fn selected_key_source<'a>(
    provider: &'a str,
    row: &'a Value,
    scoped: &'a Value,
    public: &'a Value,
    raw_digest: &'a str,
) -> ModelResult<SelectedKeySourceV2<'a>> {
    if !schema_v2::valid_digest(raw_digest)
        || super::provider_fields::public_from_raw(row)? != *public
    {
        return Err(err("capture_credential_scope"));
    }
    for field in ["adapter", "baseUrl", "authMode", "apiKeyTransport"] {
        if scoped.get(field) != public.get(field) {
            return Err(err("capture_credential_scope"));
        }
    }
    if row.get("authMode").is_some_and(|mode| mode != "key") {
        return Err(err("capture_key_selection_unsupported"));
    }
    let active = match row.get("apiKey") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if s.is_empty() => None,
        Some(Value::String(s)) if valid_key(s) => Some(s.as_str()),
        _ => return Err(err("capture_key_source_invalid")),
    };
    let pool = match row.get("apiKeyPool") {
        None | Some(Value::Null) => &[][..],
        Some(Value::Array(a)) if a.len() <= 256 => a.as_slice(),
        _ => return Err(err("capture_key_source_invalid")),
    };
    let mut keys = Vec::new();
    let mut ids = BTreeSet::new();
    for slot in pool {
        if slot.as_object().is_none_or(|fields| {
            fields
                .keys()
                .any(|key| !["id", "key", "label", "addedAt"].contains(&key.as_str()))
        }) {
            return Err(err("capture_key_selection_unsupported"));
        }
        let key = slot
            .get("key")
            .and_then(Value::as_str)
            .filter(|s| valid_key(s))
            .ok_or_else(|| err("capture_key_source_invalid"))?;
        let id = slot
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| err("capture_key_source_invalid"))?;
        let hash = schema_v2::digest(key.as_bytes());
        if id.len() != 8
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !ids.insert(id)
            || (id != &hash[7..15] && key != format!("keychain:{provider}/{id}"))
        {
            return Err(err("capture_key_source_invalid"));
        }
        keys.push(key);
    }
    let value = if let Some(active) = active {
        if !keys.is_empty() && !keys.contains(&active) {
            return Err(err("capture_key_source_invalid"));
        }
        active
    } else {
        match keys.as_slice() {
            [] => return Err(err("capture_credential_missing")),
            [only] => only,
            _ => return Err(err("capture_key_selection_unsupported")),
        }
    };
    owned_key_reference(value, provider)?;
    Ok(SelectedKeySourceV2 {
        provider,
        value,
        scoped_public: scoped,
        actual_source_public: public,
        raw_digest,
    })
}
struct ActualAuthorization;
impl AuthorizationValidator for ActualAuthorization {
    fn validate(&self, c: &PortableChannelV2, a: &PortableAuthorization) -> ModelResult<()> {
        provider_auth::validate_portable(
            c.connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
                .ok_or_else(|| err("capture_credential_scope"))?,
            a,
        )
    }
}
fn err(code: &str) -> ModelError {
    invalid(code, "已保存源捕获失败；请重新核对所选来源")
}
fn mapped(e: schema_v2::MatchErrorV2) -> ModelError {
    err(&e.code)
}
fn hashed<T: Serialize>(value: &T) -> ModelResult<String> {
    Ok(schema_v2::digest(
        &schema_v2::canonical_bytes(value).map_err(mapped)?,
    ))
}
fn unique(values: &[String], limit: usize) -> ModelResult<()> {
    if values.len() > limit
        || values.iter().any(|s| s.is_empty() || s.len() > 8192)
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return Err(err("capture_scope_invalid"));
    }
    Ok(())
}
pub(crate) fn validate_request(r: &CaptureRequestV2) -> ModelResult<()> {
    let s = &r.selection;
    if s.channels.len() > 4096
        || s.template_refs.len() > 4096
        || s.adopt_template_defaults.len() > 4096
    {
        return Err(err("capture_scope_invalid"));
    }
    unique(
        &s.channels
            .iter()
            .map(|c| c.portable_channel_id.clone())
            .collect::<Vec<_>>(),
        4096,
    )?;
    for c in &s.channels {
        schema_v2::check_id(&c.portable_channel_id).map_err(mapped)?;
        if c.model_ids.is_some() && c.family_ids.is_some()
            || c.include_key && c.include_authorization
            || (c.include_key || c.include_authorization) && !c.include_connection
            || c.family_ids.is_some() && c.include_connection
        {
            return Err(err("capture_scope_invalid"));
        }
        for ids in [&c.model_ids, &c.family_ids].into_iter().flatten() {
            unique(ids, 20000)?;
            if ids.is_empty() {
                return Err(err("capture_scope_invalid"));
            }
        }
    }
    unique(&s.skill_names, 500)?;
    unique(
        &s.mcp_sources
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>(),
        1000,
    )?;
    unique(&s.extension_credentials.skill_names, 500)?;
    unique(&s.extension_credentials.mcp_names, 1000)?;
    unique(&s.manager.account_ids, 1000)?;
    unique(&s.manager.webdav_ids, 256)?;
    unique(&s.manager.webdav_password_ids, 256)?;
    if s.extension_credentials
        .skill_names
        .iter()
        .any(|n| !s.skill_names.contains(n))
        || s.extension_credentials
            .mcp_names
            .iter()
            .any(|n| !s.mcp_sources.iter().any(|m| &m.name == n))
        || s.manager
            .webdav_password_ids
            .iter()
            .any(|id| !s.manager.webdav_ids.contains(id))
        || r.mcp_clients.iter().any(|(n, c)| {
            c.is_empty() || c.len() > 128 || !s.mcp_sources.iter().any(|m| &m.name == n)
        })
    {
        return Err(err("capture_scope_invalid"));
    }
    if s.usage == UsageV2::RemoteSync
        && (!s.manager.webdav_ids.is_empty() || !s.manager.webdav_password_ids.is_empty())
    {
        return Err(err("sync_self_scope"));
    }
    for m in &s.mcp_sources {
        if !schema_v2::valid_digest(&m.definition_digest) {
            return Err(err("capture_scope_invalid"));
        }
    }
    unique(
        &s.adopt_template_defaults
            .iter()
            .map(|r| r.template_id.clone())
            .collect::<Vec<_>>(),
        4096,
    )?;
    for refs in [&s.template_refs, &s.adopt_template_defaults] {
        if refs
            .iter()
            .map(|r| (&r.template_id, &r.revision_id))
            .collect::<BTreeSet<_>>()
            .len()
            != refs.len()
        {
            return Err(err("capture_scope_invalid"));
        }
    }
    Ok(())
}
pub(crate) fn scope_digest(r: &CaptureRequestV2) -> ModelResult<String> {
    let mut s = r.selection.clone();
    s.channels
        .sort_by(|a, b| a.portable_channel_id.cmp(&b.portable_channel_id));
    for c in &mut s.channels {
        if let Some(v) = &mut c.model_ids {
            v.sort();
        }
        if let Some(v) = &mut c.family_ids {
            v.sort();
        }
    }
    for v in [&mut s.template_refs, &mut s.adopt_template_defaults] {
        v.sort_by(|a, b| {
            (&a.template_id, &a.revision_id, &a.digest).cmp(&(
                &b.template_id,
                &b.revision_id,
                &b.digest,
            ))
        });
    }
    s.skill_names.sort();
    s.mcp_sources.sort_by(|a, b| a.name.cmp(&b.name));
    s.extension_credentials.skill_names.sort();
    s.extension_credentials.mcp_names.sort();
    s.manager.account_ids.sort();
    s.manager.webdav_ids.sort();
    s.manager.webdav_password_ids.sort();
    hashed(
        &json!({"format":"OCXD_CAPTURE_SCOPE_V2", "selection":s,"reviewed":r.assets_reviewed,"mcpClients":r.mcp_clients}),
    )
}
/// Copy original per-model public values from every actual protocol group,
/// including unknown nested children. Never rebuild them from ModelRules.
pub(crate) fn public_source(
    raw: &Value,
    channel: &ChannelV2,
    mappings: &MappingFileV2,
) -> ModelResult<Value> {
    let provider = |id: &str| {
        raw.get("providers")
            .and_then(|p| p.get(id))
            .ok_or_else(|| err("capture_selection_missing"))
    };
    let mut public =
        super::provider_fields::public_from_raw(provider(&channel.connection.provider_id)?)?;
    let fields = public
        .as_object_mut()
        .ok_or_else(|| err("provider_fields"))?;
    for mapping in mappings
        .mappings
        .iter()
        .filter(|m| m.channel_id == channel.id && m.provider_id != channel.connection.provider_id)
    {
        let group = super::provider_fields::public_from_raw(provider(&mapping.provider_id)?)?;
        for (key, value) in group.as_object().ok_or_else(|| err("provider_fields"))? {
            if (key.starts_with("model") && key != "modelPreset")
                || key == "autoReviewModelOverrides"
            {
                if let Some(map) = value.as_object() {
                    let target = fields
                        .entry(key.clone())
                        .or_insert_with(|| json!({}))
                        .as_object_mut()
                        .ok_or_else(|| err("capture_public_group_conflict"))?;
                    for (id, original) in map {
                        if target.get(id).is_some_and(|old| old != original) {
                            return Err(err("capture_public_group_conflict"));
                        }
                        target.insert(id.clone(), original.clone());
                    }
                } else if key == "models" {
                    merge_list(fields, key, value)?;
                } else if fields.get(key) != Some(value) {
                    return Err(err("capture_public_group_conflict"));
                }
            } else if key == "retainModels" || key == "selectedModels" || key.ends_with("Models") {
                merge_list(fields, key, value)?;
            }
        }
    }
    Ok(public)
}
fn merge_list(
    fields: &mut serde_json::Map<String, Value>,
    key: &str,
    value: &Value,
) -> ModelResult<()> {
    let list = value
        .as_array()
        .ok_or_else(|| err("capture_public_group_conflict"))?;
    let target = fields
        .entry(key.to_owned())
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| err("capture_public_group_conflict"))?;
    for v in list {
        if !v.is_string() {
            return Err(err("capture_public_group_conflict"));
        }
        if !target.contains(v) {
            target.push(v.clone());
        }
    }
    Ok(())
}
struct ModelRead {
    captured: CapturedObjects,
    root_revision: String,
    observation: String,
}
fn models(
    root: &Path,
    home: &Path,
    r: &CaptureRequestV2,
    credentials: &dyn CaptureCredentialsV2,
) -> ModelResult<ModelRead> {
    // Nest ModelResult so the callback never serializes raw values into errors.
    super::config_file::v2::with_saved_config(
        root,
        home,
        |library, policy, mappings, raw, raw_digest, revision| {
            Ok((|| -> ModelResult<ModelRead> {
                let mut scope = ModelExportScopeV2 {
                    template_refs: r.selection.template_refs.clone(),
                    include_global: r.selection.include_global_template_refs,
                    ..Default::default()
                };
                scope
                    .template_refs
                    .extend(r.selection.adopt_template_defaults.clone());
                for c in &r.selection.channels {
                    match (&c.model_ids, &c.family_ids) {
                        (Some(ids), None) => {
                            scope.channel_models.insert(
                                c.portable_channel_id.clone(),
                                ids.iter().cloned().collect(),
                            );
                        }
                        (None, Some(ids)) => {
                            scope.channel_families.insert(
                                c.portable_channel_id.clone(),
                                ids.iter().cloned().collect(),
                            );
                        }
                        (None, None) => {
                            scope.channel_ids.insert(c.portable_channel_id.clone());
                        }
                        _ => return Err(err("capture_scope_invalid")),
                    }
                }
                let mut package = portable_v2::export_portable_models_v2(library, policy, &scope)
                    .map_err(mapped)?;
                exact_revisions(&mut package, library, &r.selection)?;
                let mut sources = BTreeMap::new();
                for c in &r.selection.channels {
                    let actual = policy
                        .channels
                        .iter()
                        .find(|a| a.id == c.portable_channel_id)
                        .ok_or_else(|| err("capture_selection_missing"))?;
                    if let Some(ids) = &c.family_ids {
                        for id in ids {
                            if !actual.family_overrides.contains_key(id)
                                && !actual.template_refs.iter().any(|r| &r.template_id == id)
                                && !actual.models.iter().any(|m| {
                                    m.binding
                                        .as_ref()
                                        .is_some_and(|b| &b.reference().template_id == id)
                                })
                            {
                                return Err(err("capture_selection_missing"));
                            }
                        }
                    }
                    let public = public_source(raw, actual, mappings)?;
                    if c.family_ids.is_none() {
                        sources.insert(actual.id.clone(), public);
                    }
                }
                package =
                    portable_v2::attach_public_sources_v2(&package, &sources).map_err(mapped)?;
                let mut keys = SecretValues::new();
                let mut auth = BTreeMap::new();
                for c in &r.selection.channels {
                    let actual = policy
                        .channels
                        .iter()
                        .find(|a| a.id == c.portable_channel_id)
                        .ok_or_else(|| err("capture_selection_missing"))?;
                    if c.include_key {
                        let scoped = package
                            .payload
                            .policy
                            .as_ref()
                            .and_then(|p| p.channels.iter().find(|p| p.id == actual.id))
                            .and_then(|p| p.connection.as_ref())
                            .and_then(|p| p.provider.as_ref())
                            .ok_or_else(|| err("capture_credential_scope"))?;
                        let row = &raw["providers"][&actual.connection.provider_id];
                        let source_public = super::provider_fields::public_from_raw(row)?;
                        let source = selected_key_source(
                            &actual.connection.provider_id,
                            row,
                            scoped,
                            &source_public,
                            raw_digest,
                        )?;
                        let key = credentials
                            .channel_key(home, &source)
                            .map_err(|e| err(&e.code))?;
                        if !valid_key(&key) || key.starts_with('$') || key.starts_with("keychain:")
                        {
                            return Err(err("capture_key_source_invalid"));
                        }
                        keys.insert(actual.id.clone(), key)?;
                    }
                    if c.include_authorization {
                        let scoped = package
                            .payload
                            .policy
                            .as_ref()
                            .and_then(|p| p.channels.iter().find(|p| p.id == actual.id))
                            .and_then(|p| p.connection.as_ref())
                            .and_then(|p| p.provider.as_ref())
                            .ok_or_else(|| err("capture_credential_scope"))?;
                        auth.insert(
                            actual.id.clone(),
                            credentials
                                .authorization(
                                    home,
                                    &actual.connection.provider_id,
                                    scoped,
                                    &super::provider_fields::public_from_raw(
                                        &raw["providers"][&actual.connection.provider_id],
                                    )?,
                                )
                                .map_err(|e| err(&e.code))?,
                        );
                    }
                    if !c.include_connection {
                        package
                            .payload
                            .policy
                            .as_mut()
                            .unwrap()
                            .channels
                            .iter_mut()
                            .find(|p| p.id == actual.id)
                            .unwrap()
                            .connection = None;
                    }
                }
                package.digest = hashed(&package.payload)?;
                Ok(ModelRead {
                    captured: CapturedObjects {
                        models: package,
                        manager: Default::default(),
                        assets: Default::default(),
                        channel_keys: keys,
                        channel_authorizations: auth,
                        webdav_passwords: SecretValues::new(),
                        content_versions: vec![],
                    },
                    root_revision: revision.into(),
                    observation: hashed(&json!([raw_digest, revision, library, mappings, policy]))?,
                })
            })())
        },
    )
    .map_err(mapped)?
}
pub(crate) fn exact_revisions(
    package: &mut portable_v2::PortableModelsV2,
    library: &LibraryV2,
    s: &SelectionV2,
) -> ModelResult<()> {
    let mut refs = s.template_refs.clone();
    refs.extend(s.adopt_template_defaults.clone());
    if let Some(p) = &package.payload.policy {
        refs.extend(p.template_refs.clone());
        for c in &p.channels {
            refs.extend(c.template_refs.clone());
            refs.extend(
                c.models
                    .iter()
                    .filter_map(|m| m.binding.as_ref().map(|b| b.reference().clone())),
            );
        }
    }
    for r in &refs {
        schema_v2::revision(library, r).map_err(mapped)?;
    }
    for r in &s.adopt_template_defaults {
        if library
            .templates
            .iter()
            .find(|t| t.id == r.template_id)
            .and_then(|t| t.default_revision.as_ref())
            != Some(&r.revision_id)
        {
            return Err(err("capture_default_source_changed"));
        }
    }
    for t in &mut package.payload.templates {
        t.revisions.retain(|v| {
            refs.iter()
                .any(|r| r.template_id == t.id && r.revision_id == v.id)
        });
        if !s
            .adopt_template_defaults
            .iter()
            .any(|r| r.template_id == t.id)
        {
            t.default_revision = None;
        }
    }
    package.digest = hashed(&package.payload)?;
    Ok(())
}
fn require_file(root: &Path, relative: &str) -> ModelResult<()> {
    let path = root.join(relative);
    safety::check_path(root, &path, false).map_err(|_| err("capture_source_invalid"))?;
    if !path.is_file() {
        return Err(err("capture_selection_missing"));
    }
    Ok(())
}
fn assets(
    root: &Path,
    home: &Path,
    r: &CaptureRequestV2,
) -> ModelResult<(portable_assets::Assets, String)> {
    if r.selection.skill_names.is_empty() && r.selection.mcp_sources.is_empty() {
        return Ok((Default::default(), hashed(&json!([]))?));
    }
    let mut clients = BTreeMap::new();
    for m in &r.selection.mcp_sources {
        let detail = portable_assets::detail(
            root,
            home,
            "mcps",
            &m.name,
            r.mcp_clients.get(&m.name).map(String::as_str),
        )?;
        let chosen = if let Some(c) = r.mcp_clients.get(&m.name) {
            if !detail.clients.contains(c) {
                return Err(err("capture_mcp_source_changed"));
            }
            c.clone()
        } else if detail.clients.len() == 1 {
            detail.clients[0].clone()
        } else {
            return Err(err("capture_mcp_source_required"));
        };
        clients.insert(m.name.clone(), chosen);
    }
    let credential_ids: Vec<_> = r
        .selection
        .extension_credentials
        .skill_names
        .iter()
        .map(|n| format!("skill:{n}"))
        .chain(
            r.selection
                .extension_credentials
                .mcp_names
                .iter()
                .map(|n| format!("mcp:{n}")),
        )
        .collect();
    let sources = clients
        .iter()
        .map(|(n, c)| (n.clone(), c.clone()))
        .collect();
    let (assets, conflicts) = portable_assets::export_with_credential_scope(
        root,
        home,
        &r.selection.skill_names,
        &r.selection
            .mcp_sources
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>(),
        &sources,
        !credential_ids.is_empty(),
        r.assets_reviewed,
        Some(&credential_ids),
    )?;
    if !conflicts.is_empty()
        || assets.skills.len() != r.selection.skill_names.len()
        || assets.mcps.len() != r.selection.mcp_sources.len()
    {
        return Err(err("capture_selection_missing"));
    }
    let mut bindings = BTreeMap::new();
    for m in &assets.mcps {
        let digest = objects_overlay_v2::mcp_source_digest(m)?;
        if !r
            .selection
            .mcp_sources
            .iter()
            .any(|s| s.name == m.name && s.definition_digest == digest)
        {
            return Err(err("capture_mcp_source_changed"));
        }
        if r.selection
            .extension_credentials
            .mcp_names
            .contains(&m.name)
            && m.secrets.0.is_empty()
        {
            return Err(err("capture_credential_missing"));
        }
        bindings.insert(m.name.clone(), json!([clients[&m.name], digest]));
    }
    for s in &assets.skills {
        if r.selection
            .extension_credentials
            .skill_names
            .contains(&s.name)
            && !s.contains_secrets
        {
            return Err(err("capture_credential_missing"));
        }
    }
    Ok((assets, hashed(&bindings)?))
}
struct Observation {
    snapshot: CapturedSnapshotV2,
    sources: String,
}
fn once(
    root: &Path,
    home: &Path,
    r: &CaptureRequestV2,
    credentials: &dyn CaptureCredentialsV2,
) -> ModelResult<Observation> {
    let mut model = models(root, home, r, credentials)?;
    let s = &r.selection;
    if s.manager.preferences {
        require_file(root, crate::modules::preferences::PREFERENCES_RELATIVE_PATH)?;
    }
    if s.manager.extensions {
        require_file(
            root,
            crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
        )?;
    }
    let scope = portable_manager::Scope {
        preferences: s.manager.preferences,
        extensions: s.manager.extensions,
        webdav: !s.manager.webdav_ids.is_empty(),
        account_ids: s.manager.account_ids.clone(),
        upstream_root: Some(home.into()),
        local_home: Some(home.into()),
        ..Default::default()
    };
    let mut manager = portable_manager::export(root, &scope).map_err(|e| err(&e.code))?;
    for id in &s.manager.webdav_ids {
        if !manager.webdav.iter().any(|e| &e.id == id) {
            return Err(err("capture_selection_missing"));
        }
    }
    manager
        .webdav
        .retain(|e| s.manager.webdav_ids.contains(&e.id));
    for id in &s.manager.webdav_password_ids {
        model.captured.webdav_passwords.insert(
            id.clone(),
            credentials
                .webdav_password(root, id)
                .map_err(|e| err(&e.code))?,
        )?;
    }
    model.captured.manager = manager;
    let (assets, mcp_sources_digest) = assets(root, home, r).map_err(|e| err(&e.code))?;
    model.captured.assets = assets;
    let objects = SavedObjects::capture(model.captured, Some(&ActualAuthorization))?;
    let save_state = file_objects_v2::save_state(&objects)?;
    let semantic_digest = file_objects_v2::semantic_digest(&objects)?;
    let saved_source_digest = hashed(&json!([
        "OCXD_SAVED_CAPTURE_V2",
        save_state,
        semantic_digest
    ]))?;
    let source_guard = SourceGuard {
        saved_source_digest,
        scope_digest: scope_digest(r)?,
        mcp_sources_digest,
    };
    let mut captured = objects.into_captured();
    captured.content_versions = content_versions::observe(root, &save_state)?
        .into_iter()
        .filter(|v| {
            v.time_kind != TimeKind::Unknown
                && save_state.get(&v.scope_key) == Some(&v.content_digest)
        })
        .collect();
    let objects = SavedObjects::capture(captured, Some(&ActualAuthorization))?;
    let final_source = super::config_file::v2::with_saved_config(
        root,
        home,
        |library, policy, mappings, _, raw_digest, revision| {
            schema_v2::canonical_bytes(&json!([raw_digest, revision, library, mappings, policy]))
                .map(|b| schema_v2::digest(&b))
        },
    )
    .map_err(mapped)?;
    if final_source != model.observation {
        return Err(err("capture_stale"));
    }
    let summary = objects.summary();
    Ok(Observation {
        snapshot: CapturedSnapshotV2 {
            objects,
            source_guard,
            root_revision: model.root_revision,
            semantic_digest,
            save_state,
            summary,
        },
        sources: model.observation,
    })
}
/// At most three observations. Mismatches retry once; no time or nonce enters
/// comparison. External writes after the final observation remain possible.
pub fn capture(
    root: &Path,
    home: &Path,
    request: &CaptureRequestV2,
    credentials: &dyn CaptureCredentialsV2,
) -> ModelResult<CapturedSnapshotV2> {
    validate_request(request)?;
    if !root.is_absolute() || !root.is_dir() || !home.is_absolute() || !home.is_dir() {
        return Err(err("capture_root_invalid"));
    }
    let mut previous = once(root, home, request, credentials)?;
    for _ in 1..3 {
        let current = once(root, home, request, credentials).map_err(|_| err("capture_stale"))?;
        if previous.sources == current.sources
            && previous.snapshot.root_revision == current.snapshot.root_revision
            && previous.snapshot.source_guard == current.snapshot.source_guard
            && previous.snapshot.semantic_digest == current.snapshot.semantic_digest
            && previous.snapshot.save_state == current.snapshot.save_state
        {
            return Ok(current.snapshot);
        }
        previous = current;
    }
    Err(err("capture_stale"))
}
#[cfg(test)]
#[path = "saved_objects_capture_v2_tests.rs"]
mod tests;
