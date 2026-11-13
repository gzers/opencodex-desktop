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
fn hook(c: &Context, input: &Path, update: bool) -> std::process::Output {
    let marker = c.program_root.join("hook-passed.txt");
    if marker.exists() {
        fs::remove_file(&marker).unwrap();
    }
    Command::new(input.join("hook-probe.exe"))
        .raw_arg(format!(
            "/S {} /D={}",
            if update { "/UPDATE" } else { "" },
            c.program_root.display()
        ))
        .output()
        .unwrap()
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
        br#"{"name":"@bitkyc08/opencodex","version":"9.0.0","bin":{"ocx":"probe.cmd"}}"#,
    );
    create(&package.join("probe.cmd"),br#"const fs=require('fs');const input=fs.readFileSync(0);process.stdout.write(JSON.stringify({args:process.argv.slice(2),stdin:input.toString('base64'),home:process.env.HOME,profile:process.env.USERPROFILE,h:process.env.OPENCODEX_HOME,node:process.execPath,script:__filename}));process.stderr.write('native-fixture-stderr');process.exit(23);"#);
    let digest = |path: &Path| hash::sha256_file(path).unwrap();
    let manifest = program_selection::ImageManifest {
        schema_version: 1,
        package: OFFICIAL_PACKAGE.into(),
        version: "9.0.0".into(),
        platform: "win32-x64".into(),
        official_archive_sha256: hash::sha256_hex(b"explicit synthetic argv fixture, not official"),
        tree_sha256: program_image::content_digest(&p.version_root).unwrap(),
        bin: "probe.cmd".into(),
        package_sha256: digest(&package.join("package.json")),
        bin_sha256: digest(&package.join("probe.cmd")),
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
                "probe.cmd"
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

#[test]
#[ignore = "requires a freshly created isolated ocxw1c account, exact SID guard, real MSVC product and pinned Windows bundle"]
fn isolated_account_terminal_w1c() {
    let token = current_identity().unwrap();
    let expected = std::env::var("OCXD024_ACCOUNT_SID").expect("explicit test SID");
    assert_eq!(token.sid, expected);
    assert!(!token.elevated);
    assert!(!crate::modules::test_sandbox::enabled());
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1c"));
    assert_eq!(
        profile,
        PathBuf::from(std::env::var_os("USERPROFILE").unwrap())
    );
    if let Some(path) = std::env::var_os("OCXD024_RESUME_CONTEXT") {
        let c: Context = decode(&fs::read(path).unwrap()).unwrap();
        assert_eq!(c.user_profile, profile);
        assert!(c.program_root.starts_with(profile.join("w1c native")));
        let p = inspect_with(&c, &[], &Native).unwrap();
        assert!(p.pending);
        apply_with(&c, p.action, &[], &p.confirmation_digest, &Native, &|_| {
            Ok(())
        })
        .unwrap();
        assert!(!c.pending().exists());
        println!(
            "fresh native process resumed original persisted action: {:?}",
            p.action
        );
        return;
    }
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    let e = Evidence(PathBuf::from(
        std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap(),
    ));
    let r = ObservedNative::new();
    e.save(
        "account-path-before.private.json",
        &serde_json::to_vec(&r.original).unwrap(),
    );
    e.event("identity",json!({"sid":token.sid,"profile":profile,"session":token.session_id,"elevated":token.elevated,"scope":"one isolated account; no W2"}));
    let base = profile.join("w1c native");
    assert!(!base.exists());
    platform::create_private_dir_all(&base, 0o700).unwrap();
    let i = base.join("I 空格 & !");
    platform::create_private_dir_all(&i, 0o700).unwrap();
    copy(&input.join("ocxd.exe"), &i.join("ocxd.exe"));
    copy(
        &input.join("opencodex-desktop.exe"),
        &i.join("opencodex-desktop.exe"),
    );
    let pristine = Command::new(i.join("ocxd.exe"))
        .args(["--uninstall-terminal-cli", "--confirm"])
        .output()
        .unwrap();
    save_output(&e, "helper-pristine", &pristine);
    assert!(pristine.status.success());
    assert!(!profile.join(".codex-desktop").exists());
    r.restored();
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
    let active =
        data_root::startup_v2::selected_config(&profile.join(".codex-desktop/desktop")).unwrap();
    let c = Context {
        user_profile: profile.clone(),
        desktop_root: active.active_data_root.clone(),
        opencodex_home: data_root::resolve_opencodex_home(&active),
        program_root: i.clone(),
        manager_executable: i.join("opencodex-desktop.exe"),
    };
    let no_entry = hook(&c, &input, false);
    save_output(&e, "hook-never-enabled", &no_entry);
    assert!(no_entry.status.success());
    assert!(i.join("hook-passed.txt").exists());
    let fixture = publish_fixture(
        &c,
        &input.join("node.exe"),
        &input.join("native024-fixture.exe"),
    );
    let resume_context = e.0.join("resume-context.private.json");
    fs::write(&resume_context, encode(&c).unwrap()).unwrap();
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(!command(&c).exists() && !c.record().exists() && !c.pending().exists());
    assert!(
        apply_with(&c, Action::Enable, &[], "wrong-confirmation", &r, &|_| Ok(
            ()
        ))
        .is_err()
    );
    assert!(!c.pending().exists());
    r.restored();
    e.event(
        "preview-cancel-and-wrong-confirmation",
        json!({"noCommandPathRecordWrites":true,"digest":p.confirmation_digest}),
    );
    execute(&c, Action::Enable, &r);
    assert_eq!(
        private_read(&i, &command(&c)).unwrap().as_deref(),
        Some(SCRIPT)
    );
    let command_before = fs::read(command(&c)).unwrap();
    let record_before = fs::read(c.record()).unwrap();
    execute(&c, Action::Enable, &r);
    assert_eq!(fs::read(command(&c)).unwrap(), command_before);
    assert_eq!(fs::read(c.record()).unwrap(), record_before);
    e.event(
        "enable-idempotent",
        json!({"script":command(&c),"schema":2,"version":"9.0.0","h":c.opencodex_home}),
    );
    // Shells are real. Payload contains no service/business commands.
    let payload = b"stdin binary \0 Unicode \xe9\x9b\xaa\r\n";

    let batch = base.join("argv.cmd");

    create(&batch,format!("@echo off\r\nchcp 65001 >nul\r\nsetlocal DisableDelayedExpansion\r\n\"{}\" plain \"arg space\" \"雪\" \"&|<>^!()\" \"literal%%text\" \"trailing\\\\\" \"\" \"quoted\\\"inside\"\r\n",command(&c).display()).as_bytes());
    let expected_args = json!([
        "plain",
        "arg space",
        "雪",
        "&|<>^!()",
        "literal%text",
        "trailing\\",
        "",
        "quoted\"inside"
    ]);
    let mut failed_cells = Vec::new();
    for (label,shell,script) in [
        ("cmd",PathBuf::from(std::env::var_os("COMSPEC").unwrap()),format!("\"\"{}\"\"",batch.display())),
        ("powershell",PathBuf::from(std::env::var_os("OCXD024_PWSH").unwrap()),format!("& '{}' @('plain','arg space','雪','&|<>^!()','literal%text','trailing\\','','quoted\"inside'); exit $LASTEXITCODE",command(&c).display().to_string().replace('\'',"''"))),
    ] {
        let mut cmd=Command::new(shell);
        if label=="cmd" {cmd.args(["/d","/s","/c"]).raw_arg(script);} else {cmd.args(["-NoLogo","-NoProfile","-NonInteractive","-Command",&script]);}
        let path=r.read().unwrap().unwrap().text().unwrap();
        cmd.env("PATH",format!("{};{}",std::env::var("SystemRoot").unwrap()+"\\System32",path))
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child=cmd.spawn().unwrap(); child.stdin.take().unwrap().write_all(payload).unwrap();
        let out=child.wait_with_output().unwrap(); save_output(&e,&format!("argv-{label}"),&out);
        let value=serde_json::from_slice::<Value>(&out.stdout);
        let passed=out.status.code()==Some(23) && out.stderr==b"native-fixture-stderr" && value.as_ref().is_ok_and(|v|
            v["home"]==profile.to_str().unwrap() && v["profile"]==profile.to_str().unwrap() && v["h"]==c.opencodex_home.to_str().unwrap()
            && v["stdin"]==base64::Engine::encode(&base64::engine::general_purpose::STANDARD,payload) && v["args"]==expected_args);
        e.event("argv-comparison",json!({"shell":label,"passed":passed,"expectedArgs":expected_args,"observed":value.ok()}));
        if !passed {failed_cells.push(format!("argv-{label}"));}
    }
    execute(&c, Action::Disable, &r);
    r.restored();
    execute(&c, Action::Enable, &r);
    let enabled = r.read().unwrap().unwrap();
    let later = PathValue::encode(
        enabled.value_type,
        &(enabled.text().unwrap() + ";C:\\later-owned-test-entry"),
    )
    .unwrap();
    r.owned_external(Some(later.clone()));
    execute(&c, Action::Disable, &r);
    let retained = r.read().unwrap().unwrap();
    assert!(retained
        .text()
        .unwrap()
        .contains("C:\\later-owned-test-entry"));
    assert!(!retained
        .text()
        .unwrap()
        .contains(command(&c).parent().unwrap().to_str().unwrap()));
    e.event(
        "disable-preserves-later-path",
        json!({"originalType":enabled.value_type,"laterBytesPreserved":true}),
    );
    r.owned_external(r.original.clone());
    r.restored();
    for action in [Action::Enable, Action::Disable] {
        for at in ["pending", "command", "path", "record", "notified"] {
            if action == Action::Disable {
                execute(&c, Action::Enable, &r);
            }
            let p = plan(&c, action, &[], &r).unwrap().1;
            let result = apply_with(&c, action, &[], &p.confirmation_digest, &r, &|point| {
                if point == at {
                    Err(fail("explicit native interruption boundary"))
                } else {
                    Ok(())
                }
            });
            assert!(result.is_err());
            let raw = fs::read(c.pending()).unwrap();
            e.save(&format!("pending-{action:?}-{at}.private.json"), &raw);
            let p = inspect_with(&c, &[], &r).unwrap();
            assert!(p.pending);
            assert_eq!(p.action, action);
            let opposite = if action == Action::Enable {
                Action::Disable
            } else {
                Action::Enable
            };
            assert!(plan(&c, opposite, &[], &r).is_err());
            assert_eq!(fs::read(c.pending()).unwrap(), raw);
            let transaction: Transaction = decode(&raw).unwrap();
            let resumed = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "modules::runtime::terminal_cli::terminal_cli_windows::native_tests::isolated_account_terminal_w1c", "--ignored", "--test-threads=1", "--nocapture"])
                .env("OCXD024_RESUME_CONTEXT", &resume_context).output().unwrap();
            save_output(&e, &format!("resume-{action:?}-{at}"), &resumed);
            assert!(
                resumed.status.success(),
                "{}",
                String::from_utf8_lossy(&resumed.stderr)
            );
            assert_eq!(r.read().unwrap(), transaction.path_after);
            *r.last.borrow_mut() = transaction.path_after;
            assert!(!c.pending().exists());
            e.event("boundary-resumed",json!({"action":format!("{action:?}"),"boundary":at,"samePendingSha256":hash::sha256_hex(&raw)}));
            if action == Action::Enable {
                execute(&c, Action::Disable, &r);
            }
            r.restored();
        }
    }
    // A real registry/file operation precedes these explicit test interruptions.
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
    assert!(c.pending().exists());
    r.fail_notify.set(false);
    execute(&c, Action::Enable, &r);
    execute(&c, Action::Disable, &r);
    r.restored();
    for object in ["command", "record", "pending", "path"] {
        let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
        assert!(apply_with(
            &c,
            Action::Enable,
            &[],
            &p.confirmation_digest,
            &r,
            &|point| if point == "command" {
                Err(fail("explicit interruption before external conflict"))
            } else {
                Ok(())
            }
        )
        .is_err());
        let pending_bytes = fs::read(c.pending()).unwrap();
        let original_path = r.read().unwrap();
        if object == "command" {
            atomic_write(&command(&c), b"foreign file bytes", 0o600).unwrap();
        } else if object == "record" {
            atomic_write(&c.record(), b"foreign record bytes", 0o600).unwrap();
        } else if object == "pending" {
            let mut t: Transaction = decode(&pending_bytes).unwrap();
            t.path_after = Some(PathValue::encode(1, "C:\\external-native-test").unwrap());
            atomic_write(&c.pending(), &encode(&t).unwrap(), 0o600).unwrap();
        } else {
            r.owned_external(Some(
                PathValue::encode(1, "C:\\external-native-test").unwrap(),
            ));
        }
        assert!(plan(&c, Action::Enable, &[], &r).is_err());
        if object != "pending" {
            assert_eq!(fs::read(c.pending()).unwrap(), pending_bytes);
        }
        if object == "command" {
            assert_eq!(fs::read(command(&c)).unwrap(), b"foreign file bytes");
            atomic_write(&command(&c), SCRIPT, 0o600).unwrap();
        } else if object == "record" {
            assert_eq!(fs::read(c.record()).unwrap(), b"foreign record bytes");
            fs::remove_file(c.record()).unwrap();
        } else if object == "pending" {
            let retained: Transaction = decode(&fs::read(c.pending()).unwrap()).unwrap();
            assert_eq!(
                retained.path_after,
                Some(PathValue::encode(1, "C:\\external-native-test").unwrap())
            );
            atomic_write(&c.pending(), &pending_bytes, 0o600).unwrap();
        } else {
            assert_eq!(
                r.read().unwrap(),
                Some(PathValue::encode(1, "C:\\external-native-test").unwrap())
            );
            r.owned_external(original_path);
        }
        execute(&c, Action::Enable, &r);
        execute(&c, Action::Disable, &r);
        r.restored();
        e.event(
            "external-conflict-retained",
            json!({"object":object,"pendingPreserved":true}),
        );
    }
    // Pending enable is never completed by the actual product uninstall helper.
    let p = plan(&c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|point| if point == "pending" {
            Err(fail("explicit pending enable"))
        } else {
            Ok(())
        }
    )
    .is_err());
    let out = helper(&c, &["--uninstall-terminal-cli", "--confirm"]);
    save_output(&e, "helper-pending-enable", &out);
    assert_eq!(out.status.code(), Some(1));
    assert!(c.pending().exists());
    let before_pending = fs::read(c.pending()).unwrap();
    let before_helper = hash::sha256_file(&console(&c)).unwrap();
    let denied = hook(&c, &input, false);
    save_output(&e, "hook-pending-enable-refused", &denied);
    assert_eq!(denied.status.code(), Some(1));
    assert!(!i.join("hook-passed.txt").exists());
    assert_eq!(fs::read(c.pending()).unwrap(), before_pending);
    assert_eq!(hash::sha256_file(&console(&c)).unwrap(), before_helper);
    execute(&c, Action::Enable, &r);
    let expected_after = r.read().unwrap();
    *r.last.borrow_mut() = expected_after;
    let out = helper(&c, &["--uninstall-terminal-cli", "--confirm", "extra"]);
    save_output(&e, "helper-extra-argument", &out);
    assert_eq!(out.status.code(), Some(2));
    assert!(c.record().exists());
    let before_record = fs::read(c.record()).unwrap();
    let before_path = r.read().unwrap();
    let upgrade = hook(&c, &input, true);
    save_output(&e, "hook-upgrade-keeps-terminal", &upgrade);
    assert!(upgrade.status.success());
    assert!(i.join("hook-passed.txt").exists());
    assert_eq!(r.read().unwrap(), before_path);
    assert_eq!(fs::read(c.record()).unwrap(), before_record);
    let out = helper(&c, &["--uninstall-terminal-cli", "--confirm"]);
    save_output(&e, "helper-normal-revoke", &out);
    assert!(out.status.success());
    *r.last.borrow_mut() = r.original.clone();
    r.restored();
    assert!(!command(&c).exists() && !c.record().exists() && !c.pending().exists());
    execute(&c, Action::Enable, &r);
    let normal = hook(&c, &input, false);
    save_output(&e, "hook-normal-revokes-terminal", &normal);
    assert!(normal.status.success());
    assert!(i.join("hook-passed.txt").exists());
    *r.last.borrow_mut() = r.original.clone();
    r.restored();
    assert!(!command(&c).exists() && !c.record().exists() && !c.pending().exists());
    e.event(
        "synthetic-phase-complete",
        json!({"fixtureSelection":fixture,"allOriginalPathBytesRestored":true}),
    );
    let start = Instant::now();
    let descriptor = fs::read(input.join("runtime-bundle.manifest.json")).unwrap();
    let pin = hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a"
    );
    // Delete only this test's current selector/stable entry before invoking the
    // real first-supply path. The synthetic image itself remains for evidence.
    fs::remove_file(i.join("runtime/current.json")).unwrap();
    fs::remove_file(i.join("runtime/bin/ocx-runtime.cmd")).unwrap();
    let provision = bundled::versioned::provision(
        &i,
        &c.desktop_root,
        &input.join("runtime-bundle-0.2.24-win32-x64.tar.gz"),
        &descriptor,
        &pin,
    );
    if let Err(error) = provision {
        e.event("official-product-blocked",json!({"passed":false,"error":error.to_string(),"scope":"unchanged official bin/ocx.mjs rejected by shared executable-extension admission; outside W1-C write set"}));
        r.restored();
        e.save(
            "account-path-after.private.json",
            &serde_json::to_vec(&r.read().unwrap()).unwrap(),
        );
        e.event("component-results",json!({"pathRestored":true,"argvFailedCells":failed_cells,"officialIntegrated":false,"transactionAndHookPhaseCompleted":true}));
        panic!("official product admission blocked; see original event evidence; argv failures: {failed_cells:?}");
    }
    let selected = program_selection::read_complete(
        &i,
        crate::modules::data_root::layout::Platform::Windows,
        "win32-x64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(selected.selection.version, "2.99.0");
    execute(&c, Action::Enable, &r);
    create(&c.opencodex_home.join("config.json"), br#"{"port": 18241}"#);
    for (label, args) in [
        ("official-version", vec!["--version"]),
        (
            "official-custom-home-port",
            vec!["config", "get", "port", "--json"],
        ),
    ] {
        let command_line = format!("\"\"{}\" {}\"", command(&c).display(), args.join(" "));
        let out = Command::new(std::env::var_os("COMSPEC").unwrap())
            .args(["/d", "/s", "/c"])
            .raw_arg(command_line)
            .output()
            .unwrap();
        save_output(&e, label, &out);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        if label == "official-version" {
            assert!(String::from_utf8_lossy(&out.stdout).contains("2.99.0"));
        } else {
            assert!(String::from_utf8_lossy(&out.stdout).contains("18241"));
        }
    }
    let forwarded = super::forwarding_command(&c, vec!["--version".into()]).unwrap();
    assert_eq!(forwarded.get_program(), selected.paths.node.as_os_str());
    assert_eq!(
        forwarded.get_args().next(),
        Some(selected.script.as_os_str())
    );
    execute(&c, Action::Disable, &r);
    r.restored();
    program_selection::verify_current(&i, &selected).unwrap();
    program_image::verify(&selected).unwrap();
    e.save(
        "account-path-after.private.json",
        &serde_json::to_vec(&r.read().unwrap()).unwrap(),
    );
    e.event("complete",json!({"officialVersion":selected.selection.version,"officialSourceSha256":selected.image.official_archive_sha256,"h":c.opencodex_home,"node":selected.paths.node,"script":selected.script,"officialPhaseMs":start.elapsed().as_millis(),"pathRestored":true,"noServiceStarted":true,"argvFailedCells":failed_cells}));
    assert!(
        failed_cells.is_empty(),
        "native argv failures retained in evidence: {failed_cells:?}"
    );
}
