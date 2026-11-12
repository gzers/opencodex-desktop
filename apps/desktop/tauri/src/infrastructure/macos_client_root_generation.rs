//! Successor directory custody and its atomic initial generation event. This
//! does NOT grant service Start or ordinary capture permission. Old binding,
//! events, Originals and archives remain immutable.
use super::*;
pub(in crate::infrastructure::macos_client_originals) struct CreationAuthority<'a> {
    index: &'a Index,
    closed: Head,
}
impl CreationAuthority<'_> {
    pub(in crate::infrastructure::macos_client_originals) fn check(&self) -> io::Result<()> {
        if self.closed.state != State::RootAbsent
            || self
                .index
                .root_registration
                .as_ref()
                .is_none_or(|r| r.id != self.closed.ticket)
        {
            return Err(denied(
                "successor root predecessor generation not supported",
            ));
        }
        self.index.closed_root_history(&self.closed)?;
        Ok(())
    }
    pub(in crate::infrastructure::macos_client_originals) fn slots(
        &self,
    ) -> io::Result<Vec<String>> {
        self.check()?;
        let history = vault_history::read(self.index.connection()?, &self.closed)?;
        Ok(root_lineage::parse(&self.index.binding, &history)?.slots())
    }
    pub(in crate::infrastructure::macos_client_originals) fn paths(&self) -> (&Path, &Path, &Path) {
        (
            self.index.data.path(),
            &self.index.profile,
            self.index.action_parent.path(),
        )
    }
    pub(in crate::infrastructure::macos_client_originals) fn clone_action(
        &self,
    ) -> io::Result<File> {
        self.check()?;
        self.index.action_file.try_clone()
    }
}
impl Index {
    /// One product preparation boundary. An absent logical root is either a
    /// genuinely uninitialized first root or the exact independently closed
    /// successor source. Never release and reacquire the successor action.
    pub fn prepare_start_capture(
        context: &ProcessContext,
        data: &Path,
        deadline: Instant,
    ) -> io::Result<(Self, Head)> {
        if Instant::now() >= deadline {
            return Err(denied("client capture preparation expired"));
        }
        let logical = context.working_directory.join(".codex");
        match std::fs::symlink_metadata(&logical) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let pinned = PinnedDirectory::open(data)?;
                pinned.private()?;
                let initialized = if observation::entry(&pinned, "manager-state")?.is_some() {
                    observation::entry(&pinned.child_private("manager-state", false)?, FILE)?
                        .is_some()
                } else {
                    false
                };
                if initialized {
                    let index = Self::open_existing(context, data, &logical)?;
                    let head = index
                        .head()?
                        .ok_or_else(|| denied("successor index has no source"))?;
                    index.create_successor_root(context, &head, deadline)
                } else {
                    let root = super::super::root_creation::reserve(
                        data,
                        &context.working_directory,
                        deadline,
                    )?
                    .publish()?;
                    Self::register_first_root(context, data, root)
                }
            }
            Err(e) => Err(e),
            Ok(_) => {
                let client = std::fs::canonicalize(&logical)?;
                let index = Self::open(context, data, &client)?;
                let head = index.reserve_start()?;
                if Instant::now() >= deadline {
                    return Err(denied(
                        "client preparation registered after deadline; retained",
                    ));
                }
                Ok((index, head))
            }
        }
    }
    pub fn successor_eligible(
        &self,
        context: &ProcessContext,
        expected: &Head,
    ) -> io::Result<bool> {
        if self.head()?.as_ref() != Some(expected)
            || context.working_directory != self.profile
            || context.opencodex_home != self.context_home
        {
            return Err(denied("successor eligibility is not current context"));
        }
        let history = vault_history::read(self.connection()?, expected)?;
        successor_eligible_checked(
            context,
            self.data.path(),
            expected,
            &history,
            self.root_registration.as_ref(),
        )
    }
    pub fn record_successor_root_originals(
        &self,
        context: &ProcessContext,
        expected: &Head,
        refs: References,
    ) -> io::Result<Head> {
        if self
            .root_registration
            .as_ref()
            .is_none_or(|r| r.generation_slot.as_deref() != Some(r.id.as_str()))
        {
            return Err(denied(
                "successor Original registration lacks indexed generation",
            ));
        }
        self.record_root_originals(context, expected, refs)
    }
    /// Successor capture requires the ORIGINAL live creation fd and
    /// deadline. A facts-only reopen can resolve Preparing, but cannot mint
    /// this reservation from the provenance files or an empty pathname.
    pub fn reserve_successor_root_capture(&self, expected: &Head) -> io::Result<Head> {
        if self.head()?.as_ref() != Some(expected) {
            return Err(denied("successor capture source is not current"));
        }
        let origin = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("successor capture has no registered root"))?;
        let generation = expected
            .root_generation
            .as_ref()
            .ok_or_else(|| denied("successor capture has no initial generation event"))?;
        if expected.state != State::Preparing
            || expected.ticket != origin.id
            || origin.generation_slot.as_deref() != Some(origin.id.as_str())
            || generation.preparing_revision != expected.revision
            || generation.predecessor_revision.checked_add(1) != Some(expected.revision)
            || self.root_created_fd.is_none()
            || self
                .root_creation_deadline
                .is_none_or(|d| Instant::now() >= d)
            || !super::super::tree::names(self.client.file())?.is_empty()
        {
            return Err(denied(
                "successor capture lacks live empty-root creation custody",
            ));
        }
        root_binding::verify_initial_event(self.connection()?, origin)?;
        let next = self.reserve_capture_inner(expected)?;
        if self
            .root_creation_deadline
            .is_none_or(|d| Instant::now() >= d)
        {
            return Err(denied(
                "successor capture reservation expired; references retained",
            ));
        }
        Ok(next)
    }
    /// A successor-generation primitive. All retained historical
    /// facts are audited before creation; new origin/Preparing are committed
    /// together. No production Start calls this until capture/history rollover
    /// and current status eligibility are integrated and verified.
    pub fn create_successor_root(
        self,
        context: &ProcessContext,
        expected: &Head,
        deadline: Instant,
    ) -> io::Result<(Self, Head)> {
        self.create_successor_root_checked(context, expected, deadline, &|_| Ok(()))
    }
    fn create_successor_root_checked(
        mut self,
        context: &ProcessContext,
        expected: &Head,
        deadline: Instant,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<(Self, Head)> {
        let time = || {
            if Instant::now() >= deadline {
                Err(denied("successor root creation expired"))
            } else {
                Ok(())
            }
        };
        time()?;
        if expected.state != State::RootAbsent
            || self.head()?.as_ref() != Some(expected)
            || context.working_directory != self.profile
            || context.opencodex_home != self.context_home
            || self
                .root_registration
                .as_ref()
                .is_none_or(|r| r.id != expected.ticket)
        {
            return Err(denied(
                "successor root requires exact current closed generation",
            ));
        }
        if !self.successor_eligible(context, expected)? {
            return Err(denied(
                "current root is not eligible for successor creation",
            ));
        }
        if observation::entry(&PinnedDirectory::open(&self.profile)?, ".codex")?.is_some() {
            return Err(denied("new user root must not be adopted as successor"));
        }
        let history = vault_history::read(self.connection()?, expected)?;
        let previous = encode(expected)?;
        let authority = CreationAuthority {
            index: &self,
            closed: expected.clone(),
        };
        authority.check()?;
        checkpoint("predecessor_verified")?;
        time()?;
        let reservation = super::super::root_creation::reserve_generation(&authority, deadline)?;
        checkpoint("generation_reserved")?;
        let published = reservation.publish()?;
        checkpoint("generation_published")?;
        // The live Reservation now pins the exact old-plus-pending namespace.
        // Full predecessor evidence is revalidated below using that transfer,
        // not by pretending the new slot has already been committed.
        time()?;
        let client = PinnedDirectory::open(published.path())?;
        let transfer = published.into_generation_transfer(&self.data, &self.profile, &client)?;
        if (
            transfer.action_identity.clone(),
            transfer.action_protection.clone(),
        ) != (self.action_identity.clone(), self.action_protection.clone())
        {
            return Err(denied("successor lost original action custody"));
        }
        let binding = root_binding::encode_binding(
            &scope(context, &self.data, &client)?,
            Some(&transfer.references),
        )?;
        let revision = expected
            .revision
            .checked_add(1)
            .ok_or_else(|| denied("successor generation revision overflow"))?;
        let next = Head {
            revision,
            ticket: transfer.references.id.clone(),
            state: State::Preparing,
            references: None,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
            root_generation: Some(RootGeneration {
                preparing_revision: revision,
                predecessor_revision: expected.revision,
                predecessor_sha256: digest(&previous),
                binding,
            }),
        };
        validate(&next)?;
        self.check_storage()?;
        let c = self.connection()?;
        c.execute_batch("BEGIN IMMEDIATE").map_err(database_error)?;
        let transaction = (|| -> io::Result<()> {
            time()?;
            if read_head(c)?.as_ref() != Some(expected)
                || vault_history::read(c, expected)? != history
            {
                return Err(denied(
                    "successor predecessor changed before generation commit",
                ));
            }
            let stored: Vec<u8> = c
                .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                    r.get(0)
                })
                .map_err(database_error)?;
            if stored != self.binding {
                return Err(denied("successor original binding changed"));
            }
            vault_history::inspect_root_closures_pending(
                &self.data,
                &self.profile,
                &stored,
                &history,
                &transfer,
            )?;
            super::super::root_creation::verify_indexed(
                &self.action_parent,
                &self.data,
                &self.profile,
                &client,
                &transfer.references,
            )?;
            if !super::super::tree::names(client.file())?.is_empty()
                || identity(&stat(&transfer.created)?) != *client.identity()
            {
                return Err(denied(
                    "new root capture custody changed before independent registration",
                ));
            }
            checkpoint("before_generation_commit")?;
            time()?;
            super::super::root_creation::verify_indexed(
                &self.action_parent,
                &self.data,
                &self.profile,
                &client,
                &transfer.references,
            )?;
            if !super::super::tree::names(client.file())?.is_empty()
                || identity(&stat(&transfer.created)?) != *client.identity()
            {
                return Err(denied("successor root changed at commit boundary"));
            }
            let raw = encode(&next)?;
            c.execute(
                "INSERT INTO events(revision,document) VALUES(?1,?2)",
                params![revision, &raw],
            )
            .map_err(database_error)?;
            checkpoint("generation_event_inserted")?;
            if c.execute("UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1 AND revision=?3 AND document=?4",params![revision,&raw,expected.revision,&previous]).map_err(database_error)?!=1{return Err(denied("successor generation compare-and-swap failed"));}
            checkpoint("generation_head_updated")?;
            // Validate the WHOLE candidate and the old closure AFTER its
            // references are written but BEFORE COMMIT. Do not publish a
            // Preparing that adopted newly added user files or a changed old
            // archive. This uses the original live connection/action owner.
            if read_head(c)?.as_ref() != Some(&next) {
                return Err(denied("candidate successor Head changed before commit"));
            }
            let committed_binding: Vec<u8> = c
                .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                    r.get(0)
                })
                .map_err(database_error)?;
            if committed_binding != self.binding {
                return Err(denied(
                    "original binding changed during candidate transaction",
                ));
            }
            let (resolved, origin) = root_binding::resolve(
                context,
                &self.data,
                &self.action_parent,
                &self.binding,
                Some(&next),
                c,
            )?;
            if resolved.identity() != client.identity()
                || origin.as_ref() != Some(&transfer.references)
            {
                return Err(denied(
                    "successor candidate provenance changed before commit",
                ));
            }
            super::super::root_creation::verify_indexed(
                &self.action_parent,
                &self.data,
                &self.profile,
                &client,
                &transfer.references,
            )?;
            if !super::super::tree::names(client.file())?.is_empty()
                || identity(&stat(&transfer.created)?) != *client.identity()
            {
                return Err(denied(
                    "successor candidate lost empty root custody before commit",
                ));
            }
            self.action_parent
                .child_private("client-root-generations", false)?
                .sync()?;
            self.action_parent.sync()?;
            self.check_storage()?;
            time()?;
            Ok(())
        })();
        if let Err(e) = transaction {
            let _ = c.execute_batch("ROLLBACK");
            return Err(e);
        }
        c.execute_batch("COMMIT").map_err(database_error)?;
        self.client = client;
        self.root_registration = Some(transfer.references);
        self.root_created_fd = Some(transfer.created);
        self.root_creation_deadline = Some(transfer.deadline);
        // The duplicate action fd shares the ORIGINAL flock description. It is
        // not a reacquisition and is dropped only while Index keeps its owner.
        drop(transfer.action);
        checkpoint("generation_committed")?;
        if self.head()?.as_ref() != Some(&next) {
            return Err(denied("successor generation commit readback differs"));
        }
        if !super::super::tree::names(self.client.file())?.is_empty()
            || self.root_created_fd.as_ref().is_none_or(|fd| {
                stat(fd)
                    .map(|s| identity(&s) != *self.client.identity())
                    .unwrap_or(true)
            })
        {
            return Err(denied(
                "successor committed root changed before returning custody; references retained",
            ));
        }
        time()?;
        Ok((self, next))
    }
}
/// Private projection of ALREADY verified independent root/history facts. It
/// creates no object and is shared by held-Index Start and readonly status.
pub(super) fn successor_eligible_checked(
    context: &ProcessContext,
    data: &Path,
    head: &Head,
    history: &[Head],
    origin: Option<&super::root_creation::References>,
) -> io::Result<bool> {
    if head.state != State::RootAbsent {
        return Ok(false);
    }
    let origin = origin.ok_or_else(|| denied("closed-root eligibility lacks indexed origin"))?;
    if history.last() != Some(head)
        || head.ticket != origin.id
        || head
            .references
            .as_ref()
            .is_none_or(|r| r.attempt != origin.id)
    {
        return Err(denied("closed-root eligibility source changed"));
    }
    let profile = PinnedDirectory::open(&context.working_directory)?;
    if observation::entry(&profile, ".codex")?.is_some() {
        return Err(denied("successor root already exists; not adopted"));
    }
    let manager = PinnedDirectory::open(data)?.child_private("manager-state", false)?;
    origin.verify_namespace(&manager)?;
    let generations = history
        .iter()
        .filter(|h| h.state == State::Preparing)
        .count();
    // Preflight the actual serialized upper-bound shapes of one complete
    // normal root cycle, using this exact context and maximal root integer
    // widths. No global reduced quota replaces the existing 4MiB limit.
    let total = history.iter().try_fold(0usize, |n, h| {
        n.checked_add(encode(h)?.len())
            .ok_or_else(|| denied("eligibility history overflow"))
    })?;
    let future = successor_cycle_capacity(context, data, head, origin)?;
    if !successor_capacity_fits(history.len(), total, generations, &future) {
        return Ok(false);
    }
    profile.recheck()?;
    if observation::entry(&profile, ".codex")?.is_some() {
        return Err(denied("successor root appeared during eligibility check"));
    }
    origin.verify_namespace(&manager)?;
    Ok(true)
}
fn successor_capacity_fits(
    events: usize,
    bytes: usize,
    generations: usize,
    future: &[usize],
) -> bool {
    generations < 512
        && events.checked_add(future.len()).is_some_and(|n| n <= 4096)
        && future.iter().all(|size| *size <= 65536)
        && future
            .iter()
            .try_fold(bytes, |n, size| n.checked_add(*size))
            .is_some_and(|n| n <= 4 * 1024 * 1024)
}
fn successor_cycle_capacity(
    context: &ProcessContext,
    data: &Path,
    closed: &Head,
    origin: &super::root_creation::References,
) -> io::Result<Vec<usize>> {
    let movement = closed
        .root_retirement
        .as_ref()
        .ok_or_else(|| denied("closed capacity source missing movement"))?;
    let refs = closed
        .references
        .as_ref()
        .ok_or_else(|| denied("closed capacity Original missing"))?;
    let pinned = PinnedDirectory::open(data)?;
    let archived = pinned
        .child_private("manager-state", false)?
        .child_private("client-originals", false)?
        .child_private(&refs.attempt, false)?
        .child_private(&format!("root-retired-{}", movement.plan.id), false)?;
    let mut tuple: serde_json::Value = serde_json::from_slice(&scope_at(
        context,
        &pinned,
        &archived,
        &context.working_directory.join(".codex"),
    )?)
    .map_err(|_| denied("capacity scope invalid"))?;
    tuple[8] = serde_json::to_value(Identity {
        device: i32::MIN,
        inode: u64::MAX,
        birth_seconds: i64::MIN,
        birth_nanos: i64::MIN,
    })
    .map_err(|_| denied("capacity root identity invalid"))?;
    let ticket = "ffffffff-ffff-4fff-bfff-ffffffffffff".to_string();
    let mut next_origin = origin.clone();
    next_origin.id = ticket.clone();
    next_origin.generation_slot = Some(ticket.clone());
    next_origin.intent_sha256 = "f".repeat(64);
    next_origin.created_sha256 = "f".repeat(64);
    next_origin.published_sha256 = "f".repeat(64);
    let binding = root_binding::encode_binding(
        &serde_json::to_vec(&tuple).map_err(|_| denied("capacity encoding failed"))?,
        Some(&next_origin),
    )?;
    let revision = closed
        .revision
        .checked_add(1)
        .ok_or_else(|| denied("capacity revision overflow"))?;
    let mut next = Head {
        revision,
        ticket: ticket.clone(),
        state: State::Preparing,
        references: None,
        restore_plan: None,
        completion_sha256: None,
        root_retirement: None,
        root_generation: Some(RootGeneration {
            preparing_revision: revision,
            predecessor_revision: closed.revision,
            predecessor_sha256: "f".repeat(64),
            binding,
        }),
    };
    let mut sizes = vec![encode(&next)?.len()];
    for state in [
        State::CaptureReserved,
        State::Captured,
        State::Ready,
        State::StopPrepared,
        State::Stopped,
        State::RestorePrepared,
        State::RootRetirementPrepared,
        State::RootAbsent,
    ] {
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| denied("capacity revision overflow"))?;
        next.root_generation = None;
        next.state = state;
        if state == State::Captured {
            next.references = Some(References {
                attempt: ticket.clone(),
                original_sha256: "f".repeat(64),
                scope_sha256: "f".repeat(64),
                phases: Vec::new(),
                live_sha256: None,
            });
        }
        if let Some(refs) = &mut next.references {
            for (required, phase) in [
                (State::Ready, Phase::Ready),
                (State::StopPrepared, Phase::PreStop),
                (State::Stopped, Phase::Stopped),
            ] {
                if state == required {
                    refs.phases.push((phase, "f".repeat(64)));
                    refs.live_sha256 = Some("f".repeat(64));
                }
            }
        }
        if state == State::RestorePrepared {
            next.restore_plan = Some(RestorePlan {
                id: ticket.clone(),
                sha256: "f".repeat(64),
            });
        }
        if state == State::RootRetirementPrepared {
            next.root_retirement = Some(RootRetirement {
                source_revision: next.revision - 1,
                plan: RestorePlan {
                    id: ticket.clone(),
                    sha256: "f".repeat(64),
                },
            });
        }
        if state == State::RootAbsent {
            next.completion_sha256 = Some("f".repeat(64));
        }
        validate(&next)?;
        sizes.push(encode(&next)?.len());
    }
    Ok(sizes)
}
/// No new writable Index or lock is acquired for the status projection. The
/// observer keeps its original shared action and SQLite view through callback.
pub fn inspect_successor_eligibility(context: &ProcessContext, data: &Path) -> io::Result<bool> {
    observation::inspect_map_history_root(context, data, |head, history, origin| match head {
        Some(head) => successor_eligible_checked(context, data, head, history, origin),
        None => Ok(false),
    })
}
pub fn inspect_client_recovery(
    context: &ProcessContext,
    data: &Path,
) -> io::Result<crate::modules::status::ClientRecoveryState> {
    use crate::modules::status::ClientRecoveryState as Recovery;
    observation::inspect_map_history_root(context, data, |head, history, origin| {
        Ok(match head {
            None => Recovery::NotRequired,
            Some(h) if h.state == State::Ready => Recovery::Active,
            Some(h) if h.state == State::RootAbsent => {
                if successor_eligible_checked(context, data, h, history, origin)? {
                    Recovery::NotRequired
                } else {
                    Recovery::Required
                }
            }
            Some(h)
                if matches!(
                    h.state,
                    State::Restored
                        | State::Unchanged
                        | State::CancelledBeforeCapture
                        | State::CancelledCapture
                ) =>
            {
                Recovery::NotRequired
            }
            Some(_) => Recovery::Required,
        })
    })
}
#[cfg(test)]
#[path = "macos_client_root_successor_capture_tests.rs"]
mod capture_tests;
#[cfg(test)]
#[path = "macos_client_root_rollover_tests.rs"]
mod rollover_tests;
#[cfg(test)]
#[path = "macos_client_root_start_tests.rs"]
mod start_tests;
#[cfg(test)]
#[path = "macos_client_root_generation_tests.rs"]
mod tests;
