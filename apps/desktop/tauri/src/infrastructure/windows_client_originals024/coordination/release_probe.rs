//! Native same-thread release observation. Every temporary original-range
//! acquisition remains reachable on failure; no Drop cleanup can mint success.
use super::*;
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct ReleaseProbe {
    preparation: LeasePreparation,
    held: Vec<Held>,
    attempted: bool,
    finished: bool,
    failed: bool,
    end: Option<Instant>,
    epoch: Option<String>,
    claim: Option<String>,
    facts: Option<String>,
    token: Option<TokenIdentity>,
}
impl ReleaseProbe {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_held_ranges_for_test(
        &self,
    ) -> Vec<usize> {
        self.held.iter().map(|h| h.ranges.len()).collect()
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_cleanup_for_test(
        &mut self,
    ) -> io::Result<()> {
        for h in self.held.iter_mut().rev() {
            h.release()?;
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn started(&self) -> bool {
        self.attempted
    }
    pub(super) fn inspect(
        &mut self,
        original: &mut ReleasedLease,
        stable_path: &Path,
        stable_file: &mut File,
        end: Instant,
    ) -> io::Result<()> {
        if self.attempted {
            return Err(refused());
        }
        self.attempted = true;
        self.failed = true;
        self.end = Some(end);
        let result = (|| {
            original.check(end)?;
            let (pins, facts) = original.probe_originals(end)?;
            self.epoch = Some(facts.epoch.clone());
            self.claim = Some(facts.claim_key.clone());
            self.facts = Some(facts.objects_digest.clone());
            self.token = Some(native::current_identity()?);
            let sid = self.token.as_ref().unwrap();
            if sid.elevated || sid.elevation_type == 2 {
                return Err(refused());
            }
            native::verify_private_file(stable_file)?;
            tree::streams(stable_path, false)?;
            let stable_id = file_id(stable_file)?;
            let stable_security = native_stamp(stable_file, end).map_err(io_error)?;
            let stable_raw = snapshot(stable_file, end)?;
            if pins.len() != 4
                || stable_path
                    != pins[0]
                        .path
                        .parent()
                        .ok_or_else(refused)?
                        .join(".opencodex-native-main.owner.sqlite")
            {
                return Err(refused());
            }
            for (i, old) in pins.iter().enumerate() {
                let pin = self
                    .preparation
                    .capture_probe_file(&old.path, i == 0, end)?;
                self.held.push(Held {
                    pin,
                    ranges: Vec::new(),
                });
                let held = self.held.last_mut().unwrap();
                if held.pin.id != old.id
                    || held.pin.security != old.security
                    || held.pin.raw != old.raw
                {
                    return Err(refused());
                }
                held.range(BASE, 512, true)?;
                held.pin.check(end)?;
            }
            let pin = self
                .preparation
                .capture_probe_file(stable_path, false, end)?;
            self.held.push(Held {
                pin,
                ranges: Vec::new(),
            });
            let held = self.held.last_mut().unwrap();
            if held.pin.id != stable_id
                || held.pin.security != stable_security
                || held.pin.raw != stable_raw
            {
                return Err(refused());
            }
            held.range(BASE, 512, true)?;
            // All five exact description ranges are held concurrently here,
            // including both original stable anchors and the original N/L/K.
            for held in &mut self.held {
                held.pin.check(end)?;
                if held.ranges.len() != 1
                    || held.ranges[0].offset != BASE
                    || held.ranges[0].length != 512
                {
                    return Err(refused());
                }
            }
            if file_id(stable_file)? != stable_id
                || native_stamp(stable_file, end).map_err(io_error)? != stable_security
                || snapshot(stable_file, end)? != stable_raw
            {
                return Err(refused());
            }
            for held in self.held.iter_mut().rev() {
                before(end)?;
                held.release()?;
            }
            for held in &mut self.held {
                if !held.ranges.is_empty() {
                    return Err(refused());
                }
                held.pin.check(end)?;
            }
            if file_id(stable_file)? != stable_id
                || native_stamp(stable_file, end).map_err(io_error)? != stable_security
                || snapshot(stable_file, end)? != stable_raw
            {
                return Err(refused());
            }
            original.check(end)?;
            before(end)?;
            self.finished = true;
            self.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check(
        &mut self,
        original: &mut ReleasedLease,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            before(end)?;
            if self.failed
                || !self.finished
                || self.end != Some(end)
                || self.held.len() != 5
                || self.token.as_ref() != Some(&native::current_identity()?)
            {
                return Err(refused());
            }
            original.check(end)?;
            let facts = original.facts();
            if self.epoch.as_ref() != Some(&facts.epoch)
                || self.claim.as_ref() != Some(&facts.claim_key)
                || self.facts.as_ref() != Some(&facts.objects_digest)
            {
                return Err(refused());
            }
            for held in &mut self.held {
                if !held.ranges.is_empty() {
                    return Err(refused());
                }
                held.pin.check(end)?;
            }
            before(end)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn data(
        &self,
    ) -> io::Result<serde_json::Value> {
        if self.failed || !self.finished {
            return Err(refused());
        }
        Ok(
            serde_json::json!({"purpose":"actual-released-five-range-readback-data","epoch":self.epoch,"claim":self.claim,"objectsDigest":self.facts,"observedRanges":5,"explicitProbeRelease":true,"runtimeAuthority":false}),
        )
    }
}
