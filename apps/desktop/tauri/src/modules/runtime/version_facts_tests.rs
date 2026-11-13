use super::*;
use crate::modules::runtime::program_activation::tests::Fixture;
#[test]
fn disk_record_and_selection_are_independent_and_observation_never_rewrites_external_package_or_history(
) {
    let f = Fixture::new(true);
    let record = fs::read(RuntimeStore::new(&f.d).path()).unwrap();
    let current = fs::read(f.i.join("runtime/current.json")).unwrap();
    let first = observe(&f.i, &f.d).unwrap();
    assert_eq!(first.integrity, Integrity::Consistent);
    assert_eq!(first.disk_version.as_deref(), Some("2.99.0"));
    let path =
        f.i.join("runtime/versions/2.99.0/opencodex")
            .join(super::super::install::PACKAGE_SUBPATH)
            .join("package.json");
    let mut value: serde_json::Value = decode(&fs::read(&path).unwrap()).unwrap();
    value["version"] = "3.1.0".into();
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(&path, &bytes).unwrap();
    let after = observe(&f.i, &f.d).unwrap();
    assert_eq!(after.integrity, Integrity::Changed);
    assert_eq!(after.disk_version.as_deref(), Some("3.1.0"));
    assert_eq!(after.selected_version.as_deref(), Some("2.99.0"));
    assert_eq!(after.recorded_version.as_deref(), Some("2.99.0"));
    assert_ne!(first.confirmation_digest, after.confirmation_digest);
    assert!(require_confirmation(&f.i, &f.d, &first.confirmation_digest).is_err());
    assert!(require_confirmation(&f.i, &f.d, &after.confirmation_digest).is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read(RuntimeStore::new(&f.d).path()).unwrap(), record);
    assert_eq!(fs::read(f.i.join("runtime/current.json")).unwrap(), current);
}
#[test]
fn concurrent_disk_or_record_changes_invalidate_the_observation_instead_of_mixing_versions() {
    for record in [true, false] {
        let f = Fixture::new(true);
        assert!(observe_with(&f.i, &f.d, || {
            let p = if record {
                RuntimeStore::new(&f.d).path().to_path_buf()
            } else {
                f.i.join("runtime/current.json")
            };
            fs::write(p, b"external replacement").unwrap();
        })
        .is_err());
    }
}
#[test]
fn absent_roots_and_pending_or_invalid_identity_do_not_invent_installed_versions() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    let result = observe(&root.join("program"), &root.join("data")).unwrap();
    assert_eq!(result.integrity, Integrity::Missing);
    assert_eq!(fs::read_dir(root).unwrap().count(), 0);
    let f = Fixture::new(true);
    fs::write(
        f.i.join("runtime/.repair-pending.json"),
        b"original pending bytes",
    )
    .unwrap();
    assert_eq!(observe(&f.i, &f.d).unwrap().integrity, Integrity::Pending);
    let f = Fixture::new(true);
    let path =
        f.i.join("runtime/versions/2.99.0/opencodex")
            .join(super::super::install::PACKAGE_SUBPATH)
            .join("package.json");
    fs::write(path, br#"{"name":"foreign","version":"9.9.9"}"#).unwrap();
    let r = observe(&f.i, &f.d).unwrap();
    assert!(r.disk_version.is_none());
    assert_eq!(r.integrity, Integrity::Unavailable);
}
#[test]
fn current_observation_is_not_execution_permission_for_unknown_noncritical_content() {
    let f = Fixture::new(true);
    let before = observe(&f.i, &f.d).unwrap();
    assert!(require_confirmation(&f.i, &f.d, &before.confirmation_digest).is_ok());
    let extra = f.i.join("runtime/versions/2.99.0/extra-user-file");
    fs::write(&extra, b"retained user content").unwrap();
    assert!(require_confirmation(&f.i, &f.d, &before.confirmation_digest).is_err());
    assert_eq!(fs::read(extra).unwrap(), b"retained user content");
}
