//! Captured portable configuration objects. Pure memory codec: no filesystem,
//! keychain, discovery, application, or implicit multi-baseline merge.
//!
//! This is a NEW document domain, OCXD_CONFIG_OBJECTS_V2/version 2, not an old
//! OCXD_CONFIG_OBJECTS/version 2 reader. Only L1's real v6 model validator owns
//! source/explicit/ref semantics. Secret-bearing types deliberately implement
//! neither Debug nor Serialize; only the private wire and public counts do.
use super::{
    content_versions::{TimeKind, Version},
    portable_v2::{self, PortableChannelV2, PortableModelsV2},
    provider_auth::PortableAuthorization,
    schema::{invalid, ModelResult},
    sync_v2::types::{ModelSchemaRef, PayloadSchemaRef, SchemaRef},
};
use crate::modules::sync::{portable_assets, portable_file, portable_manager};
use serde::{
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Write,
};
use zeroize::{Zeroize, Zeroizing};

pub const DOCUMENT_FORMAT: &str = "OCXD_CONFIG_OBJECTS_V2";
pub const DOCUMENT_VERSION: u32 = 2;
pub const DOCUMENT_LIMIT: usize = portable_file::DOCUMENT_LIMIT;
pub const FILE_LIMIT: usize = portable_file::FILE_LIMIT;
const SECRET_LIMIT: usize = 8192;
const MAX_VERSIONS: usize = 30000;

