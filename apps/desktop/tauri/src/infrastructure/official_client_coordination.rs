//! Bounded, scoped official 2.99 claim -> N -> K coordination. This does not
//! grant native service ownership or authorize client file compensation.
//! No official module is patched, copied or substituted with another lock.
use super::{macos_client_originals::PinnedDirectory, runtime_executable::SharedRuntimeExecutable};
use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{
        install::PACKAGE_SUBPATH, program_image, program_layout, program_selection,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fs::{self},
    io::{BufRead, BufReader, Read, Write},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
const HELPER: &str = include_str!("client_coordination299.mjs");
const SCHEMA: &str = "ocxd-client-coordination/1";
const MAX_HOLD: Duration = Duration::from_millis(2000);
const MAX_RECORD: usize = 2048;
fn fail(code: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: "官方客户端协调未确认，原件及当前文件保留；未执行未获准的恢复".into(),
    }
}
fn io<T>(_: T) -> AppError {
    fail("client_coordination_unconfirmed")
}

pub struct Context {
    pub runtime: SharedRuntimeExecutable,
    pub user_profile: PathBuf,
    pub codex_home: PathBuf,
    pub opencodex_home: PathBuf,
    /// Existing dedicated private transaction directory, never a program root.
    pub control_root: PathBuf,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Input<'a> {
    schema: &'static str,
    operation: &'a str,
    package_root: &'a Path,
    profile: &'a Path,
    codex_home: &'a Path,
    opencodex_home: &'a Path,
    release_path: &'a Path,
    hold_ms: u64,
}
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct LockIdentity {
    device: String,
    inode: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema: String,
    kind: String,
    operation: String,
    pid: u32,
    context_sha256: String,
    locks: [LockIdentity; 3],
}
fn record(bytes: &[u8]) -> AppResult<Record> {
    let value = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes).map_err(io)?;
    if value["kind"] == "failed" {
        let fields = ["schema", "kind", "stage", "acquired", "released"];
        if value["schema"] != SCHEMA
            || value
                .as_object()
                .is_none_or(|m| m.len() != 5 || m.keys().any(|k| !fields.contains(&k.as_str())))
            || !value["acquired"].is_boolean()
            || !value["released"].is_boolean()
        {
            return Err(fail("client_coordination_protocol"));
        }
        let code = match value["stage"].as_str() {
            Some("input") => "client_coordination_input",
            Some("imports") => "client_coordination_imports",
            Some("context") => "client_coordination_context",
            Some("claim") => "client_coordination_claim",
            Some("native") => "client_coordination_native",
            Some("catalog") => "client_coordination_catalog",
            Some("held") => "client_coordination_held",
            Some("released") => "client_coordination_release",
            _ => return Err(fail("client_coordination_protocol")),
        };
        return Err(fail(code));
    }
    let r: Record = serde_json::from_value(value).map_err(io)?;
    if r.schema != SCHEMA
        || !["held", "released"].contains(&r.kind.as_str())
        || r.locks.iter().any(|x| {
            [&x.device, &x.inode]
                .iter()
                .any(|s| s.is_empty() || s.len() > 32 || !s.bytes().all(|b| b.is_ascii_digit()))
        })
    {
        return Err(fail("client_coordination_protocol"));
    }
    Ok(r)
}
fn spawn_reader(
    stdout: std::process::ChildStdout,
) -> (mpsc::Receiver<AppResult<Record>>, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut count = 0;
        loop {
            let mut line = Vec::new();
            let n = match (&mut reader)
                .take(MAX_RECORD as u64 + 1)
                .read_until(b'\n', &mut line)
            {
                Ok(n) => n,
                Err(e) => {
                    let _ = tx.send(Err(io(e)));
                    return;
                }
            };
            if n == 0 {
                return;
            }
            count += 1;
            if n > MAX_RECORD || count > 2 || !line.ends_with(b"\n") {
                let _ = tx.send(Err(fail("client_coordination_protocol")));
                return;
            }
            let parsed = record(&line);
            let failed = parsed.is_err();
            if tx.send(parsed).is_err() || failed {
                return;
            }
        }
    });
    (rx, thread)
}

