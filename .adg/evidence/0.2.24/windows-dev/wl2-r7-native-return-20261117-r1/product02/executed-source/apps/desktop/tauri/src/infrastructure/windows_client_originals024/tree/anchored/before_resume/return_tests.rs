use super::quiesce_tests::{launcher, OriginalSynthetic};
use super::*;
use crate::infrastructure::windows_client_originals024::{
    before_resume_record::{AttachmentStorage, Event, Record},
    tree::{CapturedTrees, ReturnGuard, ReturnStep},
};
use native::QuiescentLauncher;
struct Guard<'a> {
    native: &'a mut OriginalSynthetic,
    proof: &'a QuiescentLauncher,
    root: PathBuf,
    end: Instant,
    epoch: String,
    key: String,
    canonical: serde_json::Value,
    storage: AttachmentStorage,
    last: Record,
    raw: Vec<u8>,
    fail: &'static str,
    steps: usize,
}
impl ReturnGuard for Guard<'_> {
    fn check(
        &mut self,
        journal: &mut Journal,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        check(end)?;
        if end != self.end || claim.epoch() != self.epoch || claim.key() != self.key {
            return Err(invalid());
        }
        claim.check(&self.root, end)?;
        self.native.0.verify_before_resume_quiescent(self.proof)?;
        if journal.component_failed_canonical_for_test(end)? != self.canonical {
            return Err(invalid());
        }
        let stored = self.storage.read_data(end)?;
        if stored.last() != Some(&self.raw) {
            return Err(invalid());
        }
        Ok(())
    }
    fn record(&mut self, step: &ReturnStep, verified: bool, end: Instant) -> io::Result<()> {
        if verified && self.fail == "before-result" {
            return Err(io::Error::other(
                "injected after actual move before result attachment",
            ));
        }
        let event = step.event(verified);
        let next = self.last.next_data(&self.raw, event)?;
        self.raw = next.encode_data()?;
        self.last = next;
        self.storage.append_data(&self.raw, end)?;
        if !verified {
            self.steps += 1;
            if self.fail == "after-intent" {
                return Err(io::Error::other(
                    "injected after durable intent before native move",
                ));
            }
            if self.fail == "occupied-target" {
                let record: serde_json::Value = serde_json::from_slice(&self.raw).unwrap();
                let node = record["event"]["nodeId"].as_str().unwrap();
                let plan = &self.canonical["plan"];
                for root in plan["roots"].as_array().unwrap() {
                    if let Some(n) = root["nodes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["nodeId"] == node)
                    {
                        let target = if record["event"]["kind"] == "WorkRetireIntent" {
                            PathBuf::from(root["custody"].as_str().unwrap())
                                .join(format!("retired-{node}"))
                        } else {
                            PathBuf::from(root["root"].as_str().unwrap())
                                .join(n["relative"].as_str().unwrap())
                        };
                        std::fs::write(target, b"foreign occupied target; do not overwrite")
                            .unwrap();
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
fn setup_data(
    journal: &mut Journal,
    claim: &mut AnchoredClaim<'_>,
    scene: &Scene,
    end: Instant,
    native: &mut OriginalSynthetic,
) -> (
    AttachmentStorage,
    Record,
    Vec<u8>,
    serde_json::Value,
    QuiescentLauncher,
) {
    let prepared = native
        .0
        .prepare_before_first_resume_abort(Instant::now(), end)
        .unwrap();
    let proof = native.0.quiesce_before_first_resume(prepared).unwrap();
    let canonical = journal.component_failed_canonical_for_test(end).unwrap();
    let initial = Record::initial_data(
        [
            canonical["sessionId"].as_str().unwrap().into(),
            "22222222-2222-2222-2222-222222222222".into(),
            canonical["manifestDigest"].as_str().unwrap().into(),
            canonical["bindingDigest"].as_str().unwrap().into(),
            canonical["storagePlanDigest"].as_str().unwrap().into(),
            canonical["securityPolicyDigest"].as_str().unwrap().into(),
            hash::sha256_hex(b"component native prefix no product Owner"),
            "33333333-3333-3333-3333-333333333333".into(),
        ],
        ["1000", "11000", "1000"].map(String::from),
    )
    .unwrap();
    let mut storage = AttachmentStorage::default();
    let directory = native::create_private_directory(
        &scene
            .fixture
            .as_ref()
            .unwrap()
            .path()
            .join("abort-attachment"),
    )
    .unwrap();
    storage.initialize(directory, end).unwrap();
    let raw = initial.encode_data().unwrap();
    storage.append_data(&raw, end).unwrap();
    let b = &native.0.binding().process;
    let last = initial
        .next_data(
            &raw,
            Event::LauncherQuiesced {
                pid: b.pid,
                creation_filetime: b.creation_filetime.to_string(),
                forced: true,
                raw_exit: proof.forced_raw_exit(),
                job_total: 1,
                job_active: 0,
            },
        )
        .unwrap();
    let raw = last.encode_data().unwrap();
    storage.append_data(&raw, end).unwrap();
    claim.check(&scene.root, end).unwrap();
    (storage, last, raw, canonical, proof)
}
#[test]
fn wl2_r7_native_return_same_actual_claim_moves_only_original_files_and_keeps_all8_retired_work_and_canonical(
) {
    for fault in [0, 101, 141, 254, 255] {
        let scene = Scene::new();
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        let original = trees.each_ref().map(|t| t.plan());
        if fault != 0 {
            if fault != 255 && fault != 254 {
                journal.inject_driver_fault(fault);
            }
            let result = journal.prepare_initial_batches(
                super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            );
            assert_eq!(result.is_err(), fault != 255 && fault != 254);
            if fault == 254 {
                super::super::super::super::observation::open_trees(
                    &mut trees,
                    &mut journal,
                    &mut claim,
                    old,
                )
                .unwrap();
            }
        }
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for tree in &mut trees {
            tree.poisoned = true;
        }
        let (mut native, outputs) = launcher(&scene);
        let fresh = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
        let (storage, last, raw, canonical, proof) =
            setup_data(&mut journal, &mut claim, &scene, fresh, &mut native);
        let mut scope = Guard {
            native: &mut native,
            proof: &proof,
            root: scene.root.clone(),
            end: fresh,
            epoch: claim.epoch().into(),
            key: claim.key(),
            canonical: canonical.clone(),
            storage,
            last,
            raw,
            fail: "none",
            steps: 0,
        };
        let mut core = CapturedTrees::retain(trees, journal);
        let result = core
            .component_return_before_resume_for_test(&mut scope, &mut claim, fresh)
            .unwrap();
        assert_eq!(result["allOriginalFilesAtClient"], true);
        assert_eq!(result["locksReleased"], false);
        assert_eq!(result["closed"], false);
        assert!(core
            .component_return_before_resume_for_test(&mut scope, &mut claim, fresh)
            .is_err());
        let state = core.component_abort_return_state();
        assert!(!state.0 && state.1 && !state.4);
        for tree in &original {
            for node in tree["nodes"].as_array().unwrap() {
                let path = PathBuf::from(tree["root"].as_str().unwrap())
                    .join(node["relative"].as_str().unwrap());
                let file = open_node(
                    &path,
                    if node["kind"] == "DirectoryAnchor" {
                        Kind::Directory
                    } else if node["kind"] == "CoordinatorAnchor" {
                        Kind::Coordinator
                    } else {
                        Kind::File
                    },
                    true,
                )
                .unwrap();
                assert_eq!(
                    id(&file, node["kind"] == "DirectoryAnchor")
                        .unwrap()
                        .index
                        .to_string(),
                    node["fileId"].as_str().unwrap()
                );
                assert_eq!(
                    serde_json::to_value(native_stamp(&file, fresh).unwrap()).unwrap(),
                    node["security"]
                );
                if node["kind"] != "DirectoryAnchor" {
                    assert_eq!(
                        hash::sha256_file(&path).unwrap(),
                        node["contentDigest"].as_str().unwrap()
                    );
                }
            }
        }
        assert_eq!(
            core.journal()
                .component_failed_canonical_for_test(fresh)
                .unwrap(),
            canonical
        );
        let chain = scope.storage.read_data(fresh).unwrap();
        crate::infrastructure::windows_client_originals024::before_resume_record::validate_chain(
            &chain,
        )
        .unwrap();
        assert!(
            chain.iter().all(|r| {
                let v: serde_json::Value = serde_json::from_slice(r).unwrap();
                v["event"]["kind"] != "AbortRestoreVerified"
            }),
            "locks are still held; no final restore proof"
        );
        assert!(core.native_prefix().1);
        claim.check(&scene.root, fresh).unwrap();
        println!("r7 native return prefix{fault}: exact original ID/bytes/all8 to original targets, work retained, durable v1 per-node records and same claim; canonical unchanged, no lockrelease/Stop/Closed/productOwner acceptance");
        drop(scope);
        drop(proof);
        drop(native);
        drop(outputs);
        drop(core);
        drop(claim);
        lease.release(fresh).unwrap();
        drop(scene);
    }
}
#[test]
fn wl2_r7_native_return_failure_after_intent_move_or_conflict_keeps_native_prefix_and_never_retries(
) {
    for failure in ["after-intent", "before-result", "occupied-target"] {
        let scene = Scene::new();
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        journal
            .prepare_initial_batches(
                super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            )
            .unwrap();
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for tree in &mut trees {
            tree.poisoned = true;
        }
        let (mut native, outputs) = launcher(&scene);
        let fresh = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
        let (storage, last, raw, canonical, proof) =
            setup_data(&mut journal, &mut claim, &scene, fresh, &mut native);
        let mut scope = Guard {
            native: &mut native,
            proof: &proof,
            root: scene.root.clone(),
            end: fresh,
            epoch: claim.epoch().into(),
            key: claim.key(),
            canonical: canonical.clone(),
            storage,
            last,
            raw,
            fail: failure,
            steps: 0,
        };
        let mut core = CapturedTrees::retain(trees, journal);
        assert!(core
            .component_return_before_resume_for_test(&mut scope, &mut claim, fresh)
            .is_err());
        let state = core.component_abort_return_state();
        assert!(state.0 && !state.1 && state.4);
        assert_eq!(state.2,0,"sealed original work writer already retains DELETE and denies external write/delete; no redundant reopen");
        assert!(core
            .component_return_before_resume_for_test(&mut scope, &mut claim, fresh)
            .is_err());
        assert_eq!(state, core.component_abort_return_state());
        let native_prefix = core.native_prefix();
        assert!(native_prefix.1);
        assert!(native_prefix.2.iter().any(|s| s
            == if failure == "before-result" {
                "Retired"
            } else {
                "Working"
            }));
        assert_eq!(
            core.journal()
                .component_failed_canonical_for_test(fresh)
                .unwrap(),
            canonical
        );
        let records = scope.storage.read_data(fresh).unwrap();
        let last: serde_json::Value = serde_json::from_slice(records.last().unwrap()).unwrap();
        assert_eq!(last["event"]["kind"], "WorkRetireIntent");
        let node = last["event"]["nodeId"].as_str().unwrap();
        for root in canonical["plan"]["roots"].as_array().unwrap() {
            if let Some(n) = root["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["nodeId"] == node)
            {
                let actual = if failure == "before-result" {
                    PathBuf::from(root["custody"].as_str().unwrap()).join(format!("retired-{node}"))
                } else {
                    PathBuf::from(root["root"].as_str().unwrap())
                        .join(n["relative"].as_str().unwrap())
                };
                assert!(
                    std::fs::remove_file(&actual).is_err(),
                    "exact original work object still retained against deletion"
                );
                if failure == "occupied-target" {
                    assert_eq!(
                        std::fs::read(
                            PathBuf::from(root["custody"].as_str().unwrap())
                                .join(format!("retired-{node}"))
                        )
                        .unwrap(),
                        b"foreign occupied target; do not overwrite"
                    );
                }
            }
        }
        claim.check(&scene.root, fresh).unwrap();
        println!("r7 native return {failure}: failed driver/frozen exact work handle/pending intent/actual after-image retained; no overwrite/retry/completion synthesis; original claim stays held");
        drop(scope);
        drop(proof);
        drop(native);
        drop(outputs);
        drop(core);
        drop(claim);
        lease.release(fresh).unwrap();
        drop(scene);
    }
}
