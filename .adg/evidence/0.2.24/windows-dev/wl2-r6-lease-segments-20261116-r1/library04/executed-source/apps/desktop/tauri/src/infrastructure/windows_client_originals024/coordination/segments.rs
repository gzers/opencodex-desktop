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
            if self.poisoned || self.original.active || end > self.original.deadline {
                return Err(refused());
            }
            before(end)?;
            before(self.original.deadline)?;
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
