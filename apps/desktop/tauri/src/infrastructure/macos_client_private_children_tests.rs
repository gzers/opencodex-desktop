//! Pure directory metadata scan only; no provenance inputs or permissions.
use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
};
fn fixture() -> (tempfile::TempDir, PathBuf, PinnedDirectory) {
    let t = tempfile::tempdir().unwrap();
    let base = t.path().canonicalize().unwrap().join("parent");
    fs::DirBuilder::new().mode(0o700).create(&base).unwrap();
    for child in ["one", "two"] {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(base.join(child))
            .unwrap();
    }
    let pin = PinnedDirectory::open(&base).unwrap();
    (t, base, pin)
}
#[test]
fn private_children_scan_keeps_unsafe_objects_missing_paths_and_literal_name_rules() {
    let (_t, base, pin) = fixture();
    let names = vec!["one".into(), "two".into()];
    pin.check_private_children(&[]).unwrap();
    assert!(pin
        .check_private_children(&vec!["one".into(); 512])
        .is_err());
    pin.check_private_children(&names).unwrap();
    for bad in ["", ".", "..", "/", "one/two", "missing"] {
        assert!(pin.check_private_children(&[bad.into()]).is_err(), "{bad}");
    }
    assert!(!base.join("missing").exists());
    symlink(base.join("one"), base.join("link")).unwrap();
    assert!(pin.check_private_children(&["link".into()]).is_err());
    fs::write(base.join("file"), b"kept").unwrap();
    assert!(pin.check_private_children(&["file".into()]).is_err());
    fs::set_permissions(base.join("one"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(pin.check_private_children(&names).is_err());
    assert_eq!(
        fs::metadata(base.join("one")).unwrap().permissions().mode() & 0o777,
        0o755
    );
}
#[test]
fn private_children_scan_refuses_child_and_parent_mutations_before_return() {
    for (stage, root, target_index) in [
        ("child-opened", "child", 0),
        ("child-first-pass", "child", 0),
        ("child-opened", "child", 1),
        ("child-first-pass", "child", 1),
        ("children-checked", "parent", 2),
    ] {
        for change in ["mode", "acl", "xattr", "replacement", "symlink"] {
            let (_t, base, pin) = fixture();
            let path = if root == "parent" {
                base.clone()
            } else {
                base.join(if target_index == 1 { "two" } else { "one" })
            };
            let called = std::cell::Cell::new(false);
            let result =
                pin.check_private_children_checked(&["one".into(), "two".into()], &|point, i| {
                    if point == stage && (point == "children-checked" || i == target_index) {
                        called.set(true);
                        match change {
                            "acl" => {
                                let user = unsafe { libc::getpwuid(libc::geteuid()) };
                                assert!(!user.is_null());
                                let name = unsafe { std::ffi::CStr::from_ptr((*user).pw_name) }
                                    .to_str()
                                    .unwrap();
                                let ace = format!("user:{name} allow read");
                                let changed = std::process::Command::new("/bin/chmod")
                                    .args(["+a", &ace])
                                    .arg(&path)
                                    .output()
                                    .unwrap();
                                assert!(changed.status.success());
                            }
                            "mode" => fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                                .unwrap(),
                            "xattr" => {
                                let p = CString::new(path.as_os_str().as_bytes()).unwrap();
                                assert_eq!(
                                    unsafe {
                                        libc::setxattr(
                                            p.as_ptr(),
                                            c"user.ocxd-children".as_ptr(),
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
                                let retained = path.with_file_name(format!(
                                    "retained-{}",
                                    path.file_name().unwrap().to_string_lossy()
                                ));
                                fs::rename(&path, &retained).unwrap();
                                if change == "symlink" {
                                    symlink(retained, &path).unwrap();
                                } else {
                                    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Ok(())
                });
            assert!(called.get());
            assert!(result.is_err(), "{stage}/{change}");
            assert!(fs::symlink_metadata(path).is_ok());
        }
    }
}
