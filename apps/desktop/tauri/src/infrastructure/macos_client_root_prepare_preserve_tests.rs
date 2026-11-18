use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};

#[test]
fn preservation_authority_cannot_be_deserialized_or_created_by_plan_or_readback() {
    // The only constructor is cfg(test); production review and plan bytes cannot
    // create a MovementAuthority. Do not add a public callback-based factory.
    let raw = b"{\"schemaVersion\":1,\"id\":\"not-an-authority\"}";
    assert!(decode::<Plan>(raw).is_err());
    assert!(decode::<Plan>(b"{\"canonicalCancellationClosed\":true}").is_err());
}

#[test]
fn cancellation_registration_rejects_missing_or_unbound_plan_and_rolls_back_the_event() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let data = root.join("desktop");
    let profile = root.join("profile");
    let home = profile.join("home");
    for path in [&data, &profile, &home] {
        fs::DirBuilder::new().mode(0o700).create(path).unwrap();
    }
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: profile.clone(),
        opencodex_home: home,
    };
    let (index, head) =
        Index::prepare_start_capture(&context, &data, Instant::now() + Duration::from_secs(10))
            .unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let candidate = Head {
        revision: head.revision + 1,
        ticket: head.ticket.clone(),
        state: State::RootCancellationPrepared,
        references: None,
        restore_plan: None,
        completion_sha256: None,
        root_retirement: Some(RootRetirement {
            source_revision: head.revision,
            plan: RestorePlan {
                id: id.clone(),
                sha256: digest(b"not-a-registered-plan"),
            },
        }),
        root_generation: None,
    };
    for stage in 0..3 {
        if stage == 1 {
            let parent = index
                .action_parent
                .child_private("client-root-cancel-plans", true)
                .unwrap();
            let slot = parent.child_private(&id, true).unwrap();
            create(&slot, "plan.json", b"not-a-registered-plan").unwrap();
        }
        let mut wrong = candidate.clone();
        if stage == 2 {
            wrong.completion_sha256 = Some(digest(b"sidecar claims success"));
        }
        assert!(index.write(Some(&head), &wrong).is_err());
        assert_eq!(index.head().unwrap().as_ref(), Some(&head));
        assert!(
            vault_history::read(index.connection().unwrap(), &head).unwrap() == vec![head.clone()]
        );
        assert!(super::super::super::tree::names(index.client.file())
            .unwrap()
            .is_empty());
    }
    drop(index);
    drop(context);
    let path = temp.path().to_path_buf();
    temp.close().unwrap();
    assert!(!path.exists());
    println!("ROOT_CANCEL_INDEX_FIXTURE_REMOVED {}", path.display());
}

