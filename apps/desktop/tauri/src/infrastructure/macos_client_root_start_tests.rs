//! Product preparation and readonly status share verified successor facts.
//! These synthetic storage tests do not launch an official service.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            let metadata = fs::symlink_metadata(&p).unwrap();
            if metadata.is_dir() {
                walk(root, &p, out);
            } else if metadata.is_file() {
                out.insert(p.strip_prefix(root).unwrap().into(), fs::read(p).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, root, &mut result);
    result
}
#[test]
fn root_start_closed_status_and_product_preparation_use_same_verified_successor_source() {
    for count in [1, 3] {
        let (_temp, context, index, closed) = super::rollover_tests::fixture(count);
        let data = index.data_root().to_path_buf();
        assert!(index.successor_eligible(&context, &closed).unwrap());
        drop(index);
        use crate::modules::status::StatusSource;
        let runtime = crate::modules::runtime::RuntimeHandle::initialize_versioned(
            &data.parent().unwrap().join("status-fixture-data"),
            &data.parent().unwrap().join("status-fixture-install"),
        );
        let source = crate::infrastructure::status_source::OfficialStatusSource::new(
            runtime,
            context.working_directory.clone(),
            crate::modules::process::EnvironmentPolicy {
                opencodex_home: context.opencodex_home.clone(),
                ..Default::default()
            },
        )
        .with_client_recovery_root(data.clone());
        let before = bytes(&data);
        assert!(inspect_successor_eligibility(&context, &data).unwrap());
        assert_eq!(
            inspect_client_recovery(&context, &data).unwrap(),
            crate::modules::status::ClientRecoveryState::NotRequired
        );
        assert_eq!(
            source.client_recovery(),
            crate::modules::status::ClientRecoveryState::NotRequired
        );
        let preview = preview::inspect_preview(&context, &data);
        assert_eq!(preview.status, preview::Status::Restored);
        assert_eq!(preview.phase, Some(State::RootAbsent));
        assert_eq!(preview.completion_evidence, preview::Evidence::Verified);
        assert_eq!(
            bytes(&data),
            before,
            "readonly eligibility must not write or create new custody"
        );
        let (index, preparing) = Index::prepare_start_capture(&context, &data, deadline()).unwrap();
        assert_eq!(preparing.revision, closed.revision + 1);
        assert_ne!(preparing.ticket, closed.ticket);
        assert!(preparing.root_generation.is_some());
        assert!(index.has_original_root_absence());
        let (captured, originals) = super::capture_tests::captured(&context, &index, &preparing);
        assert_eq!(captured.state, State::Captured);
        assert!(originals
            .original_root_absence
            .as_ref()
            .is_some_and(|r| r.id == preparing.ticket));
        assert!(index.reserve_start().is_err());
        drop(originals);
        drop(index);
        assert!(!inspect_successor_eligibility(&context, &data).unwrap());
        assert_eq!(
            inspect_client_recovery(&context, &data).unwrap(),
            crate::modules::status::ClientRecoveryState::Required
        );
        assert_eq!(
            source.client_recovery(),
            crate::modules::status::ClientRecoveryState::Required
        );
        let captured_bytes = bytes(&data);
        assert!(Index::prepare_start_capture(&context, &data, deadline()).is_err());
        assert_eq!(bytes(&data), captured_bytes);
        println!(
            "ROOT_START_PREPARATION_EVIDENCE {}",
            serde_json::json!({"accepted":false,"closedGenerations":count,"closed":closed,"preparing":preparing,"captured":captured,"readonlyBytesUnchanged":true,"pendingRestartRefused":true,"noOfficialService":true})
        );
    }
}
#[test]
fn root_start_first_initialization_and_ordinary_existing_root_remain_distinct() {
    for existing in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let profile = root.join("profile");
        let home = profile.join("home");
        for p in [&data, &profile, &home] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        if existing {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(profile.join(".codex"))
                .unwrap();
        }
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile,
            opencodex_home: home,
        };
        assert!(!inspect_successor_eligibility(&context, &data).unwrap());
        let (index, preparing) = Index::prepare_start_capture(&context, &data, deadline()).unwrap();
        assert_eq!(preparing.revision, 1);
        assert!(preparing.root_generation.is_none());
        assert_eq!(index.has_original_root_absence(), !existing);
        let reserved = if existing {
            assert!(index.reserve_first_root_capture(&preparing).is_err());
            index.reserve_capture(&preparing).unwrap()
        } else {
            assert!(index.reserve_capture(&preparing).is_err());
            index.reserve_first_root_capture(&preparing).unwrap()
        };
        assert_eq!(reserved.state, State::CaptureReserved);
        assert!(index.reserve_successor_root_capture(&preparing).is_err());
    }
}
#[test]
fn root_start_invalid_scene_is_refused_by_both_readonly_qualification_and_product_preparation() {
    for case in [
        "user-root",
        "user-link",
        "unknown-slot",
        "old-original",
        "wrong-home",
    ] {
        let (_temp, mut context, index, _closed) = super::rollover_tests::fixture(2);
        let data = index.data_root().to_path_buf();
        drop(index);
        match case {
            "user-root" => {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(context.working_directory.join(".codex"))
                    .unwrap();
                fs::write(context.working_directory.join(".codex/keep"), b"user").unwrap();
            }
            "user-link" => std::os::unix::fs::symlink(
                &context.opencodex_home,
                context.working_directory.join(".codex"),
            )
            .unwrap(),
            "unknown-slot" => fs::DirBuilder::new()
                .mode(0o700)
                .create(
                    data.join("manager-state/client-root-generations")
                        .join(uuid::Uuid::new_v4().to_string()),
                )
                .unwrap(),
            "old-original" => {
                let vault = data.join("manager-state/client-originals");
                let attempt = fs::read_dir(vault).unwrap().next().unwrap().unwrap().path();
                let p = attempt.join("original.json");
                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(p, b"changed original").unwrap();
            }
            "wrong-home" => {
                let home = context.working_directory.join("other-home");
                fs::DirBuilder::new().mode(0o700).create(&home).unwrap();
                context.opencodex_home = home;
            }
            _ => unreachable!(),
        }
        let before = bytes(&data);
        assert!(
            inspect_successor_eligibility(&context, &data).is_err(),
            "{case}"
        );
        assert!(inspect_client_recovery(&context, &data).is_err(), "{case}");
        assert!(
            Index::prepare_start_capture(&context, &data, deadline()).is_err(),
            "{case}"
        );
        assert_eq!(bytes(&data), before, "{case}");
        if case == "user-root" {
            assert_eq!(
                fs::read(context.working_directory.join(".codex/keep")).unwrap(),
                b"user"
            );
        }
    }
}
#[test]
fn root_start_preparing_reopen_has_no_qualification_or_renewed_creation_custody() {
    let (_temp, context, index, _closed) = super::rollover_tests::fixture(1);
    let data = index.data_root().to_path_buf();
    drop(index);
    let (index, preparing) = Index::prepare_start_capture(&context, &data, deadline()).unwrap();
    drop(index);
    assert!(!inspect_successor_eligibility(&context, &data).unwrap());
    assert_eq!(
        inspect_client_recovery(&context, &data).unwrap(),
        crate::modules::status::ClientRecoveryState::Required
    );
    let index =
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).unwrap();
    assert_eq!(index.head().unwrap(), Some(preparing.clone()));
    assert!(index.reserve_successor_root_capture(&preparing).is_err());
    drop(index);
    let before = bytes(&data);
    assert!(Index::prepare_start_capture(&context, &data, deadline()).is_err());
    assert_eq!(bytes(&data), before);
}
#[test]
fn root_start_expired_entry_refuses_without_new_root_namespace_or_event() {
    let (_temp, context, index, _closed) = super::rollover_tests::fixture(1);
    let data = index.data_root().to_path_buf();
    drop(index);
    let before = bytes(&data);
    assert!(Index::prepare_start_capture(&context, &data, Instant::now()).is_err());
    assert_eq!(bytes(&data), before);
    assert!(!context.working_directory.join(".codex").exists());
}
#[test]
fn root_start_observation_callback_cannot_hide_new_root_or_changed_archive_after_eligibility() {
    for case in ["new-root", "old-original", "new-slot"] {
        let (_temp, context, index, _closed) = super::rollover_tests::fixture(2);
        let data = index.data_root().to_path_buf();
        drop(index);
        let reached = std::cell::Cell::new(false);
        let result =
            observation::inspect_map_history_root(&context, &data, |head, history, origin| {
                assert!(successor_eligible_checked(
                    &context,
                    &data,
                    head.unwrap(),
                    history,
                    origin
                )?);
                reached.set(true);
                match case {
                    "new-root" => {
                        fs::DirBuilder::new()
                            .mode(0o700)
                            .create(context.working_directory.join(".codex"))
                            .unwrap();
                    }
                    "new-slot" => {
                        fs::DirBuilder::new()
                            .mode(0o700)
                            .create(
                                data.join("manager-state/client-root-generations")
                                    .join(uuid::Uuid::new_v4().to_string()),
                            )
                            .unwrap();
                    }
                    "old-original" => {
                        let p = fs::read_dir(data.join("manager-state/client-originals"))
                            .unwrap()
                            .next()
                            .unwrap()
                            .unwrap()
                            .path()
                            .join("original.json");
                        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                        fs::write(p, b"late external bytes").unwrap();
                    }
                    _ => unreachable!(),
                }
                Ok(true)
            });
        assert!(reached.get());
        assert!(result.is_err(), "{case}");
    }
}
#[test]
fn root_start_capacity_preflights_complete_cycle_before_allocating_root_or_slot() {
    let (_temp, context, index, closed) = super::rollover_tests::fixture(1);
    let data = index.data_root().to_path_buf();
    let origin = index.root_registration.as_ref().unwrap();
    let future = successor_cycle_capacity(&context, &data, &closed, origin).unwrap();
    assert_eq!(future.len(), 9);
    let required = future.iter().sum::<usize>();
    assert!(successor_capacity_fits(
        4096 - 9,
        4 * 1024 * 1024 - required,
        511,
        &future
    ));
    assert!(!successor_capacity_fits(
        4096 - 8,
        4 * 1024 * 1024 - required,
        511,
        &future
    ));
    assert!(!successor_capacity_fits(
        4096 - 9,
        4 * 1024 * 1024 - required + 1,
        511,
        &future
    ));
    assert!(!successor_capacity_fits(10, 0, 512, &future));
    assert!(!successor_capacity_fits(10, 0, 1, &[65537]));
    let c = index.connection().unwrap();
    let history = vault_history::read(c, &closed).unwrap();
    let prepared = history[3].clone();
    c.execute_batch("BEGIN IMMEDIATE").unwrap();
    let mut final_head = closed.clone();
    for revision in 6..=4095 {
        let mut h = if revision % 2 == 0 {
            prepared.clone()
        } else {
            closed.clone()
        };
        h.revision = revision;
        c.execute(
            "INSERT INTO events(revision,document) VALUES(?1,?2)",
            params![revision, encode(&h).unwrap()],
        )
        .unwrap();
        final_head = h;
    }
    c.execute(
        "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
        params![final_head.revision, encode(&final_head).unwrap()],
    )
    .unwrap();
    c.execute_batch("COMMIT").unwrap();
    assert_eq!(index.head().unwrap(), Some(final_head.clone()));
    assert!(!index.successor_eligible(&context, &final_head).unwrap());
    drop(index);
    let before = bytes(&data);
    assert!(!inspect_successor_eligibility(&context, &data).unwrap());
    assert_eq!(
        inspect_client_recovery(&context, &data).unwrap(),
        crate::modules::status::ClientRecoveryState::Required
    );
    assert!(Index::prepare_start_capture(&context, &data, deadline()).is_err());
    assert_eq!(bytes(&data), before);
    assert!(!context.working_directory.join(".codex").exists());
    assert!(!data.join("manager-state/client-root-generations").exists());
}
#[test]
fn root_start_observer_detects_external_same_inode_write_even_if_sqlite_cached_old_pages() {
    const TEST:&str="infrastructure::macos_client_originals::session_index::generation::start_tests::root_start_observer_detects_external_same_inode_write_even_if_sqlite_cached_old_pages";
    if let Some(root) = std::env::var_os("OCXD024_START_EXTERNAL_INDEX_ROOT") {
        use std::os::unix::fs::FileExt;
        let root = PathBuf::from(root);
        assert!(root.starts_with("/private/tmp"));
        assert!(root.ancestors().any(|p| p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("ocxd024-root-start-"))));
        let path = root.join("desktop/manager-state/client-session-index.sqlite");
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut byte = [0u8; 1];
        f.read_exact_at(&mut byte, 100).unwrap();
        byte[0] ^= 1;
        f.write_all_at(&byte, 100).unwrap();
        f.sync_all().unwrap();
        return;
    }
    let temp = tempfile::Builder::new()
        .prefix("ocxd024-root-start-external-")
        .tempdir_in("/private/tmp")
        .unwrap();
    let fixture_root = temp.path().canonicalize().unwrap();
    let (context, index, _closed) = super::rollover_tests::fixture_at(fixture_root.clone(), 1);
    let data = index.data_root().to_path_buf();
    drop(index);
    use std::os::unix::fs::MetadataExt;
    let index_path = data.join("manager-state/client-session-index.sqlite");
    let metadata = fs::metadata(&index_path).unwrap();
    let before_identity = (metadata.dev(), metadata.ino());
    let reached = std::cell::Cell::new(false);
    let result = observation::inspect_map_history_root(&context, &data, |head, history, origin| {
        assert!(successor_eligible_checked(
            &context,
            &data,
            head.unwrap(),
            history,
            origin
        )?);
        reached.set(true);
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env("OCXD024_START_EXTERNAL_INDEX_ROOT", &fixture_root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(true)
    });
    assert!(reached.get());
    assert!(result.is_err());
    let after = fs::metadata(&index_path).unwrap();
    assert_eq!((after.dev(), after.ino()), before_identity);
    println!(
        "ROOT_START_EXTERNAL_INDEX_EVIDENCE {}",
        serde_json::json!({"accepted":false,"externalWriteCompleted":true,"sameInodeRetained":true,"observationRefused":true,"noAuxiliaryParentSqliteFd":true})
    );
}
