use super::*;
use crate::commands::update_schedule::{complete_at_root, SharedPanelQuery};
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
                held.sequence = held.sequence.wrapping_add(1);
                held.value = metadata.as_ref().ok().cloned();
                worker_query.1.store(held.sequence, Ordering::Release);
                complete_at_root(&worker_root, Target::Panel, held.value.as_ref(), 101)?;
                let _ = committed.send(if succeed {
                    "panel-check-succeeded"
                } else {
                    "panel-check-failed"
                });
                metadata
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
