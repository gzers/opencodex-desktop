//! Bounded content classification for the three catalog artifacts that official
//! 2.99 does not restore byte for byte. This is NOT write authorization.
//!
//! The lifecycle transaction must separately retain native process/file
//! generations, attest readiness and normal exit, hold official claim/N/K
//! coordination, protect its journal, and CAS every write. A hash, audit PID or
//! this assessment alone grants none of that authority. Unknown is not absence.
use crate::modules::model_config::schema_v2::parse_strict_json_v2;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_MODELS: usize = 4096;
const MAX_AUDIT_EVENT: usize = 2048;
const MAX_AUDIT_EVENTS: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Artifact {
    Catalog,
    ModelsCache,
    CatalogAudit,
}

/// Opaque identities come from a separately verified native snapshot. This
/// module compares them but cannot create or verify them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Snapshot<'a> {
    Unknown,
    Absent,
    File {
        bytes: &'a [u8],
        generation: &'a str,
        protection: &'a str,
    },
}
impl std::fmt::Debug for Snapshot<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => f.write_str("Unknown"),
            Self::Absent => f.write_str("Absent"),
            Self::File { bytes, .. } => f
                .debug_struct("File")
                .field("bytes", &bytes.len())
                .finish_non_exhaustive(),
        }
    }
}
impl<'a> Snapshot<'a> {
    fn check(self) -> Result<(), Refusal> {
        match self {
            Self::Unknown => Err(Refusal::Unknown),
            Self::Absent => Ok(()),
            Self::File {
                bytes,
                generation,
                protection,
            } if bytes.len() <= MAX_BYTES
                && !generation.is_empty()
                && generation.len() <= 4096
                && !protection.is_empty()
                && protection.len() <= 65536 =>
            {
                Ok(())
            }
            _ => Err(Refusal::Snapshot),
        }
    }
    fn bytes(self) -> Result<Option<&'a [u8]>, Refusal> {
        self.check()?;
        Ok(match self {
            Self::File { bytes, .. } => Some(bytes),
            _ => None,
        })
    }
    fn content_and_protection(self, other: Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) => true,
            (
                Self::File {
                    bytes: a,
                    protection: ap,
                    ..
                },
                Self::File {
                    bytes: b,
                    protection: bp,
                    ..
                },
            ) => a == b && ap == bp,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Stage<'a> {
    pub catalog: Snapshot<'a>,
    pub cache: Snapshot<'a>,
    pub audit: Snapshot<'a>,
}
impl<'a> Stage<'a> {
    fn rows(self) -> [Snapshot<'a>; 3] {
        [self.catalog, self.cache, self.audit]
    }
}

/// All observations must belong to the same independently captured native
/// lifecycle. Deliberately not deserializable: JSON is no ownership certificate.
pub struct Observations<'a> {
    pub before: Stage<'a>,
    pub ready: Stage<'a>,
    pub pre_stop: Stage<'a>,
    pub stopped: Stage<'a>,
    /// Independently declared before startup; never learned from the cache.
    /// Some(empty) is a complete empty set; None is unresolved discovery.
    pub declared_routes: Option<&'a [String]>,
    /// Native retained generations must independently bind these PIDs.
    pub retained_pids: &'a [u32],
    /// Official audit projection, including redaction/truncation; NOT a path or
    /// a native HOME proof. The caller computes it from its admitted context.
    pub audit_home: &'a str,
    pub audit_not_before: chrono::DateTime<chrono::Utc>,
    pub audit_not_after: chrono::DateTime<chrono::Utc>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unknown,
    Snapshot,
    Context,
    ChangedSinceReady,
    ProtectionChanged,
    Format,
    Catalog,
    Cache,
    Audit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    AlreadyOriginal,
    RestoreOriginal,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment(pub [(Artifact, Action); 3]);

