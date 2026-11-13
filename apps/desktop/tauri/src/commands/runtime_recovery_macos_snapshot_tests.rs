// Exit-race observation only. No source of normal Stop authority or signals.
use super::*;
use std::{
    cell::Cell,
    io::Write,
    process::{Command, Stdio},
};
fn identity(pid: u32, status: u32) -> libc::proc_bsdinfo {
    let mut b: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    b.pbi_pid = pid;
    b.pbi_uid = unsafe { libc::geteuid() };
    b.pbi_start_tvsec = 42;
    b.pbi_start_tvusec = 1;
    b.pbi_status = status;
    b
}
fn read_error() -> AppError {
    io(
        "synthetic procargs read",
        std::io::Error::from_raw_os_error(libc::EINVAL),
    )
}
#[test]
fn process_snapshot_errors_require_fresh_absence_and_never_adopt_live_or_reused_pid() {
    for mode in [
        "absent-before",
        "absent-after",
        "alive",
        "zombie-before",
        "zombie-after",
        "uid-reused",
        "birth-reused",
        "pid-reused",
        "observer-error",
        "success",
    ] {
        let observations = Cell::new(0);
        let read = Cell::new(false);
        let result = process_snapshot(
            42,
            || {
                let n = observations.get() + 1;
                observations.set(n);
                if mode == "absent-before" || n == 2 && mode == "absent-after" {
                    return Ok(None);
                }
                if n == 2 && mode == "observer-error" {
                    return Err(AppError::NotConfigured);
                }
                let mut b = identity(
                    42,
                    if mode == "zombie-before" || n == 2 && mode == "zombie-after" {
                        5
                    } else {
                        2
                    },
                );
                if n == 2 {
                    match mode {
                        "uid-reused" => b.pbi_uid += 1,
                        "birth-reused" => b.pbi_start_tvusec += 1,
                        "pid-reused" => b.pbi_pid += 1,
                        _ => (),
                    }
                }
                Ok(Some(b))
            },
            || {
                read.set(true);
                if mode == "success" {
                    Ok((
                        PathBuf::from("/owned/bin"),
                        vec!["owned".into()],
                        Some(PathBuf::from("/owned/home")),
                    ))
                } else {
                    Err(read_error())
                }
            },
        );
        match mode {
            "absent-before" | "absent-after" => assert!(matches!(result, Ok(None))),
            "zombie-before" | "zombie-after" => assert!(
                matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="recovery_process_exiting")
            ),
            "uid-reused" | "birth-reused" | "pid-reused" => assert!(
                matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="recovery_process_reused")
            ),
            "alive" => assert!(
                matches!(result,Err(AppError::FileSystem{operation,..}) if operation=="synthetic procargs read")
            ),
            "observer-error" => assert!(matches!(result, Err(AppError::NotConfigured))),
            "success" => {
                let p = result.unwrap().unwrap();
                assert_eq!(p.pid, 42);
                assert_eq!(p.created, "42:1");
                assert!(!p.zombie);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            read.get(),
            !matches!(mode, "absent-before" | "zombie-before")
        );
        assert_eq!(
            observations.get(),
            if matches!(mode, "absent-before" | "zombie-before") {
                1
            } else {
                2
            }
        );
    }
}
#[test]
fn process_snapshot_real_child_exit_during_arguments_read_is_native_absence_only() {
    let mut child = Command::new("/bin/sh")
        .args(["-c", "read value"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let reads = Cell::new(0);
    let args_failed = Cell::new(false);
    let result = process_snapshot(
        pid,
        || {
            reads.set(reads.get() + 1);
            bsd(pid)
        },
        || {
            let mut input = child.stdin.take().unwrap();
            input.write_all(b"done\n").unwrap();
            drop(input);
            let exit = child.wait().unwrap();
            assert!(exit.success());
            let result = arguments(pid);
            args_failed.set(result.is_err());
            let (argv, home) = result?;
            Ok((PathBuf::from("/bin/sh"), argv, home))
        },
    );
    assert!(matches!(result, Ok(None)));
    assert!(args_failed.get());
    assert_eq!(reads.get(), 2);
    assert!(bsd(pid).unwrap().is_none());
    println!(
        "PROCARGS_EXIT_RACE_EVIDENCE {}",
        serde_json::json!({"accepted":false,"childExitedNormally":true,"failedReadReobservedNativeAbsent":true,"identityObservations":reads.get(),"notServiceStopAuthority":true,"noSignalsSent":true})
    );
}
