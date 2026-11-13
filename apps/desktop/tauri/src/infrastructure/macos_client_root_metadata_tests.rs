//! Metadata-only admission and per-frame dynamic provenance regression.
use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    time::Instant,
};
#[test]
fn root_metadata_union_keeps_all_leaf_policies_and_reduces_shared_ancestor_reads() {
    let (_temp, context, index, _head, originals) =
        super::super::root_retirement::fixture_captured_generation();
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let start = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            if reference {
                observation_metrics::root_metadata_reference(|| {
                    verify_anchor_metadata(&originals.profile, &originals.data, &originals.client)
                })
            } else {
                verify_anchor_metadata(&originals.profile, &originals.data, &originals.client)
            }
        });
        result.unwrap();
        samples.push(serde_json::json!({"reference":reference,"elapsedNanos":start.elapsed().as_nanos(),"metrics":metrics}));
    }
    let old = samples[0]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    let new = samples[1]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    assert!(new < old);
    assert_eq!(samples[2]["metrics"]["protection"]["calls"], new);
    assert_eq!(samples[3]["metrics"]["protection"]["calls"], old);
    assert!(context.working_directory.exists() && index.has_original_root_absence());
    println!(
        "ROOT_METADATA_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"fresh metadata only, no record reads, source check skipped or service permission"})
    );
}
#[test]
fn root_metadata_union_refuses_changes_after_leaf_policies_before_any_source_read() {
    for target in ["profile", "data", "client"] {
        for change in ["mode", "xattr", "replacement", "symlink"] {
            let (_temp, context, _index, _head, originals) =
                super::super::root_retirement::fixture_captured_generation();
            let path = match target {
                "profile" => context.working_directory.clone(),
                "data" => originals.data.path().to_path_buf(),
                _ => originals.client.path().to_path_buf(),
            };
            let reached = std::cell::Cell::new(false);
            let result = verify_anchor_metadata_checked(
                &originals.profile,
                &originals.data,
                &originals.client,
                &|| {
                    reached.set(true);
                    match change {
                        "mode" => {
                            fs::set_permissions(&path, fs::Permissions::from_mode(0o777)).unwrap()
                        }
                        "xattr" => {
                            let name = CString::new("user.ocxd-root-metadata").unwrap();
                            let p = CString::new(path.as_os_str().as_bytes()).unwrap();
                            assert_eq!(
                                unsafe {
                                    libc::setxattr(
                                        p.as_ptr(),
                                        name.as_ptr(),
                                        b"x".as_ptr().cast(),
                                        1,
                                        0,
                                        0,
                                    )
                                },
                                0
                            );
                        }
                        "replacement" | "symlink" => {
                            let kept = path.with_file_name(format!(
                                "retained-{}",
                                path.file_name().unwrap().to_string_lossy()
                            ));
                            fs::rename(&path, &kept).unwrap();
                            if change == "symlink" {
                                symlink(kept, &path).unwrap()
                            } else {
                                fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
                            }
                        }
                        _ => unreachable!(),
                    }
                    Ok(())
                },
            );
            assert!(reached.get());
            assert!(result.is_err(), "{target}/{change}");
            assert!(fs::symlink_metadata(path).is_ok());
        }
    }
}
#[test]
fn root_metadata_prefix_must_refuse_transient_origin_changes_before_next_archive() {
    for mutation in ["content", "mode", "namespace"] {
        let (_temp, _context, _index, _head, originals) =
            super::super::root_retirement::fixture_captured_generation();
        for phase in [Phase::Ready, Phase::PreStop, Phase::Stopped] {
            originals.seal(phase).unwrap();
        }
        let source = originals
            .data
            .path()
            .join("manager-state/client-root-reservation/created.json");
        let before = fs::read(&source).unwrap();
        let generations = originals
            .data
            .path()
            .join("manager-state/client-root-generations");
        let slot = generations.join(uuid::Uuid::new_v4().to_string());
        let observed = std::cell::RefCell::new(Vec::new());
        let restored = std::cell::Cell::new(false);
        let (result, metrics) = observation_metrics::measure(|| {
            originals.frames_through_checked(Phase::Stopped, &|phase| {
                observed.borrow_mut().push(phase);
                if phase == Phase::Original {
                    match mutation {
                        "content" => {
                            fs::set_permissions(&source, fs::Permissions::from_mode(0o600))
                                .unwrap();
                            fs::write(&source, b"temporary corruption").unwrap();
                            fs::set_permissions(&source, fs::Permissions::from_mode(0o400))
                                .unwrap();
                        }
                        "mode" => {
                            fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap()
                        }
                        "namespace" => {
                            fs::DirBuilder::new()
                                .mode(0o700)
                                .create(&generations)
                                .unwrap();
                            fs::DirBuilder::new().mode(0o700).create(&slot).unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                if phase == Phase::PreStop {
                    restored.set(true);
                    if mutation == "namespace" {
                        fs::remove_dir(&slot).unwrap()
                    } else {
                        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
                        fs::write(&source, &before).unwrap();
                        fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
                    }
                }
                Ok(())
            })
        });
        assert!(result.is_err(), "{mutation}");
        assert_eq!(*observed.borrow(), vec![Phase::Original]);
        assert!(!restored.get());
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(metrics["frame"]["calls"], 2);
        assert_eq!(metrics["frame_archives"]["calls"], 1);
        assert_eq!(metrics["root_provenance"]["calls"], 2);
        if mutation == "mode" {
            assert_eq!(
                fs::metadata(&source).unwrap().permissions().mode() & 0o7777,
                0o600
            );
        }
        if mutation == "content" {
            assert_eq!(fs::read(&source).unwrap(), b"temporary corruption");
        }
        if mutation == "namespace" {
            assert!(slot.is_dir());
        }
    }
}
