//! A real timer releases another thread's collector lock. Blocking the sole
//! executor thread prevents the timer from running. The isolated child's
//! watchdog exits91 so an old deadlock cannot hang the test harness.
use opencodex_desktop_lib::{
    modules::status::{
        polling::{BackgroundStateProvider, StatusPollingService, StatusSnapshotEmitter},
        StatusCollector, VirtualStatusSource,
    },
    types::runtime_status::StatusSnapshotDto,
};
use std::{
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
struct Emitter;
impl StatusSnapshotEmitter for Emitter {
    fn emit(&self, _: &StatusSnapshotDto) {}
}
struct Foreground;
impl BackgroundStateProvider for Foreground {
    fn backgrounded(&self) -> bool {
        false
    }
}

#[test]
fn occupied_sampler_never_blocks_the_executor_that_must_release_it() {
    const CHILD: &str = "OCXD024_POLLING_CONTENTION_CHILD";
    const TEST: &str = "occupied_sampler_never_blocks_the_executor_that_must_release_it";
    if std::env::var_os(CHILD).is_none() {
        for phase in ["poll", "schedule"] {
            let out = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture"])
                .env(CHILD, phase)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{phase}: child={:?}; stdout={}; stderr={}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            println!("{}", String::from_utf8_lossy(&out.stdout));
        }
        return;
    }
    let phase = std::env::var(CHILD).unwrap();
    assert!(["poll", "schedule"].contains(&phase.as_str()));
    let (done, watch) = mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if watch.recv_timeout(Duration::from_secs(2)).is_err() {
            eprintln!(
                "COLLECTOR_EXECUTOR_STARVATION_WATCHDOG phase={}",
                std::env::var(CHILD).unwrap()
            );
            std::process::exit(91);
        }
    });
    let collector = Arc::new(Mutex::new(StatusCollector::new(VirtualStatusSource::new(
        Ok(serde_json::json!({"status":"stopped"})),
    ))));
    let mut service = StatusPollingService::new(collector.clone(), Emitter, Foreground);
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let held = collector.clone();
    let holder = std::thread::spawn(move || {
        let _held = held.lock().unwrap();
        entered.send(()).unwrap();
        released.recv().unwrap();
    });
    entry.recv().unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    executor.block_on(async {
        let releaser = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            release.send(()).unwrap();
        });
        let observed = tokio::spawn(async move {
            let now = Instant::now();
            if phase == "poll" {
                service.poll_once().await;
            } else {
                assert!(!service.next_wait().is_zero());
            }
            assert!(now.elapsed() < Duration::from_secs(1));
        });
        observed.await.unwrap();
        releaser.await.unwrap();
    });
    holder.join().unwrap();
    done.send(()).unwrap();
    watchdog.join().unwrap();
    assert_eq!(collector.lock().unwrap().failure_count(), 0);
    println!("COLLECTOR_EXECUTOR_RESPONSIVE");
}
