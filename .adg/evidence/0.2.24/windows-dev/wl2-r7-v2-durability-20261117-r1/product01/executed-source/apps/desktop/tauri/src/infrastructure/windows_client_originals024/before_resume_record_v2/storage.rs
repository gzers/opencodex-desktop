//! Native durability for independent strict V2 attachment DATA only. No original Owner/action,
//! claim, quiescence or restoration authority can be created by reading files.
use super::*;
use crate::infrastructure::{
    windows_client_originals024::{
        bounded::{native_stamp, SnapshotData},
        tree,
    },
    windows_official_runtime024::{self as native, FileIdentity, PinnedDirectory},
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::PathBuf,
    time::Instant,
};
const MAX_RECORD: usize = 65536;
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct AttachmentStorageV2 {
    directory: Option<PinnedDirectory>,
    directory_file: Option<File>,
    directory_security: Option<SnapshotData>,
    end: Option<Instant>,
    files: Vec<NativeRecord>,
    pending_file: Option<File>,
    pending_raw: Option<Vec<u8>>,
    pending_path: Option<PathBuf>,
    pending_id: Option<FileIdentity>,
    pending_security: Option<SnapshotData>,
    attempted: bool,
    ready: bool,
    failed: bool,
    #[cfg(test)]
    fault: u8,
    #[cfg(test)]
    after_read: Option<fn(&std::path::Path)>,
}
struct NativeRecord {
    file: File,
    path: PathBuf,
    id: FileIdentity,
    security: SnapshotData,
    raw: Vec<u8>,
}
fn native_id(file: &File, directory: bool) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || data.dwFileAttributes & 0x400 != 0
        || (data.dwFileAttributes & 0x10 != 0) != directory
        || !directory && data.nNumberOfLinks != 1
    {
        return Err(invalid());
    }
    Ok(FileIdentity {
        volume: data.dwVolumeSerialNumber,
        index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
    })
}
fn bytes(file: &mut File) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > MAX_RECORD as u64 {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.take(MAX_RECORD as u64 + 1).read_to_end(&mut raw)?;
    if raw.is_empty() || raw.len() > MAX_RECORD {
        return Err(invalid());
    }
    Ok(raw)
}
fn stamp(file: &File, end: Instant) -> io::Result<SnapshotData> {
    native_stamp(file, end).map_err(io::Error::other)
}
fn members(path: &std::path::Path) -> io::Result<Vec<String>> {
    let mut names = std::fs::read_dir(path)?
        .map(|e| e.and_then(|e| e.file_name().into_string().map_err(|_| invalid())))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    Ok(names)
}
impl NativeRecord {
    fn check(&mut self, end: Instant) -> io::Result<()> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        native::verify_private_file(&self.file)?;
        tree::streams(&self.path, false)?;
        let fresh = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(3)
            .custom_flags(0x200000)
            .open(&self.path)?;
        if native_id(&self.file, false)? != self.id
            || native_id(&fresh, false)? != self.id
            || stamp(&self.file, end)? != self.security
            || stamp(&fresh, end)? != self.security
            || bytes(&mut self.file)? != self.raw
        {
            return Err(invalid());
        }
        tree::streams(&self.path, false)?;
        if native_id(&self.file, false)? != self.id
            || native_id(&fresh, false)? != self.id
            || stamp(&self.file, end)? != self.security
            || stamp(&fresh, end)? != self.security
        {
            return Err(invalid());
        }
        Ok(())
    }
}
impl AttachmentStorageV2 {
    fn cutoff(&self, end: Instant) -> io::Result<()> {
        if self.end != Some(end) {
            return Err(invalid());
        }
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    fn checkpoint(&self, stage: u8, end: Instant) -> io::Result<()> {
        self.cutoff(end)?;
        #[cfg(test)]
        if stage != 0 && self.fault == stage {
            return Err(io::Error::other(format!(
                "injected V2 attachment native stage{stage}"
            )));
        }
        #[cfg(not(test))]
        let _ = stage;
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn initialize(
        &mut self,
        directory: PinnedDirectory,
        end: Instant,
    ) -> io::Result<()> {
        if self.attempted {
            return Err(invalid());
        }
        self.attempted = true;
        self.failed = true;
        self.directory = Some(directory);
        self.end = Some(end);
        let result = (|| {
            self.cutoff(end)?;
            self.directory.as_ref().unwrap().recheck()?;
            let token = native::current_identity()?;
            if token.elevated || token.elevation_type == 2 {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            self.directory_file = Some(
                OpenOptions::new()
                    .access_mode(0x20080)
                    .share_mode(3)
                    .custom_flags(0x2200000)
                    .open(self.directory.as_ref().unwrap().path())?,
            );
            let file = self.directory_file.as_ref().unwrap();
            native::verify_private_directory(file)?;
            if native_id(file, true)? != self.directory.as_ref().unwrap().identity() {
                return Err(invalid());
            }
            tree::streams(self.directory.as_ref().unwrap().path(), true)?;
            self.directory_security = Some(stamp(file, end)?);
            if std::fs::read_dir(self.directory.as_ref().unwrap().path())?
                .next()
                .transpose()?
                .is_some()
            {
                return Err(invalid());
            }
            self.ready = true;
            self.failed = false;
            self.read_inner(end).map(|_| ())
        })();
        if result.is_err() {
            self.failed = true;
            self.ready = false;
        }
        result
    }
    fn read_inner(&mut self, end: Instant) -> io::Result<Vec<Vec<u8>>> {
        self.cutoff(end)?;
        if self.failed
            || !self.ready
            || self.pending_file.is_some()
            || self.pending_path.is_some()
            || self.pending_raw.is_some()
        {
            return Err(invalid());
        }
        let directory = self.directory.as_ref().ok_or_else(invalid)?;
        directory.recheck()?;
        let file = self.directory_file.as_ref().ok_or_else(invalid)?;
        native::verify_private_directory(file)?;
        tree::streams(directory.path(), true)?;
        if native_id(file, true)? != directory.identity()
            || Some(&stamp(file, end)?) != self.directory_security.as_ref()
        {
            return Err(invalid());
        }
        let mut out = Vec::new();
        let expected = (1..=self.files.len())
            .map(|i| format!("v2-record-{i:06}.json"))
            .collect::<Vec<_>>();
        if members(directory.path())? != expected {
            return Err(invalid());
        }
        for (ordinal, record) in self.files.iter_mut().enumerate() {
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let name = format!("v2-record-{:06}.json", ordinal + 1);
            if record.path != directory.path().join(&name) {
                return Err(invalid());
            }
            record.check(end)?;
            out.push(record.raw.clone());
        }
        if !out.is_empty() {
            validate_chain_v2(&out)?;
        }
        #[cfg(test)]
        if let Some(hook) = self.after_read.take() {
            hook(directory.path());
        }
        for record in &mut self.files {
            record.check(end)?;
        }
        if members(directory.path())? != expected {
            return Err(invalid());
        }
        directory.recheck()?;
        if Some(&stamp(file, end)?) != self.directory_security.as_ref() {
            return Err(invalid());
        }
        self.cutoff(end)?;
        Ok(out)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn read_data(
        &mut self,
        end: Instant,
    ) -> io::Result<Vec<Vec<u8>>> {
        let result = self.read_inner(end);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn append_data(
        &mut self,
        raw: &[u8],
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let mut chain = self.read_inner(end)?;
            if chain.len() >= 4096 {
                return Err(invalid());
            }
            RecordV2::decode(raw)?;
            chain.push(raw.to_vec());
            validate_chain_v2(&chain)?;
            self.pending_raw = Some(raw.to_vec());
            self.pending_path = Some(
                self.directory
                    .as_ref()
                    .unwrap()
                    .path()
                    .join(format!("v2-record-{:06}.json", chain.len())),
            );
            self.checkpoint(1, end)?;
            self.pending_file = Some(native::create_private_journal_file(
                self.pending_path.as_ref().unwrap(),
            )?);
            self.checkpoint(2, end)?;
            let file = self.pending_file.as_ref().unwrap();
            native::verify_private_file(file)?;
            tree::streams(self.pending_path.as_ref().unwrap(), false)?;
            self.pending_id = Some(native_id(file, false)?);
            self.pending_security = Some(stamp(file, end)?);
            #[cfg(test)]
            if self.fault == 3 {
                let raw = self.pending_raw.as_ref().unwrap();
                let file = self.pending_file.as_mut().unwrap();
                file.write_all(&raw[..raw.len() / 2])?;
                file.sync_all()?;
                return Err(io::Error::other(
                    "injected actual half-written V2 attachment file",
                ));
            }
            let file = self.pending_file.as_mut().unwrap();
            file.write_all(self.pending_raw.as_ref().unwrap())?;
            file.sync_all()?;
            self.checkpoint(4, end)?;
            let file = self.pending_file.as_mut().unwrap();
            if bytes(file)? != *self.pending_raw.as_ref().unwrap()
                || native_id(file, false)? != self.pending_id.unwrap()
                || stamp(file, end)? != *self.pending_security.as_ref().unwrap()
            {
                return Err(invalid());
            }
            self.files.push(NativeRecord {
                file: self.pending_file.take().unwrap(),
                path: self.pending_path.take().unwrap(),
                id: self.pending_id.take().unwrap(),
                security: self.pending_security.take().unwrap(),
                raw: self.pending_raw.take().unwrap(),
            });
            self.checkpoint(5, end)?;
            self.read_inner(end).map(|_| ())
        })();
        if result.is_err() {
            self.failed = true;
            self.ready = false;
        }
        result
    }
}
#[cfg(test)]
mod tests;
