//! Reversible whole-directory preservation for an originally absent client root.
//! Storage/native-move layer only: production lifecycle authority factory and
//! independent terminal registration are NOT implemented here. Public disk
//! records or successful readback cannot construct MutationPermit.
use super::*;
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
}
pub struct Pending<'a> {
    originals: &'a Originals,
    id: String,
    sha256: String,
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
        if !original.tree.restored_matches(&tree, true) || !original.grok.restored_matches(&grok) {
            return Err(denied(
                "execution client files are not restored before root retirement",
            ));
        }
        if originals.client.identity().device != originals.directory.identity().device {
            return Err(denied("root retirement crosses volumes"));
        }
        let plan = Plan {
            schema_version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            source: source.clone(),
            origin: origin.clone(),
            execution_tree: tree,
            grok,
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
        if plan.schema_version != 1
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
        Ok(plan)
    }
    /// Open the SAME directory object in its plan-bound private archive. Its
    /// original logical path remains profile/.codex in immutable scope facts.
    fn archive_view(&self, plan: &Plan) -> io::Result<Originals> {
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
        archived.frames_through(last)?;

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
        session::verify_index_scope(&archived, refs, Path::new(home))
            .map_err(|_| denied("archived root lost original logical scope"))?;
        let original = archived.frame(Phase::Original)?.0;
        if !original.tree.restored_matches(&plan.execution_tree, true)
            || !original.grok.restored_matches(&plan.grok)
        {
            return Err(denied(
                "retired execution tree differs from original initialized state",
            ));
        }
        Ok(archived)
    }
    fn live_view(&self, plan: &Plan) -> io::Result<Originals> {
        let refs = plan
            .source
            .references
            .as_ref()
            .ok_or_else(|| denied("root source references missing"))?;
        Originals::open_profile_origin(
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
    fn live_matches(&self, plan: &Plan) -> io::Result<()> {
        let live = self.live_view(plan)?;
        if entry(&self.originals.directory, &self.slot())?.is_some()
            || TreeSnapshot::read(&live.client)? != plan.execution_tree
            || GrokSnapshot::read(&live.profile)? != plan.grok
        {
            return Err(denied(
                "live root changed or archive destination already exists",
            ));
        }
        Ok(())
    }
    fn retired_matches(&self, plan: &Plan) -> io::Result<Originals> {
        if entry(&self.originals.profile, ".codex")?.is_some() {
            return Err(denied(
                "a current client root exists; absence is not restored",
            ));
        }
        let archived = self.archive_view(plan)?;
        if TreeSnapshot::read(&archived.client)? != plan.execution_tree
            || GrokSnapshot::read(&archived.profile)? != plan.grok
        {
            return Err(denied("retired root contents or protection changed"));
        }
        if entry(&archived.profile, ".codex")?.is_some() {
            return Err(denied("client root absence changed during readback"));
        }
        Ok(archived)
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
    ) -> io::Result<Readback> {
        self.apply_checked(
            direction,
            &|| {
                time(deadline)?;
                (permit.check)()
            },
            &|_| Ok(()),
        )
    }
    fn apply_checked(
        &self,
        direction: Direction,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Readback> {
        check()?;
        let plan = self.load()?;
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
            checkpoint("after_native")?;
            check()?;
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
        }
        check()?;
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
        let receipt = encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"direction":direction,"sourceHead":plan.source,"originalRootAbsent":true,"logicalClient":self.originals.profile.path().join(".codex"),"archivedSlot":self.slot(),"rootIdentity":actual_tree.root,"executionTreeSha256":digest(&encode(&actual_tree)?),"grokSha256":digest(&encode(&plan.grok)?),"currentRootAbsent":direction==Direction::Retire}),
        )?;
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
