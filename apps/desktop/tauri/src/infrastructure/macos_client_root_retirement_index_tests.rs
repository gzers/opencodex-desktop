//! Storage/index integration with explicit synthetic move permits. These tests
//! do not claim official service exit or production lifecycle authorization.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
use tests::Fixture;
#[test]
fn terminal_readback_does_not_trust_a_prior_successful_retired_inspection() {
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
    pending.inspect_retired().unwrap();
    let path = f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}.json", pending.id()));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"persistent change after successful inspection").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(pending
        .verify_retire_readback(closed.completion_sha256.as_deref().unwrap())
        .is_err());
    assert_eq!(
        fs::read(path).unwrap(),
        b"persistent change after successful inspection"
    );
    assert!(!f.context.working_directory.join(".codex").exists());
}
#[test]
fn root_stop_movement_registration_requires_actual_current_index_and_exact_original_source() {
    let f = Fixture::new();
    let pending = f.prepare();
    let plan = session_index::RestorePlan {
        id: pending.id().into(),
        sha256: pending.sha256().into(),
    };
    let mut forged = f.source.clone();
    forged.revision += 1;
    forged.state = session_index::State::RootRetirementPrepared;
    forged.root_retirement = Some(session_index::RootRetirement {
        source_revision: f.source.revision,
        plan: plan.clone(),
    });
    assert!(f
        ._index
        .root_retirement_registration(&f.source, &forged, &plan)
        .is_err());
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let registration = f
        ._index
        .root_retirement_registration(&f.source, &prepared, &plan)
        .unwrap();
    registration.require(&f.source, &plan).unwrap();
    let mut wrong = f.source.clone();
    wrong.state = session_index::State::Stopped;
    assert!(registration.require(&wrong, &plan).is_err());
    let mut wrong = plan.clone();
    wrong.sha256 = "a".repeat(64);
    assert!(registration.require(&f.source, &wrong).is_err());
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    registration.require(&f.source, &plan).unwrap();
    f._index.close_root_absence(&prepared, &proof).unwrap();
    assert!(registration.require(&f.source, &plan).is_err());
    println!(
        "ROOT_STOP_REGISTRATION_EVIDENCE {}",
        serde_json::json!({"accepted":false,"headCopyWithoutActualRegistrationRefused":true,"exactSourceEventRequired":true,"currentRegistrationVerifiedAcrossMove":true,"staleAfterCloseRefused":true,"storageOnlyNoNormalStopPermission":true})
    );
}
#[test]
fn indexed_root_move_plan_survives_absent_logical_root_and_only_current_readback_closes_it() {
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    assert_eq!(prepared.state, session_index::State::RootRetirementPrepared);
    assert_eq!(prepared.revision, 4);
    assert!(prepared
        .root_retirement
        .as_ref()
        .is_some_and(|r| r.source_revision == 3));
    assert!(f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .is_err());
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert!(!f.originals.profile.path().join(".codex").exists());
    assert_eq!(f._index.head().unwrap(), Some(prepared.clone()));
    let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
    assert_eq!(closed.state, session_index::State::RootAbsent);
    assert_eq!(closed.revision, 5);
    assert!(closed.root_retirement == prepared.root_retirement);
    assert!(closed.references == f.source.references);
    assert!(f._index.close_root_absence(&prepared, &proof).is_err());
    drop(pending);
    drop(f._index);
    let before = inventory(f.originals.data.path());
    let head = session_index::inspect(&f.context, f.originals.data.path())
        .unwrap()
        .unwrap();
    assert_eq!(head, closed);
    assert_eq!(inventory(f.originals.data.path()), before);
    let reopened = session_index::Index::open_existing(
        &f.context,
        f.originals.data.path(),
        &f.context.working_directory.join(".codex"),
    )
    .unwrap();
    assert_eq!(reopened.head().unwrap(), Some(closed.clone()));
    assert!(reopened.reserve_start().is_err());
    drop(reopened);
    let p = session_index::preview::inspect_preview(&f.context, f.originals.data.path());
    assert_eq!(p.status, session_index::preview::Status::Restored);
    assert!(session_index::generation::inspect_successor_eligibility(
        &f.context,
        f.originals.data.path()
    )
    .unwrap());
    assert_eq!(p.phase, Some(session_index::State::RootAbsent));
    assert_eq!(
        p.original_evidence,
        session_index::preview::Evidence::Verified
    );
    assert_eq!(
        p.completion_evidence,
        session_index::preview::Evidence::Verified
    );
    assert!(
        !p.can_continue
            && !p.can_finalize
            && !p.can_rollback
            && !p.can_finish_unchanged
            && !p.can_cancel_preparation
    );
    println!(
        "INDEXED_ROOT_ABSENT_EVIDENCE {}",
        serde_json::json!({"accepted":false,"source":f.source,"prepared":prepared,"closed":closed,"currentRootAbsent":true,"storageOnlyNoProductionAuthority":true,"files":inventory(f.originals.data.path())})
    );
}
fn inventory(root: &Path) -> Vec<serde_json::Value> {
    fn walk(root: &Path, path: &Path, v: &mut Vec<serde_json::Value>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(root, &p, v);
            } else {
                let b = fs::read(&p).unwrap();
                v.push(serde_json::json!({"path":p.strip_prefix(root).unwrap(),"bytes":b.len(),"sha256":digest(&b),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,b)}));
            }
        }
    }
    let mut v = Vec::new();
    walk(root, root, &mut v);
    v.sort_by_key(|x| x["path"].as_str().unwrap().to_string());
    v
}
#[test]
fn indexed_root_move_before_terminal_is_readonly_pending_even_after_directory_relocation() {
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    drop(pending);
    drop(f._index);
    assert_eq!(
        session_index::inspect(&f.context, f.originals.data.path()).unwrap(),
        Some(prepared)
    );
    let preview = session_index::preview::inspect_preview(&f.context, f.originals.data.path());
    assert_eq!(preview.status, session_index::preview::Status::Pending);
    assert_eq!(
        preview.completion_evidence,
        session_index::preview::Evidence::NotRecorded
    );
    assert!(!preview.can_finalize && !preview.can_continue);
}
#[test]
fn indexed_root_absence_refuses_new_user_root_or_modified_readback_without_overwriting() {
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    f._index.close_root_absence(&prepared, &proof).unwrap();
    let root = f.originals.profile.path().join(".codex");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    fs::write(root.join("outside"), b"new user data").unwrap();
    assert!(f._index.head().is_err());
    drop(pending);
    drop(f._index);
    assert!(session_index::inspect(&f.context, f.originals.data.path()).is_err());
    assert_eq!(fs::read(root.join("outside")).unwrap(), b"new user data");
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
    let path = f.originals.directory.path().join(format!(
        "root-retirement-{}-Retire-readback.json",
        pending.id()
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"{}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(f._index.head().is_err());
    drop(pending);
    drop(f._index);
    assert!(session_index::inspect(&f.context, f.originals.data.path()).is_err());
    assert_eq!(fs::read(path).unwrap(), b"{}");
    assert_eq!(closed.state, session_index::State::RootAbsent);
}
#[test]
fn indexed_root_move_reference_and_source_cannot_be_rebased() {
    let f = Fixture::new();
    let pending = f.prepare();
    let mut wrong = f.source.clone();
    wrong.revision += 1;
    assert!(f
        ._index
        .record_root_retirement(&wrong, pending.id(), pending.sha256())
        .is_err());
    assert_eq!(f._index.head().unwrap(), Some(f.source.clone()));
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    assert!(f
        ._index
        .advance(
            &prepared,
            session_index::State::Failed,
            prepared.references.clone()
        )
        .is_err());
    assert_eq!(f._index.head().unwrap(), Some(prepared));
}

#[test]
fn indexed_root_absence_expired_current_proof_does_not_close_or_expire_readonly_facts() {
    let f = Fixture::new();
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let mut proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    proof.deadline = Instant::now();
    assert!(f._index.close_root_absence(&prepared, &proof).is_err());
    assert_eq!(f._index.head().unwrap(), Some(prepared.clone()));
    drop(pending);
    drop(f._index);
    assert_eq!(
        session_index::inspect(&f.context, f.originals.data.path()).unwrap(),
        Some(prepared)
    );
}
#[test]
fn indexed_root_absence_real_process_exit_keeps_exact_plan_direction_and_readonly_receipt_binding()
{
    const CHILD: &str = "OCXD024_INDEXED_ROOT_EXIT";
    if let Some(path) = std::env::var_os(CHILD) {
        let f = Fixture::at(std::path::PathBuf::from(path));
        let pending = f.prepare();
        let prepared = f
            ._index
            .record_root_retirement(&f.source, pending.id(), pending.sha256())
            .unwrap();
        let point = std::env::var("OCXD024_INDEXED_ROOT_POINT").unwrap();
        if point == "plan_registered" {
            std::process::exit(91);
        }
        let proof = pending
            .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
            .unwrap();
        if point == "root_moved" {
            std::process::exit(91);
        }
        f._index.close_root_absence(&prepared, &proof).unwrap();
        if point == "root_closed" {
            std::process::exit(91);
        }
        panic!("exit boundary missing");
    }
    for point in ["plan_registered", "root_moved", "root_closed"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_retirement::index_tests::indexed_root_absence_real_process_exit_keeps_exact_plan_direction_and_readonly_receipt_binding","--nocapture"]).env(CHILD,&root).env("OCXD024_INDEXED_ROOT_POINT",point).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let data = root.join("desktop");
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let before = inventory(&data);
        let head = session_index::inspect(&context, &data).unwrap().unwrap();
        let expected = if point == "root_closed" {
            session_index::State::RootAbsent
        } else {
            session_index::State::RootRetirementPrepared
        };
        assert_eq!(head.state, expected);
        assert_eq!(head.revision, if point == "root_closed" { 5 } else { 4 });
        assert_eq!(
            root.join("profile/.codex").exists(),
            point == "plan_registered"
        );
        let reopened =
            session_index::Index::open_existing(&context, &data, &root.join("profile/.codex"))
                .unwrap();
        assert_eq!(reopened.head().unwrap(), Some(head.clone()));
        assert!(reopened.reserve_start().is_err());
        drop(reopened);
        assert_eq!(inventory(&data), before);
        println!(
            "INDEXED_ROOT_EXIT_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"childExitCode":out.status.code(),"head":head,"currentRootAbsent":point!="plan_registered","storageOnlyNoProductionNativePermission":true,"files":before})
        );
    }
}
