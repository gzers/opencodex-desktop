//! Sealed successful original-owner release, not Drop/record/PID inference.
//! Exact native file generations and all four SQLite exclusion scopes are
//! checked again without closing an auxiliary pin during a held transaction.
use super::*;

pub struct ReleasedCoordination {
    pub(super) runtime: SharedRuntimeExecutable,
    pub(super) profile: PathBuf,
    pub(super) home: PathBuf,
    pub(super) logical: PathBuf,
    pub(super) paths: [PathBuf; 4],
    pub(super) keys: [LockIdentity; 4],
    pub(super) deadline: Instant,
}
impl ReleasedCoordination {
    pub fn prove(
        &self,
        context: &crate::state::ProcessContext,
        physical: &Path,
        deadline: Instant,
    ) -> AppResult<()> {
        let end = deadline.min(self.deadline);
        if Instant::now() >= end
            || !std::sync::Arc::ptr_eq(&context.runtime, &self.runtime)
            || context.working_directory != self.profile
            || context.opencodex_home != self.home
            || self.logical != context.working_directory.join(".codex")
            || self.paths[0] != physical.join(".opencodex-native-main.claim.sqlite")
        {
            return Err(fail("client_native_release_scope"));
        }
        let mut pins = Vec::new();
        for (path, key) in self.paths.iter().zip(&self.keys) {
            let pin = CoordinationFilePin::open(path).map_err(io)?;
            if pin.key() != (key.device.clone(), key.inode.clone()) {
                return Err(fail("client_native_release_generation"));
            }
            pins.push(pin);
        }
        let mut probe = Probe {
            connections: Vec::new(),
        };
        for path in &self.paths {
            if Instant::now() >= end {
                return Err(fail("client_native_release_timeout"));
            }
            probe.connections.push(open_connection(path)?);
            let connection = probe.connections.last().unwrap();
            begin(connection, "BEGIN EXCLUSIVE")?;
            if connection.is_autocommit() {
                return Err(fail("client_native_release_scope"));
            }
            for pin in &pins {
                pin.recheck().map_err(io)?;
            }
        }
        probe.finish()?;
        for pin in &pins {
            pin.recheck().map_err(io)?;
        }
        if Instant::now() >= end {
            return Err(fail("client_native_release_timeout"));
        }
        Ok(())
    }
}
struct Probe {
    connections: Vec<rusqlite::Connection>,
}
impl Probe {
    fn finish(&mut self) -> AppResult<()> {
        let mut failed = false;
        while let Some(connection) = self.connections.pop() {
            if connection.execute_batch("ROLLBACK").is_err() {
                failed = true;
            }
            if connection.close().is_err() {
                failed = true;
            }
        }
        if failed {
            Err(fail("client_native_release_probe"))
        } else {
            Ok(())
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
