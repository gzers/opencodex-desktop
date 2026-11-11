//! Persistence adapter for an already-resolved D root. No discovery, migration,
//! config.json writes, partial policy commit, network or implicit template adoption.
use super::{
    protocol_groups::{validate_mappings_v2, MappingFileV2},
    schema_v2::*,
    source_v2::*,
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        locking::{lock_path_for, TargetFileLock},
    },
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub const LIBRARY_PATH: &str = "manager-state/model-family-templates.json";
pub const POLICY_PATH: &str = "manager-state/model-policy.json";
pub const DRAFT_PATH: &str = "manager-state/model-config-draft.json";
pub const LEDGER_PATH: &str = "manager-state/model-materialization-ledger.json";
pub const MAPPING_PATH: &str = "manager-state/model-protocol-mappings.json";
const LOCK_PATH: &str = "manager-state/model-v2";
const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DraftV2 {
    pub schema_version: u32,
    pub base: ReadBaseV2,
    pub policy: PolicyV2,
    /// Read-only actual observation used for a three-way source/draft merge.
    /// None exists only for this unreleased v2 fixture shape, never v1 migration.
    #[serde(default)]
    pub observed_baseline: Option<PolicyV2>,
    /// Server-side digest of the exact saved JSON except known runtime bookkeeping.
    /// Absent evidence keeps the raw-byte conflict check mandatory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_config_digest: Option<String>,
    /// Private provider fingerprints at the observed source; never serialized to IPC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_providers: Option<BTreeMap<String, ProviderObservationV2>>,
    /// Server-owned intent only. References carry no key material; clear does
    /// not change authMode and cannot be inferred from a missing/null key.
    #[serde(default)]
    pub credential_edits: BTreeMap<String, DraftCredentialEditV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderObservationV2 {
    pub row_digest: String,
    pub name_digest: String,
    pub non_connection_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DraftCredentialEditV2 {
    Replace { credential_ref: String },
    Clear,
}
impl<'de> Deserialize<'de> for DraftCredentialEditV2 {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> std::result::Result<Self, D::Error> {
        // serde's internally tagged unit variant ignores extra fields even
        // under deny_unknown_fields. The empty struct variant enforces the
        // actual wire contract while preserving the public unit Clear API.
        #[derive(Deserialize)]
        #[serde(
            tag = "kind",
            rename_all = "snake_case",
            rename_all_fields = "camelCase",
            deny_unknown_fields
        )]
        enum Strict {
            Replace { credential_ref: String },
            Clear {},
        }
        Ok(match Strict::deserialize(decoder)? {
            Strict::Replace { credential_ref } => Self::Replace { credential_ref },
            Strict::Clear {} => Self::Clear,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LedgerFileV2 {
    pub schema_version: u32,
    pub ledger: MaterializationLedgerV2,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoreDigestsV2 {
    /// Semantic canonical Library digest, used by save_revision_plan_v2.
    pub library: String,
    /// Raw byte digests; absent files hash empty bytes. Never a v1 fallback.
    pub files: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreSnapshotV2 {
    pub library: LibraryV2,
    pub policy: PolicyV2,
    pub draft: Option<DraftV2>,
    pub ledger: Option<MaterializationLedgerV2>,
    pub mappings: MappingFileV2,
    pub digests: StoreDigestsV2,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWriteV2 {
    pub relative_path: String,
    pub before: Option<Vec<u8>>,
    pub after: Vec<u8>,
}
pub struct StoreV2 {
    root: PathBuf,
}
pub struct LockedStoreV2<'a> {
    store: &'a StoreV2,
    _guard: TargetFileLock,
}
fn io_error<T>(_: T) -> MatchErrorV2 {
    error("model_storage")
}
pub fn empty_policy_v2() -> PolicyV2 {
    PolicyV2 {
        schema_version: 2,
        channels: vec![],
        template_refs: vec![],
        default_channel_id: None,
        pending_removals: vec![],
        recovery_id: None,
    }
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value = parse_strict_json_v2(bytes)?;
    if value["schemaVersion"] != 2 {
        return Err(error("model_schema_unsupported"));
    }
    serde_json::from_value(value).map_err(json_error)
}
fn validate_draft(draft: &DraftV2) -> Result<()> {
    if draft.schema_version != 2 || draft.base.dto_version != 2 {
        return Err(error("model_schema_unsupported"));
    }
    check_id(&draft.base.draft_id)?;
    for digest in [
        &draft.base.source_digest,
        &draft.base.policy_digest,
        &draft.base.library_digest,
    ] {
        if !valid_digest(digest) {
            return Err(error("model_source_invalid"));
        }
    }
    if draft.base.root_revision.is_empty() {
        return Err(error("model_source_invalid"));
    }
    validate_policy_v2(&draft.policy)?;
    for (channel_id, edit) in &draft.credential_edits {
        check_id(channel_id)?;
        let channel = draft
            .policy
            .channels
            .iter()
            .find(|channel| &channel.id == channel_id)
            .ok_or_else(|| error("model_credential_edit_invalid"))?;
        match edit {
            DraftCredentialEditV2::Replace { credential_ref } => {
                check_id(credential_ref)?;
                if channel.connection.credential_ref.as_ref() != Some(credential_ref) {
                    return Err(error("model_credential_edit_invalid"));
                }
            }
            DraftCredentialEditV2::Clear if channel.connection.credential_ref.is_some() => {
                return Err(error("model_credential_edit_invalid"));
            }
            DraftCredentialEditV2::Clear => {}
        }
    }
    if let Some(baseline) = &draft.observed_baseline {
        validate_policy_v2(baseline)?;
        // Existing runtime identities cannot be silently rebound to another
        // management UUID. A new draft-only UUID/provider is an explicit add;
        // model edits, source refreshes and legitimate rename remain allowed.
        for observed in &baseline.channels {
            if draft.policy.channels.iter().any(|channel| {
                channel.connection.provider_id == observed.connection.provider_id
                    && channel.id != observed.id
            }) {
                return Err(error("model_source_invalid"));
            }
        }
    }
    if draft
        .observed_config_digest
        .as_deref()
        .is_some_and(|value| !valid_digest(value))
    {
        return Err(error("model_source_invalid"));
    }
    if draft.observed_providers.as_ref().is_some_and(|providers| {
        providers.len() > 10000
            || providers.iter().any(|(id, row)| {
                id.is_empty()
                    || id.len() > 512
                    || !valid_digest(&row.row_digest)
                    || !valid_digest(&row.name_digest)
                    || !valid_digest(&row.non_connection_digest)
            })
    }) {
        return Err(error("model_source_invalid"));
    }
    Ok(())
}
impl StoreV2 {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
        }
    }
    pub fn lock(&self) -> Result<LockedStoreV2<'_>> {
        let target = self.root.join(LOCK_PATH);
        safety::check_path(&self.root, &target, true).map_err(io_error)?;
        safety::check_path(&self.root, &lock_path_for(&target), true).map_err(io_error)?;
        let guard = TargetFileLock::lock(&target).map_err(io_error)?;
        Ok(LockedStoreV2 {
            store: self,
            _guard: guard,
        })
    }
    pub fn with_locked<T>(&self, work: impl FnOnce(&LockedStoreV2<'_>) -> Result<T>) -> Result<T> {
        let locked = self.lock()?;
        work(&locked)
    }
    pub fn snapshot(&self) -> Result<StoreSnapshotV2> {
        self.with_locked(|locked| locked.snapshot_unlocked())
    }
    /// Saved-source capture only: never examines the unrelated draft target.
    /// Mutations still read the complete snapshot under their own CAS checks.
    pub fn snapshot_saved(&self) -> Result<StoreSnapshotV2> {
        self.with_locked(|locked| locked.snapshot_saved_unlocked())
    }
    pub fn save_library(&self, request: &SaveRevisionRequestV2) -> Result<LibraryV2> {
        self.with_locked(|locked| locked.save_library_unlocked(request))
    }
    pub fn stage_draft(&self, expected: Option<&ReadBaseV2>, next: DraftV2) -> Result<DraftV2> {
        self.with_locked(|locked| locked.stage_draft_unlocked(expected, next))
    }
    fn path(&self, relative: &str) -> Result<PathBuf> {
        if ![
            LIBRARY_PATH,
            POLICY_PATH,
            DRAFT_PATH,
            LEDGER_PATH,
            MAPPING_PATH,
        ]
        .contains(&relative)
        {
            return Err(error("model_storage"));
        }
        let path = self.root.join(relative);
        safety::check_path(&self.root, &path, true).map_err(io_error)?;
        Ok(path)
    }
    fn bytes(&self, relative: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path(relative)?;
        match std::fs::symlink_metadata(&path) {
            Ok(_) => Ok(Some(safety::read_file(&path, MAX_BYTES).map_err(io_error)?)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(io_error(err)),
        }
    }
    fn atomic(&self, relative: &str, bytes: &[u8]) -> Result<()> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err(error("model_limit"));
        }
        let path = self.path(relative)?;
        atomic_write(&path, bytes, 0o600).map_err(io_error)?;
        if self.bytes(relative)?.as_deref() != Some(bytes) {
            return Err(error("model_storage_readback"));
        }
        Ok(())
    }
}
impl LockedStoreV2<'_> {
    pub fn snapshot_unlocked(&self) -> Result<StoreSnapshotV2> {
        self.snapshot_with_draft_unlocked(true)
    }
    /// Caller already owns model-v2 after the normal config lock. This does
    /// not recover transactions, inspect draft paths or grant write authority.
    pub fn snapshot_saved_unlocked(&self) -> Result<StoreSnapshotV2> {
        self.snapshot_with_draft_unlocked(false)
    }
    fn snapshot_with_draft_unlocked(&self, include_draft: bool) -> Result<StoreSnapshotV2> {
        let mut files = BTreeMap::new();
        let mut raw = BTreeMap::new();
        for relative in [
            LIBRARY_PATH,
            POLICY_PATH,
            DRAFT_PATH,
            LEDGER_PATH,
            MAPPING_PATH,
        ] {
            if relative == DRAFT_PATH && !include_draft {
                // Skip before path()/check_path()/metadata/read, including a
                // symlink, unreadable, broken or newer-format draft target.
                files.insert(relative.into(), digest(b""));
                raw.insert(relative, None);
                continue;
            }
            let bytes = self.store.bytes(relative)?;
            files.insert(
                relative.into(),
                digest(bytes.as_deref().unwrap_or_default()),
            );
            raw.insert(relative, bytes);
        }
        let library: LibraryV2 = match &raw[LIBRARY_PATH] {
            Some(bytes) => {
                let library = decode(bytes)?;
                validate_library_v2(&library)?;
                library
            }
            // The immutable compiled seed is validated before its cached clone
            // is returned. Every user-provided file still validates above.
            None => seed_library_v2()?,
        };
        let policy: PolicyV2 = match &raw[POLICY_PATH] {
            Some(bytes) => decode(bytes)?,
            None => empty_policy_v2(),
        };
        validate_policy_v2(&policy)?;
        let draft: Option<DraftV2> = raw[DRAFT_PATH].as_deref().map(decode).transpose()?;
        if let Some(draft) = &draft {
            validate_draft(draft)?;
        }
        let ledger = raw[LEDGER_PATH]
            .as_deref()
            .map(decode::<LedgerFileV2>)
            .transpose()?
            .map(|l| l.ledger);
        if let Some(ledger) = &ledger {
            if ledger.source_policy_digest != policy_semantic_digest_v2(&policy)?
                || !valid_digest(&ledger.written_file_digest)
            {
                return Err(error("model_source_provenance_unknown"));
            }
        } else if raw[POLICY_PATH].is_some() && !policy.channels.is_empty() {
            return Err(error("model_source_provenance_unknown"));
        }
        let mappings = raw[MAPPING_PATH]
            .as_deref()
            .map(decode::<MappingFileV2>)
            .transpose()?
            .unwrap_or_default();
        validate_mappings_v2(&mappings)?;
        let digests = StoreDigestsV2 {
            library: digest(&canonical_bytes(&library)?),
            files,
        };
        Ok(StoreSnapshotV2 {
            library,
            policy,
            draft,
            ledger,
            mappings,
            digests,
        })
    }
    pub fn save_library_unlocked(&self, request: &SaveRevisionRequestV2) -> Result<LibraryV2> {
        let current = self.snapshot_unlocked()?;
        let next = save_revision_plan_v2(&current.library, request)?;
        self.store.atomic(LIBRARY_PATH, &canonical_bytes(&next)?)?;
        Ok(next)
    }
    pub fn stage_draft_unlocked(
        &self,
        expected: Option<&ReadBaseV2>,
        next: DraftV2,
    ) -> Result<DraftV2> {
        validate_draft(&next)?;
        let snapshot = self.snapshot_unlocked()?;
        match (&snapshot.draft, expected) {
            (None, None) if next.base.draft_generation == 0 => {}
            (Some(old), Some(expected)) if &old.base == expected => {
                if old == &next {
                    return Ok(old.clone());
                }
                if next.base.draft_id != old.base.draft_id
                    || next.base.draft_generation
                        != old
                            .base
                            .draft_generation
                            .checked_add(1)
                            .ok_or_else(|| error("model_limit"))?
                {
                    return Err(error("model_read_stale"));
                }
            }
            _ => return Err(error("model_read_stale")),
        }
        if next.base.library_digest != snapshot.digests.library {
            return Err(error("model_read_stale"));
        }
        self.store.atomic(DRAFT_PATH, &canonical_bytes(&next)?)?;
        Ok(next)
    }
    /// Returns bytes only. L0 journals config + these objects under config→model-v2.
    pub fn prepare_save_unlocked(
        &self,
        expected: &StoreDigestsV2,
        policy: &PolicyV2,
        ledger: &MaterializationLedgerV2,
        mappings: &MappingFileV2,
    ) -> Result<Vec<PendingWriteV2>> {
        let snapshot = self.snapshot_unlocked()?;
        if &snapshot.digests != expected {
            return Err(error("model_save_stale"));
        }
        validate_policy_v2(policy)?;
        validate_mappings_v2(mappings)?;
        if ledger.source_policy_digest != policy_semantic_digest_v2(policy)?
            || !valid_digest(&ledger.written_file_digest)
        {
            return Err(error("model_source_provenance_unknown"));
        }
        for reference in policy
            .template_refs
            .iter()
            .chain(policy.channels.iter().flat_map(|c| {
                c.template_refs.iter().chain(
                    c.models
                        .iter()
                        .filter_map(|m| m.binding.as_ref().map(ModelBindingV2::reference)),
                )
            }))
        {
            revision(&snapshot.library, reference)?;
        }
        let values = [
            (POLICY_PATH, canonical_bytes(policy)?),
            (
                LEDGER_PATH,
                canonical_bytes(&LedgerFileV2 {
                    schema_version: 2,
                    ledger: ledger.clone(),
                })?,
            ),
            (MAPPING_PATH, canonical_bytes(mappings)?),
        ];
        let mut writes = vec![];
        for (relative, after) in values {
            if after.len() as u64 > MAX_BYTES {
                return Err(error("model_limit"));
            }
            let before = self.store.bytes(relative)?;
            if before.as_ref() != Some(&after) {
                writes.push(PendingWriteV2 {
                    relative_path: relative.into(),
                    before,
                    after,
                });
            }
        }
        Ok(writes)
    }
}
