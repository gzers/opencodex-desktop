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
    observe(data, profile, stored, head, origin, &history, false).map(|(root, _)| root)
}
/// One independently indexed initial cancellation, checked without reading the
/// live client/Grok of a later generation. It grants no current permission.
pub(in crate::infrastructure::macos_client_originals) struct HistoricalObservation {
    binding: Vec<u8>,
    head: Head,
    origin: super::super::super::root_creation::References,
    history: Vec<Head>,
    fingerprint: String,
}
impl HistoricalObservation {
    pub(in crate::infrastructure::macos_client_originals) fn begin(
        data: &PinnedDirectory,
        profile: &Path,
        binding: &[u8],
        head: &Head,
        origin: &super::super::super::root_creation::References,
        history: &[Head],
    ) -> std::io::Result<(Self, PinnedDirectory)> {
        if head.state != State::RootCancellationAbsent || history.last() != Some(head) {
            return Err(denied(
                "cancellation history is not the independently closed prefix",
            ));
        }
        let (root, fingerprint) = observe(data, profile, binding, head, origin, history, true)?;
        Ok((
            Self {
                binding: binding.to_vec(),
                head: head.clone(),
                origin: origin.clone(),
                history: history.to_vec(),
                fingerprint,
            },
            root,
        ))
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
            true,
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
    historical: bool,
) -> std::io::Result<(PinnedDirectory, String)> {
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
    if observation::entry(&manager, "client-originals")?.is_some()
        && (!historical
            || observation::entry(
                &manager.child_private("client-originals", false)?,
                &head.ticket,
            )?
            .is_some())
    {
        return Err(denied(
            "root-cancel cannot resolve an uncaptured reference over a vault",
        ));
    }
    let parent = manager.child_private("client-root-cancel-plans", false)?;
    let names = super::super::super::tree::names(parent.file())?;
    if names != vec![OsString::from(&reference.plan.id)] {
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
    if plan.schema_version != 1
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
        || super::super::super::tree::names(parent.file())? != names
        || !historical
            && observation::entry(&profile, ".codex")?.map(|s| identity(&s))
                != logical.map(|s| identity(&s))
        || observation::entry(&directory, "root")?.map(|s| identity(&s))
            != archived.map(|s| identity(&s))
    {
        return Err(denied("indexed root-cancel readback changed"));
    }
    directory.private()?;
    parent.private()?;
    super::super::super::root_creation::verify_indexed_anchor(
        &manager,
        data,
        profile.path(),
        &root,
        origin,
        &path,
    )?;
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
