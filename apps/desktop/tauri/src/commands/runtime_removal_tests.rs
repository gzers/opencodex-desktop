use super::*;
use crate::modules::runtime::terminal_cli::tests::Fixture;
use std::fs;
fn fixture() -> Fixture {
    let f = Fixture::new();
    let c = &f.c;
    fs::write(
        c.opencodex_home.join("config.json"),
        br#"{"port":41234,"providers":{"test":{"apiKey":"synthetic-only"}}}"#,
    )
    .unwrap();
    f.register_program();
    f
}
#[test]
fn review_is_readonly_and_one_use_confirm_removes_registered_program_and_owned_cli_only() {
    for enabled in [false, true] {
        let f = fixture();
        let c = &f.c;
        if enabled {
            f.change(terminal_cli::Action::Enable);
        }
        let config = fs::read(c.opencodex_home.join("config.json")).unwrap();
        let reviews = Reviews::default();
        let review = prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews)
            .unwrap()
            .unwrap();
        assert_eq!(review.command_path.is_some(), enabled);
        assert!(
            crate::modules::backup::safety::scan(&c.desktop_root.join("backups"))
                .unwrap()
                .is_empty()
        );
        assert!(c.program_root.join("runtime/current.json").exists());
        assert!(!serde_json::to_string(&review)
            .unwrap()
            .contains("synthetic-only"));
        let request = Request {
            id: review.id.clone(),
            confirmation_digest: review.confirmation_digest.clone(),
        };
        let ticket = consume(&reviews, &request, c).unwrap();
        assert!(consume(&reviews, &request, c).is_err());
        let outcome = execute(ticket, || Ok(())).unwrap();
        assert_eq!(outcome.operation_id, review.removal.operation_id);
        assert!(outcome.configuration_preserved);
        assert_eq!(
            fs::read(c.opencodex_home.join("config.json")).unwrap(),
            config
        );
        assert!(!c.user_profile.join(".local/bin/ocx").exists());
        assert!(!c.program_root.join("runtime/bin/ocx").exists());
        assert!(program_removal::intentionally_removed(&c.program_root, &c.desktop_root).unwrap());
        assert!(prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews)
            .unwrap()
            .is_none());
    }
}
#[test]
fn terminal_changes_unknown_service_and_wrong_confirmation_refuse_without_code_removal() {
    for case in ["terminal", "service", "digest", "context", "expired"] {
        let f = fixture();
        let c = &f.c;
        f.change(terminal_cli::Action::Enable);
        let reviews = Reviews::default();
        let review = prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews)
            .unwrap()
            .unwrap();
        let request = Request {
            id: review.id,
            confirmation_digest: if case == "digest" {
                "wrong".into()
            } else {
                review.confirmation_digest
            },
        };
        let mut changed = c.clone();
        if case == "context" {
            changed.opencodex_home = c.opencodex_home.with_file_name("other");
        }
        if case == "expired" {
            reviews.0.lock().unwrap().as_mut().unwrap().selected_at =
                Instant::now() - Duration::from_secs(601);
        }
        if matches!(case, "digest" | "context" | "expired") {
            assert!(consume(&reviews, &request, &changed).is_err());
        } else {
            let ticket = consume(&reviews, &request, c).unwrap();
            if case == "terminal" {
                fs::write(c.user_profile.join(".zshrc"), b"new unrelated user content").unwrap();
            }
            assert!(execute(ticket, || if case == "service" {
                Err(AppError::NotConfigured)
            } else {
                Ok(())
            })
            .is_err());
        }
        assert!(c.program_root.join("runtime/current.json").exists());
        assert!(!c
            .program_root
            .join("runtime/.removal-pending.json")
            .exists());
    }
}
#[test]
fn ipc_never_accepts_arbitrary_program_paths_or_invented_removal_inventory() {
    assert!(serde_json::from_value::<Request>(
        serde_json::json!({"id":"review","confirmationDigest":"hash","programRoot":"/arbitrary"})
    )
    .is_err());
    let f = fixture();
    let c = &f.c;
    fs::create_dir(c.program_root.join("runtime/versions/unknown-version")).unwrap();
    let reviews = Reviews::default();
    assert!(prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews).is_err());
    assert!(reviews.0.lock().unwrap().is_none());
}

