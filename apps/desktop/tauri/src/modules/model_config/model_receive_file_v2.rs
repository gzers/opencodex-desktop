//! Host-only v6 receive consumer. Mounted as config_file::v2::receive.
//! Outer LocalApplication owns global admission and selected-capture CAS.
//! Raw images are private, bounded, zeroized, and never exposed by the trait.
use super::*;
use crate::modules::{
    backup::{self, BackupAction},
    model_config::{
        content_versions,
        file_objects_v2::{self, AuthorizationValidator, SavedObjects},
        model_transaction_v2::{self, TargetV2, WriteV2},
        objects_overlay_v2::SelectionV2,
        portable_v2::PortableChannelV2,
        provider_auth::{self, PortableAuthorization},
        receive_models_v2::{self, ReceiveBindingModeV2, ReceiveBindingV2},
        schema::{invalid, ModelError, ModelResult},
        sync_local_v2::{ModelApplyPlanHandle, ModelApplyReceipt, ModelApplyStatus, ModelApplyV2},
    },
};
#[cfg(windows)]
use std::fs::OpenOptions;
use std::{io::Write, path::PathBuf};
use zeroize::Zeroize;

const MAX_IMAGE: usize = 32 * 1024 * 1024;
const MAX_TOTAL: usize = 128 * 1024 * 1024;
const MAX_MANIFEST: u64 = 128 * 1024;
const RECEIVES: &str = "manager-state/model-receive-v2";
const SYNC_IDENTITIES: &str = "manager-state/sync-identity-mappings-v2.json";
const IDENTITY: &str = "manager-state/model-v2-provider-identities.json";
const METADATA: [&str; 5] = [
    store_v2::LIBRARY_PATH,
    store_v2::POLICY_PATH,
    store_v2::LEDGER_PATH,
    store_v2::MAPPING_PATH,
    content_versions::SAVED_VERSIONS_PATH,
];

