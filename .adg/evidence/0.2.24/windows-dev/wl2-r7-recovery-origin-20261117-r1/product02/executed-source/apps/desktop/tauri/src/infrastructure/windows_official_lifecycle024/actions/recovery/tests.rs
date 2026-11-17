use super::*;
fn start(end: Instant) -> ActionOrigin {
    let now = Instant::now();
    let window = shared::ActionWindow::begin(LifecycleAction::Start, now, end).unwrap();
    ActionOrigin::admit(OriginalActionKind::Start, &window).unwrap()
}
#[test]
fn wl2_r7_recovery_origin_expired_start_is_metadata_only_and_fresh_end_never_replaces_it() {
    let original = start(Instant::now() + Duration::from_millis(30));
    let owner = uuid::Uuid::new_v4();
    while Instant::now() < original.end {
        std::thread::sleep(Duration::from_millis(1));
    }
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(2)).unwrap();
    let attempt = ProcessAbortAttempt::pending(owner, &original, &window);
    assert!(matches!(
        original.check(original.end),
        Err(AppError::Timeout)
    ));
    attempt
        .check_origin(owner, &original, window.deadline())
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
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(1)).unwrap();
    let mut attempt = ProcessAbortAttempt::pending(owner, &original, &window);
    attempt
        .check_origin(owner, &original, window.deadline())
        .unwrap();
    assert!(attempt
        .check_origin(uuid::Uuid::new_v4(), &original, window.deadline())
        .is_err());
    assert!(attempt
        .check_origin(owner, &other, window.deadline())
        .is_err());
    assert!(attempt
        .check_origin(
            owner,
            &original,
            window.deadline() + Duration::from_nanos(1)
        )
        .is_err());
    attempt.original_stop += Duration::from_nanos(1);
    assert!(attempt
        .check_origin(owner, &original, window.deadline())
        .is_err());
    attempt.original_stop = original.stop_end;
    attempt.id = attempt.failed_start;
    assert!(attempt
        .check_origin(owner, &original, window.deadline())
        .is_err());
}
#[test]
fn wl2_r7_recovery_origin_every_retained_phase_blocks_normal_dispatch_without_dropping_state() {
    require_unoccupied(&None).unwrap();
    let original = start(Instant::now() + Duration::from_secs(2));
    let owner = uuid::Uuid::new_v4();
    let now = Instant::now();
    let window = BeforeResumeRecoveryWindow::begin(now, now + Duration::from_secs(1)).unwrap();
    for phase in [
        ProcessPhase::Pending,
        ProcessPhase::Prepared,
        ProcessPhase::Failed,
    ] {
        let mut slot = Some(ProcessAbortAttempt::pending(owner, &original, &window));
        slot.as_mut().unwrap().phase = phase;
        let id = slot.as_ref().unwrap().id;
        assert!(require_unoccupied(&slot).is_err());
        assert_eq!(slot.as_ref().unwrap().id, id);
        assert!(slot.as_ref().unwrap().launcher.is_none());
    }
    println!("r7 original occupied field denies ordinary dispatch in all phases; metadata-only test does not fabricate a native prepared proof");
}
