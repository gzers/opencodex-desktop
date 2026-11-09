//! Dedicated L0-resolved transaction directory. Atomic checksummed records are
//! independent of payload files; verify can work after local payload eviction.
use super::types::*;
use serde::{
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn digest(bytes: &[u8]) -> Digest {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn canonical<T: Serialize>(value: &T) -> SyncResult<Vec<u8>> {
    fn sort(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .collect::<std::collections::BTreeMap<_, _>>()
                    .into_iter()
                    .map(|(k, v)| (k, sort(v)))
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.into_iter().map(sort).collect()),
            v => v,
        }
    }
    serde_json::to_vec(&sort(
        serde_json::to_value(value)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?,
    ))
    .map_err(|_| failure(ErrorCode::SyncJournalInvalid, Step::Persist))
}
pub fn plan_digest<T: Serialize>(value: &T) -> SyncResult<Digest> {
    let mut v = serde_json::to_value(value)
        .map_err(|_| failure(ErrorCode::SyncPlanChanged, Step::Decode))?;
    v.as_object_mut()
        .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::Decode))?
        .remove("planDigest");
    Ok(digest(&canonical(&v)?))
}
/// serde_json::Value normally accepts duplicate keys. That would allow two
/// readers to disagree about a signed/hash-bound field, so reject recursively.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("integer-only JSON with unique fields and no null")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut v = Vec::new();
                while let Some(Unique(x)) = a.next_element()? {
                    v.push(x);
                }
                Ok(Unique(Value::Array(v)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut v = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if v.contains_key(&k) {
                        return Err(de::Error::custom("duplicate JSON field"));
                    }
                    let Unique(x) = a.next_value()?;
                    v.insert(k, x);
                }
                Ok(Unique(Value::Object(v)))
            }
        }
        d.deserialize_any(V)
    }
}
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> SyncResult<T> {
    let Unique(v) = serde_json::from_slice(bytes)
        .map_err(|_| failure(ErrorCode::SyncRemoteInvalid, Step::Decode))?;
    for field in ["dtoVersion", "journalVersion"] {
        if v.get(field).is_some_and(|n| n.as_u64() != Some(1)) {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
        }
    }
    serde_json::from_value(v).map_err(|_| failure(ErrorCode::SyncRemoteInvalid, Step::Decode))
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record<T> {
    version: u32,
    body: T,
    digest: Digest,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Seen {
    pub generations: Vec<GenerationRef>,
    pub names: BTreeSet<String>,
    #[serde(default)]
    pub commits: BTreeMap<String, Digest>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiveRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download: Option<DownloadPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoveryPlan>,
    pub payload_handle: Uuid,
    pub payload_schema: PayloadSchemaRef,
    pub root_revision: Digest,
    pub connection_revision: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact: Option<OperationFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<RecoveryReceipt>,
}
#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}
pub struct FileGuard(File);
impl Drop for FileGuard {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
fn io_error(_: impl fmt::Debug) -> SyncFailure {
    failure(ErrorCode::SyncLocalStateIo, Step::Persist)
}
impl Store {
    /// L0 supplies the dedicated transaction directory, never a business HOME.
    pub fn open(root: &Path) -> SyncResult<Self> {
        if !root.is_absolute() {
            return Err(io_error("root must be absolute"));
        }
        crate::infrastructure::platform::create_private_dir_all(root, 0o700).map_err(io_error)?;
        if fs::symlink_metadata(root)
            .map_err(io_error)?
            .file_type()
            .is_symlink()
        {
            return Err(io_error("symlink root"));
        }
        Ok(Self {
            root: root.canonicalize().map_err(io_error)?,
        })
    }
    fn path(&self, name: &str) -> SyncResult<PathBuf> {
        if name.is_empty()
            || name.len() > 240
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            || name == "."
            || name == ".."
        {
            return Err(io_error("invalid record name"));
        }
        let p = self.root.join(name);
        match fs::symlink_metadata(&p) {
            Ok(m) if !m.is_file() || m.file_type().is_symlink() => {
                return Err(io_error("nonregular record"))
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(io_error(e)),
        }
        Ok(p)
    }
    fn lock_file(&self, key: &str) -> SyncResult<File> {
        let p = self.path(&format!("{key}.lock"))?;
        #[cfg(not(windows))]
        let mut o = OpenOptions::new();
        #[cfg(not(windows))]
        o.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        #[cfg(windows)]
        {
            crate::infrastructure::platform::open_or_create_private_file(&p, 0o600)
                .map_err(io_error)
        }
        #[cfg(not(windows))]
        {
            o.open(p).map_err(io_error)
        }
    }
    pub async fn lock(&self, key: &str) -> SyncResult<FileGuard> {
        #[cfg(all(test, windows))]
        let _timing = super::TestPhaseTiming::start("store-lock-wait");
        let f = self.lock_file(key)?;
        tokio::task::spawn_blocking(move || {
            f.lock().map_err(io_error)?;
            Ok(FileGuard(f))
        })
        .await
        .map_err(io_error)?
    }
    pub fn try_lock(&self, key: &str) -> SyncResult<Option<FileGuard>> {
        let f = self.lock_file(key)?;
        match f.try_lock() {
            Ok(()) => Ok(Some(FileGuard(f))),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(e) => Err(io_error(e)),
        }
    }
    fn read(&self, name: &str, limit: usize) -> SyncResult<Option<Vec<u8>>> {
        let p = self.path(name)?;
        let mut o = OpenOptions::new();
        o.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.custom_flags(libc::O_NOFOLLOW);
        }
        let f = match o.open(p) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_error(e)),
        };
        let mut b = Vec::new();
        f.take(limit as u64 + 1)
            .read_to_end(&mut b)
            .map_err(io_error)?;
        if b.len() > limit {
            return Err(failure(ErrorCode::SyncLimit, Step::Persist));
        }
        Ok(Some(b))
    }
    fn atomic(&self, name: &str, bytes: &[u8]) -> SyncResult<()> {
        let p = self.path(name)?;
        crate::infrastructure::atomic_write::atomic_write(&p, bytes, 0o600).map_err(io_error)?;
        #[cfg(unix)]
        File::open(&self.root)
            .and_then(|d| d.sync_all())
            .map_err(io_error)?;
        Ok(())
    }
    fn write_record<T: Serialize>(&self, name: &str, body: &T) -> SyncResult<()> {
        let digest = digest(&canonical(body)?);
        let bytes = canonical(&Record {
            version: 1,
            body,
            digest,
        })?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(failure(ErrorCode::SyncLimit, Step::Persist));
        }
        self.atomic(name, &bytes)
    }
    fn read_record<T: DeserializeOwned + Serialize>(&self, name: &str) -> SyncResult<Option<T>> {
        let Some(bytes) = self.read(name, 16 * 1024 * 1024)? else {
            return Ok(None);
        };
        let record: Record<T> =
            decode(&bytes).map_err(|_| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        if record.version != 1 || digest(&canonical(&record.body)?) != record.digest {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        Ok(Some(record.body))
    }
    pub fn freeze(&self, id: &str, bytes: &[u8]) -> SyncResult<()> {
        require_uuid(id)?;
        if bytes.len() > Budgets::default().payload_bytes {
            return Err(failure(ErrorCode::SyncLimit, Step::Persist));
        }
        if let Some(old) = self.read(&format!("{id}.payload"), Budgets::default().payload_bytes)? {
            if old != bytes {
                return Err(failure(ErrorCode::SyncOperationCollision, Step::Persist));
            }
            return Ok(());
        }
        let p = self.path(&format!("{id}.payload"))?;
        let mut f = tempfile::NamedTempFile::new_in(&self.root).map_err(io_error)?;
        f.write_all(bytes).map_err(io_error)?;
        f.as_file().sync_all().map_err(io_error)?;
        f.persist_noclobber(p).map_err(io_error)?;
        Ok(())
    }
    pub fn payload(&self, id: &str, expected: &str) -> SyncResult<Vec<u8>> {
        require_uuid(id)?;
        require_digest(expected)?;
        let b = self
            .read(&format!("{id}.payload"), Budgets::default().payload_bytes)?
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        if digest(&b) != expected {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        Ok(b)
    }
    pub fn journal(&self, id: &str) -> SyncResult<Journal> {
        require_uuid(id)?;
        let j: Journal = self
            .read_record(&format!("{id}.upload.json"))?
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
        if j.operation_id != id
            || j.plan.operation_id != id
            || j.fact.operation_id != id
            || plan_digest(&j.plan)? != j.plan.plan_digest
            || j.journal_version != 1
            || j.plan.dto_version != 1
            || j.fact.dto_version != 1
            || j.fact.payload_digest != j.plan.payload_digest
            || j.fact.kind != OperationKind::Upload
            || j.fact.local_apply_status != LocalApplyStatus::NotApplicable
            || j.previous_failures.len() > 64
            || (j.plan.initialization == Initialization::OnConfirm
                && (j.plan.generation.is_some() || !j.plan.parents.is_empty()))
            || (j.plan.initialization == Initialization::None
                && j.plan.generation != j.fact.generation)
        {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        require_uuid(&j.payload_handle)?;
        require_digest(&j.root_revision)?;
        require_digest(&j.connection_revision)?;
        require_guard(&j.plan.source_guard)?;
        if j.fact.last_write_step != WriteStep::None
            && (j.generation_bytes_utf8.is_none() || j.commit_bytes_utf8.is_none())
        {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        if j.fact.phase == Phase::CancelledBeforeWrite && j.fact.last_write_step != WriteStep::None
        {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        if j.fact.storage_status == StorageStatus::Stored
            && (j.fact.commit.is_none() || j.fact.stored_at.is_none())
        {
            return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
        }
        if let Some(ref bytes) = j.generation_bytes_utf8 {
            let g: Generation = decode(bytes.as_bytes())?;
            let reference = j
                .fact
                .generation
                .as_ref()
                .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
            if g.format != "OCXD_CONFIG_GENERATION"
                || g.format_version != 1
                || g.generation_id != reference.generation_id
                || digest(bytes.as_bytes()) != reference.digest
            {
                return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
            }
        }
        if let Some(ref bytes) = j.commit_bytes_utf8 {
            let c: Commit = decode(bytes.as_bytes())?;
            let generation = j
                .fact
                .generation
                .as_ref()
                .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
            super::graph::validate_commit(&c, &generation.generation_id, id, &Budgets::default())?;
            if c.operation_id != id
                || c.payload_digest != j.plan.payload_digest
                || c.payload_bytes != j.plan.payload_bytes
                || c.parents != j.plan.parents
                || c.payload_schema != j.plan.payload_schema
                || c.intent != j.plan.intent
                || c.resolution != j.plan.resolution
                || j.fact.commit.as_ref()
                    != Some(&CommitRef {
                        operation_id: id.into(),
                        digest: digest(bytes.as_bytes()),
                    })
                || execution_digest(&j, &c)? != j.execution_digest.clone().unwrap_or_default()
            {
                return Err(failure(ErrorCode::SyncJournalInvalid, Step::Persist));
            }
        }
        Ok(j)
    }
    pub fn save_journal(&self, j: &Journal) -> SyncResult<()> {
        require_uuid(&j.operation_id)?;
        self.write_record(&format!("{}.upload.json", j.operation_id), j)
    }
    pub fn seen(&self, binding: &str) -> SyncResult<Seen> {
        require_digest(binding)?;
        Ok(self
            .read_record(&format!("{}.seen.json", &binding[7..]))?
            .unwrap_or_default())
    }
    pub fn save_seen(&self, binding: &str, v: &Seen) -> SyncResult<()> {
        require_digest(binding)?;
        self.write_record(&format!("{}.seen.json", &binding[7..]), v)
    }
    pub fn receive(&self, id: &str) -> SyncResult<ReceiveRecord> {
        require_uuid(id)?;
        self.read_record(&format!("{id}.receive.json"))?
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))
    }
    pub fn save_receive(&self, id: &str, v: &ReceiveRecord) -> SyncResult<()> {
        require_uuid(id)?;
        self.write_record(&format!("{id}.receive.json"), v)
    }
    pub fn is_cancelled(&self, id: &str) -> SyncResult<bool> {
        require_uuid(id)?;
        Ok(self.read(&format!("{id}.cancel"), 1)?.is_some())
    }
    pub fn set_cancelled(&self, id: &str) -> SyncResult<()> {
        require_uuid(id)?;
        self.atomic(&format!("{id}.cancel"), b"1")
    }
    pub fn clear_cancelled(&self, id: &str) -> SyncResult<()> {
        require_uuid(id)?;
        let p = self.path(&format!("{id}.cancel"))?;
        match fs::remove_file(p) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io_error(e)),
        }
    }
}
pub fn execution_digest(j: &Journal, c: &Commit) -> SyncResult<Digest> {
    let g = j
        .fact
        .generation
        .as_ref()
        .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid, Step::Persist))?;
    Ok(digest(&canonical(
        &serde_json::json!({"hashDomain":"OCXD_CONFIG_EXECUTION_V1","operationId":j.operation_id,"planDigest":j.plan.plan_digest,"generation":g,"commitDigest":digest(j.commit_bytes_utf8.as_ref().ok_or_else(||failure(ErrorCode::SyncJournalInvalid,Step::Persist))?.as_bytes()),"payloadDigest":c.payload_digest}),
    )?))
}