/// Construction is trusted host authority, never an IPC-deserialized request.
/// The preview has already assigned every target identity.
pub struct ReceiveFileHostV2 {
    pub root: PathBuf,
    pub home: PathBuf,
    pub bindings: Vec<ReceiveBindingV2>,
    pub trusted_mapping_write: Option<IdentityMappingWriteV2>,
}
/// Host-owned exact images for ONE fixed metadata path, never arbitrary writes.
/// No Debug/Serialize; these bytes never cross IPC.
pub struct IdentityMappingWriteV2 {
    pub before: Option<Vec<u8>>,
    pub after: Vec<u8>,
}
impl Drop for IdentityMappingWriteV2 {
    fn drop(&mut self) {
        if let Some(b) = &mut self.before {
            b.zeroize();
        }
        self.after.zeroize();
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IdentityMappings {
    schema_version: u32,
    bindings: Vec<IdentityMapping>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IdentityMapping {
    endpoint_binding_id: String,
    portable_id: String,
    local_id: String,
    adopted_operation_id: String,
}
fn mapping_document(bytes: &[u8]) -> Result<IdentityMappings> {
    if bytes.len() > MAX_IMAGE {
        return Err(fail("model_receive_identity_limit"));
    }
    let doc: IdentityMappings =
        serde_json::from_value(parse_strict_json_v2(bytes)?).map_err(json_error)?;
    if doc.schema_version != 2 {
        return Err(fail("model_schema_unsupported"));
    }
    if doc.bindings.len() > 20000 {
        return Err(fail("model_receive_identity_limit"));
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &doc.bindings {
        check_id(&row.portable_id)?;
        check_id(&row.local_id)?;
        operation_id(&row.adopted_operation_id)?;
        if !valid_digest(&row.endpoint_binding_id)
            || !sources.insert((&row.endpoint_binding_id, &row.portable_id))
            || !targets.insert((&row.endpoint_binding_id, &row.local_id))
        {
            return Err(fail("model_receive_identity_invalid"));
        }
    }
    Ok(doc)
}
fn validate_mapping_write(
    id: &str,
    bindings: &[ReceiveBindingV2],
    write: &IdentityMappingWriteV2,
) -> Result<()> {
    let before = write
        .before
        .as_deref()
        .map(mapping_document)
        .transpose()?
        .unwrap_or(IdentityMappings {
            schema_version: 2,
            bindings: vec![],
        });
    let after = mapping_document(&write.after)?;
    for old in &before.bindings {
        if !after.bindings.contains(old) {
            return Err(fail("model_receive_identity_overwrite"));
        }
    }
    for row in after
        .bindings
        .iter()
        .filter(|row| !before.bindings.contains(row))
    {
        if row.adopted_operation_id != id
            || !bindings.iter().any(|b| {
                b.portable_channel_id == row.portable_id && b.local_channel_id == row.local_id
            })
        {
            return Err(fail("model_receive_identity_invalid"));
        }
    }
    Ok(())
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Image {
    len: usize,
    digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    target: TargetV2,
    before: Option<Image>,
    after: Image,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Frozen {
    schema_version: u32,
    operation_id: String,
    root_revision: String,
    objects_hash: String,
    selection_hash: String,
    bindings_hash: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    namespace_adoption: Vec<String>,
    expected_target: String,
    source_hash: String,
    identity: Option<Image>,
    entries: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    operation_id: String,
    plan_hash: String,
    root_revision: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    namespace_adoption: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BackupProof {
    target_index: usize,
    payload: String,
    image: Image,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Protection {
    schema_version: u32,
    operation_id: String,
    plan_hash: String,
    backup: bool,
    proofs: Vec<BackupProof>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checked<T> {
    value: T,
    checksum: String,
}
struct SecretValue(Value);
impl Drop for SecretValue {
    fn drop(&mut self) {
        erase(&mut self.0);
    }
}
fn erase(v: &mut Value) {
    match v {
        Value::String(s) => s.zeroize(),
        Value::Array(a) => a.iter_mut().for_each(erase),
        Value::Object(o) => {
            for (_, mut v) in std::mem::take(o) {
                erase(&mut v);
            }
        }
        _ => {}
    }
}
fn model_error(e: MatchErrorV2) -> ModelError {
    invalid(&e.code, "模型接收计划未通过核对，原件已保留")
}
fn fail(code: &str) -> MatchErrorV2 {
    error(code)
}
fn checked<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    canonical_bytes(&Checked {
        checksum: digest(&canonical_bytes(value)?),
        value,
    })
}
fn decoded<T: serde::de::DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    let c: Checked<T> = serde_json::from_value(parse_strict_json_v2(bytes)?).map_err(json_error)?;
    if !valid_digest(&c.checksum) || digest(&canonical_bytes(&c.value)?) != c.checksum {
        return Err(fail("model_receive_checksum"));
    }
    Ok(c.value)
}
fn descriptor(bytes: &[u8]) -> Image {
    Image {
        len: bytes.len(),
        digest: digest(bytes),
    }
}
fn optional(bytes: &Option<Zeroizing<Vec<u8>>>) -> Option<Image> {
    bytes.as_ref().map(|b| descriptor(b))
}
fn operation_id(id: &str) -> Result<()> {
    if !uuid::Uuid::parse_str(id).is_ok_and(|u| {
        u.to_string() == id && u.get_version_num() == 4 && u.get_variant() == uuid::Variant::RFC4122
    }) {
        return Err(fail("model_receive_operation_id"));
    }
    Ok(())
}
struct ActualAuthorization;
impl AuthorizationValidator for ActualAuthorization {
    fn validate(&self, c: &PortableChannelV2, a: &PortableAuthorization) -> ModelResult<()> {
        provider_auth::validate_portable(
            c.connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
                .ok_or_else(|| invalid("model_receive_authorization", "缺少授权原件方向"))?,
            a,
        )
    }
}
fn objects_hash(objects: &SavedObjects) -> Result<String> {
    Ok(digest(
        &file_objects_v2::encode_document(objects, Some(&ActualAuthorization)).map_err(io)?,
    ))
}
impl ReceiveFileHostV2 {
    fn binding_hash(&self) -> Result<String> {
        let mut rows: Vec<_> = self
            .bindings
            .iter()
            .map(|b| {
                (
                    &b.portable_channel_id,
                    &b.local_channel_id,
                    &b.provider_id,
                    match b.mode {
                        ReceiveBindingModeV2::New => 0,
                        ReceiveBindingModeV2::Existing => 1,
                    },
                )
            })
            .collect();
        rows.sort();
        let mapping = self.trusted_mapping_write.as_ref().map(|w| {
            (
                w.before.as_ref().map(|b| descriptor(b)),
                descriptor(&w.after),
            )
        });
        Ok(digest(&canonical_bytes(&(rows, mapping))?))
    }
    fn dir(&self, id: &str) -> Result<PathBuf> {
        operation_id(id)?;
        let p = self.root.join(RECEIVES).join(id);
        safety::check_path(&self.root, &p, true).map_err(io)?;
        Ok(p)
    }
    fn read_at(&self, anchor: &Path, p: &Path, max: u64) -> Result<Option<Zeroizing<Vec<u8>>>> {
        safety::check_path(anchor, p, true).map_err(io)?;
        match std::fs::symlink_metadata(p) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io(e)),
            Ok(_) => Ok(Some(Zeroizing::new(safety::read_file(p, max).map_err(io)?))),
        }
    }
    fn private_read(&self, p: &Path, max: u64) -> Result<Option<Zeroizing<Vec<u8>>>> {
        let bytes = self.read_at(&self.root, p, max)?;
        if bytes.is_some() {
            private(p, false)?;
        }
        Ok(bytes)
    }
    fn target_path(&self, t: &TargetV2) -> Result<(PathBuf, PathBuf)> {
        match t {
            TargetV2::OfficialConfig => Ok((self.home.clone(), self.home.join("config.json"))),
            TargetV2::Manager(p) if METADATA.contains(&p.as_str()) || p == SYNC_IDENTITIES => {
                Ok((self.root.clone(), self.root.join(p)))
            }
            _ => Err(fail("model_receive_target")),
        }
    }
    fn target_bytes(&self, t: &TargetV2) -> Result<Option<Zeroizing<Vec<u8>>>> {
        let (a, p) = self.target_path(t)?;
        self.read_at(&a, &p, MAX_IMAGE as u64)
    }
    fn namespace_binding_hash(&self, adopt_ids: &[String]) -> Result<String> {
        let original = self.binding_hash()?;
        if adopt_ids.is_empty() {
            return Ok(original);
        }
        Ok(digest(&canonical_bytes(&(original, adopt_ids))?))
    }
    fn manifest_with_namespace(&self, id: &str, adopt_ids: &[String]) -> Result<Option<Frozen>> {
        let dir = self.dir(id)?;
        let Some(bytes) = self.private_read(&dir.join("plan.json"), MAX_MANIFEST)? else {
            return Ok(None);
        };
        private(&dir, true)?;
        let frozen: Frozen = decoded(&bytes)?;
        if frozen.schema_version != 2 {
            return Err(fail("model_schema_unsupported"));
        }
        if frozen.operation_id != id
            || frozen.root_revision != root_revision(&self.root, &self.home)?
            || frozen.namespace_adoption != adopt_ids
            || frozen.bindings_hash != self.namespace_binding_hash(adopt_ids)?
        {
            return Err(fail("model_receive_plan_binding"));
        }
        if ![6, 7].contains(&frozen.entries.len())
            || !matches!(frozen.entries[0].target, TargetV2::OfficialConfig)
        {
            return Err(fail("model_receive_plan_invalid"));
        }
        let mut total = 0usize;
        for (i, e) in frozen.entries.iter().enumerate() {
            if i > 0
                && e.target
                    != TargetV2::Manager(
                        if i == 6 {
                            SYNC_IDENTITIES
                        } else {
                            METADATA[i - 1]
                        }
                        .into(),
                    )
            {
                return Err(fail("model_receive_plan_invalid"));
            }
            for image in e.before.iter().chain(std::iter::once(&e.after)) {
                if !valid_digest(&image.digest) || image.len > MAX_IMAGE {
                    return Err(fail("model_receive_plan_limit"));
                }
                total = total
                    .checked_add(image.len)
                    .ok_or_else(|| fail("model_receive_plan_limit"))?;
            }
        }
        if total > MAX_TOTAL
            || [
                &frozen.objects_hash,
                &frozen.selection_hash,
                &frozen.bindings_hash,
                &frozen.expected_target,
                &frozen.source_hash,
            ]
            .iter()
            .any(|d| !valid_digest(d))
        {
            return Err(fail("model_receive_plan_invalid"));
        }
        if frozen
            .identity
            .as_ref()
            .is_some_and(|i| i.len > MAX_IMAGE || !valid_digest(&i.digest))
        {
            return Err(fail("model_receive_plan_invalid"));
        }
        if frozen.source_hash
            != frozen.entries[0]
                .before
                .as_ref()
                .map(|i| i.digest.clone())
                .unwrap_or_else(|| digest(b""))
        {
            return Err(fail("model_receive_plan_invalid"));
        }
        Ok(Some(frozen))
    }
    fn plan_hash(f: &Frozen) -> Result<String> {
        Ok(digest(&canonical_bytes(f)?))
    }
    fn load(&self, id: &str, h: &ModelApplyPlanHandle) -> Result<Frozen> {
        self.load_with_namespace(id, h, &[])
    }
    fn load_with_namespace(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
        adopt_ids: &[String],
    ) -> Result<Frozen> {
        let f = self
            .manifest_with_namespace(id, adopt_ids)?
            .ok_or_else(|| fail("model_receive_plan_missing"))?;
        if Self::plan_hash(&f)? != h.token() {
            return Err(fail("model_receive_handle_mismatch"));
        }
        Ok(f)
    }
    fn receipt_bytes(&self, id: &str, hash: &str, adopt_ids: &[String]) -> Result<Vec<u8>> {
        checked(&Receipt {
            schema_version: 2,
            operation_id: id.into(),
            plan_hash: hash.into(),
            root_revision: root_revision(&self.root, &self.home)?,
            namespace_adoption: adopt_ids.to_vec(),
        })
    }
    fn applied(&self, id: &str, h: &ModelApplyPlanHandle) -> Result<bool> {
        self.applied_with_namespace(id, h, &[])
    }
    fn applied_with_namespace(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
        adopt_ids: &[String],
    ) -> Result<bool> {
        let p = self.dir(id)?.join("receipt.json");
        let Some(b) = self.private_read(&p, 16 * 1024)? else {
            return Ok(false);
        };
        let r: Receipt = decoded(&b)?;
        if r.schema_version != 2 {
            return Err(fail("model_schema_unsupported"));
        }
        if r.operation_id != id
            || r.namespace_adoption != adopt_ids
            || r.plan_hash != h.token()
            || r.root_revision != root_revision(&self.root, &self.home)?
        {
            return Err(fail("model_receive_receipt_mismatch"));
        }
        Ok(true)
    }
    fn receipt(id: &str, h: &ModelApplyPlanHandle) -> ModelApplyReceipt {
        ModelApplyReceipt {
            operation_id: id.into(),
            plan: h.clone(),
        }
    }
    fn locks<T>(
        &self,
        recover: bool,
        f: impl FnOnce(&store_v2::LockedStoreV2<'_>) -> Result<T>,
    ) -> Result<T> {
        let _admission = content_versions::file_saves::admit(&self.root, &self.home).map_err(io)?;
        let _rename = super::super::channel_rename::admit(&self.root, &self.home).map_err(io)?;
        let _config = TargetFileLock::lock(&self.home.join("config.json")).map_err(io)?;
        let store = StoreV2::new(&self.root);
        let locked = store.lock()?;
        let _versions = content_versions::lock_versions(&self.root).map_err(io)?;
        if recover {
            // Recovery of raw owned Keychain moves belongs to L0's vault host.
            if self
                .read_at(
                    &self.root,
                    &self
                        .root
                        .join("manager-state/credential-references-v2.pending.json"),
                    MAX_MANIFEST,
                )?
                .is_some()
            {
                return Err(fail("model_receive_reference_recovery_required"));
            }
            if self
                .read_at(
                    &self.root,
                    &self
                        .root
                        .join("manager-state/model-transaction-v2.pending.json"),
                    MAX_MANIFEST,
                )?
                .is_some()
            {
                recover_model_transactions(&self.root, &self.home)?;
            }
        }
        f(&locked)
    }
    fn no_draft(&self) -> Result<()> {
        let p = self.root.join(store_v2::DRAFT_PATH);
        safety::check_path(&self.root, &p, true).map_err(io)?;
        match std::fs::symlink_metadata(p) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(fail("model_receive_unsaved_draft")),
        }
    }
    fn check_base(&self, f: &Frozen) -> Result<()> {
        self.no_draft()?;
        if f.root_revision != root_revision(&self.root, &self.home)? {
            return Err(fail("model_receive_stale"));
        }
        for e in &f.entries {
            if optional(&self.target_bytes(&e.target)?) != e.before {
                return Err(fail("model_receive_stale"));
            }
        }
        if optional(&self.read_at(&self.root, &self.root.join(IDENTITY), MAX_IMAGE as u64)?)
            != f.identity
        {
            return Err(fail("model_receive_stale"));
        }
        Ok(())
    }
    fn image(
        &self,
        id: &str,
        i: usize,
        before: bool,
        expected: &Image,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let p = self.dir(id)?.join(format!(
            "{i}-{}.bin",
            if before { "before" } else { "after" }
        ));
        let b = self
            .private_read(&p, MAX_IMAGE as u64)?
            .ok_or_else(|| fail("model_receive_image_missing"))?;
        if descriptor(&b) != *expected {
            return Err(fail("model_receive_image_corrupt"));
        }
        Ok(b)
    }
    fn writes(&self, id: &str, f: &Frozen, h: &ModelApplyPlanHandle) -> Result<Vec<WriteV2>> {
        let mut out = vec![];
        for (i, e) in f.entries.iter().enumerate() {
            out.push(WriteV2 {
                target: e.target.clone(),
                before: e
                    .before
                    .as_ref()
                    .map(|b| self.image(id, i, true, b).map(|b| b.to_vec()))
                    .transpose()?,
                after: Some(self.image(id, i, false, &e.after)?.to_vec()),
            });
        }
        let receipt = self
            .private_read(&self.dir(id)?.join("receipt-after.bin"), 16 * 1024)?
            .ok_or_else(|| fail("model_receive_image_missing"))?;
        if receipt.as_slice() != self.receipt_bytes(id, h.token(), &f.namespace_adoption)? {
            return Err(fail("model_receive_image_corrupt"));
        }
        out.push(WriteV2 {
            target: TargetV2::ManagerOperationReceipt(id.into()),
            before: None,
            after: Some(receipt.to_vec()),
        });
        Ok(out)
    }
    fn freeze(
        &self,
        id: &str,
        f: &Frozen,
        writes: &[WriteV2],
        h: &ModelApplyPlanHandle,
    ) -> Result<()> {
        let dir = self.dir(id)?;
        if std::fs::symlink_metadata(&dir).is_ok() {
            return Err(fail("model_receive_plan_incomplete"));
        }
        let parent = dir.parent().ok_or_else(|| fail("model_receive_path"))?;
        let existed = parent.exists();
        crate::infrastructure::platform::create_private_dir_all(parent, 0o700).map_err(io)?;
        safety::check_path(&self.root, parent, false).map_err(io)?;
        if existed {
            private(parent, true)?;
        } else {
            restrict(parent, true)?;
        }
        crate::infrastructure::platform::create_private_dir_all(&dir, 0o700).map_err(io)?;
        restrict(&dir, true)?;
        for (i, w) in writes.iter().enumerate() {
            if let Some(b) = &w.before {
                publish(&self.root, &dir.join(format!("{i}-before.bin")), b)?;
            }
            publish(
                &self.root,
                &dir.join(format!("{i}-after.bin")),
                w.after
                    .as_deref()
                    .ok_or_else(|| fail("model_receive_plan_invalid"))?,
            )?;
        }
        publish(
            &self.root,
            &dir.join("receipt-after.bin"),
            &self.receipt_bytes(id, h.token(), &f.namespace_adoption)?,
        )?;
        let bytes = checked(f)?;
        if bytes.len() as u64 > MAX_MANIFEST {
            return Err(fail("model_receive_plan_limit"));
        }
        publish(&self.root, &dir.join("plan.json"), &bytes)?;
        Ok(())
    }
    fn protection(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
        backup: bool,
        f: &Frozen,
    ) -> Result<bool> {
        let Some(bytes) =
            self.private_read(&self.dir(id)?.join("protection.json"), MAX_MANIFEST)?
        else {
            return Ok(false);
        };
        let p: Protection = decoded(&bytes)?;
        if p.schema_version != 2
            || p.operation_id != id
            || p.plan_hash != h.token()
            || p.backup != backup
        {
            return Err(fail("model_receive_protection_mismatch"));
        }
        let expected: Vec<_> = f
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.before.is_some() && backup)
            .map(|(i, _)| i)
            .collect();
        if p.proofs.iter().map(|p| p.target_index).collect::<Vec<_>>() != expected {
            return Err(fail("model_receive_backup_invalid"));
        }
        for proof in &p.proofs {
            if Some(&proof.image) != f.entries[proof.target_index].before.as_ref() {
                return Err(fail("model_receive_backup_invalid"));
            }
            let path = self.root.join(&proof.payload);
            if !path.starts_with(self.root.join("backups")) {
                return Err(fail("model_receive_backup_invalid"));
            }
            let bytes = self
                .read_at(&self.root, &path, MAX_IMAGE as u64)?
                .ok_or_else(|| fail("model_receive_backup_invalid"))?;
            if descriptor(&bytes) != proof.image {
                return Err(fail("model_receive_backup_invalid"));
            }
        }
        Ok(true)
    }
    fn prepare_inner(
        &self,
        id: &str,
        objects: &SavedObjects,
        scope: &SelectionV2,
        expected: &str,
        adopt_ids: &[String],
    ) -> Result<ModelApplyPlanHandle> {
        operation_id(id)?;
        if let Some(w) = &self.trusted_mapping_write {
            validate_mapping_write(id, &self.bindings, w)?;
        }
        if !valid_digest(expected) {
            return Err(fail("model_receive_target_digest"));
        }
        let object_hash = objects_hash(objects)?;
        let selection_hash = digest(&canonical_bytes(scope)?);
        if let Some(f) = self.manifest_with_namespace(id, adopt_ids)? {
            if f.objects_hash != object_hash
                || f.selection_hash != selection_hash
                || f.expected_target != expected
            {
                return Err(fail("model_receive_operation_collision"));
            }
            let h = ModelApplyPlanHandle::new(Self::plan_hash(&f)?).map_err(io)?;
            if self.applied_with_namespace(id, &h, adopt_ids)? {
                return Ok(h);
            }
        }
        self.locks(true, |locked| {
            if let Some(f) = self.manifest_with_namespace(id, adopt_ids)? {
                if f.objects_hash != object_hash
                    || f.selection_hash != selection_hash
                    || f.expected_target != expected
                {
                    return Err(fail("model_receive_operation_collision"));
                }
                return ModelApplyPlanHandle::new(Self::plan_hash(&f)?).map_err(io);
            }
            if self
                .read_at(&self.root, &self.dir(id)?.join("receipt.json"), 16 * 1024)?
                .is_some()
            {
                return Err(fail("model_receive_operation_collision"));
            }
            self.no_draft()?;
            let mut baseline = vec![self.target_bytes(&TargetV2::OfficialConfig)?];
            for path in METADATA {
                baseline.push(self.target_bytes(&TargetV2::Manager(path.into()))?);
            }
            let identity_bytes =
                self.read_at(&self.root, &self.root.join(IDENTITY), MAX_IMAGE as u64)?;
            let identity = optional(&identity_bytes);
            let state = locked.snapshot_saved_unlocked()?;
            for (i, path) in METADATA.iter().take(4).enumerate() {
                if state.digests.files.get(*path)
                    != Some(&digest(
                        baseline[i + 1]
                            .as_deref()
                            .map(|b| b.as_slice())
                            .unwrap_or_default(),
                    ))
                {
                    return Err(fail("model_receive_stale"));
                }
            }
            let (raw, bytes) = read_raw(&self.home)?;
            let raw = SecretValue(raw);
            if optional(&bytes) != optional(&baseline[0]) {
                return Err(fail("model_receive_stale"));
            }
            let mut provider_bindings: Vec<_> = state
                .policy
                .channels
                .iter()
                .map(|c| ProviderBindingV2 {
                    channel_id: c.id.clone(),
                    provider_id: c.connection.provider_id.clone(),
                })
                .collect();
            // Saved-only inventory can assign stable UUIDs before the user has
            // saved a manager Policy. Read its already frozen identity registry;
            // never allocate IDs or persist a new registry during receive.
            if let Some(registry_bytes) = identity_bytes.as_deref() {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase", deny_unknown_fields)]
                struct ProviderIdentities {
                    schema_version: u32,
                    providers: BTreeMap<String, String>,
                }
                let identities: ProviderIdentities =
                    serde_json::from_value(parse_strict_json_v2(registry_bytes)?)
                        .map_err(json_error)?;
                if identities.schema_version != 2 || identities.providers.len() > 10000 {
                    return Err(fail("model_receive_identity_invalid"));
                }
                let mut ids = BTreeSet::new();
                for (provider, channel) in identities.providers {
                    check_id(&channel)?;
                    if provider.is_empty() || provider.len() > 512 || !ids.insert(channel.clone()) {
                        return Err(fail("model_receive_identity_invalid"));
                    }
                    if let Some(existing) =
                        provider_bindings.iter().find(|b| b.provider_id == provider)
                    {
                        if existing.channel_id != channel {
                            return Err(fail("model_receive_identity_invalid"));
                        }
                        continue;
                    }
                    if provider_bindings.iter().any(|b| b.channel_id == channel)
                        || state
                            .mappings
                            .mappings
                            .iter()
                            .any(|m| m.provider_id == provider && !m.anchor)
                    {
                        return Err(fail("model_receive_identity_invalid"));
                    }
                    if raw.0["providers"].get(&provider).is_some() {
                        provider_bindings.push(ProviderBindingV2 {
                            channel_id: channel,
                            provider_id: provider,
                        });
                    }
                }
            }
            if raw.0.get("providers").is_some_and(|p| !p.is_object()) {
                return Err(fail("model_source_invalid"));
            }
            for id in raw
                .0
                .get("providers")
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|p| p.keys())
            {
                if !provider_bindings.iter().any(|b| &b.provider_id == id)
                    && !state.mappings.mappings.iter().any(|m| &m.provider_id == id)
                {
                    return Err(fail("model_receive_unbound_source"));
                }
            }
            let actual = provider_import::read_policy_v2(
                &raw.0,
                &source_digest(&bytes),
                &state.policy,
                state.ledger.as_ref(),
                &state.mappings,
                &provider_bindings,
            )?;
            let mut received = receive_models_v2::plan_with_namespace_adoption(
                &state.library,
                &actual,
                objects.models(),
                scope,
                &self.bindings,
                adopt_ids,
            )?;
            let mut output = SecretValue(raw.0.clone());
            if output.0.get("providers").is_none() {
                output.0["providers"] = json!({});
            }
            materialize_sources(
                &mut output.0,
                &actual,
                &mut received.policy,
                &received.public_sources,
                objects,
                scope,
                &self.bindings,
            )?;
            let plan = plan_materialization_v2(&received.library, &actual, &received.policy)?;
            let mappings =
                allocate_mappings(&output.0, &received.policy, &plan.groups, &state.mappings)?;
            let auth = auth_bindings(&output.0, &received.policy)?;
            let mut compiled = compiler_v2::compile_file_v2(
                &received.library,
                &actual,
                &received.policy,
                &output.0,
                &mappings,
                &auth,
            )?;
            // Never copy a raw external reference into a new protocol provider.
            if !reference_moves(&raw.0, &compiled.next_config)?.is_empty() {
                erase(&mut compiled.next_config);
                return Err(fail("model_receive_reference_moves_required"));
            }
            let mut config = SecretValue(std::mem::take(&mut compiled.next_config));
            project_selection(&mut config.0, &compiled.policy, &compiled.mappings)?;
            let selected_channels: BTreeSet<_> = scope
                .channels
                .iter()
                .filter_map(|s| {
                    self.bindings
                        .iter()
                        .find(|b| b.portable_channel_id == s.portable_channel_id)
                        .map(|b| b.local_channel_id.as_str())
                })
                .collect();
            // The compiler sees the complete mapping for dependency/ledger
            // checks. Unselected provider objects retain their actual bytes'
            // values, including unknown fields and absent optional fields.
            for c in actual
                .channels
                .iter()
                .filter(|c| !selected_channels.contains(c.id.as_str()))
            {
                for provider in std::iter::once(&c.connection.provider_id).chain(
                    state
                        .mappings
                        .mappings
                        .iter()
                        .filter(|m| m.channel_id == c.id)
                        .map(|m| &m.provider_id),
                ) {
                    if let Some(original) = raw.0["providers"].get(provider) {
                        if let Some(mut discarded) = config.0["providers"]
                            .as_object_mut()
                            .unwrap()
                            .insert(provider.clone(), original.clone())
                        {
                            erase(&mut discarded);
                        }
                    }
                }
            }
            compiled.ledger =
                finalize_materialization_ledger_v2(&compiled.policy, &config.0, &compiled.targets)?;
            let before_state = saved_state(&state.library, &actual, &raw.0)?;
            let after_state = saved_state(&received.library, &compiled.policy, &config.0)?;
            let versions = content_versions::plan_save_at(
                &self.root,
                &before_state,
                &after_state,
                &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                id,
            )
            .map_err(io)?;
            if versions.before.as_deref() != baseline[5].as_deref().map(|b| b.as_slice()) {
                return Err(fail("model_receive_stale"));
            }
            let afters = [
                canonical_bytes(&received.library)?,
                canonical_bytes(&compiled.policy)?,
                canonical_bytes(&store_v2::LedgerFileV2 {
                    schema_version: 2,
                    ledger: compiled.ledger,
                })?,
                canonical_bytes(&compiled.mappings)?,
                versions.after,
            ];
            let mut writes = vec![WriteV2 {
                target: TargetV2::OfficialConfig,
                before: baseline[0].as_ref().map(|b| b.to_vec()),
                after: Some(canonical_bytes(&config.0)?),
            }];
            for (i, (path, after)) in METADATA.iter().zip(afters).enumerate() {
                let t = TargetV2::Manager((*path).into());
                writes.push(WriteV2 {
                    before: baseline[i + 1].as_ref().map(|b| b.to_vec()),
                    target: t,
                    after: Some(after),
                });
            }
            if let Some(w) = &self.trusted_mapping_write {
                let target = TargetV2::Manager(SYNC_IDENTITIES.into());
                if self.target_bytes(&target)?.as_deref().map(|b| b.as_slice())
                    != w.before.as_deref()
                {
                    return Err(fail("model_receive_stale"));
                }
                writes.push(WriteV2 {
                    target,
                    before: w.before.clone(),
                    after: Some(w.after.clone()),
                });
            }
            let frozen = Frozen {
                schema_version: 2,
                operation_id: id.into(),
                root_revision: root_revision(&self.root, &self.home)?,
                objects_hash: object_hash,
                selection_hash,
                bindings_hash: self.namespace_binding_hash(adopt_ids)?,
                namespace_adoption: adopt_ids.to_vec(),
                expected_target: expected.into(),
                source_hash: source_digest(&bytes),
                identity,
                entries: writes
                    .iter()
                    .map(|w| Entry {
                        target: w.target.clone(),
                        before: w.before.as_ref().map(|b| descriptor(b)),
                        after: descriptor(w.after.as_ref().unwrap()),
                    })
                    .collect(),
            };
            let h = ModelApplyPlanHandle::new(Self::plan_hash(&frozen)?).map_err(io)?;
            let mut total = 0;
            for w in &writes {
                for b in w.before.iter().chain(&w.after) {
                    total += b.len();
                    if b.len() > MAX_IMAGE || total > MAX_TOTAL {
                        return Err(fail("model_receive_plan_limit"));
                    }
                }
            }
            self.check_base(&frozen)?;
            self.freeze(id, &frozen, &writes, &h)?;
            Ok(h)
        })
    }
}
impl ReceiveFileHostV2 {
    fn protect_authorized(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
        backup: bool,
        adopt_ids: &[String],
    ) -> ModelResult<()> {
        self.load_with_namespace(id, h, adopt_ids)
            .map_err(model_error)?;
        if self
            .applied_with_namespace(id, h, adopt_ids)
            .map_err(model_error)?
        {
            return Ok(());
        }
        self.locks(false, |_| {
            let f = self.load_with_namespace(id, h, adopt_ids)?;
            if self.applied_with_namespace(id, h, adopt_ids)? {
                return Ok(());
            }
            self.check_base(&f)?;
            let writes = self.writes(id, &f, h)?;
            if self.protection(id, h, backup, &f)? {
                return Ok(());
            }
            let mut proofs = vec![];
            if backup {
                for (i, w) in writes.iter().take(f.entries.len()).enumerate() {
                    if let Some(before) = &w.before {
                        let (_, path) = self.target_path(&w.target)?;
                        let record = backup::backup_file(
                            &self.root,
                            BackupAction::SyncOverwrite,
                            &path,
                            before,
                            chrono::Utc::now(),
                            Some("模型接收采用前保护".into()),
                        )
                        .map_err(io)?;
                        if !backup::verify_backup(&record).map_err(io)? {
                            return Err(fail("model_receive_backup_invalid"));
                        }
                        let payload = record.directory.join(
                            path.file_name()
                                .ok_or_else(|| fail("model_receive_backup_invalid"))?,
                        );
                        proofs.push(BackupProof {
                            target_index: i,
                            payload: payload
                                .strip_prefix(&self.root)
                                .map_err(io)?
                                .to_str()
                                .ok_or_else(|| fail("model_receive_path"))?
                                .into(),
                            image: descriptor(before),
                        });
                    }
                }
            }
            self.check_base(&f)?;
            publish(
                &self.root,
                &self.dir(id)?.join("protection.json"),
                &checked(&Protection {
                    schema_version: 2,
                    operation_id: id.into(),
                    plan_hash: h.token().into(),
                    backup,
                    proofs,
                })?,
            )?;
            Ok(())
        })
        .map_err(model_error)
    }
    fn apply_authorized(
        &self,
        id: &str,
        objects: &SavedObjects,
        h: &ModelApplyPlanHandle,
        backup: bool,
        adopt_ids: &[String],
    ) -> ModelResult<ModelApplyReceipt> {
        // Validate replay identity before any admission helper can recover an
        // unrelated pending save. Applied is a historical, read-only fast path.
        let f = self
            .load_with_namespace(id, h, adopt_ids)
            .map_err(model_error)?;
        if f.objects_hash != objects_hash(objects).map_err(model_error)? {
            return Err(model_error(fail("model_receive_operation_collision")));
        }
        if self
            .applied_with_namespace(id, h, adopt_ids)
            .map_err(model_error)?
        {
            return Ok(Self::receipt(id, h));
        }
        self.locks(false, |_| {
            let f = self.load_with_namespace(id, h, adopt_ids)?;
            if f.objects_hash != objects_hash(objects)? {
                return Err(fail("model_receive_operation_collision"));
            }
            if self.applied_with_namespace(id, h, adopt_ids)? {
                return Ok(Self::receipt(id, h));
            }
            if self
                .read_at(
                    &self.root,
                    &self
                        .root
                        .join("manager-state/credential-references-v2.pending.json"),
                    MAX_MANIFEST,
                )?
                .is_some()
            {
                return Err(fail("model_receive_reference_recovery_required"));
            }
            if self
                .read_at(
                    &self.root,
                    &self
                        .root
                        .join("manager-state/model-transaction-v2.pending.json"),
                    MAX_MANIFEST,
                )?
                .is_some()
            {
                recover_model_transactions(&self.root, &self.home)?;
            }
            if self.applied_with_namespace(id, h, adopt_ids)? {
                return Ok(Self::receipt(id, h));
            }
            self.check_base(&f)?;
            if !self.protection(id, h, backup, &f)? {
                return Err(fail("model_receive_protection_required"));
            }
            let writes = self.writes(id, &f, h)?;
            let result = model_transaction_v2::commit(&self.root, &self.home, writes);
            if result.is_err() {
                // Interrupted transactions are resumed only through the real engine.
                if self
                    .read_at(
                        &self.root,
                        &self
                            .root
                            .join("manager-state/model-transaction-v2.pending.json"),
                        MAX_MANIFEST,
                    )?
                    .is_some()
                {
                    recover_model_transactions(&self.root, &self.home)?;
                }
                if !self.applied_with_namespace(id, h, adopt_ids)? {
                    return Err(result.err().unwrap());
                }
            }
            if !self.applied_with_namespace(id, h, adopt_ids)? {
                return Err(fail("model_receive_receipt_missing"));
            }
            for e in &f.entries {
                if optional(&self.target_bytes(&e.target)?)
                    != Some(Image {
                        len: e.after.len,
                        digest: e.after.digest.clone(),
                    })
                {
                    return Err(fail("model_receive_readback"));
                }
            }
            Ok(Self::receipt(id, h))
        })
        .map_err(model_error)
    }
    fn status_authorized(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
        adopt_ids: &[String],
    ) -> ModelResult<ModelApplyStatus> {
        // Immutable plan publication and atomic own receipt suffice. Status
        // never acquires a save admission which could recover business files.
        (|| {
            if !adopt_ids.is_empty() {
                self.load_with_namespace(id, h, adopt_ids)?;
            }
            if self.applied_with_namespace(id, h, adopt_ids)? {
                return Ok(ModelApplyStatus::Applied(Self::receipt(id, h)));
            }
            if self.manifest_with_namespace(id, adopt_ids)?.is_none() {
                return Ok(ModelApplyStatus::Unknown);
            }
            self.load_with_namespace(id, h, adopt_ids)?;
            Ok(ModelApplyStatus::Prepared)
        })()
        .map_err(model_error)
    }
}
impl ModelApplyV2 for ReceiveFileHostV2 {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        scope: &SelectionV2,
        expected: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        self.prepare_inner(id, objects, scope, expected, &[])
            .map_err(model_error)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, backup: bool) -> ModelResult<()> {
        self.protect_authorized(id, h, backup, &[])
    }
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        h: &ModelApplyPlanHandle,
        backup: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        self.apply_authorized(id, objects, h, backup, &[])
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        self.status_authorized(id, h, &[])
    }
}
#[path = "namespace_file_receiver_v2.rs"]
pub mod namespace;
pub use namespace::SystemNamespaceReceiveV2;

fn deep_overlay(target: &mut Value, incoming: &Value) {
    match (target, incoming) {
        (Value::Object(t), Value::Object(s)) => {
            for (k, v) in s {
                if let Some(old) = t.get_mut(k) {
                    deep_overlay(old, v);
                } else {
                    t.insert(k.clone(), v.clone());
                }
            }
        }
        (target, source) => {
            erase(target);
            *target = source.clone();
        }
    }
}
fn overlay_public(
    row: &mut Value,
    public: &Value,
    ids: &BTreeSet<String>,
    connection: bool,
    credential: bool,
) -> Result<()> {
    for (key, value) in public
        .as_object()
        .ok_or_else(|| fail("model_source_invalid"))?
    {
        // Compiler alone materializes final protocol; credential direction is separate.
        if key == "adapter" || key == "authMode" && !credential {
            continue;
        }
        if key != "modelPreset" && key.starts_with("model") && value.is_object()
            || key == "autoReviewModelOverrides"
        {
            let old = row
                .as_object_mut()
                .ok_or_else(|| fail("model_source_invalid"))?
                .entry(key.clone())
                .or_insert_with(|| json!({}));
            let map = old
                .as_object_mut()
                .ok_or_else(|| fail("model_receive_public_shape"))?;
            for (id, value) in value
                .as_object()
                .ok_or_else(|| fail("model_receive_public_shape"))?
            {
                // L1 already constrained autoReview's slash-compatible selectors.
                if ids.contains(id) || key == "autoReviewModelOverrides" {
                    if let Some(old) = map.get_mut(id) {
                        deep_overlay(old, value);
                    } else {
                        map.insert(id.clone(), value.clone());
                    }
                }
            }
        } else if matches!(key.as_str(), "models" | "retainModels" | "selectedModels")
            || key.ends_with("Models") && value.is_array()
        {
            let source = value
                .as_array()
                .ok_or_else(|| fail("model_receive_public_shape"))?;
            let mut list = row
                .get(key)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            list.retain(|id| !id.as_str().is_some_and(|id| ids.contains(id)));
            for id in source {
                if id.as_str().is_some_and(|id| ids.contains(id)) && !list.contains(id) {
                    list.push(id.clone());
                }
            }
            row[key] = Value::Array(list);
        } else if connection {
            if let Some(old) = row.get_mut(key) {
                deep_overlay(old, value);
            } else {
                row[key] = value.clone();
            }
        }
    }
    Ok(())
}
fn materialize_sources(
    raw: &mut Value,
    before: &PolicyV2,
    intended: &mut PolicyV2,
    public: &BTreeMap<String, Value>,
    objects: &SavedObjects,
    scope: &SelectionV2,
    bindings: &[ReceiveBindingV2],
) -> Result<()> {
    let parts = objects.partitions();
    for s in &scope.channels {
        let b = bindings
            .iter()
            .find(|b| b.portable_channel_id == s.portable_channel_id)
            .ok_or_else(|| fail("model_receive_scope_invalid"))?;
        let source = objects
            .models()
            .payload
            .policy
            .as_ref()
            .and_then(|p| p.channels.iter().find(|c| c.id == b.portable_channel_id))
            .ok_or_else(|| fail("model_receive_scope_invalid"))?;
        let channel = intended
            .channels
            .iter_mut()
            .find(|c| c.id == b.local_channel_id)
            .ok_or_else(|| fail("model_receive_scope_invalid"))?;
        if b.mode == ReceiveBindingModeV2::New
            && raw["providers"]
                .as_object()
                .unwrap()
                .keys()
                .any(|id| id.eq_ignore_ascii_case(&b.provider_id))
        {
            return Err(fail("model_receive_identity_conflict"));
        }
        if b.mode == ReceiveBindingModeV2::Existing
            && (before.channels.iter().all(|c| c.id != b.local_channel_id)
                || protected(&b.provider_id, &raw["providers"][&b.provider_id]))
        {
            return Err(fail("model_protected_provider"));
        }
        if b.mode == ReceiveBindingModeV2::New && s.family_ids.is_some() {
            return Err(fail("model_receive_family_target_required"));
        }
        if b.mode == ReceiveBindingModeV2::Existing && s.include_connection {
            let old = &raw["providers"][&b.provider_id];
            let incoming = source
                .connection
                .as_ref()
                .ok_or_else(|| fail("model_receive_connection_required"))?;
            let endpoint = incoming
                .nodes
                .iter()
                .find(|n| n.id == incoming.active_node_id)
                .map(|n| n.url.as_str());
            if endpoint != old.get("baseUrl").and_then(Value::as_str) && s.include_key {
                let other_private = crate::modules::model_config::provider_fields::PRIVATE
                    .iter()
                    .filter(|k| !matches!(**k, "apiKey" | "apiKeyPool"))
                    .any(|k| {
                        old.get(*k)
                            .is_some_and(|v| !v.is_null() && v != &json!({}) && v != &json!([]))
                    });
                let other_pool_key =
                    old.get("apiKeyPool")
                        .and_then(Value::as_array)
                        .is_some_and(|pool| {
                            pool.iter().any(|slot| slot.get("key") != old.get("apiKey"))
                        });
                if other_private || other_pool_key {
                    return Err(fail("model_receive_credentials_direction"));
                }
            }
            if !s.include_key && !s.include_authorization {
                if let Some(public) = incoming.provider.as_ref() {
                    if public
                        .get("apiKeyTransport")
                        .is_some_and(|v| Some(v) != old.get("apiKeyTransport"))
                    {
                        return Err(fail("model_receive_credentials_direction"));
                    }
                }
            }
        }
        let selected: BTreeSet<_> = source
            .models
            .iter()
            .filter(|m| {
                s.model_ids
                    .as_ref()
                    .is_none_or(|ids| ids.contains(&m.raw_model_id))
                    && s.family_ids.as_ref().is_none_or(|ids| {
                        m.binding
                            .as_ref()
                            .is_some_and(|b| ids.contains(&b.reference().template_id))
                    })
            })
            .map(|m| m.raw_model_id.clone())
            .collect();
        if raw["providers"].get(&b.provider_id).is_none() {
            raw["providers"][&b.provider_id] = json!({});
        }
        let row = &mut raw["providers"][&b.provider_id];
        if let Some(p) = public.get(&b.local_channel_id) {
            overlay_public(
                row,
                p,
                &selected,
                s.include_connection,
                s.include_key || s.include_authorization,
            )?;
        }
        if s.include_connection {
            let c = source
                .connection
                .as_ref()
                .ok_or_else(|| fail("model_receive_connection_required"))?;
            // includeConnection is the preview's explicit adoption of connection
            // enabled/private-network flags; no unselected connection is touched.
            channel.connection.enabled = c.enabled;
            channel.connection.allow_private_network = c.allow_private_network;
            if let Some(node) = c.nodes.iter().find(|n| n.id == c.active_node_id) {
                row["baseUrl"] = json!(node.url);
            }
            row["disabled"] = json!(!c.enabled);
            row["allowPrivateNetwork"] = json!(c.allow_private_network);
            if s.include_key
                || s.include_authorization
                || b.mode == ReceiveBindingModeV2::New
                    && matches!(c.auth_mode.as_deref(), Some("none" | "local"))
            {
                channel.connection.auth_mode = c.auth_mode.clone();
                if let Some(mode) = &c.auth_mode {
                    row["authMode"] = json!(mode);
                } else {
                    row.as_object_mut().unwrap().remove("authMode");
                }
            }
        }
        if s.include_key {
            let key = parts
                .channel_keys
                .get(&b.portable_channel_id)
                .ok_or_else(|| fail("model_receive_key_missing"))?;
            if key.starts_with("keychain:") || key.starts_with('$') {
                return Err(fail("model_receive_external_credential"));
            }
            super::super::write_credential_row(
                row.as_object_mut()
                    .ok_or_else(|| fail("model_source_invalid"))?,
                "replace",
                Some(key),
            )
            .map_err(io)?;
        }
        if s.include_authorization {
            let auth = parts
                .channel_authorizations
                .get(&b.portable_channel_id)
                .ok_or_else(|| fail("model_receive_authorization_missing"))?;
            let public = source
                .connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
                .ok_or_else(|| fail("model_receive_authorization_missing"))?;
            let fields =
                SecretValue(provider_auth::portable_file_fields(public, auth).map_err(io)?);
            for key in crate::modules::model_config::provider_fields::PRIVATE {
                if let Some(mut old) = row.as_object_mut().unwrap().remove(*key) {
                    erase(&mut old);
                }
            }
            row.as_object_mut().unwrap().extend(
                fields
                    .0
                    .as_object()
                    .ok_or_else(|| fail("model_receive_authorization_missing"))?
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone())),
            );
        }
        if s.include_key || s.include_authorization {
            for key in ["apiKeySelectionRevision", "_apiKeyAttempt"] {
                row.as_object_mut().unwrap().remove(key);
            }
        }
    }
    Ok(())
}
fn project_selection(raw: &mut Value, policy: &PolicyV2, mappings: &MappingFileV2) -> Result<()> {
    for c in &policy.channels {
        let ids = std::iter::once(&c.connection.provider_id)
            .chain(
                mappings
                    .mappings
                    .iter()
                    .filter(|m| m.channel_id == c.id)
                    .map(|m| &m.provider_id),
            )
            .collect::<BTreeSet<_>>();
        for id in ids {
            if let Some(row) = raw["providers"].get_mut(id) {
                row["liveModels"] = json!(matches!(c.selection, ModelSelection::All));
            }
        }
    }
    Ok(())
}
fn private(path: &Path, directory: bool) -> Result<()> {
    let m = safety::inspect(path).map_err(io)?;
    if m.is_dir() != directory {
        return Err(fail("model_receive_private_storage"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if m.uid() != unsafe { libc::geteuid() }
            || m.permissions().mode() & 0o7777 != if directory { 0o700 } else { 0o600 }
            || !directory && m.nlink() != 1
        {
            return Err(fail("model_receive_private_storage"));
        }
    }
    #[cfg(windows)]
    {
        let handle = windows_storage_handle(path, false)?;
        receive_private_acl::verify(&handle, directory)?;
        if !directory {
            receive_private_acl::single_link(&handle)?;
        }
    }
    Ok(())
}
fn restrict(path: &Path, directory: bool) -> Result<()> {
    #[cfg(windows)]
    receive_private_acl::protect(&windows_storage_handle(path, true)?, directory)?;
    crate::infrastructure::platform::set_mode(path, if directory { 0o700 } else { 0o600 })
        .map_err(io)?;
    private(path, directory)
}
fn publish(root: &Path, path: &Path, bytes: &[u8]) -> Result<()> {
    safety::check_path(root, path, true).map_err(io)?;
    let mut f =
        crate::infrastructure::platform::create_new_private_file(path, 0o600).map_err(io)?;
    private(path, false)?;
    f.write_all(bytes).map_err(io)?;
    f.sync_all().map_err(io)?;
    safety::sync_directories(
        root,
        path.parent().ok_or_else(|| fail("model_receive_path"))?,
    )
    .map_err(io)?;
    Ok(())
}
#[cfg(test)]
#[path = "model_receive_file_v2_tests.rs"]
mod tests;

// OS ACL primitive only. It deliberately owns no config path or save authority.
#[cfg(windows)]
fn windows_storage_handle(path: &Path, write_acl: bool) -> Result<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .access_mode(if write_acl { 0x60000 } else { 0x20000 })
        .share_mode(3)
        .custom_flags(0x02200000)
        .open(path)
        .map_err(io)
}
#[cfg(windows)]
mod receive_private_acl {
    use super::*;
    use std::fs::File;
    use std::{
        ffi::c_void,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            EqualSid, GetSecurityDescriptorControl, GetSecurityDescriptorDacl, GetTokenInformation,
            TokenUser, ACL, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            length: *mut u32,
        ) -> i32;
        fn GetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut *mut c_void,
            group: *mut *mut c_void,
            dacl: *mut *mut ACL,
            sacl: *mut *mut ACL,
            descriptor: *mut *mut c_void,
        ) -> u32;
        fn SetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut c_void,
            group: *mut c_void,
            dacl: *const ACL,
            sacl: *const ACL,
        ) -> u32;
    }
    struct Descriptor(*mut c_void);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            // SAFETY: Windows returned this LocalAlloc-owned descriptor.
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    fn descriptor(directory: bool) -> Result<Descriptor> {
        let text = if directory {
            "D:P(A;OICI;FA;;;OW)"
        } else {
            "D:P(A;;FA;;;OW)"
        };
        let text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let mut sd = ptr::null_mut();
        // SAFETY: terminated input and a live output pointer; revision is SDDL 1.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(fail("model_receive_private_storage"));
        }
        Ok(Descriptor(sd))
    }
    fn dacl(sd: &Descriptor) -> Result<*mut ACL> {
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = ptr::null_mut();
        // SAFETY: descriptor is owned and valid; outputs are initialized by Win32.
        if unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut acl, &mut defaulted) } == 0
            || present == 0
            || acl.is_null()
        {
            return Err(fail("model_receive_private_storage"));
        }
        Ok(acl)
    }
    pub(super) fn protect(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let acl = dacl(&expected)?;
        // SAFETY: live file handle and ACL; protect the DACL against inheritance.
        if unsafe {
            SetSecurityInfo(
                file.as_raw_handle(),
                1,
                0x80000004,
                ptr::null_mut(),
                ptr::null_mut(),
                acl,
                ptr::null(),
            )
        } != 0
        {
            return Err(fail("model_receive_private_storage"));
        }
        verify(file, directory)
    }
    fn current_owner(owner: *mut c_void) -> Result<()> {
        let mut token = ptr::null_mut();
        // SAFETY: pseudo process handle and a live output pointer.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(fail("model_receive_private_storage"));
        }
        // SAFETY: successful OpenProcessToken transfers a new owned handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut size = 0;
        // SAFETY: the first call only queries the required buffer length.
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut size,
            );
        }
        if size < std::mem::size_of::<TOKEN_USER>() as u32 || size > 65536 {
            return Err(fail("model_receive_private_storage"));
        }
        let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: buffer is aligned for TOKEN_USER and has at least size bytes.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        } == 0
        {
            return Err(fail("model_receive_private_storage"));
        }
        // SAFETY: successful TokenUser query initialized this aligned structure.
        let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
        // SAFETY: both SIDs originate from successful Windows security queries.
        if owner.is_null() || unsafe { EqualSid(owner, sid) } == 0 {
            return Err(fail("model_receive_private_storage"));
        }
        Ok(())
    }
    pub(super) fn verify(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let expected_acl = dacl(&expected)?;
        let mut owner = ptr::null_mut();
        let mut acl = ptr::null_mut();
        let mut sd = ptr::null_mut();
        // SAFETY: live handle and output pointers; request owner and DACL only.
        if unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                1,
                5,
                &mut owner,
                ptr::null_mut(),
                &mut acl,
                ptr::null_mut(),
                &mut sd,
            )
        } != 0
        {
            return Err(fail("model_receive_private_storage"));
        }
        let sd = Descriptor(sd);
        current_owner(owner)?;
        let mut control = 0;
        let mut revision = 0;
        // SAFETY: live security descriptor and valid output pointers.
        if unsafe { GetSecurityDescriptorControl(sd.0, &mut control, &mut revision) } == 0
            || control & 0x1000 == 0
            || acl.is_null()
        {
            return Err(fail("model_receive_private_storage"));
        }
        // SAFETY: successful security queries guarantee valid ACL headers and
        // AclSize-byte buffers, both kept alive by their descriptor guards.
        let equal = unsafe {
            let a = std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize);
            let b = std::slice::from_raw_parts(
                expected_acl.cast::<u8>(),
                (*expected_acl).AclSize as usize,
            );
            a == b
        };
        if !equal {
            return Err(fail("model_receive_private_storage"));
        }
        Ok(())
    }
    pub(super) fn single_link(file: &File) -> Result<()> {
        let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: live file handle and writable output storage.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err(fail("model_receive_private_storage"));
        }
        // SAFETY: successful query initialized the output.
        if unsafe { info.assume_init() }.nNumberOfLinks != 1 {
            return Err(fail("model_receive_private_storage"));
        }
        Ok(())
    }
}

#[path = "model_receive_mapping_proof_v2.rs"]
mod mapping_proof;
pub use mapping_proof::AppliedMappingReceiptProofV2;
