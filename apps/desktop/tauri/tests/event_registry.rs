use chrono::{DateTime, Utc};
use opencodex_desktop_lib::{
    commands::notifications::NotificationPublisher,
    modules::notifications::{
        persistence::{load_notifications, notifications_path},
        registry::*,
        Notification, NotificationCategory, NotificationLevel, NotificationSource,
        NotificationStore,
    },
    types::{notifications::NotificationDto, status::RuntimeState},
};
use std::sync::{Arc, Mutex};

fn opaque(n: u128) -> OpaqueId {
    OpaqueId::parse(&uuid::Uuid::from_u128(n).to_string()).unwrap()
}
fn at(n: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(n, 0).unwrap()
}
fn failure() -> Delivery<'static> {
    Delivery {
        event: "run-start-failed",
        job: Job::Start,
        trigger: Trigger::User,
        identity: EventIdentity {
            object: ObjectKind::Runtime,
            object_id: opaque(1),
            action: Action::Start,
            phase: Phase::Execution,
            channel: Channel::Local,
            candidate: opaque(2),
        },
        evidence: Evidence::Failure,
        occurred_at: at(100),
    }
}
fn success() -> Delivery<'static> {
    let mut d = failure();
    d.event = "run-start-succeeded";
    d.evidence = Evidence::Success {
        candidate: d.identity.candidate.clone(),
        verified: true,
    };
    d.occurred_at = at(110);
    d
}
fn prefs() -> NotificationPreferences {
    NotificationPreferences { lifecycle: true }
}
fn legacy(id: &str) -> Notification {
    Notification::new(
        id,
        NotificationLevel::Danger,
        NotificationCategory::Run,
        NotificationSource::Runtime,
        "title",
        "body",
        at(100).to_rfc3339(),
        None,
    )
    .unwrap()
}
fn setup() -> (tempfile::TempDir, Arc<Mutex<NotificationStore>>) {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
    (root, Arc::new(Mutex::new(NotificationStore::new())))
}

#[test]
fn registry_is_closed_and_references_are_consistent() {
    assert_eq!(
        lookup("unknown /secret/path token=secret").unwrap_err(),
        RegistryError::UnknownEvent
    );
    let config = registry().unwrap();
    for event in &config.events {
        assert_eq!(config.events.iter().filter(|e| e.id == event.id).count(), 1);
        let job = config.jobs.iter().find(|j| j.id == event.job).unwrap();
        assert_eq!(job.availability, event.availability);
        assert!(job.triggers.iter().all(|t| config.triggers.contains(t)));
        if let Some(id) = event.policy {
            assert_eq!(
                config
                    .notification_policies
                    .iter()
                    .filter(|p| p.id == id)
                    .count(),
                1
            );
        }
        for failure in &event.resolves {
            let failed = lookup(failure).unwrap();
            assert_eq!(failed.fact, Fact::Failure);
            assert_eq!(failed.job, event.job);
            assert_eq!(failed.object, event.object);
            assert_eq!(failed.action, event.action);
            assert_eq!(failed.phase, event.phase);
            assert_eq!(failed.channels, event.channels);
        }
        if event.availability == Availability::Planned {
            assert_eq!(lookup(&event.id).unwrap_err(), RegistryError::Planned);
            assert!(event.policy.is_none());
        }
    }
    for site in &config.emission_sites {
        assert!(config.events.iter().any(|e| e.id == site.event));
    }
}

#[test]
fn legacy_gate_discards_all_raw_content_and_refuses_unknown_and_planned_without_writes() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    for id in [
        "unknown",
        "backup-completed",
        "migration-completed",
        "status-snapshot-changed",
    ] {
        assert!(publisher.publish(legacy(id)).is_err());
        assert!(store.lock().unwrap().all().is_empty());
        assert!(!notifications_path(root.path()).exists());
    }
    let mut input = legacy("run-start-failed");
    input.title = "/Users/private/home".into();
    input.body = "raw error: password=secret token=credential".into();
    input.operation_id = Some("/private/key".into());
    input.dedupe_key = Some("Bearer raw".into());
    input.read = true;
    input.resolved = true;
    input.deleted = true;
    input.event_identity = Some(failure().identity);
    publisher.publish(input).unwrap();
    let loaded = load_notifications(&notifications_path(root.path())).unwrap();
    let saved = &loaded.all()[0];
    assert!(saved.operation_id.is_none());
    assert!(saved.event_identity.is_none());
    assert!(!saved.read && !saved.resolved && !saved.deleted);
    let text = std::fs::read_to_string(notifications_path(root.path())).unwrap();
    for raw in [
        "/Users/",
        "/private/",
        "password=",
        "token=",
        "Bearer raw",
        "raw error",
    ] {
        assert!(!text.contains(raw));
    }
}

#[test]
fn invalid_job_trigger_scope_and_fact_never_mutate() {
    let mut store = NotificationStore::new();
    let mut wrong = failure();
    wrong.job = Job::Restart;
    assert_eq!(
        deliver(&mut store, &wrong, prefs()),
        Err(RegistryError::InvalidJob)
    );
    wrong = failure();
    wrong.trigger = Trigger::Deadline;
    assert_eq!(
        deliver(&mut store, &wrong, prefs()),
        Err(RegistryError::InvalidJob)
    );
    wrong = failure();
    wrong.identity.phase = Phase::Query;
    assert_eq!(
        deliver(&mut store, &wrong, prefs()),
        Err(RegistryError::InvalidIdentity)
    );
    wrong = failure();
    wrong.evidence = Evidence::Signal;
    assert_eq!(
        deliver(&mut store, &wrong, prefs()),
        Err(RegistryError::InvalidFact)
    );
    assert!(store.all().is_empty());
    for raw in [
        "/Users/private",
        "token=secret",
        "00000000-0000-0000-0000-000000000000",
    ] {
        assert!(OpaqueId::parse(raw).is_err());
    }
    assert!(serde_json::from_str::<OpaqueId>("\"/private/path\"").is_err());
}

