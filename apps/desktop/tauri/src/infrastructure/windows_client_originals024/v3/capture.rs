//! New existing-root v3 capture store. Old pending namespaces are not adopted,
//! converted, scanned as Closed or overwritten. Public Start gate stays closed.
use super::super::{
    coordination::NativeLease,
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
struct CaptureStore {
    source: Source,
    routes: RoutePin,
    core: CapturedTrees,
    failed: bool,
    ready: Option<(u32, u16, String)>,
    stop_deadline: Option<Instant>,
    content_start: super::super::content::StartContentScope,
    restored_history: bool,
}
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct CaptureSlot {
    retained: Option<CaptureStore>,
    failed: bool,
}
fn refused(error: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_v3_capture_required".into(),
        detail: format!("原v3捕获/来源/保管未确认，保留已发生前缀：{error}"),
    }
}
fn private_child(parent: &PinnedDirectory, name: &str) -> io::Result<PinnedDirectory> {
    parent.recheck()?;
    let path = parent.path().join(name);
    let child = match std::fs::symlink_metadata(&path) {
        Ok(_) => PinnedDirectory::open_private(&path)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => create_private_directory(&path)?,
        Err(e) => return Err(e),
    };
    parent.recheck()?;
    child.recheck()?;
    Ok(child)
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
    fn capture_held(owner: &mut Owner, window: &ActionWindow) -> AppResult<(Self, NativeLease)> {
        window.remaining()?;
        let end = window.deadline();
        let (mut source, _) = Source::capture(owner, end)?;
        let routes = RoutePin::capture(owner.home_path(), end).map_err(refused)?;
        let mut routes = routes;
        let content_start = super::super::content::StartContentScope::capture(
            &mut source,
            owner,
            &mut routes,
            end,
        )?;
        let context = source.journal_context(owner, OriginalActionKind::Start, end)?;
        let (data, profile) = (
            PinnedDirectory::open_private(owner.data_path()).map_err(refused)?,
            PinnedDirectory::open(owner.profile_path()).map_err(refused)?,
        );
        let official = private_child(&data, "official024").map_err(refused)?;
        let windows = private_child(&official, "windows").map_err(refused)?;
        require_fresh_namespace(&windows).map_err(refused)?;
        let mut lease = NativeLease::acquire_existing(owner, end).map_err(refused)?;
        let created = (|| -> io::Result<_> {
            let root = profile.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            // Both complete native trees are captured before any ordinary-file
            // move. New private custody directories are never old client data.
            let reservation = uuid::Uuid::new_v4();
            let custody = ["codex", "grok"]
                .map(|name| {
                    create_private_directory(
                        &profile
                            .path()
                            .join(format!(".ocxd024-originals-{reservation}-{name}")),
                    )
                })
                .into_iter()
                .collect::<io::Result<Vec<_>>>()?;
            let trees = [
                AnchoredTree::capture(&root, custody[0].path(), &mut claim, &root, end)?,
                AnchoredTree::capture(
                    &profile.path().join(".grok"),
                    custody[1].path(),
                    &mut claim,
                    &root,
                    end,
                )?,
            ];
            let plan = StoragePlan::from_native_data(
                trees.iter().map(AnchoredTree::plan).collect(),
                profile.path(),
                profile.identity(),
            )?;
            require_fresh_namespace(&windows)?;
            let directory =
                create_private_directory(&windows.path().join(uuid::Uuid::new_v4().to_string()))?;
            let journal =
                Journal::create_from_source(directory, context, plan, &mut claim, owner, end)?;
            Ok(CapturedTrees::retain(trees, journal))
        })();
        match created {
            Ok(core) => Ok((
                Self {
                    source,
                    routes,
                    core,
                    failed: true,
                    ready: None,
                    stop_deadline: None,
                    content_start,
                    restored_history: false,
                },
                lease,
            )),
            Err(error) => {
                let _ = lease.release(end);
                Err(refused(error))
            }
        }
    }
    fn check_source(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        self.source.check(owner, end)?;
        self.routes
            .declared(owner.home_path(), end)
            .map_err(refused)?;
        self.core
            .journal()
            .check_source_owner(owner, end)
            .map_err(refused)
    }
}
impl CaptureSlot {
    /// Actual internal restore/history lane only. This does NOT return a
    /// ClosedEligibility or ordinary Stop success: original-handle ownership,
    /// process/supervisor completion history and final adapter remain pending.
    pub(in crate::infrastructure::windows_client_originals024) fn restore_stopped_existing(
        &mut self,
        owner: &mut Owner,
        normal: &crate::infrastructure::windows_diagnostic_roles024::NormalExit,
        absent: &crate::infrastructure::windows_supervisor024::Absent,
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
                .stopped(&mut store.source, owner, normal, &mut store.routes, end)
                .map_err(refused)?;
            store
                .core
                .observe_original_stopped(
                    &mut store.source,
                    owner,
                    normal,
                    absent,
                    official,
                    &mut lease,
                    end,
                )
                .map_err(refused)?;
            content
                .check(&mut store.source, owner, normal, &mut store.routes, end)
                .map_err(refused)?;
            store
                .core
                .restore_original_stopped(
                    &mut store.source,
                    owner,
                    normal,
                    absent,
                    official,
                    &mut store.routes,
                    &content,
                    &mut lease,
                    end,
                )
                .map_err(refused)?;
            store.check_source(owner, end)?;
            content
                .check(&mut store.source, owner, normal, &mut store.routes, end)
                .map_err(refused)?;
            if lease.is_some() {
                return Err(refused("原恢复段lease未显式释放"));
            }
            store.restored_history = true;
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
        let mut lease = None;
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
            store
                .core
                .verify_original_pre_resume(&mut store.source, owner, &mut lease, end)
                .map_err(refused)?;
            owner.originals_require_unresumed(end)?;
            store.check_source(owner, end)?;
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
    pub(in crate::infrastructure::windows_client_originals024) fn record_ready_existing(
        &mut self,
        owner: &mut Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
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
            if store.failed || store.ready.is_some() {
                return Err(refused("重复或未知原v3 Ready"));
            }
            store.failed = true;
            store.check_source(owner, end)?;
            owner.originals_check_ready(ready, end)?;
            store
                .core
                .record_original_ready(&mut store.source, owner, ready, &mut lease, end)
                .map_err(refused)?;
            store.check_source(owner, end)?;
            owner.originals_check_ready(ready, end)?;
            store.ready = Some((ready.pid(), ready.port(), ready.record_sha256().into()));
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
    pub(in crate::infrastructure::windows_client_originals024) fn occupied(&self) -> bool {
        // A failed attempt is also a persistent local refusal, even if source
        // admission failed before a native Store existed. No fallback to v2.
        self.failed || self.retained.is_some()
    }
    pub(in crate::infrastructure::windows_client_originals024) fn capture_existing(
        &mut self,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<()> {
        let result = (|| {
            if self.failed || self.retained.is_some() {
                return Err(refused("原v3 slot已有状态；不得替换或追赶"));
            }
            let (store, lease) = CaptureStore::capture_held(owner, window)?;
            // Store is reachable through the ORIGINAL adapter before the first
            // journaled client move. Active lease stays on this stack only.
            self.retained = Some(store);
            let mut lease = Some(lease);
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
                        lease.as_mut().ok_or_else(|| refused("原lease缺席"))?,
                        end,
                    )
                    .map_err(refused)?;
                store
                    .core
                    .open_prepared_working(
                        lease.as_mut().ok_or_else(|| refused("原lease缺席"))?,
                        end,
                    )
                    .map_err(refused)?;
                owner.originals_require_unresumed(end)?;
                store.check_source(owner, end)?;
                store
                    .core
                    .release_prepared(&mut lease, end)
                    .map_err(refused)?;
                owner.originals_require_unresumed(end)?;
                store.check_source(owner, end)?;
                store.failed = false;
                Ok(())
            })();
            if work.is_err() {
                if let Some(held) = lease.take() {
                    let _ = held.release(end);
                }
            }
            work
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
