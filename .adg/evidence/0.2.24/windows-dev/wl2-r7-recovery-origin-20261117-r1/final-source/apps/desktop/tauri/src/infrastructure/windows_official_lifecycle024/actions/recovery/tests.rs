use super::*;
fn failed(owner: uuid::Uuid, start: &ActionOrigin) -> StartFailure {
    StartFailure::record(owner, start.id, false, &Err(AppError::Timeout)).unwrap()
}
fn start(end: Instant) -> ActionOrigin {
    let now = Instant::now();
    let window = shared::ActionWindow::begin(LifecycleAction::Start, now, end).unwrap();
    ActionOrigin::admit(OriginalActionKind::Start, &window).unwrap()
}
#[test]
fn wl2_r7_recovery_origin_expired_start_is_metadata_only_and_fresh_end_never_replaces_it() {
    let original = start(Instant::now() + Duration::from_millis(30));
    let owner = uuid::Uuid::new_v4();
    let failure = failed(owner, &original);
    while Instant::now() < original.end {
        std::thread::sleep(Duration::from_millis(1));
    }
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(2)).unwrap();
    let attempt = ProcessAbortAttempt::pending(owner, &original, Some(&failure), &window);
    assert!(matches!(
        original.check(original.end),
        Err(AppError::Timeout)
    ));
    attempt
        .check_origin(owner, &original, Some(&failure), window.deadline())
        .unwrap();
    assert!(original.check(window.deadline()).is_err());
    assert_eq!(attempt.original_end, original.end);
    assert_eq!(attempt.end, window.deadline());
    assert!(attempt.launcher.is_none() && attempt.scope.is_none() && attempt.token.is_none());
    assert!(attempt.phase == ProcessPhase::Pending);
    println!("r7 actual admitted expired Start stays Timeout; new bounded recovery comparison never updates old permission; no native/restore capability constructed");
}
#[test]
fn wl2_r7_recovery_origin_owner_generation_deadline_and_old_scope_substitutions_refuse() {
    let original = start(Instant::now() + Duration::from_secs(2));
    let other = start(Instant::now() + Duration::from_secs(2));
    let owner = uuid::Uuid::new_v4();
    let failure = failed(owner, &original);
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(1)).unwrap();
    let mut attempt = ProcessAbortAttempt::pending(owner, &original, Some(&failure), &window);
    attempt
        .check_origin(owner, &original, Some(&failure), window.deadline())
        .unwrap();
    assert!(attempt
        .check_origin(
            uuid::Uuid::new_v4(),
            &original,
            Some(&failure),
            window.deadline()
        )
        .is_err());
    assert!(attempt
        .check_origin(owner, &other, Some(&failure), window.deadline())
        .is_err());
    assert!(attempt
        .check_origin(
            owner,
            &original,
            Some(&failure),
            window.deadline() + Duration::from_nanos(1)
        )
        .is_err());
    attempt.original_stop += Duration::from_nanos(1);
    assert!(attempt
        .check_origin(owner, &original, Some(&failure), window.deadline())
        .is_err());
    attempt.original_stop = original.stop_end;
    attempt.id = attempt.failed_start;
    assert!(attempt
        .check_origin(owner, &original, Some(&failure), window.deadline())
        .is_err());
}
#[test]
fn wl2_r7_recovery_origin_every_retained_phase_blocks_normal_dispatch_without_dropping_state() {
    require_unoccupied(&None).unwrap();
    let original = start(Instant::now() + Duration::from_secs(2));
    let owner = uuid::Uuid::new_v4();
    let failure = failed(owner, &original);
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(1)).unwrap();
    for phase in [
        ProcessPhase::Pending,
        ProcessPhase::Prepared,
        ProcessPhase::Failed,
    ] {
        let mut slot = Some(ProcessAbortAttempt::pending(
            owner,
            &original,
            Some(&failure),
            &window,
        ));
        slot.as_mut().unwrap().phase = phase;
        let id = slot.as_ref().unwrap().id;
        assert!(require_unoccupied(&slot).is_err());
        assert_eq!(slot.as_ref().unwrap().id, id);
        assert!(slot.as_ref().unwrap().launcher.is_none());
    }
    println!("r7 original occupied field denies ordinary dispatch in all phases; metadata-only test does not fabricate a native prepared proof");
}
#[test]
fn wl2_r7_recovery_origin_requires_actual_local_failed_start_result_and_never_resumed_branch() {
    let original = start(Instant::now() + Duration::from_secs(2));
    let owner = uuid::Uuid::new_v4();
    assert!(
        StartFailure::record(owner, original.id, false, &Ok(LifecycleResult::Started)).is_none()
    );
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(1)).unwrap();
    let absent = ProcessAbortAttempt::pending(owner, &original, None, &window);
    assert!(absent
        .check_origin(owner, &original, None, window.deadline())
        .is_err());
    let result: AppResult<LifecycleResult> = Err(AppError::Timeout);
    let failure = StartFailure::record(owner, original.id, false, &result).unwrap();
    let actual = ProcessAbortAttempt::pending(owner, &original, Some(&failure), &window);
    actual
        .check_origin(owner, &original, Some(&failure), window.deadline())
        .unwrap();
    assert!(
        matches!(result, Err(AppError::Timeout)),
        "old Result unchanged"
    );
    let resumed = StartFailure::record(owner, original.id, true, &Err(AppError::Timeout)).unwrap();
    let bad = ProcessAbortAttempt::pending(owner, &original, Some(&resumed), &window);
    assert!(bad
        .check_origin(owner, &original, Some(&resumed), window.deadline())
        .is_err());
    let foreign = StartFailure::record(
        uuid::Uuid::new_v4(),
        original.id,
        false,
        &Err(AppError::Timeout),
    )
    .unwrap();
    let bad = ProcessAbortAttempt::pending(owner, &original, Some(&foreign), &window);
    assert!(bad
        .check_origin(owner, &original, Some(&foreign), window.deadline())
        .is_err());
    println!("r7 local Result provenance: success/absent/resumed/foreign failure cannot admit original beforeResume; old Timeout result remains unchanged; metadata only, no native proof fabricated");
}
#[test]
fn wl2_r7_recovery_origin_native_errors_keep_original_raw_code() {
    let error = native_failure(std::io::Error::from_raw_os_error(5));
    match error {
        AppError::RuntimeManaged { code, detail } => {
            assert_eq!(code, "windows024_before_resume_native_unconfirmed");
            assert!(detail.contains("raw=Some(5)"));
            assert!(detail.contains("kind=PermissionDenied"));
        }
        _ => panic!("raw native beforeResume error was lost"),
    }
}
