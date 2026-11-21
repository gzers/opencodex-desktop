//! A fresh action's actual guarded movement AND sealed native release must
//! both finish before an independent cancellation terminal can be committed.
//! This is never a normal Stop/OriginalSession/Closed or successor factory.
use super::*;
use crate::infrastructure::official_client_coordination::native::released::ReleasedCoordination;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Receipt {
    schema: String,
    plan_sha256: String,
    preserved_sha256: String,
    root: super::super::super::Identity,
    tree_sha256: String,
    grok_sha256: String,
    source_image_sha256: String,
    logical_root_absent: bool,
    coordinators_released: bool,
    normal_stop: bool,
    successor_eligible: bool,
}
pub struct Completed<'a> {
    index: &'a Index,
    head: Head,
    sha256: String,
    deadline: Instant,
    service: &'a service::CurrentService<'a>,
    coordination: ReleasedCoordination,
    physical: PinnedDirectory,
}
impl Completed<'_> {
    fn require_binding(&self, index: &Index, head: &Head) -> std::io::Result<&str> {
        check_time(self.deadline)?;
        if !std::ptr::eq(self.index, index) || self.head != *head {
            return Err(denied(
                "root-cancel completion belongs to another index/action",
            ));
        }
        Ok(&self.sha256)
    }
    pub(super) fn require(&self, index: &Index, head: &Head) -> std::io::Result<&str> {
        self.require_binding(index, head)?;
        self.service.released()?;
        self.coordination
            .prove(self.service.context(), self.physical.path(), self.deadline)
            .map_err(io)?;
        self.physical.recheck()?;
        let observed = index
            .head()?
            .ok_or_else(|| denied("root-cancel completion index disappeared"))?;
        let mut terminal = self.head.clone();
        terminal.revision += 1;
        terminal.state = State::RootCancellationAbsent;
        terminal.completion_sha256 = Some(self.sha256.clone());
        if observed != self.head && observed != terminal {
            return Err(denied("root-cancel completion current phase changed"));
        }
        check_time(self.deadline)?;
        Ok(&self.sha256)
    }
}
pub fn verify_released<'a>(
    index: &'a Index,
    service: &'a service::CurrentService<'a>,
    preserved: PreservedRoot,
    coordination: ReleasedCoordination,
) -> std::io::Result<Completed<'a>> {
    check_time(preserved.deadline)?;
    if !preserved.admitted {
        return Err(denied(
            "storage-only preservation is not a production cancellation",
        ));
    }
    let head = index
        .head()?
        .ok_or_else(|| denied("root-cancel current index missing"))?;
    let reference = head
        .root_retirement
        .as_ref()
        .ok_or_else(|| denied("root-cancel reference missing"))?;
    if head.state != State::RootCancellationPrepared
        || reference.plan.id != preserved.id
        || reference.plan.sha256 != preserved.plan_sha256
    {
        return Err(denied("root-cancel current plan differs"));
    }
    let origin = index
        .root_registration
        .as_ref()
        .ok_or_else(|| denied("root-cancel original absence missing"))?;
    let indexed::Observation {
        root: physical,
        directory,
        plan,
        preserved: receipt,
        plan_image,
        ..
    } = indexed::inspect_plan(
        &index.data,
        &index.profile,
        &index.binding,
        &head,
        origin,
        index.connection()?,
    )?;
    if physical.identity() != &preserved.root_identity {
        return Err(denied("root-cancel preserved identity changed"));
    }
    coordination
        .prove(service.context(), physical.path(), preserved.deadline)
        .map_err(io)?;
    service.released()?;
    let action_head = if preserved.continued {
        &head
    } else {
        &plan.source
    };
    service.require_action(index, action_head, preserved.deadline)?;
    let released = plan.root.without_config_guard(&plan.writer)?;
    if super::super::super::TreeSnapshot::read(&physical)? != released
        || receipt.bytes().map(digest).as_deref() != Some(preserved.receipt_sha256.as_str())
        || capture_file(&directory, "preserved-readback.json", MAX_FRAME)? != receipt
        || capture_file(&directory, "plan.json", MAX_FRAME)? != plan_image
        || observation::entry(&PinnedDirectory::open(&index.profile)?, ".codex")?.is_some()
    {
        return Err(denied("root-cancel full released tree or receipt differs"));
    }
    let bytes = encode(&Receipt {
        schema: "macos-first-root-cancel-complete/1".into(),
        plan_sha256: preserved.plan_sha256,
        preserved_sha256: preserved.receipt_sha256,
        root: preserved.root_identity,
        tree_sha256: digest(&encode(&released)?),
        grok_sha256: digest(&encode(&plan.grok)?),
        source_image_sha256: service.image_sha256().into(),
        logical_root_absent: true,
        coordinators_released: true,
        normal_stop: false,
        successor_eligible: false,
    })?;
    check_time(preserved.deadline)?;
    continuation::create_or_confirm(&directory, "cancellation-completed.json", &bytes)?;
    // The pending resolver admits the exact strict DATA shape. Only this live
    // proof may bind its digest into the independent terminal below.
    coordination
        .prove(service.context(), physical.path(), preserved.deadline)
        .map_err(io)?;
    service.observe_released()?;
    if index.head()?.as_ref() != Some(&head)
        || capture_file(&directory, "cancellation-completed.json", MAX_FRAME)?.bytes()
            != Some(bytes.as_slice())
    {
        return Err(denied("root-cancel completion readback changed"));
    }
    check_time(preserved.deadline)?;
    Ok(Completed {
        index,
        head,
        sha256: digest(&bytes),
        deadline: preserved.deadline,
        service,
        coordination,
        physical,
    })
}
impl Index {
    pub fn close_first_root_cancellation(
        &self,
        expected: &Head,
        proof: &Completed<'_>,
    ) -> std::io::Result<Head> {
        // No mutation occurs before the original SQL transaction's complete
        // admission below. Avoid a duplicate full image/absence/lock audit at
        // this identical boundary; keep that audit inside SQL and after COMMIT.
        let digest = proof.require_binding(self, expected)?.to_string();
        if expected.state != State::RootCancellationPrepared {
            return Err(denied("root-cancel terminal source not pending"));
        }
        let mut next = expected.clone();
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| denied("root-cancel revision overflow"))?;
        next.state = State::RootCancellationAbsent;
        next.completion_sha256 = Some(digest);
        self.write_checked_admitted(Some(expected), &next, &|_| Ok(()), &|originals| {
            proof.require(self, expected)?;
            if originals.is_some() {
                return Err(denied(
                    "root-cancel terminal cannot adopt Original references",
                ));
            }
            Ok(())
        })?;
        proof.require(self, expected)?;
        Ok(next)
    }
}
pub(super) fn verify_receipt(
    directory: &PinnedDirectory,
    plan: &Plan,
    head: &Head,
    root: &PinnedDirectory,
    plan_sha256: &str,
) -> std::io::Result<Image> {
    let released = plan.root.without_config_guard(&plan.writer)?;
    let image = verify_receipt_projected(directory, plan, head, root, plan_sha256, &released)?;
    if image != Image::Absent && TreeSnapshot::read(root)? != released {
        return Err(denied(
            "root-cancel completion does not bind the released tree",
        ));
    }
    Ok(image)
}

