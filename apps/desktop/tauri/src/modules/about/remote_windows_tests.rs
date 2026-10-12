//! Actual Windows metadata-query process tests. Fixtures use only the explicitly
//! supplied test Node binary and their own temporary directories.
use super::*;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use windows_sys::Win32::Foundation::{FILETIME, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
    WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

const FIXTURE: &str = r#"
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
if (process.argv[2] === '--sleep') {
  setTimeout(() => process.exit(0), 30000);
} else {
  fs.writeFileSync(path.join(__dirname, 'parent-pid'), String(process.pid));
  const mode = fs.readFileSync(path.join(__dirname, 'mode'), 'utf8');
  const child = spawn(process.execPath, [__filename, '--sleep'], {
    windowsHide: true,
    stdio: ['ignore', mode === 'timeout' ? 'inherit' : 'ignore', 'ignore'],
  });
  fs.writeFileSync(path.join(__dirname, 'child-pid'), String(child.pid));
  if (mode === 'overflow') process.stdout.write('x'.repeat(17000));
  if (mode === 'success') {
    process.stdout.write('{"version":"2.50.0","dist.integrity":"sha512-fixture"}',
      () => process.exit(0));
  }
  if (mode === 'nonzero') process.exit(9);
  setTimeout(() => process.exit(0), 30000);
}
"#;

struct ObservedChild(OwnedHandle);
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
}
impl Drop for ObservedChild {
    fn drop(&mut self) {
        // On the red regression, terminate only this verified retained fixture
        // process; never search or kill by name or a later reused PID.
        unsafe {
            TerminateProcess(self.0.as_raw_handle(), 1);
            WaitForSingleObject(self.0.as_raw_handle(), 2000);
        }
    }
}

fn metadata_process_case(mode: &str) {
    let node =
        PathBuf::from(std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit test Node path"));
    assert!(node.is_absolute() && node.is_file());
    let root = tempfile::tempdir().unwrap();
    let fixture = root.path().join("metadata-fixture.cjs");
    std::fs::write(&fixture, FIXTURE).unwrap();
    std::fs::write(root.path().join("mode"), mode).unwrap();
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
    let mut control = std::process::Command::new(&node)
        .args([fixture.as_os_str(), std::ffi::OsStr::new("--sleep")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x0800_0000)
        .spawn()
        .unwrap();
    let created_after = std::time::SystemTime::now();
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().to_owned(),
        home: Some(root.path().join("home").into_os_string()),
        ..Default::default()
    };
    let result = run_npm_view_with_timeout(
        &npm,
        root.path(),
        &root.path().join("cache/npm"),
        &environment,
        "test@latest",
        std::time::Duration::from_secs(2),
    );
    let pid: u32 = std::fs::read_to_string(root.path().join("child-pid"))
        .expect("fixture must have started its descendant within the total deadline")
        .trim()
        .parse()
        .unwrap();
    let descendant = ObservedChild::open(pid, &node, created_after);
    let parent_pid: u32 = std::fs::read_to_string(root.path().join("parent-pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let parent = ObservedChild::open(parent_pid, &node, created_after);
    let stopped = descendant.as_ref().is_none_or(ObservedChild::stopped);
    let parent_stopped = parent.as_ref().is_none_or(ObservedChild::stopped);
    let control_live = control.try_wait().unwrap().is_none();
    let _ = control.kill();
    let _ = control.wait();
    drop(parent);
    drop(descendant);
    assert!(control_live, "unrelated control process was stopped");
    match mode {
        "timeout" => assert!(matches!(result, Err(AppError::Timeout))),
        "success" => assert!(result.is_ok()),
        _ => assert!(result.is_err()),
    }
    assert!(
        stopped,
        "metadata query left its own {mode} descendant running"
    );
    assert!(
        parent_stopped,
        "metadata query left its own {mode} npm worker running"
    );
}

#[test]
fn metadata_timeout_cleans_descendant_and_preserves_unrelated_process() {
    metadata_process_case("timeout");
}

#[test]
fn metadata_output_overflow_cleans_descendant() {
    metadata_process_case("overflow");
}

#[test]
fn metadata_nonzero_exit_cleans_descendant() {
    metadata_process_case("nonzero");
}

#[test]
fn metadata_success_also_reaps_its_owned_helper_tree() {
    metadata_process_case("success");
}
