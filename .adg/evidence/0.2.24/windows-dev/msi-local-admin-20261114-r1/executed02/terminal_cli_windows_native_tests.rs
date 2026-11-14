//! Opt-in product tests. Writes real HKCU PATH only for one freshly provisioned
//! ocxw1c* account with an exact SID guard, never the developer's daily account.
use super::*;
use crate::infrastructure::windows_official_runtime024::current_identity;
use crate::modules::runtime::{bundled, install, program_image, OFFICIAL_PACKAGE};
use serde_json::{json, Value};
use std::os::windows::process::CommandExt;
use std::{cell::RefCell, io::Write, process::Stdio, time::Instant};

struct Evidence(PathBuf);
impl Evidence {
    fn event(&self, name: &str, value: Value) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.0.join("native-events.jsonl"))
            .unwrap();
        writeln!(file, "{}", json!({"event":name,"facts":value})).unwrap();
        file.sync_all().unwrap();
    }
    fn save(&self, name: &str, bytes: &[u8]) {
        fs::write(self.0.join(name), bytes).unwrap();
    }
}
struct ObservedNative {
    last: RefCell<Option<PathValue>>,
    original: Option<PathValue>,
    fail_notify: std::cell::Cell<bool>,
}
impl ObservedNative {
    fn new() -> Self {
        let original = Native.read().unwrap();
        Self {
            last: RefCell::new(original.clone()),
            original,
            fail_notify: false.into(),
        }
    }
    fn owned_external(&self, value: Option<PathValue>) {
        let before = self.read().unwrap();
        self.replace(&before, &value).unwrap();
    }
    fn restored(&self) {
        assert_eq!(Native.read().unwrap(), self.original);
    }
}
impl Registry for ObservedNative {
    fn read(&self) -> AppResult<Option<PathValue>> {
        Native.read()
    }
    fn replace(&self, before: &Option<PathValue>, after: &Option<PathValue>) -> AppResult<()> {
        Native.replace(before, after)?;
        *self.last.borrow_mut() = after.clone();
        Ok(())
    }
    fn notify(&self) -> AppResult<()> {
        if self.fail_notify.get() {
            Err(fail("explicit test notification interruption"))
        } else {
            Native.notify()
        }
    }
}
impl Drop for ObservedNative {
    fn drop(&mut self) {
        let current = Native.read().expect("read isolated-account final PATH");
        // An unknown external value is never overwritten, even by test cleanup.
        assert!(
            current == *self.last.borrow() || current == self.original,
            "isolated-account PATH has unknown external bytes; retained"
        );
        Native
            .replace(&current, &self.original)
            .expect("restore exact isolated-account PATH");
        assert_eq!(Native.read().unwrap(), self.original);
    }
}
fn create(path: &Path, bytes: &[u8]) {
    platform::create_private_dir_all(path.parent().unwrap(), 0o700).unwrap();
    let mut file = platform::create_new_private_file(path, 0o600).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}
fn copy(source: &Path, target: &Path) {
    create(target, &fs::read(source).unwrap());
}
fn execute(c: &Context, action: Action, r: &impl Registry) {
    let p = plan(c, action, &[], r).unwrap().1;
    apply_with(c, action, &[], &p.confirmation_digest, r, &|_| Ok(())).unwrap();
}
fn helper(c: &Context, args: &[&str]) -> std::process::Output {
    Command::new(console(c)).args(args).output().unwrap()
}
fn save_output(e: &Evidence, label: &str, output: &std::process::Output) {
    e.save(&format!("{label}.stdout"), &output.stdout);
    e.save(&format!("{label}.stderr"), &output.stderr);
    e.event(label,json!({"exitCode":output.status.code(),"stdoutBytes":output.stdout.len(),"stderrBytes":output.stderr.len()}));
}
fn publish_fixture(
    c: &Context,
    node: &Path,
    small_exe: &Path,
) -> program_selection::CurrentSelection {
    // Explicit synthetic package for argv/stdio/exit facts, not official bytes.
    let (_, platform) = program_layout::host_target().unwrap();
    let p = program_layout::RuntimeVersionPaths::new(
        &c.program_root.join("runtime/versions"),
        "9.0.0",
        platform,
    )
    .unwrap();
    let package = p.package_root.join(install::PACKAGE_SUBPATH);
    copy(node, &p.node);
    copy(small_exe, &p.bun);
    copy(
        small_exe,
        &p.package_root.join("node_modules/bun/bin/bun.exe"),
    );
    create(&p.npm, b"// synthetic npm fixture, never invoked\n");
    create(
        &package.join("package.json"),
        br#"{"name":"@bitkyc08/opencodex","version":"9.0.0","bin":{"ocx":"probe.cjs"}}"#,
    );
    create(&package.join("probe.cjs"),br#"const fs=require('fs');const input=fs.readFileSync(0);process.stdout.write(JSON.stringify({args:process.argv.slice(2),stdin:input.toString('base64'),home:process.env.HOME,profile:process.env.USERPROFILE,h:process.env.OPENCODEX_HOME,node:process.execPath,script:__filename}));process.stderr.write('native-fixture-stderr');process.exit(23);"#);
    let digest = |path: &Path| hash::sha256_file(path).unwrap();
    let manifest = program_selection::ImageManifest {
        schema_version: 1,
        package: OFFICIAL_PACKAGE.into(),
        version: "9.0.0".into(),
        platform: "win32-x64".into(),
        official_archive_sha256: hash::sha256_hex(b"explicit synthetic argv fixture, not official"),
        tree_sha256: program_image::content_digest(&p.version_root).unwrap(),
        bin: "probe.cjs".into(),
        package_sha256: digest(&package.join("package.json")),
        bin_sha256: digest(&package.join("probe.cjs")),
        node_sha256: digest(&p.node),
        npm_sha256: digest(&p.npm),
        bun_sha256: digest(&p.bun),
        bun_launcher_sha256: digest(&p.package_root.join("node_modules/bun/bin/bun.exe")),
    };
    let bytes = serde_json::to_vec(&manifest).unwrap();
    create(
        &p.version_root.join(program_selection::IMAGE_MANIFEST),
        &bytes,
    );
    let selection = program_selection::CurrentSelection {
        schema_version: 1,
        version: "9.0.0".into(),
        platform: "win32-x64".into(),
        image_sha256: hash::sha256_hex(&bytes),
    };
    select(c, &selection);
    selection
}
fn select(c: &Context, selection: &program_selection::CurrentSelection) {
    let (_, platform) = program_layout::host_target().unwrap();
    let p = program_layout::RuntimeVersionPaths::new(
        &c.program_root.join("runtime/versions"),
        &selection.version,
        platform,
    )
    .unwrap();
    let script =
        p.package_root
            .join(install::PACKAGE_SUBPATH)
            .join(if selection.version == "9.0.0" {
                "probe.cjs"
            } else {
                "bin/ocx.mjs"
            });
    atomic_write(
        &c.program_root.join("runtime/current.json"),
        &serde_json::to_vec(selection).unwrap(),
        0o600,
    )
    .unwrap();
    atomic_write(
        &c.program_root.join("runtime/bin/ocx-runtime.cmd"),
        install::entry_script(Some(&p.node), &script).as_bytes(),
        0o600,
    )
    .unwrap();
    program_selection::read_complete(&c.program_root, platform, "win32-x64")
        .unwrap()
        .unwrap();
}

#[test]
fn private_reader_matches_w0_bound_without_touching_user_path() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("owned-private-root");
    platform::create_private_dir_all(&root, 0o700).unwrap();
    let record = root.join("terminal.json");
    create(&record, b"private record bytes");
    assert_eq!(
        private_read(&root, &record).unwrap(),
        Some(b"private record bytes".to_vec())
    );
    assert_eq!(PRIVATE_MAX, 1024 * 1024);
    assert!(
        crate::infrastructure::windows_official_runtime024::PrivateRecord::open(
            &record,
            PRIVATE_MAX + 1
        )
        .is_err()
    );
    assert_eq!(fs::read(record).unwrap(), b"private record bytes");
}

