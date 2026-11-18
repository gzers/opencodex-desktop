//! Exact isolated debug qualification of actual product broker+official Owner.
//! Not an empty host, not public Start admission, not a serialized native proof.
use super::empty_host::EmptyBootstrap;
use crate::{
    infrastructure::{
        hash, platform,
        runtime_executable::SharedRuntimeExecutable,
        windows_client_originals024::existing_qualification::{ExistingQualification, BASE, SCOPE},
        windows_official_lifecycle024 as lifecycle, windows_official_runtime024 as native,
    },
    modules::{
        process::{LifecycleAction, ProcessCommand},
        runtime::{self, official_lifecycle024::ActionWindow},
        status::ClientRecoveryState,
    },
};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    time::{Duration, Instant},
};
fn invalid(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}
fn digest(p: &Path) -> io::Result<String> {
    let mut b = Vec::new();
    File::open(p)?
        .take(128 * 1024 * 1024 + 1)
        .read_to_end(&mut b)?;
    if b.len() > 128 * 1024 * 1024 {
        return Err(invalid("bounded broker/manager image"));
    }
    Ok(hash::sha256_hex(&b))
}
fn write_new(path: &Path, v: &Value) -> io::Result<()> {
    let mut f = platform::create_new_private_file(path, 0o600)?;
    f.write_all(&serde_json::to_vec_pretty(v).map_err(invalid)?)?;
    f.sync_all()
}
fn event(log: &mut File, stage: &str, v: Value) -> io::Result<()> {
    serde_json::to_writer(
        &mut *log,
        &json!({"stage":stage,"facts":v,"publicStartOpen":false}),
    )
    .map_err(invalid)?;
    log.write_all(b"\n")?;
    log.sync_all()
}
pub(super) fn run(c: EmptyBootstrap) -> io::Result<()> {
    let base = Path::new(BASE);
    let profile = Path::new(r"C:\Users\marenben");
    let current = native::RetainedPeer::current()?;
    let token = &current.identity().token;
    if c.schema != 1
        || c.scope != SCOPE
        || c.expected_sid != token.sid
        || c.expected_session != 7
        || token.session_id != 7
        || token.elevated
        || token.elevation_type != 1
        || c.profile != profile
        || c.installation != base.join("I")
        || c.data != base.join("D")
        || c.home != base.join("H")
        || platform::home_dir().as_deref() != Some(profile)
    {
        return Err(invalid("exact original ordinary qualification bootstrap"));
    }
    current.no_outer_job()?;
    current.verify_image(&c.installation.join("ocxd.exe"))?;
    let _original_images = ["ocxd.exe", "opencodex-desktop.exe", "ocx.exe"]
        .iter()
        .map(|name| {
            OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(0x200000)
                .open(c.installation.join(name))
        })
        .collect::<io::Result<Vec<_>>>()?;
    if digest(&c.installation.join("ocxd.exe"))? != c.broker_sha256
        || digest(&c.installation.join("opencodex-desktop.exe"))? != c.manager_sha256
    {
        return Err(invalid("exact task product images changed"));
    }
    let mut log =
        platform::create_new_private_file(&c.data.join("qualification-events.jsonl"), 0o600)?;
    event(
        &mut log,
        "original-broker-admitted",
        json!({"pid":current.identity().pid,"filetime":current.identity().creation_filetime.to_string(),"sid":token.sid,"session":token.session_id,"elevated":false,"noOuterJob":true,"officialOwnerCreated":false,"profile":profile,"I":c.installation,"D":c.data,"H":c.home}),
    )?;
    // Establish reachability before any fallible native Owner creation. The
    // locator is DATA only; every request authenticates the native pipe peer.
    let control = super::failure_control::Control::create(&c, current)?;
    let recovery_control = super::recovery_control::install_before_capture(&c)?;
    let mut runtime_for_trace: Option<SharedRuntimeExecutable> = None;
    let result = (|| -> io::Result<Value> {
        recovery_control.check_alive()?;
        let q = ExistingQualification::prepare()?;
        event(
            &mut log,
            "fixed-source-and-synthetic-coordination-prepared",
            json!({"nativeLeaseGranted":false,"officialInitializationProven":false}),
        )?;
        let runtime: SharedRuntimeExecutable =
            runtime::RuntimeHandle::initialize_versioned(&c.data, &c.installation);
        runtime_for_trace = Some(runtime.clone());
        recovery_control.bind_runtime(runtime.clone())?;
        let mut command = ProcessCommand::new(
            LifecycleAction::Start,
            c.installation.join("runtime/bin/ocx-runtime.cmd"),
            c.profile.clone(),
            c.home.clone(),
        );
        command.environment.home = Some(c.profile.as_os_str().into());
        let mut rows = Vec::new();
        for (name, action) in [
            ("start1", LifecycleAction::Start),
            ("stop1", LifecycleAction::Stop),
            ("start2", LifecycleAction::Start),
            ("restart2", LifecycleAction::Restart),
            ("stop3", LifecycleAction::Stop),
        ] {
            recovery_control.check_alive()?;
            command.action = action;
            let started = Instant::now();
            let window = ActionWindow::begin(action, started, started + Duration::from_secs(30))
                .map_err(invalid)?;
            event(
                &mut log,
                &format!("{name}-intent"),
                json!({"action":format!("{action:?}"),"outerBudgetMs":window.deadline().duration_since(started).as_millis(),"stopBudgetMs":window.stop_deadline().duration_since(started).as_millis()}),
            )?;
            let outcome = lifecycle::qualify_existing(&q, &runtime, &command, &c.data, window);
            let elapsed = started.elapsed();
            let trace = lifecycle::qualification_trace(&runtime, &c.home);
            let row = json!({"action":name,"elapsedMs":elapsed.as_millis(),"outerBudgetMs":window.deadline().duration_since(started).as_millis(),"stopBudgetMs":window.stop_deadline().duration_since(started).as_millis(),"result":outcome.as_ref().ok().map(|v|format!("{v:?}")),"error":outcome.as_ref().err().map(|e|e.to_string()),"trace":trace});
            write_new(&c.data.join(format!("qualification-{name}.json")), &row)?;
            event(&mut log, &format!("{name}-result"), row.clone())?;
            rows.push(row);
            outcome.map_err(invalid)?;
            // One actual original5s Status sample joins native fetch and recovery.
            let sampled = Instant::now();
            let end = sampled + Duration::from_secs(5);
            let (payload, receipt) =
                tauri::async_runtime::block_on(lifecycle::status_sample(&runtime, &c.home, end))
                    .map_err(|e| invalid(format!("same-sample Status {e:?}")))?;
            let recovery = lifecycle::recovery_for_sample(&runtime, &c.home, end, receipt);
            if sampled.elapsed() > Duration::from_secs(5)
                || matches!(action, LifecycleAction::Stop)
                    && recovery != ClientRecoveryState::NotRequired
            {
                return Err(invalid(
                    "actual same-sample Status/recovery not confirmed in original5s",
                ));
            }
            let status = json!({"after":name,"elapsedMs":sampled.elapsed().as_millis(),"payload":payload,"recovery":format!("{recovery:?}"),"sameSample":true,"budgetMs":5000});
            write_new(
                &c.data.join(format!("qualification-status-{name}.json")),
                &status,
            )?;
            event(&mut log, "same-sample-status", status)?;
        }
        Ok(
            json!({"actions":rows,"nativeProductionSequencePassed":true,"actualProductionExecuted":true,"actualGuiOfficialOwnerProven":false,"managerHandoffProven":false,"officialInitializationProven":false,"originalAbsentRetirementProven":false,"allAccepted":false,"startOpen":false}),
        )
    })();
    let trace = runtime_for_trace
        .as_ref()
        .map(|r| lifecycle::qualification_trace(r, &c.home))
        .unwrap_or_else(
            || json!({"nativeOwner":false,"creatorResumed":false,"preparationOnly":true}),
        );
    let failed = result.is_err();
    let final_result = match result {
        Ok(v) => v,
        Err(e) => {
            json!({"nativeProductionSequencePassed":false,"raw":e.raw_os_error(),"firstError":e.to_string(),"actualProductionExecuted":trace["creatorResumed"]==true,"nativeOwnerRetained":trace["nativeOwner"]==true,"trace":trace,"allAccepted":false,"startOpen":false})
        }
    };
    // A failed evidence write must not strand an original Owner by dropping
    // the host. The control loop records these errors and remains reachable.
    let evidence_error = event(&mut log, "qualification-result", final_result.clone())
        .and_then(|_| write_new(&c.data.join("qualification-result.json"), &final_result))
        .err();
    // Success retains original closed history; failure retains ALL original
    // Owner/Job/custody and exact prefixes. No implicit exit/Drop, termination,
    // retry/rollforward or public admission follows this diagnostic result.
    control.serve(&mut log, runtime_for_trace.as_ref(), failed, evidence_error)
}
