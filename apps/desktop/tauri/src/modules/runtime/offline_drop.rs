//! Bounded HTML file-drop reception. The window keeps HTML model sorting;
//! only an explicitly supplied archive is copied into an owned temporary file.
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::archive;
use crate::errors::{AppError, AppResult};
use crate::infrastructure::platform::OpenOptionsModeExt;

pub const CHUNK_BYTES: usize = 1024 * 1024;
#[derive(Clone, Default)]
pub struct SharedOfflineDrop(pub Arc<Mutex<OfflineDrop>>);
#[derive(Default)]
pub struct OfflineDrop {
    active: Option<Pending>,
}
struct Pending {
    id: String,
    file: std::fs::File,
    _directory: tempfile::TempDir,
    path: PathBuf,
    length: u64,
    received: u64,
}
fn error(detail: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "receive offline package".into(),
        detail: detail.into(),
    }
}
impl OfflineDrop {
    pub fn begin(&mut self, root: &Path, name: &str, length: u64) -> AppResult<String> {
        if name.len() > 240 || name.chars().any(char::is_control) {
            return Err(error("离线包文件名无效。"));
        }
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
            || name.contains(['/', '\\'])
            || archive::declared_version(Path::new(name)).is_err()
        {
            return Err(error("请选择单个官方 opencodex-<版本>.tgz 离线包。"));
        }
        if length == 0 || length > archive::MAX_TOTAL_BYTES {
            return Err(error("离线包为空或超过允许体积。"));
        }
        let cache = root.join("cache/runtime-offline-drop");
        crate::modules::backup::safety::check_path(root, &cache, true)?;
        std::fs::create_dir_all(&cache).map_err(|e| error(e.to_string()))?;
        let directory = tempfile::Builder::new()
            .prefix("package-")
            .tempdir_in(&cache)
            .map_err(|e| error(e.to_string()))?;
        let path = directory.path().join(name);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .create_mode(0o600)
            .open(&path)
            .map_err(|e| error(e.to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();
        self.active = Some(Pending {
            id: id.clone(),
            file,
            _directory: directory,
            path,
            length,
            received: 0,
        });
        Ok(id)
    }
    pub fn append(&mut self, id: &str, offset: u64, bytes: &[u8]) -> AppResult<u64> {
        let item = self
            .active
            .as_mut()
            .filter(|p| p.id == id)
            .ok_or_else(|| error("离线包读取已取消，请重新选择。"))?;
        if item.received != offset
            || bytes.is_empty()
            || bytes.len() > CHUNK_BYTES
            || item
                .received
                .checked_add(bytes.len() as u64)
                .is_none_or(|n| n > item.length)
        {
            return Err(error("离线包分块或顺序无效，未继续读取。"));
        }
        item.file
            .write_all(bytes)
            .map_err(|e| error(e.to_string()))?;
        item.received += bytes.len() as u64;
        Ok(item.received)
    }
    pub fn finish(&mut self, id: &str) -> AppResult<PathBuf> {
        let item = self
            .active
            .as_mut()
            .filter(|p| p.id == id)
            .ok_or_else(|| error("离线包读取已取消，请重新选择。"))?;
        if item.received != item.length {
            return Err(error("离线包尚未读取完整。"));
        }
        item.file
            .flush()
            .and_then(|_| item.file.sync_all())
            .map_err(|e| error(e.to_string()))?;
        Ok(item.path.clone())
    }
    pub fn clear(&mut self, id: &str) {
        if self.active.as_ref().is_some_and(|p| p.id == id) {
            self.active = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_names_and_sizes_before_creating_any_cache() {
        let root = tempfile::tempdir().unwrap();
        let mut drop = OfflineDrop::default();
        for name in [
            "../opencodex-2.91.0.tgz",
            "/opencodex-2.91.0.tgz",
            "C:\\opencodex-2.91.0.tgz",
            "package.tgz",
        ] {
            assert!(drop.begin(root.path(), name, 1).is_err());
        }
        for size in [0, archive::MAX_TOTAL_BYTES + 1] {
            assert!(drop
                .begin(root.path(), "opencodex-2.91.0.tgz", size)
                .is_err());
        }
        assert!(!root.path().join("cache").exists());
    }
    #[test]
    fn incomplete_or_replayed_chunks_cannot_be_finished_and_clear_only_owns_the_copy() {
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join("opencodex-2.91.0.tgz");
        std::fs::write(&original, b"source remains unchanged").unwrap();
        let mut drop = OfflineDrop::default();
        let id = drop.begin(root.path(), "opencodex-2.91.0.tgz", 4).unwrap();
        assert!(drop.finish(&id).is_err());
        assert_eq!(drop.append(&id, 0, b"12").unwrap(), 2);
        assert!(drop.append(&id, 0, b"34").is_err());
        assert!(drop.append(&id, 2, b"345").is_err());
        assert!(drop.append(&id, 2, &vec![1; CHUNK_BYTES + 1]).is_err());
        assert_eq!(drop.append(&id, 2, b"34").unwrap(), 4);
        let path = drop.finish(&id).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"1234");
        assert!(
            archive::inspect(&path, None).is_err(),
            "reception does not certify arbitrary bytes as an official package"
        );
        drop.clear("another-request");
        assert!(path.exists());
        drop.clear(&id);
        assert!(!path.exists());
        assert_eq!(
            std::fs::read(&original).unwrap(),
            b"source remains unchanged"
        );
    }
    #[test]
    fn replacing_a_selection_invalidates_old_lease_and_drops_its_temporary_file() {
        let root = tempfile::tempdir().unwrap();
        let mut drop = OfflineDrop::default();
        let a = drop.begin(root.path(), "opencodex-2.91.0.tgz", 1).unwrap();
        drop.append(&a, 0, b"a").unwrap();
        let old = drop.finish(&a).unwrap();
        let b = drop.begin(root.path(), "opencodex-2.93.0.tgz", 1).unwrap();
        assert_ne!(a, b);
        assert!(!old.exists());
        assert!(drop.append(&a, 0, b"b").is_err());
        assert!(drop.finish(&a).is_err());
        drop.clear(&a);
        drop.append(&b, 0, b"b").unwrap();
        assert!(drop.finish(&b).unwrap().exists());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_cache_without_writing_outside_the_active_root() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("cache")).unwrap();
        assert!(OfflineDrop::default()
            .begin(root.path(), "opencodex-2.91.0.tgz", 1)
            .is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }
}

#[cfg(test)]
mod valid_package_tests {
    use super::*;
    #[test]
    fn received_archive_is_byte_identical_and_uses_the_same_official_inspector() {
        let root = tempfile::tempdir().unwrap();
        let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut tar = tar::Builder::new(gzip);
        for (name, body) in [
            (
                "package/package.json",
                br#"{"name":"@bitkyc08/opencodex","version":"2.91.0","bin":{"ocx":"bin/ocx.mjs"}}"#
                    .as_slice(),
            ),
            ("package/bin/ocx.mjs", b"console.log('2.91.0');".as_slice()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, name, body).unwrap();
        }
        let bytes = tar.into_inner().unwrap().finish().unwrap();
        let source = root.path().join("opencodex-2.91.0.tgz");
        std::fs::write(&source, &bytes).unwrap();
        let original = archive::inspect(&source, None).unwrap();
        let mut receiver = OfflineDrop::default();
        let id = receiver
            .begin(root.path(), "opencodex-2.91.0.tgz", bytes.len() as u64)
            .unwrap();
        let middle = bytes.len() / 2;
        receiver.append(&id, 0, &bytes[..middle]).unwrap();
        receiver
            .append(&id, middle as u64, &bytes[middle..])
            .unwrap();
        let path = receiver.finish(&id).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let observed = archive::inspect(&path, None).unwrap();
        assert_eq!(observed, original);
        receiver.clear(&id);
        assert!(!path.exists());
        assert!(source.exists());
    }
}
