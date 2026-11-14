//! Typed HTTP facts for an unchanged official runtime. No OS-exit or client
//! restoration authority is produced by these methods or their native callback.
use super::*;
use std::{sync::Arc, time::Instant};
/// Implement only by the production Windows owner of original retained handles.
/// The callback is consulted after HMAC and again immediately before POST on the
/// SAME TCP connection. A copied PID/record/ReadyObservation is not an owner.
pub trait RetainedService {
    fn verify_live(&mut self, pid: u32, port: u16, record_sha256: &str)
        -> Result<(), CollectError>;
}
/// A single completed health/readiness/record observation, not a durable token.
/// No serde/Clone/public fields, secrets or normal-exit assertions.
pub struct ServiceObservation {
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    source: crate::modules::runtime::install::ManagedRuntimeIdentity,
    pid: u32,
    port: u16,
    ready: bool,
    record_sha256: String,
    deadline: Instant,
}
impl ServiceObservation {
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn ready(&self) -> bool {
        self.ready
    }
    pub fn version(&self) -> &str {
        &self.source.package_version
    }
    pub fn record_sha256(&self) -> &str {
        &self.record_sha256
    }
    pub fn check_current(
        &self,
        runtime: &SharedRuntimeExecutable,
        home: &Path,
    ) -> Result<(), CollectError> {
        if Instant::now() >= self.deadline {
            return Err(CollectError::Timeout);
        }
        if !Arc::ptr_eq(runtime, &self.runtime) || home != self.home {
            return Err(CollectError::Unreachable);
        }
        let (current, bytes) = record(home)?;
        if current.pid != self.pid
            || current.port != self.port
            || crate::infrastructure::hash::sha256_hex(&bytes) != self.record_sha256
            || runtime.managed_program_identity().map_err(unavailable)? != self.source
        {
            return Err(CollectError::Unreachable);
        }
        if Instant::now() >= self.deadline {
            return Err(CollectError::Timeout);
        }
        Ok(())
    }
}
/// Status remains at most5 seconds, nested inside the action's original absolute
/// deadline. Unknown/missing/invalid source or record is never a Ready result.
pub async fn observe_official024(
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    parent_deadline: Instant,
) -> Result<ServiceObservation, CollectError> {
    let deadline = parent_deadline.min(Instant::now() + status_collect_timeout());
    tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), async move {
        if Instant::now() >= deadline {
            return Err(CollectError::Timeout);
        }
        let source = runtime.managed_program_identity().map_err(unavailable)?;
        if source.package_version != "2.99.0"
            || runtime.executable().as_ref() != Some(&source.entry)
        {
            return Err(CollectError::Unreachable);
        }
        let (expected, bytes) = record(&home)?;
        let value = observe(&home, &expected, &bytes, &source.package_version).await?;
        let result = ServiceObservation {
            runtime: runtime.clone(),
            home: home.clone(),
            source,
            pid: expected.pid,
            port: expected.port,
            ready: value["ready"] == true,
            record_sha256: crate::infrastructure::hash::sha256_hex(&bytes),
            deadline,
        };
        result.check_current(&runtime, &home)?;
        Ok(result)
    })
    .await
    .map_err(|_| CollectError::Timeout)?
}
/// Only the official authenticated POST response. Never supplies a native
/// normal-exit, released-lock, complete-process-set or restored-client proof.
pub struct StopAccepted {
    pid: u32,
    port: u16,
    record_sha256: String,
}
impl StopAccepted {
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn record_sha256(&self) -> &str {
        &self.record_sha256
    }
}
/// Independent of maintenance-package desktop_lifetime. The actual owner checks
/// SID/FILETIME/Job membership/listeners from original handles via RetainedService;
/// official unchanged HMAC, record/source stability, admin capability and POST
/// response checks stay in this shared transport. No fallback to a new socket.
pub async fn request_stop_official024(
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    pid: u32,
    deadline: Instant,
    owner: &mut impl RetainedService,
    phase: impl Fn(&'static str),
) -> Result<StopAccepted, CollectError> {
    tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), async move {
        if Instant::now() >= deadline {
            return Err(CollectError::Timeout);
        }
        let identity = runtime.managed_program_identity().map_err(unavailable)?;
        if identity.package_version != "2.99.0" {
            return Err(CollectError::Unreachable);
        }
        let checked_runtime = runtime.clone();
        let checked_home = home.clone();
        let mut record_sha256 = None;
        let accepted = stop_official_with_checks(
            runtime,
            home,
            pid,
            phase,
            |_, _, r, _, _| {
                // The record digest is filled by the independent record-bound check
                // below; capturing this does not claim anything about process exit.
                Ok((r.pid, r.port))
            },
            |r, raw| {
                let digest = crate::infrastructure::hash::sha256_hex(raw);
                if record_sha256.as_ref().is_some_and(|old| old != &digest) {
                    return Err(CollectError::Unreachable);
                }
                owner.verify_live(r.pid, r.port, &digest)?;
                // Native enumeration can take time. Close its read window on
                // the shared source/record before releasing the reusable token
                // or sending POST; the legacy no-hook path is unchanged.
                if record(&checked_home)?.1 != raw
                    || checked_runtime
                        .managed_program_identity()
                        .map_err(unavailable)?
                        != identity
                {
                    return Err(CollectError::Unreachable);
                }
                if Instant::now() >= deadline {
                    return Err(CollectError::Timeout);
                }
                record_sha256 = Some(digest);
                Ok(())
            },
        )
        .await?;
        if Instant::now() >= deadline {
            return Err(CollectError::Timeout);
        }
        // verify_live saw the same exact record at both original checkpoints.
        // Expose its digest by retaining the first checked value, not by reading
        // a now removed/replaced record after a successful official Stop.
        Ok(StopAccepted {
            pid: accepted.0,
            port: accepted.1,
            record_sha256: record_sha256.ok_or(CollectError::Unreachable)?,
        })
    })
    .await
    .map_err(|_| CollectError::Timeout)?
}