#[test]
fn exact_verified_candidate_success_resolves_after_restart_without_marking_read() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    assert!(publisher.publish_event(&failure()).unwrap().added);
    assert!(!publisher.publish_event(&failure()).unwrap().added);
    let loaded = load_notifications(&notifications_path(root.path())).unwrap();
    let restarted = Arc::new(Mutex::new(loaded));
    let publisher = NotificationPublisher {
        store: &restarted,
        data_root: root.path(),
    };
    let result = publisher.publish_event(&success()).unwrap();
    assert_eq!(
        result,
        DeliveryOutcome {
            changed: true,
            added: false,
            resolved: 1
        }
    );
    let saved = load_notifications(&notifications_path(root.path())).unwrap();
    assert!(saved.all()[0].resolved);
    assert!(!saved.all()[0].read);
    assert_eq!(
        saved.all()[0].resolved_at.as_deref(),
        Some(at(110).to_rfc3339().as_str())
    );
    assert_eq!(publisher.publish_event(&success()).unwrap().resolved, 0);
}

#[test]
fn recovery_requires_object_action_phase_channel_and_candidate_and_cannot_clear_other_risks() {
    let mut base = NotificationStore::new();
    deliver(&mut base, &failure(), prefs()).unwrap();
    // Legacy has no scope; observation risk has another action/phase, never inferred from a start success.
    base.push(canonical_legacy(&legacy("run-start-failed")).unwrap());
    let mut risk = failure();
    risk.event = "run-at-risk";
    risk.job = Job::Observe;
    risk.trigger = Trigger::StatusChange;
    risk.identity.action = Action::Observe;
    risk.identity.phase = Phase::Observation;
    risk.evidence = Evidence::Risk;
    deliver(&mut base, &risk, prefs()).unwrap();
    for dimension in 0..7 {
        let mut d = success();
        match dimension {
            0 => d.identity.object = ObjectKind::Manager,
            1 => d.identity.object_id = opaque(3),
            2 => {
                d.event = "run-stop-succeeded";
                d.job = Job::Stop;
                d.identity.action = Action::Stop;
            }
            3 => d.identity.phase = Phase::Observation,
            4 => d.identity.channel = Channel::Beta,
            5 => {
                d.identity.candidate = opaque(3);
                d.evidence = Evidence::Success {
                    candidate: opaque(3),
                    verified: true,
                };
            }
            _ => d.occurred_at = at(99),
        }
        let mut store = base.clone();
        assert!(!deliver(&mut store, &d, prefs()).is_ok_and(|o| o.resolved > 0));
        assert_eq!(store, base);
    }
    let result = deliver(&mut base, &success(), prefs()).unwrap();
    assert_eq!(result.resolved, 1);
    assert_eq!(base.aggregate().unresolved, 2);
}

#[test]
fn pending_unverified_or_wrong_candidate_success_is_rejected_and_deleted_history_is_not_revived() {
    let mut store = NotificationStore::new();
    deliver(&mut store, &failure(), prefs()).unwrap();
    let before = store.clone();
    for (candidate, verified) in [(opaque(2), false), (opaque(3), true)] {
        let mut d = success();
        d.evidence = Evidence::Success {
            candidate,
            verified,
        };
        assert_eq!(
            deliver(&mut store, &d, prefs()),
            Err(RegistryError::UnverifiedSuccess)
        );
        assert_eq!(store, before);
    }
    let id = store.all()[0].notification_id.clone();
    store.delete(&id);
    assert_eq!(
        deliver(&mut store, &success(), prefs()).unwrap().resolved,
        0
    );
    assert!(store.all()[0].deleted);
    assert!(!store.all()[0].resolved);
}

#[test]
fn cooldown_preserves_read_timestamp_and_reopens_only_after_recovery_or_deadline() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    assert_eq!(
        publisher.publish_event(&failure()).unwrap(),
        DeliveryOutcome {
            changed: true,
            added: true,
            resolved: 0
        }
    );
    assert_eq!(
        publisher.publish_event(&failure()).unwrap(),
        DeliveryOutcome::default()
    );
    let mut later = failure();
    later.occurred_at = at(105);
    let id = store.lock().unwrap().all()[0].notification_id.clone();
    opencodex_desktop_lib::commands::notifications::mark_notification_read_with_store(
        &id,
        &store,
        root.path(),
    )
    .unwrap();
    assert_eq!(
        publisher.publish_event(&later).unwrap(),
        DeliveryOutcome {
            changed: true,
            added: false,
            resolved: 0
        }
    );
    let persisted = load_notifications(&notifications_path(root.path())).unwrap();
    assert_eq!(persisted, *store.lock().unwrap());
    assert_eq!(persisted.all()[0].created_at, at(100).to_rfc3339());
    assert!(persisted.all()[0].read);
    assert_eq!(persisted.all()[0].occurrence_count, 2);
    assert_eq!(
        persisted.all()[0].last_observed_at,
        Some(at(105).to_rfc3339())
    );
    publisher.publish_event(&success()).unwrap();
    later.occurred_at = at(120);
    assert_eq!(
        publisher.publish_event(&later).unwrap(),
        DeliveryOutcome {
            changed: true,
            added: false,
            resolved: 0
        }
    );
    let persisted = load_notifications(&notifications_path(root.path())).unwrap();
    assert_eq!(persisted, *store.lock().unwrap());
    assert!(!persisted.all()[0].read);
    assert!(!persisted.all()[0].resolved);
    assert_eq!(persisted.all().len(), 1);
    assert_eq!(persisted.all()[0].occurrence_count, 1);
    // The first observed failure after the cooldown refreshes the existing entry.
    later.occurred_at = at(120 + 86_400);
    assert!(publisher.publish_event(&later).unwrap().changed);
    assert_eq!(
        store.lock().unwrap().all()[0].created_at,
        later.occurred_at.to_rfc3339()
    );
    assert_eq!(store.lock().unwrap().all()[0].occurrence_count, 2);
    // Historical duplicates of an unresolved candidate do not create a new reminder.
    let persisted = load_notifications(&notifications_path(root.path())).unwrap();
    let current = persisted.all()[0].clone();
    let mut older = current.clone();
    older.created_at = at(115 + 86_400).to_rfc3339();
    let mut duplicates = NotificationStore::with_items(vec![older, current]);
    let before = duplicates.clone();
    let outcome = deliver(&mut duplicates, &later, prefs()).unwrap();
    assert!(!outcome.changed);
    assert!(!outcome.added);
    assert_eq!(before, duplicates);
}

