//! An independent creation-handle observer launching the actual product EXE.
//! Empty-broker continuity is incremental evidence, not official Owner proof.
use super::{
    desktop_tests::{facts, wait_exit, wait_file, write_json},
    empty_host,
};
use crate::infrastructure::{hash, platform, windows_official_runtime024 as native};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    io::Write,
    os::windows::io::FromRawHandle,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
fn end() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn gui(base: &Path, mode: &str) -> Child {
    let data = base.join("D");
    let stdout =
        platform::create_new_private_file(&data.join(format!("gui-{mode}.stdout.log")), 0o600)
            .unwrap();
    let stderr =
        platform::create_new_private_file(&data.join(format!("gui-{mode}.stderr.log")), 0o600)
            .unwrap();
    // Each actual GUI gets fresh normal application stores and WebView state.
    // No runtime is installed in these sandboxes. Do not rewrite OS Profile.
    let sandbox = base.join(format!("GUI-{mode}"));
    platform::create_private_dir_all(&sandbox, 0o700).unwrap();
    let mut command = Command::new(base.join("I/opencodex-desktop.exe"));
    command
        .current_dir(base)
        .env("OCXD024_ACTUAL_GUI_BASE", base)
        .env("OCXD024_ACTUAL_GUI_MODE", mode)
        .env("OPENCODEX_SANDBOX", "1")
        .env("OPENCODEX_SANDBOX_ROOT", &sandbox)
        .env_remove("OPENCODEX_SANDBOX_BOUNDARY")
        .env_remove("OCXD024_H_BASE")
        .env_remove("OCXD024_H_MODE")
        // Existing Windows smoke support builds the REAL main window and gives
        // it an explicit isolated WebView directory. CDP is diagnostic only.
        .env(
            "OPENCODEX_WINDOWS_SMOKE_CDP_PORT",
            match mode {
                "normal" => "19371",
                "hold" => "19372",
                "after-failure" => "19373",
                "finish" => "19374",
                _ => panic!("unknown GUI mode"),
            },
        )
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
    command.spawn().unwrap()
}
fn verify_window(child: &Child, receipt: &serde_json::Value) {
    assert_eq!(receipt["pid"].as_u64(), Some(child.id() as u64));
    assert_eq!(receipt["actualGUI"], true);
    assert_eq!(receipt["fixtureManager"], false);
    assert_eq!(receipt["mainPageFinished"], true);
    assert_eq!(receipt["fullNormalSetup"], true);
    assert_eq!(receipt["officialOwnerCreated"], false);
    assert_eq!(receipt["reply"]["lifecycleAccepted"], false);
    assert_eq!(receipt["reply"]["oldConnectionReplayRefused"], true);
    let hwnd = receipt["nativeWindow"]
        .as_str()
        .unwrap()
        .parse::<usize>()
        .unwrap() as *mut std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetWindowThreadProcessId(hwnd: *mut std::ffi::c_void, pid: *mut u32) -> u32;
        fn IsWindowVisible(hwnd: *mut std::ffi::c_void) -> i32;
    }
    let mut pid = 0;
    assert_ne!(unsafe { GetWindowThreadProcessId(hwnd, &mut pid) }, 0);
    assert_eq!(pid, child.id());
    assert_ne!(unsafe { IsWindowVisible(hwnd) }, 0);
}
fn request_exit(data: &Path, mode: &str, receipt: &serde_json::Value) {
    let raw = format!(
        "exit-original-gui:{}:{}",
        receipt["pid"].as_u64().unwrap(),
        receipt["creationFiletime"].as_str().unwrap()
    );
    drop(
        native::PrivateRecord::create(
            &data.join(format!("gui-{mode}.exit-request")),
            raw.as_bytes(),
            256,
        )
        .unwrap(),
    );
}
#[test]
#[ignore = "exact marenben ordinary Explorer desktop; actual product main GUI, no official Owner or lifecycle"]
fn isolated_actual_gui_observer() {
    let profile = platform::home_dir().unwrap();
    let token = native::current_identity().unwrap();
    let evidence = PathBuf::from(std::env::var_os("OCXD024_H_EVIDENCE").unwrap());
    let input = PathBuf::from(std::env::var_os("OCXD024_H_INPUT").unwrap());
    let bootstrap = native::RetainedPeer::current().unwrap();
    write_json(
        &evidence.join("bootstrap-admission.json"),
        &serde_json::json!({
        "identity":facts(bootstrap.identity()),"actualProfile":profile,"noJob":bootstrap.no_outer_job().is_ok(),
        "actualGUIObserver":true,"officialOwnerCreated":false,"accepted":false}),
    );
    assert_eq!(token.sid, "S-1-5-21-587408064-856152790-2583328096-1003");
    assert_eq!(token.session_id, 7);
    assert!(!token.elevated);
    assert_ne!(token.elevation_type, 2);
    bootstrap.no_outer_job().unwrap();
    let base = profile.join(format!("ocx024-h-{}", uuid::Uuid::new_v4()));
    platform::create_private_dir_all(&base, 0o700).unwrap();
    for n in ["I", "D", "H"] {
        platform::create_private_dir_all(&base.join(n), 0o700).unwrap();
    }
    let installation = base.join("I");
    let data = base.join("D");
    let home = base.join("H");
    for name in ["ocxd.exe", "opencodex-desktop.exe"] {
        fs::copy(input.join(name), installation.join(name)).unwrap();
    }
    let broker_sha = hash::sha256_file(&installation.join("ocxd.exe")).unwrap();
    let manager_sha = hash::sha256_file(&installation.join("opencodex-desktop.exe")).unwrap();
    write_json(
        &evidence.join("task-location.json"),
        &serde_json::json!({
        "base":base,"data":data,"brokerSha":broker_sha,"managerSha":manager_sha,
        "fixtureManager":false,"actualGUIRequested":true,"actualGUI":false,"serviceStarted":false}),
    );
    let config = empty_host::EmptyBootstrap {
        schema: 1,
        scope: "isolated-empty-owner-continuity".into(),
        expected_sid: token.sid.clone(),
        expected_session: token.session_id,
        profile: profile.clone(),
        installation: installation.clone(),
        data: data.clone(),
        home: home.clone(),
        broker_sha256: broker_sha,
        manager_sha256: manager_sha,
    };
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreatePipe(
            read: *mut *mut std::ffi::c_void,
            write: *mut *mut std::ffi::c_void,
            sa: *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
            size: u32,
        ) -> i32;
    }
    let (mut read, mut write) = (std::ptr::null_mut(), std::ptr::null_mut());
    assert_ne!(
        unsafe { CreatePipe(&mut read, &mut write, std::ptr::null(), 4096) },
        0
    );
    let read = unsafe { File::from_raw_handle(read) };
    let mut write = unsafe { File::from_raw_handle(write) };
    let out = platform::create_new_private_file(&data.join("broker.stdout.log"), 0o600).unwrap();
    let err = platform::create_new_private_file(&data.join("broker.stderr.log"), 0o600).unwrap();
    let mut env = BTreeMap::<OsString, OsString>::new();
    for key in [
        "SystemRoot",
        "PATH",
        "TEMP",
        "TMP",
        "APPDATA",
        "LOCALAPPDATA",
    ] {
        if let Some(value) = std::env::var_os(key) {
            env.insert(key.into(), value);
        }
    }
    let executable = installation.join("ocxd.exe");
    let arguments = [OsString::from("--internal-runtime-owner024")];
    let mut host = native::SuspendedOwnerHost::create(
        native::LaunchSpec {
            executable: &executable,
            arguments: &arguments,
            working_directory: &base,
            user_profile: &profile,
            opencodex_home: &home,
            environment: &env,
            stdout: &out,
            stderr: &err,
        },
        &read,
    )
    .unwrap();
    let original = host.original_peer().identity().clone();
    write_json(&data.join("original-broker.json"), &facts(&original));
    write_json(
        &evidence.join("suspended-original.json"),
        &serde_json::json!({
        "identity":facts(&original),"originalCreationHandleRetained":true,
        "beforeResumeNoJob":host.original_peer().no_outer_job().is_ok(),
        "bootstrapStillNoJob":bootstrap.no_outer_job().is_ok(),"source":"actual ocxd; actual product GUI image"}),
    );
    let bytes = serde_json::to_vec(&config).unwrap();
    assert!(bytes.len() < 4092);
    write
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .unwrap();
    write.write_all(&bytes).unwrap();
    drop(write);
    drop(read);
    host.resume().unwrap();
    drop(out);
    drop(err);
    wait_file(&data.join("rendezvous.json"), end());
    let mut generations = vec![];
    for mode in ["normal", "hold", "after-failure", "finish"] {
        host.original_peer().no_outer_job().unwrap();
        assert_eq!(host.original_peer().identity(), &original);
        let mut child = gui(&base, mode);
        let result = data.join(format!("gui-{mode}.json"));
        wait_file(&result, end());
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&result).unwrap()).unwrap();
        verify_window(&child, &receipt);
        let generation = receipt["reply"]["managerGeneration"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        generations.push(generation);
        let forced = mode == "hold";
        if forced {
            // std Child retains the handle from this exact original GUI spawn;
            // never enumerate a name/PID and never terminate broker or family.
            child.kill().unwrap();
        } else {
            request_exit(&data, mode, &receipt);
        }
        let status = wait_exit(&mut child, end());
        if !forced {
            assert!(
                status.success(),
                "actual GUI {mode} rawExit={:?}",
                status.code()
            );
        }
        if mode != "finish" {
            host.original_peer().no_outer_job().unwrap();
        }
        write_json(
            &evidence.join(format!("after-{mode}.json")),
            &serde_json::json!({
            "broker":facts(host.original_peer().identity()),"originalCreationHandleRetained":true,
            "brokerStillAlive":!host.original_peer().exited().unwrap(),"managerPid":child.id(),
            "managerExit":status.code(),"forcedManagerOnly":forced,"managerGeneration":generation.to_string(),
            "nativeWindowVerifiedByObserver":true,"actualGUI":true,"fixtureManager":false,
            "officialOwnerCreated":false,"allAccepted":false}),
        );
    }
    assert!(generations.windows(2).all(|w| w[1] > w[0]));
    let deadline = end();
    while !host.original_peer().exited().unwrap() {
        assert!(
            Instant::now() < deadline,
            "empty broker did not exit explicitly"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    write_json(
        &evidence.join("result.json"),
        &serde_json::json!({
        "sid":token.sid,"session":7,"originalBroker":facts(&original),"originalCreationHandleRetained":true,
        "sameOriginalBrokerAcrossActualGUIs":true,"managerGenerations":generations,
        "brokerExitedExplicitly":true,"brokerRawExit":null,"actualGUI":true,"fixtureManager":false,
        "officialOwnerCreated":false,"lifecycleAccepted":false,"allAccepted":false,"base":base,
        "taskBaseRetained":true,"originalProfileAndPATHUnchangedByTest":true}),
    );
    // The launcher exports every bounded task log under this same SID even on
    // failure. Native source remains private; GUI caches are not public logs.
}
