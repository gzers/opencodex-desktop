//! Production ClientOriginals adapter. Native state stays in this adapter when
//! consumed phase tags fail; JSON/phase text never recreates this state. Public
//! Start remains separately refused until the complete integration is validated.
use super::super::bounded::{native_stamp, BoundedAccessV1};
use super::capture::TreeCaptureV2;
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        windows_diagnostic_roles024::{ClosedExitHistory, NormalExit},
        windows_official_lifecycle024::Owner,
        windows_official_runtime024::RetainedPeer,
        windows_supervisor024::{Absent, ClosedAbsentHistory, WindowsSupervisor},
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
        if expected.schema_version != 1
            || expected.version != "2.99.0"
            || expected.platform != "win32-x64"
            || !digest(&expected.image_sha256)
        {
            return Err(invalid());
        }
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
}
struct Store {
    tree: TreeCaptureV2,
    source: Source,
    state: State,
    ready: Option<(u32, u16, String)>,
    stop_deadline: Option<Instant>,
    closed_exit: Option<ClosedExitHistory>,
    closed_supervisor: Option<ClosedAbsentHistory>,
    failed: bool,
    routes: Option<super::routes::RoutePin>,
}
impl Store {
    fn check_source(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        self.source.check(owner, end)?;
        if self.state != State::Closed {
            self.routes
                .as_mut()
                .ok_or_else(|| refused("本代独立路由原件缺席"))?
                .declared(owner.home_path(), end)
                .map_err(refused)?;
        }
        before(end).map_err(refused)
    }
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
pub(crate) enum LivePhase {
    Original(OriginalSession),
    Ready(ReadySession),
    Prepared(PreparedRestore),
    Closed(ClosedEligibility),
}
#[derive(Default)]
pub(crate) struct WindowsOriginals {
    store: Option<Store>,
    closed: Vec<Store>,
    successor: Option<SuccessorTicket>,
    failed: bool,
}
// Memory-only, one consumed original ClosedEligibility and exact ActionWindow.
// Matching strings never create this ticket or the retained native histories.
struct SuccessorTicket {
    deadline: Instant,
    stop_deadline: Instant,
    binding: Vec<u8>,
    previous_session: String,
}
impl WindowsOriginals {
    /// Borrow the original Closed tag; no boolean/serialized status may request
    /// a successful close. This is a fresh native observation in THIS window.
    pub(crate) fn check_closed(
        &mut self,
        closed: &ClosedEligibility,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        self.recheck_closed(owner, end)?;
        let result = (|| {
            let store = self
                .store
                .as_mut()
                .ok_or_else(|| refused("原闭合Store缺席"))?;
            if store.failed
                || store.state != State::Closed
                || store.tree.session_id() != closed.session
            {
                return Err(refused("原闭合tag/Store代次或阶段不成立"));
            }
            store.failed = true;
            store.check_source(owner, end)?;
            store
                .closed_exit
                .as_ref()
                .ok_or_else(|| refused("原退出历史缺席"))?
                .check(owner, end)?;
            store
                .closed_supervisor
                .as_ref()
                .ok_or_else(|| refused("原监督缺席历史缺席"))?
                .check(owner, end)?;
            let absent = WindowsSupervisor.observe_absent(owner, end)?;
            WindowsSupervisor.recheck_absent(owner, &absent, end)?;
            owner.originals_check_stopped_records(end)?;
            store.tree.recheck(end).map_err(refused)?;
            store.check_source(owner, end)?;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
            return result;
        }
        self.recheck_closed(owner, end)
    }
    fn recheck_closed(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        if self.failed {
            return Err(refused("原件历史adapter已poison；不得自动重试"));
        }
        let result = (|| {
            before(end).map_err(refused)?;
            let before_start = self
                .store
                .as_ref()
                .is_none_or(|store| store.state == State::Closed)
                || !owner.originals_creator_has_resumed();
            let previous = owner.originals_previous_owners();
            if previous.len() != self.closed.len() {
                return Err(refused("原生历史Owner集合与原件集合不匹配"));
            }
            for (store, old) in self.closed.iter_mut().zip(previous) {
                if store.failed || store.state != State::Closed || !old.originals_is_stopped() {
                    return Err(refused("旧Owner/原件没有准确原闭合资格"));
                }
                store.check_source(old, end)?;
                store
                    .closed_exit
                    .as_ref()
                    .ok_or_else(|| refused("旧原始退出历史缺席"))?
                    .check(old, end)?;
                store
                    .closed_supervisor
                    .as_ref()
                    .ok_or_else(|| refused("旧监督缺席历史缺席"))?
                    .check(old, end)?;
                if before_start {
                    let absent = WindowsSupervisor.observe_absent(old, end)?;
                    WindowsSupervisor.recheck_absent(old, &absent, end)?;
                    old.originals_check_stopped_records(end)?;
                }
                store.tree.recheck(end).map_err(refused)?;
                store.check_source(old, end)?;
            }
            let mut histories = self
                .closed
                .iter_mut()
                .map(|store| &mut store.tree)
                .collect::<Vec<_>>();
            if let Some(current) = &mut self.store {
                if current.failed {
                    return Err(refused("当前原件Store已poison"));
                }
                current
                    .tree
                    .recheck_with_history(&mut histories, end)
                    .map_err(refused)?;
            } else if let Some(last) = histories.last() {
                let raw = last.binding_bytes().map_err(refused)?;
                let binding: Binding = serde_json::from_slice(&raw).map_err(refused)?;
                let windows =
                    PinnedDirectory::open_private(&binding.paths()[1].join("official024/windows"))
                        .map_err(refused)?;
                TreeCaptureV2::native_history_set(&binding, &mut histories, &windows, None, end)
                    .map_err(refused)?;
            }
            before(end).map_err(refused)
        })();
        if result.is_err() {
            self.failed = true;
            if let Some(store) = &mut self.store {
                store.failed = true;
            }
        }
        result
    }
    pub(crate) fn before_resume(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        self.recheck_closed(owner, end)?;
        let result = (|| {
            let store = self.store.as_mut().ok_or_else(|| refused("原始捕获缺席"))?;
            if store.failed || store.state != State::Original {
                return Err(refused("不是未Resume原捕获阶段"));
            }
            store.failed = true;
            owner.originals_require_unresumed(end)?;
            store.check_source(owner, end)?;
            store.tree.verify_working_unchanged(end).map_err(refused)?;
            owner.originals_require_unresumed(end)?;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
            return result;
        }
        self.recheck_closed(owner, end)
    }
    pub(crate) fn check_ready(
        &mut self,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> AppResult<()> {
        self.recheck_closed(owner, end)?;
        let store = self
            .store
            .as_mut()
            .ok_or_else(|| refused("原生Ready session缺席"))?;
        if store.failed || store.state != State::Ready {
            return Err(refused("原生Ready阶段未知"));
        }
        store.failed = true;
        store.check_source(owner, end)?;
        owner.originals_check_ready(ready, end)?;
        if store.ready.as_ref() != Some(&(ready.pid(), ready.port(), ready.record_sha256().into()))
        {
            return Err(refused("Ready原代次变化"));
        }
        store.tree.verify_working_unchanged(end).map_err(refused)?;
        store.failed = false;
        self.recheck_closed(owner, end)
    }
    fn begin(
        &mut self,
        session: &str,
        state: State,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<(&mut Store, &mut [Store])> {
        self.recheck_closed(owner, end)?;
        let store = self
            .store
            .as_mut()
            .ok_or_else(|| refused("原生session缺席"))?;
        if store.failed || store.state != state || store.tree.session_id() != session {
            return Err(refused("旧tag/错误阶段/未知或poisoned session"));
        }
        store.failed = true;
        store.check_source(owner, end)?;
        store.tree.recheck(end).map_err(refused)?;
        Ok((store, &mut self.closed))
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
        self.recheck_closed(owner, end)?;
        let successor = if self.closed.is_empty() {
            if self.successor.is_some() {
                return Err(refused("首代出现后继ticket"));
            }
            None
        } else {
            Some(
                self.successor
                    .take()
                    .ok_or_else(|| refused("已消费或缺席后继动作ticket"))?,
            )
        };
        let (source, binding) = Source::capture(owner, end)?;
        let routes = super::routes::RoutePin::capture(owner.home_path(), end).map_err(refused)?;
        if let Some(ticket) = successor {
            if ticket.deadline != end
                || ticket.stop_deadline != window.stop_deadline()
                || ticket.binding != source.binding_raw
                || self
                    .closed
                    .last()
                    .is_none_or(|old| old.tree.session_id() != ticket.previous_session)
            {
                return Err(refused("后继ticket不是本次原动作/上下文/前驱"));
            }
        }
        let mut histories = self
            .closed
            .iter_mut()
            .map(|old| &mut old.tree)
            .collect::<Vec<_>>();
        let tree =
            TreeCaptureV2::capture_with_history(binding, &mut histories, end).map_err(refused)?;
        drop(histories);
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
            closed_exit: None,
            closed_supervisor: None,
            failed: true,
            routes: Some(routes),
        });
        let store = self.store.as_mut().unwrap();
        let mut histories = self
            .closed
            .iter_mut()
            .map(|old| &mut old.tree)
            .collect::<Vec<_>>();
        store
            .tree
            .create_working_trees_with_history(&mut histories, end)
            .map_err(refused)?;
        owner.originals_require_unresumed(end)?;
        store.check_source(owner, end)?;
        store.failed = false;
        drop(histories);
        self.recheck_closed(owner, end)?;
        Ok(OriginalSession { session })
    }
    fn record_ready(
        &mut self,
        original: OriginalSession,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> AppResult<ReadySession> {
        let (store, _) = self.begin(&original.session, State::Original, owner, end)?;
        owner.originals_check_ready(ready, end)?;
        // Actual working bytes are archived before the unchanged-generation
        // veto inside record_working_phase. Unknown edits remain a refusal,
        // not a new adopted baseline or declared-side-effect implementation.
        store
            .tree
            .record_working_phase(Phase::Ready, end)
            .map_err(refused)?;
        store.check_source(owner, end)?;
        store.ready = Some((ready.pid(), ready.port(), ready.record_sha256().into()));
        store.state = State::Ready;
        store.failed = false;
        self.recheck_closed(owner, end)?;
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
        let (store, _) = self.begin(&ready.session, State::Ready, owner, end)?;
        let (pid, port, digest) = store
            .ready
            .as_ref()
            .ok_or_else(|| refused("原typed Ready缺席"))?;
        owner.originals_check_live_record(*pid, *port, digest, end)?;
        store
            .tree
            .record_working_phase(Phase::PreparedRestore, end)
            .map_err(refused)?;
        // OfficialTeardown is an intention, persisted before the actual POST.
        store
            .tree
            .advance_phase(Phase::OfficialTeardown, end)
            .map_err(refused)?;
        store.check_source(owner, end)?;
        store.stop_deadline = Some(end);
        store.state = State::Prepared;
        store.failed = false;
        self.recheck_closed(owner, end)?;
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
        let (store, previous) = self.begin(&prepared.session, State::Prepared, owner, end)?;
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
        let mut histories = previous
            .iter_mut()
            .map(|old| &mut old.tree)
            .collect::<Vec<_>>();
        store
            .tree
            .restore_after_teardown_with_history(&mut histories, end)
            .map_err(refused)?;
        // Native predicates are reobserved AFTER restoration, inside the same
        // absolute Stop window. Phase text alone is never a predicate.
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.check_source(owner, end)?;
        store.tree.seal_closed_originals(end).map_err(refused)?;
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.check_source(owner, end)?;
        store
            .tree
            .release_closed_original_handles(end)
            .map_err(refused)?;
        native_exit.check(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &supervisor, end)?;
        owner.originals_check_stopped_records(end)?;
        store.check_source(owner, end)?;
        store.closed_exit = Some(native_exit.into_closed_history(owner, end)?);
        store.closed_supervisor = Some(supervisor.into_closed_history(owner, end)?);
        store.state = State::Closed;
        // Closed history no longer constrains a future session's config. Each
        // successor obtains a new native pin and independent declaration.
        store.routes = None;
        store.failed = false;
        drop(histories);
        self.recheck_closed(owner, end)?;
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
        if self.successor.is_some() || self.closed.len() >= 512 {
            return Err(refused("后继ticket重复或原历史集合预算耗尽"));
        }
        let (store, _) = self.begin(&closed.session, State::Closed, owner, end)?;
        store
            .closed_exit
            .as_ref()
            .ok_or_else(|| refused("原Stop窗口内完成的退出历史缺席"))?
            .check(owner, end)?;
        store
            .closed_supervisor
            .as_ref()
            .ok_or_else(|| refused("原监督缺席历史缺席"))?
            .check(owner, end)?;
        // The new action does not reuse a previous Stop proof/deadline.
        let absent = WindowsSupervisor.observe_absent(owner, end)?;
        WindowsSupervisor.recheck_absent(owner, &absent, end)?;
        owner.originals_check_stopped_records(end)?;
        store.tree.recheck(end).map_err(refused)?;
        store.check_source(owner, end)?;
        let ticket = SuccessorTicket {
            deadline: end,
            stop_deadline: window.stop_deadline(),
            binding: store.source.binding_raw.clone(),
            previous_session: store.tree.session_id().into(),
        };
        store.failed = false;
        self.closed.push(
            self.store
                .take()
                .ok_or_else(|| refused("原生闭合Store缺席"))?,
        );
        self.successor = Some(ticket);
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
        for (schema, version, platform, digest) in [
            (2, "2.99.0", "win32-x64", "a".repeat(64)),
            (1, "2.99.1", "win32-x64", "a".repeat(64)),
            (1, "2.99.0", "darwin-arm64", "a".repeat(64)),
            (1, "2.99.0", "win32-x64", "A".repeat(64)),
        ] {
            let unsupported = CurrentSelection {
                schema_version: schema,
                version: version.into(),
                platform: platform.into(),
                image_sha256: digest,
            };
            std::fs::write(&path, serde_json::to_vec(&unsupported).unwrap()).unwrap();
            assert!(
                CurrentPin::capture(&path, &unsupported, end()).is_err(),
                "self-consistent unsupported selection must refuse"
            );
        }
        std::fs::write(&path, serde_json::to_vec(&expected).unwrap()).unwrap();
        std::fs::hard_link(&path, dir.path().join("alias.json")).unwrap();
        assert!(CurrentPin::capture(&path, &expected, end()).is_err());
        println!("unknown source field and multiple native links refused; no adapter/native phase tag issued");
    }
}