#[test]
fn same_scope_dedupes_but_different_objects_and_candidates_coexist() {
    let mut store = NotificationStore::new();
    let d = failure();
    deliver(&mut store, &d, prefs()).unwrap();
    let mut other = failure();
    other.identity.object_id = opaque(3);
    deliver(&mut store, &other, prefs()).unwrap();
    other = failure();
    other.identity.candidate = opaque(4);
    deliver(&mut store, &other, prefs()).unwrap();
    assert_eq!(store.live().len(), 3);
    assert_eq!(
        deliver(&mut store, &success(), prefs()).unwrap().resolved,
        1
    );
    assert_eq!(store.aggregate().unresolved, 2);
}

#[test]
fn lifecycle_preference_is_independent_of_environment_risks_and_recovery() {
    let mut store = NotificationStore::new();
    let disabled = NotificationPreferences { lifecycle: false };
    assert!(!deliver(&mut store, &failure(), disabled).unwrap().added);
    assert!(store.all().is_empty());
    let mut risk = failure();
    risk.event = "external-takeover";
    risk.job = Job::Observe;
    risk.trigger = Trigger::StatusChange;
    risk.identity.action = Action::Observe;
    risk.identity.phase = Phase::Observation;
    risk.evidence = Evidence::Risk;
    deliver(&mut store, &risk, disabled).unwrap();
    assert_eq!(
        NotificationDto::from_domain(&store.all()[0]).target,
        Some(RuntimeState::ExternalTakeover)
    );
    deliver(&mut store, &failure(), prefs()).unwrap();
    assert_eq!(
        deliver(&mut store, &success(), disabled).unwrap().resolved,
        1
    );
    assert!(!store.all()[0].resolved);
}

#[test]
fn signal_delivery_and_suppressed_notifications_do_not_create_history_files() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    let mut signal = failure();
    signal.event = "status-snapshot-changed";
    signal.job = Job::Observe;
    signal.trigger = Trigger::Deadline;
    signal.identity.action = Action::Observe;
    signal.identity.phase = Phase::Observation;
    signal.evidence = Evidence::Signal;
    assert_eq!(
        publisher.publish_event(&signal).unwrap(),
        DeliveryOutcome::default()
    );
    assert!(!notifications_path(root.path()).exists());
}

#[test]
fn persistence_failure_rolls_back_new_notification_and_recovery() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    let path = notifications_path(root.path());
    std::fs::create_dir(&path).unwrap();
    assert!(publisher.publish_event(&failure()).is_err());
    assert!(store.lock().unwrap().all().is_empty());
    std::fs::remove_dir(&path).unwrap();
    publisher.publish_event(&failure()).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    let before = store.lock().unwrap().clone();
    assert!(publisher.publish_event(&success()).is_err());
    assert_eq!(*store.lock().unwrap(), before);
}

#[test]
fn old_history_without_identity_still_loads_and_is_not_automatically_resolved() {
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    publisher.publish(legacy("run-start-failed")).unwrap();
    let text = std::fs::read_to_string(notifications_path(root.path())).unwrap();
    assert!(!text.contains("event_identity"));
    let mut loaded = load_notifications(&notifications_path(root.path())).unwrap();
    assert_eq!(
        deliver(&mut loaded, &success(), prefs()).unwrap().resolved,
        0
    );
    assert_eq!(loaded.aggregate().unresolved, 1);
}

#[test]
fn embedded_config_rejects_unknown_duplicate_reference_paths_and_planned_activation() {
    fn rejected(mutator: impl FnOnce(&mut serde_json::Value)) {
        let mut value: serde_json::Value = serde_json::from_str(EMBEDDED_CONFIG).unwrap();
        mutator(&mut value);
        assert_eq!(
            parse_config(&value.to_string()).unwrap_err(),
            RegistryError::InvalidConfig
        );
    }
    rejected(|v| v["events"][0]["id"] = "unknown /private/key token=raw".into());
    rejected(|v| v["jobs"][0]["id"] = "unknown".into());
    rejected(|v| {
        let copy = v["events"][0].clone();
        v["events"].as_array_mut().unwrap().push(copy);
    });
    rejected(|v| {
        let copy = v["jobs"][0].clone();
        v["jobs"].as_array_mut().unwrap().push(copy);
    });
    rejected(|v| {
        let copy = v["notification_policies"][0].clone();
        v["notification_policies"]
            .as_array_mut()
            .unwrap()
            .push(copy);
    });
    rejected(|v| v["events"][4]["job"] = "restart".into());
    rejected(|v| v["events"][4]["resolves"] = serde_json::json!(["manager-install-failed"]));
    rejected(|v| v["emission_sites"][0]["event"] = "unknown".into());
    rejected(|v| v["paths"][0]["relative_path"] = "/private/key".into());
    rejected(|v| v["paths"][0]["relative_path"] = "../private/key".into());
    rejected(|v| v["cleanup_policies"][3]["mode"] = "age_and_line_limit".into());
    rejected(|v| {
        for job in v["jobs"].as_array_mut().unwrap() {
            if job["id"] == "sync" {
                job["availability"] = "active".into();
            }
        }
        for event in v["events"].as_array_mut().unwrap() {
            if event["id"] == "sync-conflict-detected" {
                event["availability"] = "active".into();
            }
        }
    });
    let mut value: serde_json::Value = serde_json::from_str(EMBEDDED_CONFIG).unwrap();
    value["notification_policies"][0]["title"] = "配置定义的固定标题".into();
    assert_eq!(
        parse_config(&value.to_string())
            .unwrap()
            .notification_policies[0]
            .title,
        "配置定义的固定标题"
    );
}

