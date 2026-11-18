//! A NEW original-host endpoint. Never connects to oldloaded357 or adopts its
//! namespace/Job. Install before capture, keep original RuntimeArc and native
//! peer/locator handles for the lifetime of the host. No cleanup/shutdown RPC.
use super::wire::{self, AckV2, CommandV2, HelloV2, LocatorV2, ReplyV2, RequestV2};
use crate::{
    infrastructure::{
        hash, platform, runtime_executable::SharedRuntimeExecutable,
        windows_official_lifecycle024 as lifecycle, windows_official_runtime024 as native,
    },
    modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
pub(super) const LOCATOR: &str = "before-resume-private-control-v2.json";
fn denied(reason: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason.to_string())
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
// Follow only the worker registered by this action. Projection contention can
// clear while that worker proceeds. Unknown states and all other errors stop.
fn await_worker_result(
    end: Instant,
    mut observe: impl FnMut(Instant) -> io::Result<serde_json::Value>,
) -> io::Result<serde_json::Value> {
    loop {
        before(end)?;
        match observe(end) {
            Ok(facts) if !matches!(facts["phase"].as_str(), Some("Registered" | "Running")) => {
                before(end)?;
                return Ok(facts);
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }
        thread::sleep(
            end.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(5)),
        );
    }
}

#[cfg(test)]
mod worker_result_tests {
    use super::*;

    #[test]
    fn wl2_r7_private_worker_wait_uses_same_deadline_after_projection_contention() {
        let projection = Mutex::new("ReturnedForValidation");
        let mut publisher = Some(projection.lock().unwrap());
        let end = Instant::now() + Duration::from_secs(2);
        let mut polls = 0;
        let result = await_worker_result(end, |registered_end| {
            assert_eq!(registered_end, end);
            polls += 1;
            let observed = projection.try_lock();
            drop(publisher.take());
            let phase = observed.map_err(|error| {
                assert!(matches!(error, std::sync::TryLockError::WouldBlock));
                io::Error::from(io::ErrorKind::WouldBlock)
            })?;
            Ok(serde_json::json!({"phase":*phase}))
        })
        .unwrap();
        assert_eq!(polls, 2);
        assert_eq!(result["phase"], "ReturnedForValidation");
    }

    #[test]
    fn wl2_r7_private_worker_wait_preserves_terminal_unknown_error_and_expiry() {
        let end = Instant::now() + Duration::from_secs(2);
        let mut polls = 0;
        let unknown = await_worker_result(end, |_| {
            polls += 1;
            Ok(serde_json::json!({"phase":"LostWorkerUnknown"}))
        })
        .unwrap();
        let raw = await_worker_result(end, |_| {
            polls += 1;
            Err(io::Error::from_raw_os_error(5))
        })
        .unwrap_err();
        assert_eq!(polls, 2);
        assert_eq!(unknown["phase"], "LostWorkerUnknown");
        assert_eq!(raw.raw_os_error(), Some(5));

        let cutoff = Instant::now() + Duration::from_millis(30);
        let timeout = await_worker_result(cutoff, |registered_end| {
            assert_eq!(registered_end, cutoff);
            Err(io::ErrorKind::WouldBlock.into())
        })
        .unwrap_err();
        assert_eq!(timeout.kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            await_worker_result(cutoff, |_| panic!("expired window observed again"))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
    }
}

fn event(log: &mut File, stage: &str, facts: serde_json::Value) -> io::Result<()> {
    writeln!(
        log,
        "{}",
        serde_json::json!({"stage":stage,"facts":facts,"publicStartOpen":false,"normalStop":false,"closed":false})
    )?;
    log.sync_all()
}
fn peer_data(peer: &native::RetainedPeer) -> serde_json::Value {
    let identity = peer.identity();
    serde_json::json!({"pid":identity.pid,"creationFiletime":identity.creation_filetime.to_string(),"sid":identity.token.sid,"sessionId":identity.token.session_id,"elevated":identity.token.elevated,"elevationType":identity.token.elevation_type,"image":identity.executable,"imageVolume":identity.executable_id.volume,"imageFileId":identity.executable_id.index.to_string()})
}
type OriginalRuntime = Arc<Mutex<Option<SharedRuntimeExecutable>>>;
pub(in crate::infrastructure::windows_owner_host024) struct Registration {
    worker: thread::JoinHandle<()>,
    runtime: OriginalRuntime,
    installation: PathBuf,
}
impl Registration {
    pub(in crate::infrastructure::windows_owner_host024) fn check_alive(&self) -> io::Result<()> {
        if self.worker.is_finished() {
            Err(denied(
                "original V2 control worker lost; keep original host/Owner",
            ))
        } else {
            Ok(())
        }
    }
    pub(in crate::infrastructure::windows_owner_host024) fn bind_runtime(
        &self,
        runtime: SharedRuntimeExecutable,
    ) -> io::Result<()> {
        self.check_alive()?;
        if runtime.managed_installation_root().as_ref() != Some(&self.installation) {
            return Err(denied("not original installation runtime"));
        }
        let mut original = self
            .runtime
            .try_lock()
            .map_err(|_| denied("original runtime binding busy"))?;
        if original.is_some() {
            return Err(denied("original runtime already bound; no replacement"));
        }
        *original = Some(runtime);
        self.check_alive()
    }
}
pub(in crate::infrastructure::windows_owner_host024) fn install_before_capture(
    c: &super::super::empty_host::EmptyBootstrap,
) -> io::Result<Registration> {
    let runtime = Arc::new(Mutex::new(None));
    let control = Control::create(c, runtime.clone())?;
    let mut log = platform::create_new_private_file(
        &c.data.join("before-resume-private-control-v2-events.jsonl"),
        0o600,
    )?;
    event(
        &mut log,
        "v2-original-endpoint-created-before-capture",
        serde_json::json!({"locator":LOCATOR,"broker":peer_data(&control.broker),"noOuterJobObservedAtAdmission":true,"originalOwnerCreated":false,"runtimeAuthority":false}),
    )?;
    let worker = thread::Builder::new()
        .name("windows024-original-private-v2".into())
        .spawn(move || control.serve(&mut log))?;
    let registration = Registration {
        worker,
        runtime,
        installation: c.installation.clone(),
    };
    registration.check_alive()?;
    Ok(registration)
}
struct Control {
    #[cfg(test)]
    partial_capture_fixture: bool,
    pipe: native::PrivatePipe,
    locator: LocatorV2,
    record: native::PrivateRecord,
    original_record: Vec<u8>,
    broker: native::RetainedPeer,
    image: PathBuf,
    home: PathBuf,
    pins: Vec<native::PinnedDirectory>,
    runtime: OriginalRuntime,
}
impl Control {
    fn create(
        c: &super::super::empty_host::EmptyBootstrap,
        runtime: OriginalRuntime,
    ) -> io::Result<Self> {
        let broker = native::RetainedPeer::current()?;
        let token = &broker.identity().token;
        if c.schema != 1
            || token.sid != c.expected_sid
            || token.session_id != c.expected_session
            || token.elevated
            || token.elevation_type != 1
            || platform::home_dir().as_deref() != Some(c.profile.as_path())
        {
            return Err(denied(
                "original ordinary host/bootstrap/runtime scope required",
            ));
        }
        broker.no_outer_job()?;
        let image = c.installation.join("ocxd.exe");
        broker.verify_image(&image)?;
        let pins = [
            native::PinnedDirectory::open(&c.profile)?,
            native::PinnedDirectory::open_private(&c.data)?,
            native::PinnedDirectory::open_private(&c.home)?,
            native::PinnedDirectory::open(&c.installation)?,
        ]
        .into_iter()
        .collect::<Vec<_>>();
        let instance_id = uuid::Uuid::new_v4().to_string();
        let scope_digest=hash::sha256_hex(&serde_json::to_vec(&serde_json::json!({"schema":wire::SCHEMA,"bootstrap":c,"pid":broker.identity().pid,"filetime":broker.identity().creation_filetime.to_string(),"instance":instance_id})).map_err(io::Error::other)?);
        let pipe = native::PrivatePipe::create(&scope_digest)?;
        let locator = LocatorV2 {
            schema: wire::SCHEMA.into(),
            instance_id,
            pipe_name: pipe.name().into(),
            broker_pid: broker.identity().pid,
            broker_filetime: broker.identity().creation_filetime.to_string(),
            scope_digest,
        };
        let original_record = serde_json::to_vec(&locator).map_err(io::Error::other)?;
        LocatorV2::decode(&original_record)?;
        let record = native::PrivateRecord::create(&c.data.join(LOCATOR), &original_record, 65536)?;
        Ok(Self {
            #[cfg(test)]
            partial_capture_fixture: c.scope == crate::infrastructure::windows_client_originals024::partial_capture_fixture::SCOPE,
            pipe,
            locator,
            record,
            original_record,
            broker,
            image,
            home: c.home.clone(),
            pins,
            runtime,
        })
    }
    fn recheck(&mut self, peer: &native::RetainedPeer, end: Instant) -> io::Result<()> {
        before(end)?;
        self.broker.no_outer_job()?;
        self.broker.verify_image(&self.image)?;
        peer.no_outer_job()?;
        peer.verify_image(&self.image)?;
        if peer.identity().token != self.broker.identity().token
            || self.pipe.peer()?.identity() != peer.identity()
            || self.record.read()? != self.original_record
        {
            return Err(denied("original ordinary peer/locator changed"));
        }
        for pin in &self.pins {
            pin.recheck()?;
        }
        before(end)
    }
    fn dispatch(
        &self,
        request: &wire::AdmittedV2,
        attempted: &mut bool,
    ) -> io::Result<serde_json::Value> {
        let end = request.deadline.end;
        before(end)?;
        let runtime = self
            .runtime
            .try_lock()
            .map_err(|_| denied("original runtime binding busy"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| denied("original runtime not yet bound; no Owner authority"))?;
        match request.command {
            CommandV2::ReadWorkerDiagnostic => {
                let snapshot = lifecycle::before_resume_worker_status(&runtime, &self.home, end)
                    .map_err(io::Error::other)?;
                #[cfg(test)]
                if self.partial_capture_fixture {
                    let status = super::super::partial_capture_tests::native_status(
                        &runtime, &self.home, end,
                    )?;
                    before(end)?;
                    return Ok(
                        serde_json::json!({"purpose":"worker-metadata-only-not-full-history","snapshot":snapshot,"sameSampleStatus":status,"recoveryRequired":true,"runtimeAuthority":false}),
                    );
                }
                before(end)?;
                Ok(
                    serde_json::json!({"purpose":"worker-metadata-only-not-full-history","snapshot":snapshot,"recoveryRequired":true,"runtimeAuthority":false}),
                )
            }
            CommandV2::ReadCanonicalHistory => {
                lifecycle::read_canonical_before_resume_history(&runtime, &self.home, end)
                    .map_err(io::Error::other)
            }
            CommandV2::BeginCanonicalAbort => {
                if *attempted {
                    return Err(denied(
                        "original explicit recovery already attempted, no new window",
                    ));
                }
                *attempted = true;
                let window = BeforeResumeRecoveryWindow::begin(request.deadline.started, end)
                    .map_err(io::Error::other)?;
                if window.deadline() != end {
                    return Err(denied("wire original recovery end changed"));
                }
                let worker =
                    lifecycle::start_canonical_before_resume_recovery(&runtime, &self.home, window)
                        .map_err(io::Error::other)?;
                let observed = await_worker_result(end, |end| worker.observe(end))?;
                if observed["phase"] != "ReturnedForValidation" {
                    let facts = lifecycle::before_resume_worker_status(&runtime, &self.home, end)
                        .ok()
                        .flatten()
                        .unwrap_or(observed);
                    return Err(denied(format!(
                        "original recovery retained/unknown; worker DATA={facts}"
                    )));
                }
                let result =
                    lifecycle::validate_canonical_before_resume_return(&runtime, &self.home, end)
                        .map_err(io::Error::other)?
                        .ok_or_else(|| denied("original return not delivered for validation"))?;
                before(end)?;
                Ok(
                    serde_json::json!({"purpose":"actual-original-slot-validated-before-resume-abort","transportActionId":request.transport_action_id,"history":result,"recoveryRequired":true,"normalStop":false,"closed":false,"runtimeAuthority":false}),
                )
            }
        }
    }
    fn serve(mut self, log: &mut File) -> ! {
        let mut used = BTreeSet::new();
        let mut generation = 0u64;
        let mut attempted = false;
        loop {
            let peer = match self.pipe.accept(Instant::now() + Duration::from_secs(5)) {
                Ok(peer) => peer,
                Err(error) => {
                    if error.kind() != io::ErrorKind::TimedOut {
                        let _ = event(
                            log,
                            "v2-native-accept-refused",
                            serde_json::json!({"raw":error.raw_os_error(),"error":error.to_string()}),
                        );
                        thread::park_timeout(Duration::from_millis(100));
                    }
                    continue;
                }
            };
            let handshake_end = Instant::now() + Duration::from_secs(5);
            let result = (|| {
                self.recheck(&peer, handshake_end)?;
                generation = generation
                    .checked_add(1)
                    .ok_or_else(|| denied("original native generation exhausted"))?;
                let hello = HelloV2 {
                    schema: wire::SCHEMA.into(),
                    instance_id: self.locator.instance_id.clone(),
                    manager_generation: generation.to_string(),
                    challenge: hash::sha256_hex(&rand::random::<[u8; 32]>()),
                    scope_digest: self.locator.scope_digest.clone(),
                };
                hello.validate()?;
                self.pipe.write_frame(
                    &serde_json::to_vec(&hello).map_err(io::Error::other)?,
                    handshake_end,
                )?;
                let request = RequestV2::decode(&self.pipe.read_frame(handshake_end)?)?
                    .admit(&hello, &mut used)?;
                self.recheck(&peer, request.deadline.end)?;
                event(
                    log,
                    "v2-original-request-intent",
                    serde_json::json!({"requestId":request.request_id,"transportActionId":request.transport_action_id,"instance":self.locator.instance_id,"peer":peer_data(&peer),"broker":peer_data(&self.broker),"bothNoOuterJobObservedAtAdmission":true,"nativePeerAndLocatorRechecked":true}),
                )?;
                before(request.deadline.end)?;
                let facts = self.dispatch(&request, &mut attempted);
                let reply = match &facts {
                    Ok(data) => ReplyV2::result(&request, &self.locator.scope_digest, data.clone()),
                    Err(error) => ReplyV2::refused(&request, &self.locator.scope_digest, error),
                };
                event(
                    log,
                    "v2-original-request-result",
                    serde_json::to_value(&reply).map_err(io::Error::other)?,
                )?;
                self.recheck(&peer, request.deadline.end)?;
                self.pipe.write_frame(
                    &serde_json::to_vec(&reply).map_err(io::Error::other)?,
                    request.deadline.end,
                )?;
                AckV2::check(
                    &self.pipe.read_frame(request.deadline.end)?,
                    &request,
                    &self.locator.scope_digest,
                )?;
                // Every native identity/source fence already ran before and
                // after the action and before the result frame. ACK is delivery
                // DATA only; an exiting exact client cannot revoke an already
                // observed result or authorize another original action.
                before(request.deadline.end)
            })();
            if let Err(error) = result {
                let _ = event(
                    log,
                    "v2-native-request-refused-retained",
                    serde_json::json!({"raw":error.raw_os_error(),"kind":format!("{:?}",error.kind()),"error":error.to_string(),"recoveryAttempted":attempted}),
                );
            }
            let _ = self.pipe.disconnect();
            // No client disconnect, failed evidence/response or elapsed end can
            // drop an original Owner or restart/force/release its retained work.
        }
    }
}
