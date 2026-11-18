//! Same-object preservation for an INITIAL uncaptured root. It does not publish
//! a cancellation terminal or turn the old index into a new Start permission.
//! Plans are registered in the original independent index. Concrete current
//! service/source admission is required for production movement.
use super::*;
use crate::infrastructure::official_client_coordination::native::{config::Guard, NativeLease};
use std::os::fd::AsRawFd;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    schema_version: u32,
    id: String,
    source: Head,
    origin: super::super::super::root_creation::References,
    binding_sha256: String,
    initial_review_sha256: String,
    history_sha256: String,
    capture_vault: vault_history::Evidence,
    root: super::super::super::TreeSnapshot,
    grok: super::super::super::GrokSnapshot,
    writer: super::super::super::Image,
}
pub struct Prepared<'a> {
    review: Review<'a>,
    slot: root_cancel_namespace::RegisteringSlot,
    id: String,
    bytes: Vec<u8>,
    sha256: String,
    registered: Option<Head>,
}
/// An action owner must supply its original canonical registration and current
/// service/source/native permissions. No callbacks or deserialized plan can
/// construct this type in production yet.
pub struct MovementAuthority<'a> {
    prepared: &'a Prepared<'a>,
    deadline: Instant,
    service: Option<&'a service::CurrentService<'a>>,
}
/// Original native movement/readback from this invocation only. This is not
/// Closed, CancelledBeforeCapture, normal Stop, or successor eligibility.
pub struct PreservedRoot {
    id: String,
    plan_sha256: String,
    receipt_sha256: String,
    root_identity: super::super::super::Identity,
    admitted: bool,
    deadline: Instant,
}
impl PreservedRoot {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }
    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }
    pub fn root_identity(&self) -> &super::super::super::Identity {
        &self.root_identity
    }
}
fn io(error: crate::errors::AppError) -> std::io::Error {
    let _ = error;
    denied("root-cancel native coordinator unconfirmed")
}
fn facts(review: &Review<'_>, physical: &PinnedDirectory, expected: &Head) -> std::io::Result<()> {
    check_time(review.deadline)?;
    review.index.check_storage()?;
    if read_head(review.index.connection()?)?.as_ref() != Some(expected) {
        return Err(denied("root-cancel original independent head changed"));
    }
    if digest(&encode(&vault_history::read(
        review.index.connection()?,
        &review.head,
    )?)?)
        != review.history_sha256
    {
        return Err(denied("root-cancel independently indexed history changed"));
    }
    let stored: Vec<u8> = review
        .index
        .connection()?
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    if stored != review.index.binding {
        return Err(denied("root-cancel independent binding changed"));
    }
    let origin = review
        .index
        .root_registration
        .as_ref()
        .ok_or_else(|| denied("root origin missing"))?;
    super::super::super::root_creation::verify_indexed_anchor(
        &review.index.action_parent,
        &review.index.data,
        &review.index.profile,
        physical,
        origin,
        physical.path(),
    )?;
    let history = vault_history::read(review.index.connection()?, &review.head)?;
    review
        .capture_vault
        .verify_root_retained(&review.index.data, &review.head, &history, false)?;
    if vault_history::inspect(
        &review.index.data,
        physical,
        &review.index.profile,
        &review.index.binding,
        &review.head,
        &history,
    )? != review.capture_vault
        || super::super::super::GrokSnapshot::read(&PinnedDirectory::open(&review.index.profile)?)?
            != review.initial_grok
    {
        return Err(denied("root-cancel capture storage or Grok changed"));
    }
    check_time(review.deadline)
}
impl<'a> Prepared<'a> {
    /// The current concrete service/source admission is rechecked under the
    /// actual original Guard/claim. Registration alone cannot mint this permit.
    pub fn authorize_current(
        &'a self,
        service: &'a service::CurrentService<'a>,
        native: &NativeLease,
        guard: &mut Guard<'_>,
    ) -> std::io::Result<MovementAuthority<'a>> {
        if self.registered.is_none() {
            return Err(denied("root-cancel movement plan unregistered"));
        }
        self.require_live()?;
        service.guarded(self, native, guard)?;
        Ok(MovementAuthority {
            prepared: self,
            deadline: self.review.deadline,
            service: Some(service),
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn destination(&self) -> PathBuf {
        self.slot.directory().path().join("root")
    }
    /// The Review was minted BEFORE coordination initialization on an empty
    /// root. Only actual official Guard/NativeLease may explain those new locks.
    pub fn under_native_guard(
        review: Review<'a>,
        native: &NativeLease,
        guard: &mut Guard<'_>,
    ) -> std::io::Result<Self> {
        native
            .require_context(review.context, review.index.client.path())
            .map_err(io)?;
        facts(&review, &review.index.client, &review.head)?;
        let image = guard
            .root_image(native, review.index.client.path())
            .map_err(io)?;
        native.begin_claim_handoff().map_err(io)?;
        native.require_claim_handoff().map_err(io)?;
        if guard
            .root_image(native, review.index.client.path())
            .map_err(io)?
            != image
        {
            return Err(denied("root-cancel writer changed during claim handoff"));
        }
        let tree = super::super::super::TreeSnapshot::read(&review.index.client)?;
        let released = tree.without_config_guard(&image)?;
        if !released.first_root_execution_baseline() {
            return Err(denied("root-cancel root contains unexpected client data"));
        }
        let plan = Plan {
            schema_version: 2,
            id: uuid::Uuid::new_v4().to_string(),
            source: review.head.clone(),
            origin: review.index.root_registration.clone().unwrap(),
            binding_sha256: digest(&review.index.binding),
            initial_review_sha256: review.fingerprint.clone(),
            history_sha256: review.history_sha256.clone(),
            capture_vault: review.capture_vault.clone(),
            root: tree,
            grok: review.initial_grok.clone(),
            writer: image,
        };
        let parent = review
            .index
            .action_parent
            .child_private("client-root-cancel-plans", true)?;
        if !super::super::super::tree::names(parent.file())?.is_empty() {
            return Err(denied(
                "root-cancel retained plan namespace requires resolution",
            ));
        }
        let slot = root_cancel_namespace::RegisteringSlot::create(&parent, &plan.id)?;
        let bytes = encode(&plan)?;
        let sha256 = digest(&bytes);
        create(slot.directory(), "plan.json", &bytes)?;
        parent.sync()?;
        review.index.action_parent.sync()?;
        let result = Self {
            review,
            slot,
            id: plan.id,
            bytes,
            sha256,
            registered: None,
        };
        result.require_live()?;
        native.check().map_err(io)?;
        guard.check().map_err(io)?;
        Ok(result)
    }
    fn expected(&self) -> &Head {
        self.registered.as_ref().unwrap_or(&self.review.head)
    }
    /// Publish the actual guarded plan in the original CAS/event stream BEFORE
    /// any move. This is a pending reference, never service-absence or mutation
    /// authority. A failed commit/readback retains the plan and rejects reuse.
    pub fn register(mut self) -> std::io::Result<Self> {
        self.require_live()?;
        if self.registered.is_some() {
            return Err(denied("root-cancel plan already registered"));
        }
        let next = Head {
            revision: self
                .review
                .head
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root-cancel revision overflow"))?,
            ticket: self.review.head.ticket.clone(),
            state: State::RootCancellationPrepared,
            references: None,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: Some(RootRetirement {
                source_revision: self.review.head.revision,
                plan: RestorePlan {
                    id: self.id.clone(),
                    sha256: self.sha256.clone(),
                },
            }),
            root_generation: None,
        };
        self.review.index.write_checked_projected(
            Some(&self.review.head),
            &next,
            &|_| Ok(()),
            &|originals| {
                if originals.is_some() {
                    return Err(denied("root-cancel cannot register captured originals"));
                }
                self.require_live()
            },
            &|_, _, _| Ok(()),
            Some(&self.slot),
        )?;
        self.registered = Some(next);
        self.require_live()?;
        Ok(self)
    }
    pub fn registered_head(&self) -> Option<&Head> {
        self.registered.as_ref()
    }
    fn plan(&self) -> std::io::Result<Plan> {
        self.slot.directory().private()?;
        match capture_file(self.slot.directory(), "plan.json", MAX_FRAME)? {
            Image::File {
                bytes, protection, ..
            } if bytes == self.bytes && digest(&bytes) == self.sha256 => {
                require_private(&protection, false)?;
                let plan: Plan = decode(&bytes)?;
                if plan.schema_version != 2
                    || plan.id != self.id
                    || plan.source != self.review.head
                    || plan.initial_review_sha256 != self.review.fingerprint
                    || plan.binding_sha256 != digest(&self.review.index.binding)
                    || plan.history_sha256 != self.review.history_sha256
                    || plan.capture_vault != self.review.capture_vault
                {
                    return Err(denied("root-cancel plan source changed"));
                }
                Ok(plan)
            }
            _ => Err(denied("root-cancel exact plan changed or missing")),
        }
    }
    fn require_live(&self) -> std::io::Result<()> {
        let plan = self.plan()?;
        facts(&self.review, &self.review.index.client, self.expected())?;
        if observation::entry(self.slot.directory(), "root")?.is_some()
            || super::super::super::TreeSnapshot::read(&self.review.index.client)? != plan.root
        {
            return Err(denied("root-cancel root or destination changed"));
        }
        Ok(())
    }
    /// Exact object movement, continuously protected by the real claim/Guard.
    /// No production MovementAuthority constructor exists before canonical wiring.
    pub fn preserve_guarded(
        &self,
        authority: &MovementAuthority<'_>,
        native: &NativeLease,
        guard: Guard<'_>,
    ) -> std::io::Result<PreservedRoot> {
        self.preserve_checked(authority, native, guard, &|_| Ok(()))
    }
    fn preserve_checked(
        &self,
        authority: &MovementAuthority<'_>,
        native: &NativeLease,
        mut guard: Guard<'_>,
        checkpoint: &impl Fn(&str) -> std::io::Result<()>,
    ) -> std::io::Result<PreservedRoot> {
        if self.registered.is_none() || !std::ptr::eq(authority.prepared, self) {
            return Err(denied("root-cancel authority belongs to another plan"));
        }
        #[cfg(not(test))]
        if authority.service.is_none() {
            return Err(denied("root-cancel concrete service admission missing"));
        }
        let end = self.review.deadline.min(authority.deadline);
        check_time(end)?;
        self.require_live()?;
        native
            .require_context(self.review.context, self.review.index.client.path())
            .map_err(io)?;
        native.require_claim_handoff().map_err(io)?;
        let plan = self.plan()?;
        if let Some(service) = authority.service {
            service.guarded(self, native, &mut guard)?;
        }
        if guard
            .root_image(native, self.review.index.client.path())
            .map_err(io)?
            != plan.writer
        {
            return Err(denied("root-cancel writer differs"));
        }
        let target = self.destination();
        guard.prepare_relocation(&target).map_err(io)?;
        let profile = PinnedDirectory::open(&self.review.index.profile)?;
        // Persist intent before the first rename. A missing readback is pending,
        // not permission to retry/overwrite or infer that no rename occurred.
        create(
            self.slot.directory(),
            "move-intent.json",
            &encode(&(self.sha256.as_str(), self.review.index.client.identity()))?,
        )?;
        self.require_live()?;
        native.check().map_err(io)?;
        guard.check().map_err(io)?;
        check_time(end)?;
        checkpoint("before_rename")?;
        if let Some(service) = authority.service {
            service.guarded(self, native, &mut guard)?;
        }
        let from = name(OsStr::new(".codex"))?;
        let to = name(OsStr::new("root"))?;
        if unsafe {
            libc::renameatx_np(
                profile.file().as_raw_fd(),
                from.as_ptr(),
                self.slot.directory().file().as_raw_fd(),
                to.as_ptr(),
                libc::RENAME_EXCL,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error());
        }
        profile.sync()?;
        self.slot.directory().sync()?;
        // On failure AFTER rename, preserve the exact root and all records. No
        // catch-all rollback to a potentially newly created logical path.
        let archived = PinnedDirectory::open(&target)?;
        if archived.identity() != self.review.index.client.identity()
            || super::super::super::TreeSnapshot::read(&archived)? != plan.root
        {
            return Err(denied("root-cancel preserved root differs"));
        }
        native.reanchor_claim(&target).map_err(io)?;
        guard.reanchor(&target).map_err(io)?;
        checkpoint("after_reanchor")?;
        facts(&self.review, &archived, self.expected())?;
        native.check().map_err(io)?;
        guard.check().map_err(io)?;
        if let Some(service) = authority.service {
            service.guarded(self, native, &mut guard)?;
        }
        check_time(end)?;
        guard.finish().map_err(io)?;
        let released = plan.root.without_config_guard(&plan.writer)?;
        if super::super::super::TreeSnapshot::read(&archived)? != released
            || observation::entry(&profile, ".codex")?.is_some()
        {
            return Err(denied("root-cancel release or logical absence unconfirmed"));
        }
        facts(&self.review, &archived, self.expected())?;
        native.check().map_err(io)?;
        check_time(end)?;
        let bytes = encode(&serde_json::json!({
            "schema":"macos-root-preparation-preserved-v1","planSha256":self.sha256,
            "rootIdentity":archived.identity(),"logicalRootAbsent":true,"writerReleased":true,
            "canonicalCancellationClosed":false,"normalStop":false,"successorEligible":false,
        }))?;
        create(self.slot.directory(), "preserved-readback.json", &bytes)?;
        if super::super::super::TreeSnapshot::read(&archived)? != released {
            return Err(denied("root-cancel readback root changed"));
        }
        check_time(end)?;
        Ok(PreservedRoot {
            id: self.id.clone(),
            plan_sha256: self.sha256.clone(),
            receipt_sha256: digest(&bytes),
            root_identity: archived.identity().clone(),
            admitted: authority.service.is_some(),
            deadline: end,
        })
    }
    #[cfg(test)]
    fn fixture_authority(&'a self) -> MovementAuthority<'a> {
        assert!(self.registered.is_some());
        MovementAuthority {
            prepared: self,
            deadline: self.review.deadline,
            service: None,
        }
    }
}

#[path = "macos_client_root_cancel_complete.rs"]
pub mod completion;
#[path = "macos_client_root_prepare_indexed.rs"]
pub(in crate::infrastructure::macos_client_originals) mod indexed;
#[path = "macos_client_root_cancel_service.rs"]
pub mod service;

#[cfg(test)]
#[path = "macos_client_root_cancel_service_tests.rs"]
mod service_tests;
#[cfg(test)]
#[path = "macos_client_root_prepare_preserve_tests.rs"]
mod tests;
