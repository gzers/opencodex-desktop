//! Shared Windows query/short-probe process fixtures. Only the explicitly
//! supplied test Node binary and their own temporary directories are used for
//! the queried tree. An unrelated control is cmd.exe blocked on its own stdin.
use crate::modules::process::EnvironmentPolicy;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use windows_sys::Win32::Foundation::{FILETIME, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
    WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

const FIXTURE: &str = r#"
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
function publishPid(name, pid) {
  const destination = path.join(__dirname, name);
  const temporary = destination + '.pending';
  fs.writeFileSync(temporary, String(pid));
  fs.renameSync(temporary, destination);
}
function publishPhase(name) {
  const destination = path.join(__dirname, name + '.json');
  const temporary = destination + '.pending';
  fs.writeFileSync(temporary, JSON.stringify({ pid: process.pid, atUnixMs: Date.now() }));
  fs.renameSync(temporary, destination);
}
if (process.argv[2] === '--sleep') {
  publishPhase('descendant-started');
  setInterval(() => {}, 1000);
} else {
  publishPhase('parent-started');
  publishPid('parent-pid', process.pid);
  const mode = fs.readFileSync(path.join(__dirname, 'mode'), 'utf8');
  publishPhase('before-child-spawn');
  const child = spawn(process.execPath, [__filename, '--sleep'], {
    windowsHide: true,
    detached: true,
    stdio: ['ignore', mode === 'timeout' ? 'inherit' : 'ignore', 'ignore'],
  });
  publishPid('child-pid', child.pid);
  publishPhase('after-child-spawn');
  const readyDeadline = Date.now() + 10000;
  while (!fs.existsSync(path.join(__dirname, 'ownership-ready'))) {
    if (Date.now() > readyDeadline) process.exit(12);
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 5);
  }
  publishPhase('ownership-ready-seen');
  if (mode === 'overflow') process.stdout.write('x'.repeat(17000));
  if (mode === 'success') {
    process.stdout.write(fs.readFileSync(path.join(__dirname, 'success-output'), 'utf8'),
      () => process.exit(0));
  }
  if (mode === 'nonzero') {
    publishPhase('before-nonzero-exit');
    process.exit(9);
  }
  setInterval(() => {}, 1000);
}
"#;

