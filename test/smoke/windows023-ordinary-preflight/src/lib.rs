//! Executor preflight only. This fixture does not certify the product suite.
#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::Write,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    const CHILD_MARKER: &str = "OCXD_ORDINARY_PREFLIGHT_CHILD";
    const CHILD_TEST: &str = "tests::child_helper";

    fn same_directory(a: &Path, b: &Path) {
        assert_eq!(
            a.canonicalize().expect("first directory must exist"),
            b.canonicalize().expect("second directory must exist"),
            "preflight directory binding differs"
        );
    }

    fn check_context() -> String {
        #[cfg(windows)]
        {
            let (profile, local, sid) = windows::context();
            same_directory(
                &PathBuf::from(env::var_os("USERPROFILE").expect("USERPROFILE")),
                &profile,
            );
            same_directory(&PathBuf::from(env::var_os("HOME").expect("HOME")), &profile);
            same_directory(
                &PathBuf::from(env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA")),
                &local,
            );
            same_directory(&local, &profile.join("AppData/Local"));
            let temp = local.join("Temp");
            same_directory(&env::temp_dir(), &temp);
            same_directory(&PathBuf::from(env::var_os("TEMP").expect("TEMP")), &temp);
            same_directory(&PathBuf::from(env::var_os("TMP").expect("TMP")), &temp);
            for (name, _) in env::vars_os() {
                let name = name.to_string_lossy().to_ascii_uppercase();
                let forbidden = [
                    "GITHUB_TOKEN",
                    "GH_TOKEN",
                    "GH_ENTERPRISE_TOKEN",
                    "GITHUB_ENTERPRISE_TOKEN",
                    "ACTIONS_RUNTIME_TOKEN",
                    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
                    "ACTIONS_ID_TOKEN_REQUEST_URL",
                    "ACTIONS_RESULTS_URL",
                    "ACTIONS_CACHE_URL",
                    "ACTIONS_RUNTIME_URL",
                    "SYSTEM_ACCESSTOKEN",
                    "NPM_TOKEN",
                    "NODE_AUTH_TOKEN",
                    "CARGO_REGISTRY_TOKEN",
                    "RUST_TEST_THREADS",
                    "RUST_TEST_TIME_UNIT",
                    "RUST_TEST_TIME_INTEGRATION",
                    "RUST_TEST_TIME_DOCTEST",
                    "RUST_TEST_NOCAPTURE",
                    "RUST_TEST_SHUFFLE",
                    "RUST_TEST_SHUFFLE_SEED",
                    "RUST_TEST_FORCE_RUN_IN_PROCESS",
                    "HTTP_PROXY",
                    "HTTPS_PROXY",
                    "ALL_PROXY",
                ]
                .contains(&name.as_str())
                    || name.starts_with("GITHUB_SECRET")
                    || name.starts_with("GH_SECRET")
                    || name.starts_with("RUST_TEST_");
                assert!(
                    !forbidden,
                    "unexpected credential or test-control environment variable: {name}"
                );
            }
            sid
        }
        #[cfg(not(windows))]
        {
            let home = PathBuf::from(env::var_os("HOME").expect("controlled local HOME"));
            let temp = PathBuf::from(env::var_os("TMPDIR").expect("controlled local TMPDIR"));
            assert!(home.is_absolute() && temp.is_absolute());
            same_directory(&env::temp_dir(), &temp);
            assert!(temp
                .canonicalize()
                .unwrap()
                .starts_with(home.canonicalize().unwrap()));
            "local-only-no-Windows-SID-evidence".into()
        }
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = env::temp_dir().canonicalize().expect("own TEMP");
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = root.join(format!(
                "ocxd023-ordinary-preflight-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("unique own TEMP fixture");
            assert_eq!(path.parent(), Some(root.as_path()));
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only this create_dir-owned fixture; never scan or delete other TEMP entries.
            let _ = fs::remove_file(self.0.join("parent.txt"));
            let _ = fs::remove_file(self.0.join("child.txt"));
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn explicit_readonly_official_ownership_diagnostic() {
        let Some(runtime) = option_env!("OCXD_OWNERSHIP_PROBE_RUNTIME") else {
            return;
        };
        let identity = check_context();
        let runtime = PathBuf::from(runtime);
        assert!(runtime.is_absolute());
        let bun = runtime.join("runtime/bun/bun.exe");
        let package = runtime.join("runtime/opencodex/node_modules/@bitkyc08/opencodex");
        assert!(bun.is_file() && package.join("package.json").is_file());
        let profile = PathBuf::from(env::var_os("USERPROFILE").unwrap());
        let h = profile.join(".codex-desktop/opencodex/home");
        fs::create_dir_all(&h).unwrap();
        let started = std::time::Instant::now();
        let mut child = Command::new(bun)
            .args(["--no-env-file", "-e", include_str!("../ownership-probe.ts")])
            .env("HOME", &profile)
            .env("USERPROFILE", &profile)
            .env("OPENCODEX_HOME", &h)
            .env("CODEX_HOME", profile.join(".codex"))
            .env("OCX_NO_DESKTOP_HANDOFF", "1")
            .env("OCXD_OWNERSHIP_DIAG_PACKAGE", &package)
            .current_dir(&h)
            .spawn()
            .expect("actual adopted Bun readonly ownership probe");
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > std::time::Duration::from_secs(60) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("readonly ownership probe exceeded its diagnostic budget");
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        };
        assert!(
            status.success(),
            "readonly diagnostic execution failed; no ownership verdict is accepted"
        );
        assert!(
            !profile.join(".codex").exists(),
            "readonly diagnostic created client home"
        );
        assert!(
            !h.join("config.json").exists(),
            "readonly diagnostic created configuration"
        );
        println!("ownership-diagnostic-only: SID={identity} elapsed_ms={} fullSoftwarePassed=false accepted=false", started.elapsed().as_millis());
    }

    #[test]
    fn explicit_official_cli_startup_diagnostic() {
        let Some(program) = option_env!("OCXD_STARTUP_PROBE_RUNTIME") else {
            return;
        };
        let identity = check_context();
        let program = PathBuf::from(program);
        let bun = program.join("runtime/bun/bun.exe");
        assert!(program.is_absolute() && bun.is_file());
        let started = std::time::Instant::now();
        let mut child = Command::new(bun)
            .args(["--no-env-file", "-e", include_str!("../startup-probe.ts")])
            .env("OCXD_STARTUP_DIAG_PROGRAM", &program)
            .env(
                "OCXD_STARTUP_DIAG_INITIALIZE_CODEX",
                option_env!("OCXD_STARTUP_PROBE_INITIALIZE_CODEX").unwrap_or("0"),
            )
            .spawn()
            .unwrap();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > std::time::Duration::from_secs(75) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("startup diagnosis exceeded its own budget");
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        };
        assert!(
            status.success(),
            "diagnostic execution failed, never accepted"
        );
        println!("startup-diagnostic-only: SID={identity} accepted=false fullSoftwarePassed=false original executor owns terminal process cleanup");
    }

    #[test]
    fn ordinary_profile_temp_and_exact_child_round_trip() {
        assert!(
            env::var_os(CHILD_MARKER).is_none(),
            "child must run only exact helper"
        );
        let identity = check_context();
        let fixture = Fixture::new();
        let parent_file = fixture.0.join("parent.txt");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&parent_file)
            .unwrap();
        file.write_all(b"ordinary-preflight-parent").unwrap();
        file.sync_all().unwrap();
        drop(file);
        assert_eq!(
            fs::read(&parent_file).unwrap(),
            b"ordinary-preflight-parent"
        );
        let nonce = format!(
            "{}-{}",
            std::process::id(),
            fixture.0.file_name().unwrap().to_string_lossy()
        );
        let status = Command::new(env::current_exe().expect("actual test binary"))
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .env(CHILD_MARKER, &nonce)
            .env("OCXD_PREFLIGHT_EXPECTED_MARKER", &nonce)
            .env("OCXD_PREFLIGHT_DIRECTORY", &fixture.0)
            .env("OCXD_PREFLIGHT_PARENT_IDENTITY", &identity)
            .status()
            .expect("launch only this exact child helper");
        assert!(status.success(), "exact child helper failed");
        assert_eq!(
            fs::read(fixture.0.join("child.txt")).unwrap(),
            b"ordinary-preflight-child"
        );
        fs::remove_file(&parent_file).unwrap();
        fs::remove_file(fixture.0.join("child.txt")).unwrap();
        fs::remove_dir(&fixture.0).unwrap();
        assert!(
            !fixture.0.exists(),
            "own preflight fixture cleanup incomplete"
        );
        println!("executor-preflight-only: own TEMP roundtrip and exact child passed; fullSoftwarePassed=false");
    }

    #[test]
    fn child_helper() {
        let Some(marker) = env::var_os(CHILD_MARKER) else {
            return;
        };
        assert_eq!(Some(marker), env::var_os("OCXD_PREFLIGHT_EXPECTED_MARKER"));
        let identity = check_context();
        assert_eq!(
            Some(identity.into()),
            env::var_os("OCXD_PREFLIGHT_PARENT_IDENTITY"),
            "parent/child SID differs"
        );
        let dir = PathBuf::from(
            env::var_os("OCXD_PREFLIGHT_DIRECTORY").expect("exact fixture directory"),
        );
        same_directory(dir.parent().unwrap(), &env::temp_dir());
        assert!(dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("ocxd023-ordinary-preflight-"));
        assert_eq!(
            fs::read(dir.join("parent.txt")).unwrap(),
            b"ordinary-preflight-parent"
        );
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("child.txt"))
            .unwrap();
        file.write_all(b"ordinary-preflight-child").unwrap();
        file.sync_all().unwrap();
        println!(
            "executor-preflight-only: exact child wrote own fixture; fullSoftwarePassed=false"
        );
    }

    #[cfg(windows)]
    mod windows {
        use std::{ffi::c_void, path::PathBuf, ptr};
        #[repr(C)]
        struct Guid(u32, u16, u16, [u8; 8]);
        #[link(name = "shell32")]
        extern "system" {
            fn SHGetKnownFolderPath(
                id: *const Guid,
                flags: u32,
                token: *mut c_void,
                path: *mut *mut u16,
            ) -> i32;
        }
        #[link(name = "ole32")]
        extern "system" {
            fn CoTaskMemFree(value: *mut c_void);
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut c_void;
            fn CloseHandle(handle: *mut c_void) -> i32;
            fn LocalFree(value: *mut c_void) -> *mut c_void;
        }
        #[link(name = "advapi32")]
        extern "system" {
            fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
            fn GetTokenInformation(
                token: *mut c_void,
                class: u32,
                buffer: *mut c_void,
                length: u32,
                needed: *mut u32,
            ) -> i32;
            fn ConvertSidToStringSidW(sid: *mut c_void, text: *mut *mut u16) -> i32;
        }
        unsafe fn wide(pointer: *const u16) -> String {
            let mut len = 0;
            while *pointer.add(len) != 0 {
                len += 1;
                assert!(len < 32768);
            }
            String::from_utf16(std::slice::from_raw_parts(pointer, len)).expect("UTF16 OS value")
        }
        fn folder(id: Guid) -> PathBuf {
            let mut path = ptr::null_mut();
            // Read only the actual current token's known folder; flags=0 never creates it.
            unsafe {
                assert_eq!(
                    SHGetKnownFolderPath(&id, 0, ptr::null_mut(), &mut path),
                    0,
                    "KnownFolder read failed"
                );
                assert!(!path.is_null());
                let value = wide(path);
                CoTaskMemFree(path.cast());
                PathBuf::from(value)
            }
        }
        pub fn context() -> (PathBuf, PathBuf, String) {
            let profile = folder(Guid(
                0x5e6c858f,
                0x0e22,
                0x4760,
                [0x9a, 0xfe, 0xea, 0x33, 0x17, 0xb6, 0x71, 0x73],
            ));
            let local = folder(Guid(
                0xf1b32785,
                0x6fba,
                0x4fcf,
                [0x9d, 0x55, 0x7b, 0x8e, 0x7f, 0x15, 0x70, 0x91],
            ));
            unsafe {
                let mut token = ptr::null_mut();
                assert_ne!(OpenProcessToken(GetCurrentProcess(), 8, &mut token), 0);
                let mut needed = 0;
                GetTokenInformation(token, 1, ptr::null_mut(), 0, &mut needed);
                assert!(needed > 0 && needed < 65536);
                let mut aligned = vec![
                    0usize;
                    (needed as usize + std::mem::size_of::<usize>() - 1)
                        / std::mem::size_of::<usize>()
                ];
                assert_ne!(
                    GetTokenInformation(token, 1, aligned.as_mut_ptr().cast(), needed, &mut needed),
                    0
                );
                let sid = *(aligned.as_ptr() as *const *mut c_void);
                let mut text = ptr::null_mut();
                assert_ne!(ConvertSidToStringSidW(sid, &mut text), 0);
                let identity = wide(text);
                LocalFree(text.cast());
                CloseHandle(token);
                (profile, local, identity)
            }
        }
    }
}