fn json(bytes: &[u8]) -> Result<Value, Refusal> {
    if bytes.len() > MAX_BYTES {
        return Err(Refusal::Format);
    }
    parse_strict_json_v2(bytes).map_err(|_| Refusal::Format)
}
fn model_map(value: &Value) -> Result<BTreeMap<&str, &Value>, Refusal> {
    let rows = value
        .get("models")
        .and_then(Value::as_array)
        .ok_or(Refusal::Catalog)?;
    if rows.len() > MAX_MODELS {
        return Err(Refusal::Catalog);
    }
    let mut map = BTreeMap::new();
    for row in rows {
        let slug = row
            .get("slug")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 1024)
            .ok_or(Refusal::Catalog)?;
        if map.insert(slug, row).is_some() {
            return Err(Refusal::Catalog);
        }
    }
    Ok(map)
}
fn catalog(snapshot: Snapshot<'_>) -> Result<Value, Refusal> {
    match snapshot.bytes()? {
        Some(bytes) => {
            let value = json(bytes)?;
            model_map(&value)?;
            Ok(value)
        }
        None => Ok(serde_json::json!({"models":[]})),
    }
}
fn permitted_fields(after: &Value, before: &Value, ready: &Value) -> bool {
    after.as_object().is_some_and(|map| {
        map.iter().all(|(key, value)| {
            key == "models" || before.get(key) == Some(value) || ready.get(key) == Some(value)
        })
    })
}
fn native_subset(after: &Value, before: &Value, ready: &Value) -> Result<(), Refusal> {
    let originals = model_map(before)?;
    let prepared = model_map(ready)?;
    let actual = model_map(after)?;
    if !permitted_fields(after, before, ready)
        || actual.iter().any(|(slug, row)| {
            !originals.contains_key(slug)
                || (originals.get(slug) != Some(row) && prepared.get(slug) != Some(row))
        })
    {
        return Err(Refusal::Catalog);
    }
    Ok(())
}
fn cache(bytes: &[u8]) -> Result<Value, Refusal> {
    let value = json(bytes)?;
    let map = value.as_object().ok_or(Refusal::Cache)?;
    if map.len() != 3
        || value["fetched_at"] != "2000-01-01T00:00:00Z"
        || value["client_version"] != "0.0.0"
        || !map.contains_key("models")
    {
        return Err(Refusal::Cache);
    }
    model_map(&value).map_err(|_| Refusal::Cache)?;
    Ok(value)
}
fn audit_suffix(
    before: Snapshot<'_>,
    after: Snapshot<'_>,
    ctx: &Observations<'_>,
) -> Result<(), Refusal> {
    let original = before.bytes()?.unwrap_or_default();
    let actual = after.bytes()?.unwrap_or_default();
    let suffix = actual.strip_prefix(original).ok_or(Refusal::Audit)?;
    if suffix.is_empty() {
        return Ok(());
    }
    if !suffix.ends_with(b"\n") {
        return Err(Refusal::Audit);
    }
    let mut count = 0;
    for line in suffix[..suffix.len() - 1].split(|b| *b == b'\n') {
        count += 1;
        if line.len() > MAX_AUDIT_EVENT || count > MAX_AUDIT_EVENTS {
            return Err(Refusal::Audit);
        }
        let row = json(line)?;
        let map = row.as_object().ok_or(Refusal::Audit)?;
        let fields = [
            "at",
            "pid",
            "ppid",
            "command",
            "opencodexHome",
            "target",
            "outcome",
            "reason",
            "intent",
            "writer",
            "configSource",
            "routedBefore",
            "routedAfter",
        ];
        if map.keys().any(|k| !fields.contains(&k.as_str())) {
            return Err(Refusal::Audit);
        }
        if fields
            .iter()
            .filter(|key| **key != "reason")
            .any(|key| !map.contains_key(*key))
        {
            return Err(Refusal::Audit);
        }
        let writer = row["writer"].as_str().ok_or(Refusal::Audit)?;
        let allowed = matches!(
            (writer, row["target"].as_str(), row["intent"].as_str()),
            (
                "retained-sync" | "convergence",
                Some("catalog"),
                Some("refresh")
            ) | (
                "codex-restore" | "catalog-restore",
                Some("catalog"),
                Some("restore")
            ) | (
                "startup-cache" | "cache-invalidate" | "cache-from-catalog" | "sync-cache",
                Some("cache"),
                Some("cache")
            )
        );
        let pid = row["pid"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Refusal::Audit)?;
        let parent = row["ppid"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Refusal::Audit)?;
        let at = chrono::DateTime::parse_from_rfc3339(row["at"].as_str().ok_or(Refusal::Audit)?)
            .map_err(|_| Refusal::Audit)?;
        if !allowed
            || !ctx.retained_pids.contains(&pid)
            || !ctx.retained_pids.contains(&parent)
            || row["command"] != writer
            || row["opencodexHome"] != ctx.audit_home
            || row["outcome"] != "written"
            || row["configSource"] != "file"
            || at < ctx.audit_not_before
            || at > ctx.audit_not_after
        {
            return Err(Refusal::Audit);
        }
        for name in ["routedBefore", "routedAfter"] {
            if !row[name].is_null() && row[name].as_u64().is_none_or(|n| n > MAX_MODELS as u64) {
                return Err(Refusal::Audit);
            }
        }
        if row.get("reason").is_some_and(|v| !v.is_null()) {
            return Err(Refusal::Audit);
        }
    }
    Ok(())
}

