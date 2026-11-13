use super::*;
use crate::modules::runtime::{bundled::bundled_tests::versioned_fixture, preparation::Phase};
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
fn failed_first_supply_can_be_retried_only_by_its_current_attempt_and_completes_actual_supply() {
    let f = versioned_fixture();
    let resources = f.i.with_file_name("resources");
    std::fs::create_dir(&resources).unwrap();
    let (descriptor, pin) = f.bytes();
    let pin: &'static str = Box::leak(pin.into_boxed_str());
    let parsed: serde_json::Value = serde_json::from_slice(&descriptor).unwrap();
    let name = parsed["archiveName"].as_str().unwrap();
    std::fs::write(resources.join("runtime-bundle.manifest.json"), &descriptor).unwrap();
    std::fs::create_dir_all(f.d.join("logs")).unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&f.d, &f.i);
    let install = SharedRuntimeInstall::new();
    start(
        f.i.clone(),
        f.d.clone(),
        resources.clone(),
        pin,
        runtime.clone(),
        install.clone(),
        || {},
    )
    .unwrap();
    let failure = wait(&runtime);
    assert_eq!(failure.phase, Phase::Failed);
    assert!(failure.retryable);
    assert!(
        f.d.is_dir(),
        "Desktop bootstrap root exists before the worker"
    );
    assert!(
        !resources.join(name).exists(),
        "the fixed archive is deliberately absent"
    );
    assert_eq!(failure.failed_phase, Some(Phase::Preparing));
    assert_eq!(
        crate::infrastructure::hash::sha256_hex(
            &std::fs::read(resources.join("runtime-bundle.manifest.json")).unwrap()
        ),
        pin
    );
    assert!(runtime.selected_program().unwrap().is_none());
    std::fs::copy(&f.archive, resources.join(name)).unwrap();
    assert!(retry(
        f.d.clone(),
        resources.clone(),
        pin,
        runtime.clone(),
        install.clone(),
        "foreign".into(),
        || {}
    )
    .is_err());
    let (sender, receiver) = std::sync::mpsc::channel();
    retry(
        f.d.clone(),
        resources,
        pin,
        runtime.clone(),
        install.clone(),
        failure.attempt_id.unwrap(),
        move || sender.send(()).unwrap(),
    )
    .unwrap();
    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap_or_else(|e| {
            panic!(
                "{e:?}; state={:?}; log={}",
                runtime.preparation.snapshot(),
                std::fs::read_to_string(
                    crate::infrastructure::runtime_log::RuntimeLog::new(&f.d).path()
                )
                .unwrap_or_default()
            )
        });
    let done = wait(&runtime);
    assert_eq!(done.phase, Phase::Supplied);
    assert!(!done.retryable);
    assert!(runtime.selected_program().unwrap().is_some());
    assert!(install.acquire().is_some());
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
