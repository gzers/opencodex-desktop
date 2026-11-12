//! Explicit unmodified official-runtime tests only. Synthetic root movement is
//! not a production root permit or evidence of normal service Stop.
use super::*;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
fn config_probe(
    context: &Context,
    selected: &program_selection::SelectedProgram,
    target: &Path,
    blocked: bool,
) {
    let input = serde_json::json!({"package":selected.paths.package_root.join(PACKAGE_SUBPATH),"target":target});
    let script = r#"import {pathToFileURL} from 'node:url';import {join} from 'node:path';const p=JSON.parse(process.argv.at(-1));const {withConfigWriteLock}=await import(pathToFileURL(join(p.package,'src/codex/config-write-lock.ts')).href);const r=withConfigWriteLock(join(p.target,'config.toml'),()=>true);console.log(JSON.stringify({ok:r.ok,error:r.ok?null:r.error}));"#;
    let output = Command::new(&selected.paths.bun)
        .args(["--eval", script, "--", &input.to_string()])
        .env_clear()
        .env("HOME", &context.user_profile)
        .env("CODEX_HOME", &context.codex_home)
        .env("OPENCODEX_HOME", &context.opencodex_home)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .current_dir(&context.user_profile)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual,
        if blocked {
            serde_json::json!({"ok":false,"error":"locked"})
        } else {
            serde_json::json!({"ok":true,"error":null})
        }
    );
}
fn gone(pid: u32, deadline: Instant) {
    loop {
        if unsafe { libc::kill(pid as i32, 0) } != 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "holder still alive after parent EOF"
        );
        thread::sleep(Duration::from_millis(3));
    }
}
fn eof_normal(mut guard: Guard<'_>) {
    guard.child.stdin.take();
    loop {
        assert!(
            Instant::now() < guard.native.deadline,
            "EOF holder did not exit within original deadline"
        );
        if let Some(status) = guard.child.try_wait().unwrap() {
            assert_eq!(
                status.code(),
                Some(1),
                "EOF cleanup is refusal, never success or forced signal"
            );
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    if let Some(reader) = guard.reader.take() {
        reader.join().unwrap();
    }
    guard.released = true;
}
pub(crate) fn official_round(
    context: &Context,
    selected: &program_selection::SelectedProgram,
    root: &Path,
) {
    let config_before = fs::read(context.codex_home.join("config.toml")).unwrap();
    let mut rows = Vec::new();
    // Each source has restored root/config and its own immutable destination.
    for scene in [
        "normal",
        "eof-prepared",
        "eof-moved",
        "eof-reanchored",
        "replacement-lock",
        "new-user-root",
        "wrong-command",
    ] {
        let archive = context.user_profile.join(format!("retired-{scene}"));
        let started = Instant::now();
        let mut holder_pid = 0;
        let mut moved = false;
        let result = with_native_relocation_coordination(
            context,
            started + Duration::from_secs(10),
            |native| {
                let mut guard = native.config_write_guard()?;
                holder_pid = guard.child.id();
                config_probe(context, selected, &context.codex_home, true);
                if scene == "normal" {
                    thread::sleep(Duration::from_millis(2100));
                    guard.check()?;
                    config_probe(context, selected, &context.codex_home, true);
                }
                if scene == "wrong-command" {
                    guard.poisoned = true;
                    guard.command("prepare", Some(&context.codex_home))?;
                    assert!(guard.event("prepared", &context.codex_home).is_err());
                    assert!(guard.check().is_err());
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                    config_probe(context, selected, &context.codex_home, false);
                    return Ok(());
                }
                guard.prepare_relocation(&archive)?;
                assert!(guard.prepare_relocation(&archive).is_err());
                if scene == "eof-prepared" {
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                    config_probe(context, selected, &context.codex_home, false);
                    return Ok(());
                }
                native.begin_claim_handoff()?;
                fs::rename(&context.codex_home, &archive).map_err(guard_error)?;
                moved = true;
                native.reanchor_claim(&archive)?;
                if scene == "eof-moved" {
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                    config_probe(context, selected, &archive, false);
                    return Ok(());
                }
                if scene == "new-user-root" {
                    fs::DirBuilder::new()
                        .mode(0o700)
                        .create(&context.codex_home)
                        .unwrap();
                    fs::write(context.codex_home.join("user-data"), b"external user").unwrap();
                    // EOF cleanup may release ONLY the exact original at the
                    // prepared archive; the replacement logical root stays intact.
                    assert!(native.check().is_err());
                    assert!(guard.reanchor(&archive).is_err());
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                    assert!(!archive.join("config.toml.ocx-write.lock").exists());
                    assert_eq!(
                        fs::read(context.codex_home.join("user-data")).unwrap(),
                        b"external user"
                    );
                    assert!(!context
                        .codex_home
                        .join("config.toml.ocx-write.lock")
                        .exists());
                    return Err(guard_error(()));
                }
                if scene == "replacement-lock" {
                    let lock = archive.join("config.toml.ocx-write.lock");
                    let retained = archive.join("retained-original-config-lock");
                    fs::rename(&lock, &retained).unwrap();
                    // Same bytes/token must NOT authorize a replacement inode.
                    fs::copy(&retained, &lock).unwrap();
                    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
                    assert!(guard.reanchor(&archive).is_err());
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                    assert!(lock.is_file() && retained.is_file());
                    rows.push(serde_json::json!({"scene":"replacement-preservation","bothUnknownAndOriginalRetained":true,"sameTokenCannotReplaceIdentity":true}));
                    // Test-only restoration of the exact held file before making
                    // official acquisition positive controls; never product repair.
                    fs::remove_file(&lock).unwrap();
                    fs::rename(&retained, &lock).unwrap();
                    return Ok(());
                }
                guard.reanchor(&archive)?;
                guard.check()?;
                config_probe(context, selected, &archive, true);
                if scene == "eof-reanchored" {
                    eof_normal(guard);
                    gone(holder_pid, started + Duration::from_secs(10));
                } else {
                    guard.finish()?;
                }
                config_probe(context, selected, &archive, false);
                native.check()?;
                Ok(())
            },
        );
        fs::write(root.join("native-summary.json"),serde_json::to_vec_pretty(&serde_json::json!({"accepted":false,"scope":"incremental official holder diagnosis","completedRows":rows,"currentScene":scene,"currentResult":if result.is_ok(){"ok"}else{"refused"},"currentElapsedMs":started.elapsed().as_millis()})).unwrap()).unwrap();
        if scene == "new-user-root" {
            assert!(result.is_err());
        } else {
            result.unwrap();
        }
        if moved {
            assert_eq!(
                fs::read(archive.join("config.toml")).unwrap(),
                config_before
            );
            if scene == "new-user-root" {
                // Preserve external fixture bytes in evidence before removing
                // only the two paths created by this test, after all locks exit.
                fs::write(
                    root.join("preserved-new-user-data"),
                    fs::read(context.codex_home.join("user-data")).unwrap(),
                )
                .unwrap();
                fs::remove_file(context.codex_home.join("user-data")).unwrap();
                fs::remove_dir(&context.codex_home).unwrap();
            }
            fs::rename(&archive, &context.codex_home).unwrap();
            if scene == "replacement-lock" {
                // Unknown/retained lock was deliberately not deleted by helper.
                // Record its exact original bytes privately; owned fixture only.
                let lock = context.codex_home.join("config.toml.ocx-write.lock");
                fs::copy(&lock, root.join("preserved-original-lock.json")).unwrap();
                fs::remove_file(lock).unwrap();
            }
        }
        assert_eq!(
            fs::read(context.codex_home.join("config.toml")).unwrap(),
            config_before
        );
        assert!(!context
            .codex_home
            .join("config.toml.ocx-write.lock")
            .exists());
        assert!(started.elapsed() < Duration::from_secs(10));
        rows.push(serde_json::json!({"scene":scene,"holderPid":holder_pid,"elapsedMs":started.elapsed().as_millis(),"result":"passed","configBytesUnchanged":true,"noServiceOrProductionRootPermission":true}));
    }
    // Actual parent exit, not Drop: helper inherits pipe EOF and must release
    // the pre-registered original destination even before relocate is acked.
    let output=Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::official_client_coordination::native::config::relocation_tests::official_holder_parent_exit_after_move","--ignored","--nocapture","--test-threads=1"])
        .env("OCXD_NATIVE_WRITER_EXIT_ROOT",root).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(91),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let pid: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("writer-exit-pid.json")).unwrap()).unwrap();
    gone(
        pid["pid"].as_u64().unwrap() as u32,
        Instant::now() + Duration::from_secs(5),
    );
    let archive = context.user_profile.join("retired-parent-exit");
    assert!(!archive.join("config.toml.ocx-write.lock").exists());
    config_probe(context, selected, &archive, false);
    fs::rename(&archive, &context.codex_home).unwrap();
    rows.push(serde_json::json!({"scene":"actual-parent-exit91","actualWaitExitCode":91,"holderGoneByESRCH":true,"lockReleasedAtPreparedDestination":true,"noCleanupSignal":true}));
    let report = serde_json::json!({"accepted":false,"scope":"Unmodified official299 config writer holder relocation with synthetic roots; no service or production root movement permission","rows":rows,"result":"passed"});
    fs::write(
        root.join("native-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("WRITER_RELOCATION_EVIDENCE {report}");
}
#[test]
#[ignore = "requires owned pinned official299 runtime; spawned by the explicit relocation round only"]
fn official_holder_parent_exit_after_move() {
    let root = PathBuf::from(std::env::var_os("OCXD_NATIVE_WRITER_EXIT_ROOT").expect("owned root"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ocxd024-native-coordination-")
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let context = Context {
        runtime: crate::modules::runtime::RuntimeHandle::initialize_versioned(
            &profile.join("desktop"),
            &root.join("program"),
        ),
        user_profile: profile.clone(),
        codex_home: profile.join("codex"),
        opencodex_home: profile.join("home"),
        control_root: root.join("control"),
    };
    with_native_relocation_coordination(
        &context,
        Instant::now() + Duration::from_secs(10),
        |native| -> AppResult<()> {
            let mut guard = native.config_write_guard()?;
            guard.prepare_relocation(&profile.join("retired-parent-exit"))?;
            native.begin_claim_handoff()?;
            fs::rename(&context.codex_home, profile.join("retired-parent-exit"))
                .map_err(guard_error)?;
            fs::write(
                root.join("writer-exit-pid.json"),
                serde_json::to_vec(&serde_json::json!({"pid":guard.child.id()})).unwrap(),
            )
            .unwrap();
            std::process::exit(91);
        },
    )
    .unwrap();
    panic!("exit boundary not reached");
}
