//! Actual original initializer resources retained by the original CaptureSlot.
//! No receipt/JSON constructor or fallback from failed initialization exists.
use super::*;
mod read;
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct JournalPreparation {
    directory: Option<PinnedDirectory>,
    pins: Option<[PinnedDirectory; 4]>,
    peer: Option<RetainedPeer>,
    binding: Option<Binding>,
    action: Option<crate::infrastructure::windows_official_lifecycle024::NativeAction>,
    plan: Option<StoragePlan>,
    predecessors: Vec<super::super::predecessors::Predecessor>,
    manifest: Option<Manifest>,
    manifest_raw: Option<Vec<u8>>,
    manifest_open: Option<File>,
    manifest_record: Option<Record>,
    current_open: Option<File>,
    current_expected: Option<Vec<u8>>,
    completed: Option<Journal>,
    attempted: bool,
    initialization_started: bool,
    ready: bool,
    failed: bool,
    #[cfg(test)]
    fault: u8,
}
impl JournalPreparation {
    pub(in crate::infrastructure::windows_client_originals024) fn retained_v2_membership_data(
        &self,
    ) -> serde_json::Value {
        // A finite projection of actual enrolled resource presence. Expected
        // bytes/old snapshots are deliberately not labeled current verified SD
        // or content; incomplete preparation stays Pending in the V2 producer.
        serde_json::json!({"attempted":self.attempted,"initializationStarted":self.initialization_started,"ready":self.ready,"failed":self.failed,
            "directory":self.directory.is_some(),"pins":self.pins.as_ref().map(|p|p.len()),"peer":self.peer.is_some(),"binding":self.binding.is_some(),"action":self.action.is_some(),
            "plan":self.plan.is_some(),"predecessors":self.predecessors.len(),"manifest":self.manifest.is_some(),"manifestExpected":self.manifest_raw.is_some(),
            "manifestOriginalHandle":self.manifest_open.is_some(),"manifestRecord":self.manifest_record.is_some(),"currentOriginalHandle":self.current_open.is_some(),"currentExpected":self.current_expected.is_some(),"completeOriginalJournal":self.completed.is_some()})
    }
    fn enroll(
        &mut self,
        directory: PinnedDirectory,
        pins: [PinnedDirectory; 4],
        peer: RetainedPeer,
        binding: Binding,
        plan: StoragePlan,
        predecessors: Vec<super::super::predecessors::Predecessor>,
        action: Option<crate::infrastructure::windows_official_lifecycle024::NativeAction>,
    ) -> io::Result<()> {
        if self.attempted || self.directory.is_some() || self.completed.is_some() {
            return Err(invalid());
        }
        self.attempted = true;
        self.failed = true;
        self.directory = Some(directory);
        self.pins = Some(pins);
        self.peer = Some(peer);
        self.binding = Some(binding);
        self.plan = Some(plan);
        self.predecessors = predecessors;
        self.action = action;
        Ok(())
    }
    fn checkpoint(&self, stage: u8, end: Instant) -> io::Result<()> {
        before(end)?;
        #[cfg(test)]
        if self.fault == stage {
            return Err(io::Error::other(format!(
                "injected original journal initializer stage{stage}"
            )));
        }
        #[cfg(not(test))]
        let _ = stage;
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn take_complete(
        &mut self,
    ) -> io::Result<Journal> {
        if !self.ready || self.failed {
            return Err(invalid());
        }
        self.completed.take().ok_or_else(invalid)
    }
    fn poison(&mut self) {
        self.ready = false;
        self.failed = true;
        if let Some(journal) = &mut self.completed {
            journal.poisoned = true;
        }
    }
    fn initialize(
        &mut self,
        epoch: &str,
        claim: &str,
        scope: &str,
        end: Instant,
    ) -> io::Result<()> {
        let result = self.initialize_inner(epoch, claim, scope, end);
        if result.is_err() {
            self.poison();
        }
        result
    }
    fn initialize_inner(
        &mut self,
        epoch: &str,
        claim: &str,
        scope: &str,
        end: Instant,
    ) -> io::Result<()> {
        if self.initialization_started {
            return Err(invalid());
        }
        self.initialization_started = true;
        self.checkpoint(1, end)?;
        let token = native::current_identity()?;
        if token.elevated || token.elevation_type == 2 {
            return Err(invalid());
        }
        let pins = self.pins.as_ref().ok_or_else(invalid)?;
        let binding = self.binding.as_ref().ok_or_else(invalid)?;
        binding.validate(Some(&token.sid))?;
        binding.check_native(self.peer.as_ref().ok_or_else(invalid)?, pins)?;
        super::super::predecessors::validate_set(&self.predecessors, binding)?;
        self.plan
            .as_ref()
            .ok_or_else(invalid)?
            .validate(pins[0].path(), pins[0].identity())?;
        let directory = self.directory.as_ref().ok_or_else(invalid)?;
        directory.recheck()?;
        let namespace_session = session_namespace(directory, &pins[1])?;
        if std::fs::read_dir(directory.path())?
            .next()
            .transpose()?
            .is_some()
        {
            return Err(invalid());
        }
        let session = namespace_session.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let binding_digest = hash::sha256_hex(&encoded(binding)?);
        self.manifest = Some(Manifest {
            predecessors: std::mem::take(&mut self.predecessors),
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "manifest".into(),
            session_id: session.clone(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            binding: self.binding.take().unwrap(),
            security_policy: Policy::fixed(),
            security_policy_digest: Policy::fixed().digest()?,
            storage_plan_digest: self.plan.as_ref().unwrap().digest()?,
            storage_plan: self.plan.take().unwrap(),
            initial_lease_epoch: epoch.into(),
            claim_key: claim.into(),
            deadline_scope: scope.into(),
        });
        self.manifest_raw = Some(encoded(self.manifest.as_ref().unwrap())?);
        if self.manifest_raw.as_ref().unwrap().len() > 1024 * 1024 {
            return Err(invalid());
        }
        let path = self
            .directory
            .as_ref()
            .unwrap()
            .path()
            .join("manifest.json");
        self.manifest_open = Some(native::create_private_journal_file(&path)?);
        self.checkpoint(2, end)?;
        let file = self.manifest_open.as_mut().unwrap();
        let id = identity(file)?;
        native::verify_private_file(file)?;
        tree::streams(&path, false)?;
        #[cfg(test)]
        if self.fault == 10 {
            let expected = self.manifest_raw.as_ref().unwrap();
            file.write_all(&expected[..expected.len() / 2])?;
            file.sync_all()?;
            return Err(io::Error::other(
                "injected partially written original manifest",
            ));
        }
        file.write_all(self.manifest_raw.as_ref().unwrap())?;
        file.sync_all()?;
        self.checkpoint(3, end)?;
        let mut bytes = Vec::new();
        let file = self.manifest_open.as_mut().unwrap();
        file.seek(SeekFrom::Start(0))?;
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes != *self.manifest_raw.as_ref().unwrap()
            || identity(self.manifest_open.as_ref().unwrap())? != id
        {
            return Err(invalid());
        }
        let manifest_security = security(self.manifest_open.as_ref().unwrap(), end)?;
        let parent = PinnedDirectory::open_private(self.directory.as_ref().unwrap().path())?;
        self.manifest_record = Some(Record {
            file: RecordFile::Original {
                file: self.manifest_open.take().unwrap(),
                parent,
                id,
                max: 1024 * 1024,
            },
            path,
            raw: self.manifest_raw.take().unwrap(),
            security: manifest_security,
            entry: None,
        });
        self.manifest_record.as_mut().unwrap().check(end)?;
        self.checkpoint(4, end)?;
        let progress = Progress::new(
            &self.manifest.as_ref().unwrap().storage_plan,
            &session,
            &hash::sha256_hex(&self.manifest_record.as_ref().unwrap().raw),
            &binding_digest,
            epoch,
            claim,
            scope,
        )?;
        let tip = Current {
            schema: ANCHORED_CURRENT_SCHEMA_V3.into(),
            session_id: session,
            sequence: 0,
            event_digest: progress.tip.clone(),
            previous_event_digest: progress.tip.clone(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            storage_plan_digest: progress.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: progress.manifest.clone(),
            binding_digest,
            lease_epoch: epoch.into(),
            claim_key: claim.into(),
            deadline_scope: scope.into(),
            lease_state: LeaseState::Active,
            lease_objects_digest: None,
            work_mode: super::super::observations::WorkMode::Sealed,
            work_frames: None,
            observed_frames: None,
            snapshot_pending: None,
            snapshot_links: std::collections::BTreeMap::new(),
            observed_sequence: 0,
            linked_frames: std::collections::BTreeMap::new(),
            lifecycle: super::super::snapshots::LifecyclePhase::Preparing,
            ready_generation: None,
            action_pending: None,
            action_kind: super::super::actions::ActionKind::Start,
            recovery: None,
            history: None,
            handles: None,
            batch: None,
            content: None,
            closed: None,
            route_close: None,
        };
        let raw = encoded(&tip)?;
        if raw.is_empty() || raw.len() > 4096 {
            return Err(invalid());
        }
        self.current_expected = Some(raw.clone());
        let path = self.directory.as_ref().unwrap().path().join("current.json");
        self.current_open = Some(native::create_private_journal_file(&path)?);
        self.checkpoint(5, end)?;
        let current = self.current_open.as_mut().unwrap();
        let current_id = identity(current)?;
        native::verify_private_file(current)?;
        tree::streams(&path, false)?;
        #[cfg(test)]
        if self.fault == 11 {
            current.write_all(&raw[..raw.len() / 2])?;
            current.sync_all()?;
            return Err(io::Error::other(
                "injected partially written original current",
            ));
        }
        current.write_all(&raw)?;
        current.sync_all()?;
        self.checkpoint(6, end)?;
        let current = self.current_open.as_mut().unwrap();
        if contents(current)? != raw || identity(current)? != current_id {
            return Err(invalid());
        }
        let current_security = security(self.current_open.as_ref().unwrap(), end)?;
        let directory_security = directory_security(self.directory.as_ref().unwrap(), end)?;
        self.checkpoint(7, end)?;
        self.completed = Some(Journal {
            directory: self.directory.take().unwrap(),
            directory_security,
            pins: self.pins.take().unwrap(),
            peer: self.peer.take().unwrap(),
            token,
            manifest: self.manifest.take().unwrap(),
            manifest_record: self.manifest_record.take().unwrap(),
            events: Vec::new(),
            progress,
            current: self.current_open.take().unwrap(),
            current_id,
            current_security,
            current_raw: self.current_expected.take().unwrap(),
            deadline: end,
            poisoned: false,
            released_lease: None,
            previous_releases: Vec::new(),
            source_action: self.action.take(),
            pending_source_action: None,
            old_source_actions: Vec::new(),
            original_handoff: None,
            close_inventory: None,
            retired_lease_histories: Vec::new(),
            close_completion: None,
            stop_completion: None,
            native_batches: Vec::new(),
            #[cfg(test)]
            fault: 0,
        });
        self.checkpoint(8, end)?;
        self.completed.as_mut().unwrap().check(end)?;
        Ok(())
    }
}
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024) fn create_from_source_retained(
        directory: PinnedDirectory,
        context: super::super::super::source::NativeJournalContext,
        plan: StoragePlan,
        predecessors: &super::super::NativePredecessors,
        claim: &mut super::super::super::coordination::AnchoredClaim<'_>,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        end: Instant,
        prefix: &mut JournalPreparation,
    ) -> io::Result<()> {
        let (pins, peer, binding, action) = context.into_parts();
        prefix.enroll(
            directory,
            pins,
            peer,
            binding,
            plan,
            Vec::new(),
            Some(action),
        )?;
        let result = (|| {
            prefix.predecessors = predecessors.data()?;
            let pins = prefix.pins.as_ref().unwrap();
            if session_namespace(prefix.directory.as_ref().unwrap(), &pins[1])?.is_none() {
                return Err(invalid());
            }
            let action = prefix.action.as_ref().unwrap();
            action.check(owner, end).map_err(io::Error::other)?;
            if action.deadline() != end {
                return Err(invalid());
            }
            let root = pins[0].path().join(".codex");
            claim.check(&root, end)?;
            let epoch = claim.epoch().to_string();
            let key = claim.key();
            let scope = action.scope_id();
            prefix.initialize(&epoch, &key, &scope, end)?;
            prefix
                .completed
                .as_mut()
                .unwrap()
                .check_source_owner(owner, end)?;
            claim.check(&root, end)?;
            prefix.checkpoint(9, end)?;
            prefix.ready = true;
            prefix.failed = false;
            Ok(())
        })();
        if result.is_err() {
            prefix.poison();
        }
        result
    }
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;