// userenv constructs a fresh current-token environment, including the real
// isolated HKCU PATH. No parent PATH concatenation pretends to be a new terminal.
fn fresh_shell(exe: &Path) -> Command {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt, ptr};
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::TOKEN_QUERY,
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    #[link(name = "userenv")]
    unsafe extern "system" {
        fn CreateEnvironmentBlock(
            block: *mut *mut std::ffi::c_void,
            token: *mut std::ffi::c_void,
            inherit: i32,
        ) -> i32;
        fn DestroyEnvironmentBlock(block: *mut std::ffi::c_void) -> i32;
    }
    let mut token = ptr::null_mut();
    let mut block = ptr::null_mut();
    assert_ne!(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | 2, &mut token) },
        0
    );
    assert_ne!(unsafe { CreateEnvironmentBlock(&mut block, token, 0) }, 0);
    let mut c = Command::new(exe);
    c.env_clear();
    let mut offset = 0usize;
    while unsafe { *block.cast::<u16>().add(offset) } != 0 {
        let n = (0..32768)
            .find(|n| unsafe { *block.cast::<u16>().add(offset + n) } == 0)
            .unwrap();
        let units = unsafe { std::slice::from_raw_parts(block.cast::<u16>().add(offset), n) };
        if let Some(at) = units.iter().position(|u| *u == 61) {
            if at > 0 {
                c.env(
                    OsString::from_wide(&units[..at]),
                    OsString::from_wide(&units[at + 1..]),
                );
            }
        }
        offset += n + 1;
        assert!(offset < 1024 * 1024);
    }
    unsafe {
        DestroyEnvironmentBlock(block);
        CloseHandle(token);
    }
    c
}
fn installer(
    e: &Evidence,
    msi: &Path,
    i: &Path,
    label: &str,
    remove: bool,
) -> std::process::Output {
    let mut c = Command::new("C:\\Windows\\System32\\msiexec.exe");
    // MSI's PROPERTY="value" grammar is not C argv parsing. Quote the value,
    // not the complete PROPERTY=value argument (Rust args() would do the latter).
    for path in [msi, i, &e.0] {
        assert!(!path.to_string_lossy().contains('"'));
    }
    let mut text = format!(
        "{} \"{}\" /qn /norestart /L*v \"{}\"",
        if remove { "/x" } else { "/i" },
        msi.to_string_lossy().replace('/', "\\"),
        e.0.join(format!("{label}.msi.log"))
            .to_string_lossy()
            .replace('/', "\\")
    );
    if !remove {
        text.push_str(&format!(
            " INSTALLDIR=\"{}\" WIXUI_EXITDIALOGOPTIONALCHECKBOX=0",
            i.to_string_lossy().replace('/', "\\")
        ));
    }
    c.raw_arg(text);
    let out = c.output().unwrap();
    save_output(e, label, &out);
    out
}

#[link(name = "msi")]
unsafe extern "system" {
    fn MsiQueryProductStateW(product: *const u16) -> i32;
    fn MsiGetProductInfoExW(
        product: *const u16,
        sid: *const u16,
        context: u32,
        property: *const u16,
        value: *mut u16,
        length: *mut u32,
    ) -> u32;
}

struct MsiRegistration<'a> {
    product: &'a str,
    evidence: &'a Evidence,
}
impl MsiRegistration<'_> {
    fn state(&self) -> i32 {
        let product: Vec<u16> = self.product.encode_utf16().chain(Some(0)).collect();
        unsafe { MsiQueryProductStateW(product.as_ptr()) }
    }
    fn context_query(&self, context: u32) -> u32 {
        let product: Vec<u16> = self.product.encode_utf16().chain(Some(0)).collect();
        let property: Vec<u16> = "LocalPackage".encode_utf16().chain(Some(0)).collect();
        let mut length = 0;
        unsafe {
            MsiGetProductInfoExW(
                product.as_ptr(),
                std::ptr::null(),
                context,
                property.as_ptr(),
                std::ptr::null_mut(),
                &mut length,
            )
        }
    }
}
impl Drop for MsiRegistration<'_> {
    fn drop(&mut self) {
        // Do not erase an account/Profile with an unresolved per-user product.
        // This is observation, never an unreviewed uninstall/registry cleanup.
        let state = self.state();
        let bytes = serde_json::to_vec(&json!({
            "productCode":self.product,"state":state,"safeToRemoveTestIdentity":state == -1
        }))
        .unwrap();
        if let Err(error) = fs::write(self.evidence.0.join("msi-final-registration.json"), bytes) {
            eprintln!("MSI registration cleanup readback could not be saved: {error}");
        }
    }
}