#[test]
fn restarted_observer_continues_original_partial_and_marker_phase_removal_with_original_terminal_scope(
) {
    for phase in ["metadata", "file", "marker"] {
        let f = fixture();
        let c = &f.c;
        f.change(terminal_cli::Action::Enable);
        let original = program_removal::prepare_registered(
            &c.program_root,
            &c.desktop_root,
            &c.opencodex_home,
        )
        .unwrap();
        let preview = terminal_cli::removal_preview(c, "/bin/zsh", &[])
            .unwrap()
            .unwrap();
        assert!(original
            .interrupt_fixture(phase, || terminal_cli::revoke_for_removal(
                c,
                "/bin/zsh",
                &[],
                Some(&preview.confirmation_digest)
            ))
            .is_err());
        let config = fs::read(c.opencodex_home.join("config.json")).unwrap();
        let reviews = Reviews::default();
        let review = prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews)
            .unwrap()
            .unwrap();
        assert!(review.resume);
        assert_eq!(
            review.removal.operation_id,
            original.preview().unwrap().operation_id
        );
        assert!(review.command_path.is_none());
        let request = Request {
            id: review.id,
            confirmation_digest: review.confirmation_digest,
        };
        let ticket = consume(&reviews, &request, c).unwrap();
        let result = execute(ticket, || Ok(())).unwrap();
        assert_eq!(result.operation_id, review.removal.operation_id);
        assert_eq!(
            fs::read(c.opencodex_home.join("config.json")).unwrap(),
            config
        );
        assert!(program_removal::intentionally_removed(&c.program_root, &c.desktop_root).unwrap());
    }
}

#[test]
#[ignore = "requires the explicit pinned official2.99 complete bundle; never starts a service"]
fn real_official_image_removal_uses_registered_files_preserves_config_and_revokes_terminal() {
    let archive = std::path::PathBuf::from(
        std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit bundle"),
    );
    let descriptor = std::path::PathBuf::from(
        std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
    );
    let pin = std::env::var("OCXD_TEST_BUNDLE_PIN").expect("trusted pin");
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let f = Fixture::new();
    let c = &f.c;
    // Replace only this helper's newly created synthetic runtime with the complete
    // pinned source. No real installation or directory discovery participates.
    fs::remove_dir_all(c.program_root.join("runtime")).unwrap();
    crate::modules::runtime::bundled::versioned::provision(
        &c.program_root,
        &c.desktop_root,
        &archive,
        &fs::read(descriptor).unwrap(),
        &pin,
    )
    .unwrap();
    let config=br#"{ "port":41234, "providers":{"sample":{"apiKey":"isolated-synthetic-only"}},"keep":true }"#;
    fs::write(c.opencodex_home.join("config.json"), config).unwrap();
    f.change(terminal_cli::Action::Enable);
    let reviews = Reviews::default();
    let started = Instant::now();
    let review = prepare(c.clone(), "/bin/zsh".into(), vec![], &reviews)
        .unwrap()
        .unwrap();
    assert_eq!(review.removal.versions, vec!["2.99.0"]);
    assert!(review.removal.files > 10000);
    assert!(review.removal.bytes > 500_000_000);
    let request = Request {
        id: review.id,
        confirmation_digest: review.confirmation_digest,
    };
    let ticket = consume(&reviews, &request, c).unwrap();
    let result = execute(ticket, || Ok(())).unwrap();
    assert!(result.configuration_preserved);
    assert_eq!(result.removed_versions, vec!["2.99.0"]);
    assert_eq!(
        fs::read(c.opencodex_home.join("config.json")).unwrap(),
        config
    );
    assert!(!c.user_profile.join(".local/bin/ocx").exists());
    assert!(!c.user_profile.join(".zshrc").exists());
    assert!(!c.program_root.join("runtime/bin/ocx").exists());
    assert!(!c.program_root.join("runtime/bin/ocx-runtime").exists());
    assert_eq!(
        fs::read_dir(c.program_root.join("runtime/versions"))
            .unwrap()
            .count(),
        0
    );
    let backups = crate::modules::backup::safety::scan(&c.desktop_root.join("backups")).unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        fs::read(backups[0].directory.join("config.json")).unwrap(),
        config
    );
    assert!(program_removal::intentionally_removed(&c.program_root, &c.desktop_root).unwrap());
    println!(
        "REAL_OFFICIAL_REMOVAL {}",
        serde_json::json!({"version":"2.99.0","removedFiles":review.removal.files,"removedBytes":review.removal.bytes,"elapsedMs":started.elapsed().as_millis(),"configurationPreserved":true,"terminalRevoked":true,"serviceProof":"injected isolated fixture","serviceStarted":false})
    );
}
