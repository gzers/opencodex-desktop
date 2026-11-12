//! Index-first inventory of RETAINED CLOSED attempts. Directory observations
//! are only compared to explicit indexed references, never used to invent an
//! attempt/hash or adopt a partially published capture.
use super::*;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Evidence {
    history_sha256: String,
    vault: Option<(Identity, Protection)>,
    attempts: Vec<(String, Identity, String)>,
    reserved_capture: Option<(String, Option<TreeSnapshot>)>,
}
impl Evidence {
    pub(super) fn matches_history(&self, history: &[Head]) -> bool {
        encode(&history).is_ok_and(|raw| digest(&raw) == self.history_sha256)
    }
    pub(super) fn reserved_capture(&self) -> Option<&(String, Option<TreeSnapshot>)> {
        self.reserved_capture.as_ref()
    }
}
/// Read only through the caller's EXISTING SQLite connection. Never open or
/// close an auxiliary index fd while its process-associated lock is held.
pub(super) fn read(connection: &Connection, current: &Head) -> io::Result<Vec<Head>> {
    let count: u64 = connection
        .query_row(
            "SELECT count(*) FROM events WHERE revision<=?1",
            [current.revision],
            |r| r.get(0),
        )
        .map_err(database_error)?;
    if count != current.revision || count > 4096 {
        return Err(denied("cancel history incomplete or over bounded range"));
    }
    let mut statement = connection
        .prepare("SELECT revision,document FROM events WHERE revision<=?1 ORDER BY revision")
        .map_err(database_error)?;
    let mut rows = statement
        .query([current.revision])
        .map_err(database_error)?;
    let mut heads = Vec::new();
    let mut total = 0usize;
    while let Some(row) = rows.next().map_err(database_error)? {
        let revision: u64 = row.get(0).map_err(database_error)?;
        let bytes: Vec<u8> = row.get(1).map_err(database_error)?;
        total = total
            .checked_add(bytes.len())
            .filter(|v| *v <= 4 * 1024 * 1024)
            .ok_or_else(|| denied("cancel history payload exceeds bound"))?;
        if bytes.len() > 65536 || revision != heads.len() as u64 + 1 {
            return Err(denied("cancel history sequence invalid"));
        }
        let head: Head = decode(&bytes)?;
        validate(&head)?;
        if head.revision != revision || encode(&head)? != bytes {
            return Err(denied("cancel history document differs"));
        }
        heads.push(head);
    }
    if heads.last() != Some(current) {
        return Err(denied("cancel history is not current committed head"));
    }
    Ok(heads)
}
/// Complete independent history review for retained CLOSED root attempts. It
/// is a fact-only prerequisite for generation rollover, never a Start permit.
/// Current scene absence must be checked separately by the lifecycle owner.
pub(super) fn inspect_root_closures(
    data: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    history: &[Head],
) -> io::Result<Vec<super::super::root_retirement::ClosedRootReferences>> {
    inspect_root_closures_inner(data, profile, binding, history, None)
}
/// The caller has already resolved this current generation from its immutable
/// Preparing event. Its directory is a separate, exactly named projection,
/// never a historical close or a source of capture authority.
pub(super) fn inspect_root_closures_with_current(
    data: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    history: &[Head],
    current: &Head,
) -> io::Result<Vec<super::super::root_retirement::ClosedRootReferences>> {
    inspect_root_closures_inner(data, profile, binding, history, Some(current))
}
fn inspect_root_closures_inner(
    data: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    history: &[Head],
    current: Option<&Head>,
) -> io::Result<Vec<super::super::root_retirement::ClosedRootReferences>> {
    let origin = root_binding::original_root(binding)?;
    if history
        .first()
        .is_none_or(|h| h.revision != 1 || h.ticket != origin.id || h.state != State::Preparing)
        || history
            .last()
            .is_none_or(|h| h.ticket != origin.id || h.state != State::RootAbsent)
        || history.iter().any(|h| h.ticket != origin.id)
    {
        return Err(denied(
            "root history has an unclosed or unsupported generation",
        ));
    }
    for pair in history.windows(2) {
        let before = &pair[0];
        let after = &pair[1];
        if before.revision.checked_add(1) != Some(after.revision)
            || !matches!(
                (before.state, after.state),
                (State::Preparing, State::CaptureReserved)
                    | (State::CaptureReserved, State::Captured)
                    | (State::Captured, State::Ready)
                    | (State::Ready, State::StopPrepared)
                    | (State::StopPrepared, State::Stopped)
                    | (State::Stopped, State::RestorePrepared)
                    | (
                        State::Captured | State::Stopped | State::RestorePrepared,
                        State::RootRetirementPrepared
                    )
                    | (State::RootRetirementPrepared, State::RootAbsent)
                    | (State::RootAbsent, State::RootRetirementPrepared)
            )
        {
            return Err(denied(
                "root history phase sequence is not a registered closed lifecycle",
            ));
        }
        if let Some(refs) = &before.references {
            if after.references.as_ref().is_none_or(|next| {
                next.attempt != refs.attempt
                    || next.original_sha256 != refs.original_sha256
                    || next.scope_sha256 != refs.scope_sha256
                    || !next.phases.starts_with(&refs.phases)
                    || refs.live_sha256.is_some() && next.live_sha256 != refs.live_sha256
            }) {
                return Err(denied("root historical phase chain rebases references"));
            }
        }
        if before.root_retirement.is_some() && before.root_retirement != after.root_retirement
            || before.restore_plan.is_some() && before.restore_plan != after.restore_plan
        {
            return Err(denied(
                "root history changes an already registered recovery decision",
            ));
        }
    }
    let mut latest = BTreeMap::<String, &Head>::new();
    let mut immutable = BTreeMap::new();
    for head in history {
        if let Some(refs) = &head.references {
            let key = (refs.original_sha256.clone(), refs.scope_sha256.clone());
            if refs.attempt != head.ticket
                || immutable
                    .insert(refs.attempt.clone(), key.clone())
                    .is_some_and(|old| old != key)
            {
                return Err(denied("closed root history rebases original references"));
            }
            latest.insert(refs.attempt.clone(), head);
        }
    }
    if latest.is_empty() || latest.len() > 512 {
        return Err(denied(
            "closed root history has no bounded indexed attempt set",
        ));
    }
    let mut references = Vec::with_capacity(latest.len());
    for closed in latest.values() {
        if closed.state != State::RootAbsent {
            return Err(denied(
                "root history contains an unclosed or interrupted attempt",
            ));
        }
        let movement = closed
            .root_retirement
            .as_ref()
            .ok_or_else(|| denied("closed root history movement missing"))?;
        let source = history
            .iter()
            .find(|h| h.revision == movement.source_revision)
            .ok_or_else(|| denied("historical source event missing"))?;
        let predecessor = history
            .iter()
            .find(|h| h.revision.checked_add(1) == Some(closed.revision))
            .ok_or_else(|| denied("historical close predecessor missing"))?;
        let initial_plan = history
            .iter()
            .find(|h| source.revision.checked_add(1) == Some(h.revision))
            .ok_or_else(|| denied("historical initial plan registration missing"))?;
        let matches_prepared = |head: &Head| {
            head.state == State::RootRetirementPrepared
                && head.ticket == closed.ticket
                && head.references == closed.references
                && head.restore_plan == closed.restore_plan
                && head.root_retirement == closed.root_retirement
                && head.completion_sha256.is_none()
        };
        if !matches_prepared(predecessor) || !matches_prepared(initial_plan) {
            return Err(denied(
                "historical root close lacks exact registered plan predecessor",
            ));
        }
        // First-generation origin remains independently fixed in the original
        // binding. Later generations must supply their own indexed provenance
        // when that registration is implemented; never reuse this origin.
        if closed.ticket != origin.id {
            return Err(denied("root history generation has no registered origin"));
        }
        let reference = super::super::root_retirement::ClosedRootReferences {
            source: source.clone(),
            closed: (**closed).clone(),
            original_root: origin.clone(),
        };
        super::super::root_retirement::open_closed_history(data.path(), profile, &reference)?;
        references.push(reference);
    }
    let manager = data.child_private("manager-state", false)?;
    let vault = manager.child_private("client-originals", false)?;
    let names = super::super::tree::names(vault.file())?;
    let actual = names
        .iter()
        .map(|n| {
            n.to_str()
                .map(str::to_owned)
                .ok_or_else(|| denied("unknown historical root vault name"))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut expected = latest.keys().cloned().collect::<Vec<_>>();
    let active_directory = if let Some(current) = current {
        if uuid::Uuid::parse_str(&current.ticket)
            .map(|id| id.to_string() != current.ticket)
            .unwrap_or(true)
            || latest.contains_key(&current.ticket)
            || current
                .references
                .as_ref()
                .is_some_and(|r| r.attempt != current.ticket)
        {
            return Err(denied(
                "current root attempt is not distinct indexed generation",
            ));
        }
        let present = observation::entry(&vault, &current.ticket)?.is_some();
        if current.state == State::Preparing && present
            || current.state != State::Preparing
                && current.state != State::CaptureReserved
                && (current.references.is_none() || !present)
        {
            return Err(denied(
                "current root vault does not match registered capture phase",
            ));
        }
        if present {
            let directory = vault.child_private(&current.ticket, false)?;
            // This directory is NOT a closed original. Only project its
            // registered membership and pinned access/identity here. Frames,
            // blobs and scope are admitted separately by verify_capture; a
            // partial reservation cannot be adopted by the create-only writer.
            // Client single-tree budgets do not describe an aggregate vault.
            let info = stat(directory.file())?;
            let observed = (
                version(&info),
                protection(directory.file(), &info)?,
                super::super::tree::attempt_names(directory.file())?,
            );
            expected.push(current.ticket.clone());
            Some((directory, observed))
        } else {
            None
        }
    } else {
        None
    };
    expected.sort();
    if actual != expected {
        return Err(denied(
            "root historical vault differs from exact indexed closed attempts",
        ));
    }
    // Repeat exact retained evidence after the directory projection. No names
    // or current file values are promoted into independent history facts.
    for reference in &references {
        super::super::root_retirement::open_closed_history(data.path(), profile, reference)?;
    }
    if let Some((directory, observed)) = active_directory {
        let info = stat(directory.file())?;
        if (
            version(&info),
            protection(directory.file(), &info)?,
            super::super::tree::attempt_names(directory.file())?,
        ) != observed
        {
            return Err(denied(
                "current root capture directory changed during history review",
            ));
        }
        directory.private()?;
    }
    if super::super::tree::names(vault.file())? != names {
        return Err(denied("root history vault changed during review"));
    }
    vault.private()?;
    manager.recheck()?;
    Ok(references)
}
pub(super) fn inspect(
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    current: &Head,
    history: &[Head],
) -> io::Result<Evidence> {
    if history.last() != Some(current) {
        return Err(denied("vault review missing original indexed history"));
    }
    let mut expected = BTreeMap::<String, &Head>::new();
    let mut cancelled = BTreeMap::<String, Option<TreeSnapshot>>::new();
    let mut originals = BTreeMap::<String, (&str, &str)>::new();
    for head in history.iter().take(history.len().saturating_sub(1)) {
        if head.state == State::CancelledCapture {
            let (attempt, snapshot) = preparation::retained_capture(data, binding, head, history)?
                .ok_or_else(|| denied("cancelled capture binding missing"))?;
            if cancelled.insert(attempt, snapshot).is_some() {
                return Err(denied("duplicate cancelled capture reservation"));
            }
        }
        if let Some(refs) = &head.references {
            if originals
                .insert(
                    refs.attempt.clone(),
                    (&refs.original_sha256, &refs.scope_sha256),
                )
                .is_some_and(|old| old != (&refs.original_sha256, &refs.scope_sha256))
            {
                return Err(denied("history rebases original references"));
            }
            expected.insert(refs.attempt.clone(), head);
        }
    }
    if expected.len() > 512 {
        return Err(denied("retained closed attempt count over bound"));
    }
    // Latest event for EVERY indexed attempt must be a complete, independently
    // verifiable close. A rollback or interrupted attempt cannot disappear into
    // the history just because a later Preparing was written.
    for head in expected.values() {
        match head.state {
            State::Restored => {
                completion::verify_registered(data.path(), client.path(), profile, head)?
            }
            State::Unchanged => {
                unchanged::verify_registered(data.path(), client.path(), profile, head)?
            }
            _ => {
                return Err(denied(
                    "retained attempt has no registered complete closure",
                ))
            }
        }
    }
    if expected
        .keys()
        .any(|attempt| cancelled.contains_key(attempt))
    {
        return Err(denied(
            "attempt is both cancelled capture and admitted original",
        ));
    }
    let current_reserved =
        (current.state == State::CaptureReserved).then(|| current.ticket.clone());
    if current_reserved
        .as_ref()
        .is_some_and(|ticket| expected.contains_key(ticket) || cancelled.contains_key(ticket))
    {
        return Err(denied("reserved capture reuses prior attempt"));
    }
    let manager = data.child_private("manager-state", false)?;
    if observation::entry(&manager, "client-originals")?.is_none() {
        if !expected.is_empty() || cancelled.values().any(Option::is_some) {
            return Err(denied("indexed closed originals disappeared"));
        }
        return Ok(Evidence {
            history_sha256: digest(&encode(&history)?),
            vault: None,
            attempts: Vec::new(),
            reserved_capture: current_reserved.map(|id| (id, None)),
        });
    }
    if expected.is_empty() && cancelled.is_empty() && current_reserved.is_none() {
        return Err(denied("vault exists without closed indexed attempts"));
    }
    let vault = manager.child_private("client-originals", false)?;
    let before = (
        vault.identity().clone(),
        protection(vault.file(), &stat(vault.file())?)?,
    );
    require_private(&before.1, true)?;
    let names = super::super::tree::names(vault.file())?;
    let mut wanted: Vec<_> = expected
        .keys()
        .cloned()
        .chain(
            cancelled
                .iter()
                .filter_map(|(id, s)| s.as_ref().map(|_| id.clone())),
        )
        .collect();
    let reserved_capture = if let Some(ticket) = current_reserved {
        if observation::entry(&vault, &ticket)?.is_some() {
            let directory = vault.child_private(&ticket, false)?;
            let snapshot = TreeSnapshot::read_mode(&directory, false)?;
            wanted.push(ticket.clone());
            Some((ticket, Some(snapshot)))
        } else {
            Some((ticket, None))
        }
    } else {
        None
    };
    wanted.sort();
    for (id, expected) in &cancelled {
        if let Some(expected) = expected {
            let directory = vault.child_private(id, false)?;
            if TreeSnapshot::read_mode(&directory, false)? != *expected {
                return Err(denied("retained cancelled capture changed"));
            }
        }
    }
    let mut actual = names
        .iter()
        .map(|n| {
            n.to_str()
                .map(str::to_owned)
                .ok_or_else(|| denied("unindexed vault entry encoding"))
        })
        .collect::<io::Result<Vec<_>>>()?;
    actual.sort();
    if actual != wanted {
        return Err(denied(
            "vault entries differ from closed indexed attempt set",
        ));
    }
    let mut attempts = Vec::new();
    for (attempt, head) in expected {
        let directory = vault.child_private(&attempt, false)?;
        attempts.push((
            attempt,
            directory.identity().clone(),
            digest(&encode(head)?),
        ));
        directory.recheck()?;
    }
    if super::super::tree::names(vault.file())? != names {
        return Err(denied("vault entries changed during review"));
    }
    vault.private()?;
    vault.recheck()?;
    manager.recheck()?;
    if protection(vault.file(), &stat(vault.file())?)? != before.1 {
        return Err(denied("vault access changed during review"));
    }
    Ok(Evidence {
        history_sha256: digest(&encode(&history)?),
        vault: Some(before),
        attempts,
        reserved_capture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn head(revision: u64) -> Head {
        Head {
            revision,
            ticket: "3483ced2-c257-4cf6-9c52-98dfbfb3614a".into(),
            state: State::Preparing,
            references: None,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
            root_generation: None,
        }
    }
    fn database() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE events(revision INTEGER PRIMARY KEY, document BLOB NOT NULL)",
        )
        .unwrap();
        c
    }
    fn insert(c: &Connection, revision: u64, value: &Head) {
        c.execute(
            "INSERT INTO events VALUES(?1,?2)",
            params![revision, encode(value).unwrap()],
        )
        .unwrap();
    }
    #[test]
    fn bounded_history_is_canonical_contiguous_and_exactly_the_current_snapshot() {
        let c = database();
        insert(&c, 1, &head(1));
        insert(&c, 2, &head(2));
        assert_eq!(read(&c, &head(2)).unwrap(), vec![head(1), head(2)]);
        let mut changed = head(2);
        changed.ticket = uuid::Uuid::new_v4().to_string();
        assert!(read(&c, &changed).is_err());
        let c = database();
        insert(&c, 1, &head(1));
        insert(&c, 3, &head(3));
        assert!(read(&c, &head(3)).is_err());
        let c = database();
        insert(&c, 1, &head(1));
        insert(&c, 2, &head(1));
        assert!(read(&c, &head(2)).is_err());
        let c = database();
        let raw = serde_json::to_vec_pretty(&head(1)).unwrap();
        c.execute("INSERT INTO events VALUES(1,?1)", [raw]).unwrap();
        assert!(read(&c, &head(1)).is_err());
        let c = database();
        c.execute_batch("WITH RECURSIVE n(v) AS (SELECT 1 UNION ALL SELECT v+1 FROM n WHERE v<4097) INSERT INTO events SELECT v, x'7b7d' FROM n").unwrap();
        assert!(read(&c, &head(4097)).is_err());
    }
}
