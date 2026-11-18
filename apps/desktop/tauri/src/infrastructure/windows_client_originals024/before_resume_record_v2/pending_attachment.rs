//! Records failed preparation under the original host-private directory.
use super::{storage, validate_chain_v2, EventV2, RecordV2, ReferenceV2};
use crate::infrastructure::{
    windows_client_originals024::bounded::SnapshotData,
    windows_official_runtime024::{self as native, PinnedDirectory, PrivateDirectoryPreparation},
};
use std::{
    fs::File,
    io,
    path::Path,
    thread::{self, ThreadId},
    time::Instant,
};
use uuid::Uuid;

pub(crate) struct PendingAttachment {
    parent: PinnedDirectory,
    parent_file: File,
    security: SnapshotData,
    token: native::TokenIdentity,
    thread: ThreadId,
    origin: [Uuid; 3], // host instance, Owner session, original Start action
    capture_session: Uuid,
    directory: PrivateDirectoryPreparation,
    storage: storage::AttachmentStorageV2,
    attempted: bool,
}

fn check_time(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}

impl PendingAttachment {
    pub(crate) fn bind(
        parent: PinnedDirectory,
        origin: [Uuid; 3],
        capture_session: Uuid,
        end: Instant,
    ) -> io::Result<Self> {
        check_time(end)?;
        let token = native::current_identity()?;
        if token.elevated
            || token.elevation_type == 2
            || origin.iter().any(Uuid::is_nil)
            || capture_session.is_nil()
        {
            return Err(super::invalid());
        }
        let parent_file = storage::open_directory_data(&parent)?;
        let security = storage::private_directory_stamp(&parent, &parent_file, end)?;
        check_time(end)?;
        Ok(Self {
            parent,
            parent_file,
            security,
            token,
            thread: thread::current().id(),
            origin,
            capture_session,
            directory: PrivateDirectoryPreparation::default(),
            storage: storage::AttachmentStorageV2::default(),
            attempted: false,
        })
    }

    fn check_parent(&self, end: Instant) -> io::Result<()> {
        check_time(end)?;
        if self.thread != thread::current().id()
            || self.token != native::current_identity()?
            || self.security
                != storage::private_directory_stamp(&self.parent, &self.parent_file, end)?
        {
            return Err(super::invalid());
        }
        check_time(end)
    }

    pub(in crate::infrastructure::windows_client_originals024) fn persist(
        &mut self,
        data: &Path,
        origin: [Uuid; 3],
        recovery: Uuid,
        records: &[Vec<u8>; 2],
        end: Instant,
    ) -> io::Result<()> {
        if std::mem::replace(&mut self.attempted, true) {
            return Err(super::invalid());
        }
        self.check_parent(end)?;
        if origin != self.origin || data != self.parent.path() {
            return Err(super::invalid());
        }
        let [host, _, failed] = origin.map(|id| id.to_string());
        validate_chain_v2(records)?;
        let head = RecordV2::decode(&records[0])?;
        let tail = RecordV2::decode(&records[1])?;
        let ReferenceV2::Preparation {
            owner_instance_id,
            last_stage,
            ..
        } = &head.reference
        else {
            return Err(super::invalid());
        };
        if owner_instance_id != &host
            || head.session_id != self.capture_session.to_string()
            || head.failed_action_id != failed
            || head.recovery_action_id != recovery.to_string()
            || !matches!(tail.event, EventV2::AbortRecoveryPending { stage, .. } if stage == *last_stage)
        {
            return Err(super::invalid());
        }
        let path = data.join(format!("before-resume-pending-{recovery}"));
        self.check_parent(end)?;
        self.directory.create(&path, end)?;
        let directory = self.directory.take_complete()?;
        self.storage.initialize(directory, end)?;
        for raw in records {
            self.check_parent(end)?;
            self.storage.append_data(raw, end)?;
        }
        if self.storage.read_data(end)? != *records {
            return Err(super::invalid());
        }
        self.check_parent(end)
    }
}

#[cfg(test)]
mod tests;
