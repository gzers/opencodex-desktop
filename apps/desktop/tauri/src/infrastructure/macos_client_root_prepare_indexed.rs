//! Read the actual plan ONLY through its independently committed Head. This
//! resolves pending live/moved states but grants no movement, cancellation,
//! normal Stop or successor permission. A receipt file alone has no authority.
use super::*;
use std::{ffi::OsString, os::unix::ffi::OsStrExt};

pub(in crate::infrastructure::macos_client_originals) fn inspect(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
) -> std::io::Result<PinnedDirectory> {
    inspect_plan(data, profile, stored, head, origin, connection)
        .map(|observation| observation.root)
}

pub(super) struct Observation {
    pub(super) root: PinnedDirectory,
    pub(super) directory: PinnedDirectory,
    pub(super) plan: Plan,
    pub(super) preserved: Image,
    pub(super) plan_image: Image,
    pub(super) completed: Image,
    pub(super) fingerprint: String,
}

enum RootView<'a, 'g> {
    Current,
    Historical,
    Guarded(&'a NativeLease, &'a mut Guard<'g>),
}

pub(super) fn inspect_plan(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
) -> std::io::Result<Observation> {
    let history = vault_history::read(connection, head)?;
    let namespace = root_cancel_namespace::Namespace::from_history(&history)?;
    observe(
        data,
        profile,
        stored,
        head,
        origin,
        &history,
        (&namespace, RootView::Current),
    )
}

/// Reopen the pending movement with this action's live writer and audit the
/// entire lineage through the caller's original locked index connection.
pub(super) fn inspect_plan_guarded(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
    native: &NativeLease,
    guard: &mut Guard<'_>,
) -> std::io::Result<Observation> {
    let observation = inspect_pending(
        data,
        profile,
        stored,
        head,
        origin,
        connection,
        RootView::Guarded(native, guard),
    )?;
    if observation.plan.reacquisition.is_none() {
        return Err(denied(
            "storage-only root cancellation has no continuation authority",
        ));
    }
    guard
        .root_image(native, observation.root.path())
        .map_err(io)?;
    Ok(observation)
}

/// Read-only confirmation for the exact physical root pinned by the preview.
/// A validated sidecar is still unregistered while the Head is pending.
pub(in crate::infrastructure::macos_client_originals) fn pending_confirmation(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
    expected_physical: &PinnedDirectory,
) -> std::io::Result<(Option<String>, bool)> {
    expected_physical.recheck()?;
    let observation = inspect_pending(
        data,
        profile,
        stored,
        head,
        origin,
        connection,
        RootView::Current,
    )?;
    expected_physical.recheck()?;
    if observation.root.path() != expected_physical.path()
        || observation.root.coordination_expectation()
            != expected_physical.coordination_expectation()
    {
        return Err(denied("pending root cancellation differs from preview pin"));
    }
    Ok((
        observation
            .plan
            .reacquisition
            .is_some()
            .then_some(observation.fingerprint),
        observation.completed != Image::Absent,
    ))
}

fn inspect_pending(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
    view: RootView<'_, '_>,
) -> std::io::Result<Observation> {
    let confirm_index = || -> std::io::Result<()> {
        let binding: Vec<u8> = connection
            .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                r.get(0)
            })
            .map_err(database_error)?;
        if binding != stored || read_head(connection)?.as_ref() != Some(head) {
            return Err(denied("pending root cancellation index changed"));
        }
        Ok(())
    };
    if head.state != State::RootCancellationPrepared {
        return Err(denied("indexed root cancellation Head is not pending"));
    }
    confirm_index()?;
    let history = vault_history::read(connection, head)?;
    let lineage = root_lineage::parse(stored, &history)?;
    if lineage.current().origin != *origin {
        return Err(denied("pending root cancellation origin changed"));
    }
    let manager = data.child_private("manager-state", false)?;
    let slots = lineage.slots();
    root_creation::verify_generation_slots(&manager, &slots)?;
    let mut observed = None;
    let mut view = Some(view);
    vault_history::inspect_root_history_with_pending(
        data,
        profile,
        stored,
        &history,
        &mut |pending, origin, prefix, namespace, closures_sha256| {
            if pending != head || observed.is_some() {
                return Err(denied(
                    "pending root cancellation differs from current Head",
                ));
            }
            let observation = observe(
                data,
                profile,
                stored,
                pending,
                origin,
                prefix,
                (
                    namespace,
                    view.take()
                        .ok_or_else(|| denied("pending cancellation observed twice"))?,
                ),
            )?;
            require_closures(&observation.plan, closures_sha256)?;
            let retained = retained_slot(&observation.plan);
            observed = Some(observation);
            Ok(retained)
        },
    )?;
    root_creation::verify_generation_slots(&manager, &slots)?;
    if vault_history::read(connection, head)? != history {
        return Err(denied("pending root cancellation events changed"));
    }
    confirm_index()?;
    observed.ok_or_else(|| denied("indexed pending root cancellation is missing"))
}

