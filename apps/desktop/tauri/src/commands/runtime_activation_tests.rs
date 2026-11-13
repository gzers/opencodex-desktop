use super::*;
use crate::modules::{backup, runtime::program_activation::tests::Fixture};
use std::fs;
fn home(f: &Fixture) -> std::path::PathBuf {
    let home = f.d.with_file_name("opencodex-home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.json"),
        br#"{"port":41234,"providers":{"example":{"apiKey":"synthetic-only"}}}"#,
    )
    .unwrap();
    home
}
fn request(preview: &RecoveryPreview, decision: Decision) -> RecoveryRequest {
    RecoveryRequest {
        attempt_id: preview.attempt_id.clone(),
        confirmation_digest: preview.confirmation_digest.clone(),
        decision,
    }
}

#[test]
fn preview_is_readonly_and_confirmation_resolves_actual_pending_with_config_protection() {
    for initial in [false, true] {
        for decision in [Decision::Complete, Decision::Rollback] {
            let f = Fixture::new(initial);
            let home = home(&f);
            f.interrupt();
            let before = f.snapshot();
            let config = fs::read(home.join("config.json")).unwrap();
            let preview = inspect(&f.i, &f.d, &home).unwrap().unwrap();
            assert_eq!(preview.from, initial.then(|| "2.99.0".into()));
            assert_eq!(preview.to, "3.0.0");
            assert_eq!(f.snapshot(), before);
            assert!(!f.d.join("backups").exists());
            assert!(!serde_json::to_string(&preview)
                .unwrap()
                .contains("synthetic-only"));
            let result = apply(&f.i, &f.d, &home, request(&preview, decision), || Ok(())).unwrap();
            assert_eq!(
                result.selected_version,
                if decision == Decision::Complete {
                    Some("3.0.0".into())
                } else {
                    initial.then(|| "2.99.0".into())
                }
            );
            assert!(inspect(&f.i, &f.d, &home).unwrap().is_none());
            assert_eq!(fs::read(home.join("config.json")).unwrap(), config);
            let backups = backup::safety::scan(&f.d.join("backups")).unwrap();
            assert_eq!(backups.len(), 1);
            assert!(backup::verify_backup(&backups[0]).unwrap());
            assert_eq!(
                fs::read(backups[0].directory.join("config.json")).unwrap(),
                config
            );
            assert!(apply(&f.i, &f.d, &home, request(&preview, decision), || Ok(())).is_err());
            assert_eq!(backup::safety::scan(&f.d.join("backups")).unwrap().len(), 1);
        }
    }
}
#[test]
fn wrong_home_stale_state_and_unknown_owner_leave_originals_without_backup() {
    for case in ["home", "state", "owner"] {
        let f = Fixture::new(true);
        let h = home(&f);
        f.interrupt();
        let preview = inspect(&f.i, &f.d, &h).unwrap().unwrap();
        let target = if case == "home" {
            let other = h.with_file_name("other-home");
            fs::create_dir(&other).unwrap();
            other
        } else {
            h.clone()
        };
        if case == "state" {
            fs::write(f.i.join("runtime/current.json"), b"outside writer").unwrap();
        }
        let before = f.snapshot();
        assert!(
            apply(
                &f.i,
                &f.d,
                &target,
                request(&preview, Decision::Complete),
                || {
                    if case == "owner" {
                        Err(AppError::NotConfigured)
                    } else {
                        Ok(())
                    }
                }
            )
            .is_err(),
            "{case}"
        );
        assert_eq!(f.snapshot(), before);
        assert!(!f.d.join("backups").exists());
        assert!(f.i.join("runtime/.selection-pending.json").exists());
    }
}
#[test]
fn backup_failure_or_service_reappearance_cannot_claim_recovery() {
    for case in ["backup", "service", "configuration"] {
        let f = Fixture::new(true);
        let h = home(&f);
        f.interrupt();
        let preview = inspect(&f.i, &f.d, &h).unwrap().unwrap();
        let before = f.snapshot();
        if case == "backup" {
            fs::write(f.d.join("backups"), b"foreign obstacle").unwrap();
        }
        let checks = std::cell::Cell::new(0);
        let result = apply(
            &f.i,
            &f.d,
            &h,
            request(&preview, Decision::Complete),
            || {
                checks.set(checks.get() + 1);
                if checks.get() > 1 && case == "configuration" {
                    fs::write(h.join("config.json"), b"external edit").unwrap();
                }
                if checks.get() > 1 && case == "service" {
                    Err(AppError::NotConfigured)
                } else {
                    Ok(())
                }
            },
        );
        assert!(result.is_err());
        assert_eq!(f.snapshot(), before);
        assert!(f.i.join("runtime/.selection-pending.json").exists());
        if case == "configuration" {
            assert_eq!(fs::read(h.join("config.json")).unwrap(), b"external edit");
        }
    }
}
#[test]
fn preview_of_absent_program_creates_nothing_and_ipc_rejects_paths() {
    let t = tempfile::tempdir().unwrap();
    let base = t.path().canonicalize().unwrap();
    assert!(inspect(
        &base.join("program"),
        &base.join("desktop"),
        &base.join("home")
    )
    .unwrap()
    .is_none());
    assert_eq!(fs::read_dir(&base).unwrap().count(), 0);
    let payload = serde_json::json!({"attemptId":"op","confirmationDigest":"hash","decision":"rollback","programRoot":"untrusted"});
    assert!(serde_json::from_value::<RecoveryRequest>(payload).is_err());
}
