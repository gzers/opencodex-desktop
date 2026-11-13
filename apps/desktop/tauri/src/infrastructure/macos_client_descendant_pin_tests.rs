//! Existing-directory acquisition only; no file-content or movement authority.
use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    time::Instant,
};
fn fixture() -> (tempfile::TempDir, PathBuf, PinnedDirectory) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut path = root.clone();
    for component in ["base", "one", "two", "three"] {
        path.push(component);
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    }
    let base = root.join("base");
    let pin = PinnedDirectory::open(&base).unwrap();
    (temp, base, pin)
}
#[test]
fn descendant_pin_matches_sequential_acquisition_with_fewer_complete_security_reads() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_temp, base, pin) = fixture();
        let started = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            if reference {
                pin.child_private("one", false)?
                    .child_private("two", false)?
                    .child_private("three", false)
            } else {
                pin.descendant_private(&["one", "two", "three"])
            }
        });
        let elapsed = started.elapsed().as_nanos();
        let result = result.unwrap();
        assert_eq!(result.path(), base.join("one/two/three"));
        let baseline = PinnedDirectory::open(result.path()).unwrap();
        assert_eq!(result.identity(), baseline.identity());
        assert_eq!(
            result.chain.last().unwrap().2,
            baseline.chain.last().unwrap().2
        );
        drop(pin);
        result.private().unwrap();
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
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
    println!(
        "DESCENDANT_PIN_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"pure existing directory handles; no content, native permission or Stop budget claim"})
    );
}
#[test]
fn descendant_pin_rejects_base_intermediate_and_leaf_changes_and_preserves_final_parent_boundary() {
    for (stage, index, suffix) in [
        ("base-admitted", 0, ""),
        ("component-pinned", 0, "one"),
        ("component-pinned", 1, "one/two"),
        ("component-pinned", 2, "one/two/three"),
        ("chain-checked", 3, "one/two"),
    ] {
        for change in ["mode", "replacement", "symlink", "xattr"] {
            let (_temp, base, pin) = fixture();
            let target = if suffix.is_empty() {
                base.clone()
            } else {
                base.join(suffix)
            };
            let reached = std::cell::Cell::new(false);
            let result = pin.descendant_private_checked(&["one", "two", "three"], &|point, i| {
                if point == stage && i == index {
                    reached.set(true);
                    match change {
                        "mode" => {
                            fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap()
                        }
                        "replacement" | "symlink" => {
                            let retained = target.with_file_name(format!(
                                "retained-{}",
                                target.file_name().unwrap().to_string_lossy()
                            ));
                            fs::rename(&target, &retained).unwrap();
                            if change == "symlink" {
                                symlink(&retained, &target).unwrap();
                            } else {
                                fs::DirBuilder::new().mode(0o700).create(&target).unwrap();
                            }
                        }
                        "xattr" => {
                            let path = CString::new(target.as_os_str().as_bytes()).unwrap();
                            let key = CString::new("user.ocxd-descendant-test").unwrap();
                            assert_eq!(
                                unsafe {
                                    libc::setxattr(
                                        path.as_ptr(),
                                        key.as_ptr(),
                                        b"x".as_ptr().cast(),
                                        1,
                                        0,
                                        0,
                                    )
                                },
                                0
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            });
            assert!(reached.get());
            assert!(result.is_err(), "{stage}/{index}/{change}");
            assert!(fs::symlink_metadata(&target).is_ok());
        }
    }
}
#[test]
fn descendant_pin_never_creates_and_rejects_unsafe_paths_and_security() {
    let (_temp, base, pin) = fixture();
    for children in [
        vec![],
        vec![""],
        vec!["."],
        vec![".."],
        vec!["/"],
        vec!["one/two"],
        vec!["one", "two/three"],
        vec!["one"; 9],
    ] {
        assert!(pin.descendant_private(&children).is_err());
    }
    assert!(pin.descendant_private(&["missing"]).is_err());
    assert!(!base.join("missing").exists());
    fs::write(base.join("file"), b"retained").unwrap();
    assert!(pin.descendant_private(&["file"]).is_err());
    assert_eq!(fs::read(base.join("file")).unwrap(), b"retained");
    symlink(base.join("one"), base.join("link")).unwrap();
    assert!(pin.descendant_private(&["link"]).is_err());
    fs::set_permissions(base.join("one/two"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(pin.descendant_private(&["one", "two", "three"]).is_err());
    assert_eq!(
        fs::metadata(base.join("one/two"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    fs::set_permissions(base.join("one/two"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(base.join("back\\slash"))
        .unwrap();
    assert!(pin.descendant_private(&["back\\slash"]).is_ok());
    let descendant = pin.descendant_private(&["one", "two", "three"]).unwrap();
    fs::set_permissions(base.join("one/two"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(descendant.recheck().is_err());
    assert!(pin.descendant_private(&["one", "two", "three"]).is_err());
}