#[test]
fn persistent_scope_reuses_root_candidate_and_rotates_on_real_context_changes() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let executable = root.path().join("runtime-file");
    std::fs::write(&executable, b"candidate A").unwrap();
    let scope = || {
        lifecycle_identity(
            root.path(),
            &executable,
            root.path(),
            &root.path().join("controlled-home"),
            Action::Start,
        )
        .unwrap()
    };
    let first = scope();
    assert_eq!(first, scope());
    assert_eq!(first.object_id, data_root_object_id(root.path()).unwrap());
    assert_ne!(first.object_id, data_root_object_id(other.path()).unwrap());
    let mut store = NotificationStore::new();
    let mut failed = failure();
    failed.identity = first.clone();
    deliver(&mut store, &failed, prefs()).unwrap();
    let mut ok = success();
    ok.identity = scope();
    ok.evidence = Evidence::Success {
        candidate: ok.identity.candidate.clone(),
        verified: true,
    };
    assert_eq!(deliver(&mut store, &ok, prefs()).unwrap().resolved, 1);
    std::fs::write(&executable, b"candidate B").unwrap();
    let changed = scope();
    assert_eq!(first.object_id, changed.object_id);
    assert_ne!(first.candidate, changed.candidate);
    let other_home = lifecycle_identity(
        root.path(),
        &executable,
        root.path(),
        &root.path().join("other-home"),
        Action::Start,
    )
    .unwrap();
    assert_ne!(changed.candidate, other_home.candidate);
    std::fs::write(&executable, b"candidate A").unwrap();
    assert_ne!(first.candidate, scope().candidate);
    let disk = std::fs::read_to_string(
        root.path()
            .join(registry().unwrap().path(PathId::EventScope)),
    )
    .unwrap();
    assert!(!disk.contains(root.path().to_str().unwrap()));
    assert!(!disk.contains("candidate A") && !disk.contains("controlled-home"));
    std::fs::write(
        root.path()
            .join(registry().unwrap().path(PathId::EventScope)),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(
        data_root_object_id(root.path()),
        Err(RegistryError::ScopeUnavailable)
    );
}

#[test]
fn concurrent_scope_creation_keeps_one_identity() {
    let root = tempfile::tempdir().unwrap();
    let ids = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| data_root_object_id(root.path()).unwrap()))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(ids.iter().all(|id| id == &ids[0]));
}

#[cfg(unix)]
#[test]
fn scope_rejects_symlinks_without_writing_external_files() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("manager-state")).unwrap();
    assert_eq!(
        data_root_object_id(root.path()),
        Err(RegistryError::ScopeUnavailable)
    );
    assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 0);
}

#[test]
fn manager_check_and_pending_restart_never_resolve_install_failure() {
    let mut failed = failure();
    failed.event = "manager-install-failed";
    failed.job = Job::ManagerInstall;
    failed.identity.object = ObjectKind::Manager;
    failed.identity.action = Action::Install;
    failed.identity.channel = Channel::Stable;
    let mut store = NotificationStore::new();
    deliver(&mut store, &failed, prefs()).unwrap();
    for event in ["manager-check-succeeded", "manager-install-pending-restart"] {
        let registration = lookup(event).unwrap();
        let mut d = failure();
        d.event = event;
        d.job = registration.job;
        d.identity = failed.identity.clone();
        d.identity.action = registration.action;
        d.identity.phase = registration.phase;
        d.evidence = if registration.fact == Fact::Signal {
            Evidence::Signal
        } else {
            Evidence::Success {
                candidate: d.identity.candidate.clone(),
                verified: true,
            }
        };
        assert_eq!(deliver(&mut store, &d, prefs()).unwrap().resolved, 0);
    }
    assert!(!store.all()[0].resolved);
}

