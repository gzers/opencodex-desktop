//! One immutable v3 manifest/event chain and exact exclusive current object.
//! Data-only replay never rebuilds the live tree/lease/native phase. No v2 read,
//! migration, Ready, Closed or recovery capability factory is exposed here.
use super::super::{binding::Binding, bounded::native_stamp, tree};
use super::segments::{ChainEntry, LeaseState};
use super::*;
use crate::infrastructure::windows_official_runtime024::{
    self as native, PinnedDirectory, PrivateRecord, RetainedPeer, TokenIdentity,
};
use crate::modules::runtime::windows_lifecycle024_contract::{
    ANCHORED_CURRENT_SCHEMA_V3, ANCHORED_ORIGINALS_SCHEMA_V3,
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::PathBuf,
    time::Instant,
};
mod actions;
mod before_resume;
mod preparation;
pub(in super::super) use preparation::JournalPreparation;
mod batches;
mod closed_stop;
mod completed;
pub(in crate::infrastructure::windows_client_originals024) use closed_stop::CompletedStopGuard;
mod driver;
mod handles;
mod history;
mod leases;
mod observations;
mod predecessors;
mod production;
pub(in crate::infrastructure::windows_client_originals024) use production::ProductionStop;
mod recovery;
pub(in super::super) use history::HistoryContext;
mod snapshots;
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn identity(file: &File) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if i.dwFileAttributes & 0x410 != 0 || i.nNumberOfLinks != 1 {
        return Err(invalid());
    }
    Ok(FileIdentity {
        volume: i.dwVolumeSerialNumber,
        index: ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64,
    })
}
fn observe(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .access_mode(0x20080)
        .share_mode(1 | 2)
        .custom_flags(0x200000)
        .open(path)
}
fn security(file: &File, end: Instant) -> io::Result<SnapshotData> {
    native_stamp(file, end).map_err(io::Error::other)
}
fn directory_security(pin: &PinnedDirectory, end: Instant) -> io::Result<SnapshotData> {
    let file = OpenOptions::new()
        .access_mode(0x20080)
        .share_mode(1 | 2)
        .custom_flags(0x2200000)
        .open(pin.path())?;
    native::verify_private_directory(&file)?;
    security(&file, end)
}
fn session_namespace(
    directory: &PinnedDirectory,
    data: &PinnedDirectory,
) -> io::Result<Option<String>> {
    data.recheck()?;
    directory.recheck()?;
    // Direct child is the existing isolated COMPONENT journal layout. Native
    // Source factory below requires the exact production namespace instead.
    if directory.path().parent() == Some(data.path()) {
        return Ok(None);
    }
    let expected = data.path().join("official024/windows");
    if directory.path().parent() != Some(expected.as_path()) {
        return Err(invalid());
    }
    let official = PinnedDirectory::open_private(&data.path().join("official024"))?;
    let windows = PinnedDirectory::open_private(&expected)?;
    let session = directory
        .path()
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(invalid)?;
    if !uuid(session) {
        return Err(invalid());
    }
    official.recheck()?;
    windows.recheck()?;
    directory.recheck()?;
    data.recheck()?;
    Ok(Some(session.into()))
}
fn contents(file: &mut File) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(0))?;
    let mut out = Vec::new();
    file.take(4097).read_to_end(&mut out)?;
    if out.is_empty() || out.len() > 4096 {
        return Err(invalid());
    }
    Ok(out)
}
enum RecordFile {
    Sealed(PrivateRecord),
    // The actual CREATE_NEW descriptor remains retained; its READ-only share
    // mask denies any external writer/deleter. After publication this private
    // wrapper exposes only identity/read, never write or a mutable raw handle.
    Original {
        file: File,
        parent: PinnedDirectory,
        id: FileIdentity,
        max: usize,
    },
}
impl RecordFile {
    fn identity(&self) -> FileIdentity {
        match self {
            Self::Sealed(f) => f.identity(),
            Self::Original { id, .. } => *id,
        }
    }
    fn read(&mut self) -> io::Result<Vec<u8>> {
        match self {
            Self::Sealed(file) => file.read(),
            Self::Original {
                file,
                parent,
                id,
                max,
            } => {
                parent.recheck()?;
                native::verify_private_file(file)?;
                if identity(file)? != *id || file.metadata()?.len() > *max as u64 {
                    return Err(invalid());
                }
                file.seek(SeekFrom::Start(0))?;
                let mut bytes = Vec::new();
                file.take(*max as u64 + 1).read_to_end(&mut bytes)?;
                if bytes.len() > *max {
                    return Err(invalid());
                }
                Ok(bytes)
            }
        }
    }
}
struct Record {
    file: RecordFile,
    path: PathBuf,
    raw: Vec<u8>,
    security: SnapshotData,
    // Immutable pure DATA derived from exactly raw=encoded(event) at local
    // creation. Native ID/all8/raw-file checks still run on every observation.
    entry: Option<ChainEntry>,
}
impl Record {
    fn create(path: &Path, raw: Vec<u8>, end: Instant) -> io::Result<Self> {
        before(end)?;
        let mut file = PrivateRecord::create(path, &raw, 1024 * 1024)?;
        if file.read()? != raw {
            return Err(invalid());
        }
        let observed = observe(path)?;
        if identity(&observed)? != file.identity() {
            return Err(invalid());
        }
        let out = Self {
            file: RecordFile::Sealed(file),
            path: path.into(),
            raw,
            security: security(&observed, end)?,
            entry: None,
        };
        before(end)?;
        Ok(out)
    }
    fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        let fresh = observe(&self.path)?;
        tree::streams(&self.path, false)?;
        if identity(&fresh)? != self.file.identity()
            || security(&fresh, end)? != self.security
            || self.file.read()? != self.raw
        {
            return Err(invalid());
        }
        before(end)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    predecessors: Vec<super::predecessors::Predecessor>,
    schema: String,
    document_kind: String,
    session_id: String,
    storage_strategy: String,
    binding: Binding,
    security_policy: Policy,
    security_policy_digest: String,
    storage_plan: StoragePlan,
    storage_plan_digest: String,
    initial_lease_epoch: String,
    claim_key: String,
    deadline_scope: String,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Current {
    schema: String,
    session_id: String,
    sequence: u32,
    event_digest: String,
    previous_event_digest: String,
    storage_strategy: String,
    storage_plan_digest: String,
    security_policy_digest: String,
    manifest_digest: String,
    binding_digest: String,
    lease_epoch: String,
    claim_key: String,
    deadline_scope: String,
    lease_state: LeaseState,
    lease_objects_digest: Option<String>,
    work_mode: super::observations::WorkMode,
    work_frames: Option<[String; 2]>,
    observed_frames: Option<[String; 2]>,
    snapshot_pending: Option<(super::snapshots::SnapshotPurpose, u32)>,
    snapshot_links: std::collections::BTreeMap<super::snapshots::SnapshotPurpose, [String; 2]>,
    observed_sequence: u32,
    linked_frames: std::collections::BTreeMap<u32, [String; 2]>,
    lifecycle: super::snapshots::LifecyclePhase,
    ready_generation: Option<(u32, u16, String)>,
    action_pending: Option<(String, super::actions::ActionKind)>,
    action_kind: super::actions::ActionKind,
    recovery: Option<super::recovery::RecoverySummary>,
    history: Option<super::history::HistorySummary>,
    handles: Option<super::handles::HandleSummary>,
    batch: Option<super::batches::BatchSummary>,
    content: Option<super::qualification::ContentSummary>,
    closed: Option<super::closed::ClosedSummary>,
    route_close: Option<super::route_close::RouteCloseSummary>,
}
/// Native handles keep records/current/peer/directory bindings alive. This is
/// a component journal, deliberately NOT NativeSession or production adapter.
pub(in super::super) struct Journal {
    directory: PinnedDirectory,
    directory_security: SnapshotData,
    pins: [PinnedDirectory; 4],
    peer: RetainedPeer,
    token: TokenIdentity,
    manifest: Manifest,
    manifest_record: Record,
    events: Vec<Record>,
    progress: Progress,
    current: File,
    current_id: FileIdentity,
    current_security: SnapshotData,
    current_raw: Vec<u8>,
    deadline: Instant,
    poisoned: bool,
    released_lease: Option<super::super::coordination::ReleasedLease>,
    previous_releases: Vec<super::super::coordination::ReleasedLease>,
    source_action: Option<crate::infrastructure::windows_official_lifecycle024::NativeAction>,
    pending_source_action:
        Option<crate::infrastructure::windows_official_lifecycle024::NativeAction>,
    old_source_actions: Vec<crate::infrastructure::windows_official_lifecycle024::NativeAction>,
    original_handoff: Option<super::super::tree::OriginalHandleHandoff>,
    close_inventory: Option<super::super::handle_release::ExpectedHandleInventory>,
    retired_lease_histories: Vec<super::super::coordination::RetiredLeaseHistory>,
    close_completion: Option<completed::CompletedOriginals>,
    stop_completion: Option<completed::CompletedStop>,
    native_batches: Vec<batches::RetainedNativeBatch>,
    #[cfg(test)]
    fault: u8,
}
/// Live-journal context for snapshot DATA only. Private construction, no serde
/// recovery/phase capability; synthetic component journal stays non-production.
pub(in super::super) struct ObservationContext {
    session: String,
    plan: String,
    binding: String,
    manifest: String,
    deadline_scope: String,
    frames: [String; 2],
    purpose: super::snapshots::SnapshotPurpose,
    observation_sequence: u32,
}
impl ObservationContext {
    pub(in super::super) fn session(&self) -> &str {
        &self.session
    }
    pub(in super::super) fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"sessionId":self.session,"storagePlanDigest":self.plan,"bindingDigest":self.binding,"manifestDigest":self.manifest,"deadlineScope":self.deadline_scope,"frameDigests":self.frames,"purpose":self.purpose,"observationSequence":self.observation_sequence})
    }
    pub(in super::super) fn purpose_leaf(&self) -> String {
        if self.purpose == super::snapshots::SnapshotPurpose::Observation {
            format!("{}-{:06}", self.purpose.leaf(), self.observation_sequence)
        } else {
            self.purpose.leaf().into()
        }
    }
    pub(in super::super) fn matches(&self, root: &Path, frame: &str) -> bool {
        let index = match root.file_name().and_then(|n| n.to_str()) {
            Some(".codex") => 0,
            Some(".grok") => 1,
            _ => return false,
        };
        self.frames[index] == frame
    }
}
/// Content observation locators bound to this actual journal/claim only. They
/// do not say the observations have production Ready/PreStop/NormalExit proof.
pub(in super::super) struct ObservationSeriesContext {
    bindings: [serde_json::Value; 3],
    sequences: [u32; 3],
    frames: [[String; 2]; 3],
}
impl ObservationSeriesContext {
    pub(in super::super) fn binding(&self, stage: usize) -> io::Result<&serde_json::Value> {
        self.bindings.get(stage).ok_or_else(invalid)
    }
    pub(in super::super) fn sequences(&self) -> [u32; 3] {
        self.sequences
    }
    pub(in super::super) fn frame(&self, stage: usize, root: usize) -> io::Result<&str> {
        self.frames
            .get(stage)
            .and_then(|v| v.get(root))
            .map(String::as_str)
            .ok_or_else(invalid)
    }
}
impl Journal {
    #[cfg(test)]
    pub(in super::super) fn inspect_recovery_observation_for_test(
        &mut self,
        roots: [serde_json::Value; 2],
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if claim.epoch() != self.progress.epoch
            || claim.key() != self.progress.claim
            || self.progress.lease_state != LeaseState::Active
        {
            return Err(invalid());
        }
        super::recovery::RecoveryPlan::from_native_data(roots)?.validate_for_purpose(
            &self.manifest.storage_plan,
            &self.progress,
            super::snapshots::SnapshotPurpose::Observation,
        )?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        self.check(end)
    }
    pub(in super::super) fn observation_series_context(
        &mut self,
        prior: [u32; 2],
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<ObservationSeriesContext> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.work_mode != super::observations::WorkMode::Observed
            || self.progress.lease_state != LeaseState::Active
            || claim.epoch() != self.progress.epoch
            || claim.key() != self.progress.claim
            || prior[0] >= prior[1]
            || prior[1] >= self.progress.observed_sequence
        {
            return Err(invalid());
        }
        let sequences = [prior[0], prior[1], self.progress.observed_sequence];
        let mut frames = Vec::new();
        for sequence in &sequences {
            frames.push(
                self.progress
                    .linked_frames
                    .get(sequence)
                    .cloned()
                    .ok_or_else(invalid)?,
            );
        }
        let mut bindings = Vec::new();
        for sequence in &sequences {
            let mut matches = Vec::new();
            for record in &self.events {
                let entry: ChainEntry =
                    serde_json::from_slice(&record.raw).map_err(|_| invalid())?;
                if let ChainEntry::Snapshot(event) = entry {
                    if event.operation
                        == super::snapshots::SnapshotOperation::SnapshotLinkedVerified
                        && event.observation_sequence == *sequence
                    {
                        let refs = event.references.as_ref().ok_or_else(invalid)?;
                        matches.push(refs[0].context_metadata()?);
                    }
                }
            }
            if matches.len() != 1 {
                return Err(invalid());
            }
            bindings.push(matches.pop().ok_or_else(invalid)?);
        }
        Ok(ObservationSeriesContext {
            bindings: bindings.try_into().map_err(|_| invalid())?,
            sequences,
            frames: frames.try_into().map_err(|_| invalid())?,
        })
    }
    #[cfg(test)]
    pub(in super::super) fn observed_sequence_for_test(&self) -> u32 {
        self.progress.observed_sequence
    }
    pub(in super::super) fn observation_context(
        &mut self,
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<ObservationContext> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.work_mode != super::observations::WorkMode::Observed
            || self.progress.lease_state != LeaseState::Active
            || self.progress.epoch != claim.epoch()
            || self.progress.claim != claim.key()
        {
            return Err(invalid());
        }
        Ok(ObservationContext {
            session: self.progress.session.clone(),
            plan: self.progress.plan.clone(),
            binding: self.progress.binding.clone(),
            manifest: self.progress.manifest.clone(),
            deadline_scope: self.progress.deadline.clone(),
            frames: self.progress.observed_frames.clone().ok_or_else(invalid)?,
            purpose: super::snapshots::SnapshotPurpose::Observation,
            observation_sequence: self.progress.observed_sequence,
        })
    }
    /// Exact native source/Owner ActionWindow context only. This factory stays
    /// below the unchanged public Start gate until complete v3 adapter/Closed.
    pub(in super::super) fn create_from_source(
        directory: PinnedDirectory,
        context: super::super::source::NativeJournalContext,
        plan: StoragePlan,
        predecessors: &super::NativePredecessors,
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        end: Instant,
    ) -> io::Result<Self> {
        let (pins, peer, binding, action) = context.into_parts();
        if session_namespace(&directory, &pins[1])?.is_none() {
            return Err(invalid());
        }
        action.check(owner, end).map_err(io::Error::other)?;
        if action.deadline() != end {
            return Err(invalid());
        }
        let root = pins[0].path().join(".codex");
        claim.check(&root, end)?;
        let epoch = claim.epoch().to_string();
        let key = claim.key();
        let scope = action.scope_id();
        let references = predecessors.data()?;
        super::predecessors::validate_set(&references, &binding)?;
        let mut out = Self::create_with_predecessor_data(
            directory, pins, peer, binding, plan, references, &epoch, &key, &scope, end,
        )?;
        out.source_action = Some(action);
        if let Err(e) = out.check_source_owner(owner, end) {
            out.poisoned = true;
            return Err(e);
        }
        claim.check(&root, end)?;
        Ok(out)
    }
    pub(in super::super) fn check_source_owner(
        &mut self,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check(end)?;
            let action = self.source_action.as_ref().ok_or_else(invalid)?;
            if action.scope_id() != self.progress.deadline || action.deadline() != end {
                return Err(invalid());
            }
            action.check(owner, end).map_err(io::Error::other)?;
            self.manifest.binding.validate(Some(&self.token.sid))?;
            if self.manifest.binding.paths()
                != [
                    owner.profile_path(),
                    owner.data_path(),
                    owner.home_path(),
                    owner.installation_path(),
                ]
            {
                return Err(invalid());
            }
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in super::super) fn check_live_ready_read(
        &mut self,
        source: &mut super::super::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            before(end)?;
            if self.poisoned
                || self.pending_source_action.is_some()
                || self.progress.lifecycle != super::snapshots::LifecyclePhase::Ready
                || self.progress.lease_state != LeaseState::Released
                || self.progress.work_mode != super::observations::WorkMode::Open
                || self.progress.action_pending.is_some()
                || self.progress.snapshot_pending.is_some()
                || self.progress.batch.is_some()
                || self.progress.recovery.is_some()
                || self.progress.content.is_some()
                || self.progress.closed.is_some()
                || self.progress.ready_generation.as_ref()
                    != Some(&(ready.pid(), ready.port(), ready.record_sha256().into()))
                || !self
                    .progress
                    .snapshot_links
                    .contains_key(&super::snapshots::SnapshotPurpose::Ready)
            {
                return Err(invalid());
            }
            let action = self.source_action.as_ref().ok_or_else(invalid)?;
            if action.scope_id()!=self.progress.deadline || action.kind()!=crate::infrastructure::windows_official_lifecycle024::OriginalActionKind::Start {return Err(invalid());}
            action
                .check_retained_owner_for_read(owner, end)
                .map_err(io::Error::other)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            if source.binding_raw != encoded(&self.manifest.binding)? {
                return Err(invalid());
            }
            self.check_records(end)?;
            before(end)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    pub(in super::super) fn create(
        directory: PinnedDirectory,
        pins: [PinnedDirectory; 4],
        peer: RetainedPeer,
        binding: Binding,
        plan: StoragePlan,
        epoch: &str,
        claim: &str,
        scope: &str,
        end: Instant,
    ) -> io::Result<Self> {
        Self::create_with_predecessor_data(
            directory,
            pins,
            peer,
            binding,
            plan,
            Vec::new(),
            epoch,
            claim,
            scope,
            end,
        )
    }
    fn create_with_predecessor_data(
        directory: PinnedDirectory,
        pins: [PinnedDirectory; 4],
        peer: RetainedPeer,
        binding: Binding,
        plan: StoragePlan,
        predecessors: Vec<super::predecessors::Predecessor>,
        epoch: &str,
        claim: &str,
        scope: &str,
        end: Instant,
    ) -> io::Result<Self> {
        before(end)?;
        let token = native::current_identity()?;
        if token.elevated || token.elevation_type == 2 {
            return Err(invalid());
        }
        binding.validate(Some(&token.sid))?;
        binding.check_native(&peer, &pins)?;
        super::predecessors::validate_set(&predecessors, &binding)?;
        plan.validate(pins[0].path(), pins[0].identity())?;
        directory.recheck()?;
        let namespace_session = session_namespace(&directory, &pins[1])?;
        if std::fs::read_dir(directory.path())?.next().is_some() {
            return Err(invalid());
        }
        let session = namespace_session.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let binding_digest = hash::sha256_hex(&encoded(&binding)?);
        let manifest = Manifest {
            predecessors,
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "manifest".into(),
            session_id: session.clone(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            binding,
            security_policy: Policy::fixed(),
            security_policy_digest: Policy::fixed().digest()?,
            storage_plan_digest: plan.digest()?,
            storage_plan: plan,
            initial_lease_epoch: epoch.into(),
            claim_key: claim.into(),
            deadline_scope: scope.into(),
        };
        let manifest_record = Record::create(
            &directory.path().join("manifest.json"),
            encoded(&manifest)?,
            end,
        )?;
        let progress = Progress::new(
            &manifest.storage_plan,
            &session,
            &hash::sha256_hex(&manifest_record.raw),
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
            work_mode: super::observations::WorkMode::Sealed,
            work_frames: None,
            observed_frames: None,
            snapshot_pending: None,
            snapshot_links: std::collections::BTreeMap::new(),
            observed_sequence: 0,
            linked_frames: std::collections::BTreeMap::new(),
            lifecycle: super::snapshots::LifecyclePhase::Preparing,
            ready_generation: None,
            action_pending: None,
            action_kind: super::actions::ActionKind::Start,
            recovery: None,
            history: None,
            handles: None,
            batch: None,
            content: None,
            closed: None,
            route_close: None,
        };
        let raw = encoded(&tip)?;
        let path = directory.path().join("current.json");
        let seed = PrivateRecord::create(&path, b"", 16)?;
        let current_id = seed.identity();
        drop(seed);
        let mut current = OpenOptions::new()
            .access_mode(0xc0020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&path)?;
        if identity(&current)? != current_id {
            return Err(invalid());
        }
        native::verify_private_file(&current)?;
        before(end)?;
        current.write_all(&raw)?;
        current.sync_all()?;
        let current_security = security(&current, end)?;
        let directory_security = directory_security(&directory, end)?;
        let mut out = Self {
            directory,
            directory_security,
            pins,
            peer,
            token,
            manifest,
            manifest_record,
            events: Vec::new(),
            progress,
            current,
            current_id,
            current_security,
            current_raw: raw,
            deadline: end,
            poisoned: false,
            released_lease: None,
            previous_releases: Vec::new(),
            source_action: None,
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
        };
        out.check(end)?;
        Ok(out)
    }
    fn check_inner(&mut self, end: Instant) -> io::Result<Current> {
        let authorized_end = self
            .pending_source_action
            .as_ref()
            .map(|a| a.deadline())
            .unwrap_or(self.deadline);
        if self.poisoned || end > authorized_end {
            return Err(invalid());
        }
        before(end)?;
        before(authorized_end)?;
        self.check_records(end)
    }
    // Shared exact native comparisons. Mutation callers enter ONLY through
    // check_inner's original action cutoff; historical entry additionally
    // requires a retained timely completion object before and after reading.
    fn check_records(&mut self, end: Instant) -> io::Result<Current> {
        if self.poisoned {
            return Err(invalid());
        }
        #[cfg(test)]
        let timing = super::super::recovery_timing::readback("journal-records", end, || {
            let bytes = self.manifest_record.raw.len() + self.current_raw.len()
                + self.events.iter().map(|event| event.raw.len()).sum::<usize>();
            (self.events.len() + 2, bytes)
        });
        let result = self.read_retained_records(end);
        #[cfg(test)]
        if let Some(timing) = timing {
            timing.finish(&result);
        }
        result
    }
    // Exact immutable native observations, not permission to mutate a failed
    // chain. Ordinary/historical callers still enter via check_records.
    fn read_retained_records(&mut self, end: Instant) -> io::Result<Current> {
        before(end)?;
        if native::current_identity()? != self.token {
            return Err(invalid());
        }
        self.directory.recheck()?;
        self.manifest.binding.check_native(&self.peer, &self.pins)?;
        for release in &mut self.previous_releases {
            release.readback_history(end)?;
        }
        for history in &self.retired_lease_histories {
            history.check(end)?;
        }
        if directory_security(&self.directory, end)? != self.directory_security {
            return Err(invalid());
        }
        self.manifest_record.check(end)?;
        for record in &mut self.events {
            record.check(end)?;
        }
        let current_path = self.directory.path().join("current.json");
        let fresh = observe(&current_path)?;
        tree::streams(&current_path, false)?;
        native::verify_private_file(&self.current)?;
        if identity(&fresh)? != self.current_id
            || identity(&self.current)? != self.current_id
            || security(&self.current, end)? != self.current_security
            || security(&fresh, end)? != self.current_security
            || contents(&mut self.current)? != self.current_raw
        {
            return Err(invalid());
        }
        let tip: Current = serde_json::from_slice(&self.current_raw).map_err(|_| invalid())?;
        if tip.schema != ANCHORED_CURRENT_SCHEMA_V3
            || tip.sequence != self.progress.sequence
            || tip.event_digest != self.progress.tip
            || tip.storage_plan_digest != self.progress.plan
            || tip.manifest_digest != self.progress.manifest
            || tip.binding_digest != self.progress.binding
            || tip.session_id != self.progress.session
            || tip.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || tip.security_policy_digest != Policy::fixed().digest()?
            || tip.lease_epoch != self.progress.epoch
            || tip.claim_key != self.progress.claim
            || tip.deadline_scope != self.progress.deadline
            || tip.lease_state != self.progress.lease_state
            || tip.lease_objects_digest != self.progress.lease_objects
            || tip.work_mode != self.progress.work_mode
            || tip.work_frames != self.progress.work_frames
            || tip.observed_frames != self.progress.observed_frames
            || tip.snapshot_pending != self.progress.snapshot_pending
            || tip.snapshot_links != self.progress.snapshot_links
            || tip.observed_sequence != self.progress.observed_sequence
            || tip.linked_frames != self.progress.linked_frames
            || tip.lifecycle != self.progress.lifecycle
            || tip.ready_generation != self.progress.ready_generation
            || tip.action_pending != self.progress.action_pending
            || tip.action_kind != self.progress.action_kind
            || tip.recovery != self.progress.recovery_summary()?
            || tip.history != self.progress.history_summary()?
            || tip.handles != self.progress.handle_progress
            || tip.batch != self.progress.batch_summary()
            || tip.content != self.progress.content
            || tip.closed != self.progress.closed
            || tip.route_close != self.progress.route_close
        {
            return Err(invalid());
        }
        let mut names = vec!["manifest.json".to_string(), "current.json".to_string()];
        names.extend((1..=self.events.len()).map(|n| format!("event-{n:06}.json")));
        names.sort();
        let mut actual = std::fs::read_dir(self.directory.path())?
            .map(|e| e.and_then(|e| e.file_name().into_string().map_err(|_| invalid())))
            .collect::<io::Result<Vec<_>>>()?;
        actual.sort();
        if actual != names {
            return Err(invalid());
        }
        before(end)?;
        Ok(tip)
    }
    pub(in super::super) fn check(&mut self, end: Instant) -> io::Result<()> {
        self.checked_current(end).map(drop)
    }
    fn checked_current(&mut self, end: Instant) -> io::Result<Current> {
        let r = self.check_inner(end);
        if r.is_err() {
            self.poisoned = true;
        }
        r
    }
    pub(in super::super) fn append(&mut self, event: Event, end: Instant) -> io::Result<()> {
        let r = self.append_inner(event, end);
        if r.is_err() {
            self.poisoned = true;
        }
        r
    }
    fn append_inner(&mut self, event: Event, end: Instant) -> io::Result<()> {
        self.append_entry_inner(ChainEntry::Node(event), end)
    }
    fn append_entry_inner(&mut self, event: ChainEntry, end: Instant) -> io::Result<()> {
        // The old tip comes from this append's full native check. Keep the
        // separate disk comparison below before overwriting current.
        let old = self.checked_current(end)?;
        // Validate against an independent replay before any durable write.
        let mut next = Progress::new(
            &self.manifest.storage_plan,
            &self.progress.session,
            &self.progress.manifest,
            &self.progress.binding,
            &self.manifest.initial_lease_epoch,
            &self.manifest.claim_key,
            &self.manifest.deadline_scope,
        )?;
        for record in &self.events {
            // checked_current above independently reobserved exact raw bytes,
            // native identity/all8/membership of EVERY record. The locally
            // owned typed event cannot change and is the same encoded value;
            // replay all original progression rules without decoding it again.
            let event = record.entry.as_ref().ok_or_else(invalid)?;
            next.apply_entry(&self.manifest.storage_plan, event)?;
        }
        next.apply_entry(&self.manifest.storage_plan, &event)?;
        if self.events.len() >= 4096 {
            return Err(invalid());
        }
        let mut record = Record::create(
            &self
                .directory
                .path()
                .join(format!("event-{:06}.json", next.sequence)),
            encoded(&event)?,
            end,
        )?;
        record.entry = Some(event);
        // Retain the actual published event BEFORE any fallible current IO.
        self.events.push(record);
        self.poisoned = true;
        #[cfg(test)]
        if self.fault == 1 {
            return Err(io::Error::other(
                "injected after immutable event sync/readback before current CAS",
            ));
        }
        let tip = Current {
            schema: ANCHORED_CURRENT_SCHEMA_V3.into(),
            session_id: old.session_id,
            sequence: next.sequence,
            event_digest: next.tip.clone(),
            previous_event_digest: old.event_digest,
            storage_strategy: old.storage_strategy,
            storage_plan_digest: old.storage_plan_digest,
            security_policy_digest: old.security_policy_digest,
            manifest_digest: old.manifest_digest,
            binding_digest: old.binding_digest,
            lease_epoch: next.epoch.clone(),
            claim_key: next.claim.clone(),
            deadline_scope: next.deadline.clone(),
            lease_state: next.lease_state,
            lease_objects_digest: next.lease_objects.clone(),
            work_mode: next.work_mode,
            work_frames: next.work_frames.clone(),
            observed_frames: next.observed_frames.clone(),
            snapshot_pending: next.snapshot_pending,
            snapshot_links: next.snapshot_links.clone(),
            observed_sequence: next.observed_sequence,
            linked_frames: next.linked_frames.clone(),
            lifecycle: next.lifecycle,
            ready_generation: next.ready_generation.clone(),
            action_pending: next.action_pending.clone(),
            action_kind: next.action_kind,
            recovery: next.recovery_summary()?,
            history: next.history_summary()?,
            handles: next
                .handle_progress
                .as_ref()
                .map(|h| serde_json::from_slice(&encoded(h)?).map_err(|_| invalid()))
                .transpose()?,
            batch: next.batch_summary(),
            content: next.content.clone(),
            closed: next.closed.clone(),
            route_close: next.route_close.clone(),
        };
        let raw = encoded(&tip)?;
        if contents(&mut self.current)? != self.current_raw
            || identity(&self.current)? != self.current_id
            || security(&self.current, end)? != self.current_security
        {
            return Err(invalid());
        }
        before(end)?;
        self.current.seek(SeekFrom::Start(0))?;
        #[cfg(test)]
        if self.fault == 2 {
            self.current.write_all(&raw[..1])?;
            self.current.set_len(1)?;
            self.current.sync_all()?;
            return Err(io::Error::other("injected real partial current write/sync"));
        }
        self.current.write_all(&raw)?;
        self.current.set_len(raw.len() as u64)?;
        self.current.sync_all()?;
        if contents(&mut self.current)? != raw
            || identity(&self.current)? != self.current_id
            || security(&self.current, end)? != self.current_security
        {
            return Err(invalid());
        }
        before(end)?;
        self.current_raw = raw;
        self.progress = next;
        self.poisoned = false;
        self.check(end)
    }
}
#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
