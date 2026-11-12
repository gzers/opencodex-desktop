use super::*;
use crate::{
    infrastructure::macos_client_originals::session_index::{self, preview},
    modules::runtime::RuntimeHandle,
    state::ProcessContext,
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Instant};
#[test]
#[ignore = "single ordinary Start/Stop on owned fixture after cancelled reserved capture; retains all partial evidence"]
fn ordinary_owned_lifecycle_after_cancelled_partial_capture() {
    let root = PathBuf::from(
        std::env::var_os("OCXD024_AFTER_CAPTURE_CANCEL_ROOT").expect("owned fixture"),
    );
    assert!(
        root.canonicalize().unwrap() == root
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
    let client = profile.join(".codex");
    let home = profile.join(".codex-desktop/opencodex/home");
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space")),
        working_directory: profile.clone(),
        opencodex_home: home,
    };
    let old = session_index::inspect(&context, &data).unwrap().unwrap();
    assert_eq!(old.state, session_index::State::CancelledCapture);
    let paths = [
        client.join("config.toml"),
        client.join("opencodex-catalog.json"),
        client.join("models_cache.json"),
        client.join("opencodex-catalog-audit.jsonl"),
        profile.join(".grok/config.toml"),
        profile.join(".grok/config.toml.bak-opencodex"),
        profile.join(".grok/agents/AGENTS.md"),
    ];
    let before: Vec<_> = paths.iter().map(|p| fs::read(p).ok()).collect();
    let runner = std::sync::Mutex::new(
        crate::infrastructure::process_runner::ControlledProcessRunner::new(&profile),
    );
    let log = crate::infrastructure::runtime_log::RuntimeLog::new(&data);
    let clock = Instant::now();
    let start = super::super::process_action_with_runner(
        crate::types::process_action::ProcessActionRequest {
            action: crate::modules::process::LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &context,
        &log,
        None,
    )
    .unwrap();
    let start_ms = clock.elapsed().as_millis();
    // A failed Start may have launched the owned service; use ordinary Stop
    // exactly once, preserving any error and never using force termination.
    let clock = Instant::now();
    let stop = super::super::process_action_with_runner(
        crate::types::process_action::ProcessActionRequest {
            action: crate::modules::process::LifecycleAction::Stop,
            confirm: true,
        },
        &runner,
        &context,
        &log,
        None,
    )
    .unwrap();
    let stop_ms = clock.elapsed().as_millis();
    let head = session_index::inspect(&context, &data).unwrap().unwrap();
    let all = paths
        .iter()
        .zip(&before)
        .all(|(p, b)| fs::read(p).ok() == *b);
    let passed = start.result == Some(crate::modules::process::LifecycleResult::Started)
        && stop.result == Some(crate::modules::process::LifecycleResult::Stopped)
        && start_ms < 30000
        && stop_ms < 10000
        && all
        && head.state == session_index::State::Restored;
    let report = serde_json::json!({"accepted":false,"result":if passed{"passed"}else{"failed"},"start":start,"startElapsedMs":start_ms,"stop":stop,"stopElapsedMs":stop_ms,"headState":head.state,"headRevision":head.revision,"oldCancelledTicket":old.ticket,"newAttempt":head.references.as_ref().map(|r|&r.attempt),"all7OriginalPathsMatch":all,"noBusinessRequests":true});
    fs::write(
        root.join("after-cancel-lifecycle.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    assert!(passed);
}
#[test]
#[ignore = "directed next Start interrupted before capture on same owned recovered native fixture"]
fn prepare_next_owned_start_after_registered_closure() {
    let root =
        PathBuf::from(std::env::var_os("OCXD024_HISTORY_PREPARE_ROOT").expect("owned fixture"));
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
    assert_eq!(
        std::env::var_os("OCXD024_INTERRUPT_BEFORE_CAPTURE").as_deref(),
        Some(std::ffi::OsStr::new("1"))
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
    let previous = session_index::inspect(&context, &data).unwrap().unwrap();
    assert!(matches!(
        previous.state,
        session_index::State::Restored | session_index::State::Unchanged
    ));
    let runner = std::sync::Mutex::new(
        crate::infrastructure::process_runner::ControlledProcessRunner::new(&profile),
    );
    let started = Instant::now();
    let result = super::super::process_action_with_runner(
        crate::types::process_action::ProcessActionRequest {
            action: crate::modules::process::LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &context,
        &crate::infrastructure::runtime_log::RuntimeLog::new(&data),
        None,
    )
    .unwrap();
    let elapsed = started.elapsed().as_millis();
    assert!(
        result.result.is_none()
            && result.client_recovery_required
            && !result.can_start
            && !result.can_stop
            && !result.can_restart
    );
    let head = session_index::inspect(&context, &data).unwrap().unwrap();
    assert_eq!(head.state, session_index::State::Preparing);
    assert_eq!(head.revision, previous.revision + 1);
    assert!(head.references.is_none());
    assert_ne!(head.ticket, previous.ticket);
    assert!(elapsed < 30000);
    let preview = preview::inspect_preview(&context, &data);
    assert!(preview.can_cancel_preparation);
    fs::write(root.join("next-historical-preparing.json"),serde_json::to_vec_pretty(&serde_json::json!({"accepted":false,"elapsedMs":elapsed,"result":result,"indexState":head.state,"indexRevision":head.revision,"previousRegisteredState":previous.state,"previousRevision":previous.revision,"canCancelFromIndexedHistory":preview.can_cancel_preparation,"noNewServiceStarted":true})).unwrap()).unwrap();
}
#[test]
#[ignore = "owned pinned official fixture retained after Preparing and before capture; cancel metadata only"]
fn cancel_owned_preparing_without_vault_preserves_current_clients() {
    let root =
        PathBuf::from(std::env::var_os("OCXD024_PREPARE_CANCEL_ROOT").expect("owned fixture"));
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
    let client = profile.join(".codex");
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &root.join("program 中文 space")),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let request = session_index::preparation::review(&context, &data)
        .unwrap()
        .expect("exact unstarted reservation");
    let idx = data.join("manager-state/client-session-index.sqlite");
    let before = fs::read(&idx).unwrap();
    let config = client.join("config.toml");
    let bytes = fs::read(&config).unwrap();
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
        assert!(session_index::preparation::cancel(&context, &data, wrong).is_err());
        assert_eq!(fs::read(&idx).unwrap(), before);
    }
    let vault = data.join("manager-state/client-originals");
    let historical = std::env::var_os("OCXD024_HISTORY_CANCEL").is_some();
    assert_eq!(vault.exists(), historical);
    let injection = if historical {
        vault.join(uuid::Uuid::new_v4().to_string())
    } else {
        vault.clone()
    };
    fs::create_dir(&injection).unwrap();
    fs::write(
        injection.join("partial-original"),
        b"unknown retained evidence",
    )
    .unwrap();
    assert!(session_index::preparation::review(&context, &data).is_err());
    assert!(session_index::preparation::cancel(&context, &data, request.clone()).is_err());
    assert_eq!(
        fs::read(injection.join("partial-original")).unwrap(),
        b"unknown retained evidence"
    );
    assert_eq!(fs::read(&idx).unwrap(), before);
    fs::remove_file(injection.join("partial-original")).unwrap();
    fs::remove_dir(injection).unwrap();
    let record = home.join("runtime-port.json");
    assert!(!record.exists());
    fs::write(
        &record,
        serde_json::to_vec(
            &serde_json::json!({"pid":std::process::id(),"port":1,"hostname":"127.0.0.1"}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&record, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(session_index::preparation::cancel(&context, &data, request.clone()).is_err());
    assert!(record.exists());
    assert_eq!(fs::read(&idx).unwrap(), before);
    fs::remove_file(record).unwrap();
    let start = Instant::now();
    let result = session_index::preparation::cancel(&context, &data, request.clone());
    let elapsed = start.elapsed().as_millis();
    let report = serde_json::json!({"accepted":false,"elapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),"reopened":preview::inspect_preview(&context,&data),"noVaultCreated":!data.join("manager-state/client-originals").exists(),"noClientFileReplay":true});
    fs::write(
        root.join("prepare-cancel-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    let outcome = result.unwrap();
    assert!(elapsed < 10000);
    assert_eq!(
        outcome.status,
        if std::env::var_os("OCXD024_RESERVED_CANCEL").is_some() {
            "cancelled_capture"
        } else {
            "cancelled_before_capture"
        }
    );
    assert_eq!(outcome.revision, request.revision + 1);
    assert_eq!(fs::read(config).unwrap(), bytes);
    let after = fs::read(&idx).unwrap();
    assert!(session_index::preparation::cancel(&context, &data, request).is_err());
    assert_eq!(fs::read(&idx).unwrap(), after);
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
        crate::modules::status::ClientRecoveryState::NotRequired
    );
    assert!(snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart);
    assert!(snapshot.pid.is_none() && snapshot.port.is_none());
    fs::write(
        root.join("prepare-cancel-live-status.json"),
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();
}
#[test]
#[ignore = "owned official runtime retained after independent Captured interruption, metadata-only early closure"]
fn finish_owned_captured_session_only_when_original_clients_are_unchanged() {
    let root = PathBuf::from(std::env::var_os("OCXD024_UNCHANGED_ROOT").expect("owned fixture"));
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
    let request = session_index::unchanged::review(&context, &data)
        .unwrap()
        .expect("exact original tree captured");
    let index = data.join("manager-state/client-session-index.sqlite");
    let index_before = fs::read(&index).unwrap();
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
        assert!(session_index::unchanged::finish(&context, &data, wrong).is_err());
        assert_eq!(fs::read(&index).unwrap(), index_before);
    }
    let config = profile.join(".codex/config.toml");
    let bytes = fs::read(&config).unwrap();
    fs::write(&config, b"outside edit during early recovery").unwrap();
    assert!(session_index::unchanged::review(&context, &data)
        .unwrap()
        .is_none());
    assert!(session_index::unchanged::finish(&context, &data, request.clone()).is_err());
    assert_eq!(
        fs::read(&config).unwrap(),
        b"outside edit during early recovery"
    );
    assert_eq!(fs::read(&index).unwrap(), index_before);
    fs::write(&config, &bytes).unwrap();
    let request = session_index::unchanged::review(&context, &data)
        .unwrap()
        .unwrap();
    let runtime_record = home.join("runtime-port.json");
    assert!(!runtime_record.exists());
    fs::write(
        &runtime_record,
        serde_json::to_vec(
            &serde_json::json!({"pid":std::process::id(),"port":1,"hostname":"127.0.0.1"}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&runtime_record, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(session_index::unchanged::finish(&context, &data, request).is_err());
    assert_eq!(fs::read(&index).unwrap(), index_before);
    fs::remove_file(runtime_record).unwrap();
    let request = session_index::unchanged::review(&context, &data)
        .unwrap()
        .unwrap();
    let start = Instant::now();
    let result = session_index::unchanged::finish(&context, &data, request.clone());
    let elapsed = start.elapsed().as_millis();
    let report = serde_json::json!({"accepted":false,"elapsedMs":elapsed,"outcome":result.as_ref().ok(),"error":result.as_ref().err().map(|e|e.to_string()),"reopened":preview::inspect_preview(&context,&data),"noClientFileReplay":true,"noHistoricalExitProofReconstructed":true});
    fs::write(
        root.join("early-unchanged-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    let outcome = result.unwrap();
    assert_eq!(outcome.status, "unchanged");
    assert_eq!(outcome.revision, request.revision + 1);
    assert!(elapsed < 10000);
    assert_eq!(fs::read(config).unwrap(), bytes);
    let after = fs::read(&index).unwrap();
    assert!(session_index::unchanged::finish(&context, &data, request).is_err());
    assert_eq!(fs::read(&index).unwrap(), after);
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
        crate::modules::status::ClientRecoveryState::NotRequired
    );
    assert!(snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart);
    assert!(snapshot.pid.is_none() && snapshot.port.is_none());
    fs::write(
        root.join("early-final-live-status.json"),
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();
}
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
    assert!(before.can_continue && !before.can_finalize && before.can_rollback);
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
    assert_eq!(outcome.revision, request.revision + 1);
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
