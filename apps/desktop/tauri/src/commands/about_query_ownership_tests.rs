use super::*;
use crate::commands::update_schedule::{complete_at_root, PanelQuery, SharedPanelQuery};
use crate::infrastructure::storage_writers::WriterGate;
use crate::modules::update::schedule::{self, Schedule, Target};
use std::{sync::atomic::Ordering, time::Duration};

#[tokio::test]
async fn accepted_panel_query_commits_success_or_failure_after_observer_abort() {
    for succeed in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path().to_path_buf();
        let mut initial = Schedule::default();
        initial.complete(Target::ManagerStable, 90, true);
        initial.complete(Target::ManagerBeta, 90, false);
        initial.save(&root_path).unwrap();
        let before = Schedule::load(&root_path).unwrap();
        let gate = Arc::new(WriterGate::default());
        let query = Arc::new(SharedPanelQuery::default());
        let (entered, entry) = tokio::sync::oneshot::channel();
        let (release, released) = tokio::sync::oneshot::channel();
        let (metadata_finished, metadata_done) = tokio::sync::oneshot::channel();
        let (committed, commit_done) = tokio::sync::oneshot::channel();
        let observer_gate = gate.clone();
        let worker_gate = gate.clone();
        let worker_query = query.clone();
        let worker_root = root_path.clone();
        let observer = tokio::spawn(async move {
            run_panel_query_owned(observer_gate, move || async move {
                let mut held = worker_query.0.lock().await;
                let mut state = Schedule::load(&worker_root)?;
                state.reserve(Target::Panel, 100);
                state.save(&worker_root)?;
                // Like run_blocking(query_panel_metadata), accepted metadata work
                // survives dropping its JoinHandle. Terminal commit must survive too.
                let metadata = tokio::spawn(async move {
                    let _storage = worker_gate.admit().unwrap();
                    entered.send(()).unwrap();
                    released.await.unwrap();
                    let _ = metadata_finished.send(());
                    if succeed {
                        Ok(crate::modules::about::remote::OfficialRemoteLatest {
                            tag: "latest".into(),
                            version: "2.84.0".into(),
                            integrity: None,
                        })
                    } else {
                        Err(AppError::Timeout)
                    }
                })
                .await
                .unwrap();
                commit_panel_query_result(
                    &mut held,
                    &worker_query.1,
                    metadata,
                    |value| async move {
                        complete_at_root(&worker_root, Target::Panel, value.as_ref(), 101)?;
                        let _ = committed.send(if succeed {
                            "panel-check-succeeded"
                        } else {
                            "panel-check-failed"
                        });
                        Ok(())
                    },
                )
                .await
            })
            .await
        });
        tokio::time::timeout(Duration::from_secs(2), entry)
            .await
            .unwrap()
            .unwrap();
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert!(
            gate.freeze().unwrap().is_none(),
            "metadata worker still owns storage"
        );
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), metadata_done)
            .await
            .unwrap()
            .unwrap();
        let event = tokio::time::timeout(Duration::from_secs(2), commit_done)
            .await
            .unwrap()
            .expect("accepted panel query lost its terminal commit when observer dropped");
        assert_eq!(
            event,
            if succeed {
                "panel-check-succeeded"
            } else {
                "panel-check-failed"
            }
        );
        let after = Schedule::load(&root_path).unwrap();
        let before_json = serde_json::to_value(before).unwrap();
        let after_json = serde_json::to_value(after).unwrap();
        for key in ["manager-stable", "manager-beta"] {
            let before_entry = before_json["entries"].get(key).unwrap();
            assert_eq!(Some(before_entry), after_json["entries"].get(key));
        }
        let panel = after_json["entries"].get("panel-latest").unwrap();
        assert_eq!(panel["failures"], if succeed { 0 } else { 1 });
        assert_eq!(panel["next_due"], 101 + if succeed { 86400 } else { 300 });
        assert_eq!(
            panel["last_success"],
            if succeed {
                serde_json::json!(101)
            } else {
                serde_json::Value::Null
            }
        );
        assert_eq!(query.1.load(Ordering::Acquire), 1);
        assert_eq!(
            schedule::load_cache::<crate::modules::about::remote::OfficialRemoteLatest>(
                &root_path,
                Target::Panel
            )
            .is_some(),
            succeed
        );
        tokio::time::timeout(Duration::from_secs(2), async {
            while gate.freeze().unwrap().is_none() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn frozen_storage_refuses_panel_setup_and_query_before_side_effects() {
    let gate = Arc::new(WriterGate::default());
    gate.freeze().unwrap().unwrap().commit();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = calls.clone();
    assert!(run_panel_query_owned(gate, move || {
        counter.fetch_add(1, Ordering::SeqCst);
        async { Ok(()) }
    })
    .await
    .is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn failed_panel_commit_does_not_publish_unpersisted_result() {
    let previous = crate::modules::about::remote::OfficialRemoteLatest {
        tag: "previous".into(),
        version: "2.83.0".into(),
        integrity: None,
    };
    let next = crate::modules::about::remote::OfficialRemoteLatest {
        tag: "latest".into(),
        version: "2.84.0".into(),
        integrity: None,
    };
    let mut query = PanelQuery {
        sequence: 4,
        value: Some(previous.clone()),
        error: None,
    };
    let published_sequence = std::sync::atomic::AtomicU64::new(4);
    let result = super::commit_panel_query_result(
        &mut query,
        &published_sequence,
        Ok(next.clone()),
        |_value| async { Err(AppError::NotConfigured) },
    )
    .await;
    assert_eq!(result, Err(AppError::NotConfigured));
    assert_eq!(query.sequence, 5);
    assert_eq!(query.value, Some(previous));
    assert_eq!(query.completed_result(), Err(AppError::NotConfigured));
    assert_eq!(published_sequence.load(Ordering::Acquire), 5);

    let result = super::commit_panel_query_result(
        &mut query,
        &published_sequence,
        Ok(next.clone()),
        |value| async move {
            assert_eq!(value, Some(next));
            Ok(())
        },
    )
    .await
    .unwrap();
    assert_eq!(result.version, "2.84.0");
    assert_eq!(query.sequence, 6);
    assert!(query.error.is_none());
    assert_eq!(query.value.unwrap().version, "2.84.0");
    assert_eq!(published_sequence.load(Ordering::Acquire), 6);
}

#[tokio::test]
async fn delayed_commit_publishes_metadata_failure_only_after_persisted_backoff() {
    use std::future::{poll_fn, Future};
    use std::task::Poll;
    let root = tempfile::tempdir().unwrap();
    let mut schedule = Schedule::default();
    schedule.reserve(Target::Panel, 100);
    schedule.save(root.path()).unwrap();
    let before = std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap();
    let query = SharedPanelQuery::default();
    let mut held = query.0.lock().await;
    let (release, released) = tokio::sync::oneshot::channel();
    let root_path = root.path();
    {
        let mut completion = std::pin::pin!(super::commit_panel_query_result(
            &mut held,
            &query.1,
            Err(AppError::Timeout),
            |value| async move {
                released.await.unwrap();
                complete_at_root(root_path, Target::Panel, value.as_ref(), 101)
            },
        ));
        poll_fn(|cx| {
            assert!(completion.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert_eq!(query.1.load(Ordering::Acquire), 0);
        assert_eq!(
            std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap(),
            before
        );
        assert!(
            query.0.try_lock().is_err(),
            "commit released the waiter mutex"
        );
        release.send(()).unwrap();
        assert_eq!(completion.await, Err(AppError::Timeout));
    }
    assert_eq!(held.completed_result(), Err(AppError::Timeout));
    assert_eq!(query.1.load(Ordering::Acquire), 1);
    assert!(held.value.is_none());
    let state = Schedule::load(root.path()).unwrap();
    assert_eq!(state.entries["panel-latest"].failures, 1);
    assert_eq!(state.entries["panel-latest"].next_due, 101 + 300);
}

#[tokio::test]
async fn failed_durable_commit_is_shared_with_waiters_without_requery() {
    use crate::modules::about::remote::OfficialRemoteLatest;
    use std::future::{poll_fn, Future};
    use std::task::Poll;
    let root = tempfile::tempdir().unwrap();
    let mut schedule = Schedule::default();
    schedule.complete(Target::ManagerStable, 90, true);
    schedule.complete(Target::ManagerBeta, 90, false);
    schedule.reserve(Target::Panel, 100);
    schedule.save(root.path()).unwrap();
    let reserved_bytes = std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap();
    let previous = OfficialRemoteLatest {
        tag: "latest".into(),
        version: "2.83.0".into(),
        integrity: None,
    };
    let next = OfficialRemoteLatest {
        tag: "latest".into(),
        version: "2.84.0".into(),
        integrity: None,
    };
    // Real cache write failure, independent of permission/elevation differences.
    std::fs::write(root.path().join("cache"), b"blocked").unwrap();
    let gate = SharedPanelQuery::default();
    let mut held = gate.0.lock().await;
    held.value = Some(previous.clone());
    let mut waiter = std::pin::pin!(async {
        // Same observed-sequence/mutex branch as the production entry point.
        let observed = gate.1.load(Ordering::Acquire);
        let completed = gate.0.lock().await;
        assert_ne!(
            completed.sequence, observed,
            "waiter would start a second query"
        );
        completed.completed_result()
    });
    poll_fn(|cx| {
        assert!(
            waiter.as_mut().poll(cx).is_pending(),
            "waiter did not wait for the query"
        );
        Poll::Ready(())
    })
    .await;
    let root_path = root.path();
    let result = super::commit_panel_query_result(
        &mut held,
        &gate.1,
        Ok(next.clone()),
        |value| async move { complete_at_root(root_path, Target::Panel, value.as_ref(), 101) },
    )
    .await;
    let error = result.unwrap_err();
    assert_eq!(held.value, Some(previous));
    assert_eq!(
        std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap(),
        reserved_bytes
    );
    assert!(schedule::load_cache::<OfficialRemoteLatest>(root.path(), Target::Panel).is_none());
    assert_eq!(gate.1.load(Ordering::Acquire), 1);
    drop(held);
    // No second reserve/metadata/commit: the actual completion projection must
    // carry the same precise error rather than expose the old success value.
    let shared_error = tokio::time::timeout(Duration::from_secs(2), waiter)
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(
        serde_json::to_value(shared_error).unwrap(),
        serde_json::to_value(&error).unwrap()
    );

    // A later explicit request begins after the failed sequence and can retry.
    std::fs::remove_file(root.path().join("cache")).unwrap();
    let mut retry = gate.0.lock().await;
    assert_eq!(retry.sequence, gate.1.load(Ordering::Acquire));
    let restored = super::commit_panel_query_result(
        &mut retry,
        &gate.1,
        Ok(next.clone()),
        |value| async move { complete_at_root(root_path, Target::Panel, value.as_ref(), 102) },
    )
    .await
    .unwrap();
    assert_eq!(restored, next);
    assert_eq!(retry.completed_result().unwrap(), restored);
    assert_eq!(gate.1.load(Ordering::Acquire), 2);
    let persisted = Schedule::load(root.path()).unwrap();
    assert_eq!(persisted.entries["panel-latest"].next_due, 102 + 86400);
    assert_eq!(persisted.entries["manager-stable"].last_success, Some(90));
    assert_eq!(persisted.entries["manager-beta"].failures, 1);
}
