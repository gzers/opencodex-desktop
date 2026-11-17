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
}
