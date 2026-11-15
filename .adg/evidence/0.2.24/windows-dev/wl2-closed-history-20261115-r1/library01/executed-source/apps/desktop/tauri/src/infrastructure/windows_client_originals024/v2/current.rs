//! Concrete new-scope CAS, no generic Full/Partial adapter and no conversion.
use super::super::bounded::{native_stamp, BoundedAccessV1, SnapshotData};
use super::*;
use crate::infrastructure::windows_official_runtime024::FileIdentity;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    time::Instant,
};
fn deadline(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn failure(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}
fn bytes(file: &mut File) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > 4096 {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.take(4097).read_to_end(&mut raw)?;
    if raw.is_empty() || raw.len() > 4096 {
        return Err(invalid());
    }
    Ok(raw)
}
fn identity(file: &File) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if i.dwFileAttributes & 0x410 != 0 || i.nNumberOfLinks != 1 {
        return Err(invalid());
    }
    Ok(FileIdentity {
        volume: i.dwVolumeSerialNumber,
        index: ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64,
    })
}
/// Exact native handle+parent+bounded-query snapshot. Nonserde/nonClone. The
/// serialized current is pending locator data, never restoration authority.
pub(crate) struct BoundedCurrentV2 {
    file: File,
    parent: PinnedDirectory,
    id: FileIdentity,
    security: SnapshotData,
    raw: Vec<u8>,
    tip: Current,
    poisoned: bool,
    #[cfg(test)]
    fail_after_partial_write: bool,
}
impl BoundedCurrentV2 {
    #[cfg(test)]
    pub(super) fn inject_partial_write_failure(&mut self) {
        self.fail_after_partial_write = true;
    }
    pub(super) fn create(pending: &PendingV2, end: Instant) -> io::Result<Self> {
        deadline(end)?;
        if pending.tip.sequence != 1 {
            return Err(invalid());
        }
        let guard =
            BoundedAccessV1::capture(pending.directory.path(), true, end).map_err(failure)?;
        guard.recheck(end).map_err(failure)?;
        let raw = serde_json::to_vec(&current_for(&pending.tip)).map_err(|_| invalid())?;
        let record =
            PrivateRecord::create(&pending.directory.path().join("current.json"), &raw, 4096)?;
        let id = record.identity();
        drop(record);
        let out = Self::open(pending, end)?;
        if out.id != id {
            return Err(invalid());
        }
        Ok(out)
    }
    pub(super) fn open(pending: &PendingV2, end: Instant) -> io::Result<Self> {
        deadline(end)?;
        let parent = PinnedDirectory::open_private(pending.directory.path())?;
        if parent.identity() != pending.directory.identity() {
            return Err(invalid());
        }
        let path = parent.path().join("current.json");
        let mut record = PrivateRecord::open(&path, 4096)?;
        let id = record.identity();
        let raw = record.read()?;
        let observed = OpenOptions::new()
            .access_mode(0x20000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&path)?;
        let security = native_stamp(&observed, end).map_err(failure)?;
        if identity(&observed)? != id {
            return Err(invalid());
        }
        drop(observed);
        drop(record);
        let mut file = OpenOptions::new()
            .access_mode(0xc0020000)
            .share_mode(0)
            .custom_flags(0x200000)
            .open(path)?;
        if identity(&file)? != id
            || bytes(&mut file)? != raw
            || native_stamp(&file, end).map_err(failure)? != security
        {
            return Err(invalid());
        }
        let tip: Current = json(&raw, 4096)?;
        if tip != current_for(&pending.tip) {
            return Err(invalid());
        }
        parent.recheck()?;
        deadline(end)?;
        Ok(Self {
            file,
            parent,
            id,
            security,
            raw,
            tip,
            poisoned: false,
            #[cfg(test)]
            fail_after_partial_write: false,
        })
    }
    fn check(&mut self, end: Instant) -> io::Result<()> {
        deadline(end)?;
        if self.poisoned
            || identity(&self.file)? != self.id
            || native_stamp(&self.file, end).map_err(failure)? != self.security
            || bytes(&mut self.file)? != self.raw
        {
            return Err(invalid());
        }
        self.parent.recheck()?;
        deadline(end)
    }
    pub(super) fn verify_pending(&mut self, pending: &PendingV2, end: Instant) -> io::Result<()> {
        self.check(end)?;
        if pending.directory.identity() != self.parent.identity()
            || current_for(&pending.tip) != self.tip
        {
            return Err(invalid());
        }
        let fresh = read_pending_impl(self.parent.path(), false)?;
        if current_for(&fresh.tip) != self.tip {
            return Err(invalid());
        }
        deadline(end)
    }
    fn advance(&mut self, pending: &PendingV2, end: Instant) -> io::Result<()> {
        self.check(end)?;
        if pending.directory.identity() != self.parent.identity() {
            return Err(invalid());
        }
        let next = current_for(&pending.tip);
        if next.sequence != self.tip.sequence.checked_add(1).ok_or_else(invalid)?
            || next.session_id != self.tip.session_id
            || next.binding_digest != self.tip.binding_digest
            || next.security_policy_digest != self.tip.security_policy_digest
            || next.originals_manifest_digest != self.tip.originals_manifest_digest
            || next.previous_phase_digest != self.tip.phase_digest
        {
            return Err(invalid());
        }
        let raw = serde_json::to_vec(&next).map_err(|_| invalid())?;
        if raw.len() > 4096 {
            return Err(invalid());
        }
        self.poisoned = true;
        self.file.seek(SeekFrom::Start(0))?;
        #[cfg(test)]
        if self.fail_after_partial_write {
            // Real write+sync to the actual exclusive current object, then an
            // injected I/O fault. The flag is unavailable in production.
            self.file.write_all(&raw[..1])?;
            self.file.set_len(1)?;
            self.file.sync_all()?;
            return Err(io::Error::other(
                "injected after real partial current write and sync",
            ));
        }
        self.file.write_all(&raw)?;
        self.file.set_len(raw.len() as u64)?;
        self.file.sync_all()?;
        if bytes(&mut self.file)? != raw
            || identity(&self.file)? != self.id
            || native_stamp(&self.file, end).map_err(failure)? != self.security
        {
            return Err(invalid());
        }
        self.parent.recheck()?;
        deadline(end)?;
        self.raw = raw;
        self.tip = next;
        self.poisoned = false;
        Ok(())
    }
    pub(super) fn append_pending(
        &mut self,
        pending: &PendingV2,
        phase: Phase,
        end: Instant,
    ) -> io::Result<PendingV2> {
        self.append_record(
            pending,
            phase,
            TreeRecord::None,
            WorkingRecord::None,
            WorkingRecord::None,
            end,
        )
    }
    pub(super) fn append_working_phase(
        &mut self,
        pending: &PendingV2,
        phase: Phase,
        reference: WorkingReference,
        end: Instant,
    ) -> io::Result<PendingV2> {
        self.append_record(
            pending,
            phase,
            TreeRecord::None,
            WorkingRecord::Snapshot(reference),
            WorkingRecord::None,
            end,
        )
    }
    pub(super) fn append_tree_record(
        &mut self,
        pending: &PendingV2,
        record: TreeRecord,
        end: Instant,
    ) -> io::Result<PendingV2> {
        if record == TreeRecord::None {
            return Err(invalid());
        }
        self.append_record(
            pending,
            pending.tip.phase,
            record,
            WorkingRecord::None,
            WorkingRecord::None,
            end,
        )
    }
    pub(super) fn append_closed_history(
        &mut self,
        pending: &PendingV2,
        reference: WorkingReference,
        end: Instant,
    ) -> io::Result<PendingV2> {
        self.append_record(
            pending,
            Phase::Closed,
            TreeRecord::None,
            WorkingRecord::None,
            WorkingRecord::Snapshot(reference),
            end,
        )
    }
    fn append_record(
        &mut self,
        pending: &PendingV2,
        phase: Phase,
        tree_record: TreeRecord,
        working_record: WorkingRecord,
        history_record: WorkingRecord,
        end: Instant,
    ) -> io::Result<PendingV2> {
        self.check(end)?;
        if pending.directory.identity() != self.parent.identity()
            || current_for(&pending.tip) != self.tip
        {
            return Err(invalid());
        }
        let fresh = read_pending_impl(self.parent.path(), false)?;
        if current_for(&fresh.tip) != self.tip {
            return Err(invalid());
        }
        let manifest: Manifest = json(&fresh.manifest_raw, 1024 * 1024)?;
        let last: Stage = json(fresh.phase_raw.last().ok_or_else(invalid)?, 65536)?;
        let stage = Stage {
            schema: ORIGINALS_RECORD_SCHEMA_V2.into(),
            document_kind: "phase".into(),
            session_id: last.session_id,
            sequence: last.sequence.checked_add(1).ok_or_else(invalid)?,
            previous_digest: self.tip.phase_digest.clone(),
            previous_record_id: Some(format!("phase-{:08}.json", last.sequence)),
            binding: manifest.binding,
            phase,
            tree_record,
            working_record,
            history_record,
            originals_manifest_digest: self.tip.originals_manifest_digest.clone(),
            security_policy_digest: self.tip.security_policy_digest.clone(),
        };
        let raw = serde_json::to_vec(&stage).map_err(|_| invalid())?;
        let mut prefix = fresh.phase_raw.clone();
        prefix.push(raw.clone());
        check_prefix(&fresh.manifest_raw, &prefix, &current_identity()?.sid)?;
        deadline(end)?;
        let record = PrivateRecord::create(
            &self
                .parent
                .path()
                .join(format!("phase-{:08}.json", stage.sequence)),
            &raw,
            65536,
        )?;
        let next = read_pending_impl(self.parent.path(), false)?;
        self.advance(&next, end)?;
        drop(record);
        Ok(next)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn end() -> Instant {
        Instant::now() + std::time::Duration::from_secs(5)
    }
    #[test]
    fn wl2_r4_native_v2_partial_current_write_preserves_phase_and_poison() {
        use super::super::super::tree::tests::Fixture;
        let mut fixture = Fixture::new();
        let (manifest, records, _) =
            super::super::tests::bundle(super::super::tests::manifest_value());
        let m: Manifest = json(&manifest, 1024 * 1024).unwrap();
        let directory =
            crate::infrastructure::windows_official_runtime024::create_private_directory(
                &fixture.path().join(&m.session_id),
            )
            .unwrap();
        let manifest_handle = PrivateRecord::create(
            &directory.path().join("manifest.json"),
            &manifest,
            1024 * 1024,
        )
        .unwrap();
        let first = PrivateRecord::create(
            &directory.path().join("phase-00000001.json"),
            &records[0],
            65536,
        )
        .unwrap();
        let pending = read_pending(directory.path()).unwrap();
        let mut current = BoundedCurrentV2::create(&pending, end()).unwrap();
        let original_id = current.id;
        current.fail_after_partial_write = true;
        assert!(current
            .append_pending(&pending, Phase::Aborted, end())
            .is_err());
        assert!(current.poisoned);
        assert_eq!(identity(&current.file).unwrap(), original_id);
        assert_eq!(bytes(&mut current.file).unwrap(), b"{");
        let phase_path = directory.path().join("phase-00000002.json");
        let phase_before = std::fs::read(&phase_path).unwrap();
        let latest = read_pending_impl(directory.path(), false).unwrap();
        assert_eq!(latest.tip.sequence, 2);
        assert!(current
            .append_pending(&pending, Phase::Aborted, end())
            .is_err());
        assert!(current
            .append_tree_record(
                &latest,
                TreeRecord::Intent {
                    root: TreeRoot::Codex,
                    action: TreeAction::MoveToCustody
                },
                end()
            )
            .is_err());
        drop(current);
        assert!(read_pending(directory.path()).is_err());
        assert!(BoundedCurrentV2::open(&latest, end()).is_err());
        assert_eq!(std::fs::read(&phase_path).unwrap(), phase_before);
        assert_eq!(
            std::fs::read(directory.path().join("current.json")).unwrap(),
            b"{"
        );
        assert!(!directory.path().join("phase-00000003.json").exists());
        println!("actual bounded-v2 partial current write+sync injected failure: same fileID, phase2 retained, poisoned retry/open refused, no catch-up/rollback/Closed; synthetic manifest only");
        drop(latest);
        drop(pending);
        drop(first);
        drop(manifest_handle);
        drop(directory);
        fixture.retain();
    }
    #[test]
    fn wl2_r4_native_v2_reference_uses_real_bounded_snapshot_and_same_object_cas() {
        let (m, records, _) = super::super::tests::bundle(super::super::tests::manifest_value());
        let metadata: Manifest = json(&m, 1024 * 1024).unwrap();
        let t = tempfile::tempdir().unwrap();
        let pin = crate::infrastructure::windows_official_runtime024::create_private_directory(
            &t.path().join(&metadata.session_id),
        )
        .unwrap();
        let manifest =
            PrivateRecord::create(&pin.path().join("manifest.json"), &m, 1024 * 1024).unwrap();
        let first =
            PrivateRecord::create(&pin.path().join("phase-00000001.json"), &records[0], 65536)
                .unwrap();
        let initial = read_pending(pin.path()).unwrap();
        let mut current = BoundedCurrentV2::create(&initial, end()).unwrap();
        let id = current.id;
        assert!(BoundedCurrentV2::open(&initial, end()).is_err());
        assert!(std::fs::write(pin.path().join("current.json"), b"tamper").is_err());
        assert!(current.advance(&initial, end()).is_err());
        let aborted = current
            .append_pending(&initial, Phase::Aborted, end())
            .unwrap();
        assert_eq!(current.id, id);
        assert!(current
            .append_pending(&initial, Phase::Aborted, end())
            .is_err());
        drop(current);
        let latest = read_pending(pin.path()).unwrap();
        assert!(latest.current_matches);
        let reopened = BoundedCurrentV2::open(&aborted, end()).unwrap();
        assert_eq!(reopened.id, id);
        println!("v2 bounded current native CAS: all8+combined security, policyDigest{}, samefileID{}, no Full/Partial conversion; synthetic manifest only, OriginalSession/Closed not issued",reopened.tip.security_policy_digest,id.index);
        drop(reopened);
        drop(latest);
        drop(aborted);
        drop(initial);
        drop(first);
        drop(manifest);
        drop(pin);
    }
}