#[test]
fn lifecycle_command_only_resolves_matching_real_result_with_persisted_scope() {
    use opencodex_desktop_lib::{
        commands::process_action_with_runner,
        infrastructure::{runtime_executable::FixedRuntimeExecutable, runtime_log::RuntimeLog},
        modules::{
            preferences::{Preferences, PreferencesStore},
            process::{LifecycleAction, LifecycleResult, ProcessCommand, ProcessRunner},
        },
        state::ProcessContext,
        types::process_action::ProcessActionRequest,
    };
    struct Runner {
        result: LifecycleResult,
        replace: bool,
    }
    impl ProcessRunner for Runner {
        fn execute(
            &self,
            command: &ProcessCommand,
        ) -> opencodex_desktop_lib::errors::AppResult<LifecycleResult> {
            if self.replace {
                std::fs::write(&command.executable, b"changed during execution").unwrap();
            }
            Ok(self.result.clone())
        }
        fn cancel_pending(&self) -> opencodex_desktop_lib::errors::AppResult<()> {
            Ok(())
        }
    }
    for action in [
        LifecycleAction::Start,
        LifecycleAction::Stop,
        LifecycleAction::Restart,
    ] {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("candidate");
        std::fs::write(&executable, b"fixed candidate").unwrap();
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory: root.path().to_path_buf(),
            opencodex_home: root.path().join("home"),
        };
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let run = |result, replace, shared: &Arc<Mutex<NotificationStore>>| {
            process_action_with_runner(
                ProcessActionRequest {
                    action,
                    confirm: true,
                },
                &Mutex::new(Runner { result, replace }),
                &context,
                &RuntimeLog::new(root.path()),
                Some(NotificationPublisher {
                    store: shared,
                    data_root: root.path(),
                }),
            )
            .unwrap();
        };
        run(LifecycleResult::Failed, false, &store);
        assert_eq!(store.lock().unwrap().all().len(), 1);
        let restarted = Arc::new(Mutex::new(
            load_notifications(&notifications_path(root.path())).unwrap(),
        ));
        run(LifecycleResult::Cancelled, false, &restarted);
        assert!(!restarted.lock().unwrap().all()[0].resolved);
        let wrong = if action == LifecycleAction::Stop {
            LifecycleResult::Started
        } else {
            LifecycleResult::Stopped
        };
        run(wrong, false, &restarted);
        assert!(!restarted.lock().unwrap().all()[0].resolved);
        // Turning notifications off suppresses new failures, while success can resolve old ones.
        let mut preferences = Preferences {
            lifecycle_notifications: false,
            ..Default::default()
        };
        PreferencesStore::new(root.path())
            .save(&preferences)
            .unwrap();
        let right = if action == LifecycleAction::Stop {
            LifecycleResult::Stopped
        } else {
            LifecycleResult::Started
        };
        run(right.clone(), true, &restarted);
        assert!(!restarted.lock().unwrap().all()[0].resolved);
        std::fs::write(&executable, b"fixed candidate").unwrap();
        run(right.clone(), false, &restarted);
        assert!(
            !restarted.lock().unwrap().all()[0].resolved,
            "rotated candidates cannot resurrect old recovery scopes"
        );
        preferences.lifecycle_notifications = true;
        PreferencesStore::new(root.path())
            .save(&preferences)
            .unwrap();
        run(LifecycleResult::Failed, false, &restarted);
        assert_eq!(restarted.lock().unwrap().all().len(), 2);
        preferences.lifecycle_notifications = false;
        PreferencesStore::new(root.path())
            .save(&preferences)
            .unwrap();
        run(right, false, &restarted);
        let guard = restarted.lock().unwrap();
        assert!(!guard.all()[0].resolved);
        assert!(guard.all()[1].resolved && !guard.all()[1].read);
    }
}

fn operation_identity(root: &std::path::Path, event: &str, fingerprint: u8) -> EventIdentity {
    let e = lookup(event).unwrap();
    candidate_identity(
        root,
        e.object,
        e.action,
        e.phase,
        e.channels[0],
        [fingerprint; 32],
    )
    .unwrap()
}

#[test]
fn new_terminal_pairs_persist_and_only_exact_verified_success_recovers() {
    for stem in [
        "manager-install",
        "preferences-backup",
        "preferences-restore",
        "preferences-save",
        "preferences-reset",
        "preferences-backup-policy",
        "preferences-backup-cleanup",
        "local-cleanup",
    ] {
        let (root, store) = setup();
        let failed_id = format!("{stem}-failed");
        let succeeded_id = format!("{stem}-succeeded");
        let identity = operation_identity(root.path(), &failed_id, 1);
        let failure = terminal_delivery(
            &failed_id,
            Trigger::User,
            identity.clone(),
            Evidence::Failure,
            at(100),
        )
        .unwrap();
        let publisher = NotificationPublisher {
            store: &store,
            data_root: root.path(),
        };
        assert!(publisher.publish_event(&failure).unwrap().added);
        let mut loaded = load_notifications(&notifications_path(root.path())).unwrap();
        // Retrying, check/query and a different candidate cannot clear this record.
        let success = |identity: EventIdentity, time| {
            let candidate = identity.candidate.clone();
            terminal_delivery(
                &succeeded_id,
                Trigger::User,
                identity,
                Evidence::Success {
                    candidate,
                    verified: true,
                },
                at(time),
            )
            .unwrap()
        };
        let mut wrong = identity.clone();
        wrong.candidate = opaque(999);
        assert_eq!(
            deliver(&mut loaded, &success(wrong, 110), prefs())
                .unwrap()
                .resolved,
            0
        );
        let mut other_root = identity.clone();
        other_root.object_id = opaque(998);
        assert_eq!(
            deliver(&mut loaded, &success(other_root, 110), prefs())
                .unwrap()
                .resolved,
            0
        );
        if stem == "manager-install" {
            let mut other_channel = identity.clone();
            other_channel.channel = Channel::Beta;
            assert_eq!(
                deliver(&mut loaded, &success(other_channel, 110), prefs())
                    .unwrap()
                    .resolved,
                0
            );
        }
        assert_eq!(
            deliver(&mut loaded, &success(identity.clone(), 99), prefs())
                .unwrap()
                .resolved,
            0
        );
        assert!(matches!(
            terminal_delivery(
                &succeeded_id,
                Trigger::User,
                identity.clone(),
                Evidence::Success {
                    candidate: identity.candidate.clone(),
                    verified: false
                },
                at(110)
            ),
            Err(RegistryError::UnverifiedSuccess)
        ));
        let mut items = loaded.all().to_vec();
        items[0].deleted = true;
        let mut deleted = NotificationStore::with_items(items);
        assert_eq!(
            deliver(&mut deleted, &success(identity.clone(), 110), prefs())
                .unwrap()
                .resolved,
            0
        );
        let restarted = Arc::new(Mutex::new(loaded));
        let publisher = NotificationPublisher {
            store: &restarted,
            data_root: root.path(),
        };
        assert_eq!(
            publisher
                .publish_event(&success(identity.clone(), 110))
                .unwrap()
                .resolved,
            1
        );
        let saved = load_notifications(&notifications_path(root.path())).unwrap();
        assert!(saved.all()[0].resolved && !saved.all()[0].read);
        // New policies remain enabled even when lifecycle notifications are off.
        let mut quiet = NotificationStore::new();
        assert!(
            deliver(
                &mut quiet,
                &failure,
                NotificationPreferences { lifecycle: false }
            )
            .unwrap()
            .added
        );
        assert_eq!(
            deliver(
                &mut quiet,
                &success(identity, 110),
                NotificationPreferences { lifecycle: false }
            )
            .unwrap()
            .resolved,
            1
        );
    }
}