#[test]
#[ignore = "explicit isolated official2.99 Guard/NativeLease fixture, no service or canonical cancel"]
fn official_guard_preserves_exact_uncaptured_root_and_releases_only_its_marker() {
    use crate::infrastructure::official_client_coordination::{
        native::with_native_relocation_coordination, Context,
    };
    let root =
        PathBuf::from(std::env::var_os("OCXD024_ROOT_CANCEL_NATIVE").expect("explicit new root"));
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
    let data = profile.join("desktop");
    let home = profile.join("home");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
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
    let started = Instant::now();
    let end = started + Duration::from_secs(10);
    let (index, head) = Index::prepare_start_capture(&context, &data, end).unwrap();
    let original_root = index.client.identity().clone();
    let review = index
        .review_first_root_preparation(&context, &head, end)
        .unwrap();
    let native_context = Context {
        runtime: context.runtime.clone(),
        user_profile: profile.clone(),
        codex_home: profile.join(".codex"),
        opencodex_home: home,
        control_root: index.control_root().unwrap(),
    };
    let refusal_observed = std::cell::Cell::new(false);
    let result = with_native_relocation_coordination(&native_context, end, |native| {
        let mut guard = native.config_write_guard()?;
        let pending = Prepared::under_native_guard(review, native, &mut guard).map_err(|e| {
            crate::errors::AppError::RuntimeManaged {
                code: "test_root_prepare_preservation".into(),
                detail: e.to_string(),
            }
        })?;
        let pending = pending
            .register()
            .map_err(|e| crate::errors::AppError::RuntimeManaged {
                code: "test_root_prepare_registration".into(),
                detail: e.to_string(),
            })?;
        assert_eq!(
            pending.registered_head().unwrap().state,
            State::RootCancellationPrepared
        );
        assert_eq!(index.head().unwrap().as_ref(), pending.registered_head());
        let destination = pending.destination();
        let scenario =
            std::env::var("OCXD024_ROOT_CANCEL_CASE").unwrap_or_else(|_| "success".into());
        assert!([
            "success",
            "before_move",
            "after_move",
            "new_data",
            "new_root"
        ]
        .contains(&scenario.as_str()));
        if scenario != "success" {
            let failed =
                pending.preserve_checked(&pending.fixture_authority(), native, guard, &|point| {
                    if scenario == "before_move" && point == "before_rename"
                        || scenario == "after_move" && point == "after_reanchor"
                    {
                        return Err(denied("task-owned injected cancellation interruption"));
                    }
                    if scenario == "new_data" && point == "before_rename" {
                        fs::write(profile.join(".codex/external"), b"preserve new client data")?;
                    }
                    if scenario == "new_root" && point == "after_reanchor" {
                        fs::DirBuilder::new()
                            .mode(0o700)
                            .create(profile.join(".codex"))?;
                        fs::write(
                            profile.join(".codex/external"),
                            b"preserve replacement root data",
                        )?;
                    }
                    Ok(())
                });
            assert!(failed.is_err());
            assert!(!pending
                .slot
                .directory()
                .path()
                .join("preserved-readback.json")
                .exists());
            assert!(pending.slot.directory().path().join("plan.json").is_file());
            assert!(pending
                .slot
                .directory()
                .path()
                .join("move-intent.json")
                .is_file());
            let kept = if scenario == "before_move" {
                profile.join(".codex")
            } else {
                destination.clone()
            };
            assert_eq!(
                PinnedDirectory::open(&kept).unwrap().identity(),
                &original_root
            );
            if scenario == "new_data" {
                assert_eq!(
                    fs::read(kept.join("external")).unwrap(),
                    b"preserve new client data"
                );
            }
            if scenario == "new_root" {
                assert_eq!(
                    fs::read(profile.join(".codex/external")).unwrap(),
                    b"preserve replacement root data"
                );
            }
            fs::write(root.join("native-preservation-result.json"), serde_json::to_vec_pretty(&serde_json::json!({
                "scenario":scenario,"expectedRefusal":true,"originalRootIdentity":original_root,"keptRoot":kept,
                "noSuccessReceipt":true,"planAndIntentPreserved":true,"serviceStarted":false,"releaseAcceptance":false,
                "nativeFailureReleaseNotClaimed":true})).unwrap()).unwrap();
            refusal_observed.set(true);
            return Ok(());
        }
        let proof = pending
            .preserve_guarded(&pending.fixture_authority(), native, guard)
            .map_err(|e| crate::errors::AppError::RuntimeManaged {
                code: "test_root_prepare_preservation".into(),
                detail: e.to_string(),
            })?;
        assert_eq!(proof.root_identity(), &original_root);
        assert!(!profile.join(".codex").exists());
        assert!(!destination.join("config.toml.ocx-write.lock").exists());
        assert!(destination
            .join(".opencodex-native-main.claim.sqlite")
            .is_file());
        let receipt = serde_json::json!({"scope":"real official Guard plus explicit storage-only test authority; not product cancellation",
            "planId":proof.id(),"planSha256":proof.plan_sha256(),"readbackSha256":proof.receipt_sha256(),
            "rootIdentity":proof.root_identity(),"destination":destination,
            "logicalRootAbsent":true,"originalHead":head,"canonicalCancellationClosed":false,
            "serviceStarted":false,"releaseAcceptance":false});
        fs::write(
            root.join("native-preservation-result.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
        Ok(())
    });
    if let Err(e) = &result {
        fs::write(root.join("native-preservation-error.json"), serde_json::to_vec_pretty(&serde_json::json!({"error":e.to_string(),"elapsedMs":started.elapsed().as_millis(),"originalHead":head,"releaseAcceptance":false})).unwrap()).unwrap();
    }
    let scenario = std::env::var("OCXD024_ROOT_CANCEL_CASE").unwrap_or_else(|_| "success".into());
    if scenario == "success" {
        result.unwrap();
        assert!(Instant::now() < end);
    } else {
        assert!(
            refusal_observed.get(),
            "native counterexample was not reached"
        );
    }
    let indexed = read_head(index.connection().unwrap()).unwrap().unwrap();
    assert_eq!(indexed.state, State::RootCancellationPrepared);
    assert_eq!(indexed.revision, head.revision + 1);
    assert!(indexed.completion_sha256.is_none());
    // Registered plans resolve exact live and moved/interrupted objects, while
    // unknown new data or a replacement logical root remain refused. No close.
    if matches!(scenario.as_str(), "new_data" | "new_root") {
        assert!(index.head().is_err());
    } else {
        assert_eq!(index.head().unwrap().as_ref(), Some(&indexed));
    }
    drop(index);
    if !matches!(scenario.as_str(), "new_data" | "new_root") {
        let reopened = Index::open_existing(&context, &data, &profile.join(".codex")).unwrap();
        assert_eq!(reopened.head().unwrap().as_ref(), Some(&indexed));
        assert!(reopened.reserve_start().is_err());
        drop(reopened);
        let preview = super::super::preview::inspect_preview(&context, &data);
        assert_eq!(preview.status, super::super::preview::Status::Pending);
        assert_eq!(preview.phase, Some(State::RootCancellationPrepared));
        assert_eq!(
            preview.original_evidence,
            super::super::preview::Evidence::NotRecorded
        );
        assert_eq!(
            preview.plan_evidence,
            super::super::preview::Evidence::Verified
        );
        assert!(
            !preview.can_continue
                && !preview.can_rollback
                && !preview.can_finalize
                && !preview.can_rearm
                && !preview.can_cancel_preparation
        );
        assert!(!super::super::generation::inspect_successor_eligibility(&context, &data).unwrap());
    } else {
        assert!(Index::open_existing(&context, &data, &profile.join(".codex")).is_err());
    }
    let receipt_path = root.join("native-preservation-result.json");
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    receipt["registeredHead"] = serde_json::to_value(&indexed).unwrap();
    receipt["canonicalCancellationClosed"] = serde_json::json!(false);
    receipt["indexReopenedAndPending"] =
        serde_json::json!(!matches!(scenario.as_str(), "new_data" | "new_root"));
    receipt["newDataReopenRefused"] =
        serde_json::json!(matches!(scenario.as_str(), "new_data" | "new_root"));
    fs::write(receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
}
