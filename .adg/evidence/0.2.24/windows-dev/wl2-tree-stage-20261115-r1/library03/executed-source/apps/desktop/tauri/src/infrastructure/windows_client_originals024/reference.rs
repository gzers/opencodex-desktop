//! Same-object current-reference CAS. A pointer is a recovery locator only;
//! even a Closed phase chain never grants native restoration or successor rights.
use super::{
    descriptor,
    phase::{invalid, PendingOnly},
};
use crate::infrastructure::{
    hash,
    windows_official_runtime024::{FileIdentity, PinnedDirectory, PrivateRecord},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
    time::Instant,
};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
};
const MAX: usize = 4096;
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Tip {
    schema: String,
    session_id: String,
    sequence: u32,
    phase_digest: String,
    previous_phase_digest: String,
    binding_digest: String,
    originals_manifest_digest: String,
}
impl Tip {
    pub(super) fn from_prefix(
        session_id: String,
        sequence: u32,
        phase_digest: String,
        previous_phase_digest: String,
        binding_digest: String,
        originals_manifest_digest: String,
    ) -> Self {
        Self {
            schema: "windows-client-current-v1".into(),
            session_id,
            sequence,
            phase_digest,
            previous_phase_digest,
            binding_digest,
            originals_manifest_digest,
        }
    }
}
trait Security {
    const ACCESS: u32;
    fn read(file: &File) -> io::Result<Vec<u8>>;
}
struct Full;
impl Security for Full {
    const ACCESS: u32 =
        crate::modules::runtime::windows_lifecycle024_contract::FULL_DESCRIPTOR_QUERY_ACCESS;
    fn read(file: &File) -> io::Result<Vec<u8>> {
        descriptor(file).map_err(|e| io::Error::other(e.to_string()))
    }
}
/// The only non-test adapter is statically fixed to Full. Neither a partial
/// descriptor nor caller-provided bytes/bool can instantiate this type.
pub(super) struct CurrentReference(Reference<Full>);
impl CurrentReference {
    pub(super) fn append_pending_phase(
        &mut self,
        prefix: &mut PendingOnly,
        phase: super::phase::Phase,
        deadline: Instant,
    ) -> io::Result<PendingOnly> {
        self.0.check(deadline)?;
        prefix.check_directory(&self.0.parent)?;
        if self.0.tip != prefix.tip {
            return Err(invalid());
        }
        let next = prefix.append_pending(phase, deadline)?;
        self.0.advance(&next, deadline)?;
        Ok(next)
    }
    pub(super) fn create(
        directory: &Path,
        prefix: &PendingOnly,
        deadline: Instant,
    ) -> io::Result<Self> {
        let inspected = super::InspectedOriginal::open(directory, true)
            .map_err(|e| io::Error::other(e.to_string()))?;
        inspected
            .recheck()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(Self(Reference::create(directory, prefix, deadline)?))
    }
    pub(super) fn open(
        directory: &Path,
        prefix: &PendingOnly,
        deadline: Instant,
    ) -> io::Result<Self> {
        Ok(Self(Reference::open(directory, prefix, deadline)?))
    }
    pub(super) fn advance(&mut self, prefix: &PendingOnly, deadline: Instant) -> io::Result<()> {
        self.0.advance(prefix, deadline)
    }
}
struct Reference<G: Security> {
    file: File,
    parent: PinnedDirectory,
    id: FileIdentity,
    security: Vec<u8>,
    raw: Vec<u8>,
    tip: Tip,
    poisoned: bool,
    _guard: PhantomData<G>,
}
fn check_time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn identity(file: &File) -> io::Result<FileIdentity> {
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
fn read(file: &mut File) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > MAX as u64 {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.take(MAX as u64 + 1).read_to_end(&mut raw)?;
    if raw.is_empty() || raw.len() > MAX {
        return Err(invalid());
    }
    Ok(raw)
}
impl<G: Security> Reference<G> {
    fn create(directory: &Path, prefix: &PendingOnly, deadline: Instant) -> io::Result<Self> {
        check_time(deadline)?;
        let parent = PinnedDirectory::open_private(directory)?;
        prefix.check_directory(&parent)?;
        if prefix.tip.sequence != 1 {
            return Err(invalid());
        }
        let raw = serde_json::to_vec(&prefix.tip).map_err(|_| invalid())?;
        let created = PrivateRecord::create(&directory.join("current.json"), &raw, MAX)?;
        let id = created.identity();
        drop(created);
        let reference = Self::open(directory, prefix, deadline)?;
        if reference.id != id {
            return Err(invalid());
        }
        Ok(reference)
    }
    fn open(directory: &Path, prefix: &PendingOnly, deadline: Instant) -> io::Result<Self> {
        check_time(deadline)?;
        let parent = PinnedDirectory::open_private(directory)?;
        prefix.check_directory(&parent)?;
        if directory.file_name().and_then(|s| s.to_str()) != Some(&prefix.tip.session_id) {
            return Err(invalid());
        }
        // First establish exact private object identity/content. Closing this
        // reader does not grant adoption: exclusive reopen must match all facts.
        let path = directory.join("current.json");
        let mut initial = PrivateRecord::open(&path, MAX)?;
        let id = initial.identity();
        let raw = initial.read()?;
        let guarded = OpenOptions::new()
            .access_mode(0x80020000 | G::ACCESS)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(&path)?;
        if identity(&guarded)? != id {
            return Err(invalid());
        }
        let security = G::read(&guarded)?;
        if G::read(&guarded)? != security || initial.read()? != raw {
            return Err(invalid());
        }
        drop(guarded);
        drop(initial);
        let mut file = OpenOptions::new()
            .access_mode(0xc0020000 | G::ACCESS)
            .share_mode(0)
            .custom_flags(0x00200000)
            .open(path)?;
        if identity(&file)? != id || read(&mut file)? != raw {
            return Err(invalid());
        }
        if G::read(&file)? != security {
            return Err(invalid());
        }
        let tip: Tip = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        if tip != prefix.tip {
            return Err(invalid());
        }
        parent.recheck()?;
        check_time(deadline)?;
        Ok(Self {
            file,
            parent,
            id,
            security,
            raw,
            tip,
            poisoned: false,
            _guard: PhantomData,
        })
    }
    fn check(&mut self, deadline: Instant) -> io::Result<()> {
        check_time(deadline)?;
        if self.poisoned
            || identity(&self.file)? != self.id
            || G::read(&self.file)? != self.security
            || read(&mut self.file)? != self.raw
        {
            return Err(invalid());
        }
        self.parent.recheck()?;
        check_time(deadline)
    }
    fn advance(&mut self, prefix: &PendingOnly, deadline: Instant) -> io::Result<()> {
        self.check(deadline)?;
        prefix.check_directory(&self.parent)?;
        let next = &prefix.tip;
        if self.tip.sequence.checked_add(1) != Some(next.sequence)
            || self.tip.session_id != next.session_id
            || self.tip.binding_digest != next.binding_digest
            || self.tip.originals_manifest_digest != next.originals_manifest_digest
            || self.tip.phase_digest != next.previous_phase_digest
        {
            return Err(invalid());
        }
        let raw = serde_json::to_vec(next).map_err(|_| invalid())?;
        if raw.len() > MAX {
            return Err(invalid());
        }
        // From first write onward, a failure leaves a permanently poisoned
        // in-memory reference. No implicit retries, rollback or default empty.
        self.poisoned = true;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&raw)?;
        self.file.set_len(raw.len() as u64)?;
        self.file.sync_all()?;
        if identity(&self.file)? != self.id
            || G::read(&self.file)? != self.security
            || read(&mut self.file)? != raw
        {
            return Err(invalid());
        }
        self.parent.recheck()?;
        check_time(deadline)?;
        self.tip = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        self.raw = raw;
        self.poisoned = false;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    // Intentionally incompatible with CurrentReference<Full>. This diagnostic
    // O/G/D projection exercises the file CAS mechanism, never full security.
    struct PartialDiagnostic;
    impl Security for PartialDiagnostic {
        const ACCESS: u32 = 0x20000;
        fn read(file: &File) -> io::Result<Vec<u8>> {
            #[link(name = "advapi32")]
            unsafe extern "system" {
                fn GetSecurityInfo(
                    h: *mut std::ffi::c_void,
                    kind: i32,
                    mask: u32,
                    o: *mut *mut std::ffi::c_void,
                    g: *mut *mut std::ffi::c_void,
                    d: *mut *mut std::ffi::c_void,
                    s: *mut *mut std::ffi::c_void,
                    out: *mut *mut std::ffi::c_void,
                ) -> u32;
            }
            let mut sd = std::ptr::null_mut();
            let code = unsafe {
                GetSecurityInfo(
                    file.as_raw_handle(),
                    1,
                    7,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut sd,
                )
            };
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            if sd.is_null() {
                return Err(invalid());
            }
            struct Local(*mut std::ffi::c_void);
            impl Drop for Local {
                fn drop(&mut self) {
                    unsafe {
                        windows_sys::Win32::Foundation::LocalFree(self.0);
                    }
                }
            }
            let _local = Local(sd);
            let len = unsafe { windows_sys::Win32::Security::GetSecurityDescriptorLength(sd) };
            if !(20..=262144).contains(&len)
                || unsafe { windows_sys::Win32::Security::IsValidSecurityDescriptor(sd) } == 0
            {
                return Err(invalid());
            }
            Ok(unsafe { std::slice::from_raw_parts(sd.cast::<u8>(), len as usize) }.to_vec())
        }
    }
    fn end() -> Instant {
        Instant::now() + Duration::from_secs(5)
    }
    #[test]
    fn wl2_reference_same_object_exclusive_cas_and_stale_prefix_refuse() {
        let t = tempfile::tempdir().unwrap();
        let records = super::super::phase::tests::reference_chain();
        let id: serde_json::Value = serde_json::from_slice(&records[0]).unwrap();
        let dir = crate::infrastructure::windows_official_runtime024::create_private_directory(
            &t.path().join(id["sessionId"].as_str().unwrap()),
        )
        .unwrap();
        let first =
            PrivateRecord::create(&dir.path().join("phase-00000001.json"), &records[0], 65536)
                .unwrap();
        let mut original = super::super::phase::read_pending(dir.path()).unwrap();
        let mut current =
            Reference::<PartialDiagnostic>::create(dir.path(), &original, end()).unwrap();
        let native_id = current.id;
        assert!(Reference::<PartialDiagnostic>::open(dir.path(), &original, end()).is_err());
        assert!(std::fs::write(dir.path().join("current.json"), b"external tamper").is_err());
        assert!(
            std::fs::rename(dir.path().join("current.json"), dir.path().join("moved")).is_err()
        );
        assert!(current.advance(&original, end()).is_err());
        let ready = original
            .append_pending(super::super::phase::Phase::Ready, end())
            .unwrap();
        assert!(original
            .append_pending(super::super::phase::Phase::Ready, end())
            .is_err());
        // Phase exists and has already been sync/read back, but old pointer is
        // still old until the CAS below. No implicit catch-up on opening.
        assert!(dir.path().join("phase-00000002.json").is_file());
        current.advance(&ready, end()).unwrap();
        assert_eq!(current.id, native_id);
        assert!(current.advance(&original, end()).is_err());
        drop(current);
        assert!(Reference::<PartialDiagnostic>::open(dir.path(), &original, end()).is_err());
        let reopened = Reference::<PartialDiagnostic>::open(dir.path(), &ready, end()).unwrap();
        assert_eq!(reopened.id, native_id);
        println!("same current-reference fileID retained, stale pointer/exclusive writer refused; securityCoverage=partialDiagnosticOnly; Closed=false");
        drop(reopened);
        drop(ready);
        drop(original);
        drop(first);
        drop(dir);
    }
    #[test]
    fn wl2_reference_interrupted_write_retains_phase_records_and_refuses_reopen() {
        let t = tempfile::tempdir().unwrap();
        let records = super::super::phase::tests::reference_chain();
        let id: serde_json::Value = serde_json::from_slice(&records[0]).unwrap();
        let dir = crate::infrastructure::windows_official_runtime024::create_private_directory(
            &t.path().join(id["sessionId"].as_str().unwrap()),
        )
        .unwrap();
        let first =
            PrivateRecord::create(&dir.path().join("phase-00000001.json"), &records[0], 65536)
                .unwrap();
        let prefix = super::super::phase::read_pending(dir.path()).unwrap();
        let mut current =
            Reference::<PartialDiagnostic>::create(dir.path(), &prefix, end()).unwrap();
        // Explicit partial-write interruption, no helper retry. Surviving
        // immutable phases discover pending recovery; pointer is not rebuilt.
        current.poisoned = true;
        current.file.seek(SeekFrom::Start(0)).unwrap();
        current.file.write_all(b"{").unwrap();
        current.file.set_len(1).unwrap();
        current.file.sync_all().unwrap();
        assert!(current.check(end()).is_err());
        drop(current);
        let pending = super::super::phase::read_pending(dir.path()).unwrap();
        assert!(!pending.terminal_record_observed);
        assert!(Reference::<PartialDiagnostic>::open(dir.path(), &pending, end()).is_err());
        assert_eq!(
            std::fs::read(dir.path().join("phase-00000001.json")).unwrap(),
            records[0]
        );
        println!("partial current write: phase preserved, reference Unknown/Required; no reconstructed permission; fullSD not queried");
        drop(pending);
        drop(prefix);
        drop(first);
        drop(dir);
    }
    #[test]
    fn wl2_reference_foreign_directory_and_duplicate_pointer_never_reconstruct_authority() {
        let t = tempfile::tempdir().unwrap();
        let records = super::super::phase::tests::reference_chain();
        let id: serde_json::Value = serde_json::from_slice(&records[0]).unwrap();
        let native = crate::infrastructure::windows_official_runtime024::create_private_directory;
        let dir = native(&t.path().join(id["sessionId"].as_str().unwrap())).unwrap();
        let foreign = native(&t.path().join("foreign-session")).unwrap();
        let first =
            PrivateRecord::create(&dir.path().join("phase-00000001.json"), &records[0], 65536)
                .unwrap();
        let mut original = super::super::phase::read_pending(dir.path()).unwrap();
        assert!(Reference::<PartialDiagnostic>::create(foreign.path(), &original, end()).is_err());
        assert!(!foreign.path().join("current.json").exists());
        let mut current =
            Reference::<PartialDiagnostic>::create(dir.path(), &original, end()).unwrap();
        let mut altered = serde_json::to_value(&original.tip).unwrap();
        altered["sequence"] = serde_json::json!(2);
        altered["sessionId"] = serde_json::json!(uuid::Uuid::new_v4().to_string());
        let old = std::mem::replace(&mut original.tip, serde_json::from_value(altered).unwrap());
        assert!(current.advance(&original, end()).is_err());
        original.tip = old;
        let raw = String::from_utf8(current.raw.clone()).unwrap();
        drop(current);
        for text in [
            raw.replacen('{', "{\"schema\":\"windows-client-current-v1\",", 1),
            raw.replacen('{', "{\"ownerAuthority\":true,", 1),
        ] {
            std::fs::write(dir.path().join("current.json"), text.as_bytes()).unwrap();
            assert!(Reference::<PartialDiagnostic>::open(dir.path(), &original, end()).is_err());
        }
        println!("foreign-directory create leaves no pointer; foreign session, duplicate and unknown current fields refused; diagnostic security only");
        drop(original);
        drop(first);
        drop(foreign);
        drop(dir);
    }
}