#[test]
fn terminal_helper_rejects_queries_progress_planned_and_mismatched_evidence() {
    let root = tempfile::tempdir().unwrap();
    for event in [
        "manager-check-succeeded",
        "manager-install-progress",
        "manager-install-pending-restart",
        "preferences-backup-list",
        "preferences-backup-pin",
        "preferences-restored",
    ] {
        let identity = operation_identity(root.path(), event, 1);
        assert!(matches!(
            terminal_delivery(event, Trigger::User, identity, Evidence::Signal, at(100)),
            Err(RegistryError::InvalidFact)
        ));
    }
    let identity = operation_identity(root.path(), "manager-install-failed", 1);
    assert!(matches!(
        terminal_delivery(
            "backup-completed",
            Trigger::User,
            identity.clone(),
            Evidence::Failure,
            at(100)
        ),
        Err(RegistryError::Planned)
    ));
    assert!(matches!(
        terminal_delivery(
            "manager-install-failed",
            Trigger::User,
            identity,
            Evidence::Signal,
            at(100)
        ),
        Err(RegistryError::InvalidFact)
    ));
}

#[test]
fn candidate_partitions_isolate_actions_phases_and_channels_without_reviving_rotated_ids() {
    let root = tempfile::tempdir().unwrap();
    for event in ["manager-install-failed", "preferences-restore-failed"] {
        let first = operation_identity(root.path(), event, 1);
        for (action, phase, channel) in [
            (Action::Check, Phase::Query, first.channel),
            (Action::Read, Phase::Query, first.channel),
            (Action::Pin, Phase::Commit, first.channel),
            (first.action, Phase::Progress, first.channel),
            (first.action, first.phase, Channel::Official),
        ] {
            candidate_identity(root.path(), first.object, action, phase, channel, [2; 32]).unwrap();
        }
        assert_eq!(first, operation_identity(root.path(), event, 1));
        let changed = operation_identity(root.path(), event, 2);
        assert_ne!(first.candidate, changed.candidate);
        assert_ne!(
            first.candidate,
            operation_identity(root.path(), event, 1).candidate
        );
    }
}

fn write_legacy_scope(root: &std::path::Path) -> serde_json::Value {
    let scope = serde_json::json!({"schema_version": 1, "object_id": opaque(123), "candidates": [
        {"object":"backup", "channel":"local", "fingerprint":"01".repeat(32), "id":opaque(456)}
    ]});
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::write(
        root.join("manager-state/event-scope.json"),
        scope.to_string(),
    )
    .unwrap();
    scope
}

#[test]
fn legacy_scope_migration_preserves_old_failures_and_lazily_reuses_exact_candidates() {
    let (root, store) = setup();
    let legacy_scope = write_legacy_scope(root.path());
    let identity = EventIdentity {
        object: ObjectKind::Backup,
        object_id: opaque(123),
        action: Action::Restore,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate: opaque(456),
    };
    let failure = terminal_delivery(
        "preferences-restore-failed",
        Trigger::User,
        identity.clone(),
        Evidence::Failure,
        at(100),
    )
    .unwrap();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    publisher.publish_event(&failure).unwrap();
    let history = std::fs::read(notifications_path(root.path())).unwrap();
    // Even an object-id read persists migration; it never rewrites notification history.
    assert_eq!(data_root_object_id(root.path()).unwrap(), opaque(123));
    let scope_path = root.path().join("manager-state/event-scope.json");
    let migrated: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&scope_path).unwrap()).unwrap();
    assert_eq!(migrated["schema_version"], 2);
    assert_eq!(migrated["legacy_candidates"], legacy_scope["candidates"]);
    assert_eq!(
        std::fs::read(notifications_path(root.path())).unwrap(),
        history
    );
    // Unrelated read/pin operations cannot consume the old candidate.
    operation_identity(root.path(), "preferences-backup-list", 2);
    operation_identity(root.path(), "preferences-backup-pin", 3);
    assert_eq!(
        operation_identity(root.path(), "preferences-restore-failed", 1),
        identity
    );
    let mut loaded = load_notifications(&notifications_path(root.path())).unwrap();
    assert!(!loaded.all()[0].resolved);
    let success = terminal_delivery(
        "preferences-restore-succeeded",
        Trigger::User,
        identity.clone(),
        Evidence::Success {
            candidate: identity.candidate.clone(),
            verified: true,
        },
        at(110),
    )
    .unwrap();
    assert_eq!(deliver(&mut loaded, &success, prefs()).unwrap().resolved, 1);
    // Once this partition changes, the retained legacy seed must never revive it.
    operation_identity(root.path(), "preferences-restore-failed", 2);
    assert_ne!(
        operation_identity(root.path(), "preferences-restore-failed", 1).candidate,
        identity.candidate
    );
}

