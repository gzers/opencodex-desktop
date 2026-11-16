//! Actual unlock receipt retains original file/pin objects. It is not serde,
//! Clone, Send or a recoverable permission. No factory from text or Drop.
use super::*;
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(in super::super) struct LeaseFacts {
    pub(in super::super) epoch: String,
    pub(in super::super) claim_key: String,
    pub(in super::super) objects_digest: String,
}
pub(in super::super) struct ReleasedLease {
    original: NativeLease,
    facts: LeaseFacts,
    poisoned: bool,
    reacquired: bool,
    completed_at: Instant,
}
impl NativeLease {
    fn facts_inner(&self) -> io::Result<LeaseFacts> {
        if self.held.len() != 4 {
            return Err(refused());
        }
        let pin = &self.held[0].pin;
        let claim_key = hash::sha256_hex(
            format!("{}:{}:{}", pin.path.display(), pin.id.volume, pin.id.index).as_bytes(),
        );
        let files=self.held.iter().map(|h|serde_json::json!({"path":h.pin.path,"volume":h.pin.id.volume,"fileId":h.pin.id.index.to_string(),"security":h.pin.security,"bytes":h.pin.raw.len(),"sha256":hash::sha256_hex(&h.pin.raw)})).collect::<Vec<_>>();
        let scope = self.bound.as_ref().map(|b| b.scope.as_str());
        let raw=serde_json::to_vec(&serde_json::json!({"tokenSid":self.token.sid,"tokenSession":self.token.session_id,"tokenElevation":self.token.elevation_type,"scope":scope,"files":files})).map_err(io_error)?;
        Ok(LeaseFacts {
            epoch: self.epoch.clone(),
            claim_key,
            objects_digest: hash::sha256_hex(&raw),
        })
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
        let mut released = ReleasedLease {
            original: self,
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
            if self.poisoned || self.original.active || self.completed_at >= self.original.deadline
            {
                return Err(refused());
            }
            before(end)?;
            if native::current_identity()? != self.original.token {
                return Err(refused());
            }
            for held in &mut self.original.held {
                if !held.ranges.is_empty() {
                    return Err(refused());
                }
                held.pin.check(end)?;
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
            .held
            .iter()
            .map(|h| h.pin.path.clone())
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
