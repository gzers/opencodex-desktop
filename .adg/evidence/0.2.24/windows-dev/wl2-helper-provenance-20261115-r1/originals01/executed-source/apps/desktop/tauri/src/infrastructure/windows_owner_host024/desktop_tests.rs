use super::*;
use crate::infrastructure::{
    hash, platform,
    windows_official_runtime024::{
        self as native, FileIdentity, LaunchSpec, ProcessIdentity, SuspendedOwnerHost,
        TokenIdentity,
    },
};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::Write,
    os::windows::io::FromRawHandle,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

fn facts(p: &ProcessIdentity) -> serde_json::Value {
    serde_json::json!({"pid":p.pid,"creationFiletime":p.creation_filetime.to_string(),"sid":p.token.sid,"session":p.token.session_id,"elevated":p.token.elevated,"elevationType":p.token.elevation_type,"image":p.executable,"volume":p.executable_id.volume,"fileId":p.executable_id.index.to_string()})
}
fn parse_facts(v: &serde_json::Value) -> ProcessIdentity {
    ProcessIdentity {
        pid: v["pid"].as_u64().unwrap().try_into().unwrap(),
        creation_filetime: v["creationFiletime"].as_str().unwrap().parse().unwrap(),
        token: TokenIdentity {
            sid: v["sid"].as_str().unwrap().into(),
            session_id: v["session"].as_u64().unwrap().try_into().unwrap(),
            elevated: v["elevated"].as_bool().unwrap(),
            elevation_type: v["elevationType"].as_u64().unwrap().try_into().unwrap(),
        },
        executable: PathBuf::from(v["image"].as_str().unwrap()),
        executable_id: FileIdentity {
            volume: v["volume"].as_u64().unwrap().try_into().unwrap(),
            index: v["fileId"].as_str().unwrap().parse().unwrap(),
        },
    }
}
fn wait_file(p: &Path, deadline: Instant) {
    while !p.exists() {
        assert!(
            Instant::now() < deadline,
            "expected native file missing: {}",
            p.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn wait_exit(child: &mut Child, deadline: Instant) -> std::process::ExitStatus {
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        assert!(Instant::now() < deadline, "exact test manager did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn write_json(p: &Path, v: &serde_json::Value) {
    let mut f = platform::create_new_private_file(p, 0o600).unwrap();
    serde_json::to_writer_pretty(&mut f, v).unwrap();
    f.sync_all().unwrap();
}
fn manager(base: &Path, mode: &str) -> Child {
    let d = base.join("D");
    let out =
        platform::create_new_private_file(&d.join(format!("manager-{mode}.stdout.log")), 0o600)
            .unwrap();
    let err =
        platform::create_new_private_file(&d.join(format!("manager-{mode}.stderr.log")), 0o600)
            .unwrap();
    let mut c = Command::new(base.join("I/opencodex-desktop.exe"));
    c.args([
        "--exact",
        "infrastructure::windows_owner_host024::desktop_tests::isolated_h024_manager",
        "--ignored",
        "--test-threads=1",
        "--nocapture",
    ])
    .env("OCXD024_H_BASE", base)
    .env("OCXD024_H_MODE", mode)
    .env_remove("OPENCODEX_SANDBOX")
    .env_remove("OPENCODEX_SANDBOX_ROOT")
    .stdin(Stdio::null())
    .stdout(Stdio::from(out))
    .stderr(Stdio::from(err));
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x08000000);
    c.spawn().unwrap()
}

#[test]
#[ignore = "one marenben interactive Explorer-started ordinary desktop; no official runtime or daily configuration"]
fn isolated_h024_observer() {
    let profile = platform::home_dir().unwrap();
    let token = native::current_identity().unwrap();
    let evidence = PathBuf::from(std::env::var_os("OCXD024_H_EVIDENCE").unwrap());
    let input = PathBuf::from(std::env::var_os("OCXD024_H_INPUT").unwrap());
    let bootstrap = native::RetainedPeer::current().unwrap();
    let admission = serde_json::json!({"identity":facts(bootstrap.identity()),"expectedSID":"S-1-5-21-587408064-856152790-2583328096-1003","expectedSession":7,"actualProfile":profile,"noJob":bootstrap.no_outer_job().is_ok(),"officialOwnerCreated":false,"accepted":false});
    write_json(&evidence.join("bootstrap-admission.json"), &admission);
    assert_eq!(token.sid, "S-1-5-21-587408064-856152790-2583328096-1003");
    assert_eq!(token.session_id, 7);
    assert!(!token.elevated);
    assert_ne!(token.elevation_type, 2);
    bootstrap
        .no_outer_job()
        .expect("real desktop bootstrap enclosing Job unknown; no broker started");
    let base = profile.join(format!("ocx024-h-{}", uuid::Uuid::new_v4()));
    platform::create_private_dir_all(&base, 0o700).unwrap();
    for n in ["I", "D", "H"] {
        platform::create_private_dir_all(&base.join(n), 0o700).unwrap();
    }
    let installation = base.join("I");
    let data = base.join("D");
    let home = base.join("H");
    fs::copy(input.join("ocxd.exe"), installation.join("ocxd.exe")).unwrap();
    fs::copy(
        std::env::current_exe().unwrap(),
        installation.join("opencodex-desktop.exe"),
    )
    .unwrap();
    let broker_sha = hash::sha256_file(&installation.join("ocxd.exe")).unwrap();
    let manager_sha = hash::sha256_file(&installation.join("opencodex-desktop.exe")).unwrap();
    write_json(
        &evidence.join("task-location.json"),
        &serde_json::json!({"base":base,"data":data,"brokerSha":broker_sha,"managerSha":manager_sha,"fixtureManager":true,"actualGUI":false,"serviceStarted":false}),
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
        if let Some(v) = std::env::var_os(key) {
            env.insert(key.into(), v);
        }
    }
    let executable = installation.join("ocxd.exe");
    let arguments = [OsString::from("--internal-runtime-owner024")];
    let mut host = SuspendedOwnerHost::create(
        LaunchSpec {
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
        &serde_json::json!({"identity":facts(&original),"originalCreationHandleRetained":true,"beforeResumeNoJob":host.original_peer().no_outer_job().is_ok(),"bootstrapStillNoJob":bootstrap.no_outer_job().is_ok(),"source":"actual ocxd; fixture manager only"}),
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
    wait_file(
        &data.join("rendezvous.json"),
        Instant::now() + Duration::from_secs(5),
    );
    let mut generations = vec![];
    for mode in ["normal", "hold", "after-failure"] {
        host.original_peer().no_outer_job().unwrap();
        assert_eq!(host.original_peer().identity(), &original);
        let mut child = manager(&base, mode);
        let result = data.join(format!("manager-{mode}.json"));
        wait_file(&result, Instant::now() + Duration::from_secs(5));
        let reply: serde_json::Value = serde_json::from_slice(&fs::read(&result).unwrap()).unwrap();
        assert_eq!(reply["lifecycleAccepted"], false);
        assert_eq!(reply["oldConnectionReplayRefused"], true);
        let generation: u64 = reply["managerGeneration"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        generations.push(generation);
        let forced = mode == "hold";
        if forced {
            child.kill().unwrap();
        }
        let status = wait_exit(&mut child, Instant::now() + Duration::from_secs(5));
        if !forced {
            assert!(status.success());
        }
        host.original_peer().no_outer_job().unwrap();
        assert_eq!(host.original_peer().identity(), &original);
        write_json(
            &evidence.join(format!("after-{mode}.json")),
            &serde_json::json!({"broker":facts(host.original_peer().identity()),"originalCreationHandleRetained":true,"brokerStillAlive":true,"noJob":true,"managerPid":child.id(),"managerExit":status.code(),"forcedManagerOnly":forced,"managerGeneration":generation.to_string(),"fixtureManager":true,"actualGUI":false,"officialOwnerCreated":false,"allAccepted":false}),
        );
    }
    assert!(generations.windows(2).all(|w| w[1] > w[0]));
    let mut final_manager = manager(&base, "finish");
    assert!(wait_exit(&mut final_manager, Instant::now() + Duration::from_secs(5)).success());
    let deadline = Instant::now() + Duration::from_secs(5);
    while !host.original_peer().exited().unwrap() {
        assert!(
            Instant::now() < deadline,
            "empty host did not explicitly exit"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut files = vec![];
    for entry in fs::read_dir(&data).unwrap() {
        let p = entry.unwrap().path();
        if p.is_file() {
            let b = fs::read(&p).unwrap();
            assert!(b.len() <= 1_048_576);
            files.push(serde_json::json!({"name":p.file_name().unwrap().to_string_lossy(),"bytes":b.len(),"sha256":hash::sha256_hex(&b)}));
            fs::copy(&p, evidence.join(p.file_name().unwrap())).unwrap();
        }
    }
    write_json(
        &evidence.join("archive-manifest.json"),
        &serde_json::json!({"files":files,"allTaskDataArchived":true,"credentialsOrClientData":false}),
    );
    write_json(
        &evidence.join("result.json"),
        &serde_json::json!({"sid":token.sid,"session":token.session_id,"bootstrapNoJob":true,"suspendedBrokerNoJob":true,"sameOriginalBrokerAcrossFixtureManagers":true,"managerGenerations":generations,"forcedManagerOnly":true,"brokerExitedExplicitly":true,"officialOwnerCreated":false,"OReplayed":false,"fixtureManager":true,"actualGUI":false,"allAccepted":false,"base":base,"originalProfileAndPATHUnchangedByTest":true,"taskBaseRetained":true}),
    );
}

#[test]
#[ignore = "internal synthetic manager child of the exact ordinary H observer; not actual GUI"]
fn isolated_h024_manager() {
    let base = PathBuf::from(std::env::var_os("OCXD024_H_BASE").unwrap());
    let mode = std::env::var("OCXD024_H_MODE").unwrap();
    let profile = platform::home_dir().unwrap();
    let token = native::current_identity().unwrap();
    assert_eq!(token.sid, "S-1-5-21-587408064-856152790-2583328096-1003");
    assert!(!token.elevated);
    assert_eq!(token.session_id, 7);
    assert_eq!(base.parent(), Some(profile.as_path()));
    assert!(base
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("ocx024-h-"));
    assert!(["normal", "hold", "after-failure", "finish"].contains(&mode.as_str()));
    let data = base.join("D");
    let original = parse_facts(
        &serde_json::from_slice::<serde_json::Value>(
            &fs::read(data.join("original-broker.json")).unwrap(),
        )
        .unwrap(),
    );
    let reply = empty_host::manager_probe(
        &data,
        &original,
        if mode == "finish" {
            "explicit-recovery"
        } else {
            "status"
        },
    )
    .unwrap();
    write_json(&data.join(format!("manager-{mode}.json")), &reply);
    if mode == "hold" {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