#[test]
fn mismatched_legacy_candidate_and_invalid_scope_do_not_hide_existing_failure() {
    let root = tempfile::tempdir().unwrap();
    write_legacy_scope(root.path());
    let first = operation_identity(root.path(), "preferences-restore-failed", 2);
    assert_ne!(first.candidate, opaque(456));
    assert_ne!(
        operation_identity(root.path(), "preferences-restore-failed", 1).candidate,
        opaque(456)
    );
    let path = root.path().join("manager-state/event-scope.json");
    let valid = std::fs::read(&path).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&valid).unwrap();
    assert_eq!(
        value["legacy_candidates"][0]["id"],
        serde_json::to_value(opaque(456)).unwrap()
    );
    let mut invalids = vec![b"corrupt".to_vec()];
    let mut future = value.clone();
    future["schema_version"] = 999.into();
    invalids.push(future.to_string().into_bytes());
    let mut duplicate = value.clone();
    let copy = duplicate["candidates"][0].clone();
    duplicate["candidates"].as_array_mut().unwrap().push(copy);
    invalids.push(duplicate.to_string().into_bytes());
    for bytes in invalids {
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            data_root_object_id(root.path()),
            Err(RegistryError::ScopeUnavailable)
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn scope_capacity_refuses_new_partitions_without_evicting_or_corrupting_old_slots() {
    let root = tempfile::tempdir().unwrap();
    let mut legacy = Vec::new();
    for object in [
        "runtime",
        "notification_history",
        "manager",
        "panel",
        "window",
        "tray",
        "sync",
        "backup",
        "configuration",
    ] {
        for channel in ["local", "stable", "beta", "official"] {
            legacy.push(serde_json::json!({"object":object, "channel":channel,
                "fingerprint":"01".repeat(32), "id":opaque(1000 + legacy.len() as u128)}));
        }
    }
    std::fs::create_dir(root.path().join("manager-state")).unwrap();
    std::fs::write(
        root.path().join("manager-state/event-scope.json"),
        serde_json::json!({"schema_version":1,"object_id":opaque(123),"candidates":legacy})
            .to_string(),
    )
    .unwrap();
    let mut first = None;
    let mut count = 0;
    'outer: for object in [ObjectKind::Runtime, ObjectKind::Backup, ObjectKind::Manager] {
        for action in [Action::Install, Action::Restore, Action::Read, Action::Pin] {
            for phase in [Phase::Execution, Phase::Query, Phase::Progress] {
                for channel in [
                    Channel::Local,
                    Channel::Stable,
                    Channel::Beta,
                    Channel::Official,
                ] {
                    if count == MAX_CANDIDATE_SCOPES {
                        break 'outer;
                    }
                    let identity =
                        candidate_identity(root.path(), object, action, phase, channel, [1; 32])
                            .unwrap();
                    first.get_or_insert(identity);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, MAX_CANDIDATE_SCOPES);
    let path = root.path().join("manager-state/event-scope.json");
    let before = std::fs::read(&path).unwrap();
    assert!(before.len() <= 65536);
    let state: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(state["legacy_candidates"], serde_json::json!(legacy));
    assert_eq!(
        state["candidates"].as_array().unwrap().len(),
        MAX_CANDIDATE_SCOPES
    );
    assert_eq!(
        candidate_identity(
            root.path(),
            ObjectKind::Configuration,
            Action::Migrate,
            Phase::Execution,
            Channel::Local,
            [1; 32]
        ),
        Err(RegistryError::ScopeUnavailable)
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let first = first.unwrap();
    assert_eq!(
        candidate_identity(
            root.path(),
            first.object,
            first.action,
            first.phase,
            first.channel,
            [1; 32]
        )
        .unwrap(),
        first
    );
    assert_ne!(
        candidate_identity(
            root.path(),
            first.object,
            first.action,
            first.phase,
            first.channel,
            [2; 32]
        )
        .unwrap()
        .candidate,
        first.candidate
    );
    assert!(data_root_object_id(root.path()).is_ok());
}

#[test]
fn config_recovery_is_structural_and_queries_cannot_be_repurposed_to_resolve_install() {
    fn rejected(id: &str, change: impl FnOnce(&mut serde_json::Value)) {
        let mut value: serde_json::Value = serde_json::from_str(EMBEDDED_CONFIG).unwrap();
        let event = value["events"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["id"] == id)
            .unwrap();
        change(event);
        assert_eq!(
            parse_config(&value.to_string()).unwrap_err(),
            RegistryError::InvalidConfig
        );
    }
    rejected("manager-check-succeeded", |e| {
        e["job"] = "manager_install".into();
        e["action"] = "install".into();
        e["phase"] = "execution".into();
        e["resolves"] = serde_json::json!(["manager-install-failed"]);
    });
    rejected("preferences-restore-succeeded", |e| {
        e["resolves"] = serde_json::json!(["preferences-backup-failed"])
    });
    rejected("manager-install-succeeded", |e| e["phase"] = "query".into());
    rejected("manager-install-succeeded", |e| {
        e["channels"] = serde_json::json!(["stable"])
    });
    rejected("manager-install-succeeded", |e| {
        e["policy"] = "manager_install_failure".into()
    });
    rejected("manager-install-pending-restart", |e| {
        e["resolves"] = serde_json::json!(["manager-install-failed"])
    });
}

#[test]
fn existing_manual_sync_conflict_stays_registered_without_enabling_future_sync_jobs() {
    let config = registry().unwrap();
    let future = config.jobs.iter().find(|j| j.id == Job::Sync).unwrap();
    assert_eq!(future.availability, Availability::Planned);
    let event = lookup("sync-conflict-detected").unwrap();
    assert_eq!(event.job, Job::SyncConflict);
    let job = config.jobs.iter().find(|j| j.id == event.job).unwrap();
    assert_eq!(job.triggers, [Trigger::User]);
    let (root, store) = setup();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    publisher.publish(legacy("sync-conflict-detected")).unwrap();
    publisher.publish(legacy("sync-conflict-detected")).unwrap();
    let saved = load_notifications(&notifications_path(root.path())).unwrap();
    assert_eq!(saved.live().len(), 1);
    assert!(saved.live()[0].body.contains("保留双方历史"));
    let mut altered: serde_json::Value = serde_json::from_str(EMBEDDED_CONFIG).unwrap();
    altered["jobs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|j| j["id"] == "sync_conflict")
        .unwrap()["triggers"] = serde_json::json!(["user", "deadline"]);
    assert_eq!(
        parse_config(&altered.to_string()).unwrap_err(),
        RegistryError::InvalidConfig
    );
}

#[test]
fn transient_signals_validate_without_scopes_or_notification_writes() {
    let (root, store) = setup();
    // A hostile persistence destination cannot affect a pure progress signal.
    std::fs::create_dir(notifications_path(root.path())).unwrap();
    for _ in 0..10_000 {
        validate_signal(
            "manager-install-progress",
            Job::ManagerInstall,
            Trigger::User,
            Channel::Beta,
        )
        .unwrap();
        validate_signal(
            "status-snapshot-changed",
            Job::Observe,
            Trigger::StatusChange,
            Channel::Local,
        )
        .unwrap();
    }
    assert!(store.lock().unwrap().all().is_empty());
    let names: Vec<_> = std::fs::read_dir(root.path().join("manager-state"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        names,
        vec![notifications_path(root.path()).file_name().unwrap()]
    );
}

#[test]
fn signal_adapter_rejects_unknown_planned_wrong_owner_trigger_channel_and_fact() {
    assert_eq!(
        validate_signal("unknown", Job::Install, Trigger::User, Channel::Local),
        Err(RegistryError::UnknownEvent)
    );
    assert_eq!(
        validate_signal(
            "migration-completed",
            Job::Migration,
            Trigger::User,
            Channel::Local
        ),
        Err(RegistryError::Planned)
    );
    assert_eq!(
        validate_signal(
            "manager-install-progress",
            Job::Install,
            Trigger::User,
            Channel::Beta
        ),
        Err(RegistryError::InvalidJob)
    );
    assert_eq!(
        validate_signal(
            "manager-install-progress",
            Job::ManagerInstall,
            Trigger::Deadline,
            Channel::Beta
        ),
        Err(RegistryError::InvalidJob)
    );
    assert_eq!(
        validate_signal(
            "manager-install-progress",
            Job::ManagerInstall,
            Trigger::User,
            Channel::Local
        ),
        Err(RegistryError::InvalidIdentity)
    );
    assert_eq!(
        validate_signal(
            "manager-install-failed",
            Job::ManagerInstall,
            Trigger::User,
            Channel::Stable
        ),
        Err(RegistryError::InvalidFact)
    );
    assert_eq!(
        validate_signal(
            "manager-install-succeeded",
            Job::ManagerInstall,
            Trigger::User,
            Channel::Stable
        ),
        Err(RegistryError::InvalidFact)
    );
}

#[test]
fn validated_signal_site_cannot_be_assigned_a_persisted_terminal_fact() {
    let mut config: serde_json::Value = serde_json::from_str(EMBEDDED_CONFIG).unwrap();
    let site = config["emission_sites"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|site| site["event"] == "manager-install-progress")
        .unwrap();
    site["event"] = "manager-install-failed".into();
    assert_eq!(
        parse_config(&config.to_string()).unwrap_err(),
        RegistryError::InvalidConfig
    );
}

#[test]
fn uninstall_failure_persists_and_only_matching_verified_removal_resolves() {
    verify_local_terminal_resolution("runtime-uninstall", Action::Uninstall);
}

#[test]
fn protection_reconcile_failure_requires_matching_verified_retry() {
    verify_local_terminal_resolution("runtime-protection-reconcile", Action::Reconcile);
}

fn verify_local_terminal_resolution(stem: &str, action: Action) {
    let failure_event = format!("{stem}-failed");
    let success_event = format!("{stem}-succeeded");
    let (root, store) = setup();
    let identity = candidate_identity(
        root.path(),
        ObjectKind::Runtime,
        action,
        Phase::Execution,
        Channel::Local,
        [81; 32],
    )
    .unwrap();
    let failed = terminal_delivery(
        &failure_event,
        Trigger::User,
        identity.clone(),
        Evidence::Failure,
        at(100),
    )
    .unwrap();
    let publisher = NotificationPublisher {
        store: &store,
        data_root: root.path(),
    };
    assert!(publisher.publish_event(&failed).unwrap().added);
    let saved = std::fs::read_to_string(notifications_path(root.path())).unwrap();
    assert!(!saved.contains(root.path().to_str().unwrap()));
    let restarted = Arc::new(Mutex::new(
        load_notifications(&notifications_path(root.path())).unwrap(),
    ));
    let publisher = NotificationPublisher {
        store: &restarted,
        data_root: root.path(),
    };
    assert!(!publisher.publish_event(&failed).unwrap().added);
    assert!(terminal_delivery(
        &success_event,
        Trigger::User,
        identity.clone(),
        Evidence::Success {
            candidate: identity.candidate.clone(),
            verified: false
        },
        at(101)
    )
    .is_err());
    let mut wrong = identity.clone();
    wrong.candidate = opaque(999);
    let success = terminal_delivery(
        &success_event,
        Trigger::User,
        wrong.clone(),
        Evidence::Success {
            candidate: wrong.candidate,
            verified: true,
        },
        at(102),
    )
    .unwrap();
    assert_eq!(publisher.publish_event(&success).unwrap().resolved, 0);
    let install = candidate_identity(
        root.path(),
        ObjectKind::Runtime,
        Action::Install,
        Phase::Execution,
        Channel::Official,
        [81; 32],
    )
    .unwrap();
    let success = terminal_delivery(
        "runtime-install-succeeded",
        Trigger::User,
        install.clone(),
        Evidence::Success {
            candidate: install.candidate,
            verified: true,
        },
        at(103),
    )
    .unwrap();
    assert_eq!(publisher.publish_event(&success).unwrap().resolved, 0);
    let success = terminal_delivery(
        &success_event,
        Trigger::User,
        identity.clone(),
        Evidence::Success {
            candidate: identity.candidate.clone(),
            verified: true,
        },
        at(104),
    )
    .unwrap();
    assert_eq!(publisher.publish_event(&success).unwrap().resolved, 1);
    let loaded = load_notifications(&notifications_path(root.path())).unwrap();
    assert!(loaded.all()[0].resolved);
    assert!(!loaded.all()[0].read);
    let mut deadline = failed;
    deadline.trigger = Trigger::Deadline;
    assert!(publisher.publish_event(&deadline).is_err());
}
