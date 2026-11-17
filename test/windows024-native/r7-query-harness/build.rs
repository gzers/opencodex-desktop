//! Compile bounded, verbatim scanner sections without linking the Tauri UI.
//! Only outer module imports/docs are adapted; runtime functions stay exact.
use std::{
    env, fs,
    path::{Path, PathBuf},
};
fn source(repo: &Path, relative: &str) -> String {
    let path = repo.join(relative);
    println!("cargo:rerun-if-changed={}", path.display());
    fs::read_to_string(path)
        .expect("actual repository source")
        .replace("\r\n", "\n")
}
fn outer_docs(text: &str) -> String {
    text.lines()
        .map(|line| {
            line.strip_prefix("//!")
                .map_or_else(|| line.to_owned(), |rest| format!("//{rest}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn main() {
    let repo = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../..")
        .canonicalize()
        .unwrap();
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let image = source(
        &repo,
        "apps/desktop/tauri/src/modules/runtime/program_image.rs",
    );
    let image = image
        .split_once("pub fn verify(selected:")
        .expect("bounded inventory section marker")
        .0;
    assert!(image.contains("pub(crate) fn inventory(") && image.contains("fn file_entry_checked("));
    let tests = repo.join("apps/desktop/tauri/src/modules/runtime/program_image_windows_tests.rs");
    println!("cargo:rerun-if-changed={}", tests.display());
    fs::copy(&tests, out.join("program_image_windows_tests.rs")).unwrap();
    let windows = repo.join("apps/desktop/tauri/src/modules/runtime/program_image_windows.rs");
    println!("cargo:rerun-if-changed={}", windows.display());
    fs::copy(&windows, out.join("program_image_windows.rs")).unwrap();
    fs::write(out.join("program_image_section.rs"), outer_docs(image)).unwrap();
    let safety = source(&repo, "apps/desktop/tauri/src/modules/backup/safety.rs");
    let safety = safety
        .split_once("pub fn read_file(")
        .expect("bounded safety section marker")
        .0;
    let import = "use super::{BackupManifest, BackupRecord, MANIFEST_NAME, SCHEMA_VERSION};";
    assert_eq!(safety.matches(import).count(), 1);
    fs::write(
        out.join("safety_section.rs"),
        outer_docs(&safety.replace(
            import,
            "// Backup document types are outside this read-only section.",
        )),
    )
    .unwrap();
    let layout = source(
        &repo,
        "apps/desktop/tauri/src/modules/runtime/program_layout.rs",
    );
    let start = layout.find("pub(crate) fn clean_absolute(").unwrap();
    let end = layout[start..].find("\n}\n").unwrap() + start + 3;
    fs::write(
        out.join("layout_section.rs"),
        format!(
            "use std::path::{{Path, Component}};\n{}",
            &layout[start..end]
        ),
    )
    .unwrap();
    let tree = source(
        &repo,
        "apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree.rs",
    );
    let anchored = source(
        &repo,
        "apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored.rs",
    );
    let phase = source(
        &repo,
        "apps/desktop/tauri/src/infrastructure/windows_client_originals024/phase.rs",
    );
    let coordination = source(
        &repo,
        "apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs",
    );
    let between = |text: &str, start: &str, end: &str| -> String {
        let start = text.find(start).expect("exact native source section start");
        let end = text[start..]
            .find(end)
            .expect("exact native source section end")
            + start;
        text[start..end].to_owned()
    };
    fs::write(out.join("coordination_validation.rs"),format!("use std::io;\nfn refused()->io::Error{{io::ErrorKind::InvalidData.into()}}\nconst MAX:u64=2*1024*1024;\n{}",between(&coordination,"pub(super) fn validate_stable_bytes(","fn no_sidecars("))).unwrap();
    let mut partial=String::from("use crate::backend as native;\nuse native::{FileIdentity,validate_local_path};\nuse crate::infrastructure::windows_client_originals024::{bounded::{native_stamp,SnapshotData,QueryError},tree::streams};\nuse std::{fs::{File,OpenOptions},io::{self,Read,Seek,SeekFrom},os::windows::{fs::OpenOptionsExt,io::AsRawHandle},path::Path,time::Instant};\nconst ANCHORED_CODEX_STABLE_FILES:[&str;2]=[\".opencodex-native-main.claim.sqlite\",\".opencodex-native-main.owner.sqlite\"];\n");
    partial += &between(&phase, "pub(super) fn invalid()", "pub(super) fn digest(")
        .replace("pub(super) fn invalid()", "fn invalid()");
    partial += &between(
        &tree,
        "#[derive(PartialEq, Eq)]\nstruct Entry",
        "/// The actual working directory handle",
    );
    partial += &between(&tree, "fn check(end:", "fn open(path:")
        .replace("super::bounded::QueryError", "QueryError");
    partial += &between(&tree, "fn id(file:", "mod streams;");
    partial += &between(&tree, "fn names(path:", "fn scan(");
    partial += &between(
        &anchored,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum Kind",
        "/// All fields are actual native objects",
    );
    partial += &between(&anchored, "fn open_node(", "impl AnchoredTree {");
    partial+="\n#[cfg(test)]\nmod tests {include!(concat!(env!(\"CARGO_MANIFEST_DIR\"),\"/src/partial_collect_tests.rs\"));}\n";
    fs::write(out.join("native_partial_collect.rs"), partial).unwrap();
}
