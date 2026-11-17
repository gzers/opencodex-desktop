//! Closed predecessor DATA in the single new manifest. Only separately retained
//! actual completed slots/Owners can produce a NativePredecessors capability.
use super::super::binding::Binding;
use super::*;
use crate::infrastructure::windows_official_runtime024::PinnedDirectory;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Predecessor {
    pub(super) schema: String,
    pub(super) session_id: String,
    pub(super) binding_digest: String,
    pub(super) directory: std::path::PathBuf,
    pub(super) volume: u32,
    pub(super) directory_id: String,
    pub(super) directory_security_digest: String,
    pub(super) current_id: String,
    pub(super) current_security_digest: String,
    pub(super) current_sha256: String,
    pub(super) manifest_digest: String,
    pub(super) sequence: u32,
    pub(super) event_digest: String,
    pub(super) content_digest: String,
    pub(super) route_close_digest: String,
    pub(super) closed_digest: String,
    pub(super) native_completion_digest: String,
    pub(super) prior_set_digest: String,
}
pub(super) fn set_digest(refs: &[Predecessor]) -> io::Result<String> {
    Ok(hash::sha256_hex(&encoded(&refs)?))
}
pub(super) fn validate_set(refs: &[Predecessor], binding: &Binding) -> io::Result<()> {
    if refs.len() > 512 {
        return Err(invalid());
    }
    let binding_digest = hash::sha256_hex(&encoded(binding)?);
    let windows = binding.paths()[1].join("official024/windows");
    let mut sessions = BTreeSet::new();
    let mut objects = BTreeSet::new();
    for (i, r) in refs.iter().enumerate() {
        if r.schema != "windows-anchored-closed-predecessor-v3"
            || !uuid(&r.session_id)
            || !sessions.insert(&r.session_id)
            || r.directory != windows.join(&r.session_id)
            || r.binding_digest != binding_digest
            || !decimal(&r.directory_id)
            || !decimal(&r.current_id)
            || r.directory_id == r.current_id
            || !objects.insert((r.volume, &r.directory_id))
            || !objects.insert((r.volume, &r.current_id))
            || r.sequence == 0
            || r.sequence > 4096
            || r.prior_set_digest != set_digest(&refs[..i])?
            || [
                &r.directory_security_digest,
                &r.current_security_digest,
                &r.current_sha256,
                &r.manifest_digest,
                &r.event_digest,
                &r.content_digest,
                &r.route_close_digest,
                &r.closed_digest,
                &r.native_completion_digest,
            ]
            .into_iter()
            .any(|v| !digest(v))
        {
            return Err(invalid());
        }
        validate_local_path(&r.directory)?;
    }
    Ok(())
}
pub(super) fn check_namespace(
    windows: &PinnedDirectory,
    refs: &[Predecessor],
    binding: &Binding,
    current: Option<&PinnedDirectory>,
    end: std::time::Instant,
) -> io::Result<()> {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
    let before = || -> io::Result<()> {
        if std::time::Instant::now() >= end {
            Err(io::ErrorKind::TimedOut.into())
        } else {
            Ok(())
        }
    };
    before()?;
    validate_set(refs, binding)?;
    if windows.path() != binding.paths()[1].join("official024/windows") {
        return Err(invalid());
    }
    windows.recheck()?;
    let mut expected = refs
        .iter()
        .map(|r| r.session_id.clone())
        .collect::<Vec<_>>();
    if let Some(current) = current {
        current.recheck()?;
        let session = current
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(invalid)?;
        if current.path().parent() != Some(windows.path())
            || !uuid(session)
            || expected.iter().any(|s| s == session)
        {
            return Err(invalid());
        }
        expected.push(session.into());
    }
    expected.sort();
    let names = || -> io::Result<Vec<String>> {
        let mut names = std::fs::read_dir(windows.path())?
            .map(|e| e.and_then(|e| e.file_name().into_string().map_err(|_| invalid())))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        Ok(names)
    };
    if names()? != expected {
        return Err(invalid());
    }
    for r in refs {
        before()?;
        let directory = PinnedDirectory::open_private(&r.directory)?;
        if directory.identity().volume != r.volume
            || directory.identity().index.to_string() != r.directory_id
        {
            return Err(invalid());
        }
        let dir = OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(1 | 2)
            .custom_flags(0x2200000)
            .open(&r.directory)?;
        let security = super::super::bounded::native_stamp(&dir, end).map_err(io::Error::other)?;
        if hash::sha256_hex(&encoded(&security)?) != r.directory_security_digest {
            return Err(invalid());
        }
        let path = r.directory.join("current.json");
        super::super::tree::streams(&path, false)?;
        let file = OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(1 | 2)
            .custom_flags(0x200000)
            .open(&path)?;
        let security = super::super::bounded::native_stamp(&file, end).map_err(io::Error::other)?;
        if !security.matches_id(r.volume, &r.current_id)
            || hash::sha256_hex(&encoded(&security)?) != r.current_security_digest
        {
            return Err(invalid());
        }
        let raw = std::fs::read(&path)?;
        if raw.is_empty() || raw.len() > 4096 || hash::sha256_hex(&raw) != r.current_sha256 {
            return Err(invalid());
        }
        if super::super::bounded::native_stamp(&file, end).map_err(io::Error::other)? != security {
            return Err(invalid());
        }
        directory.recheck()?;
    }
    if names()? != expected {
        return Err(invalid());
    }
    windows.recheck()?;
    before()
}
#[cfg(test)]
#[path = "predecessor_data_tests.rs"]
mod tests;