pub(super) fn verify_receipt_projected(
    directory: &PinnedDirectory,
    plan: &Plan,
    head: &Head,
    root: &PinnedDirectory,
    plan_sha256: &str,
    released: &TreeSnapshot,
) -> std::io::Result<Image> {
    let image = capture_file(directory, "cancellation-completed.json", MAX_FRAME)?;
    if image == Image::Absent {
        if head.state == State::RootCancellationAbsent {
            return Err(denied("root-cancel terminal completion missing"));
        }
        return Ok(image);
    }
    let Image::File {
        bytes, protection, ..
    } = &image
    else {
        unreachable!()
    };
    require_private(protection, false)?;
    let receipt: Receipt = decode(bytes)?;
    if encode(&receipt)? != *bytes
        || receipt.schema != "macos-first-root-cancel-complete/1"
        || receipt.plan_sha256 != plan_sha256
        || receipt.root != *root.identity()
        || receipt.tree_sha256 != digest(&encode(released)?)
        || receipt.grok_sha256 != digest(&encode(&plan.grok)?)
        || !valid_hash(&receipt.source_image_sha256)
        || !receipt.logical_root_absent
        || !receipt.coordinators_released
        || receipt.normal_stop
        || receipt.successor_eligible
        || capture_file(directory, "preserved-readback.json", MAX_FRAME)?
            .bytes()
            .map(digest)
            != Some(receipt.preserved_sha256)
        || head.state == State::RootCancellationAbsent
            && head.completion_sha256.as_deref() != Some(digest(bytes).as_str())
    {
        return Err(denied(
            "root-cancel completion does not bind the indexed plan/tree",
        ));
    }
    Ok(image)
}
