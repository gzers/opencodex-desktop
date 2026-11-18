//! One exact state query, no GUI/cancellation/lifecycle. Reports all phases and
//! raw elapsed time under the adopted budget rather than accepting a late read.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit new isolated status-probe root and fixed official resources required"]
fn trace_one_official_status_on_reserved_partial_root() {
    use opencodex_desktop_lib::{
        commands::get_status_snapshot_with_collector,
        infrastructure::macos_client_originals::session_index::Index,
        infrastructure::{platform, status_source::OfficialStatusSource},
        modules::{
            process::EnvironmentPolicy,
            runtime::{self, RuntimeHandle},
            status::StatusCollector,
        },
        state::ProcessContext,
    };
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };
    let root =
        PathBuf::from(std::env::var_os("OCXD024_STATUS_PROBE_ROOT").expect("explicit taskroot"));
    assert!(
        root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-status-probe-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join("desktop");
    let home = profile.join("opencodex");
    let program = root.join("program");
    platform::create_private_dir_all(&data, 0o700).unwrap();
    platform::create_private_dir_all(&home, 0o700).unwrap();
    fs::write(
        home.join("config.json"),
        b"{\"hostname\":\"127.0.0.1\",\"port\":0,\"providers\":{}}\n",
    )
    .unwrap();
    fs::set_permissions(home.join("config.json"), fs::Permissions::from_mode(0o600)).unwrap();
    let resources = root.join("resources");
    let descriptor = fs::read(resources.join("runtime-bundle.manifest.json")).unwrap();
    let pin = opencodex_desktop_lib::infrastructure::hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    runtime::bundled::versioned::provision(
        &program,
        &data,
        &resources.join("runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
        &descriptor,
        &pin,
    )
    .unwrap();
    let handle = RuntimeHandle::initialize_versioned(&data, &program);
    let context = ProcessContext {
        runtime: handle.clone(),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let (index, preparing) =
        Index::prepare_start_capture(&context, &data, Instant::now() + Duration::from_secs(30))
            .unwrap();
    let reserved = index.reserve_first_root_capture(&preparing).unwrap();
    let slot = data
        .join("manager-state/client-originals")
        .join(&reserved.ticket);
    platform::create_private_dir_all(&slot, 0o700).unwrap();
    fs::write(
        slot.join("synthetic-fragment"),
        b"explicit synthetic fragment; not Originals",
    )
    .unwrap();
    fs::set_permissions(
        slot.join("synthetic-fragment"),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    drop(index);
    let source = OfficialStatusSource::new(
        handle,
        &profile,
        EnvironmentPolicy {
            home: Some(profile.clone().into_os_string()),
            opencodex_home: home.clone(),
            ..Default::default()
        },
    )
    .with_client_recovery_root(data.clone());
    let collector = Arc::new(Mutex::new(StatusCollector::new(source)));
    let contention = std::env::var_os("OCXD024_STATUS_PROBE_POLLING").as_deref()
        == Some(std::ffi::OsStr::new("1"));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(if contention { 1 } else { 2 })
        .enable_all()
        .build()
        .unwrap();
    let started = Instant::now();
    let queried = runtime.block_on(async {
        let other = collector.clone();
        let sampled =
            tokio::task::spawn_blocking(move || get_status_snapshot_with_collector(&collector));
        if contention {
            use opencodex_desktop_lib::modules::status::polling::{
                BackgroundStateProvider, StatusPollingService, StatusSnapshotEmitter,
            };
            struct Emitter;
            impl StatusSnapshotEmitter for Emitter {
                fn emit(
                    &self,
                    _: &opencodex_desktop_lib::types::runtime_status::StatusSnapshotDto,
                ) {
                    panic!("busy tick cannot emit a successful sample");
                }
            }
            struct Foreground;
            impl BackgroundStateProvider for Foreground {
                fn backgrounded(&self) -> bool {
                    false
                }
            }
            let poll = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                assert!(
                    other.try_lock().is_err(),
                    "actual official sample must still hold the collector, not a missed overlap"
                );
                let mut service = StatusPollingService::new(other, Emitter, Foreground);
                let before = Instant::now();
                service.poll_once().await;
                assert!(!service.next_wait().is_zero());
                assert!(before.elapsed() < Duration::from_millis(100));
                eprintln!("STATUS024_ACTUAL_BUSY_POLLER_RETURNED");
            });
            let result = sampled.await.unwrap();
            poll.await.unwrap();
            result
        } else {
            sampled.await.unwrap()
        }
    });
    let elapsed = started.elapsed().as_millis();
    let value = match queried {
        Ok(value) => {
            serde_json::json!({"matrix":value.matrix,"canStart":value.can_start,"clientRecovery":value.facts.client_recovery})
        }
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    let result = serde_json::json!({"scope":"one actual official status on reserved partial root; no cancel/Start/Stop or model requests","elapsedMs":elapsed,"budgetMs":5000,"withinBudget":elapsed<5000,"actualBackgroundContention":contention,"snapshot":value,"releaseAcceptance":false,"serviceStarted":false});
    fs::write(
        root.join("status-probe-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("STATUS024_PROBE_RESULT {result}");
    assert!(!home.join("runtime-port.json").exists());
    assert!(
        elapsed < 5000,
        "late status is a real diagnostic failure; preserve phase trace and original5000ms budget"
    );
}
