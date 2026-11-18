//! Read DATA from the actual directory preparation borrow. It never reopens a
//! created target without an original terminal handle, supplies missing initial
//! security from a current observation, or grants cleanup/restoration.
use super::*;
use crate::infrastructure::windows_client_originals024::{
    before_resume_record_v2::projection_digest, bounded::native_stamp,
};
use crate::infrastructure::windows_official_runtime024 as native;
use std::{
    fs::{File, OpenOptions},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
};
#[cfg(test)]
thread_local! {static AFTER_DIRECTORY:std::cell::Cell<Option<fn(&Path)>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]
pub(super) fn after_directory_for_test(hook: fn(&Path)) {
    AFTER_DIRECTORY.with(|slot| {
        assert!(slot.get().is_none());
        slot.set(Some(hook));
    });
}
fn identity(file: &File) -> io::Result<native::FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || data.dwFileAttributes & 0x400 != 0
        || data.dwFileAttributes & 0x10 == 0
    {
        return Err(invalid());
    }
    Ok(native::FileIdentity {
        volume: data.dwVolumeSerialNumber,
        index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
    })
}
fn members(path: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for row in std::fs::read_dir(path)? {
        let name = row?.file_name().into_string().map_err(|_| invalid())?;
        if names.len() >= 512 {
            return Err(invalid());
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}
pub(super) fn read(
    prefix: &native::PrivateDirectoryPreparation,
    end: Instant,
) -> io::Result<serde_json::Value> {
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    let original = prefix.retained_native_read(end)?;
    let path_digest = original
        .path
        .map(|path| projection_digest(&path))
        .transpose()?;
    let parent_ids = if let Some(parent) = original.parent {
        parent.recheck()?;
        Some(parent.owned_file_identities()?)
    } else {
        None
    };
    let parent_digest = parent_ids
        .as_ref()
        .map(|ids| {
            projection_digest(
                &ids.iter()
                    .map(|id| (id.volume, id.index.to_string()))
                    .collect::<Vec<_>>(),
            )
        })
        .transpose()?;
    let mut data = serde_json::json!({"pathDigest":path_digest,"parentIdentityDigest":parent_digest,"created":original.created,"attempted":original.attempted,"transferred":original.transferred,
        "ready":original.ready,"failed":original.failed,"identityDigest":null,"observedSecurityDigest":null,"securityDigest":null,"contentDigest":null,
        "originalSecurityBaselineVerified":false,"membershipBaselineVerified":false,"terminalObserved":false,"currentPathObservedWithoutOriginalHandle":false,"verifiedAbsent":false,"pending":true,"runtimeAuthority":false});
    if let Some(terminal) = original.terminal {
        let path = original.path.ok_or_else(invalid)?;
        native::verify_private_directory(terminal)?;
        let id = identity(terminal)?;
        if let Some(complete) = original.complete {
            complete.recheck()?;
            if complete.identity() != id {
                return Err(invalid());
            }
        }
        // This read/query observation can compare a path only because the exact
        // original terminal is retained. Missing native-open paths never enter.
        let fresh = OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(3)
            .custom_flags(0x2200000)
            .open(path)?;
        if identity(&fresh)? != id {
            return Err(invalid());
        }
        let sd = native_stamp(terminal, end).map_err(io::Error::other)?;
        if native_stamp(&fresh, end).map_err(io::Error::other)? != sd {
            return Err(invalid());
        }
        super::super::super::super::tree::streams(path, true)?;
        let first = members(path)?;
        // The producer has not transferred this directory to any consumer yet.
        // Foreign content cannot be explained by a later legitimate stage.
        if !first.is_empty() {
            return Err(invalid());
        }
        #[cfg(test)]
        AFTER_DIRECTORY.with(|slot| {
            if let Some(hook) = slot.take() {
                hook(path);
            }
        });
        let second = members(path)?;
        super::super::super::super::tree::streams(path, true)?;
        native::verify_private_directory(terminal)?;
        if first != second
            || identity(terminal)? != id
            || identity(&fresh)? != id
            || native_stamp(terminal, end).map_err(io::Error::other)? != sd
            || native_stamp(&fresh, end).map_err(io::Error::other)? != sd
        {
            return Err(invalid());
        }
        if let Some(complete) = original.complete {
            complete.recheck()?;
            if complete.identity() != id {
                return Err(invalid());
            }
        }
        data["terminalObserved"] = true.into();
        data["identityDigest"] = projection_digest(&(id.volume, id.index.to_string()))?.into();
        data["observedSecurityDigest"] = projection_digest(&sd)?.into();
        data["contentDigest"] = projection_digest(&first)?.into();
        // The creator did not retain an initial all8 snapshot before every
        // possible failure point. Its assurance field deliberately stays null.
    }
    if let Some(parent) = original.parent {
        parent.recheck()?;
        if Some(parent.owned_file_identities()?) != parent_ids {
            return Err(invalid());
        }
    }
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(data)
}