/// The one supported combination. Callers must not hardcode old Objects v2.
pub fn supported_schema() -> PayloadSchemaRef {
    PayloadSchemaRef {
        envelope: SchemaRef {
            format: "OCXDCONF".into(),
            version: 3,
        },
        document: SchemaRef {
            format: DOCUMENT_FORMAT.into(),
            version: u64::from(DOCUMENT_VERSION),
        },
        model: ModelSchemaRef {
            format: portable_v2::PORTABLE_FORMAT.into(),
            format_version: u64::from(portable_v2::PORTABLE_VERSION),
            schema_version: u64::from(super::schema_v2::SCHEMA_VERSION),
        },
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn codec_error(code: &str) -> super::schema::ModelError {
    // Never forward serde's raw error text, a remote value, or an authorization.
    invalid(code, "配置对象未通过格式、范围或完整性核对")
}
fn model_error(e: super::schema_v2::MatchErrorV2) -> super::schema::ModelError {
    codec_error(&e.code)
}
fn valid_digest(s: &str) -> bool {
    super::schema_v2::valid_digest(s)
}

/// Values are immediately owned by Zeroizing. No serde/Debug implementation is
/// exposed, so this cannot accidentally become an IPC return object.
#[derive(Default)]
pub struct SecretValues {
    values: BTreeMap<String, Zeroizing<String>>,
}
impl SecretValues {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, id: String, value: Zeroizing<String>) -> ModelResult<()> {
        if self.values.contains_key(&id) {
            return Err(codec_error("credential_scope"));
        }
        self.values.insert(id, value);
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&str> {
        self.values.get(id).map(|v| v.as_str())
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
fn serialize_secrets<S: Serializer>(v: &SecretValues, s: S) -> Result<S::Ok, S::Error> {
    s.collect_map(v.values.iter().map(|(key, value)| (key, value.as_str())))
}
fn deserialize_secrets<'de, D: Deserializer<'de>>(d: D) -> Result<SecretValues, D::Error> {
    struct SecretsVisitor;
    impl<'de> Visitor<'de> for SecretsVisitor {
        type Value = SecretValues;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("unique scoped secret values")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = SecretValues::new();
            while let Some(id) = a.next_key::<String>()? {
                let value = Zeroizing::new(a.next_value::<String>()?);
                out.insert(id, value)
                    .map_err(|_| de::Error::custom("duplicate secret identity"))?;
            }
            Ok(out)
        }
    }
    d.deserialize_map(SecretsVisitor)
}

/// L0 supplies the public service/credential direction from its controlled source
/// capture, never from an arbitrary UI value. Nonempty auth without this callback
/// is rejected. Implementations call provider_auth::validate_portable and enforce
/// the exact source binding; this codec never guesses a channel adapter.
pub trait AuthorizationValidator {
    fn validate(
        &self,
        channel: &PortableChannelV2,
        authorization: &PortableAuthorization,
    ) -> ModelResult<()>;
}

/// Already captured by L0. Intentionally no Clone, Debug, Serialize, or disk reads.
pub struct CapturedObjects {
    pub models: PortableModelsV2,
    pub manager: portable_manager::Settings,
    pub assets: portable_assets::Assets,
    pub channel_keys: SecretValues,
    pub channel_authorizations: BTreeMap<String, PortableAuthorization>,
    pub webdav_passwords: SecretValues,
    pub content_versions: Vec<Version>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireObjects {
    format: String,
    version: u32,
    models: PortableModelsV2,
    manager: portable_manager::Settings,
    assets: portable_assets::Assets,
    #[serde(
        serialize_with = "serialize_secrets",
        deserialize_with = "deserialize_secrets"
    )]
    channel_keys: SecretValues,
    channel_authorizations: BTreeMap<String, PortableAuthorization>,
    #[serde(
        serialize_with = "serialize_secrets",
        deserialize_with = "deserialize_secrets"
    )]
    webdav_passwords: SecretValues,
    content_versions: Vec<Version>,
}
impl Drop for WireObjects {
    fn drop(&mut self) {
        // Assets already zeroize skill bodies and credential maps. A rejected MCP
        // might still have unexpected secret strings in its supposed public part.
        for m in &mut self.assets.mcps {
            erase_value(&mut m.definition);
        }
        // A rejected caller capture may have smuggled a credential into an
        // unknown model source field. Clear those dynamic values on rejection
        // too, even though the actual L1 validator never accepts that field.
        if let Some(policy) = &mut self.models.payload.policy {
            for channel in &mut policy.channels {
                for model in &mut channel.models {
                    for field in model.source.fields.values_mut() {
                        erase_value(&mut field.value);
                    }
                }
            }
        }
    }
}
pub struct SavedObjects {
    wire: WireObjects,
}
impl SavedObjects {
    pub fn capture(
        v: CapturedObjects,
        auth: Option<&dyn AuthorizationValidator>,
    ) -> ModelResult<Self> {
        let out = Self {
            wire: WireObjects {
                format: DOCUMENT_FORMAT.into(),
                version: DOCUMENT_VERSION,
                models: v.models,
                manager: v.manager,
                assets: v.assets,
                channel_keys: v.channel_keys,
                channel_authorizations: v.channel_authorizations,
                webdav_passwords: v.webdav_passwords,
                content_versions: v.content_versions,
            },
        };
        out.validate(auth)?;
        Ok(out)
    }
    pub fn models(&self) -> &PortableModelsV2 {
        &self.wire.models
    }
    /// Trusted host application partitions, NOT IPC DTOs. The aggregate itself
    /// has no serde/Debug implementation; credentials remain borrowed only.
    pub fn partitions(&self) -> ObjectPartitions<'_> {
        ObjectPartitions {
            models: &self.wire.models,
            manager: &self.wire.manager,
            assets: &self.wire.assets,
            channel_keys: &self.wire.channel_keys,
            channel_authorizations: &self.wire.channel_authorizations,
            webdav_passwords: &self.wire.webdav_passwords,
            content_versions: &self.wire.content_versions,
        }
    }
    /// Transfer ownership to the trusted host without cloning secret maps. This
    /// is an application boundary, not an IPC response or an automatic merge.
    pub fn into_captured(self) -> CapturedObjects {
        let mut w = self.wire;
        let empty = PortableModelsV2 {
            format: String::new(),
            format_version: 0,
            schema_version: 0,
            created_at: String::new(),
            digest: String::new(),
            payload: portable_v2::ModelPayloadV2 {
                templates: vec![],
                policy: None,
            },
        };
        CapturedObjects {
            models: std::mem::replace(&mut w.models, empty),
            manager: std::mem::take(&mut w.manager),
            assets: std::mem::take(&mut w.assets),
            channel_keys: std::mem::take(&mut w.channel_keys),
            channel_authorizations: std::mem::take(&mut w.channel_authorizations),
            webdav_passwords: std::mem::take(&mut w.webdav_passwords),
            content_versions: std::mem::take(&mut w.content_versions),
        }
    }
    /// Generic files may carry endpoint candidates; WebDAV self-sync may not
    /// transport its own connection/password. L0 calls this at the remote codec
    /// boundary; file export/import remains a separate, explicit use case.
    pub fn validate_remote_sync_scope(&self) -> ModelResult<()> {
        if !self.wire.manager.webdav.is_empty() || !self.wire.webdav_passwords.is_empty() {
            return Err(codec_error("sync_self_scope"));
        }
        Ok(())
    }
    fn validate(&self, auth: Option<&dyn AuthorizationValidator>) -> ModelResult<()> {
        let w = &self.wire;
        if w.format != DOCUMENT_FORMAT || w.version != DOCUMENT_VERSION {
            return Err(codec_error("portable_objects_v2_version"));
        }
        // Bound serialization before L1 validation can allocate a canonical copy
        // of an arbitrarily large caller-constructed in-memory payload.
        let _ = document_bytes(w)?;
        // Exact dependencies, source evidence, and explicit intent all belong to L1.
        portable_v2::validate_portable_models_v2(&w.models).map_err(model_error)?;
        portable_manager::validate(&w.manager).map_err(|e| codec_error(&e.code))?;
        portable_assets::validate(&w.assets).map_err(|e| codec_error(&e.code))?;
        for (id, key) in &w.channel_keys.values {
            let channel = channel(&w.models, id)?;
            if channel.connection.is_none()
                || key.is_empty()
                || key.len() > SECRET_LIMIT
                || key.contains(['\r', '\n'])
            {
                return Err(codec_error("credential_scope"));
            }
        }
        for (id, authorization) in &w.channel_authorizations {
            let channel = channel(&w.models, id)?;
            if channel.connection.is_none() || w.channel_keys.values.contains_key(id) {
                return Err(codec_error("credential_scope"));
            }
            auth.ok_or_else(|| codec_error("portable_authorization_context_required"))?
                .validate(channel, authorization)
                .map_err(|_| codec_error("portable_authorization_binding"))?;
        }
        for (id, password) in &w.webdav_passwords.values {
            if password.is_empty()
                || password.len() > SECRET_LIMIT
                || !w.manager.webdav.iter().any(|e| e.id == *id)
            {
                return Err(codec_error("credential_scope"));
            }
        }
        let mut ids = BTreeSet::new();
        if w.content_versions.len() > MAX_VERSIONS {
            return Err(codec_error("content_version"));
        }
        for v in &w.content_versions {
            if v.scope_key.is_empty()
                || v.scope_key.len() > 8192
                || !ids.insert(&v.scope_key)
                || !valid_digest(&v.content_digest)
            {
                return Err(codec_error("content_version"));
            }
            if (!v.saved_at.is_empty()
                && chrono::DateTime::parse_from_rfc3339(&v.saved_at).is_err())
                || (v.time_kind == TimeKind::Saved && v.saved_at.is_empty())
                || v.trusted_floor
                    .as_deref()
                    .is_some_and(|t| chrono::DateTime::parse_from_rfc3339(t).is_err())
                || v.last_event
                    .as_deref()
                    .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
            {
                return Err(codec_error("content_version"));
            }
        }
        Ok(())
    }
    pub fn summary(&self) -> PublicSummary {
        let w = &self.wire;
        PublicSummary {
            schema: supported_schema(),
            template_ids: w
                .models
                .payload
                .templates
                .iter()
                .map(|t| t.id.clone())
                .collect(),
            channel_ids: w
                .models
                .payload
                .policy
                .iter()
                .flat_map(|p| p.channels.iter().map(|c| c.id.clone()))
                .collect(),
            account_count: w.manager.accounts.len(),
            skill_count: w.assets.skills.len(),
            mcp_count: w.assets.mcps.len(),
            has_preferences: w.manager.preferences.is_some(),
            has_extensions: w.manager.extensions.is_some(),
            webdav_count: w.manager.webdav.len(),
            channel_key_count: w.channel_keys.len(),
            channel_authorization_count: w.channel_authorizations.len(),
            webdav_password_count: w.webdav_passwords.len(),
            extension_secret_count: portable_assets::secret_count(&w.assets),
            content_version_count: w.content_versions.len(),
        }
    }
}
fn channel<'a>(models: &'a PortableModelsV2, id: &str) -> ModelResult<&'a PortableChannelV2> {
    models
        .payload
        .policy
        .as_ref()
        .and_then(|p| p.channels.iter().find(|c| c.id == id))
        .ok_or_else(|| codec_error("credential_scope"))
}
pub struct ObjectPartitions<'a> {
    pub models: &'a PortableModelsV2,
    pub manager: &'a portable_manager::Settings,
    pub assets: &'a portable_assets::Assets,
    pub channel_keys: &'a SecretValues,
    pub channel_authorizations: &'a BTreeMap<String, PortableAuthorization>,
    pub webdav_passwords: &'a SecretValues,
    pub content_versions: &'a [Version],
}
/// Deliberately counts and validated UUIDs only: no names, URLs, account metadata,
/// source evidence, warnings, MCP definitions, auth fields, or credential digests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSummary {
    pub schema: PayloadSchemaRef,
    pub template_ids: Vec<String>,
    pub channel_ids: Vec<String>,
    pub account_count: usize,
    pub skill_count: usize,
    pub mcp_count: usize,
    pub has_preferences: bool,
    pub has_extensions: bool,
    pub webdav_count: usize,
    pub channel_key_count: usize,
    pub channel_authorization_count: usize,
    pub webdav_password_count: usize,
    pub extension_secret_count: usize,
    pub content_version_count: usize,
}
pub struct EncodedFile {
    bytes: Zeroizing<Vec<u8>>,
    pub file_digest: String,
    pub semantic_digest: String,
    pub schema: PayloadSchemaRef,
    pub protected: bool,
}
impl EncodedFile {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Zeroizing<Vec<u8>> {
        self.bytes
    }
}
pub struct DecodedFile {
    pub objects: SavedObjects,
    pub protected: bool,
    pub file_digest: String,
}