fn require_closures(plan: &Plan, expected: &str) -> std::io::Result<()> {
    if plan
        .successor
        .as_ref()
        .is_some_and(|s| s.closures_sha256 != expected)
    {
        return Err(denied("indexed successor ordinary closure digest differs"));
    }
    Ok(())
}

fn retained_slot(plan: &Plan) -> Option<String> {
    plan.capture_vault
        .as_ref()
        .and_then(vault_history::Evidence::root_retained_slot)
        .map(str::to_owned)
}
/// One independently indexed cancellation, checked without reading the
/// live client/Grok of a later generation. It grants no current permission.
pub(in crate::infrastructure::macos_client_originals) struct HistoricalObservation<'a> {
    binding: Vec<u8>,
    head: Head,
    origin: super::super::super::root_creation::References,
    history: Vec<Head>,
    fingerprint: String,
    retained_slot: Option<String>,
    namespace: root_cancel_namespace::Namespace<'a>,
}
impl<'a> HistoricalObservation<'a> {
    pub(in crate::infrastructure::macos_client_originals::session_index) fn begin(
        data: &PinnedDirectory,
        profile: &Path,
        binding: &[u8],
        head: &Head,
        origin: &super::super::super::root_creation::References,
        history: &[Head],
        (namespace, closures_sha256): (&root_cancel_namespace::Namespace<'a>, &str),
    ) -> std::io::Result<(Self, PinnedDirectory)> {
        if head.state != State::RootCancellationAbsent || history.last() != Some(head) {
            return Err(denied(
                "cancellation history is not the independently closed prefix",
            ));
        }
        let observation = observe(
            data,
            profile,
            binding,
            head,
            origin,
            history,
            (namespace, RootView::Historical),
        )?;
        require_closures(&observation.plan, closures_sha256)?;
        let retained_slot = retained_slot(&observation.plan);
        Ok((
            Self {
                binding: binding.to_vec(),
                head: head.clone(),
                origin: origin.clone(),
                history: history.to_vec(),
                fingerprint: observation.fingerprint,
                retained_slot,
                namespace: namespace.clone(),
            },
            observation.root,
        ))
    }
    pub(in crate::infrastructure::macos_client_originals) fn retained_slot(&self) -> Option<&str> {
        self.retained_slot.as_deref()
    }
    pub(in crate::infrastructure::macos_client_originals) fn finish(
        self,
        data: &PinnedDirectory,
        profile: &Path,
    ) -> std::io::Result<PinnedDirectory> {
        let observation = observe(
            data,
            profile,
            &self.binding,
            &self.head,
            &self.origin,
            &self.history,
            (&self.namespace, RootView::Historical),
        )?;
        if observation.fingerprint != self.fingerprint {
            return Err(denied(
                "historical cancelled-root evidence changed during full lineage review",
            ));
        }
        Ok(observation.root)
    }
}
fn observe(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    history: &[Head],
    (namespace, mut view): (&root_cancel_namespace::Namespace<'_>, RootView<'_, '_>),
) -> std::io::Result<Observation> {
    let historical = matches!(view, RootView::Historical);
    validate(head)?;
    if !matches!(
        head.state,
        State::RootCancellationPrepared | State::RootCancellationAbsent
    ) || head.references.is_some()
        || head.restore_plan.is_some()
        || (head.state == State::RootCancellationPrepared && head.completion_sha256.is_some())
        || head.root_generation.is_some()
        || origin.id != head.ticket
    {
        return Err(denied(
            "indexed root cancellation does not match its registered phase",
        ));
    }
    let reference = head
        .root_retirement
        .as_ref()
        .ok_or_else(|| denied("root-cancel movement reference missing"))?;
    let distance = if head.state == State::RootCancellationAbsent {
        2
    } else {
        1
    };
    if reference.source_revision.checked_add(distance) != Some(head.revision) {
        return Err(denied(
            "root-cancel source is not the immediately preceding event",
        ));
    }
    let manager = data.child_private("manager-state", false)?;
    let parent = manager.child_private("client-root-cancel-plans", false)?;
    namespace.require(&parent)?;
    if !namespace.contains(&reference.plan.id) {
        return Err(denied(
            "root-cancel namespace differs from its independent reference",
        ));
    }
    let directory = parent.child_private(&reference.plan.id, false)?;
    let plan_image = capture_file(&directory, "plan.json", MAX_FRAME)?;
    let plan: Plan = match &plan_image {
        Image::File {
            bytes, protection, ..
        } => {
            require_private(protection, false)?;
            if digest(bytes) != reference.plan.sha256 {
                return Err(denied("indexed root-cancel plan digest changed"));
            }
            let plan: Plan = decode(bytes)?;
            if encode(&plan)?.as_slice() != bytes.as_slice() {
                return Err(denied("indexed root-cancel plan encoding changed"));
            }
            plan
        }
        _ => return Err(denied("indexed root-cancel plan missing")),
    };
    let successor = origin.generation_slot.is_some();
    if !matches!(
        (
            plan.schema_version,
            successor,
            &plan.capture_vault,
            &plan.successor
        ),
        (2, false, Some(_), None) | (3, true, None, Some(_))
    ) || plan.id != reference.plan.id
        || plan.source.revision != reference.source_revision
        || plan.source.ticket != head.ticket
        || !plan.source.can_source_root_cancellation(successor)
        || plan.source.references.is_some()
        || plan.source.restore_plan.is_some()
        || plan.source.root_retirement.is_some()
        || plan.source.completion_sha256.is_some()
        || plan.origin != *origin
        || plan.binding_sha256 != digest(stored)
        || !valid_hash(&plan.initial_review_sha256)
        || !historical
            && super::super::super::GrokSnapshot::read(&PinnedDirectory::open(profile)?)?
                != plan.grok
    {
        return Err(denied("indexed root-cancel source/history/context changed"));
    }
    let source_end = usize::try_from(plan.source.revision)
        .map_err(|_| denied("cancelled-root source revision overflow"))?;
    let source_history = history
        .get(..source_end)
        .ok_or_else(|| denied("cancelled-root original prefix missing"))?;
    if source_history.last() != Some(&plan.source)
        || digest(&encode(&source_history)?) != plan.history_sha256
    {
        return Err(denied("indexed root-cancel source event/history changed"));
    }
    let lineage = root_lineage::parse(stored, source_history)?;
    let generation = lineage.current();
    if generation.origin != *origin {
        return Err(denied(
            "indexed cancellation source belongs to another generation",
        ));
    }
    let released = plan.root.without_config_guard(&plan.writer)?;
    if let Some(source) = &plan.successor {
        if !source.empty_root.is_empty_root() || !valid_hash(&source.closures_sha256) {
            return Err(denied("indexed successor review evidence is invalid"));
        }
        source.require_baseline(&released)?;
    }
    let profile = PinnedDirectory::open(profile)?;
    let logical = if historical {
        None
    } else {
        observation::entry(&profile, ".codex")?
    };
    let archived = observation::entry(&directory, "root")?;
    let intent = capture_file(&directory, "move-intent.json", MAX_FRAME)?;
    let has_intent = match &intent {
        Image::Absent => false,
        Image::File {
            bytes, protection, ..
        } => {
            require_private(protection, false)?;
            if bytes.as_slice()
                != encode(&(reference.plan.sha256.as_str(), &plan.root.root))?.as_slice()
            {
                return Err(denied("root-cancel move intent changed"));
            }
            true
        }
    };
    let path = match (logical, archived) {
        (Some(_), None) => profile.path().join(".codex"),
        (None, Some(_)) if has_intent => directory.path().join("root"),
        _ => {
            return Err(denied(
                "root-cancel live/archive location is absent, ambiguous or unregistered",
            ))
        }
    };
    let root = PinnedDirectory::open(&path)?;
    super::super::super::root_creation::verify_indexed_anchor(
        &manager,
        data,
        profile.path(),
        &root,
        origin,
        &path,
    )?;
    vault_history::check_historical_scope_pins(data, &profile, &root, &generation.binding)?;
    if let Some(capture) = &plan.capture_vault {
        capture.verify_root_retained(data, &plan.source, source_history, historical)?;
        if !historical
            && vault_history::inspect(
                data,
                &root,
                profile.path(),
                stored,
                &plan.source,
                source_history,
            )? != *capture
        {
            return Err(denied("indexed partial capture or vault namespace changed"));
        }
    }
    let tree = super::super::super::TreeSnapshot::read(&root)?;
    let guarded = matches!(view, RootView::Guarded(..));
    match &mut view {
        RootView::Guarded(native, guard) => {
            let fresh = guard.root_image(native, &path).map_err(io)?;
            require_guard_projection(&tree, &fresh, &released)?;
        }
        _ if tree == plan.root || tree == released => (),
        _ => return Err(denied("root-cancel exact indexed tree changed")),
    }
    // Explicit namespace, not a scan to discover a plan or invent a source.
    let mut expected = vec![OsString::from("plan.json")];
    if has_intent {
        expected.push(OsString::from("move-intent.json"));
    }
    if archived.is_some() {
        expected.push(OsString::from("root"));
    }
    let receipt = capture_file(&directory, "preserved-readback.json", MAX_FRAME)?;
    if receipt != Image::Absent {
        // A stored preservation receipt is only DATA. The index stays pending.
        let expected_receipt = encode(&serde_json::json!({
            "schema":"macos-root-preparation-preserved-v1", "planSha256":reference.plan.sha256,
            "rootIdentity":root.identity(),"logicalRootAbsent":true,"writerReleased":true,
            "canonicalCancellationClosed":false,"normalStop":false,"successorEligible":false,
        }))?;
        match &receipt {
            Image::File {
                bytes, protection, ..
            } if has_intent
                && archived.is_some()
                && bytes.as_slice() == expected_receipt.as_slice() =>
            {
                require_private(protection, false)?;
            }
            _ => return Err(denied("root-cancel preservation receipt differs")),
        }
        if !guarded && tree != released {
            return Err(denied("root-cancel preservation receipt has a live writer"));
        }
        expected.push(OsString::from("preserved-readback.json"));
    }
    let completed = if guarded {
        completion::verify_receipt_projected(
            &directory,
            &plan,
            head,
            &root,
            &reference.plan.sha256,
            &released,
        )?
    } else {
        completion::verify_receipt(&directory, &plan, head, &root, &reference.plan.sha256)?
    };
    if completed != Image::Absent {
        if logical.is_some() || archived.is_none() || !has_intent || receipt == Image::Absent {
            return Err(denied(
                "root-cancel completion requires original logical absence",
            ));
        }
        expected.push(OsString::from("cancellation-completed.json"));
    }
    expected.sort();
    if super::super::super::tree::names(directory.file())? != expected
        || super::super::super::TreeSnapshot::read(&root)? != tree
        || !historical && super::super::super::GrokSnapshot::read(&profile)? != plan.grok
        || capture_file(&directory, "plan.json", MAX_FRAME)? != plan_image
        || capture_file(&directory, "move-intent.json", MAX_FRAME)? != intent
        || capture_file(&directory, "preserved-readback.json", MAX_FRAME)? != receipt
        || capture_file(&directory, "cancellation-completed.json", MAX_FRAME)? != completed
        || !historical
            && observation::entry(&profile, ".codex")?.map(|s| identity(&s))
                != logical.map(|s| identity(&s))
        || observation::entry(&directory, "root")?.map(|s| identity(&s))
            != archived.map(|s| identity(&s))
    {
        return Err(denied("indexed root-cancel readback changed"));
    }
    directory.private()?;
    if let Some(capture) = &plan.capture_vault {
        capture.verify_root_retained(data, &plan.source, source_history, historical)?;
    }
    vault_history::check_historical_scope_pins(data, &profile, &root, &generation.binding)?;
    super::super::super::root_creation::verify_indexed_anchor(
        &manager,
        data,
        profile.path(),
        &root,
        origin,
        &path,
    )?;
    namespace.require(&parent)?;
    let fingerprint = digest(&encode(&(
        head,
        root.path().as_os_str().as_bytes(),
        directory.identity(),
        &plan_image,
        &intent,
        &receipt,
        &completed,
        &released,
    ))?);
    Ok(Observation {
        root,
        directory,
        plan,
        preserved: receipt,
        plan_image,
        completed,
        fingerprint,
    })
}
/// Pending root movement already has an independently indexed plan.
/// Its retained capture is checked as DATA, never as admitted Originals.
pub(in crate::infrastructure::macos_client_originals::session_index) fn pending_retained(
    data: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    history: &[Head],
    (namespace, closures_sha256): (&root_cancel_namespace::Namespace<'_>, &str),
) -> std::io::Result<Option<String>> {
    if head.state != State::RootCancellationPrepared {
        return Err(denied("pending retained capture phase differs"));
    }
    let observation = observe(
        data,
        profile,
        binding,
        head,
        origin,
        history,
        (namespace, RootView::Current),
    )?;
    require_closures(&observation.plan, closures_sha256)?;
    Ok(retained_slot(&observation.plan))
}

fn require_guard_projection(
    tree: &TreeSnapshot,
    fresh: &Image,
    released: &TreeSnapshot,
) -> std::io::Result<()> {
    if tree.without_config_guard(fresh)? != *released {
        return Err(denied(
            "guarded root differs from the indexed released tree",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::DirBuilderExt};

    #[test]
    fn guarded_projection_removes_only_the_exact_current_writer() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap().join("root");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let root = PinnedDirectory::open(&path).unwrap();
        let marker = "config.toml.ocx-write.lock";
        let content = b"writer marker";
        create(&root, "config.toml", b"user config").unwrap();
        let released = TreeSnapshot::read(&root).unwrap();

        create(&root, marker, content).unwrap();
        let first = capture_file(&root, marker, MAX_FRAME).unwrap();
        let guarded = TreeSnapshot::read(&root).unwrap();
        require_guard_projection(&guarded, &first, &released).unwrap();

        // Keep the original inode alive so a same-byte replacement has a
        // distinct identity even on a filesystem that reuses freed inodes.
        let original = open_at(root.file(), std::ffi::OsStr::new(marker), false).unwrap();
        fs::remove_file(root.path().join(marker)).unwrap();
        create(&root, marker, content).unwrap();
        let second = capture_file(&root, marker, MAX_FRAME).unwrap();
        let guarded = TreeSnapshot::read(&root).unwrap();
        assert!(require_guard_projection(&guarded, &first, &released).is_err());
        require_guard_projection(&guarded, &second, &released).unwrap();
        drop(original);

        create(&root, "unexpected", b"other tree change").unwrap();
        let drifted = TreeSnapshot::read(&root).unwrap();
        assert!(require_guard_projection(&drifted, &second, &released).is_err());
    }
}
