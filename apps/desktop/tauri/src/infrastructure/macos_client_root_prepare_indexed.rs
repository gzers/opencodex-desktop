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
    validate(head)?;
    if head.state != State::RootCancellationPrepared
        || head.references.is_some()
        || head.restore_plan.is_some()
        || head.completion_sha256.is_some()
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
    if reference.source_revision.checked_add(1) != Some(head.revision) {
        return Err(denied(
            "root-cancel source is not the immediately preceding event",
        ));
    }
    let manager = data.child_private("manager-state", false)?;
    if observation::entry(&manager, "client-originals")?.is_some() {
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
        || digest(&encode(&vault_history::read(connection, &plan.source)?)?) != plan.history_sha256
        || super::super::super::GrokSnapshot::read(&PinnedDirectory::open(profile)?)? != plan.grok
    {
        return Err(denied("indexed root-cancel source/history/context changed"));
    }
    let raw_source: Vec<u8> = connection
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [reference.source_revision],
            |r| r.get(0),
        )
        .map_err(database_error)?;
    if raw_source != encode(&plan.source)? {
        return Err(denied("indexed root-cancel source event changed"));
    }
    let profile = PinnedDirectory::open(profile)?;
    let logical = observation::entry(&profile, ".codex")?;
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
    expected.sort();
    if super::super::super::tree::names(directory.file())? != expected
        || super::super::super::TreeSnapshot::read(&root)? != tree
        || super::super::super::GrokSnapshot::read(&profile)? != plan.grok
        || capture_file(&directory, "plan.json", MAX_FRAME)? != plan_image
        || capture_file(&directory, "move-intent.json", MAX_FRAME)? != intent
        || capture_file(&directory, "preserved-readback.json", MAX_FRAME)? != receipt
        || super::super::super::tree::names(parent.file())? != names
        || observation::entry(&profile, ".codex")?.map(|s| identity(&s))
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
    Ok(root)
}
