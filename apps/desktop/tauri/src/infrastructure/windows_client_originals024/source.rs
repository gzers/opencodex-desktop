//! Shared ORIGINAL Windows source pin for new-scope adapters. Strict existing
//! fixed-image/current/peer/token checks are reused; no parsed-data authority.
use super::{
    binding::Binding,
    bounded::{native_stamp, BoundedAccessV1},
    phase::{digest, invalid},
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        hash,
        windows_official_lifecycle024::Owner,
        windows_official_runtime024::{PinnedDirectory, RetainedPeer},
    },
    modules::runtime::{
        program_image,
        program_selection::{self, CurrentSelection},
        windows_lifecycle024_contract::OFFICIAL_PACKAGE_SHA256,
    },
};
use crate::{
    infrastructure::windows_official_lifecycle024::{NativeAction, OriginalActionKind},
    modules::runtime::windows_lifecycle024_contract::NativeSession,
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    time::Instant,
};
fn refused(reason: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_originals_required".into(),
        detail: format!("原件持久恢复未确认；保留原代次/记录/句柄：{reason}"),
    }
}
pub(super) fn content_error(reason: impl std::fmt::Display) -> AppError {
    refused(reason)
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
pub(super) struct CurrentPin {
    file: File,
    security: BoundedAccessV1,
    raw: Vec<u8>,
}
impl CurrentPin {
    fn bytes(file: &mut File) -> io::Result<Vec<u8>> {
        file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        file.take(65537).read_to_end(&mut raw)?;
        if raw.is_empty() || raw.len() > 65536 {
            return Err(invalid());
        }
        Ok(raw)
    }
    pub(super) fn capture(
        path: &Path,
        expected: &CurrentSelection,
        end: Instant,
    ) -> io::Result<Self> {
        before(end)?;
        if expected.schema_version != 1
            || expected.version != "2.99.0"
            || expected.platform != "win32-x64"
            || !digest(&expected.image_sha256)
        {
            return Err(invalid());
        }
        let security = BoundedAccessV1::capture(path, false, end).map_err(io::Error::other)?;
        let mut file = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(path)?;
        let raw = Self::bytes(&mut file)?;
        let observed: CurrentSelection = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        if observed != *expected
            || native_stamp(&file, end).map_err(io::Error::other)? != security.snapshot_data()?
        {
            return Err(invalid());
        }
        let mut out = Self {
            file,
            security,
            raw,
        };
        out.check(end)?;
        Ok(out)
    }
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.security.recheck(end).map_err(io::Error::other)?;
        if Self::bytes(&mut self.file)? != self.raw
            || native_stamp(&self.file, end).map_err(io::Error::other)?
                != self.security.snapshot_data()?
        {
            return Err(invalid());
        }
        before(end)
    }
}
pub(super) struct Source {
    broker: RetainedPeer,
    pins: [PinnedDirectory; 4],
    current: CurrentPin,
    owner_scope: String,
    pub(super) binding_raw: Vec<u8>,
    image_digest: String,
}
/// Fresh native context; not a serde locator or standalone recovery authority.
/// Only Source checked against the original actual Owner can construct it.
pub(super) struct NativeJournalContext {
    pins: [PinnedDirectory; 4],
    peer: RetainedPeer,
    binding: Binding,
    action: NativeAction,
}
impl NativeJournalContext {
    pub(super) fn into_parts(self) -> ([PinnedDirectory; 4], RetainedPeer, Binding, NativeAction) {
        (self.pins, self.peer, self.binding, self.action)
    }
}
impl Source {
    pub(super) fn journal_context(
        &mut self,
        owner: &mut Owner,
        kind: OriginalActionKind,
        end: Instant,
    ) -> AppResult<NativeJournalContext> {
        self.check(owner, end)?;
        let action = owner.original_action(kind, end)?;
        let peer = RetainedPeer::current().map_err(refused)?;
        let pins = [
            PinnedDirectory::open(owner.profile_path()).map_err(refused)?,
            PinnedDirectory::open_private(owner.data_path()).map_err(refused)?,
            PinnedDirectory::open(owner.home_path()).map_err(refused)?,
            PinnedDirectory::open(owner.installation_path()).map_err(refused)?,
        ];
        let binding = Binding::from_native(
            &peer,
            &pins,
            self.image_digest.clone(),
            hash::sha256_hex(&self.current.raw),
        )
        .map_err(refused)?;
        if serde_json::to_vec(&binding).map_err(refused)? != self.binding_raw {
            return Err(refused("fresh journal context不是原Source/Owner绑定"));
        }
        action.check(owner, end)?;
        self.check(owner, end)?;
        Ok(NativeJournalContext {
            pins,
            peer,
            binding,
            action,
        })
    }
    pub(super) fn capture(owner: &mut Owner, end: Instant) -> AppResult<(Self, Binding)> {
        owner.originals_require_unresumed(end)?;
        let broker = RetainedPeer::current().map_err(refused)?;
        broker.no_outer_job().map_err(refused)?;
        broker
            .verify_image(&owner.installation_path().join("ocxd.exe"))
            .map_err(refused)?;
        let token = &broker.identity().token;
        if token.elevated
            || token.elevation_type == 2
            || token != &owner.original_native().binding().process.token
        {
            return Err(refused("原普通broker与原launcher令牌不匹配"));
        }
        let selected = owner.selected_program();
        if selected.selection.version != "2.99.0"
            || selected.selection.platform != "win32-x64"
            || selected.image.official_archive_sha256 != OFFICIAL_PACKAGE_SHA256
        {
            return Err(refused("固定官方2.99来源不匹配"));
        }
        program_image::verify_until(selected, end)?;
        let current = CurrentPin::capture(
            &owner.installation_path().join("runtime/current.json"),
            &selected.selection,
            end,
        )
        .map_err(refused)?;
        let image_digest = selected.selection.image_sha256.clone();
        let pins = [
            PinnedDirectory::open(owner.profile_path()).map_err(refused)?,
            PinnedDirectory::open_private(owner.data_path()).map_err(refused)?,
            PinnedDirectory::open(owner.home_path()).map_err(refused)?,
            PinnedDirectory::open(owner.installation_path()).map_err(refused)?,
        ];
        let binding = Binding::from_native(
            &broker,
            &pins,
            image_digest.clone(),
            hash::sha256_hex(&current.raw),
        )
        .map_err(refused)?;
        let binding_raw = serde_json::to_vec(&binding).map_err(refused)?;
        let owner_scope = owner.scope()?;
        let mut out = Self {
            broker,
            pins,
            current,
            owner_scope,
            binding_raw,
            image_digest,
        };
        out.check(owner, end)?;
        Ok((out, binding))
    }
    pub(super) fn check(&mut self, owner: &mut Owner, end: Instant) -> AppResult<()> {
        owner.check_current(end)?;
        self.broker.no_outer_job().map_err(refused)?;
        self.broker
            .verify_image(&owner.installation_path().join("ocxd.exe"))
            .map_err(refused)?;
        let selected = owner.selected_program();
        if selected.selection.image_sha256 != self.image_digest
            || selected.image.official_archive_sha256 != OFFICIAL_PACKAGE_SHA256
            || owner.scope()? != self.owner_scope
        {
            return Err(refused("原owner/source代次变化"));
        }
        program_selection::verify_current(owner.installation_path(), selected)?;
        program_image::verify_until(selected, end)?;
        self.current.check(end).map_err(refused)?;
        let binding = Binding::from_native(
            &self.broker,
            &self.pins,
            self.image_digest.clone(),
            hash::sha256_hex(&self.current.raw),
        )
        .map_err(refused)?;
        if serde_json::to_vec(&binding).map_err(refused)? != self.binding_raw {
            return Err(refused("原SID/Profile/D/H/I/current绑定变化"));
        }
        owner.check_current(end)
    }
}
