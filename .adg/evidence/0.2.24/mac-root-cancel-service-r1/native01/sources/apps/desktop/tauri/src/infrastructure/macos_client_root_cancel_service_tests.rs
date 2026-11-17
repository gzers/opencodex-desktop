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
    let started = Instant::now();
    let end = started + Duration::from_secs(10);
    let (index, head) = Index::prepare_start_capture(&context, &data, end).unwrap();
    let review = index
        .review_first_root_preparation(&context, &head, end)
        .unwrap();
    let case = std::env::var("OCXD024_ROOT_CANCEL_CASE").unwrap_or_else(|_| "success".into());
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
    assert!(!index.successor_eligible(&context, &closed).unwrap());
    drop(proof);
    drop(index);
    let reopened = Index::open_existing(&context, &data, &profile.join(".codex")).unwrap();
    assert!(reopened.head().unwrap().as_ref() == Some(&closed));
    assert!(reopened.reserve_start().is_err());
    drop(reopened);
    let preview = super::super::preview::inspect_preview(&context, &data);
    assert_eq!(preview.phase, Some(State::RootCancellationAbsent));
    assert_eq!(preview.status, super::super::preview::Status::Pending);
    assert_eq!(
        preview.completion_evidence,
        super::super::preview::Evidence::Verified
    );
    fs::write(root.join("native-preservation-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"scenario":case,"actualCurrentServiceAdmission":true,"originalFourNativeScopesReleased":true,"canonicalHead":closed,"canonicalCancellationClosed":true,"normalStop":false,"successorEligible":false,"elapsedActionMs":started.elapsed().as_millis(),"budgetMs":10000,"serviceStarted":false,"releaseAcceptance":false})).unwrap()).unwrap();
}
