//! Exact test-image entries. The host remains resident once initial Start is attempted.
use super::{
    empty_host::{read_bootstrap, EmptyBootstrap},
    qpc, recovery_control,
};
use crate::{
    infrastructure::{
        runtime_executable::SharedRuntimeExecutable,
        windows_client_originals024::{
            partial_capture_fixture::{report, Fixture, Stage},
            CaptureFailureArm,
        },
        windows_official_lifecycle024 as lifecycle,
    },
    modules::{
        process::{LifecycleAction, ProcessCommand},
        runtime::{self, official_lifecycle024::ActionWindow},
    },
};
use serde_json::{json, Value};
use std::{
    io::{self, Read, Write},
    os::windows::io::AsRawHandle,
    time::{Duration, Instant},
};

fn bootstrap(end: Instant) -> io::Result<EmptyBootstrap> {
    if unsafe { windows_sys::Win32::Storage::FileSystem::GetFileType(io::stdin().as_raw_handle()) }
        != 3
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixture requires one inherited bootstrap pipe",
        ));
    }
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = send.send(read_bootstrap(io::stdin().lock()));
    });
    receive
        .recv_timeout(end.saturating_duration_since(Instant::now()))
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
}

fn stage(host: bool) {
    let outcome = (|| -> io::Result<()> {
        let end = Instant::now() + Duration::from_secs(5);
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::GetFileType(io::stdin().as_raw_handle())
        } != 3
        {
            return Err(fail("fixture staging requires an inherited pipe"));
        }
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let (release, wait) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut input = io::stdin().lock();
            let decoded = (|| -> io::Result<EmptyBootstrap> {
                let mut len = [0; 4];
                input.read_exact(&mut len)?;
                let len = u32::from_le_bytes(len) as usize;
                if len == 0 || len > 65536 {
                    return Err(fail("fixture staging frame exceeds its bound"));
                }
                let mut frame = Vec::with_capacity(len + 4);
                frame.extend_from_slice(&(len as u32).to_le_bytes());
                frame.resize(len + 4, 0);
                input.read_exact(&mut frame[4..])?;
                read_bootstrap(frame.as_slice())
            })();
            let accepted = decoded.is_ok();
            if send.send(decoded).is_ok() && accepted {
                let mut extra = [0];
                let result = input.read(&mut extra).and_then(|length| {
                    if length == 0 {
                        Ok(())
                    } else {
                        Err(fail("unexpected fixture staging input"))
                    }
                });
                let _ = release.send(result);
            }
        });
        let c = receive
            .recv_timeout(end.saturating_duration_since(Instant::now()))
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
        let pinned = Stage::admit(&c, host)?;
        println!("r7-stage-ready");
        io::stdout().flush()?;
        let result = wait.recv().map_err(io::Error::other)?;
        drop(pinned);
        result
    })();
    if let Err(error) = &outcome {
        eprintln!(
            "r7-stage-failed={}",
            json!({"raw":error.raw_os_error(),"message":error.to_string()})
        );
    }
    outcome.unwrap();
}

#[test]
#[ignore = "native stage for the fixed ordinary interactive fixture"]
fn isolated_r7_partial_capture_stage_host() {
    stage(true);
}

#[test]
#[ignore = "native stage for the fixed ordinary interactive fixture"]
fn isolated_r7_partial_capture_stage_manager() {
    stage(false);
}

