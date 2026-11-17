use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    time::Duration,
};

struct Fixture {
    _temp: tempfile::TempDir,
    context: ProcessContext,
    data: PathBuf,
    index: Index,
    head: Head,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let profile = root.join("profile");
        let home = profile.join("opencodex");
        for p in [&data, &profile, &home] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile,
            opencodex_home: home,
        };
        let (index, head) = Index::prepare_start_capture(&context, &data, end()).unwrap();
        Self {
            _temp: temp,
            context,
            data,
            index,
            head,
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, at: &Path, result: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                visit(root, &path, result);
            } else if metadata.is_file() {
                result.push((
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut result = Vec::new();
    visit(root, root, &mut result);
    result.sort();
    result
}
#[test]
fn first_root_cancel_review_is_stable_readonly_and_never_closes_or_moves_root() {
    for reserved in [false, true] {
        let f = Fixture::new();
        let head = if reserved {
            f.index.reserve_first_root_capture(&f.head).unwrap()
        } else {
            f.head.clone()
        };
        let before = bytes(f.data.parent().unwrap());
        let review = f
            .index
            .review_first_root_preparation(&f.context, &head, end())
            .unwrap();
        assert_eq!(review.fingerprint().len(), 64);
        review.recheck().unwrap();
        assert_eq!(f.index.head().unwrap().as_ref(), Some(&head));
        assert_eq!(bytes(f.data.parent().unwrap()), before);
        assert!(f.context.working_directory.join(".codex").is_dir());
        assert!(!f.data.join("manager-state/client-originals").exists());
    }
}
#[test]
fn first_root_cancel_review_refuses_new_data_even_if_head_has_no_original_references() {
    for target in ["root-data", "partial-capture", "empty-capture-storage"] {
        let f = Fixture::new();
        let review = f
            .index
            .review_first_root_preparation(&f.context, &f.head, end())
            .unwrap();
        let path = if target == "root-data" {
            f.context.working_directory.join(".codex/external")
        } else {
            let p = f.data.join("manager-state/client-originals");
            fs::create_dir(&p).unwrap();
            p.join("unregistered")
        };
        if target != "empty-capture-storage" {
            fs::write(&path, b"preserve external bytes").unwrap();
        }
        let before = bytes(f.data.parent().unwrap());
        assert!(review.recheck().is_err());
        assert!(f
            .index
            .review_first_root_preparation(&f.context, &f.head, end())
            .is_err());
        assert_eq!(before, bytes(f.data.parent().unwrap()));
    }
}
#[test]
fn first_root_cancel_review_keeps_original_deadline_and_exact_index_revision() {
    let f = Fixture::new();
    assert!(f
        .index
        .review_first_root_preparation(&f.context, &f.head, Instant::now())
        .is_err());
    let review = f
        .index
        .review_first_root_preparation(&f.context, &f.head, end())
        .unwrap();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    assert!(review.recheck().is_err());
    assert!(f
        .index
        .review_first_root_preparation(&f.context, &f.head, end())
        .is_err());
    f.index
        .review_first_root_preparation(&f.context, &reserved, end())
        .unwrap()
        .recheck()
        .unwrap();
    let review = f
        .index
        .review_first_root_preparation(&f.context, &reserved, end())
        .unwrap();
    let expired = Review {
        deadline: Instant::now(),
        ..review
    };
    assert!(expired.recheck().is_err());
}
#[test]
fn first_root_cancel_review_refuses_wrong_context_replacement_and_permissions() {
    for mode in ["profile", "home", "replacement", "symlink", "permissions"] {
        let f = Fixture::new();
        let root = f.context.working_directory.join(".codex");
        let review = f
            .index
            .review_first_root_preparation(&f.context, &f.head, end())
            .unwrap();
        let mut context = f.context.clone();
        match mode {
            "profile" => context.working_directory = f.data.clone(),
            "home" => context.opencodex_home = f.data.clone(),
            "permissions" => fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap(),
            _ => {
                fs::rename(&root, f.context.working_directory.join("retained-original")).unwrap();
                if mode == "replacement" {
                    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
                } else {
                    symlink(f.context.working_directory.join("retained-original"), &root).unwrap();
                }
            }
        }
        let before = bytes(f.data.parent().unwrap());
        assert!(
            f.index
                .review_first_root_preparation(&context, &f.head, end())
                .is_err(),
            "{mode}"
        );
        if !matches!(mode, "profile" | "home") {
            assert!(review.recheck().is_err());
        }
        assert_eq!(bytes(f.data.parent().unwrap()), before);
    }
}

#[test]
fn first_root_cancel_review_binds_grok_state_and_does_not_adopt_an_existing_root() {
    let f = Fixture::new();
    let review = f
        .index
        .review_first_root_preparation(&f.context, &f.head, end())
        .unwrap();
    let grok = f.context.working_directory.join(".grok");
    fs::DirBuilder::new().mode(0o700).create(&grok).unwrap();
    fs::write(grok.join("config.json"), b"{\"external\":true}").unwrap();
    assert!(review.recheck().is_err());
    let root = f.context.working_directory.join("other-scope");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let data = root.join("desktop");
    let profile = root.join("profile");
    for p in [
        &data,
        &profile,
        &profile.join(".codex"),
        &profile.join("home"),
    ] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    let context = ProcessContext {
        runtime: f.context.runtime.clone(),
        working_directory: profile.clone(),
        opencodex_home: profile.join("home"),
    };
    let (index, head) = Index::prepare_start_capture(&context, &data, end()).unwrap();
    let before = bytes(&root);
    assert!(index
        .review_first_root_preparation(&context, &head, end())
        .is_err());
    assert_eq!(before, bytes(&root));
}
