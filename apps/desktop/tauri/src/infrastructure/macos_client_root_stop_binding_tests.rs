//! Storage/source binding tests only. Synthetic Ready/Stopped frames and PID
//! values never construct RootRetirementAuthority or a production Stop proof.
use super::tests::Fixture;
use super::*;
use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
struct RootFixture {
    f: Fixture,
    session: StartupSession,
    head: session_index::Head,
    _index: session_index::Index,
}
impl RootFixture {
    fn new() -> Self {
        let f = Fixture::new();
        fs::remove_dir(&f.client).unwrap();
        let published = root_creation::reserve(
            &f.data,
            &f.context.working_directory,
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap()
        .publish()
        .unwrap();
        let (index, preparing) =
            session_index::Index::register_first_root(&f.context, &f.data, published).unwrap();
        let reserved = index.reserve_first_root_capture(&preparing).unwrap();
        let reservation = index.capture_reservation(&reserved).unwrap();
        let originals = Originals::begin_reserved_profile(
            &f.data,
            &f.client,
            &f.context.working_directory,
            &reservation,
        )
        .unwrap();
        let (program, selection) = source(&f.context).unwrap();
        let mut session =
            StartupSession::persist(&f.context, originals, program, selection).unwrap();
        let captured = index
            .record_first_root_originals(&f.context, &reserved, session.references().unwrap())
            .unwrap();
        drop(reservation);
        let ready = session.originals.seal(Phase::Ready).unwrap();
        session.originals.seal(Phase::PreStop).unwrap();
        session.originals.seal(Phase::Stopped).unwrap();
        let live = Live {
            schema_version: 1,
            scope_sha256: session.scope_sha256.clone(),
            ready_sha256: ready,
            record_sha256: "a".repeat(64),
            generation: [42, unsafe { libc::geteuid() }.into(), 1, 1],
            parent_generation: [41, unsafe { libc::geteuid() }.into(), 1, 1],
            port: 12345,
        };
        let raw = encode(&live).unwrap();
        create(&session.originals.directory, "startup-live.json", &raw).unwrap();
        session.live_sha256 = Some(digest(&raw));
        let head = session_index::Head {
            revision: captured.revision + 3,
            state: session_index::State::Stopped,
            references: Some(session.references().unwrap()),
            ..captured
        };
        Self {
            f,
            session,
            head,
            _index: index,
        }
    }
    fn verify(
        &self,
        plan: &session_index::RestorePlan,
        scope: &Scope,
        live: &Live,
    ) -> AppResult<()> {
        self.session
            .verify_root_stop_binding(&self.f.context, &self.head, plan, scope, live)
    }
}
#[test]
fn root_stop_binding_follows_only_original_plan_archive_and_preserves_logical_scope() {
    let f = RootFixture::new();
    let scope = f.session.scope(&f.f.context).unwrap();
    let live = f.session.live(&f.f.context).unwrap();
    let pending =
        root_retirement::Pending::prepare(&f.f.context, &f.session.originals, &f.head).unwrap();
    let plan = session_index::RestorePlan {
        id: pending.id().into(),
        sha256: pending.sha256().into(),
    };
    f.verify(&plan, &scope, &live).unwrap();
    pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert!(!f.f.client.exists());
    f.verify(&plan, &scope, &live).unwrap();
    let raw = fs::read(
        f.session
            .originals
            .directory
            .path()
            .join("startup-scope.json"),
    )
    .unwrap();
    let stored: Scope = parse(&raw).unwrap();
    assert_eq!(stored.client, f.f.client);
    let mut wrong = plan.clone();
    wrong.sha256 = "b".repeat(64);
    assert!(f.verify(&wrong, &scope, &live).is_err());
    let mut wrong_scope = scope.clone();
    wrong_scope.selection.version = "2.99.1".into();
    assert!(f.verify(&plan, &wrong_scope, &live).is_err());
    let mut wrong_live = live.clone();
    wrong_live.record_sha256 = "b".repeat(64);
    assert!(f.verify(&plan, &scope, &wrong_live).is_err());
    fs::create_dir(&f.f.client).unwrap();
    fs::write(f.f.client.join("outside"), b"new user root").unwrap();
    assert!(f.verify(&plan, &scope, &live).is_err());
    assert_eq!(
        fs::read(f.f.client.join("outside")).unwrap(),
        b"new user root"
    );
    println!(
        "ROOT_STOP_BINDING_EVIDENCE {}",
        serde_json::json!({"accepted":false,"sourceAndFramesVerifiedBeforeAndAfterMove":true,"logicalScopeUnchanged":true,"wrongPlanScopeLiveAndNewRootRefused":true,"syntheticFramesNoStopPermission":true})
    );
}
#[test]
fn root_stop_binding_rejects_modified_historical_frames_scope_and_live_after_move() {
    let f = RootFixture::new();
    let scope = f.session.scope(&f.f.context).unwrap();
    let live = f.session.live(&f.f.context).unwrap();
    let pending =
        root_retirement::Pending::prepare(&f.f.context, &f.session.originals, &f.head).unwrap();
    let plan = session_index::RestorePlan {
        id: pending.id().into(),
        sha256: pending.sha256().into(),
    };
    pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    for name in [
        "original.json",
        "ready.json",
        "pre-stop.json",
        "stopped.json",
        "startup-scope.json",
        "startup-live.json",
    ] {
        let path = f.session.originals.directory.path().join(name);
        // Phase names follow the real frame contract, never guess a fallback.
        let raw = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{}").unwrap();
        assert!(
            f.verify(&plan, &scope, &live).is_err(),
            "changed {name} accepted"
        );
        assert_eq!(fs::read(&path).unwrap(), b"{}");
        fs::write(&path, raw).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        f.verify(&plan, &scope, &live).unwrap();
    }
}
