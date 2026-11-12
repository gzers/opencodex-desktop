//! Immutable first-root provenance lives in the independent index binding,
//! never in a self-authorizing sidecar or a second progress ledger.
use super::super::root_creation;
use super::*;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RootBinding {
    schema_version: u32,
    scope: Vec<u8>,
    original_root: root_creation::References,
}
pub(super) fn encode_binding(
    scope: &[u8],
    origin: Option<&root_creation::References>,
) -> io::Result<Vec<u8>> {
    match origin {
        None => Ok(scope.to_vec()),
        Some(origin) => encode(&RootBinding {
            schema_version: 1,
            scope: scope.to_vec(),
            original_root: origin.clone(),
        }),
    }
}
/// Only independently registered initial events can name an origin, including
/// retained historical generations. This grants no current-root permission.
pub(super) fn verify_initial_event(
    connection: &Connection,
    refs: &root_creation::References,
) -> io::Result<()> {
    let stored: Vec<u8> = connection
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    let head = read_head(connection)?.ok_or_else(|| denied("root origin has no current Head"))?;
    let history = vault_history::read(connection, &head)?;
    let lineage = root_lineage::parse(&stored, &history)?;
    if !lineage.generations().iter().any(|g| g.origin == *refs) {
        return Err(denied(
            "origin is not independently registered in root lineage",
        ));
    }
    let scope: serde_json::Value = serde_json::from_slice(&scope_bytes(&stored)?)
        .map_err(|_| denied("root lineage context invalid"))?;
    let data = scope
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| denied("root lineage data path missing"))?;
    let manager = PinnedDirectory::open(Path::new(data))?.child_private("manager-state", false)?;
    root_creation::verify_generation_slots(&manager, &lineage.slots())?;
    let confirmed: Vec<u8> = connection
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    if confirmed != stored || vault_history::read(connection, &head)? != history {
        return Err(denied("root origin source changed during observation"));
    }
    Ok(())
}
/// Audit the entire bounded indexed lineage and exact namespace through the
/// held connection. The singleton binding never changes.
pub(super) fn active_binding(
    connection: &Connection,
    stored: &[u8],
    head: &Head,
) -> io::Result<(Vec<u8>, root_creation::References)> {
    let history = vault_history::read(connection, head)?;
    let lineage = root_lineage::parse(stored, &history)?;
    let scope: serde_json::Value = serde_json::from_slice(&scope_bytes(stored)?)
        .map_err(|_| denied("root lineage context invalid"))?;
    let data = scope
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| denied("root lineage data path missing"))?;
    let profile = scope
        .get(3)
        .and_then(|v| v.as_str())
        .ok_or_else(|| denied("root lineage profile path missing"))?;
    let data = PinnedDirectory::open(Path::new(data))?;
    let manager = data.child_private("manager-state", false)?;
    let slots = lineage.slots();
    root_creation::verify_generation_slots(&manager, &slots)?;
    vault_history::inspect_root_history(&data, Path::new(profile), stored, &history)?;
    root_creation::verify_generation_slots(&manager, &slots)?;
    let confirmed: Vec<u8> = connection
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    if confirmed != stored || vault_history::read(connection, head)? != history {
        return Err(denied("root lineage changed during complete observation"));
    }
    Ok((
        lineage.current().binding.clone(),
        lineage.current().origin.clone(),
    ))
}

pub(super) fn scope_bytes(stored: &[u8]) -> io::Result<Vec<u8>> {
    let binding: RootBinding = decode(stored)?;
    if binding.schema_version != 1 {
        return Err(denied("unknown root binding"));
    }
    Ok(binding.scope)
}
pub(super) fn original_root(stored: &[u8]) -> io::Result<root_creation::References> {
    let binding: RootBinding = decode(stored)?;
    if binding.schema_version != 1 {
        return Err(denied("unknown original-root binding"));
    }
    Ok(binding.original_root)
}

/// Facts-only reopen from the current independent head and origin binding.
/// No Original digest is learned by scanning an on-disk frame.
pub(super) fn verify_capture(
    stored: &[u8],
    origin: &root_creation::References,
    data: &PinnedDirectory,
    profile: &Path,
    client: &PinnedDirectory,
    head: &Head,
) -> io::Result<Option<Originals>> {
    let Some(refs) = &head.references else {
        return Ok(None);
    };
    validate(head)?;
    if refs.attempt != origin.id
        || !matches!(
            head.state,
            State::Captured
                | State::Ready
                | State::StopPrepared
                | State::Stopped
                | State::RestorePrepared
                | State::Failed
        )
    {
        return Err(denied("first-root capture has unexpected phase evidence"));
    }
    let originals = Originals::open_profile_origin(
        data.path(),
        client.path(),
        profile,
        super::super::OriginalKeys {
            attempt: &refs.attempt,
            original_sha256: &refs.original_sha256,
            expected_phases: &refs.phases,
        },
        Some(origin.clone()),
    )?;
    let scope: serde_json::Value = serde_json::from_slice(&scope_bytes(stored)?)
        .map_err(|_| denied("root context binding invalid"))?;
    let home = scope
        .get(5)
        .and_then(|v| v.as_str())
        .ok_or_else(|| denied("root-bound OpenCodex HOME missing"))?;
    session::verify_index_scope(&originals, refs, Path::new(home))
        .map_err(|_| denied("registered root Original scope changed"))?;
    Ok(Some(originals))
}

