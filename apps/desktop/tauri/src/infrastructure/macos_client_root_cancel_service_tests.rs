use super::*;
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    process::{Command, Stdio},
    time::Duration,
};

#[test]
#[ignore = "explicit fresh guarded official2.99 fixture; actual current service admission, no service Start"]
fn current_service_movement_and_native_release_bind_a_distinct_cancellation_terminal() {
    use crate::infrastructure::official_client_coordination::{
        native::with_native_relocation_release, Context,
    };
    let root =
        PathBuf::from(std::env::var_os("OCXD024_ROOT_CANCEL_NATIVE").expect("explicit task root"));
    assert!(
        root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-root-cancel-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join("desktop");
    let home = profile.join("home");
    for p in [&data, &home] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    fs::write(home.join("config.json"), b"{\"providers\":{},\"port\":0}").unwrap();
    fs::set_permissions(home.join("config.json"), fs::Permissions::from_mode(0o600)).unwrap();
    let resources = root.join("resources");
    let descriptor = fs::read(resources.join("runtime-bundle.manifest.json")).unwrap();
    let pin = crate::infrastructure::hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let program = std::env::var_os("OCXD024_ROOT_CANCEL_REUSE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("program"));
    if program != root.join("program") {
        assert!(
            program.starts_with("/private/tmp")
                && program
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-root-cancel-shared-")
        );
    }
    if !program.exists() {
        crate::modules::runtime::bundled::versioned::provision(
            &program,
            &data,
            &resources.join("runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
            &descriptor,
            &pin,
        )
        .unwrap();
    }
    let selected = crate::modules::runtime::program_selection::read_complete(
        &program,
        crate::modules::data_root::layout::Platform::Macos,
        "darwin-arm64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        selected.image.official_archive_sha256,
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    let context = ProcessContext {
        runtime: crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &program),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let partial_point = std::env::var("OCXD024_ROOT_CANCEL_PARTIAL_POINT").ok();
    let started = Instant::now();
    let start_end = started + Duration::from_secs(if partial_point.is_some() { 30 } else { 10 });
    let (index, head) = Index::prepare_start_capture(&context, &data, start_end).unwrap();
    let head = if let Some(point) = &partial_point {
        assert!([
            "directory_created",
            "codex_archived",
            "grok_archived",
            "original_published"
        ]
        .contains(&point.as_str()));
        let grok = profile.join(".grok");
        fs::DirBuilder::new().mode(0o700).create(&grok).unwrap();
        fs::write(
            grok.join("config.toml"),
            b"model = \"synthetic-partial-grok\"\n",
        )
        .unwrap();
        fs::set_permissions(grok.join("config.toml"), fs::Permissions::from_mode(0o600)).unwrap();
        let reserved = index.reserve_first_root_capture(&head).unwrap();
        let reservation = index.capture_reservation(&reserved).unwrap();
        let coordinator = Context {
            runtime: context.runtime.clone(),
            user_profile: profile.clone(),
            codex_home: profile.join(".codex"),
            opencodex_home: home.clone(),
            control_root: index.control_root().unwrap(),
        };
        let captured =
            crate::infrastructure::official_client_coordination::native::with_native_coordination(
                &coordinator,
                start_end,
                |native| {
                    crate::infrastructure::macos_client_originals::session::StartupSession::begin_reserved(&context,&data,&profile.join(".codex"),native,start_end,&reservation)
                },
            );
        assert!(
            captured.is_err(),
            "real capture must interrupt at requested point"
        );
        assert_eq!(
            crate::infrastructure::macos_client_originals::fixture_take_root_capture_point()
                .as_deref(),
            Some(point.as_str()),
            "an unrelated early error cannot satisfy the capture counterexample"
        );
        assert!(Instant::now() < start_end);
        assert!(reserved.references.is_none());
        reserved
    } else {
        head
    };
    let partial_before = partial_point.as_ref().map(|_| {
        super::super::super::TreeSnapshot::read_mode(
            &PinnedDirectory::open(
                &data
                    .join("manager-state/client-originals")
                    .join(&head.ticket),
            )
            .unwrap(),
            false,
        )
        .unwrap()
    });
    let started = if partial_point.is_some() {
        Instant::now()
    } else {
        started
    };
    let end = if partial_point.is_some() {
        started + Duration::from_secs(10)
    } else {
        start_end
    };
    let review = index
        .review_first_root_preparation(&context, &head, end)
        .unwrap();
    let case = std::env::var("OCXD024_ROOT_CANCEL_CASE").unwrap_or_else(|_| "success".into());
    if std::env::var_os("OCXD024_ROOT_CANCEL_USER_ACTION").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        assert_eq!(case, "success");
        drop(review);
        drop(index);
        let preview = super::super::preview::inspect_preview(&context, &data);
        assert!(preview.can_cancel_preparation);
        assert_eq!(
            preview.preparation_kind,
            Some(if head.state == State::CaptureReserved {
                "created_root_capture"
            } else {
                "created_root"
            })
        );
        let request = super::super::finalization::Request {
            attempt_id: head.ticket.clone(),
            revision: head.revision,
            confirmation_digest: preview.preparation_digest.unwrap(),
        };
        let stale = super::super::finalization::Request {
            attempt_id: request.attempt_id.clone(),
            revision: request.revision,
            confirmation_digest: "0".repeat(64),
        };
        assert!(super::super::root_cancellation::cancel(&context, &data, stale).is_err());
        let action_started = Instant::now();
        let outcome = super::super::root_cancellation::cancel(&context, &data, request).unwrap();
        let action_elapsed = action_started.elapsed().as_millis();
        assert!(action_elapsed < 10000);
        assert_eq!(outcome.status, "root_cancelled");
        assert_eq!(outcome.revision, head.revision + 2);
        let reopened = Index::open_existing(&context, &data, &profile.join(".codex")).unwrap();
        let closed = reopened.head().unwrap().unwrap();
        drop(reopened);
        successor_and_history(
            &root,
            &context,
            &data,
            closed,
            action_elapsed,
            &case,
            partial_before.as_ref(),
        );
        let result_path = root.join("native-preservation-result.json");
        let mut result: serde_json::Value =
            serde_json::from_slice(&fs::read(&result_path).unwrap()).unwrap();
        result["actualUserActionEntry"] = serde_json::json!(true);
        result["staleConfirmationRefused"] = serde_json::json!(true);
        fs::write(result_path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        return;
    }
    if case == "new_data" {
        // Real extra same-HOME runtime. It is not a Guard and must never be
        // excluded by name/path alone. Only this newly spawned fixture is killed.
        let mut extra = Command::new(&selected.paths.bun)
            .args([
                "--eval",
                "console.log('fixture-ready');setInterval(()=>{},1000)",
            ])
            .env_clear()
            .env("HOME", &profile)
            .env("OPENCODEX_HOME", &home)
            .env("PATH", "/usr/bin:/bin")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ready = String::new();
        BufReader::new(extra.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready.trim(), "fixture-ready");
        let refused = service::CurrentService::begin(&review).is_err();
        extra.kill().unwrap();
        let status = extra.wait().unwrap();
        assert!(refused);
        assert!(index.head().unwrap().as_ref() == Some(&head));
        assert!(super::super::super::tree::names(index.client.file())
            .unwrap()
            .is_empty());
        fs::write(root.join("native-preservation-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"scenario":case,"serviceAdmissionRefused":true,"extraRuntimeExit":status.to_string(),"extraTaskFixtureForcedDisposal":true,"originalHeadUnchanged":true,"rootUnmoved":true,"canonicalCancellationClosed":false,"serviceStarted":false,"releaseAcceptance":false})).unwrap()).unwrap();
        return;
    }
    let service = service::CurrentService::begin(&review).unwrap();
    let native_context = Context {
        runtime: context.runtime.clone(),
        user_profile: profile.clone(),
        codex_home: profile.join(".codex"),
        opencodex_home: home,
        control_root: index.control_root().unwrap(),
    };
    let checkpoint_reached = std::cell::Cell::new(false);
    let native_result = with_native_relocation_release(&native_context, end, |native| {
        let mut guard = native.config_write_guard()?;
        let pending = Prepared::under_native_guard(review, native, &mut guard)
            .and_then(Prepared::register)
            .map_err(|e| crate::errors::AppError::RuntimeManaged {
                code: "test_current_root_cancel_plan".into(),
                detail: e.to_string(),
            })?;
        let authority = pending
            .authorize_current(&service, native, &mut guard)
            .map_err(|e| crate::errors::AppError::RuntimeManaged {
                code: "test_current_root_cancel_admission".into(),
                detail: e.to_string(),
            })?;
        let proof = pending
            .preserve_checked(&authority, native, guard, &|point| {
                if case == "before_move" && point == "before_rename"
                    || case == "after_move" && point == "after_reanchor"
                {
                    checkpoint_reached.set(true);
                    return Err(denied("task-owned current cancellation interruption"));
                }
                if case == "new_root" && point == "after_reanchor" {
                    fs::DirBuilder::new()
                        .mode(0o700)
                        .create(profile.join(".codex"))?;
                    fs::write(profile.join(".codex/external"), b"keep replacement root")?;
                    checkpoint_reached.set(true);
                }
                Ok(())
            })
            .map_err(|e| crate::errors::AppError::RuntimeManaged {
                code: "test_current_root_cancel_move".into(),
                detail: e.to_string(),
            })?;
        Ok(proof)
    });
    if case != "success" {
        assert!(
            checkpoint_reached.get(),
            "current production admission did not reach its real checkpoint: {}",
            native_result
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default()
        );
        assert!(native_result.is_err());
        let pending = read_head(index.connection().unwrap()).unwrap().unwrap();
        assert_eq!(pending.state, State::RootCancellationPrepared);
        assert!(pending.completion_sha256.is_none());
        if case == "new_root" {
            assert_eq!(
                fs::read(profile.join(".codex/external")).unwrap(),
                b"keep replacement root"
            );
        }
        fs::write(root.join("native-preservation-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"scenario":case,"actualCurrentServiceAdmission":true,"expectedRefusal":true,"canonicalHead":pending,"canonicalCancellationClosed":false,"normalStop":false,"serviceStarted":false,"releaseAcceptance":false})).unwrap()).unwrap();
        return;
    }
    let (preserved, released) = native_result.unwrap();
    let proof = completion::verify_released(&index, &service, preserved, released).unwrap();
    let pending = index.head().unwrap().unwrap();
    let closed = index
        .close_first_root_cancellation(&pending, &proof)
        .unwrap();
    assert_eq!(closed.state, State::RootCancellationAbsent);
    assert!(closed.references.is_none());
    assert!(closed.completion_sha256.is_some());
    assert!(Instant::now() < end);
    assert!(!profile.join(".codex").exists());
    assert!(index.successor_eligible(&context, &closed).unwrap());
    drop(proof);
    drop(index);
    successor_and_history(
        &root,
        &context,
        &data,
        closed,
        started.elapsed().as_millis(),
        &case,
        partial_before.as_ref(),
    );
}
fn successor_and_history(
    root: &Path,
    context: &ProcessContext,
    data: &Path,
    closed: Head,
    cancellation_elapsed: u128,
    case: &str,
    partial_before: Option<&super::super::super::TreeSnapshot>,
) {
    let profile = context.working_directory.clone();
    let partial_point = std::env::var("OCXD024_ROOT_CANCEL_PARTIAL_POINT").ok();
    std::env::remove_var("OCXD024_ROOT_CANCEL_PARTIAL_POINT");
    let partial_path = data
        .join("manager-state/client-originals")
        .join(&closed.ticket);
    if let Some(before) = partial_before {
        assert_eq!(
            &super::super::super::TreeSnapshot::read_mode(
                &PinnedDirectory::open(&partial_path).unwrap(),
                false
            )
            .unwrap(),
            before
        );
    }
    let reopened = Index::open_existing(context, data, &profile.join(".codex")).unwrap();
    assert!(reopened.head().unwrap().as_ref() == Some(&closed));
    assert!(reopened.reserve_start().is_err());
    drop(reopened);
    let preview = super::super::preview::inspect_preview(context, data);
    assert_eq!(preview.phase, Some(State::RootCancellationAbsent));
    assert_eq!(preview.status, super::super::preview::Status::Restored);
    assert_eq!(
        preview.completion_evidence,
        super::super::preview::Evidence::Verified
    );
    // New explicit Start action has its own original 30s window. No service is
    // launched: this verifies actual new root custody and storage-only capture.
    let next_started = Instant::now();
    let (successor, preparing) =
        Index::prepare_start_capture(context, data, next_started + Duration::from_secs(30))
            .unwrap();
    assert_ne!(preparing.ticket, closed.ticket);
    assert_eq!(preparing.revision, closed.revision + 1);
    assert!(preparing.root_generation.is_some());
    if std::env::var_os("OCXD024_SUCCESSOR_CANCEL_REVIEW_ONLY").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        // This is a fresh read-only cancellation review, not a resumed Start
        // window or a grant to move the successor root. The predecessor was
        // just closed through the actual native CurrentService/Guard path.
        let review_started = Instant::now();
        let review_end = review_started + Duration::from_secs(10);
        let review = successor
            .review_successor_root_preparation(context, &preparing, review_end)
            .unwrap();
        let fingerprint = review.fingerprint().to_owned();
        review.recheck().unwrap();
        let review_ms = review_started.elapsed().as_millis();
        assert!(review_ms < 10000);
        assert_eq!(successor.head().unwrap(), Some(preparing.clone()));
        assert!(successor
            .review_first_root_preparation(context, &preparing, review_end)
            .is_err());
        drop(review);
        drop(successor);
        let preview_started = Instant::now();
        let preview = super::super::preview::inspect_preview(context, data);
        let preview_ms = preview_started.elapsed().as_millis();
        assert!(preview_ms < 5000);
        assert_eq!(preview.status, super::super::preview::Status::Pending);
        assert_eq!(preview.phase, Some(State::Preparing));
        assert_eq!(preview.preparation_kind, Some("successor_root"));
        assert_eq!(
            preview.preparation_digest.as_deref(),
            Some(fingerprint.as_str())
        );
        assert!(!preview.can_cancel_preparation);
        let reopened = Index::open_existing(context, data, &profile.join(".codex")).unwrap();
        reopened
            .review_successor_root_preparation(context, &preparing, review_end)
            .unwrap()
            .recheck()
            .unwrap();
        let old_slot = data
            .join("manager-state/client-root-cancel-plans")
            .join(&closed.root_retirement.as_ref().unwrap().plan.id);
        fs::write(
            old_slot.join("root/unexpected"),
            b"explicit cancelled predecessor drift fixture",
        )
        .unwrap();
        assert!(reopened
            .review_successor_root_preparation(context, &preparing, review_end)
            .is_err());
        assert_eq!(
            fs::read(old_slot.join("root/unexpected")).unwrap(),
            b"explicit cancelled predecessor drift fixture"
        );
        fs::write(
            root.join("native-preservation-result.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "scenario":"successor-review-only", "actualCurrentServiceAdmission":true,
                "originalFourNativeScopesReleased":true, "canonicalCancellationClosed":true,
                "predecessor":closed, "successorPreparing":preparing,
                "reviewElapsedMs":review_ms,"reviewBudgetMs":10000,"fingerprint":fingerprint,
                "previewElapsedMs":preview_ms,"previewBudgetMs":5000,"previewFingerprintMatches":true,
                "factsOnlyReopenReviewed":true,"oldCancelledArchiveDriftRefused":true,
                "successorCaptureOrMovementStarted":false,"serviceStarted":false,
                "elapsedActionMs":cancellation_elapsed,"budgetMs":10000,"releaseAcceptance":false
            }))
            .unwrap(),
        )
        .unwrap();
        return;
    }
    let (captured, originals) =
        super::super::generation::fixture_captured_successor(context, &successor, &preparing);
    assert_eq!(captured.state, State::Captured);
    let successor_elapsed = next_started.elapsed().as_millis();
    assert!(successor_elapsed < 30000);
    drop(originals);
    drop(successor);
    let reopened = Index::open_existing(context, data, &profile.join(".codex")).unwrap();
    assert!(reopened.head().unwrap().as_ref() == Some(&captured));
    drop(reopened);
    let old_slot = data
        .join("manager-state/client-root-cancel-plans")
        .join(&closed.root_retirement.as_ref().unwrap().plan.id);
    // A historical old-root edit is not hidden by the later active generation.
    if let Some(before) = partial_before {
        assert_eq!(
            &super::super::super::TreeSnapshot::read_mode(
                &PinnedDirectory::open(&partial_path).unwrap(),
                false
            )
            .unwrap(),
            before
        );
    }
    fs::write(
        if partial_before.is_some() {
            partial_path.join("unexpected")
        } else {
            old_slot.join("root/unexpected")
        },
        b"preserve historical corruption fixture",
    )
    .unwrap();
    assert!(Index::open_existing(context, data, &profile.join(".codex")).is_err());
    fs::write(root.join("native-preservation-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"scenario":case,"actualCurrentServiceAdmission":true,"originalFourNativeScopesReleased":true,"canonicalHead":closed,"canonicalCancellationClosed":true,"normalStop":false,"successorEligible":true,"elapsedActionMs":cancellation_elapsed,"budgetMs":10000,"successorCaptureElapsedMs":successor_elapsed,"successorCaptureBudgetMs":30000,"successorCapturedHead":captured,"syntheticCaptureScopeOnlyNotService":true,"oldArchiveTamperReopenRefused":true,"partialCaptureRetainedWithoutAdoption":partial_before.is_some(),"partialCapturePoint":partial_point,"serviceStarted":false,"releaseAcceptance":false})).unwrap()).unwrap();
}
