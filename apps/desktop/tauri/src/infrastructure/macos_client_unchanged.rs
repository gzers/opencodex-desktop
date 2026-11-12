//! Metadata-only closure of an early interrupted session whose COMPLETE
//! original clients are already intact. No plan, artifact replay or historical
//! normal-exit proof is synthesized. Unknown/incomplete evidence stays pending.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    state::ProcessContext,
};
use std::time::Instant;
fn receipt_name(head: &Head) -> io::Result<String> {
    let revision = if head.state == State::Unchanged {
        head.revision
            .checked_sub(1)
            .ok_or_else(|| denied("early terminal revision invalid"))?
    } else {
        head.revision
    };
    if revision == 0 {
        return Err(denied("early receipt revision invalid"));
    }
    Ok(format!("unchanged-readback-{revision}.json"))
}
const LOCKS: [&str; 2] = [
    ".opencodex-native-main.claim.sqlite",
    ".opencodex-native-main.owner.sqlite",
];
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("early recovery budget expired"))
    } else {
        Ok(())
    }
}
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_unchanged_refused".into(),
        detail: "原件或停止状态尚未核实；未结束会话，原件与现场保留".into(),
    }
}
fn eligible(head: &Head) -> bool {
    matches!(
        head.state,
        State::Captured | State::Ready | State::StopPrepared | State::Stopped | State::Failed
    ) && head.references.is_some()
        && head.restore_plan.is_none()
        && head.completion_sha256.is_none()
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    original_head: Head,
    codex_tree_sha256: String,
    grok_tree_sha256: String,
    stable_locks: [Option<Identity>; 2],
}
struct Review {
    codex: TreeSnapshot,
    grok: GrokSnapshot,
    stable: [Option<Identity>; 2],
    fingerprint: String,
}
pub struct Proof {
    head: Head,
    data: Identity,
    client: Identity,
    sha256: String,
    deadline: Instant,
}
impl Proof {
    pub(super) fn require(
        &self,
        head: &Head,
        data: &Identity,
        client: &Identity,
    ) -> io::Result<()> {
        time(self.deadline)?;
        if self.head != *head || self.data != *data || self.client != *client || !eligible(head) {
            return Err(denied("early proof belongs to another head/root"));
        }
        Ok(())
    }
    pub(super) fn sha256(&self) -> &str {
        &self.sha256
    }
}
fn originals(context: &ProcessContext, data: &Path, head: &Head) -> io::Result<Originals> {
    let refs = head
        .references
        .as_ref()
        .ok_or_else(|| denied("early originals absent"))?;
    let client = std::fs::canonicalize(context.working_directory.join(".codex"))?;
    let originals = Originals::open_for_profile(
        data,
        &client,
        &context.working_directory,
        &refs.attempt,
        &refs.original_sha256,
        &refs.phases,
    )?;
    session::verify_index_scope(&originals, refs, &context.opencodex_home)
        .map_err(|_| denied("early scope/live unconfirmed"))?;
    Ok(originals)
}
fn review_objects(originals: &Originals, head: &Head) -> io::Result<Option<Review>> {
    if !eligible(head) {
        return Ok(None);
    }
    let original = originals.frame(Phase::Original)?.0;
    let latest = if let Some((phase, _)) = head.references.as_ref().unwrap().phases.last() {
        originals.frame(*phase)?.0
    } else {
        originals.frame(Phase::Original)?.0
    };
    let codex = TreeSnapshot::read(&originals.client)?;
    let grok = GrokSnapshot::read(&originals.profile)?;
    if !original.tree.restored_matches(&codex, true) || !original.grok.restored_matches(&grok) {
        return Ok(None);
    }
    let stable = [
        codex.optional_stable_identity(LOCKS[0])?,
        codex.optional_stable_identity(LOCKS[1])?,
    ];
    for (name, id) in LOCKS.iter().zip(&stable) {
        if latest.tree.optional_stable_identity(name)? != *id {
            return Ok(None);
        }
        if let Some(id) = id {
            if !original.tree.original_stable_identity_matches(name, id) {
                return Ok(None);
            }
        }
    }
    let fingerprint = digest(&encode(&(
        head,
        originals.data.identity(),
        originals.client.identity(),
        originals.profile.identity(),
        digest(&encode(&codex)?),
        digest(&encode(&grok)?),
        &stable,
    ))?);
    Ok(Some(Review {
        codex,
        grok,
        stable,
        fingerprint,
    }))
}
pub fn review(context: &ProcessContext, data: &Path) -> AppResult<Option<Request>> {
    observation::inspect_map(context, data, |head| {
        let Some(head) = head.filter(|h| eligible(h)) else {
            return Ok(None);
        };
        let originals = originals(context, data, head)?;
        Ok(review_objects(&originals, head)?.map(|r| Request {
            attempt_id: head.references.as_ref().unwrap().attempt.clone(),
            revision: head.revision,
            confirmation_digest: r.fingerprint,
        }))
    })
    .map_err(fail)
}
fn readback(originals: &Originals, head: &Head, deadline: Instant) -> io::Result<Proof> {
    time(deadline)?;
    let before = review_objects(originals, head)?
        .ok_or_else(|| denied("early clients differ from originals"))?;
    completion::prove_optional_released(&originals.client, &before.stable, deadline)?;
    let after = review_objects(originals, head)?
        .ok_or_else(|| denied("early clients changed during native readback"))?;
    if before.codex != after.codex || before.grok != after.grok || before.stable != after.stable {
        return Err(denied("early readback changed"));
    }
    let receipt = Receipt {
        schema_version: 1,
        original_head: head.clone(),
        codex_tree_sha256: digest(&encode(&after.codex)?),
        grok_tree_sha256: digest(&encode(&after.grok)?),
        stable_locks: after.stable,
    };
    let raw = encode(&receipt)?;
    let receipt_name = receipt_name(head)?;
    time(deadline)?;
    match capture_file(&originals.directory, &receipt_name, MAX_FRAME)? {
        Image::Absent => create(&originals.directory, &receipt_name, &raw)?,
        Image::File {
            bytes, protection, ..
        } => {
            require_private(&protection, false)?;
            if bytes != raw {
                return Err(denied("existing early receipt differs; not overwritten"));
            }
        }
    }
    let Image::File {
        bytes, protection, ..
    } = capture_file(&originals.directory, &receipt_name, MAX_FRAME)?
    else {
        return Err(denied("early receipt disappeared"));
    };
    require_private(&protection, false)?;
    if bytes != raw {
        return Err(denied("early receipt readback differs"));
    }
    time(deadline)?;
    Ok(Proof {
        head: head.clone(),
        data: originals.data.identity().clone(),
        client: originals.client.identity().clone(),
        sha256: digest(&raw),
        deadline,
    })
}
pub(super) fn verify_registered(
    data: &Path,
    client: &Path,
    profile: &Path,
    head: &Head,
) -> io::Result<()> {
    if head.state != State::Unchanged || head.restore_plan.is_some() {
        return Err(denied("not an unchanged terminal"));
    }
    let refs = head
        .references
        .as_ref()
        .ok_or_else(|| denied("unchanged references absent"))?;
    let originals = Originals::open_for_profile(
        data,
        client,
        profile,
        &refs.attempt,
        &refs.original_sha256,
        &refs.phases,
    )?;
    let receipt_name = receipt_name(head)?;
    let Image::File {
        bytes, protection, ..
    } = capture_file(&originals.directory, &receipt_name, MAX_FRAME)?
    else {
        return Err(denied("registered early receipt absent"));
    };
    require_private(&protection, false)?;
    if head.completion_sha256.as_deref() != Some(digest(&bytes).as_str()) {
        return Err(denied("registered early receipt changed"));
    }
    let receipt: Receipt = serde_json::from_value(
        parse_strict_json_v2(&bytes).map_err(|_| denied("early receipt JSON invalid"))?,
    )
    .map_err(|_| denied("early receipt fields invalid"))?;
    validate(&receipt.original_head)?;
    if receipt.schema_version != 1
        || !eligible(&receipt.original_head)
        || receipt.original_head.ticket != head.ticket
        || receipt.original_head.references != head.references
        || receipt.original_head.revision.checked_add(1) != Some(head.revision)
        || !valid_hash(&receipt.codex_tree_sha256)
        || !valid_hash(&receipt.grok_tree_sha256)
    {
        return Err(denied("early receipt does not bind original head"));
    }
    // Verify immutable captured scope/live; later legitimate client edits are
    // not reinterpreted as a reason to replay a previously closed session.
    if refs.live_sha256.is_some() {
        session::verify_closed_reference_files(&originals, refs)
            .map_err(|_| denied("closed early references changed"))?;
    } else {
        session::verify_captured_reference_files(&originals, refs)
            .map_err(|_| denied("closed captured references changed"))?;
    }
    Ok(())
}
pub fn finish(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let deadline = started + crate::modules::process::stop_timeout();
    let phase = std::cell::Cell::new("open-existing-index");
    let result: AppResult<Outcome> = (|| {
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        phase.set("confirm-original-trees");
        let original = originals(context, data, &head).map_err(fail)?;
        let checked = review_objects(&original, &head)
            .map_err(fail)?
            .ok_or_else(|| fail(()))?;
        if head.revision != request.revision
            || head.references.as_ref().unwrap().attempt != request.attempt_id
            || checked.fingerprint != request.confirmation_digest
        {
            return Err(fail(()));
        }
        let refs = head.references.as_ref().unwrap();
        let session = session::StartupSession::open(context, data, &client, refs)?;
        phase.set("admit-current-image");
        let root = context
            .runtime
            .managed_installation_root()
            .ok_or_else(|| fail(()))?;
        let (target, platform) = crate::modules::runtime::program_layout::host_target()?;
        let program =
            crate::modules::runtime::program_selection::read_complete(&root, platform, target)?
                .ok_or_else(|| fail(()))?;
        phase.set("current-service-absence");
        session.require_early_absence(context, deadline)?;
        phase.set("full-tree-and-lock-readback");
        let proof = readback(&original, &head, deadline).map_err(fail)?;
        phase.set("recheck-current-absence-and-image");
        session.require_early_absence(context, deadline)?;
        crate::modules::runtime::program_selection::verify_current(&root, &program)?;
        crate::modules::runtime::program_image::verify(&program)?;
        phase.set("close-unchanged-index");
        let next = index.close_unchanged(&head, &proof).map_err(fail)?;
        Ok(Outcome {
            attempt_id: request.attempt_id,
            revision: next.revision,
            status: "unchanged",
        })
    })();
    if result.is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data).append_event(&format!(
            "client-recovery-unchanged: refused phase={} elapsed_ms={}",
            phase.get(),
            started.elapsed().as_millis()
        ));
    }
    result.map_err(fail)
}