/// Content eligibility only. Applying it additionally requires fresh native
/// ownership/exit, official locks and CAS. No partial plan is returned when any
/// artifact conflicts. Native generation changes after this call must refuse.
pub fn assess(ctx: &Observations<'_>) -> Result<Assessment, Refusal> {
    let declarations = ctx.declared_routes.ok_or(Refusal::Context)?;
    let routes: BTreeSet<_> = declarations.iter().map(String::as_str).collect();
    if routes.len() != declarations.len()
        || routes.len() > MAX_MODELS
        || routes.iter().any(|s| s.is_empty() || s.len() > 1024)
        || ctx.retained_pids.is_empty()
        || ctx.retained_pids.len() > 256
        || ctx.retained_pids.contains(&0)
        || ctx.retained_pids.iter().collect::<BTreeSet<_>>().len() != ctx.retained_pids.len()
        || ctx.audit_not_before > ctx.audit_not_after
        || ctx.audit_home.is_empty()
        || ctx.audit_home.encode_utf16().count() > 256
    {
        return Err(Refusal::Context);
    }
    for stage in [ctx.before, ctx.ready, ctx.pre_stop, ctx.stopped] {
        for row in stage.rows() {
            row.check()?;
        }
    }
    if ctx.ready.rows() != ctx.pre_stop.rows() {
        return Err(Refusal::ChangedSinceReady);
    }
    for ((original, ready), after) in ctx
        .before
        .rows()
        .into_iter()
        .zip(ctx.ready.rows())
        .zip(ctx.stopped.rows())
    {
        if let Snapshot::File { protection, .. } = after {
            if ![original, ready]
                .iter()
                .any(|state| matches!(state,Snapshot::File{protection:p,..} if *p==protection))
            {
                return Err(Refusal::ProtectionChanged);
            }
        }
    }
    let before = catalog(ctx.before.catalog)?;
    let ready = catalog(ctx.ready.catalog)?;
    let originals = model_map(&before)?;
    let prepared = model_map(&ready)?;
    if routes.iter().any(|slug| originals.contains_key(slug))
        || prepared
            .keys()
            .any(|slug| !routes.contains(slug) && !originals.contains_key(slug))
        || routes.iter().any(|slug| !prepared.contains_key(slug))
    {
        return Err(Refusal::Catalog);
    }
    let catalog_action = if ctx
        .stopped
        .catalog
        .content_and_protection(ctx.before.catalog)
    {
        Action::AlreadyOriginal
    } else {
        if ctx.stopped.catalog == Snapshot::Absent {
            return Err(Refusal::Catalog);
        }
        let after = catalog(ctx.stopped.catalog)?;
        native_subset(&after, &before, &ready)?;
        Action::RestoreOriginal
    };
    let cache_action = if ctx.stopped.cache.content_and_protection(ctx.before.cache) {
        Action::AlreadyOriginal
    } else {
        let ready_bytes = ctx.ready.cache.bytes()?.ok_or(Refusal::Cache)?;
        let ready_cache = cache(ready_bytes)?;
        if ready_cache["models"] != ready["models"] {
            return Err(Refusal::Cache);
        }
        let after_bytes = ctx.stopped.cache.bytes()?.ok_or(Refusal::Cache)?;
        if after_bytes != ready_bytes {
            let after = cache(after_bytes)?;
            // The cache envelope is validated independently above. Compare its
            // rows with the catalogs, not the cache-only wrapper fields.
            let model_projection = serde_json::json!({"models":after["models"]});
            native_subset(&model_projection, &before, &ready).map_err(|_| Refusal::Cache)?;
        }
        Action::RestoreOriginal
    };
    // Diagnostic log consistency is supplemental; it produces no ownership proof.
    audit_suffix(ctx.before.audit, ctx.ready.audit, ctx)?;
    audit_suffix(ctx.before.audit, ctx.stopped.audit, ctx)?;
    if !ctx.stopped.audit.content_and_protection(ctx.before.audit)
        && !ctx
            .stopped
            .audit
            .bytes()?
            .unwrap_or_default()
            .starts_with(ctx.ready.audit.bytes()?.unwrap_or_default())
    {
        return Err(Refusal::Audit);
    }
    let audit_action = if ctx.stopped.audit.content_and_protection(ctx.before.audit) {
        Action::AlreadyOriginal
    } else {
        Action::RestoreOriginal
    };
    Ok(Assessment([
        (Artifact::Catalog, catalog_action),
        (Artifact::ModelsCache, cache_action),
        (Artifact::CatalogAudit, audit_action),
    ]))
}

#[cfg(test)]
#[path = "client_artifact_policy_tests.rs"]
mod tests;
