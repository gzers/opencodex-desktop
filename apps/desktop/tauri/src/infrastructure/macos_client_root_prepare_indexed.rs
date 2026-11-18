//! Read the actual plan ONLY through its independently committed Head. This
//! resolves pending live/moved states but grants no movement, cancellation,
//! normal Stop or successor permission. A receipt file alone has no authority.
use super::*;
use std::ffi::OsString;

pub(in crate::infrastructure::macos_client_originals) fn inspect(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    connection: &Connection,
) -> std::io::Result<PinnedDirectory> {
    let history = vault_history::read(connection, head)?;
    let namespace = root_cancel_namespace::Namespace::from_history(&history)?;
    observe(
        data,
        profile,
        stored,
        head,
        origin,
        &history,
        (false, &namespace),
    )
    .map(|(root, _)| root)
}
/// One independently indexed initial cancellation, checked without reading the
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
        namespace: &root_cancel_namespace::Namespace<'a>,
    ) -> std::io::Result<(Self, PinnedDirectory)> {
        if head.state != State::RootCancellationAbsent || history.last() != Some(head) {
            return Err(denied(
                "cancellation history is not the independently closed prefix",
            ));
        }
        let (root, fingerprint) = observe(
            data,
            profile,
            binding,
            head,
            origin,
            history,
            (true, namespace),
        )?;
        let reference = head
            .root_retirement
            .as_ref()
            .ok_or_else(|| denied("cancelled capture plan reference missing"))?;
        let slot = data
            .child_private("manager-state", false)?
            .child_private("client-root-cancel-plans", false)?
            .child_private(&reference.plan.id, false)?;
        let raw = capture_file(&slot, "plan.json", MAX_FRAME)?;
        if raw.bytes().map(digest).as_deref() != Some(reference.plan.sha256.as_str()) {
            return Err(denied(
                "cancelled capture plan changed after full observation",
            ));
        }
        let plan: Plan = decode(
            raw.bytes()
                .ok_or_else(|| denied("cancelled capture plan missing"))?,
        )?;
        let retained_slot = plan.capture_vault.root_retained_slot().map(str::to_owned);
        Ok((
            Self {
                binding: binding.to_vec(),
                head: head.clone(),
                origin: origin.clone(),
                history: history.to_vec(),
                fingerprint,
                retained_slot,
                namespace: namespace.clone(),
            },
            root,
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
        let (root, fingerprint) = observe(
            data,
            profile,
            &self.binding,
            &self.head,
            &self.origin,
            &self.history,
            (true, &self.namespace),
        )?;
        if fingerprint != self.fingerprint {
            return Err(denied(
                "historical cancelled-root evidence changed during full lineage review",
            ));
        }
        Ok(root)
    }
}
fn observe(
    data: &PinnedDirectory,
    profile: &Path,
    stored: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    history: &[Head],
    mode: (bool, &root_cancel_namespace::Namespace<'_>),
) -> std::io::Result<(PinnedDirectory, String)> {
    let (historical, namespace) = mode;
    validate(head)?;
    if !matches!(
        head.state,
        State::RootCancellationPrepared | State::RootCancellationAbsent
    ) || head.references.is_some()
        || head.restore_plan.is_some()
        || (head.state == State::RootCancellationPrepared && head.completion_sha256.is_some())
        || head.root_generation.is_some()
        || origin.id != head.ticket
        || origin.generation_slot.is_some()
    {
        return Err(denied(
            "indexed root cancellation is not an initial pending plan",
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
    if plan.schema_version != 2
        || plan.id != reference.plan.id
        || plan.source.revision != reference.source_revision
        || plan.source.ticket != head.ticket
        || !matches!(plan.source.state, State::Preparing | State::CaptureReserved)
        || plan.source.references.is_some()
        || plan.source.restore_plan.is_some()
        || plan.source.root_retirement.is_some()
        || plan.source.completion_sha256.is_some()
        || plan.source.root_generation.is_some()
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
    plan.capture_vault
        .verify_root_retained(data, &plan.source, source_history, historical)?;
    if !historical {
        let profile_pin = PinnedDirectory::open(profile)?;
        let root_path = if observation::entry(&profile_pin, ".codex")?.is_some() {
            profile_pin.path().join(".codex")
        } else {
            directory.path().join("root")
        };
        let client = PinnedDirectory::open(&root_path)?;
        if vault_history::inspect(data, &client, profile, stored, &plan.source, source_history)?
            != plan.capture_vault
        {
            return Err(denied("indexed partial capture or vault namespace changed"));
        }
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
    let tree = super::super::super::TreeSnapshot::read(&root)?;
    if tree != plan.root && tree != plan.root.without_config_guard(&plan.writer)? {
        return Err(denied("root-cancel exact indexed tree changed"));
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
        if tree != plan.root.without_config_guard(&plan.writer)? {
            return Err(denied("root-cancel preservation receipt has a live writer"));
        }
        expected.push(OsString::from("preserved-readback.json"));
    }
    let completed =
        completion::verify_receipt(&directory, &plan, head, &root, &reference.plan.sha256)?;
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
    plan.capture_vault
        .verify_root_retained(data, &plan.source, source_history, historical)?;
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
        &plan_image,
        &intent,
        &receipt,
        &completed,
        &tree,
    ))?);
    Ok((root, fingerprint))
}
/// Pending first-root movement already has an independently indexed plan.
/// Its retained capture is checked as DATA, never as admitted Originals.
pub(in crate::infrastructure::macos_client_originals::session_index) fn pending_retained(
    data: &PinnedDirectory,
    profile: &Path,
    binding: &[u8],
    head: &Head,
    origin: &super::super::super::root_creation::References,
    history: &[Head],
    namespace: &root_cancel_namespace::Namespace<'_>,
) -> std::io::Result<Option<String>> {
    if head.state != State::RootCancellationPrepared {
        return Err(denied("pending retained capture phase differs"));
    }
    observe(
        data,
        profile,
        binding,
        head,
        origin,
        history,
        (false, namespace),
    )?;
    let reference = head
        .root_retirement
        .as_ref()
        .ok_or_else(|| denied("pending retained capture plan missing"))?;
    let slot = data
        .child_private("manager-state", false)?
        .child_private("client-root-cancel-plans", false)?
        .child_private(&reference.plan.id, false)?;
    let image = capture_file(&slot, "plan.json", MAX_FRAME)?;
    if image.bytes().map(digest).as_deref() != Some(reference.plan.sha256.as_str()) {
        return Err(denied("pending retained capture plan changed"));
    }
    let plan: Plan = decode(image.bytes().unwrap())?;
    Ok(plan.capture_vault.root_retained_slot().map(str::to_owned))
}
