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
#[cfg(test)]
#[path = "macos_client_root_successor_capture_tests.rs"]
mod capture_tests;
#[cfg(test)]
#[path = "macos_client_root_rollover_tests.rs"]
mod rollover_tests;
#[cfg(test)]
#[path = "macos_client_root_generation_tests.rs"]
mod tests;
