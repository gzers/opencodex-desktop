use super::*;
use crate::modules::runtime::{
    program_activation::tests::Fixture as Activation, program_selection::tests::Fixture,
};
fn selected(f: &Fixture) -> program_selection::SelectedProgram {
    let (target, platform) = program_layout::host_target().unwrap();
    program_selection::read(&f.root, platform, target)
        .unwrap()
        .unwrap()
}
#[test]
fn current_and_retained_candidates_need_explicit_admissions_not_arbitrary_manifests() {
    let f = Fixture::new();
    let current = selected(&f);
    assert!(inspect(&f.root).is_err());
    assert!(!f.root.join(DIRECTORY).exists());
    register(
        &f.root,
        &current.selection,
        &current.image.official_archive_sha256,
    )
    .unwrap();
    let c = inspect(&f.root).unwrap();
    assert_eq!(c.selections, vec![current.selection.clone()]);
    let before = read(
        &f.root,
        &f.root.join(DIRECTORY).join(filename(&current.selection)),
    )
    .unwrap();
    register(
        &f.root,
        &current.selection,
        &current.image.official_archive_sha256,
    )
    .unwrap();
    assert_eq!(inspect(&f.root).unwrap().revision, c.revision);
    assert_eq!(
        read(
            &f.root,
            &f.root.join(DIRECTORY).join(filename(&current.selection))
        )
        .unwrap(),
        before
    );
    let candidate = Activation::image(&f.root, "3.0.0");
    assert!(inspect(&f.root).is_err());
    let image = program_selection::read_image(
        &f.root,
        program_layout::host_target().unwrap().1,
        "darwin-arm64",
        &candidate,
    )
    .unwrap();
    register(&f.root, &candidate, &image.image.official_archive_sha256).unwrap();
    let after = inspect(&f.root).unwrap();
    assert_eq!(after.selections.len(), 2);
    assert_ne!(after.revision, c.revision);
}
#[test]
fn incorrect_source_ledger_or_image_refused_without_rewriting_it() {
    for kind in [
        "source",
        "ledger",
        "image",
        "unknown-directory",
        "linked-record",
        "duplicate",
    ] {
        let f = Fixture::new();
        let image = selected(&f);
        register(
            &f.root,
            &image.selection,
            &image.image.official_archive_sha256,
        )
        .unwrap();
        let path = f.root.join(DIRECTORY).join(filename(&image.selection));
        match kind {
            "source" => {
                assert!(register(&f.root, &image.selection, &"a".repeat(64)).is_err());
            }
            "ledger" => {
                fs::write(&path, b"broken original registration").unwrap();
            }
            "image" => {
                fs::write(image.paths.version_root.join("foreign-file"), b"retain").unwrap();
            }
            "unknown-directory" => {
                fs::create_dir(f.root.join("runtime/versions/unregistered")).unwrap();
            }
            "linked-record" => {
                fs::hard_link(&path, f.root.join("linked-record")).unwrap();
            }
            "duplicate" => {
                fs::copy(&path, f.root.join(DIRECTORY).join("other.json")).unwrap();
            }
            _ => unreachable!(),
        }
        let before = fs::read(&path).unwrap();
        if kind != "source" {
            assert!(inspect(&f.root).is_err(), "{kind}");
        }
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
