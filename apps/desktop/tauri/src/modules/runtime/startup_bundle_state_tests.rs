use super::*;
use crate::modules::runtime::{
    bundled::bundled_tests::{versioned_fixture, Fixture},
    preparation::{Failure, Phase, Snapshot},
};

const ARCHIVE_NAME: &str = "runtime-bundle-0.2.24-darwin-arm64.tar.gz";

struct StartupFixture {
    bundle: Fixture,
    resources: PathBuf,
    runtime: Arc<RuntimeHandle>,
    install: SharedRuntimeInstall,
}

impl StartupFixture {
    fn new() -> Self {
        let bundle = versioned_fixture();
        let resources = bundle.i.with_file_name("resources");
        fs::create_dir(&resources).unwrap();
        fs::create_dir_all(bundle.d.join("logs")).unwrap();
        let runtime = RuntimeHandle::initialize_versioned(&bundle.d, &bundle.i);
        Self {
            bundle,
            resources,
            runtime,
            install: SharedRuntimeInstall::new(),
        }
    }

    fn descriptor(&self) -> serde_json::Value {
        let (bytes, _) = self.bundle.bytes();
        let mut descriptor: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        descriptor["archiveName"] = ARCHIVE_NAME.into();
        descriptor
    }

    fn write_descriptor(&self, descriptor: &serde_json::Value) -> &'static str {
        let bytes = serde_json::to_vec(descriptor).unwrap();
        let pin = crate::infrastructure::hash::sha256_hex(&bytes);
        fs::write(self.resources.join("runtime-bundle.manifest.json"), bytes).unwrap();
        Box::leak(pin.into_boxed_str())
    }

    fn start(&self, pin: &'static str) -> Snapshot {
        start(
            self.bundle.i.clone(),
            self.bundle.d.clone(),
            self.resources.clone(),
            pin,
            self.runtime.clone(),
            self.install.clone(),
            || {},
        )
        .unwrap();
        wait(&self.runtime)
    }

    fn retry(
        &self,
        pin: &'static str,
        expected: String,
        transfer: Option<bundle_download::Transfer>,
        ready: impl FnOnce() + Send + 'static,
    ) -> AppResult<()> {
        retry(
            self.bundle.d.clone(),
            self.resources.clone(),
            pin,
            self.runtime.clone(),
            self.install.clone(),
            Retry { expected, transfer },
            ready,
        )
    }

    fn assert_no_download_stage(&self) {
        assert!(fs::read_dir(&self.bundle.d).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".runtime-bundle-download-")));
    }
}

fn assert_failure(snapshot: &Snapshot, reason: Failure, failed_phase: Phase) {
    assert_eq!(snapshot.phase, Phase::Failed);
    assert_eq!(snapshot.failure, Some(reason));
    assert_eq!(snapshot.failed_phase, Some(failed_phase));
    assert!(snapshot.retryable);
}

fn assert_stale(result: AppResult<()>) {
    assert!(
        matches!(result, Err(AppError::RuntimeManaged { code, .. }) if code == "preparation_changed")
    );
}

fn forbidden_transfer() -> bundle_download::Transfer {
    Box::new(|_, _| panic!("this attempt must not run any download"))
}

