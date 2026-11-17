//! Own-read-handle close mechanics only. Native candidate DATA/closed files do
//! NOT qualify an actual Stop, source/action, canonical result or successor.
//! Production must join the exact same canonical route close intent/result.
use super::*;
use crate::infrastructure::windows_client_originals024::{
    bounded::SnapshotData,
    handle_release::{ExpectedHandleInventory, NativeHandleBatch},
};
use crate::infrastructure::{
    hash,
    windows_official_runtime024::{self as native, TokenIdentity},
};
use serde::Serialize;
use std::path::PathBuf;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RouteData {
    home: PathBuf,
    config_security: SnapshotData,
    config_bytes: usize,
    config_sha256: String,
    route_digest: String,
    inventory_digest: String,
    handle_count: usize,
}
pub(in crate::infrastructure::windows_client_originals024) struct RetainedRouteClose {
    pin: Option<RoutePin>,
    handles: NativeHandleBatch,
    expected: Option<ExpectedHandleInventory>,
    token: Option<TokenIdentity>,
    facts: Option<RouteData>,
    routes: Vec<String>,
    end: Instant,
    completed: Option<Instant>,
    poisoned: bool,
}
impl RetainedRouteClose {
    /// Move the ORIGINAL owned pin into a reachable retained candidate before
    /// any fallible read. No IO, handle clone, close or native grant occurs.
    pub(in crate::infrastructure::windows_client_originals024) fn retain(
        pin: RoutePin,
        end: Instant,
    ) -> Self {
        Self {
            pin: Some(pin),
            handles: NativeHandleBatch::new(end),
            expected: None,
            token: None,
            facts: None,
            routes: Vec::new(),
            end,
            completed: None,
            poisoned: false,
        }
    }
    fn exact_end(&self, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.poisoned || end != self.end {
            return Err(refused());
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.exact_end(end)?;
            if self.facts.is_some() || self.expected.is_some() || self.completed.is_some() {
                return Err(refused());
            }
            let pin = self.pin.as_mut().ok_or_else(refused)?;
            pin.check(end)?;
            let token = native::current_identity()?;
            if token.elevated || token.elevation_type == 2 {
                return Err(refused());
            }
            let id = pin.security.config_close_identity(end)?;
            let security = pin.security.snapshot_data()?;
            let mut expected = ExpectedHandleInventory::new(end);
            pin.security
                .observe_config_close_inventory(&mut expected, "route/security", end)?;
            expected.observe_directory("route/home", &pin.home, end)?;
            // READ_CONTROL-only observers do not substitute primary read
            // sharing. Keep the actual deny-write primary until the LAST close.
            expected.observe("route/config".into(), &pin.file, id, false, false, end)?;
            let data = RouteData {
                home: pin.home.path().into(),
                config_security: security,
                config_bytes: pin.raw.len(),
                config_sha256: hash::sha256_hex(&pin.raw),
                route_digest: hash::sha256_hex(
                    &serde_json::to_vec(&pin.routes).map_err(|_| refused())?,
                ),
                inventory_digest: String::new(),
                handle_count: 0,
            };
            self.routes = pin.routes.clone();
            self.facts = Some(data);
            self.expected = Some(expected);
            self.token = Some(token);
            // From this point every File has exactly one retained destination;
            // even failed ID observation poisons the batch without early return
            // or dropping a file between take and full transfer.
            let RoutePin {
                home,
                file,
                security,
                raw: _,
                routes: _,
            } = self.pin.take().ok_or_else(refused)?;
            security.transfer_config_close_files(&mut self.handles, "route/security");
            for (i, file) in home.into_owned_files().into_iter().enumerate() {
                self.handles
                    .retain_observed(format!("route/home/{i}"), file, true, false);
            }
            self.handles
                .retain("route/config".into(), file, id, false, false);
            self.expected
                .as_ref()
                .ok_or_else(refused)?
                .matches(&self.handles, end)?;
            let inventory = self.handles.inventory()?;
            let facts = self.facts.as_mut().ok_or_else(refused)?;
            facts.inventory_digest =
                hash::sha256_hex(&serde_json::to_vec(&inventory["rows"]).map_err(|_| refused())?);
            facts.handle_count = inventory["rows"].as_array().ok_or_else(refused)?.len();
            self.check_before_close(end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_before_close(
        &self,
        end: Instant,
    ) -> io::Result<()> {
        self.exact_end(end)?;
        if self.pin.is_some()
            || self.completed.is_some()
            || self.token.as_ref() != Some(&native::current_identity()?)
        {
            return Err(refused());
        }
        self.expected
            .as_ref()
            .ok_or_else(refused)?
            .matches(&self.handles, end)?;
        let facts = self.facts.as_ref().ok_or_else(refused)?;
        self.handles.check_config_read(
            "route/config",
            &facts.config_security,
            facts.config_bytes,
            &facts.config_sha256,
            end,
        )?;
        let inventory = self.handles.inventory()?;
        if inventory["rows"].as_array().ok_or_else(refused)?.len() != facts.handle_count
            || hash::sha256_hex(&serde_json::to_vec(&inventory["rows"]).map_err(|_| refused())?)
                != facts.inventory_digest
        {
            return Err(refused());
        }
        before(end)
    }
    /// This closes ONLY this candidate's already-owned read handles. It does
    /// not append an intent, authorize mutation or attest actual normal Stop.
    pub(in crate::infrastructure::windows_client_originals024) fn close_owned(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        self.close_inner(0, end)
    }
    fn close_inner(&mut self, fault: u8, end: Instant) -> io::Result<()> {
        let result = (|| {
            self.check_before_close(end)?;
            #[cfg(test)]
            if fault == 85 {
                self.handles.protect_first_for_test()?;
            }
            self.handles.close_all(fault, end)?;
            self.expected
                .as_ref()
                .ok_or_else(refused)?
                .matches(&self.handles, end)?;
            if !self.handles.completed() {
                return Err(refused());
            }
            #[cfg(test)]
            if fault == 82 {
                return Err(io::Error::other(
                    "injected after all actual route handles closed before private completion",
                ));
            }
            self.completed = Some(Instant::now());
            self.check_history(end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_history(
        &self,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        if self.poisoned
            || self.pin.is_some()
            || !self.completed.is_some_and(|t| t < self.end)
            || !self.handles.completed()
            || self.token.as_ref() != Some(&native::current_identity()?)
        {
            return Err(refused());
        }
        let facts = self.facts.as_ref().ok_or_else(refused)?;
        let inventory = self.handles.inventory()?;
        if inventory["rows"].as_array().ok_or_else(refused)?.len() != facts.handle_count
            || hash::sha256_hex(&serde_json::to_vec(&inventory["rows"]).map_err(|_| refused())?)
                != facts.inventory_digest
            || hash::sha256_hex(&serde_json::to_vec(&self.routes).map_err(|_| refused())?)
                != facts.route_digest
        {
            return Err(refused());
        }
        before(end)
    }
    fn poison(&mut self) {
        self.poisoned = true;
        self.handles.invalidate();
    }
    pub(in crate::infrastructure::windows_client_originals024) fn data(
        &self,
    ) -> io::Result<serde_json::Value> {
        let facts = self.facts.as_ref().ok_or_else(refused)?;
        Ok(
            serde_json::json!({"schema":"windows-anchored-route-close-data-v3","facts":facts,"nativeAuthority":false,"typedClosed":false}),
        )
    }
    pub(in crate::infrastructure::windows_client_originals024) fn trace(
        &self,
    ) -> serde_json::Value {
        serde_json::json!({"originalPinRetained":self.pin.is_some(),"poisoned":self.poisoned,"completedInOriginalWindow":self.completed.is_some_and(|t|t<self.end),
            "handles":self.handles.trace(),"nativeAuthority":false,"typedClosed":false,"canonicalRouteCloseIntegrated":false})
    }
}
#[cfg(test)]
#[path = "close_tests.rs"]
mod tests;
