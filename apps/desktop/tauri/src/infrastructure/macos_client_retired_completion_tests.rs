//! Complete already-retired review fixture; no production native Stop proof.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
};
struct Fixture {
    _temp: tempfile::TempDir,
    context: ProcessContext,
    index: Index,
    head: Head,
    originals: Originals,
    movement_proof: Option<super::super::super::root_retirement::AuthorizedReadback>,
}
impl Fixture {
    fn new(retire: bool) -> Self {
        let (temp, context, index, mut head, originals) =
            super::super::super::root_retirement::fixture_captured_generation();
        for name in [
            ".opencodex-native-main.claim.sqlite",
            ".opencodex-native-main.owner.sqlite",
        ] {
            let path = originals.client.path().join(name);
            let c = rusqlite::Connection::open(&path).unwrap();
            c.execute_batch("CREATE TABLE marker(v INTEGER); INSERT INTO marker VALUES(1)")
                .unwrap();
            c.close().unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let ready = originals.seal(Phase::Ready).unwrap();
        let raw=encode(&serde_json::json!({"schemaVersion":1,"scopeSha256":head.references.as_ref().unwrap().scope_sha256,"readySha256":ready,"recordSha256":"a".repeat(64),"generation":[42,unsafe{libc::geteuid()},1,1],"parentGeneration":[41,unsafe{libc::geteuid()},1,1],"port":12345})).unwrap();
        super::super::super::create(&originals.directory, "startup-live.json", &raw).unwrap();
        let phases = [
            (Phase::Ready, ready),
            (Phase::PreStop, originals.seal(Phase::PreStop).unwrap()),
            (Phase::Stopped, originals.seal(Phase::Stopped).unwrap()),
        ];
        for (i, state) in [State::Ready, State::StopPrepared, State::Stopped]
            .into_iter()
            .enumerate()
        {
            head.revision += 1;
            head.state = state;
            let refs = head.references.as_mut().unwrap();
            refs.phases = phases[..=i].to_vec();
            refs.live_sha256 = Some(digest(&raw));
            let c = index.connection().unwrap();
            let b = encode(&head).unwrap();
            c.execute(
                "INSERT INTO events(revision,document) VALUES(?1,?2)",
                params![head.revision, &b],
            )
            .unwrap();
            c.execute(
                "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
                params![head.revision, &b],
            )
            .unwrap();
        }
        let artifact = restore::Pending::prepare_fixture(
            &originals,
            super::super::restore_registration_tests::evidence(),
        )
        .unwrap();
        head = index
            .record_restore_plan(&head, artifact.id(), artifact.sha256())
            .unwrap();
        let plan = Pending::prepare(&context, &originals, &head).unwrap();
        head = index
            .record_root_retirement(&head, plan.id(), plan.sha256())
            .unwrap();
        let movement_proof = if retire {
            Some(
                plan.fixture_apply_for_index(Instant::now() + std::time::Duration::from_secs(10))
                    .unwrap(),
            )
        } else {
            None
        };
        drop(plan);
        drop(artifact);
        assert_eq!(head.revision, 8);
        assert_eq!(index.head().unwrap(), Some(head.clone()));
        Self {
            _temp: temp,
            context,
            index,
            head,
            originals,
            movement_proof,
        }
    }
    fn review(&self) -> io::Result<Option<Review>> {
        let history = vault_history::read(self.index.connection()?, &self.head)?;
        let lineage = root_lineage::parse(&self.index.binding, &history)?;
        review(
            &self.context,
            self.index.data.path(),
            &self.head,
            &history,
            &lineage.current().origin,
        )
    }
}
#[test]
fn retired_completion_review_is_bound_to_existing_released_tree_and_pending_head() {
    let f = Fixture::new(true);
    let first = f.review().unwrap().unwrap();
    let second = f.review().unwrap().unwrap();
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(first.receipt_sha256, second.receipt_sha256);
    assert!(!f.context.working_directory.join(".codex").exists());
    completion::prove_optional_released(
        &f.originals
            .directory
            .child_private(
                &format!(
                    "root-retired-{}",
                    f.head.root_retirement.as_ref().unwrap().plan.id
                ),
                false,
            )
            .unwrap(),
        &first.locks.clone().map(Some),
        Instant::now() + std::time::Duration::from_secs(10),
    )
    .unwrap();
    let data = f.index.data.path().to_path_buf();
    let context = f.context.clone();
    let head = f.head.clone();
    drop(f.index);
    let p = preview::inspect_preview(&context, &data);
    assert!(p.can_finalize && !p.can_continue && !p.can_rollback);
    assert_eq!(p.phase, Some(State::RootRetirementPrepared));
    assert_eq!(p.completion_evidence, preview::Evidence::Unregistered);
    assert_eq!(
        p.confirmation_digest.as_deref(),
        Some(first.fingerprint.as_str())
    );
    assert_eq!(p.revision, Some(head.revision));
    // Synthetic program is not a production runtime. Correct review alone must
    // never allow a caller to manufacture a live recovery completion proof.
    assert!(finalize(
        &context,
        &data,
        super::super::finalization::Request {
            attempt_id: head.ticket,
            revision: head.revision,
            confirmation_digest: first.fingerprint
        },
        Instant::now() + std::time::Duration::from_secs(10)
    )
    .is_err());
    assert_eq!(
        inspect(&context, &data).unwrap().unwrap().state,
        State::RootRetirementPrepared
    );
}
#[test]
fn retired_completion_review_new_confirmation_keeps_old_postcommit_demotion_history() {
    let mut f = Fixture::new(true);
    let initial = f.review().unwrap().unwrap();
    let mut proof = f.movement_proof.take().unwrap();
    proof.fixture_limit_deadline(Instant::now() + std::time::Duration::from_secs(1));
    let closed = f
        .index
        .close_root_absence_with_eligibility_checked(&f.head, &proof, &|| {
            std::thread::sleep(std::time::Duration::from_millis(1100));
            Ok(())
        });
    assert!(closed.is_err());
    let head = f.index.head().unwrap().unwrap();
    assert_eq!(head.revision, 10);
    assert_eq!(head.state, State::RootRetirementPrepared);
    f.head = head;
    let current = f.review().unwrap().unwrap();
    assert_ne!(current.fingerprint, initial.fingerprint);
    assert_eq!(current.receipt_sha256, initial.receipt_sha256);
    let history = vault_history::read(f.index.connection().unwrap(), &f.head).unwrap();
    assert_eq!(history[8].state, State::RootAbsent);
    assert_eq!(history[9].state, State::RootRetirementPrepared);
    assert!(f.head.completion_sha256.is_none());
}
#[test]
fn retired_completion_review_proves_lock_release_instead_of_creating_missing_locks() {
    let f = Fixture::new(true);
    let observed = f.review().unwrap().unwrap();
    let archive = f
        .originals
        .directory
        .child_private(
            &format!(
                "root-retired-{}",
                f.head.root_retirement.as_ref().unwrap().plan.id
            ),
            false,
        )
        .unwrap();
    let path = archive.path().join(".opencodex-native-main.owner.sqlite");
    // Same-process distinct connection still competes through SQLite; the
    // returned error does not become a success or trigger any file repair.
    let owner = rusqlite::Connection::open(&path).unwrap();
    owner.execute_batch("BEGIN EXCLUSIVE").unwrap();
    assert!(completion::prove_optional_released(
        &archive,
        &observed.locks.clone().map(Some),
        Instant::now() + std::time::Duration::from_secs(10)
    )
    .is_err());
    owner.execute_batch("ROLLBACK").unwrap();
    owner.close().unwrap();
    completion::prove_optional_released(
        &archive,
        &observed.locks.map(Some),
        Instant::now() + std::time::Duration::from_secs(10),
    )
    .unwrap();
    assert!(path.exists());
}
#[test]
fn retired_completion_review_never_offers_finalize_for_unmoved_or_missing_receipt() {
    let f = Fixture::new(false);
    assert!(f.review().unwrap().is_none());
    let f = Fixture::new(true);
    let movement = f.head.root_retirement.as_ref().unwrap();
    let path = f.originals.directory.path().join(format!(
        "root-retirement-{}-Retire-readback.json",
        movement.plan.id
    ));
    fs::remove_file(&path).unwrap();
    assert!(f.review().unwrap().is_none());
    assert!(!path.exists());
}
#[test]
fn retired_completion_review_rejects_changed_current_grok_root_plan_and_all_evidence() {
    for mutation in [
        "plan",
        "intent",
        "receipt",
        "frame",
        "scope",
        "live",
        "archive",
        "grok",
        "root",
        "namespace",
    ] {
        let f = Fixture::new(true);
        let before = f.review().unwrap().unwrap().fingerprint;
        assert!(!before.is_empty());
        let movement = f.head.root_retirement.as_ref().unwrap();
        let name = match mutation {
            "plan" => Some(format!("root-retirement-{}.json", movement.plan.id)),
            "intent" => Some(format!(
                "root-retirement-{}-Retire-intent.json",
                movement.plan.id
            )),
            "receipt" => Some(format!(
                "root-retirement-{}-Retire-readback.json",
                movement.plan.id
            )),
            "frame" => Some("ready.json".into()),
            "scope" => Some("startup-scope.json".into()),
            "live" => Some("startup-live.json".into()),
            _ => None,
        };
        if let Some(name) = name {
            let path = f.originals.directory.path().join(name);
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            fs::write(&path, b"kept corrupt").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        }
        match mutation {
            "archive" => {
                fs::write(
                    f.originals
                        .directory
                        .path()
                        .join(format!("root-retired-{}/user", movement.plan.id)),
                    b"keep",
                )
                .unwrap();
            }
            "grok" => {
                let path = f.context.working_directory.join(".grok");
                fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
                fs::write(path.join("config.toml"), b"outside=1").unwrap();
            }
            "root" => {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(f.context.working_directory.join(".codex"))
                    .unwrap();
            }
            "namespace" => {
                let path = f
                    .index
                    .data
                    .path()
                    .join("manager-state/client-root-generations");
                fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(path.join(uuid::Uuid::new_v4().to_string()))
                    .unwrap();
            }
            _ => (),
        }
        assert!(!matches!(f.review(), Ok(Some(_))), "{mutation}");
        assert_eq!(
            read_head(f.index.connection().unwrap()).unwrap(),
            Some(f.head)
        );
    }
}
