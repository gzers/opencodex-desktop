//! Original slot-owned preparation prefix. No serialized reconstruction or
//! successful phase/repair proof exists here, even when no tag was returned.
use super::*;
use std::path::PathBuf;
mod v2;
pub(super) struct PartialCaptureAnchor {
    pub(super) session_id: uuid::Uuid,
    pub(super) host_instance_id: uuid::Uuid,
    pub(super) owner_scope: String,
    pub(super) intents: Vec<PathBuf>,
    pub(super) source: Option<Source>,
    pub(super) routes: Option<RoutePin>,
    pub(super) content_start: Option<super::super::super::content::StartContentScope>,
    pub(super) context: Option<super::super::super::source::NativeJournalContext>,
    pub(super) pins: Vec<PinnedDirectory>,
    pub(super) custody: [Option<PinnedDirectory>; 2],
    pub(super) trees: [Option<AnchoredTree>; 2],
    pub(super) journal: Option<Journal>,
    pub(super) journal_prefix: super::super::journal::JournalPreparation,
    pub(super) directory_prefixes:
        Vec<crate::infrastructure::windows_official_runtime024::PrivateDirectoryPreparation>,
    pub(super) failed: bool,
}
impl PartialCaptureAnchor {
    pub(super) fn new(
        owner_scope: String,
        session_id: uuid::Uuid,
        host_instance_id: uuid::Uuid,
    ) -> Self {
        Self {
            session_id,
            host_instance_id,
            owner_scope,
            intents: Vec::new(),
            source: None,
            routes: None,
            content_start: None,
            context: None,
            pins: Vec::new(),
            custody: [None, None],
            trees: [None, None],
            journal: None,
            journal_prefix: super::super::journal::JournalPreparation::default(),
            directory_prefixes: Vec::new(),
            failed: true,
        }
    }
    pub(super) fn intend(&mut self, path: PathBuf) -> io::Result<()> {
        if self.intents.len() >= 16 || self.intents.contains(&path) {
            return Err(invalid());
        }
        self.intents.push(path);
        Ok(())
    }
    pub(super) fn create_directory(
        &mut self,
        path: &Path,
        end: Instant,
    ) -> io::Result<PinnedDirectory> {
        if self.directory_prefixes.len() >= 8 || !self.intents.iter().any(|p| p == path) {
            return Err(invalid());
        }
        self.directory_prefixes.push(
            crate::infrastructure::windows_official_runtime024::PrivateDirectoryPreparation::default(),
        );
        let original = self.directory_prefixes.last_mut().unwrap();
        original.create(path, end)?;
        original.take_complete()
    }
    pub(super) fn into_store(&mut self) -> io::Result<CaptureStore> {
        // Validate the full native component set BEFORE moving any object out
        // of its enrolled original prefix. This transfer itself has no fallible
        // OS check; the same actual objects become the full retained Store.
        if self.source.is_none()
            || self.routes.is_none()
            || self.content_start.is_none()
            || self.journal.is_none()
            || self.trees.iter().any(Option::is_none)
        {
            return Err(invalid());
        }
        let trees = [self.trees[0].take().unwrap(), self.trees[1].take().unwrap()];
        Ok(CaptureStore {
            source: self.source.take().unwrap(),
            routes: self.routes.take(),
            route_close: None,
            core: CapturedTrees::retain(trees, self.journal.take().unwrap()),
            failed: true,
            ready: None,
            stop_deadline: None,
            content_start: self.content_start.take().unwrap(),
            restored_history: false,
            closed_exit: None,
            closed_absent: None,
            stopped_content: None,
            stop_completed: false,
        })
    }
}