fn fail(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[test]
#[ignore = "dedicated ordinary interactive account and immutable fixed source only; retains original Owner on failure"]
fn isolated_r7_partial_capture_host() {
    let c = bootstrap(Instant::now() + Duration::from_secs(5)).unwrap();
    let mut fixture = Fixture::admit(&c).unwrap();
    if let Err(error) = fixture.prepare(&c) {
        let facts = json!({"stage":"prepare","raw":error.raw_os_error(),"firstError":error.to_string(),"publicStartOpen":false});
        eprintln!("r7-partial-capture={facts}");
        let _ = report(&c.data.join("fixture-result.json"), &facts);
        panic!("fixture preparation failed");
    }
    let control = recovery_control::install_before_capture(&c).unwrap();
    let runtime: SharedRuntimeExecutable =
        runtime::RuntimeHandle::initialize_versioned(&c.data, &c.installation);
    control.bind_runtime(runtime.clone()).unwrap();
    let mut command = ProcessCommand::new(
        LifecycleAction::Start,
        c.installation.join("runtime/bin/ocx-runtime.cmd"),
        c.profile.clone(),
        c.home.clone(),
    );
    command.environment.home = Some(c.profile.as_os_str().into());
    let (arm, injection) = CaptureFailureArm::new();
    let started = Instant::now();
    let end = started + Duration::from_secs(30);
    let window = ActionWindow::begin(LifecycleAction::Start, started, end).unwrap();
    let result =
        lifecycle::qualify_partial_capture(&fixture, &runtime, &command, &c.data, window, arm);
    let elapsed = started.elapsed();
    // Once admitted, evidence errors cannot exit the original process.
    let recorded = injection.lock().ok().and_then(|evidence| evidence.clone());
    let status_started = Instant::now();
    let status_end = status_started + Duration::from_secs(5);
    let status = native_status(&runtime, &c.home, status_end);
    let facts = json!({"stage":"initial-start","elapsedMs":elapsed.as_millis(),"budgetMs":30000,
        "success":result.is_ok(),"firstError":result.as_ref().err().map(ToString::to_string),
        "injection":recorded,"sameSampleStatus":status.as_ref().ok(),"statusError":status.as_ref().err().map(ToString::to_string),
        "statusElapsedMs":status_started.elapsed().as_millis(),"statusBudgetMs":5000,
        "publicStartOpen":false,"normalStop":false,"closed":false});
    println!("r7-partial-capture={facts}");
    if let Err(error) = report(&c.data.join("fixture-result.json"), &facts) {
        eprintln!(
            "fixture-result evidence failed raw={:?} {error}",
            error.raw_os_error()
        );
    }
    loop {
        std::hint::black_box((&control, &fixture, &runtime));
        std::thread::park();
    }
}

pub(super) fn native_status(
    runtime: &SharedRuntimeExecutable,
    home: &std::path::Path,
    end: Instant,
) -> io::Result<Value> {
    let (payload, sample) =
        tauri::async_runtime::block_on(lifecycle::status_sample(runtime, home, end))
            .map_err(|error| fail(format!("same-sample Status {error:?}")))?;
    let recovery = lifecycle::recovery_for_sample(runtime, home, end, sample);
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(json!({"payload":payload,"recovery":format!("{recovery:?}"),"sameSample":true}))
}

fn manager(operation: &str) {
    let started = Instant::now();
    let (counter, frequency) = qpc().unwrap();
    let end = started + Duration::from_secs(recovery_control::command_budget(operation).unwrap());
    let outcome = (|| -> io::Result<(Value, bool)> {
        let c = bootstrap(end)?;
        let _image = Fixture::admit_client(&c, end)?;
        recovery_control::request(&c, operation, started, end, counter, frequency)
    })();
    let data = match &outcome {
        Ok((data, refused)) => json!({"operation":operation,"response":data,"refused":refused}),
        Err(error) => {
            json!({"operation":operation,"raw":error.raw_os_error(),"error":error.to_string()})
        }
    };
    println!(
        "r7-partial-manager={}",
        json!({"result":data,"elapsedMs":started.elapsed().as_millis(),"publicStartOpen":false})
    );
    let (data, refused) = outcome.unwrap();
    if operation == "begin-canonical-abort" {
        assert!(
            refused,
            "Pending is an expected canonical-abort refusal: {data}"
        );
    } else {
        assert!(!refused, "worker diagnostic refused: {data}");
    }
}

#[test]
#[ignore = "same pinned test image and ordinary interactive fixture only"]
fn isolated_r7_partial_capture_diagnostic() {
    manager("read-worker-diagnostic");
}

#[test]
#[ignore = "explicit operator-run request only; exact original ordinary fixture"]
fn isolated_r7_partial_capture_explicit() {
    manager("begin-canonical-abort");
}