fn msi_program_snapshot(c: &Context) -> Value {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let mut rows = serde_json::Map::new();
    for name in ["opencodex-desktop.exe", "ocxd.exe", "ocx.exe", ENTRY_SOURCE] {
        let path = c.program_root.join(name);
        safety::check_path(&c.program_root, &path, false).unwrap();
        let file = safety::open_file(&path, false).unwrap();
        safety::require_single_link(&file).unwrap();
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        assert_ne!(
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
            0
        );
        rows.insert(
            name.into(),
            json!({
                "volume":info.dwVolumeSerialNumber,
                "fileId":((info.nFileIndexHigh as u64)<<32)|info.nFileIndexLow as u64,
                "bytes":file.metadata().unwrap().len(),"sha256":hash::sha256_file(&path).unwrap()
            }),
        );
    }
    Value::Object(rows)
}

fn msi_log(e: &Evidence, label: &str) -> String {
    let bytes = fs::read(e.0.join(format!("{label}.msi.log"))).unwrap();
    if bytes.starts_with(&[255, 254]) {
        assert!(bytes.len().is_multiple_of(2));
        String::from_utf16(
            &bytes[2..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    } else {
        String::from_utf8(bytes).unwrap()
    }
}

fn assert_msi_cleanup_refused_before_files(log: &str) {
    // Doing action is script generation for this deferred action. Require both
    // scheduling order and actual script failure before any RemoveFiles/FileRemove.
    let scheduled = log.find("Doing action: OCX023RevokeTerminalCli").unwrap();
    assert!(scheduled < log.find("Doing action: RemoveFiles").unwrap());
    let executed = log
        .find("Executing op: CustomActionSchedule(Action=OCX023RevokeTerminalCli,")
        .unwrap();
    let failed = executed
        + log[executed..]
            .find("CustomAction OCX023RevokeTerminalCli returned actual error code 1")
            .unwrap();
    assert!(!log[..failed].contains("Executing op: ActionStart(Name=RemoveFiles,"));
    assert!(!log[..failed].contains("Executing op: FileRemove("));
    assert!(log[failed..].contains("Return value 3"));
}

#[test]
#[ignore = "MSI-only; requires an explicitly prepared separate guest and one fresh ordinary account"]
fn isolated_msi_uninstall_only() {
    msi_uninstall_only(false);
}

#[test]
#[ignore = "explicit local disposable administrator scenario; not ordinary-user acceptance"]
fn isolated_msi_uninstall_administrator() {
    msi_uninstall_only(true);
}

fn msi_uninstall_only(administrator: bool) {
    // No component-only fallback. A prepared guest acknowledgement and actual
    // MSI install success are prerequisites, not inferred from registry absence.
    let machine = std::env::var("COMPUTERNAME").unwrap();
    if administrator {
        assert_eq!(std::env::var("OCXD024_MSI_LOCAL_ADMIN").as_deref(), Ok("1"));
    } else {
        assert_eq!(std::env::var("OCXD024_MSI_ONLY_GUEST").as_deref(), Ok("1"));
        assert_ne!(
            machine.to_uppercase(),
            "EZIO-MINI",
            "daily host is excluded"
        );
    }
    assert_eq!(
        machine.to_uppercase(),
        std::env::var("OCXD024_MSI_GUEST_NAME")
            .unwrap()
            .to_uppercase()
    );
    let token = current_identity().unwrap();
    assert_eq!(token.sid, std::env::var("OCXD024_ACCOUNT_SID").unwrap());
    assert_eq!(
        token.elevated, administrator,
        "exact authorized test token required"
    );
    assert!(!crate::modules::test_sandbox::enabled());
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1d"));
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    let e = Evidence(PathBuf::from(
        std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap(),
    ));
    let msi = input.join("w1d.msi");
    assert_eq!(fs::metadata(&msi).unwrap().len(), 316784640);
    assert_eq!(
        hash::sha256_file(&msi).unwrap(),
        "9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282"
    );
    // Bound to the exact prior dev MSI; query-package-msi.ps1 independently
    // verifies its read-only Product table before any new test account exists.
    let product = "{67CF3D62-492D-4359-AAFE-8BE6FB17E647}";
    let registration = MsiRegistration {
        product,
        evidence: &e,
    };
    if administrator {
        if let Some(prior) = std::env::var_os("OCXD024_MSI_RESUME_EVIDENCE") {
            let prior = Evidence(PathBuf::from(prior));
            let pending = fs::read(prior.0.join("pending-before.private.json")).unwrap();
            let value: Value = decode(&pending).unwrap();
            let c: Context = serde_json::from_value(value["context"].clone()).unwrap();
            assert_eq!(c.user_profile, profile);
            assert_eq!(c.program_root, profile.join("w1d native/I 空格 & !"));
            assert_eq!(registration.state(), 5);
            assert_eq!(registration.context_query(2), 0);
            assert_eq!(registration.context_query(4), 1605);
            EntryPin::open(&c).unwrap();
            assert_eq!(fs::read(c.pending()).unwrap(), pending);
            let before: Value =
                decode(&fs::read(prior.0.join("failed-before.private.json")).unwrap()).unwrap();
            let after: Value =
                decode(&fs::read(prior.0.join("failed-after.private.json")).unwrap()).unwrap();
            assert_eq!(before, after);
            assert_eq!(msi_program_snapshot(&c), after["programs"]);
            let config_before = fs::read(data_root::runtime_config_path(&c.desktop_root)).unwrap();
            assert_eq!(
                hash::sha256_hex(&config_before),
                after["configSha256"].as_str().unwrap()
            );
            assert_eq!(
                hash::sha256_file(&c.desktop_root.join("exports/msi-only-retain.txt")).unwrap(),
                after["dataSentinelSha256"].as_str().unwrap()
            );
            assert_eq!(
                hash::sha256_file(&c.opencodex_home.join("msi-only-retain.txt")).unwrap(),
                after["homeSentinelSha256"].as_str().unwrap()
            );
            let r = ObservedNative::new();
            assert_eq!(serde_json::to_value(&r.original).unwrap(), after["path"]);
            assert_msi_cleanup_refused_before_files(&msi_log(&prior, "msi-only-uninstall-refused"));
            e.event("resumed-original-msi-refusal", json!({"priorEvidence":prior.0,"sameSid":token.sid,"elevated":true,"programIdentityAndBytesRetained":true,"pendingBytesRetained":true,"beforeRemoveFiles":true}));
            complete_msi_uninstall(&c, &r, &e, &registration, &msi, &config_before);
            return;
        }
    }
    assert_eq!(
        registration.state(),
        -1,
        "fresh test product identity required"
    );
    let r = ObservedNative::new();
    e.save("path-before.private.json", &encode(&r.original).unwrap());
    e.event("hypothesis",json!({"scope":"only actual MSI transactions; no terminal/official rerun", "machine":machine,"sid":token.sid,"session":token.session_id,"elevated":token.elevated,"administratorScenario":administrator,"ordinaryUserAcceptance":!administrator,"product":product}));
    let base = profile.join("w1d native");
    assert!(!base.exists());
    platform::create_private_dir_all(&base, 0o700).unwrap();
    let i = base.join("I 空格 & !");
    let installed = installer(&e, &msi, &i, "msi-only-install", false);
    assert!(
        installed.status.success(),
        "installation prerequisite failed; dependent uninstall cases not executed"
    );
    assert_eq!(
        registration.state(),
        5,
        "MSI current-user registration absent"
    );
    // Elevation must not silently turn this package into a machine install.
    let per_user = registration.context_query(2);
    let per_machine = registration.context_query(4);
    e.event(
        "installed-context",
        json!({"perUserUnmanagedQuery":per_user,"perMachineQuery":per_machine}),
    );
    assert!(
        per_user == 0 || per_user == 234,
        "per-user MSI context required"
    );
    assert_eq!(
        per_machine, 1605,
        "machine installation is outside this scenario"
    );
    let d = base.join("data/desktop");
    data_root::initialize(&d).unwrap();
    let h = base.join("custom H 雪");
    platform::create_private_dir_all(&h, 0o700).unwrap();
    data_root::set_opencodex_home(&d, data_root::OpenCodexHomeMode::External, Some(&h)).unwrap();
    data_root::startup_v2::set_location(
        &profile.join(".codex-desktop/desktop"),
        d.parent().unwrap(),
    )
    .unwrap();
    let cfg =
        data_root::startup_v2::selected_config(&profile.join(".codex-desktop/desktop")).unwrap();
    let c = Context {
        user_profile: profile,
        desktop_root: cfg.active_data_root.clone(),
        opencodex_home: data_root::resolve_opencodex_home(&cfg),
        program_root: i.clone(),
        manager_executable: i.join("opencodex-desktop.exe"),
    };
    EntryPin::open(&c).unwrap();
    publish_fixture(
        &c,
        &input.join("node.exe"),
        &input.join("native024-fixture.exe"),
    );
    let selected = program_selection::read_complete(
        &i,
        crate::modules::data_root::layout::Platform::Windows,
        "win32-x64",
    )
    .unwrap()
    .unwrap();
    let programs = msi_program_snapshot(&c);
    let config = data_root::runtime_config_path(&c.desktop_root);
    let config_before = fs::read(&config).unwrap();
    let sentinel = c.desktop_root.join("exports/msi-only-retain.txt");
    create(
        &sentinel,
        b"MSI must retain current user's D/H/live runtime",
    );
    let home_sentinel = c.opencodex_home.join("msi-only-retain.txt");
    create(
        &home_sentinel,
        b"explicit test H bytes, no official command",
    );
    let pending_preview = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &c,
        Action::Enable,
        &[],
        &pending_preview.confirmation_digest,
        &r,
        &|point| if point == "pending" {
            Err(fail("explicit MSI-only pending Enable fixture"))
        } else {
            Ok(())
        }
    )
    .is_err());
    let pending_before = fs::read(c.pending()).unwrap();
    let failed_before = json!({"programs":programs,"productState":registration.state(),"path":r.read().unwrap(),
        "pendingSha256":hash::sha256_hex(&pending_before),"configSha256":hash::sha256_hex(&config_before),"selection":selected.selection,
        "dataSentinelSha256":hash::sha256_file(&sentinel).unwrap(),"homeSentinelSha256":hash::sha256_file(&home_sentinel).unwrap()});
    e.save(
        "failed-before.private.json",
        &serde_json::to_vec(&failed_before).unwrap(),
    );
    e.save("pending-before.private.json", &pending_before);
    let refused = installer(&e, &msi, &i, "msi-only-uninstall-refused", true);
    let failed_after = json!({"programs":msi_program_snapshot(&c),"productState":registration.state(),"path":r.read().unwrap(),
        "pendingSha256":hash::sha256_hex(&fs::read(c.pending()).unwrap()),
        "configSha256":hash::sha256_hex(&fs::read(&config).unwrap()),
        "selection":program_selection::read_complete(&i,crate::modules::data_root::layout::Platform::Windows,"win32-x64").unwrap().unwrap().selection,
        "dataSentinelSha256":hash::sha256_file(&sentinel).unwrap(),"homeSentinelSha256":hash::sha256_file(&home_sentinel).unwrap()});
    e.save(
        "failed-after.private.json",
        &serde_json::to_vec(&failed_after).unwrap(),
    );
    assert_eq!(
        refused.status.code(),
        Some(1603),
        "must be actual custom-action refusal, not policy or transport failure"
    );
    assert_eq!(registration.state(), 5);
    assert_eq!(msi_program_snapshot(&c), programs);
    r.restored();
    assert_eq!(fs::read(c.pending()).unwrap(), pending_before);
    assert_eq!(fs::read(&config).unwrap(), config_before);
    assert!(!c.record().exists());
    program_image::verify(&selected).unwrap();
    let refused_log = msi_log(&e, "msi-only-uninstall-refused");
    assert_msi_cleanup_refused_before_files(&refused_log);
    assert_eq!(failed_after, failed_before);
    e.event("failed-uninstall-retained", json!({"exitCode":refused.status.code(),"programIdentityAndBytesRetained":true,
        "pendingBytesRetained":true,"productState":5,"beforeRemoveFiles":true,"pathUnchanged":true,"liveRuntimeRetained":true}));
    complete_msi_uninstall(&c, &r, &e, &registration, &msi, &config_before);
}

