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
pub(super) fn verify_binding(
    scope: &[u8],
    stored: &[u8],
    manager: &PinnedDirectory,
    data: &PinnedDirectory,
    profile: &Path,
    client: &PinnedDirectory,
) -> io::Result<Option<root_creation::References>> {
    if stored == scope {
        root_creation::require_no_pending_root(manager)?;
        return Ok(None);
    }
    let binding: RootBinding = decode(stored)?;
    if binding.schema_version != 1 || binding.scope != scope {
        return Err(denied("session root binding or context differs"));
    }
    root_creation::verify_indexed(manager, data, profile, client, &binding.original_root)?;
    Ok(Some(binding.original_root))
}

pub(super) fn verify_initial_event(
    connection: &Connection,
    refs: &root_creation::References,
) -> io::Result<()> {
    let raw: Vec<u8> = connection
        .query_row("SELECT document FROM events WHERE revision=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    let first: Head = decode(&raw)?;
    validate(&first)?;
    if first.revision != 1
        || first.ticket != refs.id
        || first.state != State::Preparing
        || first.references.is_some()
        || first.restore_plan.is_some()
        || first.completion_sha256.is_some()
    {
        return Err(denied(
            "root origin is not bound to the atomic first Preparing event",
        ));
    }
    Ok(())
}

pub(super) fn scope_bytes(stored: &[u8]) -> io::Result<Vec<u8>> {
    let binding: RootBinding = decode(stored)?;
    if binding.schema_version != 1 {
        return Err(denied("unknown root binding"));
    }
    Ok(binding.scope)
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
    if refs.attempt != origin.id
        || !refs.phases.is_empty()
        || refs.live_sha256.is_some()
        || !matches!(head.state, State::Captured | State::Failed)
    {
        return Err(denied(
            "first-root capture has unexpected unintegrated phase evidence",
        ));
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
