use super::*;
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    time::Duration,
};
struct Fixture {
    _temp: tempfile::TempDir,
    data: PathBuf,
    profile: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        Self::at_with(temp)
    }
    fn at_with(temp: tempfile::TempDir) -> Self {
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let profile = root.join("profile");
        for p in [&data, &profile] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        Self {
            _temp: temp,
            data,
            profile,
        }
    }
    fn deadline(&self) -> Instant {
        Instant::now() + Duration::from_secs(10)
    }
}
fn inventory(root: &Path) -> BTreeMap<PathBuf, (Identity, u32, Vec<u8>)> {
    fn walk(root: &Path, at: &Path, out: &mut BTreeMap<PathBuf, (Identity, u32, Vec<u8>)>) {
        for e in fs::read_dir(at).unwrap() {
            let p = e.unwrap().path();
            let m = fs::symlink_metadata(&p).unwrap();
            if m.is_dir() {
                walk(root, &p, out);
            } else {
                let f = File::open(&p).unwrap();
                out.insert(
                    p.strip_prefix(root).unwrap().into(),
                    (
                        identity(&stat(&f).unwrap()),
                        m.permissions().mode(),
                        fs::read(&p).unwrap(),
                    ),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
#[test]
fn first_root_intent_precedes_exclusive_publication_and_retains_original_absence() {
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    assert!(!f.profile.join(".codex").exists());
    let bytes = fs::read(
        f.data
            .join("manager-state/client-root-reservation/intent.json"),
    )
    .unwrap();
    let intent: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(intent["originalRootAbsent"], true);
    assert_eq!(intent["purpose"], PURPOSE);
    assert!(reserve(&f.data, &f.profile, f.deadline()).is_err());
    let root = r.publish().unwrap();
    let refs = root.references().unwrap();
    assert_eq!(refs.intent_sha256, digest(&bytes));
    assert_eq!(root.path(), f.profile.join(".codex"));
    assert!(fs::read_dir(root.path()).unwrap().next().is_none());
    let created = fs::read(
        f.data
            .join("manager-state/client-root-reservation/created.json"),
    )
    .unwrap();
    let published = fs::read(
        f.data
            .join("manager-state/client-root-reservation/published.json"),
    )
    .unwrap();
    assert_eq!(refs.created_sha256, digest(&created));
    assert_eq!(refs.published_sha256, digest(&published));
    assert!(!f
        .data
        .join("manager-state/client-session-index.sqlite")
        .exists());
    assert!(!f.data.join("manager-state/client-originals").exists());
    drop(root);
    assert!(reserve(&f.data, &f.profile, f.deadline()).is_err()); // Disk evidence grants no automatic adoption.
}
#[test]
fn first_root_existing_target_or_final_publish_race_is_never_replaced_or_adopted() {
    for kind in 0..3 {
        let f = Fixture::new();
        let target = f.profile.join(".codex");
        match kind {
            0 => fs::create_dir(&target).unwrap(),
            1 => fs::write(&target, b"external file").unwrap(),
            _ => symlink(f.profile.join("external-missing"), &target).unwrap(),
        };
        assert!(reserve(&f.data, &f.profile, f.deadline()).is_err());
        assert!(fs::symlink_metadata(&target).is_ok());
        assert!(!f.data.join("manager-state").exists());
    }
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    let target = f.profile.join(".codex");
    assert!(r
        .publish_checked(&|point| {
            if point == "before_publish" {
                fs::create_dir(&target)?;
                fs::write(target.join("outside"), b"do not replace")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(target.join("outside")).unwrap(), b"do not replace");
    assert!(f
        .data
        .join("manager-state/client-root-reservation/candidate")
        .is_dir());
    assert!(!f
        .data
        .join("manager-state/client-root-reservation/published.json")
        .exists());
}
#[test]
fn first_root_rejects_foreign_stage_prior_index_and_changed_profile_or_reference() {
    let f = Fixture::new();
    let manager = f.data.join("manager-state");
    fs::DirBuilder::new().mode(0o700).create(&manager).unwrap();
    fs::write(manager.join("client-session-index.sqlite"), b"prior index").unwrap();
    assert!(reserve(&f.data, &f.profile, f.deadline()).is_err());
    assert!(!f.profile.join(".codex").exists());
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(r.vault.path().join("candidate"))
        .unwrap();
    assert!(r.publish().is_err());
    assert!(!f.profile.join(".codex").exists());
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    let reference = f.data.join("manager-state").join(REFERENCE);
    fs::set_permissions(&reference, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&reference, b"{}").unwrap();
    assert!(r.publish().is_err());
    assert_eq!(fs::read(reference).unwrap(), b"{}");
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    fs::set_permissions(&f.profile, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(r.publish().is_err());
    assert!(!f.profile.join(".codex").exists());
}
#[test]
fn first_root_created_or_published_directory_replacement_is_refused_without_cleanup() {
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    let candidate = r.vault.path().join("candidate");
    let displaced = r.vault.path().join("displaced");
    assert!(r
        .publish_checked(&|point| {
            if point == "created" {
                fs::rename(&candidate, &displaced)?;
                fs::DirBuilder::new().mode(0o700).create(&candidate)?;
            }
            Ok(())
        })
        .is_err());
    assert!(candidate.is_dir() && displaced.is_dir());
    assert!(!f.profile.join(".codex").exists());
    let f = Fixture::new();
    let root = reserve(&f.data, &f.profile, f.deadline())
        .unwrap()
        .publish()
        .unwrap();
    fs::rename(root.path(), f.profile.join("displaced-codex")).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(root.path())
        .unwrap();
    assert!(root.references().is_err());
}
#[test]
fn first_root_action_flock_excludes_a_real_second_process() {
    const CHILD: &str = "OCXD024_ROOT_LOCK_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let e = reserve(
            &root.join("desktop"),
            &root.join("profile"),
            Instant::now() + Duration::from_secs(5),
        )
        .err()
        .expect("another action is held");
        assert!(e.to_string().contains("action is active"));
        return;
    }
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_creation::tests::first_root_action_flock_excludes_a_real_second_process","--nocapture"]).env(CHILD,f.data.parent().unwrap()).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    drop(r);
    assert!(!f.profile.join(".codex").exists());
}
#[test]
fn first_root_real_crash_preserves_absence_or_created_root_without_minting_reopen_authority() {
    const CHILD: &str = "OCXD024_ROOT_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let data = root.join("desktop");
        let profile = root.join("profile");
        for p in [&data, &profile] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        let boundary = std::env::var("OCXD024_ROOT_CRASH_BOUNDARY").unwrap();
        let checkpoint = |point: &str| {
            if point == boundary {
                std::process::exit(91);
            }
            Ok(())
        };
        let r = reserve_checked(
            &data,
            &profile,
            Instant::now() + Duration::from_secs(10),
            &checkpoint,
        )
        .unwrap();
        r.publish_checked(&checkpoint).unwrap();
        panic!("exit boundary missed");
    }
    for boundary in [
        "intent_published",
        "created",
        "root_published",
        "publication_recorded",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_creation::tests::first_root_real_crash_preserves_absence_or_created_root_without_minting_reopen_authority","--nocapture"]).env(CHILD,&root).env("OCXD024_ROOT_CRASH_BOUNDARY",boundary).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let before = inventory(&root);

        let rows=before.iter().map(|(path,(identity,mode,bytes))|serde_json::json!({"path":path,"identity":identity,"mode":mode,"bytes":bytes.len(),"sha256":digest(bytes),"rawUtf8":std::str::from_utf8(bytes).ok()})).collect::<Vec<_>>();
        println!(
            "ROOT_CREATION_CRASH_EVIDENCE {}",
            serde_json::json!({"accepted":false,"boundary":boundary,"childExitCode":out.status.code(),"rootPresent":root.join("profile/.codex").exists(),"noSessionIndex":!root.join("desktop/manager-state/client-session-index.sqlite").exists(),"files":rows})
        );
        assert!(reserve(
            &root.join("desktop"),
            &root.join("profile"),
            Instant::now() + Duration::from_secs(10)
        )
        .is_err());
        assert!(inventory(&root) == before);
        assert_eq!(
            root.join("profile/.codex").exists(),
            matches!(boundary, "root_published" | "publication_recorded")
        );
        assert!(!root
            .join("desktop/manager-state/client-session-index.sqlite")
            .exists());
    }
}
#[test]
fn first_root_deadline_and_nonempty_publication_never_grant_index_transfer() {
    let f = Fixture::new();
    assert!(reserve(&f.data, &f.profile, Instant::now()).is_err());
    assert!(!f.data.join("manager-state").exists());
    let f = Fixture::new();
    let r = reserve(&f.data, &f.profile, f.deadline()).unwrap();
    let root = r.publish().unwrap();
    fs::write(root.path().join("outside-after-publish"), b"keep").unwrap();
    assert!(root.references().is_err());
    assert_eq!(
        fs::read(root.path().join("outside-after-publish")).unwrap(),
        b"keep"
    );
}

#[test]
fn first_root_pending_creation_is_rejected_by_actual_lifecycle_index_and_read_only_observation() {
    let f = Fixture::new();
    let home = f.profile.join("opencodex-home");
    fs::DirBuilder::new().mode(0o700).create(&home).unwrap();
    let published = reserve(&f.data, &f.profile, f.deadline())
        .unwrap()
        .publish()
        .unwrap();
    let target = published.path().to_path_buf();
    drop(published);
    let context = crate::state::ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: f.profile.clone(),
        opencodex_home: home,
    };
    let before = inventory(f.data.parent().unwrap());
    let error = session_index::Index::open(&context, &f.data, &target)
        .err()
        .expect("first root cannot be auto-adopted");
    assert!(error
        .to_string()
        .contains("first client root is not registered"));
    assert!(session_index::inspect(&context, &f.data).is_err());
    assert!(inventory(f.data.parent().unwrap()) == before);
    assert!(!f
        .data
        .join("manager-state/client-session-index.sqlite")
        .exists());
    assert!(!f.data.join("manager-state/client-originals").exists());
}