fn complete_msi_uninstall(
    c: &Context,
    r: &ObservedNative,
    e: &Evidence,
    registration: &MsiRegistration<'_>,
    msi: &Path,
    config_before: &[u8],
) {
    let i = &c.program_root;
    let config = data_root::runtime_config_path(&c.desktop_root);
    let sentinel = c.desktop_root.join("exports/msi-only-retain.txt");
    let home_sentinel = c.opencodex_home.join("msi-only-retain.txt");
    let selected = program_selection::read_complete(
        i,
        crate::modules::data_root::layout::Platform::Windows,
        "win32-x64",
    )
    .unwrap()
    .unwrap();
    // Continue the original Enable with a new explicit preview; do not rewrite
    // its before/after or use a new action to compensate the refused uninstall.
    let resume = inspect_with(c, &[], r).unwrap();
    assert!(resume.pending);
    assert_eq!(resume.action, Action::Enable);
    apply_with(
        c,
        resume.action,
        &[],
        &resume.confirmation_digest,
        r,
        &|_| Ok(()),
    )
    .unwrap();
    let registered = fs::read(c.record()).unwrap();
    e.save("active-registration.private.json", &registered);
    e.save(
        "active-path.private.json",
        &encode(&r.read().unwrap()).unwrap(),
    );
    let removed = installer(e, msi, i, "msi-only-uninstall-normal", true);
    // Preserve the observed original PATH on success before any further assert,
    // so a later unrelated failure cannot make cleanup treat it as an outsider.
    if removed.status.success() && r.read().unwrap() == r.original {
        *r.last.borrow_mut() = r.original.clone();
    }
    assert!(removed.status.success());
    assert_eq!(registration.state(), -1);
    r.restored();
    assert!(!c.record().exists() && !c.pending().exists());
    for name in ["opencodex-desktop.exe", "ocxd.exe", "ocx.exe", ENTRY_SOURCE] {
        assert!(
            !i.join(name).exists(),
            "installer-owned component retained: {name}"
        );
    }
    assert_eq!(fs::read(&config).unwrap(), config_before);
    assert_eq!(
        fs::read(&sentinel).unwrap(),
        b"MSI must retain current user's D/H/live runtime"
    );
    assert_eq!(
        fs::read(&home_sentinel).unwrap(),
        b"explicit test H bytes, no official command"
    );
    program_image::verify(&selected).unwrap();
    let normal_log = msi_log(e, "msi-only-uninstall-normal");
    assert!(
        normal_log
            .find("Doing action: OCX023RevokeTerminalCli")
            .unwrap()
            < normal_log.find("Doing action: RemoveFiles").unwrap()
    );
    assert!(
        normal_log
            .find("Executing op: CustomActionSchedule(Action=OCX023RevokeTerminalCli,")
            .unwrap()
            < normal_log
                .find("Executing op: ActionStart(Name=RemoveFiles,")
                .unwrap()
    );
    e.save("normal-after.private.json", &serde_json::to_vec(&json!({
        "productState":registration.state(),"path":r.read().unwrap(),
        "terminalRecordAbsent":!c.record().exists(),"pendingAbsent":!c.pending().exists(),
        "ownedProgramsAbsent":(["opencodex-desktop.exe","ocxd.exe","ocx.exe",ENTRY_SOURCE].iter().all(|name|!i.join(name).exists())),
        "configSha256":hash::sha256_file(&config).unwrap(),
        "dataSentinelSha256":hash::sha256_file(&sentinel).unwrap(),
        "homeSentinelSha256":hash::sha256_file(&home_sentinel).unwrap(),"selection":selected.selection
    })).unwrap());
    e.save(
        "path-after.private.json",
        &encode(&r.read().unwrap()).unwrap(),
    );
    e.event("msi-only-complete",json!({"normalUninstall":true,"failedRetention":true,"pathRestored":true,
        "productState":registration.state(),"ownedComponentsRemoved":true,"DHomeLiveRuntimeRetained":true,
        "terminalTestsRerun":0,"officialCommands":0,"W2":false,"dualSid":false}));
}

