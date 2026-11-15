//! Production ClientOriginals adapter. Native state stays in this adapter when
//! consumed phase tags fail; JSON/phase text never recreates this state. Public
//! Start remains separately refused until the complete integration is validated.
use super::super::bounded::{native_stamp, BoundedAccessV1};
use super::capture::TreeCaptureV2;
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        windows_diagnostic_roles024::NormalExit,
        windows_official_lifecycle024::Owner,
        windows_official_runtime024::RetainedPeer,
        windows_supervisor024::{Absent, WindowsSupervisor},
    },
    modules::runtime::{
        official_lifecycle024::{ActionWindow, ServiceObservation, StopAccepted},
        program_image,
        program_selection::{self, CurrentSelection},
        windows_lifecycle024_contract::{
            ClientOriginals, NativeSession, SupervisorAbsence, OFFICIAL_PACKAGE_SHA256,
        },
    },
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::windows::fs::OpenOptionsExt,
    time::Instant,
};
fn refused(reason: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_originals_required".into(),
        detail: format!("原件持久恢复未确认；保留原代次/记录/句柄：{reason}"),
    }
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
struct CurrentPin {
    file: File,
    security: BoundedAccessV1,
    raw: Vec<u8>,
}
impl CurrentPin {
    fn bytes(file: &mut File) -> io::Result<Vec<u8>> {
        file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        file.take(65537).read_to_end(&mut raw)?;
        if raw.is_empty() || raw.len() > 65536 {
            return Err(invalid());
        }
        Ok(raw)
    }
    fn capture(path: &Path, expected: &CurrentSelection, end: Instant) -> io::Result<Self> {
        before(end)?;
        let security = BoundedAccessV1::capture(path, false, end).map_err(io::Error::other)?;
        let mut file = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(path)?;
        let raw = Self::bytes(&mut file)?;
        let observed: CurrentSelection = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        if observed != *expected
            || native_stamp(&file, end).map_err(io::Error::other)? != security.snapshot_data()?
        {
            return Err(invalid());
        }
        let mut out = Self {
            file,
            security,
            raw,
        };
        out.check(end)?;
        Ok(out)
    }
    fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.security.recheck(end).map_err(io::Error::other)?;
        if Self::bytes(&mut self.file)? != self.raw
            || native_stamp(&self.file, end).map_err(io::Error::other)?
                != self.security.snapshot_data()?
        {
            return Err(invalid());
        }
        before(end)
    }
}
struct Source {
    broker: RetainedPeer,
    pins: [PinnedDirectory; 4],
    current: CurrentPin,
    owner_scope: String,
    binding_raw: Vec<u8>,
    image_digest: String,
}
impl Source {
    fn capture(owner: &mut Owner, end: Instant) -> AppResult<(Self, Binding)> {
        owner.originals_require_unresumed(end)?;
        let broker = RetainedPeer::current().map_err(refused)?;
        broker.no_outer_job().map_err(refused)?;
        broker
            .verify_image(&owner.installation_path().join("ocxd.exe"))
            .map_err(refused)?;
        let token = &broker.identity().token;
        if token.elevated
            || token.elevation_type == 2
            || token != &owner.original_native().binding().process.token
        {
            return Err(refused("原普通broker与原launcher令牌不匹配"));
        }
        let selected = owner.selected_program();
        if selected.selection.version != "2.99.0"
            || selected.selection.platform != "win32-x64"
            || selected.image.official_archive_sha256 != OFFICIAL_PACKAGE_SHA256
        {
            return Err(refused("固定官方2.99来源不匹配"));
        }
        program_image::verify(selected)?;
        let current = CurrentPin::capture(
            &owner.installation_path().join("runtime/current.json"),
            &selected.selection,
            end,
        )
        .map_err(refused)?;
        let image_digest = selected.selection.image_sha256.clone();
        let pins = [
            PinnedDirectory::open(owner.profile_path()).map_err(refused)?,
            PinnedDirectory::open_private(owner.data_path()).map_err(refused)?,
            PinnedDirectory::open(owner.home_path()).map_err(refused)?,
            PinnedDirectory::open(owner.installation_path()).map_err(refused)?,
        ];
        let binding = Binding::from_native(
            &broker,
            &pins,
            image_digest.clone(),
            hash::sha256_hex(&current.raw),
        )
        .map_err(refused)?;
        let binding_raw = serde_json::to_vec(&binding).map_err(refused)?;
        let owner_scope = owner.scope()?;
        let mut out = Self {
            broker,
            pins,
            current,
            owner_scope,
            binding_raw,
            image_digest,
        };
        out.check(owner, end)?;
        Ok((out, binding))
    }
    fn check(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        owner.check_current(end)?;
        self.broker.no_outer_job().map_err(refused)?;
        self.broker
            .verify_image(&owner.installation_path().join("ocxd.exe"))
            .map_err(refused)?;
        let selected = owner.selected_program();
        if selected.selection.image_sha256 != self.image_digest
            || selected.image.official_archive_sha256 != OFFICIAL_PACKAGE_SHA256
            || owner.scope()? != self.owner_scope
        {
            return Err(refused("原owner/source代次变化"));
        }
        program_selection::verify_current(owner.installation_path(), selected)?;
        program_image::verify(selected)?;
        self.current.check(end).map_err(refused)?;
        let binding = Binding::from_native(
            &self.broker,
            &self.pins,
            self.image_digest.clone(),
            hash::sha256_hex(&self.current.raw),
        )
        .map_err(refused)?;
        if serde_json::to_vec(&binding).map_err(refused)? != self.binding_raw {
            return Err(refused("原SID/Profile/D/H/I/current绑定变化"));
        }
        owner.check_current(end)
    }
}
#[derive(PartialEq, Eq)]
enum State {
    Original,
    Ready,
    Prepared,
    Closed,
    Consumed,
}
struct Store {
    tree: TreeCaptureV2,
    source: Source,
    state: State,
    ready: Option<(u32, u16, String)>,
    stop_deadline: Option<Instant>,
    failed: bool,
}
// These tags locate retained native state in THIS adapter only. No public
// fields/serde/Clone or constructor exists; a record cannot create a tag.
pub(crate) struct OriginalSession {
    session: String,
}
pub(crate) struct ReadySession {
    session: String,
}
pub(crate) struct PreparedRestore {
    session: String,
}
pub(crate) struct ClosedEligibility {
    session: String,
}
#[derive(Default)]
pub(crate) struct WindowsOriginals {
    store: Option<Store>,
}
impl WindowsOriginals {
    fn begin(
        &mut self,
        session: &str,
        state: State,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<&mut Store> {
        let store = self
            .store
            .as_mut()
            .ok_or_else(|| refused("原生session缺席"))?;
        if store.failed || store.state != state || store.tree.session_id() != session {
            return Err(refused("旧tag/错误阶段/未知或poisoned session"));
        }
        store.failed = true;
        store.source.check(owner, end)?;
        store.tree.recheck(end).map_err(refused)?;
        Ok(store)
    }
}
impl ClientOriginals for WindowsOriginals {
    type Owner = Owner;
    type OriginalSession = OriginalSession;
    type ReadySession = ReadySession;
    type PreparedRestore = PreparedRestore;
    type NativeNormalExit = NormalExit;
    type SupervisorAbsent = Absent;
    type ClosedEligibility = ClosedEligibility;
    fn capture(&mut self, owner: &mut Owner, window: &ActionWindow) -> AppResult<OriginalSession> {
        window.remaining()?;
        if self.store.is_some() {
            return Err(refused("已有原生session；不得替换/自动追赶"));
        }
        let end = window.deadline();
        let (source, binding) = Source::capture(owner, end)?;
        let tree = TreeCaptureV2::capture(binding, end).map_err(refused)?;
        if tree.binding_bytes().map_err(refused)? != source.binding_raw {
            return Err(refused("capture/current不是原source绑定"));
        }
        let session = tree.session_id().to_string();
        // Install BEFORE the first root move. Failure below preserves all the
        // original handles/custody/journal in the adapter, not a consumed tag.
        self.store = Some(Store {
            tree,
            source,
            state: State::Original,
            ready: None,
            stop_deadline: None,
            failed: true,
        });
        let store = self.store.as_mut().unwrap();
        store.tree.create_working_trees(end).map_err(refused)?;
        owner.originals_require_unresumed(end)?;
        store.source.check(owner, end)?;
        store.failed = false;
        Ok(OriginalSession { session })
    }
    fn record_ready(
        &mut self,
        original: OriginalSession,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> AppResult<ReadySession> {
        let store = self.begin(&original.session, State::Original, owner, end)?;
        owner.originals_check_ready(ready, end)?;
        // Do not grant authority to unknown official/external client changes.
        // Declared side-effect validation is a remaining production requirement.
        store.tree.verify_working_unchanged(end).map_err(refused)?;
        store
            .tree
            .advance_phase(Phase::Ready, end)
            .map_err(refused)?;
        store.source.check(owner, end)?;
        store.ready = Some((ready.pid(), ready.port(), ready.record_sha256().into()));
        store.state = State::Ready;
        store.failed = false;
        Ok(ReadySession {
            session: original.session,
        })
    }
    fn pre_stop(
        &mut self,
        ready: ReadySession,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<PreparedRestore> {
        let store = self.begin(&ready.session, State::Ready, owner, end)?;
        let (pid, port, digest) = store
            .ready
            .as_ref()
            .ok_or_else(|| refused("原typed Ready缺席"))?;
        owner.originals_check_live_record(*pid, *port, digest, end)?;
        store.tree.verify_working_unchanged(end).map_err(refused)?;
        store
            .tree
            .advance_phase(Phase::PreparedRestore, end)
            .map_err(refused)?;
        // OfficialTeardown is an intention, persisted before the actual POST.
        store
            .tree
            .advance_phase(Phase::OfficialTeardown, end)
            .map_err(refused)?;
        store.source.check(owner, end)?;
        store.stop_deadline = Some(end);
        store.state = State::Prepared;
        store.failed = false;
        Ok(PreparedRestore {
            session: ready.session,
        })
    }
    fn complete_stop(
        &mut self,
        prepared: PreparedRestore,
        owner: &mut Owner,
        native_exit: NormalExit,
        supervisor: Absent,
        official: &StopAccepted,
        end: Instant,
    ) -> AppResult<ClosedEligibility> {
        let store = self.begin(&prepared.session, State::Prepared, owner, end)?;
        if store.stop_deadline != Some(end)
            || store.ready.as_ref()
                != Some(&(
                    official.pid(),
                    official.port(),
                    official.record_sha256().into(),
                ))
        {
            return Err(refused("Stop原窗口/typed接受代次不匹配"));
        }
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.tree.verify_working_unchanged(end).map_err(refused)?;
        store.tree.restore_after_teardown(end).map_err(refused)?;
        // Native predicates are reobserved AFTER restoration, inside the same
        // absolute Stop window. Phase text alone is never a predicate.
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.source.check(owner, end)?;
        store
            .tree
            .advance_phase(Phase::Closed, end)
            .map_err(refused)?;
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.source.check(owner, end)?;
        store.state = State::Closed;
        store.failed = false;
        Ok(ClosedEligibility {
            session: prepared.session,
        })
    }
    fn authorize_successor(
        &mut self,
        closed: ClosedEligibility,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        window.remaining()?;
        let end = window.deadline();
        let store = self.begin(&closed.session, State::Closed, owner, end)?;
        // The new action does not reuse a previous Stop proof/deadline.
        let absent = WindowsSupervisor.observe_absent(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &absent, end)?;
        owner.originals_check_stopped_records(end)?;
        store.tree.recheck(end).map_err(refused)?;
        store.source.check(owner, end)?;
        store.state = State::Consumed;
        store.failed = false;
        // Retain closed original records/objects; no migration or automatic
        // deletion. A later native capture needs explicit successor dispatch.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tree::tests::Fixture;
    use super::*;
    use crate::infrastructure::windows_official_runtime024::create_private_directory;
    fn end() -> Instant {
        Instant::now() + std::time::Duration::from_secs(10)
    }
    #[test]
    fn wl2_originals_current_pin_retains_actual_record_and_refuses_foreign_selection() {
        let fixture = Fixture::new();
        let dir = create_private_directory(&fixture.path().join("I")).unwrap();
        let path = dir.path().join("current.json");
        let expected = CurrentSelection {
            schema_version: 1,
            version: "2.99.0".into(),
            platform: "win32-x64".into(),
            image_sha256: "a".repeat(64),
        };
        let raw = serde_json::to_vec(&expected).unwrap();
        drop(PrivateRecord::create(&path, &raw, 65536).unwrap());
        let mut pin = CurrentPin::capture(&path, &expected, end()).unwrap();
        pin.check(end()).unwrap();
        assert!(std::fs::write(&path, b"tamper").is_err());
        assert!(std::fs::rename(&path, dir.path().join("foreign.json")).is_err());
        let other = CurrentSelection {
            image_sha256: "b".repeat(64),
            ..expected
        };
        assert!(CurrentPin::capture(&path, &other, end()).is_err());
        assert!(pin.check(Instant::now()).is_err());
        println!("actual source-current record bytes/ID/8projections pinned, mutation/rename/foreign selection/expired deadline refused; synthetic selection only, not production Source/Owner/Closed proof");
    }
    #[test]
    fn wl2_originals_current_pin_unknown_schema_and_hardlink_never_admit_source() {
        let fixture = Fixture::new();
        let dir = create_private_directory(&fixture.path().join("I")).unwrap();
        let path = dir.path().join("current.json");
        let expected = CurrentSelection {
            schema_version: 1,
            version: "2.99.0".into(),
            platform: "win32-x64".into(),
            image_sha256: "a".repeat(64),
        };
        let mut raw = serde_json::to_value(&expected).unwrap();
        raw["sourceAuthority"] = serde_json::json!(true);
        std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(CurrentPin::capture(&path, &expected, end()).is_err());
        std::fs::write(&path, serde_json::to_vec(&expected).unwrap()).unwrap();
        std::fs::hard_link(&path, dir.path().join("alias.json")).unwrap();
        assert!(CurrentPin::capture(&path, &expected, end()).is_err());
        println!("unknown source field and multiple native links refused; no adapter/native phase tag issued");
    }
}
