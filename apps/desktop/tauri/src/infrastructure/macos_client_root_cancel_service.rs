//! Current service/source admission for an actual uncaptured root action.
//! The complete image and full OS absence are acquired before native Guard
//! initialization; under it only its actual retained Child may be excluded.
use super::*;
use crate::commands::{
    runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider},
    runtime_recovery_v2_owner::SystemServiceOwnerProofProvider,
};
use crate::modules::runtime::{program_image, program_layout, program_selection};

pub struct CurrentService<'a> {
    context: &'a ProcessContext,
    data: &'a PinnedDirectory,
    head: Head,
    root: PathBuf,
    selected: program_selection::SelectedProgram,
    owner: SystemServiceOwnerProofProvider,
    home: PinnedDirectory,
    deadline: Instant,
}
impl<'a> CurrentService<'a> {
    pub(super) fn context(&self) -> &ProcessContext {
        self.context
    }
    pub(super) fn selection(&self) -> &program_selection::SelectedProgram {
        &self.selected
    }
    pub(super) fn image_sha256(&self) -> &str {
        &self.selected.selection.image_sha256
    }
    pub(super) fn require_action(
        &self,
        index: &Index,
        head: &Head,
        deadline: Instant,
    ) -> std::io::Result<()> {
        check_time(self.deadline)?;
        if self.data.identity() != index.data.identity()
            || self.head != *head
            || self.deadline != deadline
            || self.context.working_directory != index.profile
            || self.context.opencodex_home != index.context_home
        {
            return Err(denied(
                "root-cancel service proof belongs to another independent action",
            ));
        }
        Ok(())
    }
    /// No caller-supplied owner, stopped flag, source digest, callback or
    /// deserialized permission may replace these actual current observations.
    pub fn begin(review: &Review<'a>) -> std::io::Result<Self> {
        review.recheck()?;
        let value = Self::observe(
            review.context,
            &review.index.data,
            &review.head,
            review.deadline,
        )?;
        review.recheck()?;
        check_time(value.deadline)?;
        Ok(value)
    }
    pub(in crate::infrastructure::macos_client_originals::session_index) fn begin_pending(
        context: &'a ProcessContext,
        index: &'a Index,
        head: &Head,
        deadline: Instant,
    ) -> std::io::Result<Self> {
        if head.state != State::RootCancellationPrepared || index.head()?.as_ref() != Some(head) {
            return Err(denied("root-cancel pending service source changed"));
        }
        let value = Self::observe(context, &index.data, head, deadline)?;
        value.require_action(index, head, deadline)?;
        if index.head()?.as_ref() != Some(head) {
            return Err(denied("root-cancel pending service source changed"));
        }
        Ok(value)
    }
    fn observe(
        context: &'a ProcessContext,
        data: &'a PinnedDirectory,
        head: &Head,
        deadline: Instant,
    ) -> std::io::Result<Self> {
        check_time(deadline)?;
        let root = context
            .runtime
            .managed_installation_root()
            .ok_or_else(|| denied("root-cancel program root unconfirmed"))?;
        let (target, platform) = program_layout::host_target().map_err(io)?;
        let selected = program_selection::read_complete(&root, platform, target)
            .map_err(io)?
            .ok_or_else(|| denied("root-cancel selected image absent"))?;
        if context.runtime.executable().as_ref() != Some(&selected.stable_entry) {
            return Err(denied("root-cancel runtime selection differs"));
        }
        let owner =
            SystemServiceOwnerProofProvider::new(root.clone(), context.clone()).map_err(io)?;
        let home = PinnedDirectory::open(&context.opencodex_home)?;
        let value = Self {
            context,
            data,
            head: head.clone(),
            root,
            selected,
            owner,
            home,
            deadline,
        };
        value.require_absent()?;
        check_time(value.deadline)?;
        Ok(value)
    }
    fn require_absent(&self) -> std::io::Result<()> {
        check_time(self.deadline)?;
        if self.owner.query(self.context).map_err(io)? != ServiceOwnerProof::Stopped
            || capture_file(&self.home, "runtime-port.json", 4096)? != Image::Absent
            || capture_file(&self.home, "ocx.pid", 64)? != Image::Absent
            || self.owner.query(self.context).map_err(io)? != ServiceOwnerProof::Stopped
        {
            return Err(denied(
                "root-cancel full current service absence unconfirmed",
            ));
        }
        self.home.recheck()?;
        program_selection::verify_current(&self.root, &self.selected).map_err(io)?;
        check_time(self.deadline)
    }
    pub(super) fn guarded(
        &self,
        prepared: &Prepared<'_>,
        native: &NativeLease,
        guard: &mut Guard<'_>,
    ) -> std::io::Result<()> {
        check_time(self.deadline)?;
        if !std::ptr::eq(self.context, prepared.review.context)
            || self.data.identity() != prepared.review.index.data.identity()
            || self.head != prepared.review.head
            || self.deadline != prepared.review.deadline
        {
            return Err(denied(
                "root-cancel service admission belongs to another original review",
            ));
        }
        self.require_guarded(native, guard)
    }
    pub(super) fn require_guarded(
        &self,
        native: &NativeLease,
        guard: &mut Guard<'_>,
    ) -> std::io::Result<()> {
        check_time(self.deadline)?;
        self.owner
            .require_unstarted_guarded(self.context, native, guard)
            .map_err(io)?;
        if capture_file(&self.home, "runtime-port.json", 4096)? != Image::Absent
            || capture_file(&self.home, "ocx.pid", 64)? != Image::Absent
        {
            return Err(denied("root-cancel service records appeared"));
        }
        program_selection::verify_current(&self.root, &self.selected).map_err(io)?;
        self.home.recheck()?;
        check_time(self.deadline)
    }
    /// Final full absence/source readback after native coordination returned
    /// successfully. It cannot turn an interrupted callback into completion.
    pub fn released(&self) -> std::io::Result<()> {
        self.require_absent()?;
        program_image::verify(&self.selected).map_err(io)?;
        self.require_absent()
    }
    /// Fresh complete OS/record absence and selection generation between the
    /// just-completed full image audit and the following transaction admission.
    /// This is not an image admission and cannot itself close the index.
    pub(super) fn observe_released(&self) -> std::io::Result<()> {
        self.require_absent()
    }
}
