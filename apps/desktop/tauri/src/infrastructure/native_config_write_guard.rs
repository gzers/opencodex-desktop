//! The unchanged official file lock remains held by a dedicated, pipe-bound
//! child for this borrow. No timer unlocks a live callback. Native SQLite
//! authority stays in the mutation process. Child death/supersession refuses;
//! parent death closes the pipe, so the official finally releases its own lock.
use super::*;
use crate::infrastructure::macos_client_originals::Image;
use std::{io::Write, sync::mpsc, thread};
const HELPER: &str = include_str!("client_config_guard299.mjs");
const SCHEMA: &str = "ocxd-config-write-guard/2";
#[cfg(test)]
#[path = "native_config_guard_relocation_tests.rs"]
pub(super) mod relocation_tests;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    schema: String,
    kind: String,
    operation: String,
    pid: u32,
    #[serde(rename = "targetSha256")]
    target_sha256: String,
    phase: String,
}
pub struct Guard<'a> {
    native: &'a NativeLease,
    target: PinnedDirectory,
    child: std::process::Child,
    rx: mpsc::Receiver<AppResult<Event>>,
    reader: Option<thread::JoinHandle<()>>,
    operation: String,
    image: Option<Image>,
    released: bool,
    planned: Option<PathBuf>,
    poisoned: bool,
}
fn guard_error<T>(_: T) -> AppError {
    fail("client_config_write_guard_unconfirmed")
}
impl NativeLease {
    pub fn config_write_guard(&self) -> AppResult<Guard<'_>> {
        self.write_guard(false)
    }
    pub fn grok_write_guard(&self) -> AppResult<Guard<'_>> {
        self.write_guard(true)
    }
    fn write_guard(&self, grok: bool) -> AppResult<Guard<'_>> {
        self.check()?;
        let target_path = if grok {
            self.profile.path().join(".grok")
        } else {
            self.client.path().to_path_buf()
        };
        let target_root = PinnedDirectory::open(&target_path).map_err(guard_error)?;
        if fs::canonicalize(&target_path).map_err(guard_error)? != target_path {
            return Err(guard_error(()));
        }
        let root = self
            .runtime
            .managed_installation_root()
            .ok_or_else(|| guard_error(()))?;
        let (target, platform) = program_layout::host_target()?;
        let selected = program_selection::read_complete(&root, platform, target)?
            .ok_or_else(|| guard_error(()))?;
        if selected.selection.version != "2.99.0"
            || self.runtime.executable().as_ref() != Some(&selected.stable_entry)
        {
            return Err(guard_error(()));
        }
        let operation = uuid::Uuid::new_v4().to_string();
        let input = serde_json::json!({"schema":SCHEMA,"operation":operation,"packageRoot":selected.paths.package_root.join(PACKAGE_SUBPATH),"profile":self.profile.path(),"home":self.home.path(),"codex":self.client.path(),"target":target_path});
        let mut child = Command::new(&selected.paths.bun)
            .args(["--eval", HELPER, "--", &input.to_string()])
            .env_clear()
            .env("HOME", self.profile.path())
            .env("CODEX_HOME", self.client.path())
            .env("OPENCODEX_HOME", self.home.path())
            .env(
                "PATH",
                crate::infrastructure::platform::controlled_path(
                    [selected
                        .paths
                        .node
                        .parent()
                        .ok_or_else(|| guard_error(()))?
                        .to_path_buf()],
                    None,
                ),
            )
            .env("LANG", "C.UTF-8")
            .env("LC_ALL", "C.UTF-8")
            .current_dir(self.client.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(guard_error)?;
        let stdout = child.stdout.take().ok_or_else(|| guard_error(()))?;
        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stream = BufReader::new(stdout);
            for _ in 0..4 {
                let mut line = Vec::new();
                let result = (&mut stream).take(513).read_until(b'\n', &mut line);
                if !matches!(result,Ok(n) if n>0&&n<=512&&line.ends_with(b"\n")) {
                    let _ = tx.send(Err(guard_error(())));
                    return;
                }
                let value = crate::modules::model_config::schema_v2::parse_strict_json_v2(&line)
                    .map_err(guard_error)
                    .and_then(|v| serde_json::from_value(v).map_err(guard_error));
                if tx.send(value).is_err() {
                    return;
                }
            }
        });
        let mut guard = Guard {
            native: self,
            target: target_root,
            child,
            rx,
            reader: Some(reader),
            operation,
            image: None,
            released: false,
            planned: None,
            poisoned: false,
        };
        guard.event("held", &target_path)?;
        let image = guard
            .target
            .config_write_lock_image()
            .map_err(guard_error)?;
        let Image::File { bytes, .. } = &image else {
            return Err(guard_error(()));
        };
        let v = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes)
            .map_err(guard_error)?;
        if v["pid"].as_u64() != Some(guard.child.id().into())
            || !v["token"]
                .as_str()
                .is_some_and(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(guard_error(()));
        }
        guard.image = Some(image);
        guard.check()?;
        Ok(guard)
    }
}
impl Guard<'_> {
    fn event(&self, wanted: &str, target: &Path) -> AppResult<()> {
        let remaining = self
            .native
            .deadline
            .saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(guard_error(()));
        }
        let e = self.rx.recv_timeout(remaining).map_err(guard_error)??;
        if ![
            "input",
            "root-pin",
            "acquire",
            "initial-check",
            "command",
            "prepare",
            "relocate",
            "release",
        ]
        .contains(&e.phase.as_str())
        {
            return Err(guard_error(()));
        }
        #[cfg(test)]
        eprintln!(
            "CONFIG_GUARD_TRACE expected={} actual={} phase={}",
            wanted, e.kind, e.phase
        );
        if e.schema != SCHEMA
            || e.kind != wanted
            || e.pid != self.child.id()
            || e.operation != self.operation
            || e.target_sha256
                != crate::infrastructure::hash::sha256_hex(target.as_os_str().as_encoded_bytes())
        {
            return Err(guard_error(()));
        }
        Ok(())
    }
    fn command(&mut self, kind: &str, target: Option<&Path>) -> AppResult<()> {
        let mut value = serde_json::json!({"operation":self.operation,"kind":kind});
        if let Some(target) = target {
            value["target"] = serde_json::to_value(target).map_err(guard_error)?;
        }
        let mut bytes = serde_json::to_vec(&value).map_err(guard_error)?;
        bytes.push(b'\n');
        if bytes.len() > 16384 {
            return Err(guard_error(()));
        }
        self.child
            .stdin
            .as_mut()
            .ok_or_else(|| guard_error(()))?
            .write_all(&bytes)
            .map_err(guard_error)
    }
    /// Register the single exact absent destination before moving anything.
    /// This is holder continuity, not root movement/lifecycle authorization.
    pub fn prepare_relocation(&mut self, target: &Path) -> AppResult<()> {
        self.check()?;
        if self.planned.is_some()
            || self.target.path() != self.native.client.path()
            || self.native.claim_gate.borrow().is_none()
        {
            return Err(guard_error(()));
        }
        let parent = target.parent().ok_or_else(|| guard_error(()))?;
        if !target.is_absolute()
            || fs::canonicalize(parent).map_err(guard_error)? != parent
            || !matches!(fs::symlink_metadata(target),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
        {
            return Err(guard_error(()));
        }
        self.poisoned = true;
        self.command("prepare", Some(target))?;
        // Retain this target even when the acknowledgement fails: helper EOF
        // cleanup may already know it, but no mutation permission is returned.
        self.planned = Some(target.to_path_buf());
        self.event("prepared", target)?;
        self.poisoned = false;
        let result = self.check();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub fn reanchor(&mut self, target: &Path) -> AppResult<()> {
        self.native.check()?;
        if self.poisoned
            || self.planned.as_deref() != Some(target)
            || self.native.claim_mode.get() != ClaimMode::Ofd
            || self.child.try_wait().map_err(guard_error)?.is_some()
        {
            return Err(guard_error(()));
        }
        let parent = PinnedDirectory::open(target).map_err(guard_error)?;
        if parent.identity() != self.target.identity()
            || self.image.as_ref() != Some(&parent.config_write_lock_image().map_err(guard_error)?)
            || self
                .native
                .physical_client
                .borrow()
                .as_ref()
                .is_none_or(|p| p.path() != target || p.identity() != parent.identity())
        {
            return Err(guard_error(()));
        }
        self.poisoned = true;
        self.command("relocate", Some(target))?;
        self.event("relocated", target)?;
        self.target = parent;
        self.poisoned = false;
        let result = self.check();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub fn check(&mut self) -> AppResult<()> {
        if self.poisoned {
            return Err(guard_error(()));
        }
        self.native.check()?;
        self.target.recheck().map_err(guard_error)?;
        if self.child.try_wait().map_err(guard_error)?.is_some()
            || self.image.as_ref()
                != Some(&self.target.config_write_lock_image().map_err(guard_error)?)
        {
            return Err(guard_error(()));
        }
        Ok(())
    }
    pub fn finish(mut self) -> AppResult<()> {
        self.check()?;
        self.command("release", None)?;
        self.child.stdin.take();
        self.event("released", self.target.path())?;
        loop {
            if Instant::now() >= self.native.deadline {
                return Err(guard_error(()));
            }
            if let Some(status) = self.child.try_wait().map_err(guard_error)? {
                if !status.success() {
                    return Err(guard_error(()));
                }
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        if self.target.config_write_lock_image().map_err(guard_error)? != Image::Absent {
            return Err(guard_error(()));
        }
        self.native.check()?;
        if let Some(reader) = self.reader.take() {
            reader.join().map_err(guard_error)?;
        }
        self.released = true;
        Ok(())
    }
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if !self.released {
            // EOF is a cleanup request, never successful mutation evidence.
            self.child.stdin.take();
            let until = Instant::now() + Duration::from_millis(100);
            loop {
                match self.child.try_wait() {
                    Ok(Some(_)) => break,
                    _ if Instant::now() < until => thread::sleep(Duration::from_millis(2)),
                    _ => {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        break;
                    }
                }
            }
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "unchanged official config writer exclusion on explicitly retained owned native Profile; no service or business calls"]
    fn official_config_guard_retains_live_lock_past_old_helper_timer_and_releases_normally() {
        let root =
            PathBuf::from(std::env::var_os("OCXD024_CONFIG_GUARD_ROOT").expect("owned fixture"));
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
                    .starts_with("ocxd024-versioned-lifecycle-")
        );
        let profile = root.join("profile");
        assert_eq!(
            std::env::var_os("HOME"),
            Some(profile.clone().into_os_string())
        );
        let data = profile.join(".codex-desktop/desktop");
        let program = root.join("program 中文 space");
        let client = profile.join(".codex");
        let home = profile.join(".codex-desktop/opencodex/home");
        let control = root.join("guard-control");
        crate::infrastructure::platform::create_private_dir_all(&control, 0o700).unwrap();
        let context = Context {
            runtime: crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &program),
            user_profile: profile.clone(),
            codex_home: client.clone(),
            opencodex_home: home.clone(),
            control_root: control,
        };
        let selected = program_selection::read_complete(
            &program,
            crate::modules::data_root::layout::Platform::Macos,
            "darwin-arm64",
        )
        .unwrap()
        .unwrap();
        let package = selected.paths.package_root.join(PACKAGE_SUBPATH);
        let input = serde_json::json!({"package":package,"client":client});
        let probe = || {
            let script = r#"import {pathToFileURL} from 'node:url';import {join} from 'node:path';const p=JSON.parse(process.argv.at(-1));const {withConfigWriteLock}=await import(pathToFileURL(join(p.package,'src/codex/config-write-lock.ts')).href);const v=withConfigWriteLock(join(p.client,'config.toml'),()=>true);console.log(JSON.stringify({ok:v.ok,error:v.ok?null:v.error}));"#;
            let result = Command::new(&selected.paths.bun)
                .args(["--eval", script, "--", &input.to_string()])
                .env_clear()
                .env("HOME", &profile)
                .env("CODEX_HOME", &client)
                .env("OPENCODEX_HOME", &home)
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .env("LANG", "C.UTF-8")
                .output()
                .unwrap();
            assert!(result.status.success(), "official writer probe failed");
            serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap()
        };
        let clock = Instant::now();
        with_native_coordination(&context, clock + Duration::from_secs(10), |native| {
            let mut guard = native.config_write_guard()?;
            assert_eq!(probe(), serde_json::json!({"ok":false,"error":"locked"}));
            assert!(
                native.config_write_guard().is_err(),
                "a second holder must fail, never join the first"
            );
            thread::sleep(Duration::from_millis(2100));
            guard.check()?;
            assert_eq!(probe(), serde_json::json!({"ok":false,"error":"locked"}));
            guard.finish()?;
            assert_eq!(probe(), serde_json::json!({"ok":true,"error":null}));
            Ok(())
        })
        .unwrap();
        assert!(!client.join("config.toml.ocx-write.lock").exists());
        assert!(!client.join("config.toml").exists());
        let report = serde_json::json!({"accepted":false,"elapsedMs":clock.elapsed().as_millis(),"officialWriterRefusedWhileHeld":true,"nestedHolderRefused":true,"past2100msStillHeld":true,"officialWriterAllowedAfterNormalRelease":true,"noServiceActions":true,"configStillAbsent":true});
        fs::write(
            root.join("config-guard-proof.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
}

#[cfg(test)]
mod parent_exit_tests {
    use super::*;
    #[test]
    #[ignore = "actual parent exit91 and pipe EOF on owned official config-lock fixture; no service action or signals"]
    fn official_config_guard_parent_death_releases_without_forced_helper_stop() {
        let root =
            PathBuf::from(std::env::var_os("OCXD024_CONFIG_GUARD_ROOT").expect("owned fixture"));
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
                    .starts_with("ocxd024-versioned-lifecycle-")
        );
        let profile = root.join("profile");
        assert_eq!(
            std::env::var_os("HOME"),
            Some(profile.clone().into_os_string())
        );
        if std::env::var_os("OCXD024_CONFIG_GUARD_EXIT_CHILD").is_some() {
            let data = profile.join(".codex-desktop/desktop");
            let control = root.join("guard-parent-exit");
            crate::infrastructure::platform::create_private_dir_all(&control, 0o700).unwrap();
            let context = Context {
                runtime: crate::modules::runtime::RuntimeHandle::initialize_versioned(
                    &data,
                    &root.join("program 中文 space"),
                ),
                user_profile: profile.clone(),
                codex_home: profile.join(".codex"),
                opencodex_home: profile.join(".codex-desktop/opencodex/home"),
                control_root: control,
            };
            with_native_coordination(
                &context,
                Instant::now() + Duration::from_secs(10),
                |native| -> AppResult<()> {
                    let mut guard = native.config_write_guard()?;
                    guard.check()?;
                    fs::write(
                        root.join("guard-child-pid.json"),
                        serde_json::to_vec(&serde_json::json!({"pid":guard.child.id()})).unwrap(),
                    )
                    .unwrap();
                    std::process::exit(91);
                },
            )
            .unwrap();
            panic!("exit not reached");
        }
        let started = Instant::now();
        let result=Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::official_client_coordination::native::config::parent_exit_tests::official_config_guard_parent_death_releases_without_forced_helper_stop","--ignored","--nocapture","--test-threads=1"]).env("OCXD024_CONFIG_GUARD_EXIT_CHILD","1").output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("guard-child-pid.json")).unwrap()).unwrap();
        let pid = proof["pid"].as_u64().unwrap() as i32;
        assert!(pid > 0 && pid != std::process::id() as i32);
        let deadline = started + Duration::from_secs(10);
        loop {
            let gone = unsafe { libc::kill(pid, 0) } != 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
            if gone && !profile.join(".codex/config.toml.ocx-write.lock").exists() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "parent EOF failed to release the official holder"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!profile.join(".codex/config.toml").exists());
        let report = serde_json::json!({"accepted":false,"elapsedMs":started.elapsed().as_millis(),"parentExitCode":91,"helperPid":pid,"helperGoneByESRCH":true,"officialLockAbsentAfterPipeEOF":true,"noForcedHelperStop":true,"noServiceActions":true});
        fs::write(
            root.join("config-guard-parent-exit-proof.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
}

#[cfg(test)]
mod grok_tests {
    use super::*;
    #[test]
    #[ignore = "two actual official configuration writer paths on retained owned Profile; no service actions"]
    fn official_grok_and_codex_writer_guards_are_distinct_and_exclude_their_own_paths() {
        let root =
            PathBuf::from(std::env::var_os("OCXD024_GROK_GUARD_ROOT").expect("owned fixture"));
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
                    .starts_with("ocxd024-versioned-lifecycle-")
        );
        let profile = root.join("profile");
        assert_eq!(
            std::env::var_os("HOME"),
            Some(profile.clone().into_os_string())
        );
        let data = profile.join(".codex-desktop/desktop");
        let program = root.join("program 中文 space");
        let client = profile.join(".codex");
        let home = profile.join(".codex-desktop/opencodex/home");
        let grok = profile.join(".grok");
        assert!(grok.is_dir());
        let control = root.join("grok-guard-control");
        crate::infrastructure::platform::create_private_dir_all(&control, 0o700).unwrap();
        let context = Context {
            runtime: crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &program),
            user_profile: profile.clone(),
            codex_home: client.clone(),
            opencodex_home: home.clone(),
            control_root: control,
        };
        let selected = program_selection::read_complete(
            &program,
            crate::modules::data_root::layout::Platform::Macos,
            "darwin-arm64",
        )
        .unwrap()
        .unwrap();
        let probe = |path: &Path| {
            let input = serde_json::json!({"package":selected.paths.package_root.join(PACKAGE_SUBPATH),"path":path});
            let script = r#"import {pathToFileURL} from 'node:url';import {join} from 'node:path';const p=JSON.parse(process.argv.at(-1));const {withConfigWriteLock}=await import(pathToFileURL(join(p.package,'src/codex/config-write-lock.ts')).href);const r=withConfigWriteLock(join(p.path,'config.toml'),()=>true);console.log(JSON.stringify({ok:r.ok,error:r.ok?null:r.error}));"#;
            let result = Command::new(&selected.paths.bun)
                .args(["--eval", script, "--", &input.to_string()])
                .env_clear()
                .env("HOME", &profile)
                .env("CODEX_HOME", &client)
                .env("OPENCODEX_HOME", &home)
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .env("LANG", "C.UTF-8")
                .output()
                .unwrap();
            assert!(result.status.success());
            serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap()
        };
        let locked = serde_json::json!({"ok":false,"error":"locked"});
        let unlocked = serde_json::json!({"ok":true,"error":null});
        let clock = Instant::now();
        with_native_coordination(&context, clock + Duration::from_secs(10), |native| {
            let mut g = native.grok_write_guard()?;
            assert_eq!(probe(&grok), locked);
            assert_eq!(probe(&client), unlocked);
            let mut c = native.config_write_guard()?;
            g.check()?;
            c.check()?;
            assert_eq!(probe(&grok), locked);
            assert_eq!(probe(&client), locked);
            g.finish()?;
            assert_eq!(probe(&grok), unlocked);
            assert_eq!(probe(&client), locked);
            c.finish()?;
            assert_eq!(probe(&client), unlocked);
            Ok(())
        })
        .unwrap();
        for p in [&client, &grok] {
            assert!(!p.join("config.toml.ocx-write.lock").exists());
            assert!(!p.join("config.toml.ocx-write.lock.claims").exists());
        }
        let proof = serde_json::json!({"accepted":false,"elapsedMs":clock.elapsed().as_millis(),"GrokWriterBlockedOnlyByGrokGuard":true,"CodexWriterBlockedOnlyByCodexGuard":true,"bothIndependentGuardsCanCoexist":true,"eachOfficialWriterAllowedOnlyAfterItsGuardNormalRelease":true,"allLockPathsAndClaimsAbsent":true,"noServiceActions":true});
        fs::write(
            root.join("grok-guard-proof.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
        println!("{proof}");
    }
}
