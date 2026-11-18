//! Make only a legitimate initial Preparing state for an isolated real-GUI
//! cancellation test. No canonical JSON, source/owner proof or receipt is
//! fabricated. The app must perform cancellation through its normal command.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit fresh task-owned GUI profile with an already supplied official runtime"]
fn prepare_exact_owned_first_root_for_real_gui() {
    use opencodex_desktop_lib::{
        infrastructure::macos_client_originals::session_index::{self, Index, State},
        modules::runtime::{self, RuntimeHandle},
        state::ProcessContext,
    };
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, PermissionsExt},
        path::PathBuf,
        time::{Duration, Instant},
    };
    let root =
        PathBuf::from(std::env::var_os("OCXD024_GUI_ROOT_CANCEL_SEED").expect("explicit own root"));
    assert!(
        root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-gui-root-cancel-")
    );
    let profile = root.join("profile");
    let data = profile.join(".codex-desktop/desktop");
    let home = profile.join(".codex-desktop/opencodex/home");
    let program = profile.join("Library/Application Support/OpenCodeX Desktop");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    assert!(!profile.join(".codex").exists());
    assert!(!data
        .join("manager-state/client-session-index.sqlite")
        .exists());
    assert!(!data.join("manager-state/client-originals").exists());
    assert!(!home.join("runtime-port.json").exists() && !home.join("ocx.pid").exists());
    let (target, platform) = runtime::program_layout::host_target().unwrap();
    let selected = runtime::program_selection::read_complete(&program, platform, target)
        .unwrap()
        .unwrap();
    assert_eq!(selected.selection.version, "2.99.0");
    assert_eq!(
        selected.image.official_archive_sha256,
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    let context = ProcessContext {
        runtime: RuntimeHandle::initialize_versioned(&data, &program),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let (index, head) =
        Index::prepare_start_capture(&context, &data, Instant::now() + Duration::from_secs(30))
            .unwrap();
    assert_eq!(head.state, State::Preparing);
    assert_eq!(head.revision, 1);
    assert!(head.references.is_none() && head.completion_sha256.is_none());
    let partial = std::env::var_os("OCXD024_GUI_ROOT_PARTIAL_SEED").as_deref()
        == Some(std::ffi::OsStr::new("1"));
    let head = if partial {
        // The actual product reservation establishes the one slot's owner.
        // Bytes below are explicit synthetic interrupted-content DATA only;
        // no Original frame, source permission or completion is fabricated.
        let reserved = index.reserve_first_root_capture(&head).unwrap();
        let vault = data.join("manager-state/client-originals");
        fs::DirBuilder::new().mode(0o700).create(&vault).unwrap();
        let slot = vault.join(&reserved.ticket);
        fs::DirBuilder::new().mode(0o700).create(&slot).unwrap();
        let fragment = slot.join("synthetic-unpublished-fragment.json");
        let bytes = b"{\"fixtureOnly\":true,\"notOriginalSession\":true,\"payload\":\"preserve partial bytes\"}\n";
        fs::write(&fragment, bytes).unwrap();
        fs::set_permissions(&fragment, fs::Permissions::from_mode(0o400)).unwrap();
        reserved
    } else {
        head
    };
    drop(index);
    let preview = session_index::preview::inspect_preview(&context, &data);
    assert!(preview.can_cancel_preparation);
    assert_eq!(
        preview.preparation_kind,
        Some(if partial {
            "created_root_capture"
        } else {
            "created_root"
        })
    );
    runtime::program_selection::verify_current(&program, &selected).unwrap();
    runtime::program_image::verify(&selected).unwrap();
    let result = serde_json::json!({"schema":"owned-gui-first-root-preparation/1","head":head,
        "preview":preview,"officialArchiveSha256":selected.image.official_archive_sha256,
        "programImageSha256":selected.selection.image_sha256,"serviceStarted":false,
        "canonicalRecordsWrittenOnlyByProductIndex":true,"partialCaptureSyntheticSeed":partial,
        "syntheticFragmentIsNotOfficialCaptureExecution":true,"releaseAcceptance":false});
    std::fs::write(
        root.join("seed-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("GUI_ROOT_PREPARATION_CREATED {}", root.display());
}
