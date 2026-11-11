//! Bounded complete image inventory for publication, reuse, repair and removal.
//! It is separate from high-frequency current-selection freshness checks.
use crate::{
    errors::{AppError, AppResult},
    infrastructure::hash,
    modules::backup::safety,
};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
const MAX_ENTRIES: usize = 100_000;
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;
fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_image_unconfirmed".into(),
        detail: "运行版本文件树无法核对，原目录保留".into(),
    }
}
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Entry {
    Directory,
    File {
        bytes: u64,
        sha256: String,
        executable: bool,
    },
}
/// Excludes only the root image manifest to avoid a self-referential hash. The
/// manifest itself is bound by CurrentSelection or the exact activation receipt.
pub fn content_digest(root: &Path) -> AppResult<String> {
    if !super::program_layout::clean_absolute(root) || !safety::inspect(root)?.is_dir() {
        return Err(fail());
    }
    let mut rows = BTreeMap::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut count = 0usize;
    let mut total = 0u64;
    while let Some((directory, depth)) = stack.pop() {
        if depth > 64 {
            return Err(fail());
        }
        safety::check_path(root, &directory, false)?;
        let directory_before = fs::metadata(&directory).map_err(|_| fail())?;
        for item in fs::read_dir(&directory).map_err(|_| fail())? {
            let item = item.map_err(|_| fail())?;
            count += 1;
            if count > MAX_ENTRIES {
                return Err(fail());
            }
            let path = item.path();
            let meta = safety::inspect(&path)?;
            let relative = path.strip_prefix(root).map_err(|_| fail())?;
            let name = relative
                .to_str()
                .ok_or_else(fail)?
                .replace(char::from(92), "/");
            if depth == 0 && name == super::program_selection::IMAGE_MANIFEST {
                if !meta.is_file() {
                    return Err(fail());
                }
                continue;
            }
            if rows.contains_key(&name) {
                return Err(fail());
            }
            let entry = if meta.is_dir() {
                stack.push((path, depth + 1));
                Entry::Directory
            } else {
                let file = safety::open_file(&path, false)?;
                safety::require_single_link(&file)?;
                let before = file.metadata().map_err(|_| fail())?;
                if before.len() > MAX_FILE {
                    return Err(fail());
                }
                total = total
                    .checked_add(before.len())
                    .filter(|n| *n <= MAX_BYTES)
                    .ok_or_else(fail)?;
                let mut read = file.take(MAX_FILE + 1);
                let mut digest = sha2::Sha256::new();
                let mut buffer = [0u8; 64 * 1024];
                let mut bytes = 0u64;
                use sha2::Digest;
                loop {
                    let n = read.read(&mut buffer).map_err(|_| fail())?;
                    if n == 0 {
                        break;
                    }
                    bytes += n as u64;
                    if bytes > before.len() {
                        return Err(fail());
                    }
                    digest.update(&buffer[..n]);
                }
                let after = read.get_ref().metadata().map_err(|_| fail())?;
                safety::require_single_link(read.get_ref())?;
                if bytes != before.len()
                    || after.len() != before.len()
                    || after.modified().map_err(|_| fail())?
                        != before.modified().map_err(|_| fail())?
                {
                    return Err(fail());
                }
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    if after.permissions().mode() != before.permissions().mode() {
                        return Err(fail());
                    }
                    before.permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;
                Entry::File {
                    bytes,
                    sha256: format!("{:x}", digest.finalize()),
                    executable,
                }
            };
            rows.insert(name, entry);
        }
        let directory_after = fs::metadata(&directory).map_err(|_| fail())?;
        if directory_before.modified().map_err(|_| fail())?
            != directory_after.modified().map_err(|_| fail())?
        {
            return Err(fail());
        }
    }
    if rows.is_empty() {
        return Err(fail());
    }
    let bytes = serde_json::to_vec(&rows).map_err(|_| fail())?;
    Ok(hash::sha256_hex(&bytes))
}
pub fn verify(selected: &super::program_selection::SelectedProgram) -> AppResult<()> {
    if content_digest(&selected.paths.version_root)? != selected.image.tree_sha256 {
        return Err(fail());
    }
    let manifest_path = selected
        .paths
        .version_root
        .join(super::program_selection::IMAGE_MANIFEST);
    safety::check_path(&selected.paths.version_root, &manifest_path, false)?;
    let file = safety::open_file(&manifest_path, false)?;
    safety::require_single_link(&file)?;
    let mut manifest = Vec::new();
    file.take(64 * 1024 + 1)
        .read_to_end(&mut manifest)
        .map_err(|_| fail())?;
    if manifest.len() > 64 * 1024 {
        return Err(fail());
    }
    if hash::sha256_hex(&manifest) != selected.selection.image_sha256 {
        return Err(fail());
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::data_root::layout::Platform;
    use crate::modules::runtime::program_selection::{self, tests::Fixture};
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn action_time_inventory_detects_noncritical_changes_without_reading_other_directories() {
        for case in [
            "added",
            "empty_dir",
            "changed",
            "executable",
            "symlink",
            "hardlink",
        ] {
            let f = Fixture::new();
            let selected = program_selection::read(&f.root, Platform::Macos, "darwin-arm64")
                .unwrap()
                .unwrap();
            verify(&selected).unwrap();
            let extra = f.paths.version_root.join("added-file");
            match case {
                "added" => fs::write(extra, b"user file that must not be deleted").unwrap(),
                "empty_dir" => {
                    fs::create_dir(f.paths.version_root.join("empty-user-directory")).unwrap()
                }
                "changed" => fs::write(&f.paths.npm, b"different npm module").unwrap(),
                "executable" => {
                    fs::set_permissions(&f.paths.npm, fs::Permissions::from_mode(0o755)).unwrap()
                }
                "symlink" => {
                    std::os::unix::fs::symlink(f.root.join("never-read-missing"), extra).unwrap()
                }
                "hardlink" => fs::hard_link(&f.paths.npm, extra).unwrap(),
                _ => unreachable!(),
            }
            assert!(verify(&selected).is_err(), "{case}");
            assert!(f.paths.version_root.exists());
        }
    }
    #[test]
    fn seal_excludes_only_root_manifest_and_still_binds_it_through_selection() {
        let f = Fixture::new();
        let selected = program_selection::read(&f.root, Platform::Macos, "darwin-arm64")
            .unwrap()
            .unwrap();
        let before = content_digest(&f.paths.version_root).unwrap();
        fs::write(
            f.paths.version_root.join(program_selection::IMAGE_MANIFEST),
            b"changed manifest",
        )
        .unwrap();
        assert_eq!(content_digest(&f.paths.version_root).unwrap(), before);
        assert!(verify(&selected).is_err());
        fs::write(
            f.paths.package_root.join(program_selection::IMAGE_MANIFEST),
            b"nested manifest is ordinary content",
        )
        .unwrap();
        assert_ne!(content_digest(&f.paths.version_root).unwrap(), before);
    }
    #[test]
    fn rejects_sparse_oversize_without_allocating_payload_and_preserves_file() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        let path = root.join("oversize");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_FILE + 1).unwrap();
        assert!(content_digest(&root).is_err());
        assert_eq!(fs::metadata(path).unwrap().len(), MAX_FILE + 1);
    }
}