#[test]
#[ignore = "one fresh ordinary ocxw1d account, exact SID; actual MSVC product/dev MSI only"]
fn isolated_account_terminal_w1d() {
    let token = current_identity().unwrap();
    assert_eq!(token.sid, std::env::var("OCXD024_ACCOUNT_SID").unwrap());
    assert!(!token.elevated);
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1d"));
    assert!(!crate::modules::test_sandbox::enabled());
    if let Some(path) = std::env::var_os("OCXD024_RESUME_CONTEXT") {
        let c: Context = decode(&fs::read(path).unwrap()).unwrap();
        assert_eq!(c.user_profile, profile);
        assert!(c.program_root.starts_with(profile.join("w1d native")));
        let p = inspect_with(&c, &[], &Native).unwrap();
        assert!(p.pending);
        apply_with(&c, p.action, &[], &p.confirmation_digest, &Native, &|_| {
            Ok(())
        })
        .unwrap();
        assert!(!c.pending().exists());
        return;
    }
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    let e = Evidence(PathBuf::from(
        std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap(),
    ));
    let r = ObservedNative::new();
    e.save("path-before.private.json", &encode(&r.original).unwrap());
    let base = profile.join("w1d native");
    assert!(!base.exists());
    platform::create_private_dir_all(&base, 0o700).unwrap();
    let i = base.join("I 空格 & !");
    let msi = input.join("w1d.msi");
    e.event("identity",json!({"sid":token.sid,"profile":profile,"session":token.session_id,"elevated":token.elevated,"hypothesis":"native exe, schema3, fresh-shell environment and actual MSI uninstall"}));
    let component_only = std::env::var("OCXD024_COMPONENT_ONLY").as_deref() == Ok("1");
    if component_only {
        for name in ["opencodex-desktop.exe", "ocxd.exe", "ocx.exe", ENTRY_SOURCE] {
            copy(&input.join(name), &i.join(name));
        }
        e.event("component-fixture",json!({"reason":"real ordinary MSI blocked by machine DisableMsi policy, no policy changed","msiInstalled":false,"source":"exact current built binaries and installer source manifest"}));
    } else {
        let install = installer(&e, &msi, &i, "msi-install", false);
        assert!(install.status.success());
    }
    for n in ["opencodex-desktop.exe", "ocxd.exe", "ocx.exe", ENTRY_SOURCE] {
        assert!(i.join(n).is_file(), "required MSI program {n}");
    }
    let shim = hash::sha256_file(&i.join("ocx.exe")).unwrap();
    let pristine = Command::new(i.join("ocxd.exe"))
        .args(["--uninstall-terminal-cli", "--confirm"])
        .output()
        .unwrap();
    save_output(&e, "pristine-helper", &pristine);
    assert!(pristine.status.success());
    assert!(!profile.join(".codex-desktop").exists());
    let direct = Command::new(i.join("ocx.exe"))
        .arg("--version")
        .output()
        .unwrap();
    save_output(&e, "unregistered-shim", &direct);
    assert!(!direct.status.success());
    let d = base.join("data/desktop");
    data_root::initialize(&d).unwrap();
    let h = base.join("custom H 雪");
    platform::create_private_dir_all(&h, 0o700).unwrap();
    data_root::set_opencodex_home(&d, data_root::OpenCodexHomeMode::External, Some(&h)).unwrap();
    data_root::startup_v2::set_location(
        &profile.join(".codex-desktop/desktop"),
        d.parent().unwrap(),
    )
    .unwrap();
    let cfg =
        data_root::startup_v2::selected_config(&profile.join(".codex-desktop/desktop")).unwrap();
    let c = Context {
        user_profile: profile.clone(),
        desktop_root: cfg.active_data_root.clone(),
        opencodex_home: data_root::resolve_opencodex_home(&cfg),
        manager_executable: i.join("opencodex-desktop.exe"),
        program_root: i.clone(),
    };
    EntryPin::open(&c).unwrap();
    let synthetic = publish_fixture(
        &c,
        &input.join("node.exe"),
        &input.join("native024-fixture.exe"),
    );
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(!c.record().exists() && !c.pending().exists());
    assert!(
        apply_with(&c, Action::Enable, &[], "wrong-confirmation", &r, &|_| Ok(
            ()
        ))
        .is_err()
    );
    r.restored();
    execute(&c, Action::Enable, &r);
    execute(&c, Action::Enable, &r);
    let record: Installed = decode(&fs::read(c.record()).unwrap()).unwrap();
    assert_eq!(record.schema_version, 3);
    assert_eq!(record.entry_kind, ENTRY_KIND);
    assert_eq!(record.command_path, i.join("ocx.exe"));
    assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
    e.event("schema3-enabled",json!({"schema":3,"kind":record.entry_kind,"command":record.command_path,"pathDirectory":directory(&c).unwrap(),"shimUntouched":true}));
    let payload = b"stdin binary \0 Unicode \xe9\x9b\xaa\r\n";
    let expected = json!([
        "plain",
        "arg space",
        "雪",
        "&|<>^!()",
        "literal%text",
        "trailing\\",
        "",
        "quoted\"inside"
    ]);
    let batch = base.join("argv.cmd");
    create(&batch,b"@echo off\r\nchcp 65001 >nul\r\nsetlocal DisableDelayedExpansion\r\nocx plain \"arg space\" \"\xe9\x9b\xaa\" \"&|<>^!()\" \"literal%%text\" \"trailing\\\\\" \"\" \"quoted\\\"inside\"\r\n");
    let ps = PathBuf::from(std::env::var_os("OCXD024_PWSH").unwrap());
    let psmeta = e.0.join("powershell-version.json");
    let pscript=format!("[pscustomobject]@{{version=$PSVersionTable.PSVersion.ToString();passing=$PSNativeCommandArgumentPassing.ToString();target=(Get-Command ocx -ErrorAction Stop).Source}}|ConvertTo-Json|Set-Content -LiteralPath '{}'; & ocx @('plain','arg space','雪','&|<>^!()','literal%text','trailing\\','','quoted\"inside'); exit $LASTEXITCODE",psmeta.display());
    let mut failed = vec![];
    for (label, exe, script) in [
        (
            "cmd",
            PathBuf::from(std::env::var_os("COMSPEC").unwrap()),
            format!("\"\"{}\"\"", batch.display()),
        ),
        ("powershell", ps.clone(), pscript),
    ] {
        let mut process = fresh_shell(&exe);
        process.current_dir(&base);
        if label == "cmd" {
            process.args(["/d", "/s", "/c"]).raw_arg(script);
        } else {
            process.args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &script,
            ]);
        }
        process
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = process.spawn().unwrap();
        child.stdin.take().unwrap().write_all(payload).unwrap();
        let out = child.wait_with_output().unwrap();
        save_output(&e, &format!("argv-{label}"), &out);
        let value = serde_json::from_slice::<Value>(&out.stdout);
        let passed = out.status.code() == Some(23)
            && out.stderr == b"native-fixture-stderr"
            && value.as_ref().is_ok_and(|v| {
                v["args"] == expected
                    && v["stdin"]
                        == base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            payload,
                        )
                    && v["home"] == profile.to_str().unwrap()
                    && v["profile"] == profile.to_str().unwrap()
                    && v["h"] == c.opencodex_home.to_str().unwrap()
            });
        e.event(
            "argv",
            json!({"shell":label,"passed":passed,"expected":expected,"observed":value.ok()}),
        );
        if !passed {
            failed.push(format!("argv-{label}"));
        }
    }
    let discover = fresh_shell(&ps)
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Command ocx -ErrorAction Stop).Source; & where.exe ocx; exit $LASTEXITCODE",
        ])
        .current_dir(&base)
        .output()
        .unwrap();
    save_output(&e, "fresh-bare-discovery", &discover);
    assert!(discover.status.success());
    let text = String::from_utf8_lossy(&discover.stdout).to_lowercase();
    assert!(text.contains(&command(&c).to_string_lossy().to_lowercase()));
    execute(&c, Action::Disable, &r);
    r.restored();
    assert!(command(&c).is_file());
    assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
    let disabled = fresh_shell(&ps)
        .args([
            "-NoProfile",
            "-Command",
            "if(Get-Command ocx -ErrorAction SilentlyContinue){exit 1};exit 0",
        ])
        .current_dir(&base)
        .output()
        .unwrap();
    save_output(&e, "disabled-bare-discovery", &disabled);
    assert!(disabled.status.success());
    let resume = e.0.join("resume-context.private.json");
    fs::write(&resume, encode(&c).unwrap()).unwrap();
    for a in [Action::Enable, Action::Disable] {
        for at in ["pending", "command", "path", "record", "notified"] {
            if a == Action::Disable {
                execute(&c, Action::Enable, &r);
            }
            let p = plan(&c, a, &[], &r).unwrap().1;
            assert!(apply_with(
                &c,
                a,
                &[],
                &p.confirmation_digest,
                &r,
                &|point| if point == at {
                    Err(fail("explicit schema3 boundary interruption"))
                } else {
                    Ok(())
                }
            )
            .is_err());
            let raw = fs::read(c.pending()).unwrap();
            let t: Transaction = decode(&raw).unwrap();
            assert!(!String::from_utf8_lossy(&raw).contains("MZ"));
            e.save(&format!("pending-{a:?}-{at}.private.json"), &raw);
            let out=Command::new(std::env::current_exe().unwrap()).args(["--exact","modules::runtime::terminal_cli::terminal_cli_windows::native_tests::isolated_account_terminal_w1d","--ignored","--test-threads=1","--nocapture"]).env("OCXD024_RESUME_CONTEXT",&resume).output().unwrap();
            save_output(&e, &format!("resume-{a:?}-{at}"), &out);
            assert!(out.status.success());
            assert_eq!(r.read().unwrap(), t.path_after);
            *r.last.borrow_mut() = t.path_after;
            assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
            if a == Action::Enable {
                execute(&c, Action::Disable, &r);
            }
            r.restored();
        }
    }
    // Same-image upgrade can replace file IDs between actions without permanently
    // invalidating registration. A live action's retained handles deny replacement.
    execute(&c, Action::Enable, &r);
    let saved = fs::read(c.record()).unwrap();
    {
        let pin = EntryPin::open(&c).unwrap();
        assert!(fs::rename(command(&c), i.join("moved.exe")).is_err());
        pin.check(&c).unwrap();
    }
    let bytes = fs::read(command(&c)).unwrap();
    fs::remove_file(command(&c)).unwrap();
    create(&command(&c), &bytes);
    assert_eq!(fs::read(c.record()).unwrap(), saved);
    require_forwarding(&c).unwrap();
    execute(&c, Action::Disable, &r);
    r.restored();
    for object in ["entry", "path", "record", "pending"] {
        let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
        assert!(apply_with(
            &c,
            Action::Enable,
            &[],
            &p.confirmation_digest,
            &r,
            &|point| if point == "command" {
                Err(fail("explicit conflict checkpoint"))
            } else {
                Ok(())
            }
        )
        .is_err());
        let pending_raw = fs::read(c.pending()).unwrap();
        let before = r.read().unwrap();
        match object {
            "entry" => atomic_write(&command(&c), b"foreign exe", 0o600).unwrap(),
            "path" => r.owned_external(Some(PathValue::encode(1, "C:\\w1d-external").unwrap())),
            "record" => atomic_write(&c.record(), b"foreign record", 0o600).unwrap(),
            _ => atomic_write(&c.pending(), b"foreign pending", 0o600).unwrap(),
        }
        assert!(plan(&c, Action::Enable, &[], &r).is_err());
        match object {
            "entry" => {
                assert_eq!(fs::read(command(&c)).unwrap(), b"foreign exe");
                atomic_write(&command(&c), &bytes, 0o600).unwrap();
            }
            "path" => r.owned_external(before),
            "record" => {
                fs::remove_file(c.record()).unwrap();
            }
            _ => {
                atomic_write(&c.pending(), &pending_raw, 0o600).unwrap();
            }
        }
        // Replacing a file changes its ID, so its original transaction cannot be
        // resumed. Restore only this test-owned pending's exact original object
        // via cancelling test state, never claim production rebaselining.
        if object == "entry" {
            assert!(plan(&c, Action::Enable, &[], &r).is_err());
            fs::remove_file(c.pending()).unwrap();
        } else {
            execute(&c, Action::Enable, &r);
            execute(&c, Action::Disable, &r);
        }
        r.restored();
        e.event(
            "external-refused",
            json!({"object":object,"bytesRetained":true}),
        );
    }
    let original_current = fs::read(i.join("runtime/current.json")).unwrap();
    e.save("synthetic-current.json", &original_current);
    fs::remove_file(i.join("runtime/current.json")).unwrap();
    fs::remove_file(i.join("runtime/bin/ocx-runtime.cmd")).unwrap();
    let resource = if component_only {
        input.clone()
    } else {
        i.join("runtime-bundle")
    };
    let descriptor = fs::read(resource.join("runtime-bundle.manifest.json")).unwrap();
    let pin = hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a"
    );
    let start = Instant::now();
    let supplied = bundled::versioned::provision(
        &i,
        &c.desktop_root,
        &resource.join("runtime-bundle-0.2.24-win32-x64.tar.gz"),
        &descriptor,
        &pin,
    );
    if let Err(error) = supplied {
        e.event(
            "official-supply",
            json!({"passed":false,"error":error.to_string()}),
        );
        failed.push("official-supply".into());
        select(&c, &synthetic);
    } else {
        let selected = program_selection::read_complete(
            &i,
            crate::modules::data_root::layout::Platform::Windows,
            "win32-x64",
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected.selection.version, "2.99.0");
        execute(&c, Action::Enable, &r);
        create(&c.opencodex_home.join("config.json"), br#"{"port":18241}"#);
        let config_before = fs::read(c.opencodex_home.join("config.json")).unwrap();
        for (label, args) in [
            ("official-version", vec!["--version"]),
            ("official-home", vec!["config", "get", "port", "--json"]),
        ] {
            let script = format!("& ocx {};exit $LASTEXITCODE", args.join(" "));
            let out = fresh_shell(&ps)
                .args(["-NoProfile", "-Command", &script])
                .current_dir(&base)
                .output()
                .unwrap();
            save_output(&e, label, &out);
            let passed = out.status.success()
                && String::from_utf8_lossy(&out.stdout).contains(if label == "official-version" {
                    "2.99.0"
                } else {
                    "18241"
                });
            if !passed {
                failed.push(label.into());
            }
            e.event(label,json!({"passed":passed,"current":selected.selection,"node":selected.paths.node,"script":selected.script,"h":c.opencodex_home}));
        }
        assert_eq!(
            fs::read(c.opencodex_home.join("config.json")).unwrap(),
            config_before
        );
        program_image::verify(&selected).unwrap();
        execute(&c, Action::Disable, &r);
        r.restored();
    }
    e.event(
        "official-phase",
        json!({"elapsedMs":start.elapsed().as_millis(),"failed":failed}),
    );
    // Real Windows Installer invocation: pending Enable must abort before removal.
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|point| if point == "pending" {
            Err(fail("real MSI pending enable"))
        } else {
            Ok(())
        }
    )
    .is_err());
    let pending_raw = fs::read(c.pending()).unwrap();
    let denied = if component_only {
        helper(&c, &["--uninstall-terminal-cli", "--confirm"])
    } else {
        installer(&e, &msi, &i, "msi-uninstall-refused", true)
    };
    if component_only {
        save_output(&e, "component-helper-refused", &denied);
    }
    assert!(!denied.status.success());
    assert!(i.join("ocxd.exe").is_file() && i.join("ocx.exe").is_file());
    assert_eq!(fs::read(c.pending()).unwrap(), pending_raw);
    r.restored();
    execute(&c, Action::Enable, &r);
    let after = r.read().unwrap();
    *r.last.borrow_mut() = after;
    let sentinel = c.desktop_root.join("exports/retained.txt");
    create(&sentinel, b"user configuration retained");
    let uninstalled = if component_only {
        helper(&c, &["--uninstall-terminal-cli", "--confirm"])
    } else {
        installer(&e, &msi, &i, "msi-uninstall-normal", true)
    };
    if component_only {
        save_output(&e, "component-helper-revoke", &uninstalled);
    }
    assert!(uninstalled.status.success());
    *r.last.borrow_mut() = r.original.clone();
    r.restored();
    for name in ["ocxd.exe", "ocx.exe", "opencodex-desktop.exe"] {
        assert_eq!(i.join(name).exists(), component_only);
    }
    if component_only {
        assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
    }
    assert!(!c.record().exists() && !c.pending().exists());
    assert_eq!(fs::read(&sentinel).unwrap(), b"user configuration retained");
    e.save(
        "path-after.private.json",
        &encode(&r.read().unwrap()).unwrap(),
    );
    e.event("final",json!({"allAccepted":failed.is_empty()&&!component_only,"componentAccepted":failed.is_empty(),"msiBlockedByPolicy":component_only,"failed":failed,"schema":3,"pathRestored":true,"msiFailurePreserved":!component_only,"msiUninstall":!component_only,"userDataRetained":true,"noServiceStarted":true}));
    assert!(failed.is_empty(), "independent failed cells: {failed:?}");
}

