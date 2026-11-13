use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    profile: PathBuf,
    client: PathBuf,
    data: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-grok-archive-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let profile = root.join("profile");
        let client = profile.join(".codex");
        let data = profile.join(".codex-desktop/desktop");
        for p in [&client, &data] {
            crate::infrastructure::platform::create_private_dir_all(p, 0o700).unwrap();
        }
        fs::write(client.join("config.toml"), b"native Codex input").unwrap();
        Self {
            _temp: temp,
            root,
            profile,
            client,
            data,
        }
    }
    fn begin(&self) -> Originals {
        Originals::begin_for_profile(&self.data, &self.client, &self.profile).unwrap()
    }
    fn grok(&self) -> PathBuf {
        self.profile.join(".grok")
    }
    fn create_grok(&self) {
        fs::create_dir(self.grok()).unwrap();
        fs::set_permissions(self.grok(), fs::Permissions::from_mode(0o700)).unwrap();
    }
}
#[test]
fn missing_grok_root_is_recorded_and_not_created_while_later_presence_stays_a_distinct_phase() {
    let f = Fixture::new();
    let originals = f.begin();
    assert!(!f.grok().exists());
    assert!(originals
        .archived_grok_file(Phase::Original, "config.toml")
        .is_err());
    let value: serde_json::Value = serde_json::from_slice(
        &fs::read(originals.directory.path().join("original.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(value["grok"]["state"], "absent");
    f.create_grok();
    fs::write(f.grok().join("config.toml"), b"user created after capture").unwrap();
    let ready = originals.seal(Phase::Ready).unwrap();
    assert_eq!(
        originals
            .archived_grok_file(Phase::Ready, "config.toml")
            .unwrap()
            .as_slice(),
        b"user created after capture"
    );
    assert!(originals
        .archived_grok_file(Phase::Original, "config.toml")
        .is_err());
    let reopened = Originals::open_for_profile(
        &f.data,
        &f.client,
        &f.profile,
        originals.attempt(),
        originals.original_sha256(),
        &[(Phase::Ready, ready)],
    )
    .unwrap();
    assert!(reopened
        .archived_grok_file(Phase::Original, "config.toml")
        .is_err());
    assert_eq!(
        fs::read(f.grok().join("config.toml")).unwrap(),
        b"user created after capture"
    );
}
#[test]
fn grok_config_backup_and_nested_agent_bytes_are_originals_without_modifying_current_input() {
    let f = Fixture::new();
    f.create_grok();
    fs::write(f.grok().join("config.toml"), b"native Grok config").unwrap();
    fs::write(
        f.grok().join("config.toml.bak-opencodex"),
        b"prior user backup",
    )
    .unwrap();
    fs::create_dir(f.grok().join("agents")).unwrap();
    fs::write(
        f.grok().join("agents/AGENTS.md"),
        b"native Grok agent notes",
    )
    .unwrap();
    // A Codex-only reserved filename in another client is an ordinary file.
    fs::write(
        f.grok().join(".opencodex-native-main.claim.sqlite"),
        b"unrelated user file",
    )
    .unwrap();
    let originals = f.begin();
    fs::write(f.grok().join("config.toml"), b"routed Grok config").unwrap();
    let ready = originals.seal(Phase::Ready).unwrap();
    let reopened = Originals::open_for_profile(
        &f.data,
        &f.client,
        &f.profile,
        originals.attempt(),
        originals.original_sha256(),
        &[(Phase::Ready, ready)],
    )
    .unwrap();
    for (path, bytes) in [
        ("config.toml", b"native Grok config".as_slice()),
        ("config.toml.bak-opencodex", b"prior user backup".as_slice()),
        ("agents/AGENTS.md", b"native Grok agent notes".as_slice()),
        (
            ".opencodex-native-main.claim.sqlite",
            b"unrelated user file".as_slice(),
        ),
    ] {
        assert_eq!(
            reopened
                .archived_grok_file(Phase::Original, path)
                .unwrap()
                .as_slice(),
            bytes
        );
    }
    assert_eq!(
        reopened
            .archived_grok_file(Phase::Ready, "config.toml")
            .unwrap()
            .as_slice(),
        b"routed Grok config"
    );
    assert_eq!(
        fs::read(f.grok().join("config.toml")).unwrap(),
        b"routed Grok config"
    );
    assert!(reopened
        .archived_grok_file(Phase::Original, "../.codex/config.toml")
        .is_err());
}
#[test]
fn grok_symlink_unreadable_or_non_directory_never_becomes_empty_and_no_outside_files_change() {
    for kind in ["symlink", "file", "unreadable"] {
        let f = Fixture::new();
        fs::create_dir(f.root.join("outside")).unwrap();
        fs::write(f.root.join("outside/config.toml"), b"outside").unwrap();
        match kind {
            "symlink" => symlink(f.root.join("outside"), f.grok()).unwrap(),
            "file" => fs::write(f.grok(), b"ordinary file").unwrap(),
            _ => {
                f.create_grok();
                let p = f.grok().join("config.toml");
                fs::write(&p, b"unreadable").unwrap();
                fs::set_permissions(&p, fs::Permissions::from_mode(0o000)).unwrap();
            }
        }
        assert!(
            Originals::begin_for_profile(&f.data, &f.client, &f.profile).is_err(),
            "{kind}"
        );
        assert_eq!(
            fs::read(f.root.join("outside/config.toml")).unwrap(),
            b"outside"
        );
    }
}
#[test]
fn original_grok_references_cannot_be_applied_to_another_profile_or_repaired_from_current_files() {
    let f = Fixture::new();
    f.create_grok();
    fs::write(f.grok().join("config.toml"), b"original Grok").unwrap();
    let originals = f.begin();
    let other = f.root.join("other-profile");
    fs::create_dir(&other).unwrap();
    assert!(Originals::open_for_profile(
        &f.data,
        &f.client,
        &other,
        originals.attempt(),
        originals.original_sha256(),
        &[]
    )
    .is_err());
    let content = originals
        .directory
        .path()
        .join(format!("tree-content-{}.blob", digest(b"original Grok")));
    fs::remove_file(&content).unwrap();
    assert!(Originals::open_for_profile(
        &f.data,
        &f.client,
        &f.profile,
        originals.attempt(),
        originals.original_sha256(),
        &[]
    )
    .is_err());
    assert!(!content.exists());
    assert_eq!(
        fs::read(f.grok().join("config.toml")).unwrap(),
        b"original Grok"
    );
}