#[cfg(test)]
pub(in crate::infrastructure::macos_client_originals) fn fixture_readback(
    originals: &Originals,
    head: &Head,
    deadline: Instant,
) -> io::Result<Proof> {
    readback(originals, head, deadline)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "read-only independent tree/receipt proof for retained owned early closure"]
    fn inspect_owned_unchanged_trees_and_receipt() {
        use base64::Engine;
        let root = PathBuf::from(
            std::env::var_os("OCXD024_UNCHANGED_READBACK_ROOT").expect("owned fixture"),
        );
        assert!(
            root.canonicalize().unwrap() == root
                && root.starts_with("/private/tmp")
                && root
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-versioned-lifecycle-")
        );
        let profile = root.join("profile");
        let data = profile.join(".codex-desktop/desktop");
        let client = profile.join(".codex");
        let c = rusqlite::Connection::open_with_flags(
            data.join("manager-state/client-session-index.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let raw: Vec<u8> = c
            .query_row("SELECT document FROM current_session", [], |r| r.get(0))
            .unwrap();
        c.close().unwrap();
        let head: Head = decode(&raw).unwrap();
        verify_registered(&data, &client, &profile, &head).unwrap();
        let refs = head.references.as_ref().unwrap();
        let originals = Originals::open_for_profile(
            &data,
            &client,
            &profile,
            &refs.attempt,
            &refs.original_sha256,
            &refs.phases,
        )
        .unwrap();
        let receipt_name = receipt_name(&head).unwrap();
        let Image::File { bytes, .. } =
            capture_file(&originals.directory, &receipt_name, MAX_FRAME).unwrap()
        else {
            panic!("missing receipt");
        };
        let receipt: Receipt = serde_json::from_slice(&bytes).unwrap();
        let codex = encode(&TreeSnapshot::read(&originals.client).unwrap()).unwrap();
        let grok = encode(&GrokSnapshot::read(&originals.profile).unwrap()).unwrap();
        assert_eq!(digest(&codex), receipt.codex_tree_sha256);
        assert_eq!(digest(&grok), receipt.grok_tree_sha256);
        println!(
            "{}",
            serde_json::json!({"receiptSha256":digest(&bytes),"codexTreeBase64":base64::engine::general_purpose::STANDARD.encode(codex),"grokTreeBase64":base64::engine::general_purpose::STANDARD.encode(grok),"independentCurrentHashesMatchReceipt":true,"readOnly":true})
        );
    }
}