/// Encode only captured objects, using the existing, real protection envelope.
/// Call once per operation and persist these exact bytes; re-encrypting a retry
/// would change the ciphertext identity even when semantic content is identical.
pub fn encode(
    objects: &SavedObjects,
    id: &str,
    created_at: &str,
    passphrase: Option<&str>,
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<EncodedFile> {
    objects.validate(auth)?;
    let document = document_bytes(&objects.wire)?;
    let manifest = portable_file::Manifest {
        id: id.into(),
        created_at: created_at.into(),
        sections: sections(&objects.wire),
        secret_objects: secret_count(&objects.wire),
        document_digest: digest(&document),
    };
    let bytes = Zeroizing::new(portable_file::seal(manifest, &document, passphrase)?);
    if bytes.len() > FILE_LIMIT {
        return Err(codec_error("portable_limit"));
    }
    let file_digest = digest(&bytes);
    let semantic_digest = semantic_digest(objects)?;
    Ok(EncodedFile {
        bytes,
        file_digest,
        semantic_digest,
        schema: supported_schema(),
        protected: passphrase.is_some_and(|s| !s.is_empty()),
    })
}
pub fn decode(
    bytes: &[u8],
    passphrase: Option<&str>,
    expected: &PayloadSchemaRef,
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<DecodedFile> {
    if *expected != supported_schema() {
        return Err(codec_error("portable_objects_v2_schema"));
    }
    if bytes.len() > FILE_LIMIT {
        return Err(codec_error("portable_limit"));
    }
    scan_json(bytes, ScanScope::Other)?;
    let opened = portable_file::open(bytes, passphrase).map_err(|e| codec_error(&e.code))?;
    let objects = decode_document(&opened.document, auth)?;
    if opened.manifest.sections != sections(&objects.wire)
        || opened.manifest.secret_objects != secret_count(&objects.wire)
    {
        return Err(codec_error("portable_manifest"));
    }
    Ok(DecodedFile {
        objects,
        protected: opened.protected,
        file_digest: digest(bytes),
    })
}
pub fn decode_document(
    bytes: &[u8],
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<SavedObjects> {
    if bytes.len() > DOCUMENT_LIMIT {
        return Err(codec_error("portable_limit"));
    }
    scan_json(bytes, ScanScope::Document)?;
    let wire: WireObjects =
        serde_json::from_slice(bytes).map_err(|_| codec_error("portable_objects_v2_format"))?;
    let objects = SavedObjects { wire };
    objects.validate(auth)?;
    Ok(objects)
}
pub fn encode_document(
    objects: &SavedObjects,
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<Zeroizing<Vec<u8>>> {
    objects.validate(auth)?;
    document_bytes(&objects.wire)
}

/// Device-local semantic identity, never a credential display value. It excludes
/// only transport/model package creation time/digest and save-event receipts.
/// Model source/explicit/bindings, immutable revision contents and all secret
/// values remain bound; encryption randomness can never fabricate a change.
pub fn semantic_digest(objects: &SavedObjects) -> ModelResult<String> {
    let mut value = SecretJson(
        serde_json::to_value(&objects.wire)
            .map_err(|_| codec_error("portable_objects_v2_format"))?,
    );
    let root = value
        .0
        .as_object_mut()
        .ok_or_else(|| codec_error("portable_objects_v2_format"))?;
    if let Some(mut removed) = root.remove("contentVersions") {
        erase_value(&mut removed);
    }
    if let Some(model) = root.get_mut("models").and_then(Value::as_object_mut) {
        for key in ["createdAt", "digest"] {
            if let Some(mut removed) = model.remove(key) {
                erase_value(&mut removed);
            }
        }
    }
    // Inventory ordering is not identity; explicit channel/model order fields
    // remain untouched and still affect semantic comparison.
    for (pointer, key) in [
        ("/models/payload/templates", "id"),
        ("/models/payload/policy/channels", "id"),
        ("/manager/accounts", "id"),
        ("/manager/webdav", "id"),
        ("/assets/skills", "name"),
        ("/assets/mcps", "name"),
    ] {
        if let Some(array) = value.0.pointer_mut(pointer).and_then(Value::as_array_mut) {
            array.sort_by(|a, b| a[key].as_str().cmp(&b[key].as_str()));
        }
    }
    let bytes = secret_canonical(&value.0)?;
    Ok(digest(&bytes))
}
fn sections(w: &WireObjects) -> Vec<String> {
    let mut out = vec!["models".into()];
    if !w.channel_keys.is_empty() {
        out.push("channelKeys".into());
    }
    if !w.channel_authorizations.is_empty() {
        out.push("channelAuthorizations".into());
    }
    out.extend(portable_manager::sections(&w.manager));
    if !w.webdav_passwords.is_empty() {
        out.push("webdavPasswords".into());
    }
    if !w.assets.skills.is_empty() {
        out.push("skills".into());
    }
    if !w.assets.mcps.is_empty() {
        out.push("mcp".into());
    }
    if portable_assets::secret_count(&w.assets) > 0 {
        out.push("extensionCredentials".into());
    }
    out
}
fn secret_count(w: &WireObjects) -> usize {
    w.channel_keys.len()
        + w.webdav_passwords.len()
        // A portable authorization can contain proxy credentials or executor
        // secrets even when the legacy has_secret_material() only sees pools/
        // headers. Treat every opaque auth object as one protected secret scope.
        + w.channel_authorizations.len()
        + portable_assets::secret_count(&w.assets)
}
struct BoundedBuffer {
    bytes: Zeroizing<Vec<u8>>,
    limit: usize,
}
impl Write for BoundedBuffer {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("portable byte limit"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn document_bytes(w: &WireObjects) -> ModelResult<Zeroizing<Vec<u8>>> {
    let mut out = BoundedBuffer {
        bytes: Zeroizing::new(Vec::new()),
        limit: DOCUMENT_LIMIT,
    };
    serde_json::to_writer(&mut out, w).map_err(|_| codec_error("portable_limit"))?;
    Ok(out.bytes)
}
struct SecretJson(Value);
impl Drop for SecretJson {
    fn drop(&mut self) {
        erase_value(&mut self.0);
    }
}
fn erase_value(value: &mut Value) {
    match value {
        Value::String(s) => s.zeroize(),
        Value::Array(a) => a.iter_mut().for_each(erase_value),
        Value::Object(m) => {
            for (mut k, mut v) in std::mem::take(m) {
                k.zeroize();
                erase_value(&mut v);
            }
        }
        _ => (),
    }
}
fn secret_canonical(value: &Value) -> ModelResult<Zeroizing<Vec<u8>>> {
    fn write(value: &Value, out: &mut BoundedBuffer) -> std::io::Result<()> {
        match value {
            Value::Object(map) => {
                out.write_all(b"{")?;
                let keys: BTreeSet<_> = map.keys().collect();
                for (n, k) in keys.into_iter().enumerate() {
                    if n > 0 {
                        out.write_all(b",")?;
                    }
                    serde_json::to_writer(&mut *out, k)?;
                    out.write_all(b":")?;
                    write(&map[k], out)?;
                }
                out.write_all(b"}")
            }
            Value::Array(a) => {
                out.write_all(b"[")?;
                for (n, v) in a.iter().enumerate() {
                    if n > 0 {
                        out.write_all(b",")?;
                    }
                    write(v, out)?;
                }
                out.write_all(b"]")
            }
            _ => serde_json::to_writer(out, value).map_err(std::io::Error::other),
        }
    }
    let mut out = BoundedBuffer {
        bytes: Zeroizing::new(Vec::new()),
        limit: DOCUMENT_LIMIT,
    };
    write(value, &mut out).map_err(|_| codec_error("portable_limit"))?;
    Ok(out.bytes)
}
#[derive(Clone, Copy)]
enum ScanScope {
    Other,
    Document,
    Manager,
    Preferences,
    Extensions,
}
struct ScanSeed(ScanScope);
impl<'de> DeserializeSeed<'de> for ScanSeed {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for ScanSeed {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON with unique, supported fields")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_none<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_string<E: de::Error>(self, mut value: String) -> Result<(), E> {
        value.zeroize();
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        while a.next_element_seed(ScanSeed(ScanScope::Other))?.is_some() {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        let mut seen = BTreeSet::new();
        while let Some(key) = a.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(de::Error::custom("duplicate field"));
            }
            let allowed = match self.0 {
                ScanScope::Preferences => {
                    ["theme", "interface_scale", "visual_effects", "glow_render"]
                        .contains(&key.as_str())
                }
                ScanScope::Extensions => [
                    "skills",
                    "servers",
                    "enablement",
                    "skill_targets",
                    "source_store",
                    "sync_method",
                    "revision",
                ]
                .contains(&key.as_str()),
                _ => true,
            };
            if !allowed {
                return Err(de::Error::custom("unknown portable field"));
            }
            let next = match (self.0, key.as_str()) {
                (ScanScope::Document, "manager") => ScanScope::Manager,
                (ScanScope::Manager, "preferences") => ScanScope::Preferences,
                (ScanScope::Manager, "extensions") => ScanScope::Extensions,
                _ => ScanScope::Other,
            };
            a.next_value_seed(ScanSeed(next))?;
        }
        Ok(())
    }
}
fn scan_json(bytes: &[u8], scope: ScanScope) -> ModelResult<()> {
    let mut d = serde_json::Deserializer::from_slice(bytes);
    ScanSeed(scope)
        .deserialize(&mut d)
        .and_then(|()| d.end())
        .map_err(|_| codec_error("portable_objects_v2_format"))
}
#[cfg(test)]
#[path = "file_objects_v2_tests.rs"]
mod tests;
