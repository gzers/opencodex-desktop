//! Bounded native complete-tree images and retained original directory roots.
//! No native exit/Closed/successor proof is issued here. Original content stays
//! in memory and exact original objects; traces expose hashes only.
use super::{
    bounded::{native_stamp, Policy, SnapshotData},
    phase::invalid,
};
use crate::infrastructure::{
    hash,
    windows_official_runtime024::{
        create_private_directory, rename_original_handle, validate_local_path, FileIdentity,
        PinnedDirectory, PrivateRecord,
    },
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
    time::Instant,
};
mod anchored;
#[cfg(test)]
pub(super) use anchored::test_plan as r6_test_plan;
mod artifacts;
mod locks;
pub(super) use artifacts::ArtifactFrame;
mod working;
pub(super) use working::{SnapshotPhase, WorkingArchive, WorkingReference};
#[derive(PartialEq, Eq)]
struct Entry {
    relative: String,
    directory: bool,
    id: FileIdentity,
    security: SnapshotData,
    bytes: Vec<u8>,
    children: Vec<String>,
}
/// The actual working directory handle survives retirement. A pathname or a
/// completed restore record cannot substitute its exact full-tree generation.
struct RetiredWorking {
    root: File,
    parent: PinnedDirectory,
    path: PathBuf,
    entries: Vec<Entry>,
}
/// These are actual native objects, not serialized image capabilities. The
/// original root is opened DELETE-capable before any mutation and held until
/// restoration; path/JSON/PID reopening never becomes root-move authority.
pub(super) struct RootTree {
    parent: PinnedDirectory,
    parent_security: SnapshotData,
    root: Option<File>,
    original: PathBuf,
    location: PathBuf,
    entries: Vec<Entry>,
    custody: Option<PinnedDirectory>,
    working: Option<Vec<Entry>>,
    retired_work: Option<RetiredWorking>,
    original_handle_released: bool,
    poisoned: bool,
}
fn check(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn failure(e: super::bounded::QueryError) -> io::Error {
    io::Error::other(e)
}
fn open(path: &Path, dir: bool, root: bool) -> io::Result<File> {
    validate_local_path(path)?;
    OpenOptions::new()
        .access_mode(0x80020000 | if root { 0x10000 } else { 0 })
        .share_mode(1)
        .custom_flags(0x200000 | if dir { 0x2000000 } else { 0 })
        .open(path)
}
fn id(file: &File, dir: bool) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || i.dwFileAttributes & 0x400 != 0
        || (i.dwFileAttributes & 0x10 != 0) != dir
        || !dir && i.nNumberOfLinks != 1
    {
        return Err(invalid());
    }
    Ok(FileIdentity {
        volume: i.dwVolumeSerialNumber,
        index: ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64,
    })
}
pub(super) fn streams(path: &Path, dir: bool) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        Storage::FileSystem::{
            FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
            WIN32_FIND_STREAM_DATA,
        },
    };
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut data = WIN32_FIND_STREAM_DATA::default();
    let h = unsafe {
        FindFirstStreamW(
            name.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if h == INVALID_HANDLE_VALUE {
        let e = io::Error::last_os_error();
        return if dir && e.raw_os_error() == Some(38) {
            Ok(())
        } else {
            Err(e)
        };
    }
    struct Search(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.0);
            }
        }
    }
    let _search = Search(h);
    let n = data
        .cStreamName
        .iter()
        .position(|c| *c == 0)
        .ok_or_else(invalid)?;
    if String::from_utf16(&data.cStreamName[..n]).map_err(|_| invalid())? != "::$DATA" {
        return Err(invalid());
    }
    if unsafe { FindNextStreamW(h, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast()) } != 0
        || io::Error::last_os_error().raw_os_error() != Some(38)
    {
        return Err(invalid());
    }
    Ok(())
}
fn names(path: &Path) -> io::Result<Vec<String>> {
    let mut n = std::fs::read_dir(path)?
        .map(|e| e.and_then(|e| e.file_name().into_string().map_err(|_| invalid())))
        .collect::<io::Result<Vec<_>>>()?;
    n.sort();
    if n.len() > 512 || n.windows(2).any(|w| w[0].eq_ignore_ascii_case(&w[1])) {
        return Err(invalid());
    }
    for s in &n {
        validate_local_path(&path.join(s))?;
    }
    Ok(n)
}
fn scan(
    path: &Path,
    file: &mut File,
    relative: &str,
    dir: bool,
    entries: &mut Vec<Entry>,
    total: &mut usize,
    end: Instant,
) -> io::Result<()> {
    check(end)?;
    if entries.len() >= 512 {
        return Err(invalid());
    }
    let original_id = id(file, dir)?;
    streams(path, dir)?;
    let security = native_stamp(file, end).map_err(failure)?;
    let children = if dir { names(path)? } else { Vec::new() };
    let mut bytes = Vec::new();
    if !dir {
        if file.metadata()?.len() > 2 * 1024 * 1024 {
            return Err(invalid());
        }
        file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(invalid());
        }
        *total = total.checked_add(bytes.len()).ok_or_else(invalid)?;
        if *total > 32 * 1024 * 1024 {
            return Err(invalid());
        }
    }
    let index = entries.len();
    entries.push(Entry {
        relative: relative.into(),
        directory: dir,
        id: original_id,
        security,
        bytes,
        children: children.clone(),
    });
    for child in children {
        let p = path.join(&child);
        let meta = std::fs::symlink_metadata(&p)?;
        let mut f = open(&p, meta.is_dir(), false)?;
        let r = if relative.is_empty() {
            child
        } else {
            format!("{relative}/{child}")
        };
        scan(&p, &mut f, &r, meta.is_dir(), entries, total, end)?;
    }
    if id(file, dir)? != original_id
        || native_stamp(file, end).map_err(failure)? != entries[index].security
        || dir && names(path)? != entries[index].children
    {
        return Err(invalid());
    }
    check(end)
}
fn image(path: &Path, file: &mut File, end: Instant) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    scan(path, file, "", true, &mut entries, &mut 0, end)?;
    Ok(entries)
}
fn parent_stamp(parent: &Path, end: Instant) -> io::Result<SnapshotData> {
    native_stamp(&open(parent, true, false)?, end).map_err(failure)
}
impl RootTree {
    pub(super) fn capture(path: &Path, end: Instant) -> io::Result<Self> {
        check(end)?;
        if ![Some(".codex"), Some(".grok")].contains(&path.file_name().and_then(|n| n.to_str())) {
            return Err(invalid());
        }
        let parent = PinnedDirectory::open(path.parent().ok_or_else(invalid)?)?;
        let parent_security = parent_stamp(parent.path(), end)?;
        let (root, entries) = match std::fs::symlink_metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => (None, Vec::new()),
            Err(e) => return Err(e),
            Ok(m) => {
                if !m.is_dir() {
                    return Err(invalid());
                }
                let mut root = open(path, true, true)?;
                let entries = image(path, &mut root, end)?;
                if image(path, &mut root, end)? != entries {
                    return Err(invalid());
                }
                (Some(root), entries)
            }
        };
        parent.recheck()?;
        if parent_stamp(parent.path(), end)? != parent_security {
            return Err(invalid());
        }
        if root.is_none() && std::fs::symlink_metadata(path).is_ok() {
            return Err(invalid());
        }
        let mut captured = Self {
            parent,
            parent_security,
            root,
            original: path.into(),
            location: path.into(),
            entries,
            custody: None,
            working: None,
            retired_work: None,
            original_handle_released: false,
            poisoned: false,
        };
        captured.verify_stable_release(end)?;
        captured.check_original(end)?;
        Ok(captured)
    }
    fn check_original(&mut self, end: Instant) -> io::Result<()> {
        check(end)?;
        self.parent.recheck()?;
        if self.poisoned
            || self.original_handle_released
            || parent_stamp(self.parent.path(), end)? != self.parent_security
        {
            return Err(invalid());
        }
        match &mut self.root {
            Some(root) => {
                if image(&self.location, root, end)? != self.entries {
                    return Err(invalid());
                }
            }
            None if self.working.is_some() && self.custody.is_some() => {}
            None => match std::fs::symlink_metadata(&self.original) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                _ => return Err(invalid()),
            },
        }
        self.verify_retired(end)
    }
    fn verify_retired(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            check(end)?;
            if let Some(retired) = &mut self.retired_work {
                retired.parent.recheck()?;
                if retired.path.parent() != Some(retired.parent.path()) {
                    return Err(invalid());
                }
                if image(&retired.path, &mut retired.root, end)? != retired.entries {
                    return Err(invalid());
                }
                retired.parent.recheck()?;
            }
            check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Serializable observations only. The retained tree remains the authority;
    /// these values cannot reconstruct it or authorize restoration.
    pub(super) fn manifest_nodes(&mut self, end: Instant) -> io::Result<Vec<serde_json::Value>> {
        self.check_original(end)?;
        let root = self
            .original
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        let parent = serde_json::json!({"volume":self.parent.identity().volume,"fileId":self.parent.identity().index.to_string()});
        if self.root.is_none() {
            return Ok(vec![
                serde_json::json!({"relative":root,"kind":"Absent","identity":null,"parentIdentity":parent,"securityProjectionSnapshot":null,"parentSecurityProjectionSnapshot":self.parent_security,"length":null,"contentDigest":null,"children":[],"custodyIdentity":null}),
            ]);
        }
        self.entries.iter().map(|entry| {
            let (parent_id, parent_security) = if entry.relative.is_empty() {
                (parent.clone(), &self.parent_security)
            } else {
                let relative = entry.relative.rsplit_once('/').map_or("", |(p, _)| p);
                let p = self.entries.iter().find(|e|e.relative==relative && e.directory).ok_or_else(invalid)?;
                (serde_json::json!({"volume":p.id.volume,"fileId":p.id.index.to_string()}), &p.security)
            };
            let identity = serde_json::json!({"volume":entry.id.volume,"fileId":entry.id.index.to_string()});
            Ok(serde_json::json!({"relative":if entry.relative.is_empty(){root.to_string()}else{format!("{root}/{}",entry.relative)},"kind":if entry.directory{"Directory"}else{"File"},"identity":identity,"parentIdentity":parent_id,"securityProjectionSnapshot":entry.security,"parentSecurityProjectionSnapshot":parent_security,"length":if entry.directory{None}else{Some(entry.bytes.len())},"contentDigest":if entry.directory{None}else{Some(hash::sha256_hex(&entry.bytes))},"children":entry.children,"custodyIdentity":identity}))
        }).collect()
    }
    pub(super) fn move_from_manifest(
        &mut self,
        intent: &mut super::v2::MoveIntent<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let nodes = self.manifest_nodes(end)?;
        let custody = intent.authorize(&self.original, &nodes, end)?;
        self.move_exact(&custody, end)
    }
    /// Write the bounded native root/descendant image and exact target intention
    /// privately BEFORE moving. Data in this record is not restoration authority.
    pub(super) fn move_to_custody(
        &mut self,
        custody: &Path,
        intent_path: &Path,
        end: Instant,
    ) -> io::Result<()> {
        self.check_original(end)?;
        if self.custody.is_some() || self.retired_work.is_some() {
            return Err(invalid());
        }
        let pin = PinnedDirectory::open_private(custody)?;
        if pin.identity().volume != self.parent.identity().volume
            || pin.identity() == self.parent.identity()
        {
            return Err(invalid());
        }
        let target = custody.join(self.original.file_name().ok_or_else(invalid)?);
        require_absent(&target)?;
        let nodes=self.entries.iter().map(|e|serde_json::json!({"relative":e.relative,"directory":e.directory,"volume":e.id.volume,"fileId":e.id.index.to_string(),"contentDigest":hash::sha256_hex(&e.bytes),"children":e.children,"securityProjectionSnapshot":e.security})).collect::<Vec<_>>();
        let policy = Policy::fixed();
        let intent = serde_json::json!({"schema":"windows-client-originals-v2","documentKind":"root-move-intent","securityPolicy":policy,"securityPolicyDigest":policy.digest()?,"originalAbsent":self.root.is_none(),"original":self.original,"target":target,"custodyVolume":pin.identity().volume,"custodyFileId":pin.identity().index.to_string(),"nodes":nodes});
        let bytes = serde_json::to_vec(&intent).map_err(|_| invalid())?;
        let mut record = PrivateRecord::create(intent_path, &bytes, 1024 * 1024)?;
        if record.read()? != bytes {
            return Err(invalid());
        }
        self.move_exact(custody, end)?;
        drop(record);
        Ok(())
    }
    fn move_exact(&mut self, custody: &Path, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        if self.custody.is_some()
            || self.retired_work.is_some()
            || custody.parent() != Some(self.parent.path())
        {
            return Err(invalid());
        }
        let pin = PinnedDirectory::open_private(custody)?;
        if pin.identity().volume != self.parent.identity().volume
            || pin.identity() == self.parent.identity()
        {
            return Err(invalid());
        }
        let target = custody.join(self.original.file_name().ok_or_else(invalid)?);
        require_absent(&target)?;
        check(end)?;
        self.poisoned = true;
        if let Some(root) = &self.root {
            rename_original_handle(root, &target)?;
            self.location = target;
        }
        self.custody = Some(pin);
        if let Some(root) = &mut self.root {
            if image(&self.location, root, end)? != self.entries {
                return Err(invalid());
            }
        } else {
            require_absent(&self.original)?;
        }
        check(end)?;
        self.poisoned = false;
        Ok(())
    }
    pub(super) fn create_working_copy(&mut self, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        if self.custody.is_none() || self.working.is_some() {
            return Err(invalid());
        }
        self.poisoned = true;
        let root = create_private_directory(&self.original)?;
        drop(root);
        for entry in self.entries.iter().skip(1) {
            check(end)?;
            let target = self.original.join(&entry.relative);
            if entry.directory {
                std::fs::create_dir(&target)?;
            } else {
                use std::io::Write;
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(target)?;
                file.write_all(&entry.bytes)?;
                file.sync_all()?;
            }
        }
        let mut file = open(&self.original, true, true)?;
        let observed = image(&self.original, &mut file, end)?;
        if self.root.is_none() {
            if observed.len() != 1 || !observed[0].children.is_empty() {
                return Err(invalid());
            }
        } else if observed.len() != self.entries.len()
            || observed.iter().zip(&self.entries).any(|(w, o)| {
                w.relative != o.relative
                    || w.bytes != o.bytes
                    || w.directory != o.directory
                    || w.children != o.children
                    || w.id == o.id
            })
        {
            return Err(invalid());
        }
        self.working = Some(observed);
        self.poisoned = false;
        Ok(())
    }
    pub(super) fn verify_moved(&mut self, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        let custody = self.custody.as_ref().ok_or_else(invalid)?;
        custody.recheck()?;
        if self.working.is_some()
            || self.root.is_some()
                && self.location
                    != custody
                        .path()
                        .join(self.original.file_name().ok_or_else(invalid)?)
        {
            return Err(invalid());
        }
        require_absent(&self.original)
    }
    pub(super) fn verify_working(&mut self, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        self.custody.as_ref().ok_or_else(invalid)?.recheck()?;
        let expected = self.working.as_ref().ok_or_else(invalid)?;
        let mut working = open(&self.original, true, true)?;
        if image(&self.original, &mut working, end)? != *expected {
            return Err(invalid());
        }
        check(end)
    }
    pub(super) fn verify_restored(&mut self, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        if self.custody.is_some() || self.working.is_some() || self.location != self.original {
            return Err(invalid());
        }
        self.verify_stable_release(end)?;
        self.check_original(end)?;
        check(end)
    }
    /// Only the retained complete native close driver may call this AFTER its
    /// original-history archive has been sealed and linked by canonical Closed.
    pub(super) fn release_closed_original_handle(&mut self, end: Instant) -> io::Result<()> {
        self.verify_restored(end)?;
        self.original_handle_released = true;
        drop(self.root.take());
        self.check_closed_retention(end)
    }
    fn check_closed_retention(&mut self, end: Instant) -> io::Result<()> {
        check(end)?;
        self.parent.recheck()?;
        if self.poisoned
            || self.custody.is_some()
            || self.working.is_some()
            || self.location != self.original
            || parent_stamp(self.parent.path(), end)? != self.parent_security
        {
            return Err(invalid());
        }
        // Never query the later live client root as an old-history baseline.
        self.verify_retired(end)
    }
    fn verify_stable_release(&self, end: Instant) -> io::Result<()> {
        if self.original.file_name().and_then(|s| s.to_str()) == Some(".codex")
            && self.root.is_some()
        {
            locks::prove(
                &self.original,
                self.root.as_ref().ok_or_else(invalid)?,
                &self.entries,
                end,
            )?;
        }
        check(end)
    }
    pub(super) fn restore_original(&mut self, end: Instant) -> io::Result<()> {
        self.check_original(end)?;
        let custody = self.custody.as_ref().ok_or_else(invalid)?;
        custody.recheck()?;
        if let Some(expected) = &self.working {
            let mut working = open(&self.original, true, true)?;
            if image(&self.original, &mut working, end)? != *expected {
                return Err(invalid());
            }
            let retired = custody.path().join(format!(
                "retired-working-{}",
                self.original
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(invalid)?
            ));
            let retired_parent = PinnedDirectory::open_private(custody.path())?;
            self.poisoned = true;
            rename_original_handle(&working, &retired)?;
            // Install the original handle immediately after rename and before
            // further fallible IO. Failure preserves both custody generations.
            self.retired_work = Some(RetiredWorking {
                root: working,
                parent: retired_parent,
                path: retired,
                entries: self.working.take().ok_or_else(invalid)?,
            });
            self.verify_retired(end)?;
        }
        check(end)?;
        self.poisoned = true;
        if let Some(root) = &mut self.root {
            rename_original_handle(root, &self.original)?;
            self.location = self.original.clone();
            if image(&self.original, root, end)? != self.entries {
                return Err(invalid());
            }
        } else {
            require_absent(&self.original)?;
        }
        check(end)?;
        self.poisoned = false;
        self.custody = None;
        Ok(())
    }
    pub(super) fn diagnostic(&self) -> serde_json::Value {
        serde_json::json!({"scopeId":"windows-bounded-access-v1","originalAbsent":self.root.is_none(),"nodes":self.entries.len(),"currentLocation":self.location,"poisoned":self.poisoned,"auditSaclAssurance":"excluded-unknown","fullSecurityProven":false,"ClosedEligibility":false})
    }
    pub(super) fn retained_custody_names(&self, custody: &Path) -> io::Result<Vec<String>> {
        let mut result = Vec::new();
        if self.location != self.original {
            if self.location.parent() != Some(custody) {
                return Err(invalid());
            }
            result.push(
                self.location
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(invalid)?
                    .into(),
            );
        }
        if let Some(retired) = &self.retired_work {
            if retired.path.parent() != Some(custody) {
                return Err(invalid());
            }
            result.push(
                retired
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(invalid)?
                    .into(),
            );
        }
        Ok(result)
    }
}
fn require_absent(path: &Path) -> io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(_) => Err(invalid()),
    }
}
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    pub(crate) struct Fixture(Option<tempfile::TempDir>);
    impl Fixture {
        pub(crate) fn new() -> Self {
            Self(Some(tempfile::tempdir().unwrap()))
        }
        pub(crate) fn path(&self) -> &Path {
            self.0.as_ref().unwrap().path()
        }
        pub(crate) fn retain(&mut self) {
            let path = self.0.take().unwrap().keep();
            println!("r4-retained-synthetic-fixture={}", path.display());
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if std::thread::panicking() && self.0.is_some() {
                self.retain();
            }
        }
    }
    fn end() -> Instant {
        Instant::now() + std::time::Duration::from_secs(10)
    }
    #[test]
    fn wl2_r4_retired_working_external_edit_vetoes_restored_readback() {
        let mut temp = Fixture::new();
        let profile = create_private_directory(&temp.path().join("Profile")).unwrap();
        let codex = profile.path().join(".codex");
        std::fs::create_dir(&codex).unwrap();
        std::fs::write(codex.join("config.toml"), b"synthetic original config").unwrap();
        let custody = create_private_directory(&profile.path().join("task-custody")).unwrap();
        let journal = create_private_directory(&temp.path().join("D")).unwrap();
        let mut original = RootTree::capture(&codex, end()).unwrap();
        let original_nodes = original.manifest_nodes(end()).unwrap();
        original
            .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
            .unwrap();
        original.create_working_copy(end()).unwrap();
        original.restore_original(end()).unwrap();
        original.verify_restored(end()).unwrap();
        assert_eq!(original.manifest_nodes(end()).unwrap(), original_nodes);
        let foreign = custody.path().join("retired-working-.codex/foreign.new");
        std::fs::write(&foreign, b"synthetic unknown retirement drift").unwrap();
        assert!(
            original.verify_restored(end()).is_err(),
            "retired working generation cannot disappear from final native readback"
        );
        assert!(original.poisoned);
        assert!(original.verify_restored(end()).is_err());
        assert_eq!(
            std::fs::read(codex.join("config.toml")).unwrap(),
            b"synthetic original config"
        );
        assert!(foreign.exists());
        drop(original);
        drop(journal);
        drop(custody);
        drop(profile);
        println!("retired working unknown node refuses restored readback and poisons retained tree; exact original restored, foreign scene retained, no Closed/native lifecycle eligibility");
        temp.retain();
    }
    #[test]
    fn wl2_r4_native_whole_tree_custody_workcopy_restore_same_ids_and8projections() {
        let temp = Fixture::new();
        let profile = temp.path().join("Profile");
        std::fs::create_dir(&profile).unwrap();
        let codex = profile.join(".codex");
        std::fs::create_dir(&codex).unwrap();
        std::fs::create_dir(codex.join("nested")).unwrap();
        std::fs::write(codex.join("config.toml"), b"synthetic config").unwrap();
        std::fs::write(
            codex.join("nested/auth.json"),
            b"synthetic credential-free auth",
        )
        .unwrap();
        let custody = create_private_directory(&profile.join("task-custody")).unwrap();
        let journal = create_private_directory(&temp.path().join("D")).unwrap();
        println!("tree-stage capture existing Codex");
        let mut original = RootTree::capture(&codex, end()).unwrap();
        assert_eq!(original.entries.len(), 4);
        println!("tree-stage flush intention +move");
        original
            .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
            .unwrap();
        assert!(!codex.exists());
        println!("tree-stage working copy");
        original.create_working_copy(end()).unwrap();
        println!("tree-stage exact original restore");
        original.restore_original(end()).unwrap();
        original.check_original(end()).unwrap();
        assert_eq!(
            std::fs::read(codex.join("config.toml")).unwrap(),
            b"synthetic config"
        );
        println!("native whole-tree existing root restored bytes+all descendant IDs+8projections {}; original objects not copied; production Closed=false",original.diagnostic());
        drop(original);
        drop(journal);
        drop(custody);
    }
    #[test]
    fn wl2_r4_native_tree_absence_and_external_edit_keep_originals_required() {
        let mut temp = Fixture::new();
        let profile = temp.path().join("Profile");
        std::fs::create_dir(&profile).unwrap();
        let grok = profile.join(".grok");
        let mut absent = RootTree::capture(&grok, end()).unwrap();
        absent.check_original(end()).unwrap();
        std::fs::create_dir(&grok).unwrap();
        assert!(absent.check_original(end()).is_err());
        drop(absent);
        std::fs::write(grok.join("settings.json"), b"original synthetic").unwrap();
        let custody = create_private_directory(&profile.join("task-custody")).unwrap();
        let journal = create_private_directory(&temp.path().join("D")).unwrap();
        let mut original = RootTree::capture(&grok, end()).unwrap();
        original
            .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
            .unwrap();
        original.create_working_copy(end()).unwrap();
        std::fs::write(grok.join("foreign.new"), b"external synthetic edit").unwrap();
        assert!(original.restore_original(end()).is_err());
        assert_eq!(
            std::fs::read(grok.join("foreign.new")).unwrap(),
            b"external synthetic edit"
        );
        assert!(original.location.join("settings.json").exists());
        println!("native whole-tree external unknown generation refused; exact original custody and external edit retained for external archival; Closed=false");
        drop(original);
        drop(journal);
        drop(custody);
        temp.retain();
    }
    #[test]
    fn wl2_r4_native_absent_roots_workcopy_retirement_returns_to_absence() {
        for name in [".codex", ".grok"] {
            let temp = Fixture::new();
            let profile = temp.path().join("Profile");
            std::fs::create_dir(&profile).unwrap();
            let client = profile.join(name);
            let custody = create_private_directory(&profile.join("task-custody")).unwrap();
            let journal = create_private_directory(&temp.path().join("D")).unwrap();
            let mut original = RootTree::capture(&client, end()).unwrap();
            original
                .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
                .unwrap();
            assert!(!client.exists());
            original.create_working_copy(end()).unwrap();
            assert!(client.is_dir());
            let work_id = original.working.as_ref().unwrap()[0].id;
            original.restore_original(end()).unwrap();
            original.check_original(end()).unwrap();
            assert!(!client.exists());
            assert_eq!(
                id(&original.retired_work.as_ref().unwrap().root, true).unwrap(),
                work_id
            );
            assert!(original.create_working_copy(end()).is_err());
            println!("native original absence restored for {name}; exact working ID retired, no deletion or Closed eligibility");
        }
    }
}