fn wait(runtime: &RuntimeHandle) -> super::super::preparation::Snapshot {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let s = runtime.preparation.snapshot().unwrap();
        if !s.phase.active() {
            return s;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "worker did not settle"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
#[test]
fn absent_archive_requires_current_explicit_consent_or_can_be_retried_offline() {
    let f = StartupFixture::new();
    let pin = f.write_descriptor(&f.descriptor());
    let missing = f.start(pin);
    assert_failure(&missing, Failure::MissingArchive, Phase::Preparing);
    assert!(f.runtime.selected_program().unwrap().is_none());
    let first = missing.attempt_id.unwrap();

    f.retry(pin, first.clone(), None, || panic!("unexpected supply"))
        .unwrap();
    let retried = wait(&f.runtime);
    assert_failure(&retried, Failure::MissingArchive, Phase::Preparing);
    assert_ne!(retried.attempt_id.as_deref(), Some(first.as_str()));
    assert_stale(f.retry(pin, first, Some(forbidden_transfer()), || {}));
    f.assert_no_download_stage();

    fs::copy(&f.bundle.archive, f.resources.join(ARCHIVE_NAME)).unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    f.retry(pin, retried.attempt_id.unwrap(), None, move || {
        send.send(()).unwrap()
    })
    .unwrap();
    receive
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    let done = wait(&f.runtime);
    assert_eq!(done.phase, Phase::Supplied);
    assert!(!done.retryable);
    assert!(f.runtime.selected_program().unwrap().is_some());
    assert!(f.install.acquire().is_some());
    f.assert_no_download_stage();
}

#[test]
fn mock_download_reaches_actual_provision_and_removes_stage() {
    let f = StartupFixture::new();
    let pin = f.write_descriptor(&f.descriptor());
    let missing = f.start(pin);
    assert_failure(&missing, Failure::MissingArchive, Phase::Preparing);
    let previous = missing.attempt_id.unwrap();
    let runtime = f.runtime.clone();
    let bytes = fs::read(&f.bundle.archive).unwrap();
    let digest = crate::infrastructure::hash::sha256_hex(&bytes);
    let expected = previous.clone();
    let (send_attempt, receive_attempt) = std::sync::mpsc::channel();
    let transfer: bundle_download::Transfer = Box::new(move |asset, sink| {
        let snapshot = runtime.preparation.snapshot().unwrap();
        assert_eq!(snapshot.phase, Phase::Downloading);
        let attempt = snapshot.attempt_id.unwrap();
        assert_ne!(attempt, previous);
        send_attempt.send(attempt).unwrap();
        assert_eq!(asset.name, ARCHIVE_NAME);
        assert_eq!(asset.bytes, bytes.len() as u64);
        assert_eq!(asset.sha256, digest);
        for chunk in bytes.chunks(64) {
            sink.write(chunk)?;
        }
        Ok(())
    });
    let (send, receive) = std::sync::mpsc::channel();
    f.retry(pin, expected, Some(transfer), move || {
        send.send(()).unwrap()
    })
    .unwrap();
    receive
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    let done = wait(&f.runtime);
    assert_eq!(done.phase, Phase::Supplied);
    assert_eq!(done.attempt_id, Some(receive_attempt.try_recv().unwrap()));
    assert_eq!(
        f.runtime
            .selected_program()
            .unwrap()
            .unwrap()
            .selection
            .version,
        "2.99.0"
    );
    assert!(!f.resources.join(ARCHIVE_NAME).exists());
    assert!(f.install.acquire().is_some());
    f.assert_no_download_stage();
}

#[test]
fn absent_archive_with_bad_descriptor_is_not_eligible_for_download() {
    for invalid in ["pin", "platform", "filename", "content"] {
        let f = StartupFixture::new();
        let mut descriptor = f.descriptor();
        match invalid {
            "platform" => descriptor["platform"] = "win32-x64".into(),
            "filename" => descriptor["archiveName"] = "another-archive.tar.gz".into(),
            "content" => descriptor["archiveBytes"] = 0.into(),
            _ => (),
        }
        let mut pin = f.write_descriptor(&descriptor);
        if invalid == "pin" {
            pin = "0000000000000000000000000000000000000000000000000000000000000000";
        }
        let failure = f.start(pin);
        assert_failure(&failure, Failure::Integrity, Phase::Preparing);
        assert_stale(f.retry(
            pin,
            failure.attempt_id.unwrap(),
            Some(forbidden_transfer()),
            || {},
        ));
        assert!(f.runtime.selected_program().unwrap().is_none());
        f.assert_no_download_stage();
    }
}

#[test]
fn corrupt_present_archive_fails_closed_after_previously_observed_absence() {
    let f = StartupFixture::new();
    let pin = f.write_descriptor(&f.descriptor());
    let missing = f.start(pin);
    assert_failure(&missing, Failure::MissingArchive, Phase::Preparing);
    let mut corrupt = fs::read(&f.bundle.archive).unwrap();
    corrupt[0] ^= 1;
    let archive = f.resources.join(ARCHIVE_NAME);
    fs::write(&archive, &corrupt).unwrap();
    f.retry(
        pin,
        missing.attempt_id.unwrap(),
        Some(forbidden_transfer()),
        || {},
    )
    .unwrap();
    assert_failure(&wait(&f.runtime), Failure::Integrity, Phase::Verifying);
    assert_eq!(fs::read(archive).unwrap(), corrupt);
    assert!(f.runtime.selected_program().unwrap().is_none());
    f.assert_no_download_stage();
}

#[test]
fn incomplete_oversized_and_corrupt_downloads_leave_no_stage_or_installation() {
    for invalid in ["short", "oversized", "digest"] {
        let f = StartupFixture::new();
        let pin = f.write_descriptor(&f.descriptor());
        let missing = f.start(pin);
        assert_failure(&missing, Failure::MissingArchive, Phase::Preparing);
        let mut bytes = fs::read(&f.bundle.archive).unwrap();
        match invalid {
            "short" => {
                bytes.pop();
            }
            "oversized" => bytes.push(0),
            _ => bytes[0] ^= 1,
        }
        f.retry(
            pin,
            missing.attempt_id.unwrap(),
            Some(Box::new(move |_, sink| sink.write(&bytes))),
            || {},
        )
        .unwrap();
        assert_failure(&wait(&f.runtime), Failure::Integrity, Phase::Downloading);
        assert!(f.runtime.selected_program().unwrap().is_none());
        assert!(!f.bundle.i.join("runtime/versions/2.99.0").exists());
        f.assert_no_download_stage();
    }
}

#[test]
fn actual_bundle_callbacks_follow_completed_verification_extract_and_activation_boundaries() {
    let f = versioned_fixture();
    let (descriptor, pin) = f.bytes();
    let phases = std::sync::Mutex::new(Vec::new());
    crate::modules::runtime::bundled::versioned::provision_observed(
        &f.i,
        &f.d,
        &f.archive,
        &descriptor,
        &pin,
        &|p| phases.lock().unwrap().push(p),
    )
    .unwrap();
    assert_eq!(
        *phases.lock().unwrap(),
        vec![
            Phase::Preparing,
            Phase::Verifying,
            Phase::Extracting,
            Phase::Assembling,
            Phase::Activating
        ]
    );
    assert!(RuntimeHandle::initialize_versioned(&f.d, &f.i)
        .selected_program()
        .unwrap()
        .is_some());
}
#[test]
fn corrupt_archive_never_announces_extract_or_activation_and_existing_pending_requires_review() {
    let f = versioned_fixture();
    let (descriptor, pin) = f.bytes();
    std::fs::write(&f.archive, b"invalid complete resource").unwrap();
    let phases = std::sync::Mutex::new(Vec::new());
    assert!(
        crate::modules::runtime::bundled::versioned::provision_observed(
            &f.i,
            &f.d,
            &f.archive,
            &descriptor,
            &pin,
            &|p| phases.lock().unwrap().push(p)
        )
        .is_err()
    );
    assert!(!phases.lock().unwrap().contains(&Phase::Extracting));
    assert!(!phases.lock().unwrap().contains(&Phase::Activating));
    std::fs::create_dir_all(f.i.join("runtime")).unwrap();
    std::fs::write(
        f.i.join("runtime/.repair-pending.json"),
        b"unknown original",
    )
    .unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&f.d, &f.i);
    assert!(start(
        f.i.clone(),
        f.d.clone(),
        f.i.join("no-resources"),
        "unused",
        runtime.clone(),
        SharedRuntimeInstall::new(),
        || {}
    )
    .is_err());
    let s = runtime.preparation.snapshot().unwrap();
    assert_eq!(s.phase, Phase::Review);
    assert!(!s.retryable);
    assert_eq!(
        std::fs::read(f.i.join("runtime/.repair-pending.json")).unwrap(),
        b"unknown original"
    );
}