struct ObservedChild(OwnedHandle);
struct ControlProcess(std::process::Child);
impl ControlProcess {
    /// Observe only the retained handle created by this fixture. Never wait for
    /// Node to warm up or reinterpret a failed production query as a pass.
    fn trace(&self, label: &str, mode: &str, point: &str, started: std::time::Instant) {
        let handle = self.0.as_raw_handle();
        // SAFETY: Child retains this process handle until its own cleanup below.
        let wait_status = unsafe { WaitForSingleObject(handle, 0) };
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        // SAFETY: all output pointers refer to live initialized FILETIMEs.
        let times_ok =
            unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
        let times_error = (times_ok == 0).then(|| std::io::Error::last_os_error().raw_os_error());
        let ticks = |value: FILETIME| {
            (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
        };
        let times = (times_ok != 0).then(|| {
            serde_json::json!({
                "creation_filetime_100ns": ticks(creation), "exit_filetime_100ns": ticks(exit),
                "kernel_cpu_100ns": ticks(kernel), "user_cpu_100ns": ticks(user),
            })
        });
        let mut exit_code = 0_u32;
        // SAFETY: output points at a live u32; handle is the same owned process.
        let exit_ok = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
        let exit_error = (exit_ok == 0).then(|| std::io::Error::last_os_error().raw_os_error());
        eprintln!(
            "query fixture {label}/{mode}: control-snapshot={}",
            serde_json::json!({
                "point": point, "process_id": self.0.id(), "elapsed_ms": started.elapsed().as_millis(),
                "wait_status": wait_status, "process_times": times, "times_error": times_error,
                "exit_code": (exit_ok != 0).then_some(exit_code), "exit_error": exit_error,
            })
        );
    }
}
impl Drop for ControlProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl ObservedChild {
    fn open(pid: u32, node: &std::path::Path, not_before: std::time::SystemTime) -> Option<Self> {
        // PID comes only from our locally generated fixture. Retain the handle
        // and verify image and creation time before allowing test cleanup.
        let raw = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE | PROCESS_TERMINATE,
                0,
                pid,
            )
        };
        if raw.is_null() {
            return None; // A fully reaped descendant is already stopped.
        }
        let owned = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut image = vec![0_u16; 32768];
        let mut size = image.len() as u32;
        assert_ne!(
            unsafe { QueryFullProcessImageNameW(raw, 0, image.as_mut_ptr(), &mut size) },
            0
        );
        let actual = std::path::PathBuf::from(String::from_utf16(&image[..size as usize]).unwrap());
        assert_eq!(
            std::fs::canonicalize(actual).unwrap(),
            std::fs::canonicalize(node).unwrap()
        );
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        assert_ne!(
            unsafe { GetProcessTimes(raw, &mut creation, &mut exit, &mut kernel, &mut user) },
            0
        );
        let created_ticks =
            (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime);
        let expected_ticks = not_before
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            / 100
            + 116_444_736_000_000_000;
        assert!(
            u128::from(created_ticks) >= expected_ticks,
            "reused/stale fixture PID"
        );
        Some(Self(owned))
    }

    fn stopped(&self) -> bool {
        unsafe { WaitForSingleObject(self.0.as_raw_handle(), 2000) == WAIT_OBJECT_0 }
    }

    fn stop(&self) {
        unsafe {
            TerminateProcess(self.0.as_raw_handle(), 1);
            WaitForSingleObject(self.0.as_raw_handle(), 2000);
        }
    }
}
impl Drop for ObservedChild {
    fn drop(&mut self) {
        // On the red regression, terminate only this verified retained fixture
        // process; never search or kill by name or a later reused PID.
        self.stop();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryOutcome {
    Success,
    Failure,
    Timeout,
}

pub(crate) fn process_case(
    label: &str,
    mode: &str,
    success_output: &str,
    execute: impl FnOnce(&Path, &Path, EnvironmentPolicy) -> QueryOutcome,
) {
    let fixture_started = std::time::Instant::now();
    let fixture_started_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let node =
        PathBuf::from(std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit test Node path"));
    assert!(node.is_absolute() && node.is_file());
    let root = tempfile::tempdir().unwrap();
    let fixture = root.path().join("metadata-fixture.cjs");
    std::fs::write(&fixture, FIXTURE).unwrap();
    std::fs::write(root.path().join("mode"), mode).unwrap();
    std::fs::write(root.path().join("success-output"), success_output).unwrap();
    let npm = root.path().join("npm.cmd");
    assert!(!node.to_string_lossy().contains(['"', '%', '\r', '\n']));
    std::fs::write(
        &npm,
        format!(
            "@echo off\r\n\"{}\" \"%~dp0metadata-fixture.cjs\"\r\n",
            node.display()
        ),
    )
    .unwrap();
    // Keep an unrelated, retained process alive without starting another Node.
    // No readiness wait or warm-up is performed before the actual cold query.
    let control_command = crate::infrastructure::platform::system_path_dirs()
        .into_iter()
        .next()
        .expect("Windows system directory")
        .join("cmd.exe");
    let mut control = ControlProcess(
        std::process::Command::new(control_command)
            .args(["/d", "/c", "set /p OCX_QUERY_FIXTURE_CONTROL="])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000)
            .spawn()
            .unwrap(),
    );
    let created_after = std::time::SystemTime::now();
    let observer_root = root.path().to_owned();
    let observer_node = node.clone();
    let (query_completed, query_end) = std::sync::mpsc::channel();
    let observer = std::thread::spawn(move || {
        let ready_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let child_pid_file = observer_root.join("child-pid");
        while !child_pid_file.is_file() {
            assert!(
                std::time::Instant::now() < ready_deadline,
                "fixture descendant did not start: root={}, parent-pid={}, child-pid={}, ownership-ready={}",
                observer_root.display(),
                observer_root.join("parent-pid").is_file(),
                child_pid_file.is_file(),
                observer_root.join("ownership-ready").is_file()
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let pid: u32 = std::fs::read_to_string(child_pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let parent_pid: u32 = std::fs::read_to_string(observer_root.join("parent-pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let descendant = ObservedChild::open(pid, &observer_node, created_after);
        let parent = ObservedChild::open(parent_pid, &observer_node, created_after);
        assert!(
            descendant.is_some() && parent.is_some(),
            "fixture ended before handle ownership"
        );
        std::fs::write(observer_root.join("ownership-ready"), b"retained").unwrap();
        // Windows anonymous-pipe readers can keep a dropped Tokio runtime
        // waiting if a broken query leaves writers alive. Bound a red test by
        // cleaning only the retained fixture handles, and make that intervention
        // fail the assertion rather than count it as production cleanup.
        let watchdog_cleanup = query_end
            .recv_timeout(std::time::Duration::from_secs(8))
            .is_err();
        if watchdog_cleanup {
            parent.as_ref().unwrap().stop();
            descendant.as_ref().unwrap().stop();
        }
        (descendant, parent, watchdog_cleanup)
    });
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().to_owned(),
        home: Some(root.path().join("home").into_os_string()),
        path: Some(node.parent().unwrap().as_os_str().to_owned()),
        ..Default::default()
    };
    control.trace(label, mode, "before-query", fixture_started);
    let query_started = std::time::Instant::now();
    let result = execute(&npm, root.path(), environment);
    let query_elapsed = query_started.elapsed();
    eprintln!("query fixture {label}/{mode}: elapsed={query_elapsed:?}, result={result:?}");
    let _ = query_completed.send(());
    control.trace(label, mode, "query-returned", fixture_started);
    // Evidence only: never wait for/prewarm Node, extend the query deadline or
    // replace a failed outcome. Each marker belongs to this temporary fixture.
    eprintln!(
        "query fixture {label}/{mode}: started-unix-ms={fixture_started_unix_ms}, total-elapsed={:?}, node={}, control-pid={}",
        fixture_started.elapsed(),
        node.display(),
        control.0.id()
    );
    for phase in [
        "parent-started",
        "before-child-spawn",
        "after-child-spawn",
        "descendant-started",
        "ownership-ready-seen",
        "before-nonzero-exit",
    ] {
        let evidence = std::fs::read_to_string(root.path().join(format!("{phase}.json")));
        eprintln!("query fixture {label}/{mode}: phase={phase}, evidence={evidence:?}");
    }
    let observed = observer.join();
    // Preserve the control's state after the original observer deadline even
    // when it failed before opening query handles. Cleanup/assertions stay intact.
    control.trace(label, mode, "observer-returned", fixture_started);
    let (descendant, parent, watchdog_cleanup) =
        observed.expect("fixture handle observation failed");
    let stopped = descendant.as_ref().is_none_or(ObservedChild::stopped);
    let parent_stopped = parent.as_ref().is_none_or(ObservedChild::stopped);
    let control_live = control.0.try_wait().unwrap().is_none();
    drop(control);
    drop(parent);
    drop(descendant);
    assert!(control_live, "unrelated control process was stopped");
    assert!(
        !watchdog_cleanup,
        "query required external fixture watchdog cleanup"
    );
    assert!(
        query_elapsed < std::time::Duration::from_secs(4),
        "two-second query did not finish bounded cleanup: {query_elapsed:?}"
    );
    match mode {
        "timeout" => assert_eq!(result, QueryOutcome::Timeout),
        "success" => assert_eq!(result, QueryOutcome::Success),
        _ => assert_eq!(result, QueryOutcome::Failure),
    }
    assert!(
        stopped,
        "{label} query left its own {mode} descendant running"
    );
    assert!(
        parent_stopped,
        "{label} query left its own {mode} worker running"
    );
}
