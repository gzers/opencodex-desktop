use super::*;
use crate::{
    infrastructure::macos_client_originals::session_index::{self, preview},
    modules::runtime::RuntimeHandle,
    state::ProcessContext,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Instant};
#[test]
#[ignore = "read-only final client recovery gate on retained owned native Profile"]
fn inspect_owned_client_gate_after_rearm_without_service_actions() {
    let root = PathBuf::from(std::env::var_os("OCXD024_REARM_GATE_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join(".codex-desktop/desktop");
    let home = profile.join(".codex-desktop/opencodex/home");
    let runtime = RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space"));
    let source = crate::infrastructure::status_source::OfficialStatusSource::new(
        runtime,
        &profile,
        crate::modules::process::EnvironmentPolicy {
            home: Some(profile.clone().into_os_string()),
            opencodex_home: home,
            ..Default::default()
        },
    )
    .with_client_recovery_root(data);
    let snapshot = tauri::async_runtime::block_on(async {
        tokio::task::spawn_blocking(move || {
            let mut collector = crate::modules::status::StatusCollector::new(source);
            super::super::status_snapshot_from_collector(&mut collector)
        })
        .await
        .unwrap()
    })
    .unwrap();
    assert_eq!(
        snapshot.facts.client_recovery,
        crate::modules::status::ClientRecoveryState::NotRequired
    );
    assert!(snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart);
    assert!(snapshot.pid.is_none() && snapshot.port.is_none());
    fs::write(
        root.join("rearm-final-live-status.json"),
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();
}
#[test]
#[ignore = "explicit same owned rolled-back Profile; prepare then separately confirm original restoration"]
fn rearm_owned_rollback_then_explicitly_continue_originals() {
    let root = PathBuf::from(std::env::var_os("OCXD024_REARM_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join(".codex-desktop/desktop");
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space")),
        working_directory: profile.clone(),
        opencodex_home: profile.join(".codex-desktop/opencodex/home"),
    };
    let before = preview::inspect_preview(&context, &data);
    let prepare = if before.phase == Some(session_index::State::RestoreRolledBack) {
        let request = session_index::rearm::review(&context, &data)
            .unwrap()
            .expect("registered rolled-back plan");
        let start = Instant::now();
        let result = session_index::rearm::prepare(&context, &data, request);
        let elapsed = start.elapsed().as_millis();
        let report = serde_json::json!({"accepted":false,"elapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string())});
        fs::write(
            root.join("explicit-rearm-summary.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
        let outcome = result.unwrap();
        assert_eq!(outcome.status, "restore_prepared");
        assert_eq!(outcome.revision, before.revision.unwrap() + 1);
        assert!(elapsed < 10000);
        Some(outcome)
    } else {
        None
    };
    let ready = preview::inspect_preview(&context, &data);
    assert!(ready.can_continue && !ready.can_finalize);
    let request = Request {
        attempt_id: ready.attempt_id.clone().unwrap(),
        revision: ready.revision.unwrap(),
        confirmation_digest: ready.confirmation_digest.clone().unwrap(),
    };
    let start = Instant::now();
    let result = session_index::continuation::resume(&context, &data, request);
    let elapsed = start.elapsed().as_millis();
    let report = serde_json::json!({"accepted":false,"separateExplicitConfirmations":true,"prepare":prepare,"continueElapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),"reopened":preview::inspect_preview(&context,&data)});
    fs::write(
        root.join("rearm-continue-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    let outcome = result.unwrap();
    assert_eq!(outcome.status, "restored");
    assert_eq!(outcome.revision, ready.revision.unwrap() + 1);
    assert!(elapsed < 10000);
}
#[test]
#[ignore = "explicit owned Profile retained after native publication; independent rollback action only"]
fn rollback_owned_interrupted_plan_keeps_original_restoration_required() {
    let root = PathBuf::from(std::env::var_os("OCXD024_ROLLBACK_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join(".codex-desktop/desktop");
    let home = profile.join(".codex-desktop/opencodex/home");
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space")),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let request = session_index::rollback::review(&context, &data)
        .unwrap()
        .expect("exact pending rollback review");
    let index = data.join("manager-state/client-session-index.sqlite");
    let before = fs::read(&index).unwrap();
    if std::env::var_os("OCXD024_ROLLBACK_ONCE").is_none() {
        for wrong in [
            Request {
                revision: request.revision + 1,
                ..request.clone()
            },
            Request {
                confirmation_digest: "a".repeat(64),
                ..request.clone()
            },
        ] {
            assert!(session_index::rollback::rollback(&context, &data, wrong).is_err());
            assert_eq!(fs::read(&index).unwrap(), before);
        }
        let config = profile.join(".codex/config.toml");
        let original = fs::read(&config).unwrap();
        fs::write(&config, b"outside rollback edit").unwrap();
        assert!(session_index::rollback::review(&context, &data)
            .unwrap()
            .is_none());
        assert!(session_index::rollback::rollback(&context, &data, request.clone()).is_err());
        assert_eq!(fs::read(&config).unwrap(), b"outside rollback edit");
        assert_eq!(fs::read(&index).unwrap(), before);
        fs::write(config, original).unwrap();
    }
    let started = Instant::now();
    let result = session_index::rollback::rollback(&context, &data, request.clone());
    let elapsed = started.elapsed().as_millis();
    let reopened = preview::inspect_preview(&context, &data);
    let report = serde_json::json!({"accepted":false,"elapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),"reopened":reopened,"noHistoricalExitProofReconstructed":true,"noProductRollbackCommandOrButtonYet":true});
    fs::write(
        root.join("explicit-rollback-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    let outcome = result.unwrap();
    assert!(elapsed < 10000);
    assert_eq!(outcome.status, "rolled_back");
    assert_eq!(outcome.revision, request.revision + 1);
    assert_eq!(reopened.status, preview::Status::Pending);
    assert_eq!(
        reopened.phase,
        Some(session_index::State::RestoreRolledBack)
    );
    assert!(!reopened.can_continue && !reopened.can_finalize && !reopened.can_rollback);
    let after = fs::read(&index).unwrap();
    assert!(session_index::rollback::review(&context, &data)
        .unwrap()
        .is_none());
    assert!(session_index::rollback::rollback(&context, &data, request).is_err());
    assert_eq!(fs::read(&index).unwrap(), after);
    let client = profile.join(".codex");
    let index = session_index::Index::open_existing(&context, &data, &client).unwrap();
    assert!(index.reserve_start().is_err());
    drop(index);
    let source = crate::infrastructure::status_source::OfficialStatusSource::new(
        context.runtime.clone(),
        &profile,
        crate::modules::process::EnvironmentPolicy {
            home: Some(profile.clone().into_os_string()),
            opencodex_home: home,
            ..Default::default()
        },
    )
    .with_client_recovery_root(data);
    let snapshot = tauri::async_runtime::block_on(async {
        tokio::task::spawn_blocking(move || {
            let mut collector = crate::modules::status::StatusCollector::new(source);
            super::super::status_snapshot_from_collector(&mut collector)
        })
        .await
        .unwrap()
    })
    .unwrap();
    assert_eq!(
        snapshot.facts.client_recovery,
        crate::modules::status::ClientRecoveryState::Required
    );
    assert!(!snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart);
    assert!(snapshot.pid.is_none() && snapshot.port.is_none());
    fs::write(
        root.join("rollback-live-status.json"),
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();
}
#[test]
#[ignore = "explicit owned Profile retained after first native artifact publication; no service actions"]
fn continue_owned_interrupted_plan_with_fresh_native_authority() {
    let root = PathBuf::from(std::env::var_os("OCXD024_CONTINUE_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join(".codex-desktop/desktop");
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space")),
        working_directory: profile.clone(),
        opencodex_home: profile.join(".codex-desktop/opencodex/home"),
    };
    let before = preview::inspect_preview(&context, &data);
    assert!(before.can_continue && !before.can_finalize && !before.can_rollback);
    let request = Request {
        attempt_id: before.attempt_id.clone().unwrap(),
        revision: before.revision.unwrap(),
        confirmation_digest: before.confirmation_digest.unwrap(),
    };
    let index = data.join("manager-state/client-session-index.sqlite");
    let index_before = fs::read(&index).unwrap();
    if std::env::var_os("OCXD024_CONTINUE_ONCE").is_none() {
        for wrong in [
            Request {
                revision: request.revision + 1,
                ..request.clone()
            },
            Request {
                confirmation_digest: "a".repeat(64),
                ..request.clone()
            },
            Request {
                attempt_id: uuid::Uuid::new_v4().to_string(),
                ..request.clone()
            },
        ] {
            assert!(session_index::continuation::resume(&context, &data, wrong).is_err());
            assert_eq!(fs::read(&index).unwrap(), index_before);
        }
        let config = profile.join(".codex/config.toml");
        let original = fs::read(&config).unwrap();
        fs::write(&config, b"external config edit while pending").unwrap();
        assert!(!preview::inspect_preview(&context, &data).can_continue);
        assert!(session_index::continuation::resume(&context, &data, request.clone()).is_err());
        assert_eq!(
            fs::read(&config).unwrap(),
            b"external config edit while pending"
        );
        assert_eq!(fs::read(&index).unwrap(), index_before);
        fs::write(config, original).unwrap();
    }
    let t = Instant::now();
    let result = session_index::continuation::resume(&context, &data, request.clone());
    let elapsed = t.elapsed().as_millis();
    let report = serde_json::json!({"accepted":false,"elapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),"reopened":preview::inspect_preview(&context,&data),"noHistoricalExitProofReconstructed":true});
    fs::write(
        root.join("explicit-continue-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    let outcome = result.unwrap();
    assert!(elapsed < 10000);
    assert_eq!(outcome.status, "restored");
    assert_eq!(outcome.revision, request.revision + 1);
    let after = fs::read(&index).unwrap();
    assert!(session_index::continuation::resume(&context, &data, request).is_err());
    assert_eq!(fs::read(&index).unwrap(), after);
}
#[test]
#[ignore = "explicit retained owned Profile whose ordinary Stop completed artifacts then intentionally failed before index closure"]
fn finalize_owned_completed_fixture_with_fresh_admission() {
    let root = PathBuf::from(std::env::var_os("OCXD024_FINALIZE_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join(".codex-desktop/desktop");
    let home = profile.join(".codex-desktop/opencodex/home");
    let runtime = RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space"));
    let context = ProcessContext {
        runtime,
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let before = preview::inspect_preview(&context, &data);
    assert_eq!(before.phase, Some(session_index::State::RestorePrepared));
    assert!(before.can_finalize);
    assert!(!before.can_continue && !before.can_rollback);
    let request = Request {
        attempt_id: before.attempt_id.clone().unwrap(),
        revision: before.revision.unwrap(),
        confirmation_digest: before.confirmation_digest.clone().unwrap(),
    };
    // Target only a retained failed finalization, without repeating already
    // passed negative cases or starting/stopping the official service again.
    if std::env::var_os("OCXD024_FINALIZE_ONCE").is_some() {
        let started = Instant::now();
        let result = session_index::finalization::finalize(&context, &data, request);
        let elapsed = started.elapsed().as_millis();
        let report = serde_json::json!({"accepted":false,"scope":"single explicit finalize of retained owned pending index; no service actions", "elapsedMs":elapsed,
            "outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),
            "reopened":preview::inspect_preview(&context,&data)});
        fs::write(
            root.join("explicit-finalize-once.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
        let outcome = result.unwrap();
        assert!(elapsed < 10000);
        assert_eq!(outcome.revision, before.revision.unwrap() + 1);
        assert_eq!(outcome.status, "restored");
        // After durable closure, a legitimate user cache edit must not reopen
        // recovery or be compared against the old artifact postimages.
        let cache = profile.join(".codex/models_cache.json");
        assert!(!cache.exists());
        fs::write(&cache, b"owned user edit after closed recovery").unwrap();
        assert_eq!(
            preview::inspect_preview(&context, &data).status,
            preview::Status::Restored
        );
        assert_eq!(
            fs::read(&cache).unwrap(),
            b"owned user edit after closed recovery"
        );
        fs::remove_file(cache).unwrap();
        return;
    }
    let index = data.join("manager-state/client-session-index.sqlite");
    let index_before = fs::read(&index).unwrap();
    for wrong in [
        Request {
            confirmation_digest: "a".repeat(64),
            ..request.clone()
        },
        Request {
            revision: request.revision + 1,
            ..request.clone()
        },
        Request {
            attempt_id: uuid::Uuid::new_v4().to_string(),
            ..request.clone()
        },
    ] {
        assert!(session_index::finalization::finalize(&context, &data, wrong).is_err());
        assert_eq!(fs::read(&index).unwrap(), index_before);
    }
    let config = profile.join(".codex/config.toml");
    let config_before = fs::read(&config).unwrap();
    fs::write(&config, b"user edit while recovery pending").unwrap();
    assert!(!preview::inspect_preview(&context, &data).can_finalize);
    assert!(session_index::finalization::finalize(&context, &data, request.clone()).is_err());
    assert_eq!(
        fs::read(&config).unwrap(),
        b"user edit while recovery pending"
    );
    assert_eq!(fs::read(&index).unwrap(), index_before);
    fs::write(&config, &config_before).unwrap();
    let path = home.join("runtime-port.json");
    assert!(!path.exists());
    fs::write(
        &path,
        serde_json::to_vec(
            &serde_json::json!({"pid":std::process::id(),"port":1,"hostname":"127.0.0.1"}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(session_index::finalization::finalize(&context, &data, request.clone()).is_err());
    assert_eq!(fs::read(&index).unwrap(), index_before);
    assert!(path.exists());
    fs::remove_file(path).unwrap();
    let current = preview::inspect_preview(&context, &data);
    assert!(current.can_finalize);
    assert_eq!(
        current.confirmation_digest.as_deref(),
        Some(request.confirmation_digest.as_str())
    );
    let t = Instant::now();
    let outcome = session_index::finalization::finalize(&context, &data, request.clone()).unwrap();
    let elapsed = t.elapsed().as_millis();
    assert!(elapsed < 10000);
    assert_eq!(outcome.revision, 7);
    assert_eq!(outcome.status, "restored");
    assert_eq!(fs::read(&config).unwrap(), config_before);
    let restored = preview::inspect_preview(&context, &data);
    assert_eq!(restored.status, preview::Status::Restored);
    assert!(!restored.can_finalize);
    let after = fs::read(&index).unwrap();
    assert!(session_index::finalization::finalize(&context, &data, request).is_err());
    assert_eq!(fs::read(&index).unwrap(), after);
    let report = serde_json::json!({"accepted":false,"scope":"explicit metadata finalization after prior actual file compensation; no client artifacts written","elapsedMs":elapsed,"outcome":outcome,"reopened":restored,"staleAttemptRevisionAndDigestRefused":true,"externalConfigEditPreservedAndRefused":true,"foreignRuntimeRecordPreservedAndRefused":true,"duplicateConfirmationRefused":true,"clientConfigBytesUnchangedByFinalization":true});
    fs::write(
        root.join("explicit-finalize-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
