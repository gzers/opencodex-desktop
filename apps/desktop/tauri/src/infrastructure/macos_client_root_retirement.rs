//! Reversible whole-directory preservation for an originally absent client root.
//! Storage/native-move layer only: production lifecycle authority factory and
//! independent terminal registration are NOT implemented here. Public disk
//! records or successful readback cannot construct MutationPermit.
use super::*;
use crate::infrastructure::official_client_coordination::native::{config::Guard, NativeLease};
use session_index::{observation::entry, Head};
use std::time::Instant;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    schema_version: u32,
    id: String,
    source: Head,
    origin: root_creation::References,
    execution_tree: TreeSnapshot,
    grok: GrokSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    write_guard: Option<Image>,
}
impl Plan {
    fn released_tree(&self) -> io::Result<TreeSnapshot> {
        match &self.write_guard {
            Some(image) => self.execution_tree.without_config_guard(image),
            None => Ok(self.execution_tree.clone()),
        }
    }
    fn matches_tree(&self, tree: &TreeSnapshot) -> io::Result<bool> {
        Ok(*tree == self.execution_tree
            || self.write_guard.is_some() && *tree == self.released_tree()?)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Retire,
    RestoreExecutionRoot,
}
/// No public constructor, serde or copied bool grants execution authority.
/// A future owner must supply current service/writer/root permissions and
/// maintain them across relocation; test callbacks are explicitly not that.
pub struct MutationPermit<'a> {
    check: &'a dyn Fn() -> io::Result<()>,
    deadline: Instant,
}
pub struct Pending<'a> {
    originals: &'a Originals,
    id: String,
    sha256: String,
}
/// Created only by a current apply() invocation with an unforgeable movement
/// permit. Disk receipts cannot construct this independent index-closing proof.
pub struct AuthorizedReadback {
    readback: Readback,
    plan_id: String,
    source: Head,
    data_identity: Identity,
    profile_identity: Identity,
    deadline: Instant,
}
impl AuthorizedReadback {
    pub(super) fn require(
        &self,
        data: &PinnedDirectory,
        profile: &Path,
        expected: &Head,
    ) -> io::Result<&str> {
        time(self.deadline)?;
        let profile = PinnedDirectory::open(profile)?;
        let reference = expected
            .root_retirement
            .as_ref()
            .ok_or_else(|| denied("root movement has not been independently registered"))?;
        if expected.state != session_index::State::RootRetirementPrepared
            || reference.plan.id != self.plan_id
            || reference.plan.sha256 != self.readback.plan_sha256
            || reference.source_revision != self.source.revision
            || expected.ticket != self.source.ticket
            || expected.references != self.source.references
            || expected.restore_plan != self.source.restore_plan
            || self.readback.direction != Direction::Retire
            || data.identity() != &self.data_identity
            || profile.identity() != &self.profile_identity
        {
            return Err(denied(
                "root readback is not bound to this independent movement",
            ));
        }
        Ok(&self.readback.receipt_sha256)
    }
}
pub struct Readback {
    plan_sha256: String,
    receipt_sha256: String,
    direction: Direction,
}
impl Readback {
    pub fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }
    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }
    pub fn direction(&self) -> Direction {
        self.direction
    }
}
/// These references must come from the owning independent index. They are
/// serializable facts only; no public API converts them to MutationPermit.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchiveReferences {
    pub source: Head,
    pub original_root: root_creation::References,
    pub plan_id: String,
    pub plan_sha256: String,
}
/// Read-only bootstrap after profile/.codex is absent. No path scan or digest
/// discovery occurs; every input is checked against the immutable plan/frame.
pub fn open_archived_originals(
    data_root: &Path,
    profile_root: &Path,
    reference: &ArchiveReferences,
) -> io::Result<Originals> {
    session_index::validate(&reference.source)?;
    let refs = reference
        .source
        .references
        .as_ref()
        .ok_or_else(|| denied("archived root independent Original refs absent"))?;
    if uuid::Uuid::parse_str(&refs.attempt)
        .map(|v| v.to_string() != refs.attempt)
        .unwrap_or(true)
        || uuid::Uuid::parse_str(&reference.plan_id)
            .map(|v| v.to_string() != reference.plan_id)
            .unwrap_or(true)
        || reference.original_root.id != refs.attempt
    {
        return Err(denied("invalid root archive reference context"));
    }
    let data = PinnedDirectory::open(data_root)?;
    data.private()?;
    let profile = PinnedDirectory::open(profile_root)?;
    let directory = data
        .child_private("manager-state", false)?
        .child_private("client-originals", false)?
        .child_private(&refs.attempt, false)?;
    let slot = format!("root-retired-{}", reference.plan_id);
    let client = PinnedDirectory::open(&directory.path().join(&slot))?;
    let mut known = [Some(refs.original_sha256.clone()), None, None, None];
    for (p, h) in &refs.phases {
        known[p.index()] = Some(h.clone());
    }
    let archived = Originals {
        profile,
        data,
        client,
        directory,
        attempt: refs.attempt.clone(),
        original_sha256: refs.original_sha256.clone(),
        original_root_absence: Some(reference.original_root.clone()),
        retired_root_slot: Some(slot),
        known: std::sync::Mutex::new(known),
    };
    let pending = Pending::open(&archived, &reference.plan_id, &reference.plan_sha256)?;
    if pending.load()?.source != reference.source {
        return Err(denied(
            "root archive source Head differs from independent reference",
        ));
    }
    pending.inspect_retired()?;
    Ok(archived)
}
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("root retirement action expired"))
    } else {
        Ok(())
    }
}
fn exact_receipt(vault: &PinnedDirectory, name: &str, bytes: &[u8]) -> io::Result<()> {
    match capture_file(vault, name, MAX_FRAME)? {
        Image::Absent => create(vault, name, bytes),
        Image::File {
            bytes: actual,
            protection,
            ..
        } if actual == bytes => require_private(&protection, false),
        _ => Err(denied("root movement receipt changed; no replacement")),
    }
}
impl<'a> Pending<'a> {
    /// Read-only source admission plus a private immutable plan. The captured
    /// execution tree is NOT substituted for the original root-absence fact.
    pub fn prepare(
        context: &crate::state::ProcessContext,
        originals: &'a Originals,
        source: &Head,
    ) -> io::Result<Self> {
        Self::prepare_with_marker(context, originals, source, None)
    }
    pub fn prepare_guarded(
        context: &crate::state::ProcessContext,
        originals: &'a Originals,
        source: &Head,
        native: &NativeLease,
        guard: &mut Guard<'_>,
    ) -> io::Result<Self> {
        native
            .require_context(context, originals.client.path())
            .map_err(|_| denied("root writer context unconfirmed"))?;
        let image = guard
            .root_image(native, originals.client.path())
            .map_err(|_| denied("root writer holder unconfirmed"))?;
        // Closing the original claim transaction normally removes its journal.
        // Snapshot only after the current OFD handoff, while N/L/K and the
        // exact config writer remain held. Never filter a journal from Original.
        native
            .begin_claim_handoff()
            .map_err(|_| denied("root claim handoff unconfirmed"))?;
        if guard
            .root_image(native, originals.client.path())
            .map_err(|_| denied("root writer continuity after handoff unconfirmed"))?
            != image
        {
            return Err(denied("root writer changed during claim handoff"));
        }
        Self::prepare_with_marker(context, originals, source, Some(image))
    }
    fn prepare_with_marker(
        context: &crate::state::ProcessContext,
        originals: &'a Originals,
        source: &Head,
        write_guard: Option<Image>,
    ) -> io::Result<Self> {
        session_index::validate(source)?;
        let origin = originals
            .original_root_absence
            .as_ref()
            .ok_or_else(|| denied("root retirement requires independent original absence"))?;
        let refs = source
            .references
            .as_ref()
            .ok_or_else(|| denied("root retirement Original is not independently referenced"))?;
        if refs.attempt != origin.id
            || refs.attempt != originals.attempt
            || refs.original_sha256 != originals.original_sha256
            || !matches!(
                source.state,
                session_index::State::Captured
                    | session_index::State::Stopped
                    | session_index::State::RestorePrepared
                    | session_index::State::RecoveryPrepared
            )
        {
            return Err(denied("root retirement source references differ"));
        }
        if originals.retired_root_slot.is_some() {
            return Err(denied("archived view cannot prepare a new root baseline"));
        }
        session::verify_index_scope(originals, refs, &context.opencodex_home)
            .map_err(|_| denied("root retirement source scope changed"))?;
        let original = originals.frame(Phase::Original)?.0;
        let tree = TreeSnapshot::read(&originals.client)?;
        let grok = GrokSnapshot::read(&originals.profile)?;
        let released = match &write_guard {
            Some(image) => {
                if original.tree.has_config_guard() {
                    return Err(denied("original user tree already contains writer marker"));
                }
                tree.without_config_guard(image)?
            }
            None => tree.clone(),
        };
        if !original.tree.restored_matches(&released, true)
            || !original.grok.restored_matches(&grok)
        {
            return Err(denied(
                "execution client files are not restored before root retirement",
            ));
        }
        if originals.client.identity().device != originals.directory.identity().device {
            return Err(denied("root retirement crosses volumes"));
        }
        let plan = Plan {
            schema_version: if write_guard.is_some() { 2 } else { 1 },
            id: uuid::Uuid::new_v4().to_string(),
            source: source.clone(),
            origin: origin.clone(),
            execution_tree: tree,
            grok,
            write_guard,
        };
        let bytes = encode(&plan)?;
        create(
            &originals.directory,
            &format!("root-retirement-{}.json", plan.id),
            &bytes,
        )?;
        let result = Self {
            originals,
            id: plan.id,
            sha256: digest(&bytes),
        };
        result.load()?;
        Ok(result)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn open(originals: &'a Originals, id: &str, sha256: &str) -> io::Result<Self> {
        if uuid::Uuid::parse_str(id)
            .map(|v| v.to_string() != id)
            .unwrap_or(true)
        {
            return Err(denied("invalid root movement plan identity"));
        }
        let value = Self {
            originals,
            id: id.into(),
            sha256: sha256.into(),
        };
        value.load()?;
        Ok(value)
    }
    fn slot(&self) -> String {
        format!("root-retired-{}", self.id)
    }
    fn load(&self) -> io::Result<Plan> {
        self.originals.data.private()?;
        self.originals.profile.recheck()?;
        self.originals.directory.private()?;
        let Image::File {
            bytes, protection, ..
        } = capture_file(
            &self.originals.directory,
            &format!("root-retirement-{}.json", self.id),
            MAX_FRAME,
        )?
        else {
            return Err(denied("root movement plan is absent"));
        };
        require_private(&protection, false)?;
        if digest(&bytes) != self.sha256 {
            return Err(denied("root movement plan independent digest differs"));
        }
        let plan: Plan = serde_json::from_value(
            parse_strict_json_v2(&bytes).map_err(|_| denied("invalid root plan JSON"))?,
        )
        .map_err(|_| denied("invalid root plan fields"))?;
        session_index::validate(&plan.source)?;
        if plan.schema_version != if plan.write_guard.is_some() { 2 } else { 1 }
            || plan.id != self.id
            || Some(&plan.origin) != self.originals.original_root_absence.as_ref()
            || plan.source.references.as_ref().is_none_or(|r| {
                r.attempt != self.originals.attempt
                    || r.original_sha256 != self.originals.original_sha256
            })
            || plan.source.ticket != plan.origin.id
            || !matches!(
                plan.source.state,
                session_index::State::Captured
                    | session_index::State::Stopped
                    | session_index::State::RestorePrepared
                    | session_index::State::RecoveryPrepared
            )
            || !plan.execution_tree.valid()
            || !plan.execution_tree.coordinated()
            || plan.execution_tree.root != *self.originals.client.identity()
        {
            return Err(denied("root movement plan belongs to another Original"));
        }
        if let Some(image) = &plan.write_guard {
            let Image::File {
                bytes, protection, ..
            } = image
            else {
                return Err(denied("root writer marker is not a file"));
            };
            if protection.uid != unsafe { libc::geteuid() }
                || protection.mode != 0o600
                || protection.acl.is_some()
                || protection.flags != 0
                || protection
                    .xattrs
                    .iter()
                    .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
            {
                return Err(denied("unsafe temporary writer marker protection"));
            }
            let value = parse_strict_json_v2(bytes)
                .map_err(|_| denied("temporary writer marker is not strict JSON"))?;
            if bytes.len() > 4096
                || !value["pid"]
                    .as_u64()
                    .is_some_and(|pid| pid > 0 && pid <= i32::MAX as u64)
                || !value["token"]
                    .as_str()
                    .is_some_and(|s| s.len() == 16 && s.bytes().all(|c| c.is_ascii_hexdigit()))
                || plan.released_tree()?.has_config_guard()
            {
                return Err(denied("root writer marker projection is invalid"));
            }
        }
        Ok(plan)
    }
    /// Open the SAME directory object in its plan-bound private archive. Its
    /// original logical path remains profile/.codex in immutable scope facts.
    fn archive_prefix(&self, plan: &Plan) -> io::Result<OpenedPrefix> {
        let refs = plan
            .source
            .references
            .as_ref()
            .ok_or_else(|| denied("root source references missing"))?;
        let client = PinnedDirectory::open(&self.originals.directory.path().join(self.slot()))?;
        root_creation::verify_indexed_anchor(
            &self.originals.data.child_private("manager-state", false)?,
            &self.originals.data,
            self.originals.profile.path(),
            &client,
            &plan.origin,
            &self.originals.directory.path().join(self.slot()),
        )?;
        let mut known = [Some(refs.original_sha256.clone()), None, None, None];
        for (p, h) in &refs.phases {
            known[p.index()] = Some(h.clone());
        }
        let archived = Originals {
            profile: PinnedDirectory::open(self.originals.profile.path())?,
            data: PinnedDirectory::open(self.originals.data.path())?,
            client,
            directory: PinnedDirectory::open(self.originals.directory.path())?,
            attempt: refs.attempt.clone(),
            original_sha256: refs.original_sha256.clone(),
            original_root_absence: Some(plan.origin.clone()),
            retired_root_slot: Some(self.slot()),
            known: std::sync::Mutex::new(known),
        };
        let last = refs.phases.last().map_or(Phase::Original, |(p, _)| *p);
        let frames = archived.frames_through(last)?;

        let Image::File {
            bytes, protection, ..
        } = capture_file(&archived.directory, "startup-scope.json", MAX_FRAME)?
        else {
            return Err(denied("root source scope missing"));
        };
        require_private(&protection, false)?;
        if digest(&bytes) != refs.scope_sha256 {
            return Err(denied("root source scope digest changed"));
        }
        let value =
            parse_strict_json_v2(&bytes).map_err(|_| denied("root source scope invalid"))?;
        let home = value["home"]
            .as_str()
            .ok_or_else(|| denied("root source HOME missing"))?;
        session::verify_index_scope_prefix(&archived, refs, Path::new(home), &frames)
            .map_err(|_| denied("archived root lost original logical scope"))?;
        let original = &frames
            .first()
            .ok_or_else(|| denied("empty Original prefix"))?
            .0;
        if plan.write_guard.is_some() && original.tree.has_config_guard()
            || !original.tree.restored_matches(&plan.released_tree()?, true)
            || !original.grok.restored_matches(&plan.grok)
        {
            return Err(denied(
                "retired execution tree differs from original initialized state",
            ));
        }
        Ok((archived, frames))
    }
    fn live_view(&self, plan: &Plan) -> io::Result<Originals> {
        self.live_prefix(plan).map(|v| v.0)
    }
    fn live_prefix(&self, plan: &Plan) -> io::Result<OpenedPrefix> {
        let refs = plan
            .source
            .references
            .as_ref()
            .ok_or_else(|| denied("root source references missing"))?;
        Originals::open_profile_origin_prefix(
            self.originals.data.path(),
            &self.originals.profile.path().join(".codex"),
            self.originals.profile.path(),
            OriginalKeys {
                attempt: &refs.attempt,
                original_sha256: &refs.original_sha256,
                expected_phases: &refs.phases,
            },
            Some(plan.origin.clone()),
        )
    }
    fn checked_live(&self, plan: &Plan) -> io::Result<OpenedPrefix> {
        let (live, frames) = self.live_prefix(plan)?;
        let original = &frames
            .first()
            .ok_or_else(|| denied("empty Original prefix"))?
            .0;
        if plan.write_guard.is_some() && original.tree.has_config_guard()
            || !original.tree.restored_matches(&plan.released_tree()?, true)
            || !original.grok.restored_matches(&plan.grok)
        {
            return Err(denied(
                "live execution plan no longer matches independent Original",
            ));
        }
        if entry(&self.originals.directory, &self.slot())?.is_some()
            || !plan.matches_tree(&TreeSnapshot::read(&live.client)?)?
            || GrokSnapshot::read(&live.profile)? != plan.grok
        {
            return Err(denied(
                "live root changed or archive destination already exists",
            ));
        }
        Ok((live, frames))
    }
    fn live_matches(&self, plan: &Plan) -> io::Result<()> {
        self.checked_live(plan).map(|_| ())
    }
    fn checked_retired(&self, plan: &Plan) -> io::Result<OpenedPrefix> {
        if entry(&self.originals.profile, ".codex")?.is_some() {
            return Err(denied(
                "a current client root exists; absence is not restored",
            ));
        }
        let (archived, frames) = self.archive_prefix(plan)?;
        if !plan.matches_tree(&TreeSnapshot::read(&archived.client)?)?
            || GrokSnapshot::read(&archived.profile)? != plan.grok
        {
            return Err(denied("retired root contents or protection changed"));
        }
        if entry(&archived.profile, ".codex")?.is_some() {
            return Err(denied("client root absence changed during readback"));
        }
        Ok((archived, frames))
    }
    fn retired_matches(&self, plan: &Plan) -> io::Result<Originals> {
        self.checked_retired(plan).map(|v| v.0)
    }
    /// Read-only frames in their exact plan-bound archive. This cannot seal
    /// new history or construct root movement/service authority.
    pub fn archived_originals(&self) -> io::Result<Originals> {
        let plan = self.load()?;
        self.retired_matches(&plan)
    }
    pub fn inspect_retired(&self) -> io::Result<()> {
        let plan = self.load()?;
        self.retired_matches(&plan)?;
        Ok(())
    }
    pub fn apply(
        &self,
        direction: Direction,
        permit: &MutationPermit<'_>,
        deadline: Instant,
    ) -> io::Result<AuthorizedReadback> {
        let deadline = deadline.min(permit.deadline);
        let readback = self.apply_checked(
            direction,
            &|| {
                time(deadline)?;
                (permit.check)()
            },
            &|_| Ok(()),
        )?;
        self.authorized_readback(readback, deadline)
    }
    /// Movement remains gated by the private lifecycle permit. A real Guard
    /// supplies live continuity and normal release; serialized marker facts
    /// cannot substitute for it. Only retirement, never revive an old holder.
    pub fn apply_guarded(
        &self,
        permit: &MutationPermit<'_>,
        native: &NativeLease,
        guard: Guard<'_>,
        deadline: Instant,
    ) -> io::Result<AuthorizedReadback> {
        self.apply_guarded_checked(permit, native, guard, deadline, &|_| Ok(()))
    }
    fn apply_guarded_checked(
        &self,
        permit: &MutationPermit<'_>,
        native: &NativeLease,
        mut guard: Guard<'_>,
        deadline: Instant,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<AuthorizedReadback> {
        let deadline = deadline.min(permit.deadline);
        time(deadline)?;
        (permit.check)()?;
        let plan = self.load()?;
        let image = guard
            .root_image(native, self.originals.logical_client().as_path())
            .map_err(|_| denied("current root writer unconfirmed"))?;
        if plan.write_guard.as_ref() != Some(&image) {
            return Err(denied("root writer differs from independently bound plan"));
        }
        let target = self.originals.directory.path().join(self.slot());
        guard
            .prepare_relocation(&target)
            .map_err(|_| denied("writer destination unconfirmed"))?;
        native
            .require_claim_handoff()
            .map_err(|_| denied("root claim handoff is not current"))?;
        let holder = std::cell::RefCell::new(Some(guard));
        let released = std::cell::Cell::new(false);
        let readback = self.apply_checked_inner(
            Direction::Retire,
            &|| {
                time(deadline)?;
                (permit.check)()?;
                native
                    .check()
                    .map_err(|_| denied("root native lease unconfirmed"))?;
                if let Some(guard) = holder.borrow_mut().as_mut() {
                    guard
                        .check()
                        .map_err(|_| denied("root writer continuity unconfirmed"))?;
                }
                time(deadline)
            },
            &|phase| {
                checkpoint(phase)?;
                if phase == "after_native" {
                    native
                        .reanchor_claim(&target)
                        .map_err(|_| denied("moved native root unconfirmed"))?;
                    holder
                        .borrow_mut()
                        .as_mut()
                        .ok_or_else(|| denied("root holder missing"))?
                        .reanchor(&target)
                        .map_err(|_| denied("moved writer unconfirmed"))?;
                }
                if phase == "settle_writer" {
                    holder
                        .borrow_mut()
                        .take()
                        .ok_or_else(|| denied("root holder missing"))?
                        .finish()
                        .map_err(|_| denied("root writer release unconfirmed"))?;
                    released.set(true);
                }
                Ok(())
            },
            true,
        )?;
        if !released.get() {
            return Err(denied("root writer release not proven"));
        }
        self.authorized_readback(readback, deadline)
    }
    fn authorized_readback(
        &self,
        readback: Readback,
        deadline: Instant,
    ) -> io::Result<AuthorizedReadback> {
        time(deadline)?;
        let plan = self.load()?;
        Ok(AuthorizedReadback {
            readback,
            plan_id: self.id.clone(),
            source: plan.source,
            data_identity: self.originals.data.identity().clone(),
            profile_identity: self.originals.profile.identity().clone(),
            deadline,
        })
    }
    #[cfg(test)]
    pub(super) fn fixture_apply_for_index(
        &self,
        deadline: Instant,
    ) -> io::Result<AuthorizedReadback> {
        let readback = self.apply_checked(Direction::Retire, &|| time(deadline), &|_| Ok(()))?;
        self.authorized_readback(readback, deadline)
    }
    pub(super) fn require_source(&self, expected: &Head) -> io::Result<()> {
        if self.load()?.source != *expected {
            return Err(denied(
                "root movement source does not match independent event",
            ));
        }
        Ok(())
    }
    pub(super) fn belongs_to(&self, originals: &Originals) -> bool {
        std::ptr::eq(self.originals, originals)
    }
    /// Reopen only this independently referenced plan's exact physical root,
    /// retaining original logical scope. No new frame, scan or authority.
    pub(super) fn current_prefix(&self) -> io::Result<OpenedPrefix> {
        let plan = self.load()?;
        if entry(&self.originals.profile, ".codex")?.is_some() {
            self.checked_live(&plan)
        } else {
            self.checked_retired(&plan)
        }
    }
    /// The sole production factory takes the concrete session-owned authority,
    /// whose fields and construction live in the normal-Stop session module.
    /// No arbitrary callback or serialized record can call this as permission.
    pub(super) fn apply_completed_stop(
        &self,
        authority: session::RootRetirementAuthority<'_>,
        native: &NativeLease,
        guard: Guard<'_>,
        deadline: Instant,
    ) -> io::Result<AuthorizedReadback> {
        authority
            .require_pending(self)
            .map_err(|_| denied("root Stop authority does not bind this plan"))?;
        let deadline = deadline.min(authority.deadline());
        let check = || {
            authority
                .check()
                .map_err(|_| denied("current normal Stop root authority unconfirmed"))
        };
        let permit = MutationPermit {
            check: &check,
            deadline,
        };
        let proof = self.apply_guarded(&permit, native, guard, deadline)?;
        authority
            .finish_check()
            .map_err(|_| denied("root Stop final absence unconfirmed"))?;
        Ok(proof)
    }
    pub(super) fn inspect_live(&self) -> io::Result<()> {
        let plan = self.load()?;
        self.live_matches(&plan)
    }
    pub(super) fn verify_retire_readback(&self, expected_sha256: &str) -> io::Result<()> {
        let plan = self.load()?;
        let archive = self.retired_matches(&plan)?;
        let tree = TreeSnapshot::read(&archive.client)?;
        if tree != plan.released_tree()? {
            return Err(denied("terminal root still contains temporary writer"));
        }
        let expected = self.receipt_bytes(&plan, Direction::Retire, &tree)?;
        match capture_file(
            &self.originals.directory,
            &format!("root-retirement-{}-Retire-readback.json", self.id),
            MAX_FRAME,
        )? {
            Image::File {
                bytes, protection, ..
            } if digest(&bytes) == expected_sha256 && bytes == expected => {
                require_private(&protection, false)
            }
            _ => Err(denied(
                "independent root absence readback is missing or changed",
            )),
        }
    }
    fn receipt_bytes(
        &self,
        plan: &Plan,
        direction: Direction,
        actual_tree: &TreeSnapshot,
    ) -> io::Result<Vec<u8>> {
        encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"direction":direction,"sourceHead":plan.source,"originalRootAbsent":true,"logicalClient":self.originals.profile.path().join(".codex"),"archivedSlot":self.slot(),"rootIdentity":actual_tree.root,"executionTreeSha256":digest(&encode(actual_tree)?),"grokSha256":digest(&encode(&plan.grok)?),"currentRootAbsent":direction==Direction::Retire}),
        )
    }
    fn apply_checked(
        &self,
        direction: Direction,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Readback> {
        self.apply_checked_inner(direction, check, checkpoint, false)
    }
    fn apply_checked_inner(
        &self,
        direction: Direction,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
        guarded: bool,
    ) -> io::Result<Readback> {
        check()?;
        let plan = self.load()?;
        if guarded != plan.write_guard.is_some() || guarded && direction != Direction::Retire {
            return Err(denied("current writer required for guarded root movement"));
        }
        let (from, from_name, to, to_name) = match direction {
            Direction::Retire => (
                &self.originals.profile,
                ".codex".to_owned(),
                &self.originals.directory,
                self.slot(),
            ),
            Direction::RestoreExecutionRoot => (
                &self.originals.directory,
                self.slot(),
                &self.originals.profile,
                ".codex".to_owned(),
            ),
        };
        let already = match direction {
            Direction::Retire => self.retired_matches(&plan).is_ok(),
            Direction::RestoreExecutionRoot => self.live_matches(&plan).is_ok(),
        };
        let intent_name = format!("root-retirement-{}-{:?}-intent.json", self.id, direction);
        let intent = encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"direction":direction}),
        )?;
        if already {
            match capture_file(&self.originals.directory, &intent_name, MAX_FRAME)? {
                Image::File {
                    bytes, protection, ..
                } if bytes == intent => require_private(&protection, false)?,
                _ => return Err(denied("root at destination lacks original movement intent")),
            }
        } else {
            match direction {
                Direction::Retire => self.live_matches(&plan)?,
                Direction::RestoreExecutionRoot => {
                    self.retired_matches(&plan)?;
                }
            }
            exact_receipt(&self.originals.directory, &intent_name, &intent)?;
            checkpoint("intent")?;
            check()?;
            match direction {
                Direction::Retire => self.live_matches(&plan)?,
                Direction::RestoreExecutionRoot => {
                    self.retired_matches(&plan)?;
                }
            }
            let source =
                entry(from, &from_name)?.ok_or_else(|| denied("root source disappeared"))?;
            if identity(&source) != plan.execution_tree.root {
                return Err(denied("root move source identity differs"));
            }
            checkpoint("native_gap")?;
            check()?;
            let a = name(OsStr::new(&from_name))?;
            let b = name(OsStr::new(&to_name))?;
            checkpoint("before_rename")?;
            if unsafe {
                libc::renameatx_np(
                    from.file().as_raw_fd(),
                    a.as_ptr(),
                    to.file().as_raw_fd(),
                    b.as_ptr(),
                    libc::RENAME_EXCL,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
            from.sync()?;
            to.sync()?;
            let actual = entry(to, &to_name)?.ok_or_else(|| denied("moved root disappeared"))?;
            if identity(&actual) != plan.execution_tree.root {
                // We moved a late replacement. Preserve the complete object,
                // recording only metadata, then reverse once without overwrite.
                create(
                    &self.originals.directory,
                    &format!(
                        "root-retirement-{}-conflict-{}.json",
                        self.id,
                        uuid::Uuid::new_v4()
                    ),
                    &encode(
                        &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"expected":plan.execution_tree.root,"observed":identity(&actual),"mode":actual.st_mode,"direction":direction}),
                    )?,
                )?;
                checkpoint("compensation_gap")?;
                check()?;
                let now =
                    entry(to, &to_name)?.ok_or_else(|| denied("root conflict changed again"))?;
                if identity(&now) != identity(&actual) || entry(from, &from_name)?.is_some() {
                    return Err(denied(
                        "root conflict cannot be reversed without overwriting",
                    ));
                }
                if unsafe {
                    libc::renameatx_np(
                        to.file().as_raw_fd(),
                        b.as_ptr(),
                        from.file().as_raw_fd(),
                        a.as_ptr(),
                        libc::RENAME_EXCL,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error());
                }
                to.sync()?;
                from.sync()?;
                return Err(denied("outside root returned; movement remains failed"));
            }
            // Always identify/record a moved replacement BEFORE attempting a
            // successful root/holder reanchor. If current authority has become
            // unknown, compensation refuses but the conflict remains recorded.
            checkpoint("after_native")?;
            check()?;
        }
        check()?;
        if guarded {
            checkpoint("settle_writer")?;
            check()?;
        }
        let actual_tree = match direction {
            Direction::Retire => {
                let archive = self.retired_matches(&plan)?;
                TreeSnapshot::read(&archive.client)?
            }
            Direction::RestoreExecutionRoot => {
                self.live_matches(&plan)?;
                TreeSnapshot::read(&self.live_view(&plan)?.client)?
            }
        };
        if actual_tree != plan.released_tree()? {
            return Err(denied("temporary writer has not been exactly released"));
        }
        let receipt = self.receipt_bytes(&plan, direction, &actual_tree)?;
        checkpoint("before_readback")?;
        check()?;
        match direction {
            Direction::Retire => {
                self.retired_matches(&plan)?;
            }
            Direction::RestoreExecutionRoot => self.live_matches(&plan)?,
        }
        exact_receipt(
            &self.originals.directory,
            &format!("root-retirement-{}-{:?}-readback.json", self.id, direction),
            &receipt,
        )?;
        checkpoint("readback")?;
        check()?;
        match direction {
            Direction::Retire => {
                self.retired_matches(&plan)?;
            }
            Direction::RestoreExecutionRoot => self.live_matches(&plan)?,
        }
        Ok(Readback {
            plan_sha256: self.sha256.clone(),
            receipt_sha256: digest(&receipt),
            direction,
        })
    }
}
#[cfg(test)]
#[path = "macos_client_root_retirement_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "macos_client_root_retirement_index_tests.rs"]
mod index_tests;
#[cfg(test)]
#[path = "macos_client_root_transient_tests.rs"]
pub(crate) mod transient_tests;
