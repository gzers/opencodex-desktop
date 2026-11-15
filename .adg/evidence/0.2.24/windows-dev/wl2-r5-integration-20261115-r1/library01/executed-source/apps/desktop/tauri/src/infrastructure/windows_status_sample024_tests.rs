//! Real Windows source/receiver calls with intentionally absent native Owner.
//! A fake source identity can select a branch, never grant native authority.
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::runtime_executable::{RuntimeExecutableProvider, SharedRuntimeExecutable},
    modules::{
        runtime::install::ManagedRuntimeIdentity,
        status::{ClientRecoveryState, StatusCollector},
    },
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
#[derive(Debug)]
struct MissingOwner {
    root: PathBuf,
    calls: Arc<AtomicUsize>,
    delay: Duration,
}
impl RuntimeExecutableProvider for MissingOwner {
    fn executable(&self) -> Option<PathBuf> {
        Some(self.root.join("ocx.exe"))
    }
    fn managed_installation_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
    fn managed_program_identity(&self) -> AppResult<ManagedRuntimeIdentity> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.delay);
        Ok(ManagedRuntimeIdentity {
            installation_root: self.root.clone(),
            prefix: self.root.join("package"),
            entry: self.root.join("ocx.exe"),
            node: self.root.join("node.exe"),
            package_version: "2.99.0".into(),
        })
    }
}
struct Trace {
    source: OfficialStatusSource,
    start: Instant,
    calls: Arc<Mutex<Vec<(&'static str, Instant, Duration)>>>,
}
impl StatusSource for Trace {
    fn requires_fresh_sample(&self) -> bool {
        self.source.requires_fresh_sample()
    }
    fn fetch(&self) -> Result<serde_json::Value, CollectError> {
        panic!("legacy fetch forbidden in native sample")
    }
    fn fetch_at(&self, end: Instant) -> Result<serde_json::Value, CollectError> {
        self.calls
            .lock()
            .unwrap()
            .push(("fetch", end, self.start.elapsed()));
        self.source.fetch_at(end)
    }
    fn fetch_light_at(&self, end: Instant) -> Result<serde_json::Value, CollectError> {
        self.calls
            .lock()
            .unwrap()
            .push(("light", end, self.start.elapsed()));
        self.source.fetch_light_at(end)
    }
    fn client_recovery_at(&self, end: Instant) -> ClientRecoveryState {
        self.calls
            .lock()
            .unwrap()
            .push(("recovery", end, self.start.elapsed()));
        self.source.client_recovery_at(end)
    }
}
fn source(delay: Duration) -> (OfficialStatusSource, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let root = std::env::temp_dir().join(format!("ocx-r5-uncreated-{}", uuid::Uuid::new_v4()));
    let runtime: SharedRuntimeExecutable = Arc::new(MissingOwner {
        root: root.clone(),
        calls: calls.clone(),
        delay,
    });
    let env = crate::modules::process::EnvironmentPolicy {
        opencodex_home: root.join("H"),
        ..Default::default()
    };
    (
        OfficialStatusSource::new(runtime, root.join("Profile"), env),
        calls,
    )
}
#[tokio::test(flavor = "multi_thread")]
async fn wl2_r5_real_windows_source_full_light_same_deadline_no_owner_stays_unknown() {
    let (source, admissions) = source(Duration::ZERO);
    let started = Instant::now();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut collector = StatusCollector::new(Trace {
        source,
        start: started,
        calls: calls.clone(),
    });
    let scheduling = started + Duration::from_secs(3600);
    collector.refresh_at(scheduling).unwrap();
    collector.refresh_light_at(scheduling).unwrap();
    assert_eq!(
        collector.facts().client_recovery,
        ClientRecoveryState::Unknown
    );
    assert_eq!(admissions.load(Ordering::SeqCst), 2);
    let rows = calls.lock().unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        ["fetch", "recovery", "light", "recovery"]
    );
    for pair in rows.chunks_exact(2) {
        assert_eq!(pair[0].1, pair[1].1);
        assert!(pair[0].1 < scheduling);
        assert!(pair[0].1 <= started + status_collect_timeout() + Duration::from_millis(500));
    }
    for (stage, end, elapsed) in rows.iter() {
        println!("r5-native-source stage={stage} elapsedUs={} deadlineFromTraceStartUs={} original5=true owner=missing recovery=Unknown",elapsed.as_micros(),end.duration_since(started).as_micros());
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn wl2_r5_real_windows_source_expired_or_late_admission_cannot_mint_receipt() {
    let (source, calls) = source(Duration::from_millis(20));
    let expired = Instant::now();
    assert_eq!(source.fetch_at(expired), Err(CollectError::Timeout));
    assert_eq!(source.fetch_light_at(expired), Err(CollectError::Timeout));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        source.client_recovery_at(expired),
        ClientRecoveryState::Unknown
    );
    let end = Instant::now() + Duration::from_millis(5);
    let result = source.fetch_at(end);
    assert!(result.is_err() || result.as_ref().is_ok_and(|v| v["status"] == "unknown"));
    assert_eq!(source.client_recovery_at(end), ClientRecoveryState::Unknown);
    assert!(source.native_sample.try_lock().unwrap().is_none());
    println!("r5-native-source expired no-IO; delayed branch admission cannot grant late Closed or a new5s receipt; synthetic source/no real Owner");
}
#[test]
fn wl2_r5_native_receiver_unknown_source_and_wrong_home_refuse_without_creation() {
    let (source, _) = source(Duration::ZERO);
    let end = Instant::now() + status_collect_timeout();
    for home in [
        &source.environment.opencodex_home,
        &source.working_directory,
    ] {
        assert_eq!(
            super::super::windows_official_lifecycle024::recovery_state_at(
                &source.runtime,
                home,
                end
            ),
            ClientRecoveryState::Unknown
        );
        assert!(
            !super::super::windows_official_lifecycle024::native_stopped(&source.runtime, home)
        );
    }
    assert!(!source.working_directory.exists());
    let _ = AppError::NotConfigured;
}
