//! Thin console launcher: no shell, GUI, config discovery or runtime selection.
//! The adjacent ocxd performs the same current-user admission and forwarding.
use std::{
    ffi::OsString,
    io,
    path::Path,
    process::{Command, Stdio},
};
fn invocation(
    executable: &Path,
    arguments: impl IntoIterator<Item = OsString>,
) -> io::Result<Command> {
    if !executable.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute launcher path required",
        ));
    }
    let parent = executable
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "launcher directory absent"))?;
    let mut command = Command::new(parent.join("ocxd.exe"));
    command
        .arg("--ocxd-runtime-cli")
        .args(arguments)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    Ok(command)
}
fn main() {
    let result = std::env::current_exe()
        .and_then(|exe| invocation(&exe, std::env::args_os().skip(1)))
        .and_then(|mut c| c.status());
    match result {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(_) => {
            eprintln!("无法运行当前安装的 ocx，请在 OpenCodeX Desktop 中核对终端入口。");
            std::process::exit(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_are_native_os_values_not_shell_command_text() {
        let root = temp_root();
        let exe = root.join("program 空格 & !/ocx.exe");
        let arguments: Vec<OsString> = [
            "",
            "space value",
            "雪",
            "quote\"value",
            "&|<>^!()",
            "%FUTURE%",
            "trailing\\",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        let c = invocation(&exe, arguments.clone()).unwrap();
        assert_eq!(c.get_program(), root.join("program 空格 & !/ocxd.exe"));
        assert_eq!(
            c.get_args().collect::<Vec<_>>(),
            std::iter::once(std::ffi::OsStr::new("--ocxd-runtime-cli"))
                .chain(arguments.iter().map(OsString::as_os_str))
                .collect::<Vec<_>>()
        );
        assert_eq!(c.get_envs().count(), 0);
        assert!(c.get_current_dir().is_none());
        assert!(invocation(Path::new("ocx.exe"), Vec::<OsString>::new()).is_err());
    }
    fn temp_root() -> std::path::PathBuf {
        std::env::current_dir().unwrap()
    }
    #[cfg(unix)]
    #[test]
    fn direct_child_preserves_streams_and_exit_code_without_shell() {
        use std::{fs, io::Write, os::unix::fs::PermissionsExt};
        let root =
            std::env::temp_dir().join(format!("ocxd-native-shim-unit-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let result = std::panic::catch_unwind(|| {
            let target = root.join("ocxd.exe");
            fs::write(&target,b"#!/bin/sh\nprintf '%s\\n' \"$1\" \"$2\"\ncat\nprintf 'stderr-marker' >&2\nexit 23\n").unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
            let mut c = invocation(&root.join("ocx.exe"), [OsString::from("&|<>^!()")]).unwrap();
            c.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = c.spawn().unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(b"owned\0input\n")
                .unwrap();
            let output = child.wait_with_output().unwrap();
            assert_eq!(output.status.code(), Some(23));
            assert_eq!(
                output.stdout,
                b"--ocxd-runtime-cli\n&|<>^!()\nowned\0input\n"
            );
            assert_eq!(output.stderr, b"stderr-marker");
        });
        fs::remove_file(root.join("ocxd.exe")).unwrap();
        fs::remove_dir(&root).unwrap();
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}
