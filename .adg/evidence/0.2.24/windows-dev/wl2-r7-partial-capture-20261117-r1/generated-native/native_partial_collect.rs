use crate::backend as native;
use native::{FileIdentity,validate_local_path};
use crate::infrastructure::windows_client_originals024::{bounded::{native_stamp,SnapshotData,QueryError},tree::streams};
use std::{fs::{File,OpenOptions},io::{self,Read,Seek,SeekFrom},os::windows::{fs::OpenOptionsExt,io::AsRawHandle},path::Path,time::Instant};
const ANCHORED_CODEX_STABLE_FILES:[&str;2]=[".opencodex-native-main.claim.sqlite",".opencodex-native-main.owner.sqlite"];
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original phase chain unknown; preserve pending custody/Required",
    )
}
#[derive(PartialEq, Eq)]
struct Entry {
    relative: String,
    directory: bool,
    id: FileIdentity,
    security: SnapshotData,
    bytes: Vec<u8>,
    children: Vec<String>,
}
fn check(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn failure(e: QueryError) -> io::Error {
    io::Error::other(e)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Directory,
    Coordinator,
    File,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Original,
    Moved,
    Working,
    Retired,
    Restored,
}
struct Node {
    original: Entry,
    file: File,
    kind: Kind,
    state: State,
    work: Option<(File, Entry)>,
    pending_work: Option<File>,
    work_open: bool,
}
fn open_node(path: &Path, kind: Kind, observe: bool) -> io::Result<File> {
    validate_local_path(path)?;
    // Anchors are non-DELETE and deny directory substitution. Coordinator
    // sharing preserves official SQLite admission; actual exclusion is lease.
    OpenOptions::new()
        .access_mode(if kind == Kind::Directory {
            0x20080
        } else {
            0x80020000
                | if kind == Kind::File && !observe {
                    0x10000
                } else {
                    0
                }
        })
        .share_mode(if kind == Kind::Coordinator {
            1 | 2
        } else if observe {
            1 | 2 | 4
        } else if kind == Kind::Directory {
            1 | 2
        } else {
            1
        })
        .custom_flags(
            0x200000
                | if kind == Kind::Directory {
                    0x2000000
                } else {
                    0
                },
        )
        .open(path)
}
fn stamp(
    path: &Path,
    file: &mut File,
    relative: &str,
    kind: Kind,
    end: Instant,
) -> io::Result<Entry> {
    check(end)?;
    let directory = kind == Kind::Directory;
    let identity = id(file, directory)?;
    streams(path, directory)?;
    let security = native_stamp(file, end).map_err(failure)?;
    let children = if directory { names(path)? } else { Vec::new() };
    let mut bytes = Vec::new();
    if !directory {
        if file.metadata()?.len() > 2 * 1024 * 1024 {
            return Err(invalid());
        }
        file.seek(SeekFrom::Start(0))?;
        file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 2 * 1024 * 1024 || bytes.len() as u64 != file.metadata()?.len() {
            return Err(invalid());
        }
    }
    if id(file, directory)? != identity
        || native_stamp(file, end).map_err(failure)? != security
        || directory && names(path)? != children
    {
        return Err(invalid());
    }
    check(end)?;
    Ok(Entry {
        relative: relative.into(),
        directory,
        id: identity,
        security,
        bytes,
        children,
    })
}
fn collect(
    root: &Path,
    relative: &str,
    codex: bool,
    out: &mut Vec<Node>,
    total: &mut usize,
    pending: &mut Option<File>,
    end: Instant,
) -> io::Result<()> {
    check(end)?;
    if out.len() >= 512 {
        return Err(invalid());
    }
    let path = root.join(relative);
    let metadata = std::fs::symlink_metadata(&path)?;
    let kind = if metadata.is_dir() {
        Kind::Directory
    } else if codex && ANCHORED_CODEX_STABLE_FILES.contains(&relative) {
        Kind::Coordinator
    } else {
        Kind::File
    };
    // Unknown coordinator sidecars are never ordinary files eligible to move.
    if codex
        && ANCHORED_CODEX_STABLE_FILES
            .iter()
            .any(|s| relative.starts_with(&format!("{s}-")))
    {
        return Err(invalid());
    }
    if pending.is_some() {
        return Err(invalid());
    }
    *pending = Some(open_node(&path, kind, false)?);
    let file = pending.as_mut().ok_or_else(invalid)?;
    if kind == Kind::Directory {
        native::verify_private_directory(file)?;
    } else {
        native::verify_private_file(file)?;
    }
    let original = stamp(&path, file, relative, kind, end)?;
    if kind == Kind::Coordinator {
        super::super::coordination::validate_stable_bytes(&original.bytes)?;
    }
    *total = total
        .checked_add(original.bytes.len())
        .ok_or_else(invalid)?;
    if *total > 32 * 1024 * 1024 {
        return Err(invalid());
    }
    let children = original.children.clone();
    let file = pending.take().ok_or_else(invalid)?;
    out.push(Node {
        original,
        file,
        kind,
        state: State::Original,
        work: None,
        pending_work: None,
        work_open: false,
    });
    for name in children {
        let child = if relative.is_empty() {
            name
        } else {
            format!("{relative}/{name}")
        };
        collect(root, &child, codex, out, total, pending, end)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/src/partial_collect_tests.rs"));}