#[test]
#[ignore = "one fresh ordinary ocxw1d account; affected schema3 invariants and unchanged NSIS macro only"]
fn isolated_schema3_invariants_w1d() {
    let token = current_identity().unwrap();
    assert_eq!(token.sid, std::env::var("OCXD024_ACCOUNT_SID").unwrap());
    assert!(!token.elevated);
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1d"));
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    let e = Evidence(PathBuf::from(
        std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap(),
    ));
    let r = ObservedNative::new();
    let base = profile.join("w1d native");
    let i = base.join("I 空格 & !");
    for name in ["opencodex-desktop.exe", "ocxd.exe", "ocx.exe", ENTRY_SOURCE] {
        copy(&input.join(name), &i.join(name));
    }
    let d = base.join("data/desktop");
    data_root::initialize(&d).unwrap();
    let h = base.join("custom H 雪");
    platform::create_private_dir_all(&h, 0o700).unwrap();
    data_root::set_opencodex_home(&d, data_root::OpenCodexHomeMode::External, Some(&h)).unwrap();
    data_root::startup_v2::set_location(
        &profile.join(".codex-desktop/desktop"),
        d.parent().unwrap(),
    )
    .unwrap();
    let cfg =
        data_root::startup_v2::selected_config(&profile.join(".codex-desktop/desktop")).unwrap();
    let c = Context {
        user_profile: profile,
        desktop_root: cfg.active_data_root.clone(),
        opencodex_home: data_root::resolve_opencodex_home(&cfg),
        program_root: i.clone(),
        manager_executable: i.join("opencodex-desktop.exe"),
    };
    publish_fixture(
        &c,
        &input.join("node.exe"),
        &input.join("native024-fixture.exe"),
    );
    let shim = hash::sha256_file(&command(&c)).unwrap();
    e.save("path-before.private.json", &encode(&r.original).unwrap());
    let old = Installed {
        schema_version: 2,
        context: c.clone(),
        path_edit: policy::enable(&r.original, &directory(&c).unwrap()).unwrap(),
        command_path: command(&c),
        entry_kind: ENTRY_KIND.into(),
    };
    let raw = encode(&old).unwrap();
    create(&c.record(), &raw);
    assert!(inspect_with(&c, &[], &r).is_err());
    assert_eq!(fs::read(c.record()).unwrap(), raw);
    fs::remove_file(c.record()).unwrap();
    r.restored();
    e.event(
        "schema2-refused",
        json!({"bytesPreserved":true,"migrationExecuted":false}),
    );
    let before = Some(PathValue::encode(1, &directory(&c).unwrap()).unwrap());
    r.owned_external(before.clone());
    execute(&c, Action::Enable, &r);
    execute(&c, Action::Disable, &r);
    assert_eq!(r.read().unwrap(), before);
    r.owned_external(r.original.clone());
    e.event(
        "preexisting-equivalent-path",
        json!({"preservedExactly":true,"shimRetained":command(&c).exists()}),
    );
    r.fail_notify.set(true);
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    let pending_raw = fs::read(c.pending()).unwrap();
    e.save("notify-pending.private.json", &pending_raw);
    r.fail_notify.set(false);
    execute(&c, Action::Enable, &r);
    execute(&c, Action::Disable, &r);
    r.restored();
    e.event(
        "notification-interruption",
        json!({"injected":true,"pendingRetained":true,"continued":true}),
    );
    let run = |label: &str, upgrade: bool| {
        let marker = i.join("hook-passed.txt");
        if marker.exists() {
            fs::remove_file(&marker).unwrap();
        }
        let out = Command::new(input.join("hook-probe.exe"))
            .raw_arg(format!(
                "/S {} /D={}",
                if upgrade { "/UPDATE" } else { "" },
                i.display()
            ))
            .output()
            .unwrap();
        save_output(&e, label, &out);
        out
    };
    assert!(run("nsis-never-enabled", false).status.success());
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|point| if point == "pending" {
            Err(fail("explicit NSIS pending enable fixture"))
        } else {
            Ok(())
        }
    )
    .is_err());
    let pending = fs::read(c.pending()).unwrap();
    assert_eq!(run("nsis-pending-refused", false).status.code(), Some(1));
    assert!(!i.join("hook-passed.txt").exists());
    assert_eq!(fs::read(c.pending()).unwrap(), pending);
    assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
    assert!(run("nsis-upgrade-preserves-pending", true).status.success());
    assert_eq!(fs::read(c.pending()).unwrap(), pending);
    r.restored();
    execute(&c, Action::Enable, &r);
    let registered = fs::read(c.record()).unwrap();
    let path = r.read().unwrap();
    assert!(run("nsis-upgrade-preserves-active", true).status.success());
    assert_eq!(fs::read(c.record()).unwrap(), registered);
    assert_eq!(r.read().unwrap(), path);
    assert!(run("nsis-normal-revokes", false).status.success());
    *r.last.borrow_mut() = r.original.clone();
    r.restored();
    assert!(!c.record().exists() && !c.pending().exists());
    assert_eq!(hash::sha256_file(&command(&c)).unwrap(), shim);
    e.save(
        "path-after.private.json",
        &encode(&r.read().unwrap()).unwrap(),
    );
    e.event("final-invariants",json!({"componentAccepted":true,"nsisMacroCells":5,"schema":3,"entryUnchanged":true,"pathRestored":true,"actualInstallerRemoval":false,"msiAcceptance":false}));
}
