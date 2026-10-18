//! Windows recovery restarts only after the old process has actually exited.
//! Tauri's direct restart spawns first. A hidden system PowerShell waiter holds
//! the old generation's handle and requires a separate commit before waiting.
use std::{
    ffi::{c_void, OsStr, OsString},
    io::{self, BufRead, BufReader, Read, Write},
    os::windows::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

const READY: &str = "OCX_RESTART_READY_V1";
const COMMIT: &[u8] = b"OCX_RESTART_COMMIT_V1\n";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);
const EXIT_TIMEOUT_MS: u32 = 30_000;
const WAITER: &str = include_str!("windows_restart.ps1");

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    fn GetCurrentProcess() -> *mut c_void;
    fn GetProcessTimes(
        process: *mut c_void,
        created: *mut FileTime,
        exited: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
}

fn system_powershell() -> io::Result<PathBuf> {
    let mut buffer = vec![0u16; 32_768];
    // SAFETY: the owned writable buffer contains the stated number of WCHARs.
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
    if length == 0 {
        return Err(io::Error::last_os_error());
    }
    if length as usize >= buffer.len() {
        return Err(io::Error::other(
            "system directory exceeds the bounded buffer",
        ));
    }
    use std::os::windows::ffi::OsStringExt;
    let mut path = PathBuf::from(OsString::from_wide(&buffer[..length as usize]));
    path.push("WindowsPowerShell/v1.0/powershell.exe");
    Ok(path)
}

fn creation_time(process: *mut c_void) -> io::Result<u64> {
    let (mut created, mut exited, mut kernel, mut user) = (
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
    );
    // SAFETY: callers retain a live process handle for the whole observation.
    if unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((u64::from(created.high) << 32) | u64::from(created.low))
}

fn hex_units(units: impl IntoIterator<Item = u16>) -> String {
    use std::fmt::Write;
    let mut text = String::new();
    for unit in units {
        for byte in unit.to_le_bytes() {
            write!(&mut text, "{byte:02x}").expect("writing to a string");
        }
    }
    text
}

fn hex(value: &OsStr) -> String {
    hex_units(value.encode_wide())
}

/// Quote for CreateProcess/CommandLineToArgvW, not for a shell. PowerShell sees
/// only hex; the reconstructed argument string goes directly to ProcessStartInfo.
fn arguments(values: &[OsString]) -> Vec<u16> {
    let mut result = Vec::new();
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            result.push(b' ' as u16);
        }
        result.push(b'"' as u16);
        let mut slashes = 0;
        for unit in value.encode_wide() {
            if unit == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            if unit == b'"' as u16 {
                result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2 + 1));
            } else {
                result.extend(std::iter::repeat_n(b'\\' as u16, slashes));
            }
            slashes = 0;
            result.push(unit);
        }
        result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        result.push(b'"' as u16);
    }
    result
}

struct Request<'a> {
    parent_id: u32,
    parent_created: u64,
    parent_image: &'a Path,
    target: &'a Path,
    directory: &'a Path,
    args: &'a [OsString],
    exit_timeout_ms: u32,
}

fn request_input(request: &Request<'_>) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        request.parent_id,
        request.parent_created,
        hex(request.parent_image.as_os_str()),
        hex(request.target.as_os_str()),
        hex(request.directory.as_os_str()),
        hex_units(arguments(request.args)),
        request.exit_timeout_ms,
    )
}

pub(crate) struct RestartHandoff {
    child: Child,
    committed: bool,
}

impl RestartHandoff {
    pub(crate) fn commit(mut self) -> io::Result<()> {
        let mut input = self
            .child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("restart waiter input is unavailable"))?;
        input.write_all(COMMIT)?;
        input.flush()?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for RestartHandoff {
    fn drop(&mut self) {
        if !self.committed {
            // Only the helper created here can be stopped. Never terminate the
            // application or an arbitrary process selected by a numeric PID.
            drop(self.child.stdin.take());
            if self.child.kill().is_ok() {
                let _ = self.child.wait();
            }
        }
    }
}

fn prepare(request: &Request<'_>) -> io::Result<RestartHandoff> {
    let mut child = Command::new(system_powershell()?)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
        ])
        .arg(WAITER)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("restart waiter output is unavailable"))?;
    let mut handoff = RestartHandoff {
        child,
        committed: false,
    };
    let input = handoff
        .child
        .stdin
        .as_mut()
        .ok_or_else(|| io::Error::other("restart waiter input is unavailable"))?;
    input.write_all(request_input(request).as_bytes())?;
    input.flush()?;
    let (send, receive) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(output)
            .take(128)
            .read_line(&mut line)
            .map(|_| line.trim_end_matches(['\r', '\n']) == READY);
        let _ = send.send(result);
    });
    match receive.recv_timeout(HANDSHAKE_TIMEOUT) {
        Ok(Ok(true)) => Ok(handoff),
        _ => {
            // Drop closes the uncommitted input and stops this owned helper.
            handoff.committed = false;
            Err(io::Error::other(
                "restart waiter did not confirm the process generation",
            ))
        }
    }
}

pub(crate) fn prepare_current_restart() -> io::Result<RestartHandoff> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::other("restart executable has no parent directory"))?;
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    // SAFETY: the pseudo handle identifies this process and is never closed.
    let created = creation_time(unsafe { GetCurrentProcess() })?;
    prepare(&Request {
        parent_id: std::process::id(),
        parent_created: created,
        parent_image: &executable,
        target: &executable,
        directory,
        args: &args,
        exit_timeout_ms: EXIT_TIMEOUT_MS,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_quotes_unicode_and_trailing_slashes_are_literal_arguments() {
        let args = ["", "空 格", "a\"b", r"C:\ends\", "$();&|", "plain"].map(OsString::from);
        let encoded = String::from_utf16(&arguments(&args)).unwrap();
        assert_eq!(
            encoded,
            "\"\" \"空 格\" \"a\\\"b\" \"C:\\ends\\\\\" \"$();&|\" \"plain\""
        );
    }

    #[test]
    fn request_data_never_becomes_powershell_source() {
        let path = Path::new(r"C:\目录 $();&'\app.exe");
        let args = [OsString::from("';exit 99;#")];
        let text = request_input(&Request {
            parent_id: 123,
            parent_created: 456,
            parent_image: path,
            target: path,
            directory: Path::new(r"C:\目录 $();&'"),
            args: &args,
            exit_timeout_ms: EXIT_TIMEOUT_MS,
        });
        assert!(!text.contains("';exit 99;#"));
        assert!(!text.contains("C:\\目录"));
        assert!(text.starts_with("123\n456\n"));
        let lines = text.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 7);
        assert!(lines[2..6]
            .iter()
            .all(|line| line.bytes().all(|b| b.is_ascii_hexdigit())));
    }
}
