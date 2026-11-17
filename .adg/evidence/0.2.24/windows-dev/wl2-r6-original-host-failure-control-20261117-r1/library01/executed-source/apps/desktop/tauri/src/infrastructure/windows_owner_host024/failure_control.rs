//! Debug-only original-host failure control, established before qualification.
//! Locator/replies are diagnostic DATA, never client restore or Closed proofs.
use super::empty_host::EmptyBootstrap;
use super::{decode, qpc, Operation, Request, ValidatedDeadline, WireBudget};
use crate::infrastructure::{
    hash, platform, runtime_executable::SharedRuntimeExecutable,
    windows_client_originals024::existing_qualification::BASE,
    windows_official_lifecycle024 as lifecycle, windows_official_runtime024 as native,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
const LOCATOR: &str = "qualification-failure-control.json";
const SCHEMA: &str = "windows-existing-failure-control-v1";
fn denied(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, e.to_string())
}
fn event(log: &mut File, name: &str, facts: Value) -> io::Result<()> {
    writeln!(
        log,
        "{}",
        json!({"stage":name,"facts":facts,"publicStartOpen":false})
    )?;
    log.sync_all()
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Locator {
    schema: String,
    instance_id: String,
    pipe_name: String,
    broker_pid: u32,
    broker_filetime: String,
    scope_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Hello {
    schema: u32,
    instance_id: String,
    manager_generation: String,
    challenge: String,
    scope_digest: String,
}
fn admit(
    request: &Request,
    hello: &Hello,
    used: &mut BTreeSet<String>,
    failed: bool,
    now: u64,
    freq: u64,
    instant: Instant,
) -> io::Result<ValidatedDeadline> {
    if request.instance_id != hello.instance_id
        || request.manager_generation != hello.manager_generation
        || request.challenge != hello.challenge
        || !matches!(
            request.operation,
            Operation::Status | Operation::ExplicitRecovery
        )
        || matches!(request.operation, Operation::ExplicitRecovery) && !failed
    {
        return Err(denied("fresh original-host failed-action binding required"));
    }
    let deadline = request.deadline_at(now, freq, instant)?;
    let keys = [
        request.request_id.clone(),
        request.budget.action_id.clone(),
        hello.challenge.clone(),
    ];
    if used.len() + keys.len() > 16384
        || keys.iter().any(|k| used.contains(k))
        || keys[0] == keys[1]
    {
        return Err(denied("replayed request/action/nonce"));
    }
    used.extend(keys);
    Ok(deadline)
}
pub(super) struct Control {
    pipe: native::PrivatePipe,
    locator: Locator,
    record: native::PrivateRecord,
    original_record: Vec<u8>,
    broker: native::RetainedPeer,
    image: PathBuf,
    home: PathBuf,
    data: PathBuf,
    pins: Vec<native::PinnedDirectory>,
}
impl Control {
    pub(super) fn create(c: &EmptyBootstrap, broker: native::RetainedPeer) -> io::Result<Self> {
        let instance_id = uuid::Uuid::new_v4().to_string();
        let scope_digest = hash::sha256_hex(
            &serde_json::to_vec(&json!({
                "bootstrap":c,"pid":broker.identity().pid,
                "filetime":broker.identity().creation_filetime.to_string(),
                "instance":instance_id
            }))
            .map_err(io::Error::other)?,
        );
        let pipe = native::PrivatePipe::create(&scope_digest)?;
        let locator = Locator {
            schema: SCHEMA.into(),
            instance_id,
            pipe_name: pipe.name().into(),
            broker_pid: broker.identity().pid,
            broker_filetime: broker.identity().creation_filetime.to_string(),
            scope_digest,
        };
        let original_record = serde_json::to_vec(&locator).map_err(io::Error::other)?;
        let record = native::PrivateRecord::create(&c.data.join(LOCATOR), &original_record, 65536)?;
        let pins = [&c.profile, &c.installation, &c.data, &c.home]
            .iter()
            .map(|p| native::PinnedDirectory::open(p))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self {
            pipe,
            locator,
            record,
            original_record,
            broker,
            image: c.installation.join("ocxd.exe"),
            home: c.home.clone(),
            data: c.data.clone(),
            pins,
        })
    }
    fn recheck(&mut self, peer: &native::RetainedPeer) -> io::Result<()> {
        self.broker.no_outer_job()?;
        self.broker.verify_image(&self.image)?;
        peer.verify_image(&self.image)?;
        peer.no_outer_job()?;
        if peer.identity().token != self.broker.identity().token
            || self.pipe.peer()?.identity() != peer.identity()
            || self.record.read()? != self.original_record
        {
            return Err(denied("original ordinary native peer/locator changed"));
        }
        for pin in &self.pins {
            pin.recheck()?;
        }
        Ok(())
    }
    pub(super) fn serve(
        mut self,
        log: &mut File,
        runtime: Option<&SharedRuntimeExecutable>,
        failed: bool,
        evidence_error: Option<io::Error>,
    ) -> io::Result<()> {
        let mut used = BTreeSet::new();
        let mut generation = 0u64;
        let mut attempted = false;
        let _ = event(
            log,
            "failure-control-listening",
            json!({"failed":failed,
            "evidenceError":evidence_error.map(|e|e.to_string()),"locator":LOCATOR,"normalStop":false}),
        );
        loop {
            let peer = match self.pipe.accept(Instant::now() + Duration::from_secs(5)) {
                Ok(peer) => peer,
                Err(e) => {
                    if e.kind() != io::ErrorKind::TimedOut {
                        let _ = event(
                            log,
                            "failure-control-accept-error",
                            json!({"error":e.to_string(),"raw":e.raw_os_error()}),
                        );
                        std::thread::park_timeout(Duration::from_millis(250));
                    }
                    continue;
                }
            };
            let handshake_end = Instant::now() + Duration::from_secs(5);
            let result = (|| -> io::Result<()> {
                self.recheck(&peer)?;
                generation = generation
                    .checked_add(1)
                    .ok_or_else(|| denied("generation exhausted"))?;
                let hello = Hello {
                    schema: 1,
                    instance_id: self.locator.instance_id.clone(),
                    manager_generation: generation.to_string(),
                    scope_digest: self.locator.scope_digest.clone(),
                    challenge: hash::sha256_hex(&rand::random::<[u8; 32]>()),
                };
                self.pipe.write_frame(
                    &serde_json::to_vec(&hello).map_err(io::Error::other)?,
                    handshake_end,
                )?;
                let request = decode(&self.pipe.read_frame(handshake_end)?)?;
                let sampled = Instant::now();
                let (now, freq) = qpc()?;
                let deadline = admit(&request, &hello, &mut used, failed, now, freq, sampled)?;
                self.recheck(&peer)?;
                let facts = match request.operation {
                    Operation::Status => runtime
                        .map(|r| lifecycle::qualification_trace(r, &self.home))
                        .unwrap_or_else(|| json!({"nativeOwner":false})),
                    Operation::ExplicitRecovery => {
                        if attempted {
                            return Err(denied(
                                "failed cleanup already attempted; retain original host",
                            ));
                        }
                        let runtime = runtime
                            .ok_or_else(|| denied("no original runtime; no cleanup authority"))?;
                        event(
                            log,
                            "original-job-forced-cleanup-intent",
                            json!({"requestId":request.request_id,
                            "actionId":request.budget.action_id,"normalStop":false,"closed":false}),
                        )?;
                        attempted = true;
                        let cleaned = lifecycle::qualification_failure_cleanup(
                            runtime,
                            &self.home,
                            deadline.deadline,
                        )?;
                        // Evidence failure retains the original host. Never re-force or
                        // synthesize a successful receipt on another request.
                        let mut receipt = platform::create_new_private_file(
                            &self.data.join("qualification-forced-cleanup.json"),
                            0o600,
                        )?;
                        receipt.write_all(
                            &serde_json::to_vec_pretty(&cleaned.report)
                                .map_err(io::Error::other)?,
                        )?;
                        receipt.sync_all()?;
                        event(
                            log,
                            "original-job-forced-cleanup-result",
                            cleaned.report.clone(),
                        )?;
                        cleaned.report
                    }
                    _ => unreachable!("admit rejects service actions"),
                };
                let reply = json!({"schema":1,"requestId":request.request_id,"scopeDigest":hello.scope_digest,
                    "facts":facts,"normalStop":false,"closed":false,"allAccepted":false,"publicStartOpen":false});
                self.pipe.write_frame(
                    &serde_json::to_vec(&reply).map_err(io::Error::other)?,
                    deadline.deadline,
                )?;
                // Keep the pipe connected until the client has consumed its reply.
                // This acknowledgment is delivery DATA; it grants no new action.
                let ack = self.pipe.read_frame(deadline.deadline)?;
                let expected = serde_json::to_vec(&json!({"received":request.request_id}))
                    .map_err(io::Error::other)?;
                if ack[4..] != expected {
                    return Err(denied("reply delivery unconfirmed"));
                }
                self.recheck(&peer)?;
                Ok(())
            })();
            if let Err(e) = &result {
                let _ = event(
                    log,
                    "failure-control-refused",
                    json!({"error":e.to_string(),"raw":e.raw_os_error(),
                    "cleanupAttempted":attempted,"normalStop":false,"closed":false}),
                );
            }
            let _ = self.pipe.disconnect();
            // Even a missing LivePhase tag can hide an interrupted native
            // capture prefix. Keep the original broker and all originals alive.
        }
    }
}
/// Invoked only by the exact debug internal command in the same ordinary
/// desktop. No target PID/path parameter, Job adoption, or elevation exists.
pub(super) fn request_cleanup() -> io::Result<Value> {
    let started = Instant::now();
    let (start, freq) = qpc()?;
    let end = started + Duration::from_secs(10);
    let base = std::path::Path::new(BASE);
    let image = base.join("I/ocxd.exe");
    let current = native::RetainedPeer::current()?;
    let token = &current.identity().token;
    if token.elevated
        || token.elevation_type != 1
        || token.session_id != 7
        || platform::home_dir().as_deref() != base.parent()
    {
        return Err(denied("exact ordinary qualification desktop required"));
    }
    current.no_outer_job()?;
    current.verify_image(&image)?;
    let bytes = native::PrivateRecord::locator_snapshot(&base.join("D").join(LOCATOR), 65536)?;
    let locator: Locator = serde_json::from_slice(&bytes).map_err(denied)?;
    if locator.schema != SCHEMA {
        return Err(denied("failure locator schema"));
    }
    let pipe = native::PrivatePipe::connect_local(&locator.pipe_name)?;
    let peer = pipe.peer()?;
    peer.no_outer_job()?;
    peer.verify_image(&image)?;
    if peer.identity().token != *token
        || peer.identity().pid != locator.broker_pid
        || peer.identity().creation_filetime.to_string() != locator.broker_filetime
    {
        return Err(denied("native original broker binding changed"));
    }
    let hello: Hello = serde_json::from_slice(&pipe.read_frame(end)?[4..]).map_err(denied)?;
    if hello.schema != 1
        || hello.instance_id != locator.instance_id
        || hello.scope_digest != locator.scope_digest
    {
        return Err(denied("native challenge scope changed"));
    }
    let ticks = start
        .checked_add(freq.checked_mul(10).ok_or_else(|| denied("QPC overflow"))?)
        .ok_or_else(|| denied("QPC overflow"))?
        .to_string();
    let request = Request {
        schema: 1,
        request_id: uuid::Uuid::new_v4().to_string(),
        instance_id: hello.instance_id,
        manager_generation: hello.manager_generation,
        challenge: hello.challenge,
        operation: Operation::ExplicitRecovery,
        budget: WireBudget {
            action_id: uuid::Uuid::new_v4().to_string(),
            started_qpc: start.to_string(),
            deadline_qpc: ticks.clone(),
            stop_deadline_qpc: ticks,
            qpc_frequency: freq.to_string(),
        },
    };
    pipe.write_frame(&serde_json::to_vec(&request).map_err(denied)?, end)?;
    let reply: Value = serde_json::from_slice(&pipe.read_frame(end)?[4..]).map_err(denied)?;
    if pipe.peer()?.identity() != peer.identity()
        || reply["requestId"] != request.request_id
        || reply["scopeDigest"] != locator.scope_digest
        || reply["normalStop"] != false
        || reply["closed"] != false
        || reply["allAccepted"] != false
        || reply["publicStartOpen"] != false
        || reply["facts"]["forced"] != true
    {
        return Err(denied("forced-only reply unconfirmed"));
    }
    pipe.write_frame(
        &serde_json::to_vec(&json!({"received":request.request_id})).map_err(denied)?,
        end,
    )?;
    Ok(reply)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> (Hello, Request) {
        let h = Hello {
            schema: 1,
            instance_id: uuid::Uuid::new_v4().to_string(),
            manager_generation: "1".into(),
            challenge: "a".repeat(64),
            scope_digest: "b".repeat(64),
        };
        let r = Request {
            schema: 1,
            request_id: uuid::Uuid::new_v4().to_string(),
            instance_id: h.instance_id.clone(),
            manager_generation: h.manager_generation.clone(),
            challenge: h.challenge.clone(),
            operation: Operation::ExplicitRecovery,
            budget: WireBudget {
                action_id: uuid::Uuid::new_v4().to_string(),
                started_qpc: "1000".into(),
                deadline_qpc: "11000".into(),
                stop_deadline_qpc: "11000".into(),
                qpc_frequency: "1000".into(),
            },
        };
        (h, r)
    }
    #[test]
    fn wl2_r6_failure_control_rejects_success_service_actions_stale_binding_and_replay() {
        let (h, mut r) = request();
        let mut used = BTreeSet::new();
        assert!(admit(&r, &h, &mut used, false, 6000, 1000, Instant::now()).is_err());
        for operation in [
            Operation::Start,
            Operation::Stop,
            Operation::Restart,
            Operation::Detach,
        ] {
            r.operation = operation;
            assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        }
        r.operation = Operation::ExplicitRecovery;
        r.manager_generation = "2".into();
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        r.manager_generation = h.manager_generation.clone();
        r.challenge = "c".repeat(64);
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        r.challenge = h.challenge.clone();
        r.instance_id = uuid::Uuid::new_v4().to_string();
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        r.instance_id = h.instance_id.clone();
        let now = Instant::now();
        let d = admit(&r, &h, &mut used, true, 6000, 1000, now).unwrap();
        assert_eq!(d.deadline.duration_since(now), Duration::from_secs(5));
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, now).is_err());
        r.request_id = uuid::Uuid::new_v4().to_string();
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, now).is_err());
    }
    #[test]
    fn wl2_r6_failure_control_rejects_expired_extended_or_duplicate_action_before_consuming() {
        let (h, mut r) = request();
        let mut used = BTreeSet::new();
        assert!(admit(&r, &h, &mut used, true, 11000, 1000, Instant::now()).is_err());
        r.budget.deadline_qpc = "11001".into();
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        r.budget.deadline_qpc = "11000".into();
        r.budget.action_id = r.request_id.clone();
        assert!(admit(&r, &h, &mut used, true, 6000, 1000, Instant::now()).is_err());
        assert!(used.is_empty());
    }
}