/// A borrow valid only inside with_coordination. check() proves this helper is
/// still live before its bounded lease expires; it is NOT a service/restore
/// permit. The caller must retain its separate native writer/exit proof.
pub struct Lease {
    child: RefCell<Child>,
    expires: Instant,
    deadline: Instant,
    operation: String,
    context_sha256: String,
    held: Option<Record>,
    control: PinnedDirectory,
    release: PathBuf,
    roots: Vec<PinnedDirectory>,
    rx: mpsc::Receiver<AppResult<Record>>,
    reader: Option<thread::JoinHandle<()>>,
    released: bool,
}
impl Lease {
    pub fn check(&self) -> AppResult<()> {
        if self.held.is_none()
            || self.released
            || Instant::now() >= self.expires
            || Instant::now() >= self.deadline
            || self.child.borrow_mut().try_wait().map_err(io)?.is_some()
        {
            return Err(fail("client_coordination_expired"));
        }
        self.control.private().map_err(io)?;
        for root in &self.roots {
            root.recheck().map_err(io)?;
        }
        Ok(())
    }
    fn receive(&self) -> AppResult<Record> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(fail("client_coordination_timeout"));
        }
        let r = self
            .rx
            .recv_timeout(remaining)
            .map_err(|_| fail("client_coordination_timeout"))??;
        if r.operation != self.operation
            || r.pid != self.child.borrow().id()
            || r.context_sha256 != self.context_sha256
        {
            return Err(fail("client_coordination_identity"));
        }
        Ok(r)
    }
    fn request_release(&self) -> AppResult<()> {
        self.control.private().map_err(io)?;
        let mut f = super::platform::create_new_private_file(&self.release, 0o600).map_err(io)?;
        writeln!(f, "{}", self.operation).map_err(io)?;
        f.sync_all().map_err(io)?;
        Ok(())
    }
    fn finish(&mut self) -> AppResult<()> {
        self.check()?;
        self.request_release()?;
        let end = self.receive()?;
        if end.kind != "released"
            || self
                .held
                .as_ref()
                .is_none_or(|start| start.locks != end.locks)
        {
            return Err(fail("client_coordination_release"));
        }
        loop {
            if Instant::now() >= self.deadline {
                return Err(fail("client_coordination_timeout"));
            }
            if let Some(status) = self.child.get_mut().try_wait().map_err(io)? {
                if !status.success() {
                    return Err(fail("client_coordination_release"));
                }
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        if let Some(reader) = self.reader.take() {
            reader.join().map_err(io)?;
        }
        if self.rx.try_recv().is_ok() {
            return Err(fail("client_coordination_protocol"));
        }
        self.released = true;
        Ok(())
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        // Never signal a runtime/service or discover a process by name. This
        // dedicated lock-only direct child has an independent 2s self expiry.
        // A kill here is cleanup after failure, never proof of normal release.
        if !self.released {
            if !self.release.exists() {
                let _ = self.request_release();
            }
            let until = Instant::now() + Duration::from_millis(100);
            loop {
                match self.child.get_mut().try_wait() {
                    Ok(Some(_)) => break,
                    _ if Instant::now() < until => thread::sleep(Duration::from_millis(2)),
                    _ => {
                        let _ = self.child.get_mut().kill();
                        let _ = self.child.get_mut().wait();
                        break;
                    }
                }
            }
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        // Retain private control material. Only the owning lifecycle transaction
        // may remove its evidence after release/failed cleanup is independently checked.
    }
}

pub fn with_coordination<T>(
    context: &Context,
    deadline: Instant,
    operation: impl FnOnce(&Lease) -> AppResult<T>,
) -> AppResult<T> {
    let started = Instant::now();
    if started >= deadline {
        return Err(fail("client_coordination_timeout"));
    }
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail("client_coordination_source"))?;
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read(&root, platform, target)?
        .ok_or_else(|| fail("client_coordination_source"))?;
    if selected.selection.version != "2.99.0"
        || selected.image.official_archive_sha256
            != "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
        || context.runtime.executable().as_ref() != Some(&selected.stable_entry)
    {
        return Err(fail("client_coordination_source"));
    }
    program_image::verify(&selected)?;
    let package = selected.paths.package_root.join(PACKAGE_SUBPATH);
    let mut roots = Vec::new();
    for path in [
        &context.user_profile,
        &context.codex_home,
        &context.opencodex_home,
        &context.control_root,
        &package,
    ] {
        if !program_layout::clean_absolute(path) || fs::canonicalize(path).map_err(io)? != *path {
            return Err(fail("client_coordination_context"));
        }
        roots.push(PinnedDirectory::open(path).map_err(io)?);
    }
    roots[3].private().map_err(io)?;
    if [&context.codex_home, &context.opencodex_home, &root]
        .iter()
        .any(|path| {
            context.control_root.starts_with(path) || path.starts_with(&context.control_root)
        })
    {
        return Err(fail("client_coordination_context"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let control_path = context.control_root.join(&id);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&control_path)
        .map_err(io)?;
    let control = PinnedDirectory::open(&control_path).map_err(io)?;
    control.private().map_err(io)?;
    let release = control_path.join("release");
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(fail("client_coordination_timeout"));
    }
    let hold = remaining.min(MAX_HOLD);
    let input = Input {
        schema: SCHEMA,
        operation: &id,
        package_root: &package,
        profile: &context.user_profile,
        codex_home: &context.codex_home,
        opencodex_home: &context.opencodex_home,
        release_path: &release,
        hold_ms: hold.as_millis().try_into().map_err(io)?,
    };
    let json = serde_json::to_string(&input).map_err(io)?;
    let context_sha256 = super::hash::sha256_hex(
        &serde_json::to_vec(&[
            &package,
            &context.user_profile,
            &context.codex_home,
            &context.opencodex_home,
        ])
        .map_err(io)?,
    );
    let mut command = Command::new(&selected.paths.bun);
    command
        .args(["--eval", HELPER, "--", &json])
        .env_clear()
        .env("HOME", &context.user_profile)
        .env("CODEX_HOME", &context.codex_home)
        .env("OPENCODEX_HOME", &context.opencodex_home)
        .env(
            "PATH",
            super::platform::controlled_path(
                [selected.paths.node.parent().unwrap().to_path_buf()],
                None,
            ),
        )
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .current_dir(&context.control_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(io)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| fail("client_coordination_protocol"))?;
    let (rx, reader) = spawn_reader(stdout);
    let mut lease = Lease {
        child: RefCell::new(child),
        expires: Instant::now() + hold,
        deadline,
        operation: id,
        context_sha256,
        held: None,
        control,
        release,
        roots,
        rx,
        reader: Some(reader),
        released: false,
    };
    let held = lease.receive()?;
    if held.kind != "held" {
        return Err(fail("client_coordination_protocol"));
    }
    lease.held = Some(held);
    lease.check()?;
    let result = operation(&lease);
    let released = lease.finish();
    // A failed callback cannot be changed into success by normal lock release.
    let value = result?;
    released?;
    if Instant::now() >= deadline {
        return Err(fail("client_coordination_timeout"));
    }
    program_selection::verify_current(&root, &selected)?;
    program_image::verify(&selected)?;
    if Instant::now() >= deadline {
        return Err(fail("client_coordination_timeout"));
    }
    for root in &lease.roots {
        root.recheck().map_err(io)?;
    }
    Ok(value)
}

#[cfg(test)]
#[path = "official_client_coordination_tests.rs"]
mod tests;
