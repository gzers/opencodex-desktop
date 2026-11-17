//! Explicit release moves exact original file/pin objects into history, without
//! active ranges/OVERLAPPED or a NativeLease. No serde/Clone or Drop factory.
use super::*;
#[cfg(test)]
thread_local! {static RECOVERY_RELEASE_FAIL_AFTER:std::cell::Cell<Option<usize>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]
pub(in crate::infrastructure::windows_client_originals024) fn fail_recovery_release_after(
    count: usize,
) {
    RECOVERY_RELEASE_FAIL_AFTER.with(|fault| fault.set(Some(count)));
}
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(in super::super) struct LeaseFacts {
    pub(in super::super) epoch: String,
    pub(in super::super) claim_key: String,
    pub(in super::super) objects_digest: String,
}
pub(in super::super) struct ReleasedLease {
    original: ReleasedObjects,
    facts: LeaseFacts,
    poisoned: bool,
    reacquired: bool,
    completed_at: Instant,
}
struct ReleasedObjects {
    files: Vec<LockFile>,
    token: TokenIdentity,
    deadline: Instant,
    bound: Option<Bound>,
    epoch: String,
}
/// Native timely-release history after its original file ownership has moved
/// to the exact close batch. This has no files/ranges/reacquire constructor.
pub(in super::super) struct RetiredLeaseHistory {
    facts: LeaseFacts,
    token: TokenIdentity,
    completed_at: Instant,
    original_deadline: Instant,
}
impl RetiredLeaseHistory {
    pub(in super::super) fn check(&self, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.completed_at >= self.original_deadline || native::current_identity()? != self.token
        {
            return Err(refused());
        }
        Ok(())
    }
    pub(in super::super) fn data(&self) -> serde_json::Value {
        serde_json::json!({"facts":self.facts,"completedInOriginalWindow":self.completed_at<self.original_deadline,"nativeAuthority":false})
    }
}
impl ReleasedLease {
    pub(in super::super) fn observe_close_inventory(
        &self,
        inventory: &mut super::super::handle_release::ExpectedHandleInventory,
        prefix: &str,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        if self.poisoned
            || self.original.files.len() != 4
            || self.completed_at >= self.original.deadline
        {
            return Err(refused());
        }
        for (i, pin) in self.original.files.iter().enumerate() {
            inventory.observe(
                format!("{prefix}/coord/{i}/file"),
                &pin.file,
                pin.id,
                false,
                false,
                end,
            )?;
            match &pin.parent {
                ParentPin::Strict(pin) => {
                    inventory.observe_directory(&format!("{prefix}/coord/{i}/parent"), pin, end)?
                }
                ParentPin::Claim(pin) => {
                    inventory.observe(
                        format!("{prefix}/coord/{i}/claim-parent"),
                        &pin.file,
                        pin.id,
                        true,
                        false,
                        end,
                    )?;
                    inventory.observe_directory(
                        &format!("{prefix}/coord/{i}/ancestor"),
                        &pin.ancestor,
                        end,
                    )?;
                }
            }
        }
        if let Some(bound) = &self.original.bound {
            for (name, pin) in [
                ("profile", &bound.profile),
                ("home", &bound.home),
                ("client", &bound.client),
            ] {
                inventory.observe_directory(&format!("{prefix}/bound/{name}"), pin, end)?;
            }
        }
        before(end)
    }
    /// Caller checked original receipt immediately before ownership transfer.
    /// This operation is infallible and retains every actual file in the batch.
    pub(in super::super) fn transfer_original_files(
        self,
        batch: &mut super::super::handle_release::NativeHandleBatch,
        prefix: &str,
    ) -> RetiredLeaseHistory {
        let ReleasedLease {
            original,
            facts,
            completed_at,
            poisoned,
            ..
        } = self;
        let ReleasedObjects {
            files,
            token,
            deadline,
            bound,
            ..
        } = original;
        if poisoned {
            batch.invalidate();
        }
        for (i, pin) in files.into_iter().enumerate() {
            let LockFile {
                file, parent, id, ..
            } = pin;
            batch.retain(format!("{prefix}/coord/{i}/file"), file, id, false, false);
            match parent {
                ParentPin::Strict(pin) => {
                    for (j, file) in pin.into_owned_files().into_iter().enumerate() {
                        batch.retain_observed(
                            format!("{prefix}/coord/{i}/parent/{j}"),
                            file,
                            true,
                            false,
                        );
                    }
                }
                ParentPin::Claim(pin) => {
                    batch.retain(
                        format!("{prefix}/coord/{i}/claim-parent"),
                        pin.file,
                        pin.id,
                        true,
                        false,
                    );
                    for (j, file) in pin.ancestor.into_owned_files().into_iter().enumerate() {
                        batch.retain_observed(
                            format!("{prefix}/coord/{i}/ancestor/{j}"),
                            file,
                            true,
                            false,
                        );
                    }
                }
            }
        }
        if let Some(bound) = bound {
            for (name, pin) in [
                ("profile", bound.profile),
                ("home", bound.home),
                ("client", bound.client),
            ] {
                for (i, file) in pin.into_owned_files().into_iter().enumerate() {
                    batch.retain_observed(format!("{prefix}/bound/{name}/{i}"), file, true, false);
                }
            }
        }
        RetiredLeaseHistory {
            facts,
            token,
            completed_at,
            original_deadline: deadline,
        }
    }
}
fn facts_from_files(
    epoch: &str,
    token: &TokenIdentity,
    bound: Option<&Bound>,
    files: &[&LockFile],
) -> io::Result<LeaseFacts> {
    if files.len() != 4 {
        return Err(refused());
    }
    let pin = files[0];
    let claim_key = hash::sha256_hex(
        format!("{}:{}:{}", pin.path.display(), pin.id.volume, pin.id.index).as_bytes(),
    );
    let files=files.iter().map(|pin|serde_json::json!({"path":pin.path,"volume":pin.id.volume,"fileId":pin.id.index.to_string(),"security":pin.security,"bytes":pin.raw.len(),"sha256":hash::sha256_hex(&pin.raw)})).collect::<Vec<_>>();
    let scope = bound.map(|b| b.scope.as_str());
    let raw=serde_json::to_vec(&serde_json::json!({"tokenSid":token.sid,"tokenSession":token.session_id,"tokenElevation":token.elevation_type,"scope":scope,"files":files})).map_err(io_error)?;
    Ok(LeaseFacts {
        epoch: epoch.into(),
        claim_key,
        objects_digest: hash::sha256_hex(&raw),
    })
}
impl ReleasedObjects {
    fn facts_inner(&self) -> io::Result<LeaseFacts> {
        facts_from_files(
            &self.epoch,
            &self.token,
            self.bound.as_ref(),
            &self.files.iter().collect::<Vec<_>>(),
        )
    }
}
impl NativeLease {
    // Recovery worker owns this same reference on its owning thread. On any
    // error the exact remaining Held/ranges or already released original pins
    // stay reachable; a consuming Drop is never used as a release receipt.
    pub(in super::super) fn release_retained(
        &mut self,
        out: &mut Option<ReleasedLease>,
        end: Instant,
    ) -> io::Result<()> {
        if out.is_some() {
            return Err(refused());
        }
        let result = (|| {
            self.check(end)?;
            let facts = self.facts_inner()?;
            self.active = false;
            for (index, held) in self.held.iter_mut().rev().enumerate() {
                before(end)?;
                held.release()?;
                #[cfg(test)]
                if RECOVERY_RELEASE_FAIL_AFTER.with(|fault| fault.get() == Some(index + 1)) {
                    return Err(io::Error::from_raw_os_error(5));
                }
                #[cfg(not(test))]
                let _ = index;
            }
            for held in &mut self.held {
                if !held.ranges.is_empty() {
                    return Err(refused());
                }
                held.pin.check(end)?;
            }
            before(end)?;
            let objects = ReleasedObjects {
                files: self.held.drain(..).map(|h| h.pin).collect(),
                token: self.token.clone(),
                deadline: self.deadline,
                bound: self.bound.take(),
                epoch: self.epoch.clone(),
            };
            *out = Some(ReleasedLease {
                original: objects,
                facts,
                poisoned: false,
                reacquired: false,
                completed_at: Instant::now(),
            });
            out.as_mut().unwrap().check(end)
        })();
        if result.is_err() {
            self.active = false;
        }
        result
    }
    fn facts_inner(&self) -> io::Result<LeaseFacts> {
        facts_from_files(
            &self.epoch,
            &self.token,
            self.bound.as_ref(),
            &self.held.iter().map(|h| &h.pin).collect::<Vec<_>>(),
        )
    }
    pub(in super::super) fn release_with_receipt(
        mut self,
        end: Instant,
    ) -> io::Result<ReleasedLease> {
        let valid = self.check(end);
        let facts = self.facts_inner();
        self.active = false;
        let mut release_error = None;
        for held in self.held.iter_mut().rev() {
            if let Err(error) = held.release() {
                if release_error.is_none() {
                    release_error = Some(error);
                }
            }
        }
        // Every path, original handle, security, byte image and sidecar is
        // checked after explicit unlock. A failed/late release stays failure.
        valid?;
        let facts = facts?;
        if let Some(e) = release_error {
            return Err(e);
        }
        for held in &mut self.held {
            if !held.ranges.is_empty() {
                return Err(refused());
            }
            held.pin.check(end)?;
        }
        before(end)?;
        // All ranges are empty and read back BEFORE moving the original pins.
        // Held has no independent Drop, so exact LockFile objects can move out
        // safely. NativeLease's destructor observes an empty held vector.
        let objects = ReleasedObjects {
            files: self.held.drain(..).map(|h| h.pin).collect(),
            token: self.token.clone(),
            deadline: self.deadline,
            bound: self.bound.take(),
            epoch: self.epoch.clone(),
        };
        let mut released = ReleasedLease {
            original: objects,
            facts,
            poisoned: false,
            reacquired: false,
            completed_at: Instant::now(),
        };
        released.check(end)?;
        Ok(released)
    }
}
impl AnchoredClaim<'_> {
    pub(in super::super) fn facts(&mut self, root: &Path, end: Instant) -> io::Result<LeaseFacts> {
        self.check(root, end)?;
        self.lease.facts_inner()
    }
}
impl ReleasedLease {
    pub(in super::super) fn facts(&self) -> &LeaseFacts {
        &self.facts
    }
    pub(in super::super) fn check(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            if end > self.original.deadline {
                return Err(refused());
            }
            before(self.original.deadline)?;
            self.readback_history(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Native historical DATA only. A new deadline here is for fresh reads,
    /// never reacquisition or mutation. Only a timely explicit unlock creates
    /// this object; Drop/JSON cannot supply the completed original window.
    pub(in super::super) fn readback_history(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            if self.poisoned || self.completed_at >= self.original.deadline {
                return Err(refused());
            }
            before(end)?;
            if native::current_identity()? != self.original.token {
                return Err(refused());
            }
            for file in &mut self.original.files {
                file.check(end)?;
            }
            if let Some(bound) = &self.original.bound {
                bound.profile.recheck()?;
                bound.home.recheck()?;
                bound.client.recheck()?;
            }
            if self.original.facts_inner()? != self.facts {
                return Err(refused());
            }
            before(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in super::super) fn reacquire_for_action(
        &mut self,
        action: &crate::infrastructure::windows_official_lifecycle024::NativeAction,
        owner: &mut Owner,
        end: Instant,
    ) -> io::Result<NativeLease> {
        let result = (|| {
            use crate::infrastructure::windows_official_lifecycle024::OriginalActionKind;
            action.check(owner, end).map_err(io_error)?;
            if !matches!(
                action.kind(),
                OriginalActionKind::Stop | OriginalActionKind::Restart
            ) || action.deadline() != end
                || self.reacquired
            {
                return Err(refused());
            }
            self.readback_history(end)?;
            let bound = self.original.bound.as_ref().ok_or_else(refused)?;
            if owner.scope().map_err(io_error)? != bound.scope
                || owner.profile_path() != bound.profile.path()
                || owner.home_path() != bound.home.path()
            {
                return Err(refused());
            }
            let mut new = NativeLease::acquire_existing(owner, end)?;
            new.check_owner(owner, end)?;
            let facts = new.facts_inner()?;
            if facts.epoch == self.facts.epoch
                || facts.claim_key != self.facts.claim_key
                || facts.objects_digest != self.facts.objects_digest
            {
                return Err(refused());
            }
            action.check(owner, end).map_err(io_error)?;
            self.readback_history(end)?;
            self.reacquired = true;
            Ok(new)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// The original receipt stays retained and is single-use. A production
    /// caller must supply the original Owner for bound scopes. Synthetic test
    /// leases have no bound Owner and never manufacture one.
    pub(in super::super) fn reacquire(
        &mut self,
        owner: Option<&mut Owner>,
        end: Instant,
    ) -> io::Result<NativeLease> {
        let result = self.reacquire_inner(owner, end);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn reacquire_inner(
        &mut self,
        mut owner: Option<&mut Owner>,
        end: Instant,
    ) -> io::Result<NativeLease> {
        self.check(end)?;
        if self.reacquired {
            return Err(refused());
        }
        if let Some(bound) = &self.original.bound {
            let actual = owner.as_deref_mut().ok_or_else(refused)?;
            actual.check_current(end).map_err(io_error)?;
            if actual.scope().map_err(io_error)? != bound.scope
                || actual.profile_path() != bound.profile.path()
                || actual.home_path() != bound.home.path()
            {
                return Err(refused());
            }
        } else if owner.is_some() {
            return Err(refused());
        }
        let paths: [PathBuf; 4] = self
            .original
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| refused())?;
        let mut new = if let Some(actual) = owner {
            NativeLease::acquire_existing(actual, end)?
        } else {
            NativeLease::acquire_paths(&paths, end)?
        };
        new.check(end)?;
        let facts = new.facts_inner()?;
        if facts.epoch == self.facts.epoch
            || facts.claim_key != self.facts.claim_key
            || facts.objects_digest != self.facts.objects_digest
        {
            return Err(refused());
        }
        self.check(end)?;
        self.reacquired = true;
        Ok(new)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::windows_client_originals024::tree::tests::Fixture;
    use native::{create_private_directory, PrivateRecord};
    #[test]
    fn wl2_r6_release_objects_move_exact_original_pins_without_active_ranges_or_reopen() {
        fn send<T: Send>() {}
        send::<ReleasedLease>();
        let mut f = Fixture::new();
        let pin = create_private_directory(&f.path().join("synthetic-locks")).unwrap();
        let paths: [PathBuf; 4] = std::array::from_fn(|i| pin.path().join(format!("{i}.sqlite")));
        for p in &paths {
            drop(PrivateRecord::create(p, b"", 16).unwrap());
        }
        let end = Instant::now() + std::time::Duration::from_secs(10);
        let lease = NativeLease::acquire_paths(&paths, end).unwrap();
        let before = lease.facts_inner().unwrap();
        let receipt = lease.release_with_receipt(end).unwrap();
        assert!(receipt.facts == before);
        assert!(receipt.original.files.len() == 4);
        let mut receipt = std::thread::spawn(move || {
            let mut receipt = receipt;
            receipt.check(end).unwrap();
            receipt
        })
        .join()
        .unwrap();
        receipt.check(end).unwrap();
        assert!(receipt.facts == before);
        let mut new = receipt.reacquire(None, end).unwrap();
        assert_ne!(new.epoch, before.epoch);
        new.check(end).unwrap();
        new.release(end).unwrap();
        assert!(
            receipt.reacquire(None, end).is_err(),
            "released objects are retained history, not a repeatable permission or live ranges"
        );
        println!("r6 explicit unlock moves exact original4 LockFile+parent pins, same serialized facts before/after; worker readback and return under same ordinary Token, fresh local lease needs actual acquisition and single-use epoch, no handle clone/reopen/unsafeSend or active NativeLease transfer");
        drop(receipt);
        drop(pin);
        f.retain();
    }
    #[test]
    fn wl2_r6_release_history_fresh_read_after_original_expiry_does_not_reauthorize_lease() {
        let mut f = Fixture::new();
        let pin = create_private_directory(&f.path().join("synthetic-locks")).unwrap();
        let paths: [PathBuf; 4] = std::array::from_fn(|i| pin.path().join(format!("{i}.sqlite")));
        for p in &paths {
            drop(PrivateRecord::create(p, b"", 16).unwrap());
        }
        let original_end = Instant::now() + std::time::Duration::from_secs(1);
        let lease = NativeLease::acquire_paths(&paths, original_end).unwrap();
        let mut receipt = lease.release_with_receipt(original_end).unwrap();
        let original = receipt.facts.objects_digest.clone();
        std::thread::sleep(
            original_end.saturating_duration_since(Instant::now())
                + std::time::Duration::from_millis(1),
        );
        let fresh_read_end = Instant::now() + std::time::Duration::from_secs(1);
        receipt.readback_history(fresh_read_end).unwrap();
        assert_eq!(receipt.facts.objects_digest, original);
        assert!(receipt.completed_at < receipt.original.deadline);
        assert!(
            receipt.reacquire(None, fresh_read_end).is_err(),
            "read-only history after original expiry must not mint a new lease"
        );
        println!("r6 timely original explicit release retained4 native IDs/all8/bytes; fresh1s history read after actual original1s expiry succeeds as DATA only; original reacquire remains rejected, no generic deadline extension or new NativeAction created");
        drop(receipt);
        drop(pin);
        f.retain();
    }
}