/// Resolve the physical directory from independent SQLite references AFTER
/// opening the index. A missing logical root is never treated as no session.
pub(super) fn resolve(
    context: &ProcessContext,
    data: &PinnedDirectory,
    manager: &PinnedDirectory,
    stored: &[u8],
    head: Option<&Head>,
    connection: &Connection,
) -> io::Result<(PinnedDirectory, Option<root_creation::References>)> {
    resolve_originals(context, data, manager, stored, head, connection)
        .map(|(client, origin, _)| (client, origin))
}
pub(super) fn resolve_originals(
    context: &ProcessContext,
    data: &PinnedDirectory,
    manager: &PinnedDirectory,
    stored: &[u8],
    head: Option<&Head>,
    connection: &Connection,
) -> io::Result<(
    PinnedDirectory,
    Option<root_creation::References>,
    Option<Originals>,
)> {
    let logical = context.working_directory.join(".codex");
    let value = crate::modules::model_config::schema_v2::parse_strict_json_v2(stored)
        .map_err(|_| denied("root scope binding is not strict JSON"))?;
    if value.is_array() {
        if head.is_some_and(|h| h.root_retirement.is_some()) {
            return Err(denied(
                "ordinary existing-root binding cannot claim original root absence",
            ));
        }
        root_creation::require_no_pending_root(manager)?;
        let path = std::fs::canonicalize(&logical)?;
        let client = PinnedDirectory::open(&path)?;
        if scope(context, data, &client)? != stored {
            return Err(denied("ordinary index context changed"));
        }
        return Ok((client, None, None));
    }
    let current = head.ok_or_else(|| denied("registered root has no independent head"))?;
    let (active, origin) = active_binding(connection, stored, current)?;
    let stored = active.as_slice();
    let mut binding: RootBinding = decode(stored)?;
    binding.original_root.inherit_namespace(&origin)?;
    if binding.schema_version != 1 {
        return Err(denied("unknown first root binding schema"));
    }

    let head = head.ok_or_else(|| denied("registered root has no current independent session"))?;
    if let Some(movement) = &head.root_retirement {
        if !matches!(
            head.state,
            State::RootRetirementPrepared | State::RootAbsent
        ) {
            return Err(denied("unexpected root movement phase"));
        }
        let raw: Vec<u8> = connection
            .query_row(
                "SELECT document FROM events WHERE revision=?1",
                [movement.source_revision],
                |r| r.get(0),
            )
            .map_err(database_error)?;
        let source: Head = decode(&raw)?;
        validate(&source)?;
        if source.revision != movement.source_revision
            || source.ticket != head.ticket
            || source.references != head.references
            || source.restore_plan != head.restore_plan
            || source.root_retirement.is_some()
        {
            return Err(denied("root movement rebases independent original source"));
        }
        let archived = observation::entry(
            &PinnedDirectory::open(&context.working_directory)?,
            ".codex",
        )?
        .is_none();
        let originals = if archived {
            super::super::root_retirement::open_archived_originals(
                data.path(),
                &context.working_directory,
                &super::super::root_retirement::ArchiveReferences {
                    source: source.clone(),
                    original_root: binding.original_root.clone(),
                    plan_id: movement.plan.id.clone(),
                    plan_sha256: movement.plan.sha256.clone(),
                },
            )?
        } else {
            if head.state == State::RootAbsent {
                return Err(denied(
                    "client root reappeared after original absence restoration",
                ));
            }
            let refs = source
                .references
                .as_ref()
                .ok_or_else(|| denied("root movement Original refs absent"))?;
            Originals::open_profile_origin(
                data.path(),
                &logical,
                &context.working_directory,
                super::super::OriginalKeys {
                    attempt: &refs.attempt,
                    original_sha256: &refs.original_sha256,
                    expected_phases: &refs.phases,
                },
                Some(binding.original_root.clone()),
            )?
        };
        let pending = super::super::root_retirement::Pending::open(
            &originals,
            &movement.plan.id,
            &movement.plan.sha256,
        )?;
        pending.require_source(&source)?;
        if archived {
            pending.inspect_retired()?;
        } else {
            pending.inspect_live()?;
        }
        if head.state == State::RootAbsent {
            pending.verify_retire_readback(
                head.completion_sha256
                    .as_deref()
                    .ok_or_else(|| denied("root absence completion digest missing"))?,
            )?;
        }
        let client = PinnedDirectory::open(originals.client.path())?;
        if scope_at(context, data, &client, &logical)? != binding.scope {
            return Err(denied("retired root logical context changed"));
        }
        return Ok((client, Some(binding.original_root), Some(originals)));
    }
    let path = std::fs::canonicalize(&logical)?;
    let client = PinnedDirectory::open(&path)?;
    if scope(context, data, &client)? != binding.scope {
        return Err(denied("root origin context changed"));
    }
    root_creation::verify_indexed(
        manager,
        data,
        &context.working_directory,
        &client,
        &binding.original_root,
    )?;
    let originals = verify_capture(
        stored,
        &binding.original_root,
        data,
        &context.working_directory,
        &client,
        head,
    )?;
    Ok((client, Some(binding.original_root), originals))
}
