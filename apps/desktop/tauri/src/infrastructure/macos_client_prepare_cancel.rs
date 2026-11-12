//! Explicit cancellation BEFORE any original vault exists. This is NOT a
//! restore/unchanged-original proof. Any retained vault (even empty, unrelated,
//! unsafe or partial) refuses cancellation; nothing is scanned or deleted.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    commands::runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider},
    commands::runtime_recovery_v2_owner::SystemServiceOwnerProofProvider,
    errors::{AppError, AppResult},
};
use std::time::Instant;
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_prepare_cancel_refused".into(),
        detail: "启动准备或停止状态尚未核实；未取消，原现场保留".into(),
    }
}
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("prepare cancel budget expired"))
    } else {
        Ok(())
    }
}
fn eligible(head: &Head) -> bool {
    head.state == State::Preparing
        && head.references.is_none()
        && head.restore_plan.is_none()
        && head.completion_sha256.is_none()
}
fn filename(head: &Head) -> String {
    let revision = if head.state == State::CancelledBeforeCapture {
        head.revision.saturating_sub(1)
    } else {
        head.revision
    };
    format!("prepare-cancel-{}-{revision}.json", head.ticket)
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    original_head: Head,
    binding_sha256: String,
    codex_sha256: String,
    grok_sha256: String,
}
struct Review {
    codex: TreeSnapshot,
    grok: GrokSnapshot,
    fingerprint: String,
}
pub struct Proof {
    head: Head,
    binding_sha256: String,
    sha256: String,
    deadline: Instant,
}
impl Proof {
    pub(super) fn require(&self, head: &Head, binding: &[u8]) -> io::Result<()> {
        time(self.deadline)?;
        if !eligible(head) || *head != self.head || digest(binding) != self.binding_sha256 {
            return Err(denied("cancel proof belongs to another reservation"));
        }
        Ok(())
    }
    pub(super) fn sha256(&self) -> &str {
        &self.sha256
    }
}
fn vault_absent(data: &PinnedDirectory) -> io::Result<PinnedDirectory> {
    data.private()?;
    let manager = data.child_private("manager-state", false)?;
    if observation::entry(&manager, "client-originals")?.is_some() {
        return Err(denied(
            "original vault exists; cancellation cannot discard unknown originals",
        ));
    }
    Ok(manager)
}
fn receipt(head: &Head, binding: &[u8], review: &Review) -> io::Result<Receipt> {
    Ok(Receipt {
        schema_version: 1,
        original_head: head.clone(),
        binding_sha256: digest(binding),
        codex_sha256: digest(&encode(&review.codex)?),
        grok_sha256: digest(&encode(&review.grok)?),
    })
}
fn inspect_objects(
    context: &ProcessContext,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    head: &Head,
    binding: &[u8],
) -> io::Result<Option<Review>> {
    if !eligible(head) {
        return Ok(None);
    }
    let manager = vault_absent(data)?;
    let profile = PinnedDirectory::open(&context.working_directory)?;
    let codex = TreeSnapshot::read(client)?;
    let grok = GrokSnapshot::read(&profile)?;
    if scope(context, data, client)? != binding {
        return Err(denied("preparing scope changed"));
    }
    let fingerprint = digest(&encode(&(head, digest(binding), &codex, &grok))?);
    let result = Review {
        codex,
        grok,
        fingerprint,
    };
    match capture_file(&manager, &filename(head), MAX_FRAME)? {
        Image::Absent => {}
        Image::File {
            bytes, protection, ..
        } => {
            require_private(&protection, false)?;
            if bytes != encode(&receipt(head, binding, &result)?)? {
                return Err(denied("existing cancellation receipt differs; not adopted"));
            }
        }
    }
    vault_absent(data)?;
    Ok(Some(result))
}
pub fn review(context: &ProcessContext, data: &Path) -> AppResult<Option<Request>> {
    observation::inspect_map(context, data, |head| {
        let Some(head) = head.filter(|h| eligible(h)) else {
            return Ok(None);
        };
        let data = PinnedDirectory::open(data)?;
        let client = PinnedDirectory::open(&std::fs::canonicalize(
            context.working_directory.join(".codex"),
        )?)?;
        let binding = scope(context, &data, &client)?;
        Ok(
            inspect_objects(context, &data, &client, head, &binding)?.map(|r| Request {
                attempt_id: head.ticket.clone(),
                revision: head.revision,
                confirmation_digest: r.fingerprint,
            }),
        )
    })
    .map_err(fail)
}
/// Projection under the caller's existing shared action lock; never opens a
/// second index connection or initializes a missing vault.
pub(super) fn confirmation(
    context: &ProcessContext,
    data: &Path,
    head: &Head,
) -> io::Result<Option<String>> {
    if !eligible(head) {
        return Ok(None);
    }
    let data = PinnedDirectory::open(data)?;
    let client = PinnedDirectory::open(&std::fs::canonicalize(
        context.working_directory.join(".codex"),
    )?)?;
    let binding = scope(context, &data, &client)?;
    Ok(inspect_objects(context, &data, &client, head, &binding)?.map(|r| r.fingerprint))
}
fn readback(
    context: &ProcessContext,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    head: &Head,
    binding: &[u8],
    deadline: Instant,
) -> io::Result<Proof> {
    time(deadline)?;
    let first = inspect_objects(context, data, client, head, binding)?
        .ok_or_else(|| denied("not a preparing reservation"))?;
    let second = inspect_objects(context, data, client, head, binding)?
        .ok_or_else(|| denied("preparing state changed"))?;
    if first.codex != second.codex || first.grok != second.grok {
        return Err(denied("current client objects changed during cancellation"));
    }
    let raw = encode(&receipt(head, binding, &second)?)?;
    let manager = vault_absent(data)?;
    time(deadline)?;
    match capture_file(&manager, &filename(head), MAX_FRAME)? {
        Image::Absent => create(&manager, &filename(head), &raw)?,
        Image::File {
            bytes, protection, ..
        } => {
            require_private(&protection, false)?;
            if bytes != raw {
                return Err(denied("cancellation receipt differs"));
            }
        }
    }
    let Image::File {
        bytes, protection, ..
    } = capture_file(&manager, &filename(head), MAX_FRAME)?
    else {
        return Err(denied("cancel receipt vanished"));
    };
    require_private(&protection, false)?;
    if bytes != raw {
        return Err(denied("cancel receipt changed"));
    }
    let final_read = inspect_objects(context, data, client, head, binding)?
        .ok_or_else(|| denied("preparing state changed"))?;
    if final_read.codex != first.codex || final_read.grok != first.grok {
        return Err(denied("client objects changed before cancellation close"));
    }
    time(deadline)?;
    Ok(Proof {
        head: head.clone(),
        binding_sha256: digest(binding),
        sha256: digest(&raw),
        deadline,
    })
}
pub(super) fn verify_registered(
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    binding: &[u8],
    head: &Head,
) -> io::Result<()> {
    if head.state != State::CancelledBeforeCapture
        || head.references.is_some()
        || head.restore_plan.is_some()
    {
        return Err(denied("not a pre-capture cancellation terminal"));
    }
    let manager = data.child_private("manager-state", false)?;
    let Image::File {
        bytes, protection, ..
    } = capture_file(&manager, &filename(head), MAX_FRAME)?
    else {
        return Err(denied("registered cancel receipt missing"));
    };
    require_private(&protection, false)?;
    if head.completion_sha256.as_deref() != Some(digest(&bytes).as_str()) {
        return Err(denied("registered cancel receipt changed"));
    }
    let receipt: Receipt = serde_json::from_value(
        parse_strict_json_v2(&bytes).map_err(|_| denied("cancel receipt JSON invalid"))?,
    )
    .map_err(|_| denied("cancel receipt fields invalid"))?;
    validate(&receipt.original_head)?;
    if receipt.schema_version != 1
        || !eligible(&receipt.original_head)
        || receipt.original_head.ticket != head.ticket
        || receipt.original_head.revision.checked_add(1) != Some(head.revision)
        || receipt.binding_sha256 != digest(binding)
        || !valid_hash(&receipt.codex_sha256)
        || !valid_hash(&receipt.grok_sha256)
    {
        return Err(denied("cancel receipt does not bind original reservation"));
    }
    data.private()?;
    client.recheck()?;
    vault_absent(data)?;
    Ok(())
}
pub fn cancel(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let deadline = Instant::now() + crate::modules::process::stop_timeout();
    let client = std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
    let index = Index::open_existing(context, data, &client).map_err(fail)?;
    let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
    let review = inspect_objects(context, &index.data, &index.client, &head, &index.binding)
        .map_err(fail)?
        .ok_or_else(|| fail(()))?;
    if head.ticket != request.attempt_id
        || head.revision != request.revision
        || review.fingerprint != request.confirmation_digest
    {
        return Err(fail(()));
    }
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail(()))?;
    let (target, platform) = crate::modules::runtime::program_layout::host_target()?;
    let selected =
        crate::modules::runtime::program_selection::read_complete(&root, platform, target)?
            .ok_or_else(|| fail(()))?;
    let owner = SystemServiceOwnerProofProvider::new(root.clone(), context.clone())?;
    let home = PinnedDirectory::open(&context.opencodex_home).map_err(fail)?;
    let require_absent = || -> AppResult<()> {
        if owner.query(context)? != ServiceOwnerProof::Stopped
            || capture_file(&home, "runtime-port.json", 4096).map_err(fail)? != Image::Absent
            || capture_file(&home, "ocx.pid", 64).map_err(fail)? != Image::Absent
            || owner.query(context)? != ServiceOwnerProof::Stopped
        {
            return Err(fail(()));
        }
        Ok(())
    };
    require_absent()?;
    let proof = readback(
        context,
        &index.data,
        &index.client,
        &head,
        &index.binding,
        deadline,
    )
    .map_err(fail)?;
    require_absent()?;
    crate::modules::runtime::program_selection::verify_current(&root, &selected)?;
    crate::modules::runtime::program_image::verify(&selected)?;
    let next = index.close_preparation(&head, &proof).map_err(fail)?;
    Ok(Outcome {
        attempt_id: head.ticket,
        revision: next.revision,
        status: "cancelled_before_capture",
    })
}
#[cfg(test)]
pub(super) fn fixture_readback(
    context: &ProcessContext,
    index: &Index,
    head: &Head,
    deadline: Instant,
) -> io::Result<Proof> {
    readback(
        context,
        &index.data,
        &index.client,
        head,
        &index.binding,
        deadline,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "read-only complete current tree proof for retained task-owned cancellation"]
    fn inspect_owned_cancelled_preparation_without_replaying_clients() {
        use base64::Engine;
        let root =
            PathBuf::from(std::env::var_os("OCXD024_CANCEL_READBACK_ROOT").expect("owned fixture"));
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
        let home = profile.join(".codex-desktop/opencodex/home");
        let client = profile.join(".codex");
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile.clone(),
            opencodex_home: home,
        };
        let data = PinnedDirectory::open(&data).unwrap();
        let client = PinnedDirectory::open(&client).unwrap();
        let binding = scope(&context, &data, &client).unwrap();
        let c = rusqlite::Connection::open_with_flags(
            data.path.join("manager-state/client-session-index.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let raw: Vec<u8> = c
            .query_row("SELECT document FROM current_session", [], |r| r.get(0))
            .unwrap();
        c.close().unwrap();
        let head: Head = decode(&raw).unwrap();
        verify_registered(&data, &client, &binding, &head).unwrap();
        let manager = vault_absent(&data).unwrap();
        let Image::File { bytes, .. } =
            capture_file(&manager, &filename(&head), MAX_FRAME).unwrap()
        else {
            panic!("missing cancel receipt");
        };
        let receipt: Receipt = serde_json::from_slice(&bytes).unwrap();
        let codex = encode(&TreeSnapshot::read(&client).unwrap()).unwrap();
        let grok = encode(&GrokSnapshot::read(&PinnedDirectory::open(&profile).unwrap()).unwrap())
            .unwrap();
        assert_eq!(digest(&codex), receipt.codex_sha256);
        assert_eq!(digest(&grok), receipt.grok_sha256);
        println!(
            "{}",
            serde_json::json!({"receiptSha256":digest(&bytes),"codexTreeBase64":base64::engine::general_purpose::STANDARD.encode(codex),"grokTreeBase64":base64::engine::general_purpose::STANDARD.encode(grok),"completeCurrentTreesMatchReceipt":true,"vaultAbsent":true,"readOnly":true})
        );
    }
}
