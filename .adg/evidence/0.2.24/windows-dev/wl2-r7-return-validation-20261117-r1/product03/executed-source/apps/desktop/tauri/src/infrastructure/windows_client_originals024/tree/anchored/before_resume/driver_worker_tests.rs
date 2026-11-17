//! Actual existing native pipeline on original owning-worker transport. This
//! fixture never constructs product Owner/Source authority or touches r2.
use super::*;
use crate::infrastructure::windows_client_originals024::{
    recovery_worker::{self, component, WorkerLocal},
    tree::AbortRestoredReadback,
};
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
struct Pipeline {
    core: Option<CapturedTrees>,
    trees: Option<[AnchoredTree; 2]>,
    journal: Option<Journal>,
    native: OriginalSynthetic,
    outputs: [File; 2],
    source: File,
    source_id: FileIdentity,
    session: String,
    action: String,
    scope: String,
    revoked: bool,
    complete: Option<AbortRestoredReadback>,
    start_thread: String,
    run_thread: Option<String>,
    canonical: Option<serde_json::Value>,
    scene: Scene,
}
impl component::Task for Box<Pipeline> {
    fn actual_binding(&mut self, window: &BeforeResumeRecoveryWindow) -> io::Result<[String; 3]> {
        if self.revoked
            || Instant::now() >= window.deadline()
            || id(&self.source, false)? != self.source_id
        {
            return Err(invalid());
        }
        native::verify_private_file(&self.source)?;
        if hash::sha256_file(
            &self
                .scene
                .fixture
                .as_ref()
                .unwrap()
                .path()
                .join("I/driver-source.bin"),
        )? != hash::sha256_hex(b"actual original native driver source, no credentials")
        {
            return Err(invalid());
        }
        self.native.0.verify_binding(self.native.0.binding())?;
        Ok([
            self.session.clone(),
            self.action.clone(),
            self.scope.clone(),
        ])
    }
    fn revoke(&mut self) {
        self.revoked = true;
    }
    fn run(
        &mut self,
        local: &mut WorkerLocal,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        let end = window.deadline();
        let thread = format!("{:?}", std::thread::current().id());
        if thread == self.start_thread {
            return Err(invalid());
        }
        self.run_thread = Some(thread);
        self.actual_binding(window)?;
        component::acquire(local, &self.scene.paths, end)?;
        let lease = local.lease()?;
        let mut claim = lease.anchored_claim(&self.scene.root, end)?;
        let (storage, last, raw, canonical, proof) = setup_data(
            self.journal.as_mut().ok_or_else(invalid)?,
            &mut claim,
            &self.scene,
            end,
            &mut self.native,
        );
        self.canonical = Some(canonical.clone());
        self.core = Some(CapturedTrees::retain(
            self.trees.take().ok_or_else(invalid)?,
            self.journal.take().ok_or_else(invalid)?,
        ));
        let mut guard = Guard {
            native: &mut self.native,
            proof: &proof,
            root: self.scene.root.clone(),
            end,
            epoch: claim.epoch().into(),
            key: claim.key(),
            canonical: canonical.clone(),
            storage,
            last,
            raw,
            fail: "none",
            steps: 0,
        };
        self.core
            .as_mut()
            .unwrap()
            .component_return_before_resume_for_test(&mut guard, &mut claim, end)?;
        claim.check(&self.scene.root, end)?;
        drop(claim);
        local.release(end)?;
        let (released, probe) = local.released_and_probe()?;
        self.complete = Some(
            self.core
                .as_mut()
                .unwrap()
                .component_finish_before_resume_release_for_test(
                    &mut guard, released, probe, end,
                )?,
        );
        self.complete.as_ref().unwrap().data(end)?;
        if self
            .core
            .as_mut()
            .unwrap()
            .journal()
            .component_failed_canonical_for_test(end)?
            != canonical
        {
            return Err(invalid());
        }
        guard.native.0.verify_before_resume_quiescent(&proof)?;
        if self.outputs.iter().any(|f| f.metadata().is_err()) {
            return Err(invalid());
        }
        Ok(())
    }
}
#[test]
fn wl2_r7_concrete_driver_native_original_pipeline_runs_once_on_same_owning_worker_and_returns_released_resources(
) {
    for open in [false, true] {
        let scene = Scene::new();
        let root = scene.fixture.as_ref().unwrap().path().to_path_buf();
        println!("r7-driver-worker-fixture-start={}", root.display());
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        journal
            .prepare_initial_batches(
                super::super::super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            )
            .unwrap();
        if open {
            super::super::super::super::super::super::observation::open_trees(
                &mut trees,
                &mut journal,
                &mut claim,
                old,
            )
            .unwrap();
        }
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for t in &mut trees {
            t.poisoned = true;
        }
        let source_path = root.join("I/driver-source.bin");
        drop(
            PrivateRecord::create(
                &source_path,
                b"actual original native driver source, no credentials",
                1024,
            )
            .unwrap(),
        );
        let source = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&source_path)
            .unwrap();
        let source_id = id(&source, false).unwrap();
        let (native, outputs) = launcher(&scene);
        let start = Instant::now();
        let window =
            BeforeResumeRecoveryWindow::begin(start, start + Duration::from_secs(10)).unwrap();
        let deadline = window.deadline();
        let mut original = Some(component::OriginalTask {
            task: Box::new(Pipeline {
                core: None,
                trees: Some(trees),
                journal: Some(journal),
                native,
                outputs,
                scene,
                source,
                source_id,
                session: uuid::Uuid::new_v4().to_string(),
                action: uuid::Uuid::new_v4().to_string(),
                scope: hash::sha256_hex(b"actual own native pipeline binding"),
                revoked: false,
                complete: None,
                start_thread: format!("{:?}", std::thread::current().id()),
                run_thread: None,
                canonical: None,
            }),
        });
        let control = component::start(&mut original, window).unwrap();
        assert!(original.is_none());
        let observed = loop {
            let v = control.observe(deadline).unwrap();
            if v["phase"] == "ReturnedForValidation" {
                break v;
            }
            assert!(
                v["phase"] == "Registered" || v["phase"] == "Running",
                "native fixture failure stays on worker, do not restart: {v}"
            );
            std::thread::sleep(Duration::from_millis(5));
        };
        let mut returned = control.take_returned().unwrap().unwrap();
        returned.released.check(deadline).unwrap();
        let task = &mut returned.original.task;
        assert_eq!(
            observed["workerThread"],
            task.run_thread.as_deref().unwrap()
        );
        assert_eq!(id(&task.source, false).unwrap(), source_id);
        assert!(!task.revoked);
        let data = task.complete.as_ref().unwrap().data(deadline).unwrap();
        assert_eq!(data["normalStop"], false);
        assert_eq!(data["closed"], false);
        competing_claim(&task.scene.paths[0], false);
        assert!(task.core.as_ref().unwrap().native_prefix().1);
        task.native
            .0
            .verify_binding(task.native.0.binding())
            .unwrap();
        assert_eq!(
            task.native.0.root_state().unwrap(),
            native::ProcessState::Exited { code: 242 }
        );
        assert!(!root.join("must-not-execute").exists());
        println!("r7 original owning worker native pipeline open{open}: elapsedMs={} original sourceFileId={} sameThread={} actual Job242/claim/twoTrees/v1/explicitRelease/fiveRanges/fullreadback completed; private native data={}, productOwner/officialSource acceptance false",start.elapsed().as_millis(),source_id.index,observed["workerThread"],data);
        let recovery_worker::Returned {
            original, released, ..
        } = returned;
        component::join_success_for_test(control).unwrap();
        // Actual released coordinator history still pins its parents. Close it
        // explicitly before the original task's final TempDir fixture closes.
        drop(released);
        drop(original);
        assert!(!root.exists());
        println!("r7-driver-worker-cleanup-absent={}", root.display());
    }
}
