//! New existing-root v3 capture store. Old pending namespaces are not adopted,
//! converted, scanned as Closed or overwritten. Public Start gate stays closed.
use super::super::{
    coordination::{LeasePreparation, NativeLease},
    source::Source,
    tree::{AnchoredTree, CapturedTrees},
    v2::routes::RoutePin,
};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        windows_official_lifecycle024::{OriginalActionKind, Owner},
        windows_official_runtime024::{create_private_directory, PinnedDirectory},
    },
    modules::runtime::official_lifecycle024::ActionWindow,
};
use std::time::Instant;
mod predecessors;
mod preparation;
#[cfg(test)]
mod test_failure;
pub(in crate::infrastructure::windows_client_originals024) use predecessors::NativePredecessors;
use preparation::PartialCaptureAnchor;
#[cfg(test)]
pub(crate) use test_failure::CaptureFailureArm;
struct CaptureStore {
    source: Source,
    routes: Option<RoutePin>,
    route_close: Option<super::super::v2::routes::RetainedRouteClose>,
    core: CapturedTrees,
    failed: bool,
    ready: Option<(u32, u16, String)>,
    stop_deadline: Option<Instant>,
    content_start: super::super::content::StartContentScope,
    restored_history: bool,
    closed_exit: Option<crate::infrastructure::windows_diagnostic_roles024::ClosedExitHistory>,
    closed_absent: Option<crate::infrastructure::windows_supervisor024::ClosedAbsentHistory>,
    stopped_content: Option<super::super::content::StoppedContentScope>,
    stop_completed: bool,
}
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct CaptureSlot {
    retained: Option<CaptureStore>,
    partial: Option<PartialCaptureAnchor>,
    failed: bool,
}
/// Lives with the worker outside the fallible Start call. Legacy v2 callers
/// use the scoped variant solely to preserve their existing test path.
#[derive(Default)]
pub(crate) struct StartRanges {
    preparation: Option<LeasePreparation>,
    lease: Option<NativeLease>,
    raw: Option<i32>,
    #[cfg(test)]
    failure: Option<test_failure::BoundCaptureFailure>,
}
impl StartRanges {
    pub(crate) fn retained() -> Self {
        Self {
            preparation: Some(LeasePreparation::default()),
            ..Self::default()
        }
    }
    fn next_preparation(&mut self) -> io::Result<Option<&mut LeasePreparation>> {
        if self.lease.is_some() {
            return Err(invalid());
        }
        if let Some(previous) = &mut self.preparation {
            if !previous.empty_after_transfer() {
                return Err(invalid());
            }
            *previous = LeasePreparation::default();
        }
        Ok(self.preparation.as_mut())
    }
    pub(crate) fn empty(&self) -> bool {
        self.lease.is_none()
            && self
                .preparation
                .as_ref()
                .is_some_and(LeasePreparation::empty_after_transfer)
    }
    pub(crate) fn revoke(&mut self) {
        if let Some(preparation) = &mut self.preparation {
            preparation.revoke_retained();
        }
        if let Some(lease) = &mut self.lease {
            lease.retain_failed_start();
        }
    }
    pub(crate) fn raw_error(&self) -> Option<i32> {
        self.raw
    }
    pub(in crate::infrastructure::windows_client_originals024) fn inspect_failed_start(
        &mut self,
        owner: &mut Owner,
        origin: &crate::infrastructure::windows_official_lifecycle024::BeforeResumeLeaseBinding,
        end: Instant,
    ) -> io::Result<super::super::coordination::LeaseFacts> {
        if !self
            .preparation
            .as_ref()
            .is_some_and(LeasePreparation::empty_after_transfer)
        {
            return Err(invalid());
        }
        self.lease
            .as_mut()
            .ok_or_else(invalid)?
            .inspect_failed_start(owner, origin, end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn take_retained_lease(
        &mut self,
    ) -> io::Result<NativeLease> {
        if !self
            .preparation
            .as_ref()
            .is_some_and(LeasePreparation::empty_after_transfer)
        {
            return Err(invalid());
        }
        self.lease.take().ok_or_else(invalid)
    }
    fn refuse_io(&mut self, error: io::Error) -> AppError {
        self.raw = error.raw_os_error();
        refused(error)
    }
    fn retain_or_release_legacy(&mut self, end: Instant) {
        if self.preparation.is_none() {
            if let Some(held) = self.lease.take() {
                let _ = held.release(end);
            }
        }
    }
}
fn refused(error: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_v3_capture_required".into(),
        detail: format!("原v3捕获/来源/保管未确认，保留已发生前缀：{error}"),
    }
}
fn private_child_retained(
    anchor: &mut PartialCaptureAnchor,
    parent: usize,
    name: &str,
    end: Instant,
) -> io::Result<()> {
    anchor.pins[parent].recheck()?;
    let path = anchor.pins[parent].path().join(name);
    let child = match std::fs::symlink_metadata(&path) {
        Ok(_) => PinnedDirectory::open_private(&path)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => anchor.create_directory(&path, end)?,
        Err(e) => return Err(e),
    };
    anchor.pins.push(child);
    anchor.pins[parent].recheck()?;
    anchor.pins.last().unwrap().recheck()
}
fn require_fresh_namespace(windows: &PinnedDirectory) -> io::Result<()> {
    windows.recheck()?;
    if std::fs::read_dir(windows.path())?
        .next()
        .transpose()?
        .is_some()
    {
        return Err(invalid());
    }
    windows.recheck()
}
impl CaptureStore {
    fn capture_held(
        owner: &mut Owner,
        window: &ActionWindow,
        predecessors: &NativePredecessors,
        anchor: &mut PartialCaptureAnchor,
        ranges: &mut StartRanges,
    ) -> AppResult<Self> {
        window.remaining()?;
        let end = window.deadline();
        if anchor.owner_scope != owner.scope()? || anchor.source.is_some() {
            return Err(refused("original preparation anchor differs or reused"));
        }
        #[cfg(test)]
        let fixture_parent = match (owner.fixture_isolation(), ranges.failure.is_some()) {
            (Some(isolation), true) => {
                isolation.verify_owner(owner, end).map_err(refused)?;
                Some(isolation.pin_client_parent().map_err(refused)?)
            }
            (None, false) => None,
            _ => {
                return Err(refused(
                    "fixture capture requires exactly one bound failure arm",
                ))
            }
        };
        anchor.source = Some(Source::capture(owner, end)?.0);
        let source = anchor
            .source
            .as_mut()
            .ok_or_else(|| refused("original source prefix missing"))?;
        predecessors
            .check_source_window(&source.binding_raw, window)
            .map_err(refused)?;
        anchor.routes = Some(RoutePin::capture(owner.home_path(), end).map_err(refused)?);
        anchor.content_start = Some(super::super::content::StartContentScope::capture(
            source,
            owner,
            anchor.routes.as_mut().unwrap(),
            end,
        )?);
        anchor.context = Some(source.journal_context(owner, OriginalActionKind::Start, end)?);
        anchor
            .pins
            .push(PinnedDirectory::open_private(owner.data_path()).map_err(refused)?);
        let client_parent = || PinnedDirectory::open(owner.profile_path()).map_err(refused);
        #[cfg(test)]
        let client_parent = || fixture_parent.map_or_else(client_parent, Ok);
        anchor.pins.push(client_parent()?);
        let official_path = anchor.pins[0].path().join("official024");
        anchor.intend(official_path).map_err(refused)?;
        private_child_retained(anchor, 0, "official024", end).map_err(refused)?;
        let windows_path = anchor.pins[2].path().join("windows");
        anchor.intend(windows_path).map_err(refused)?;
        private_child_retained(anchor, 2, "windows", end).map_err(refused)?;
        predecessors
            .check_namespace(
                &anchor.pins[3],
                &anchor.source.as_ref().unwrap().binding_raw,
                None,
                end,
            )
            .map_err(refused)?;
        let acquired = (|| {
            let lease = if let Some(preparation) = ranges.next_preparation()? {
                preparation.prepare_start(owner, end)?;
                preparation.take_complete()?
            } else {
                NativeLease::acquire_existing(owner, end)?
            };
            ranges.lease = Some(lease);
            Ok::<_, io::Error>(())
        })();
        acquired.map_err(|error| ranges.refuse_io(error))?;
        let created = (|| -> io::Result<()> {
            let root = anchor.pins[1].path().join(".codex");
            let mut claim = ranges
                .lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            let reservation = uuid::Uuid::new_v4();
            for (i, name) in ["codex", "grok"].into_iter().enumerate() {
                let path = anchor.pins[1]
                    .path()
                    .join(format!(".ocxd024-originals-{reservation}-{name}"));
                anchor.intend(path.clone())?;
                anchor.custody[i] = Some(anchor.create_directory(&path, end)?);
            }
            for (i, name) in [".codex", ".grok"].into_iter().enumerate() {
                let path = anchor.pins[1].path().join(name);
                AnchoredTree::capture_retained(
                    &path,
                    anchor.custody[i].as_ref().unwrap().path(),
                    &mut claim,
                    &root,
                    end,
                    &mut anchor.trees[i],
                )?;
            }
            let plan = StoragePlan::from_native_data(
                anchor
                    .trees
                    .iter()
                    .map(|t| t.as_ref().unwrap().plan())
                    .collect(),
                anchor.pins[1].path(),
                anchor.pins[1].identity(),
            )?;
            predecessors.check_namespace(
                &anchor.pins[3],
                &anchor.source.as_ref().unwrap().binding_raw,
                None,
                end,
            )?;
            let path = anchor.pins[3].path().join(uuid::Uuid::new_v4().to_string());
            anchor.intend(path.clone())?;
            let directory = anchor.create_directory(&path, end)?;
            anchor.pins.push(directory);
            predecessors.check_namespace(
                &anchor.pins[3],
                &anchor.source.as_ref().unwrap().binding_raw,
                anchor.pins.last(),
                end,
            )?;
            let directory = PinnedDirectory::open_private(anchor.pins.last().unwrap().path())?;
            #[cfg(test)]
            if let Some(arm) = ranges.failure.take() {
                arm.fail(owner, anchor, &directory, &mut claim, &root, end)?;
            }
            #[cfg(test)]
            if owner.fixture_isolation().is_some() {
                return Err(invalid());
            }
            anchor.intend(directory.path().join("manifest.json"))?;
            anchor.intend(directory.path().join("current.json"))?;
            Journal::create_from_source_retained(
                directory,
                anchor.context.take().ok_or_else(invalid)?,
                plan,
                predecessors,
                &mut claim,
                owner,
                end,
                &mut anchor.journal_prefix,
            )?;
            anchor.journal = Some(anchor.journal_prefix.take_complete()?);
            Ok(())
        })();
        match created {
            Ok(()) => anchor.into_store().map_err(refused),
            Err(error) => {
                anchor.failed = true;
                Err(ranges.refuse_io(error))
            }
        }
    }
    fn check_source(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        self.source.check(owner, end)?;
        self.routes
            .as_mut()
            .ok_or_else(|| refused("原路由pin已转移或缺席"))?
            .declared(owner.home_path(), end)
            .map_err(refused)?;
        self.core
            .journal()
            .check_source_owner(owner, end)
            .map_err(refused)?;
        self.core
            .journal()
            .check_capture_namespace(end)
            .map_err(refused)
    }
}
impl CaptureSlot {
    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_history(
        &mut self,
        owner: &mut Owner,
        prefix: &crate::infrastructure::windows_client_originals024::PreparedAbortPrefix,
        proof: &crate::infrastructure::windows_client_originals024::QuiescentOriginals,
        attachment: &mut super::super::adapter::before_resume::attachment::BoundAttachment,
        released: &mut super::super::coordination::ReleasedLease,
        window: &crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            owner.check_before_resume_history(window, read_end)?;
            if self.partial.is_some() {
                return Err(refused("原partial不能签完整history"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原history Store缺席"))?;
            if store.ready.is_some()
                || store.stop_deadline.is_some()
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
            {
                return Err(refused("不是原beforeResume history分支"));
            }
            let mut scope =
                super::super::adapter::before_resume::history::OriginalHistoricalScope::bind(
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原history route缺席"))?,
                    &mut store.source,
                    owner,
                    prefix,
                    proof,
                    attachment,
                    window,
                    read_end,
                )?;
            store
                .core
                .read_before_resume_history(&mut scope, released, window.deadline(), read_end)
                .map_err(refused)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn recheck_before_resume_return(
        &mut self,
        owner: &mut Owner,
        prefix: &crate::infrastructure::windows_client_originals024::PreparedAbortPrefix,
        proof: &crate::infrastructure::windows_client_originals024::QuiescentOriginals,
        attachment: &mut super::super::adapter::before_resume::attachment::BoundAttachment,
        released: &mut super::super::coordination::ReleasedLease,
        window: &crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            owner.check_before_resume_quiescent(window)?;
            if self.partial.is_some() {
                return Err(refused("原preparation尚未完成"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原Store缺席"))?;
            if store.ready.is_some()
                || store.stop_deadline.is_some()
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
            {
                return Err(refused("不是原beforeResume返回分支"));
            }
            let mut scope =
                crate::infrastructure::windows_client_originals024::OriginalReleasedScope::bind(
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原route缺席"))?,
                    &mut store.source,
                    owner,
                    prefix,
                    proof,
                    attachment,
                    window,
                )?;
            store
                .core
                .recheck_before_resume_return(&mut scope, released, window.deadline())
                .map_err(refused)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_before_resume_release(
        &mut self,
        owner: &mut Owner,
        prefix: &crate::infrastructure::windows_client_originals024::PreparedAbortPrefix,
        proof: &crate::infrastructure::windows_client_originals024::QuiescentOriginals,
        attachment: &mut super::super::adapter::before_resume::attachment::BoundAttachment,
        released: &mut super::super::coordination::ReleasedLease,
        probe: &mut super::super::coordination::ReleaseProbe,
        window: &crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
    ) -> AppResult<super::super::tree::AbortRestoredReadback> {
        let result = (|| {
            owner.check_before_resume_quiescent(window)?;
            if self.partial.is_some() {
                return Err(refused("原preparation不能签canonical释放完成"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原Store缺席"))?;
            if store.ready.is_some()
                || store.stop_deadline.is_some()
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
            {
                return Err(refused("不是原beforeResume分支"));
            }
            let mut scope =
                crate::infrastructure::windows_client_originals024::OriginalReleasedScope::bind(
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原route缺席"))?,
                    &mut store.source,
                    owner,
                    prefix,
                    proof,
                    attachment,
                    window,
                )?;
            store
                .core
                .finish_before_resume_release(&mut scope, released, probe, window.deadline())
                .map_err(refused)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn return_before_resume(
        &mut self,
        owner: &mut Owner,
        prefix: &crate::infrastructure::windows_client_originals024::PreparedAbortPrefix,
        proof: &crate::infrastructure::windows_client_originals024::QuiescentOriginals,
        attachment: &mut super::super::adapter::before_resume::attachment::BoundAttachment,
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        window: &crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            owner.check_before_resume_quiescent(window)?;
            if self.partial.is_some() {
                return Err(refused("原准备分支未能完整核验，保留Required"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原Store缺席"))?;
            if store.ready.is_some()
                || store.stop_deadline.is_some()
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
            {
                return Err(refused("不是原beforeResume分支"));
            }
            store
                .routes
                .as_mut()
                .ok_or_else(|| refused("原route缺席"))?
                .declared(owner.home_path(), window.deadline())
                .map_err(refused)?;
            let mut scope =
                crate::infrastructure::windows_client_originals024::OriginalReturnScope::bind(
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原route缺席"))?,
                    &mut store.source,
                    owner,
                    prefix,
                    proof,
                    attachment,
                    window,
                )?;
            store
                .core
                .return_before_resume(&mut scope, claim, window.deadline())
                .map_err(refused)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_prefix(
        &mut self,
        owner: &mut Owner,
        claim: &mut super::super::coordination::AnchoredClaim<'_>,
        window: &crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            owner.check_before_resume_process(window)?;
            if self.partial.is_some() {
                return Err(refused("原准备前缀尚无完整canonical；不得补造manifest"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原Store缺席；不从记录重建"))?;
            if store.ready.is_some()
                || store.stop_deadline.is_some()
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
            {
                return Err(refused("不是原失败Start分支"));
            }
            store
                .routes
                .as_mut()
                .ok_or_else(|| refused("原route缺席"))?
                .declared(owner.home_path(), window.deadline())
                .map_err(refused)?;
            let data = store
                .core
                .read_before_resume_prefix(&mut store.source, owner, claim, window)
                .map_err(refused)?;
            owner.check_before_resume_process(window)?;
            Ok(data)
        })();
        if result.is_err() {
            self.failed = true;
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn completed_stop_original(
        &mut self,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            let old = self
                .retained
                .as_ref()
                .and_then(|s| s.stop_deadline)
                .ok_or_else(|| refused("原Stop截止缺席"))?;
            let read = super::read_window::HistoryReadWindow::completed_original(end, old)
                .map_err(refused)?;
            self.completed_stop_read(owner, &read)
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_ready_existing(
        &mut self,
        owner: &mut Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed {
                return Err(refused("原Ready槽已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原ReadyStore缺席"))?;
            if store.failed
                || store.stop_deadline.is_some()
                || store.route_close.is_some()
                || store.stop_completed
                || store.ready.as_ref()
                    != Some(&(ready.pid(), ready.port(), ready.record_sha256().into()))
            {
                return Err(refused("原Ready代次或阶段未知"));
            }
            store.failed = true;
            store.source.check(owner, end)?;
            store
                .routes
                .as_mut()
                .ok_or_else(|| refused("原Ready路由pin缺席"))?
                .declared(owner.home_path(), end)
                .map_err(refused)?;
            store
                .core
                .check_original_ready_read(&mut store.source, owner, ready, end)
                .map_err(refused)?;
            owner.originals_check_ready(ready, end)?;
            store.source.check(owner, end)?;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    /// Read only under the caller's SAME Status sample; old mutation cutoff
    /// stays immutable. This is internal history verification, not a status
    /// boolean/serialized Closed factory or successor authorization.
    pub(in crate::infrastructure::windows_client_originals024) fn completed_stop_at(
        &mut self,
        owner: &mut Owner,
        when: Instant,
        end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            let read = super::read_window::HistoryReadWindow::status(when, end).map_err(refused)?;
            self.completed_stop_read(owner, &read)
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn completed_stop_for_action(
        &mut self,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        let result = (|| {
            let read = super::read_window::HistoryReadWindow::action(window).map_err(refused)?;
            self.completed_stop_read(owner, &read)
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    fn poison_completed_read(&mut self) {
        self.failed = true;
        if let Some(store) = &mut self.retained {
            store.failed = true;
            store.core.poison();
        }
    }
    fn completed_stop_read(
        &mut self,
        owner: &mut Owner,
        read: &super::read_window::HistoryReadWindow,
    ) -> AppResult<()> {
        let result = (|| {
            let end = read.end().map_err(refused)?;
            if self.failed {
                return Err(refused("原完成历史已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原v3Store缺席"))?;
            if store.failed || !store.stop_completed || !store.restored_history {
                return Err(refused("原Stop未完成"));
            }
            store.failed = true;
            if store.routes.is_some() {
                return Err(refused("完成历史仍持原live路由pin"));
            }
            store
                .route_close
                .as_ref()
                .ok_or_else(|| refused("原路由关闭history缺席"))?
                .check_history(end)
                .map_err(refused)?;
            store
                .content_start
                .check_completed_history(
                    store
                        .stopped_content
                        .as_ref()
                        .ok_or_else(|| refused("原内容scope缺席"))?,
                    &mut store.source,
                    owner,
                    end,
                )
                .map_err(refused)?;
            store
                .closed_exit
                .as_ref()
                .ok_or_else(|| refused("原退出history缺席"))?
                .check(owner, end)?;
            store
                .closed_absent
                .as_ref()
                .ok_or_else(|| refused("原监督history缺席"))?
                .check(owner, end)?;
            store
                .core
                .journal()
                .completed_stop_history_read(read)
                .map_err(refused)?;
            store
                .core
                .journal()
                .check_route_closed_history(
                    store
                        .route_close
                        .as_ref()
                        .ok_or_else(|| refused("原路由关闭history缺席"))?,
                    end,
                )
                .map_err(refused)?;
            store.source.check(owner, end)?;
            store
                .closed_exit
                .as_ref()
                .ok_or_else(|| refused("原退出history缺席"))?
                .check(owner, end)?;
            store
                .closed_absent
                .as_ref()
                .ok_or_else(|| refused("原监督history缺席"))?
                .check(owner, end)?;
            read.check().map_err(refused)?;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    /// Actual internal restore/history lane only. This does NOT return a
    /// ClosedEligibility or ordinary Stop success: the core retains/transfers
    /// originals before explicit close, and retains each actual timely exit/
    /// supervisor history before final canonical closure/seal. Adapter, actual
    /// production execution and public acceptance remain pending.
    pub(in crate::infrastructure::windows_client_originals024) fn restore_stopped_existing(
        &mut self,
        owner: &mut Owner,
        normal: crate::infrastructure::windows_diagnostic_roles024::NormalExit,
        absent: crate::infrastructure::windows_supervisor024::Absent,
        official: &crate::modules::runtime::official_lifecycle024::StopAccepted,
        end: Instant,
    ) -> AppResult<()> {
        let mut lease = None;
        let result = (|| {
            if self.failed {
                return Err(refused("原v3 slot已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原v3 Store缺席"))?;
            if store.failed
                || store.restored_history
                || store.closed_exit.is_some()
                || store.closed_absent.is_some()
                || store.stopped_content.is_some()
                || store.stop_completed
                || store.stop_deadline != Some(end)
                || store.ready.as_ref()
                    != Some(&(
                        official.pid(),
                        official.port(),
                        official.record_sha256().into(),
                    ))
            {
                return Err(refused("原Stop/context代次未知"));
            }
            store.failed = true;
            let content = store
                .content_start
                .stopped(
                    &mut store.source,
                    owner,
                    &normal,
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原路由pin已转移或缺席"))?,
                    end,
                )
                .map_err(refused)?;
            store
                .core
                .observe_original_stopped(
                    &mut store.source,
                    owner,
                    &normal,
                    &absent,
                    official,
                    &mut lease,
                    end,
                )
                .map_err(refused)?;
            content
                .check(
                    &mut store.source,
                    owner,
                    &normal,
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原路由pin已转移或缺席"))?,
                    end,
                )
                .map_err(refused)?;
            store
                .core
                .restore_original_stopped(
                    &mut store.source,
                    owner,
                    &normal,
                    &absent,
                    official,
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原路由pin已转移或缺席"))?,
                    &content,
                    &mut lease,
                    end,
                )
                .map_err(refused)?;
            store.check_source(owner, end)?;
            content
                .check(
                    &mut store.source,
                    owner,
                    &normal,
                    store
                        .routes
                        .as_mut()
                        .ok_or_else(|| refused("原路由pin已转移或缺席"))?,
                    end,
                )
                .map_err(refused)?;
            if lease.is_some() {
                return Err(refused("原恢复段lease未显式释放"));
            }
            store.restored_history = true;
            // Preserve every successful original completion before the next
            // fallible operation. No partial history is discarded or retried.
            store.closed_exit = Some(normal.into_closed_history(owner, end)?);
            store.closed_absent = Some(absent.into_closed_history(owner, end)?);
            store.stopped_content = Some(content);
            if store.route_close.is_some() {
                return Err(refused("原路由关闭对象已有状态"));
            }
            // Move exact owned RoutePin into a retained slot BEFORE prepare or
            // any fallible route-close IO. No Drop release or replacement.
            let routes = store
                .routes
                .take()
                .ok_or_else(|| refused("原路由pin缺席"))?;
            store.route_close = Some(super::super::v2::routes::RetainedRouteClose::retain(
                routes, end,
            ));
            store
                .route_close
                .as_mut()
                .ok_or_else(|| refused("原路由关闭槽缺席"))?
                .prepare(end)
                .map_err(refused)?;
            let mut guard = super::journal::CompletedStopGuard::bind(
                &mut store.source,
                owner,
                store
                    .closed_exit
                    .as_ref()
                    .ok_or_else(|| refused("原Stop退出history缺席"))?,
                store
                    .closed_absent
                    .as_ref()
                    .ok_or_else(|| refused("原Stop监督history缺席"))?,
                official,
                store
                    .route_close
                    .as_mut()
                    .ok_or_else(|| refused("原路由关闭槽缺席"))?,
                &mut store.content_start,
                store
                    .stopped_content
                    .as_ref()
                    .ok_or_else(|| refused("原内容scope缺席"))?,
                store.core.journal(),
                end,
            )
            .map_err(refused)?;
            store
                .core
                .journal()
                .complete_original_stop_history(&mut guard, end)
                .map_err(refused)?;
            store.source.check(owner, end)?;
            store
                .core
                .journal()
                .check_source_owner(owner, end)
                .map_err(refused)?;
            store
                .route_close
                .as_ref()
                .ok_or_else(|| refused("原路由关闭history缺席"))?
                .check_history(end)
                .map_err(refused)?;
            store
                .closed_exit
                .as_ref()
                .ok_or_else(|| refused("原退出history缺席"))?
                .check(owner, end)?;
            store
                .closed_absent
                .as_ref()
                .ok_or_else(|| refused("原监督history缺席"))?
                .check(owner, end)?;
            store.stop_completed = true;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            if let Some(held) = lease.take() {
                let _ = held.release(end);
            }
            self.failed = true;
            if let Some(partial) = &mut self.partial {
                partial.failed = true;
            }
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_stop_existing(
        &mut self,
        owner: &mut Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        kind: OriginalActionKind,
        end: Instant,
    ) -> AppResult<()> {
        let mut lease = None;
        let result = (|| {
            if self.failed {
                return Err(refused("原v3 slot已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原v3 Store缺席"))?;
            if store.failed
                || store.stop_deadline.is_some()
                || store.ready.as_ref()
                    != Some(&(ready.pid(), ready.port(), ready.record_sha256().into()))
            {
                return Err(refused("原v3 Ready代次/Stop窗未知或重复"));
            }
            store.failed = true;
            // Prior Start window is allowed to have ended. Only fresh actual
            // Source/routes/typedReady admission precedes the dedicated native
            // StopAction bridge; do NOT call old journal with a new raw timer.
            store.source.check(owner, end)?;
            store
                .routes
                .as_mut()
                .ok_or_else(|| refused("原路由pin已转移或缺席"))?
                .declared(owner.home_path(), end)
                .map_err(refused)?;
            owner.originals_check_ready(ready, end)?;
            store
                .core
                .prepare_original_stop(&mut store.source, owner, ready, kind, &mut lease, end)
                .map_err(refused)?;
            store.check_source(owner, end)?;
            owner.originals_check_ready(ready, end)?;
            if lease.is_some() {
                return Err(refused("官方teardown前本段lease仍持有"));
            }
            store.stop_deadline = Some(end);
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            if let Some(held) = lease.take() {
                let _ = held.release(end);
            }
            self.failed = true;
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn before_resume_existing(
        &mut self,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<()> {
        self.before_resume_existing_retained(owner, &mut StartRanges::default(), end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn before_resume_existing_retained(
        &mut self,
        owner: &mut Owner,
        ranges: &mut StartRanges,
        end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed {
                return Err(refused("原v3 slot已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原v3 Store缺席"))?;
            if store.failed || store.ready.is_some() {
                return Err(refused("原v3捕获阶段未知"));
            }
            store.failed = true;
            store.check_source(owner, end)?;
            owner.originals_require_unresumed(end)?;
            ranges.next_preparation().map_err(refused)?;
            store
                .core
                .verify_original_pre_resume(
                    &mut store.source,
                    owner,
                    &mut ranges.lease,
                    ranges.preparation.as_mut(),
                    end,
                )
                .map_err(|error| ranges.refuse_io(error))?;
            owner.originals_require_unresumed(end)?;
            store.check_source(owner, end)?;
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            ranges.retain_or_release_legacy(end);
            self.failed = true;
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn record_ready_existing(
        &mut self,
        owner: &mut Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        end: Instant,
    ) -> AppResult<()> {
        self.record_ready_existing_retained(owner, ready, &mut StartRanges::default(), end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn record_ready_existing_retained(
        &mut self,
        owner: &mut Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        ranges: &mut StartRanges,
        end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed {
                return Err(refused("原v3 slot已poison"));
            }
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原v3 Store缺席"))?;
            if store.failed || store.ready.is_some() {
                return Err(refused("重复或未知原v3 Ready"));
            }
            store.failed = true;
            store.check_source(owner, end)?;
            owner.originals_check_ready(ready, end)?;
            ranges.next_preparation().map_err(refused)?;
            store
                .core
                .record_original_ready(
                    &mut store.source,
                    owner,
                    ready,
                    &mut ranges.lease,
                    ranges.preparation.as_mut(),
                    end,
                )
                .map_err(|error| ranges.refuse_io(error))?;
            store.check_source(owner, end)?;
            owner.originals_check_ready(ready, end)?;
            store.ready = Some((ready.pid(), ready.port(), ready.record_sha256().into()));
            store.failed = false;
            Ok(())
        })();
        if result.is_err() {
            ranges.retain_or_release_legacy(end);
            self.failed = true;
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn occupied(&self) -> bool {
        // A failed attempt is also a persistent local refusal, even if source
        // admission failed before a native Store existed. No fallback to v2.
        self.failed || self.retained.is_some() || self.partial.is_some()
    }
    pub(in crate::infrastructure::windows_client_originals024) fn capture_existing(
        &mut self,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        let empty = NativePredecessors::empty(window);
        self.capture_existing_with_predecessors(
            owner,
            window,
            &empty,
            uuid::Uuid::new_v4(),
            &mut StartRanges::default(),
        )
    }
    pub(in crate::infrastructure::windows_client_originals024) fn capture_existing_with_predecessors(
        &mut self,
        owner: &mut Owner,
        window: &ActionWindow,
        predecessors: &NativePredecessors,
        session_id: uuid::Uuid,
        ranges: &mut StartRanges,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed || self.retained.is_some() || self.partial.is_some() {
                return Err(refused("原v3 slot已有状态；不得替换或追赶"));
            }
            // Install the original typed prefix BEFORE source/route/custody
            // preparation. No returned tag or completed Store is required to
            // retain every successfully enrolled native object after failure.
            self.partial = Some(PartialCaptureAnchor::new(
                owner.scope()?,
                session_id,
                owner.original_host_instance(),
            ));
            let store = CaptureStore::capture_held(
                owner,
                window,
                predecessors,
                self.partial.as_mut().unwrap(),
                ranges,
            )?;
            // Store is reachable through the ORIGINAL adapter before the first
            // journaled client move; the active lease remains with the worker.
            self.retained = Some(store);
            self.partial.take();
            let end = window.deadline();
            let work = (|| {
                let store = self
                    .retained
                    .as_mut()
                    .ok_or_else(|| refused("原Store缺席"))?;
                owner.originals_require_unresumed(end)?;
                store.check_source(owner, end)?;
                store
                    .core
                    .prepare_working_batched(
                        ranges
                            .lease
                            .as_mut()
                            .ok_or_else(|| refused("原lease缺席"))?,
                        end,
                    )
                    .map_err(|error| ranges.refuse_io(error))?;
                store
                    .core
                    .open_prepared_working(
                        ranges
                            .lease
                            .as_mut()
                            .ok_or_else(|| refused("原lease缺席"))?,
                        end,
                    )
                    .map_err(|error| ranges.refuse_io(error))?;
                owner.originals_require_unresumed(end)?;
                store.check_source(owner, end)?;
                store
                    .core
                    .release_prepared(&mut ranges.lease, end)
                    .map_err(|error| ranges.refuse_io(error))?;
                owner.originals_require_unresumed(end)?;
                store.check_source(owner, end)?;
                store.failed = false;
                Ok(())
            })();
            work
        })();
        if result.is_err() {
            ranges.retain_or_release_legacy(window.deadline());
            self.failed = true;
            if let Some(store) = &mut self.retained {
                store.failed = true;
                store.core.poison();
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::windows_client_originals024::tree::tests::Fixture;
    #[test]
    fn wl2_r6_capture_namespace_old_v2_or_text_closed_never_grants_fresh_capture() {
        let mut f = Fixture::new();
        let directory = create_private_directory(&f.path().join("windows")).unwrap();
        require_fresh_namespace(&directory).unwrap();
        let old = create_private_directory(&directory.path().join("old-v2-pending")).unwrap();
        let record = crate::infrastructure::windows_official_runtime024::PrivateRecord::create(
            &old.path().join("current.json"),
            b"{\"schema\":\"windows-client-current-v2\",\"phase\":\"Closed\"}",
            1024,
        )
        .unwrap();
        assert!(require_fresh_namespace(&directory).is_err());
        assert!(old.path().join("current.json").exists());
        println!("r6 actual private old namespace and textClosed refuse fresh v3 before custody/mutation; no conversion/adoption or overwrite");
        drop(record);
        drop(old);
        drop(directory);
        f.retain();
    }
    #[test]
    fn wl2_r6_capture_retained_store_types_are_send_without_active_lease_or_unsafe_send() {
        fn send<T: Send>() {}
        send::<CaptureSlot>();
        send::<CapturedTrees>();
        send::<Journal>();
        send::<crate::infrastructure::windows_client_originals024::coordination::ReleasedLease>();
        // NativeLease intentionally retains PhantomData<*mut()> and OVERLAPPED
        // ranges; it is not included in any persisted Store field or Send list.
        println!("r6 compiler proves actual CaptureSlot/Source/route/fulltrees/journal/released-original-object set Send; live NativeLease remains stack-only marker unchanged, no unsafe impl Send or fake native factory");
    }
}
