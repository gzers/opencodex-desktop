//! Isolated native helper probe, built by the accompanying evidence harness.
//! This is not a Desktop/installer candidate and cannot certify full app isolation.
#![cfg(windows)]
#[allow(dead_code)]
#[path = "../../apps/desktop/tauri/src/infrastructure/windows_user_scope.rs"]
mod scope;
use std::{io, path::PathBuf};
fn run() -> io::Result<serde_json::Value> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut result = serde_json::json!({
        "kind": "windows023-native-helper", "pid": std::process::id(),
        "sid": scope::current_user_sid()?, "profile": scope::current_user_profile()?,
        "localAppData": scope::current_user_local_app_data()?,
        "sourceCommit": option_env!("OCX_WIN23_SOURCE_COMMIT"),
        "helperSha256": option_env!("OCX_WIN23_HELPER_SHA256"),
        "fullProductVerified": false
    });
    if args.len() == 1 && args[0] == "--identity" {
        return Ok(result);
    }
    if args.len() != 2 || args[0] != "--fixture" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--identity or --fixture <new empty windows023-fixture-* directory> required",
        ));
    }
    let root = PathBuf::from(&args[1]);
    if !root.is_absolute()
        || !root
            .file_name()
            .is_some_and(|x| x.to_string_lossy().starts_with("windows023-fixture-"))
        || std::fs::read_dir(&root)?.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "an existing, empty, isolated fixture directory is required",
        ));
    }
    scope::protect_current_user_path(&root, 0o700)?;
    let file = root.join("windows023-synthetic-secret.txt");
    std::fs::write(&file, b"synthetic fixture only; never a real credential")?;
    scope::protect_current_user_path(&file, 0o600)?;
    if std::fs::read(&file)? != b"synthetic fixture only; never a real credential" {
        return Err(io::Error::other("fixture readback differs"));
    }
    result["fixtureRoot"] = serde_json::json!(root);
    result["privateFile"] = serde_json::json!(file);
    result["ownReadback"] = serde_json::json!(true);
    Ok(result)
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(result) => {
            println!("{result}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({"kind":"windows023-native-helper", "errorKind":format!("{:?}",error.kind()), "error":error.to_string(), "osError":error.raw_os_error()})
            );
            std::process::ExitCode::from(1)
        }
    }
}
