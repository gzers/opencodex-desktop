//! Explicit isolated actual-GUI probe, registered only on Windows. This uses
//! the real main frontend and normal setup, but the empty broker still cannot
//! create an official Owner or issue any lifecycle/restore permission.
use super::empty_host;
use crate::infrastructure::{platform, windows_official_runtime024 as native};
use serde::Deserialize;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tauri::Manager;

const SID: &str = "S-1-5-21-587408064-856152790-2583328096-1003";
static STARTED: AtomicBool = AtomicBool::new(false);
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "isolated actual GUI admission refused",
    )
}
fn mode_valid(mode: &str) -> bool {
    ["normal", "hold", "after-failure", "finish"].contains(&mode)
}
fn same(a: &Path, b: &Path) -> bool {
    dunce::simplified(a)
        .to_string_lossy()
        .replace('/', "\\")
        .eq_ignore_ascii_case(&dunce::simplified(b).to_string_lossy().replace('/', "\\"))
}
fn token_profile() -> io::Result<PathBuf> {
    // platform::home_dir intentionally returns sandboxRoot/home in this GUI.
    // Task ancestry must use the original TokenUser Profile, not that app home
    // or a caller-controlled USERPROFILE environment value.
    crate::infrastructure::windows_startup_root::current_user_profile()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ExpectedBroker {
    pid: u32,
    creation_filetime: String,
    sid: String,
    session: u32,
    elevated: bool,
    elevation_type: u32,
    image: PathBuf,
    volume: u32,
    file_id: String,
}
impl ExpectedBroker {
    fn native_identity(self) -> io::Result<native::ProcessIdentity> {
        Ok(native::ProcessIdentity {
            pid: self.pid,
            creation_filetime: self.creation_filetime.parse().map_err(|_| invalid())?,
            token: native::TokenIdentity {
                sid: self.sid,
                session_id: self.session,
                elevated: self.elevated,
                elevation_type: self.elevation_type,
            },
            executable: self.image,
            executable_id: native::FileIdentity {
                volume: self.volume,
                index: self.file_id.parse().map_err(|_| invalid())?,
            },
        })
    }
}
fn write_json(path: &Path, value: &serde_json::Value) -> io::Result<()> {
    let mut file = platform::create_new_private_file(path, 0o600)?;
    serde_json::to_writer_pretty(&mut file, value).map_err(io::Error::other)?;
    file.flush()?;
    file.sync_all()
}

/// Called after the real main document Finished event. No environment variable
/// means no attachment, no files and no change to ordinary GUI operation.
pub(crate) fn page_loaded(webview: &tauri::Webview, payload: &tauri::webview::PageLoadPayload<'_>) {
    let Some(raw_base) = std::env::var_os("OCXD024_ACTUAL_GUI_BASE") else {
        return;
    };
    if webview.label() != "main"
        || payload.event() != tauri::webview::PageLoadEvent::Finished
        || STARTED.swap(true, Ordering::SeqCst)
    {
        return;
    }
    let app = webview.app_handle().clone();
    let base = PathBuf::from(raw_base);
    let mode = std::env::var("OCXD024_ACTUAL_GUI_MODE").unwrap_or_default();
    let result = (|| -> io::Result<_> {
        let profile = token_profile()?;
        let peer = native::RetainedPeer::current()?;
        let token = &peer.identity().token;
        let identity_checks = [
            ("known-mode", mode_valid(&mode)),
            ("original-sid", token.sid == SID),
            ("original-session", token.session_id == 7),
            (
                "ordinary-token",
                !token.elevated && token.elevation_type != 2,
            ),
            (
                "token-profile-parent",
                base.parent() == Some(profile.as_path()),
            ),
            (
                "task-name",
                base.file_name().is_some_and(|n| {
                    let n = n.to_string_lossy();
                    n.strip_prefix("ocx024-h-")
                        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
                }),
            ),
        ];
        if identity_checks.iter().any(|(_, passed)| !passed) {
            eprintln!("actual GUI identity admission checks: {identity_checks:?}");
            return Err(invalid());
        }
        let sandbox = base.join(format!("GUI-{mode}"));
        let sandbox_checks = [
            ("sandbox-enabled", crate::modules::test_sandbox::enabled()),
            (
                "sandbox-root",
                crate::modules::test_sandbox::sandbox_root().is_some_and(|p| same(&p, &sandbox)),
            ),
            (
                "managed-boundary",
                app.try_state::<crate::state::SharedSandboxBoundary>()
                    .is_some_and(|s| s.0.as_ref().is_some_and(|p| same(p, &sandbox))),
            ),
            (
                "managed-data-root",
                app.try_state::<crate::state::SharedDataRoot>()
                    .is_some_and(|s| {
                        dunce::simplified(&s.0).starts_with(dunce::simplified(&sandbox))
                    }),
            ),
            (
                "unresolved-runtime",
                app.try_state::<crate::state::ProcessContext>()
                    .is_some_and(|s| s.executable().is_err()),
            ),
            (
                "no-installed-runtime",
                !sandbox.join("program/runtime/current.json").exists(),
            ),
            (
                "main-origin",
                payload.url().host_str() == Some("tauri.localhost"),
            ),
            (
                "main-path",
                ["/", "/index.html"].contains(&payload.url().path()),
            ),
        ];
        if sandbox_checks.iter().any(|(_, passed)| !passed) {
            eprintln!("actual GUI sandbox admission checks: {sandbox_checks:?}");
            return Err(invalid());
        }
        peer.no_outer_job()?;
        peer.verify_image(&base.join("I/opencodex-desktop.exe"))?;
        let mut pins = Vec::new();
        for path in [
            &profile,
            &base,
            &base.join("I"),
            &base.join("D"),
            &base.join("H"),
            &sandbox,
        ] {
            pins.push(native::PinnedDirectory::open(path)?);
        }
        let data = base.join("D");
        // This is expected data from the independent creation-handle observer,
        // not an adopted process proof. manager_probe independently obtains and
        // compares the actual private-pipe server's complete native identity.
        let raw =
            native::PrivateRecord::locator_snapshot(&data.join("original-broker.json"), 65536)?;
        let original = serde_json::from_slice::<ExpectedBroker>(&raw)
            .map_err(io::Error::other)?
            .native_identity()?;
        if original.token != *token || !same(&original.executable, &base.join("I/ocxd.exe")) {
            return Err(invalid());
        }
        let window = app.get_webview_window("main").ok_or_else(invalid)?;
        if !window.is_visible().map_err(io::Error::other)? {
            return Err(invalid());
        }
        let hwnd = window.hwnd().map_err(io::Error::other)?.0 as usize;
        let url = payload.url().to_string();
        Ok((data, peer, pins, original, hwnd, url))
    })();
    match result {
        Ok((data, peer, pins, original, hwnd, url)) => {
            // Pipe IO never blocks the GUI event thread. Normal shutdown uses
            // app.exit after the observer's exact private task marker; hold is
            // terminated only through the observer's original Child handle.
            std::thread::spawn(move || {
                let result = (|| -> io::Result<()> {
                    let reply = empty_host::manager_probe(
                        &data,
                        &original,
                        if mode == "finish" {
                            "explicit-recovery"
                        } else {
                            "status"
                        },
                    )?;
                    peer.no_outer_job()?;
                    for pin in &pins {
                        pin.recheck()?;
                    }
                    let identity = peer.identity();
                    write_json(
                        &data.join(format!("gui-{mode}.json")),
                        &serde_json::json!({
                            "pid":identity.pid, "creationFiletime":identity.creation_filetime.to_string(),
                            "sid":identity.token.sid, "session":identity.token.session_id,
                            "elevated":identity.token.elevated, "image":identity.executable,
                            "nativeWindow":hwnd.to_string(), "nativeVisibleAtLoad":true,
                            "mainPageUrl":url, "mainPageFinished":true,
                            "fullNormalSetup":true, "frontendDomAssertion":false,
                            "actualGUI":true, "fixtureManager":false, "officialOwnerCreated":false,
                            "lifecycleAccepted":false, "allAccepted":false, "reply":reply
                        }),
                    )?;
                    let end = Instant::now() + Duration::from_secs(5);
                    let marker = data.join(format!("gui-{mode}.exit-request"));
                    while !marker.exists() {
                        if Instant::now() >= end {
                            return Err(io::ErrorKind::TimedOut.into());
                        }
                        peer.verify()?;
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let raw = native::PrivateRecord::locator_snapshot(&marker, 256)?;
                    if raw
                        != format!(
                            "exit-original-gui:{}:{}",
                            identity.pid, identity.creation_filetime
                        )
                        .as_bytes()
                    {
                        return Err(invalid());
                    }
                    Ok(())
                })();
                let code = if let Err(error) = result {
                    let _ = write_json(
                        &data.join(format!("gui-{mode}.error.json")),
                        &serde_json::json!({"error":error.to_string(),"rawError":error.raw_os_error(),"allAccepted":false}),
                    );
                    211
                } else {
                    0
                };
                app.exit(code);
            });
        }
        Err(error) => {
            // Do not follow an unadmitted path to write a diagnostic file.
            eprintln!(
                "actual GUI admission refused raw={:?}: {error}",
                error.raw_os_error()
            );
            app.exit(211);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_actual_gui_token_profile_survives_isolated_app_home() {
        let profile = token_profile().unwrap();
        let sandbox = profile.join(format!("ocx024-gui-profile-{}", uuid::Uuid::new_v4()));
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "infrastructure::windows_owner_host024::gui_probe::tests::isolated_profile_child",
                "--ignored",
                "--nocapture",
            ])
            .env("OPENCODEX_SANDBOX", "1")
            .env("OPENCODEX_SANDBOX_ROOT", &sandbox)
            .env_remove("OPENCODEX_SANDBOX_BOUNDARY")
            .env(
                "USERPROFILE",
                sandbox.join("caller-profile-is-not-token-profile"),
            )
            .env("OCX024_GUI_EXPECTED_TOKEN_PROFILE", &profile)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "profile subprocess raw={:?}: {} {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout)
            .contains("token Profile and isolated app home independently verified"));
        assert!(!sandbox.exists());
    }
    #[test]
    #[ignore = "private subprocess with isolated environment; no desktop or official process"]
    fn isolated_profile_child() {
        let expected =
            PathBuf::from(std::env::var_os("OCX024_GUI_EXPECTED_TOKEN_PROFILE").unwrap());
        let sandbox = crate::modules::test_sandbox::sandbox_root().unwrap();
        assert!(crate::modules::test_sandbox::enabled());
        assert_eq!(platform::home_dir().unwrap(), sandbox.join("home"));
        assert_eq!(token_profile().unwrap(), expected);
        assert_ne!(token_profile().unwrap(), platform::home_dir().unwrap());
        assert_ne!(
            token_profile().unwrap(),
            PathBuf::from(std::env::var_os("USERPROFILE").unwrap())
        );
        println!("token Profile and isolated app home independently verified; no paths created, GUI or official launched");
    }
    #[test]
    fn wl2_actual_gui_expected_identity_is_strict_data_only() {
        let value = serde_json::json!({"pid":123,"creationFiletime":"456","sid":SID,
            "session":7,"elevated":false,"elevationType":1,"image":"C:/task/I/ocxd.exe",
            "volume":8,"fileId":"9"});
        let raw = serde_json::to_string(&value).unwrap();
        let expected = serde_json::from_str::<ExpectedBroker>(&raw)
            .unwrap()
            .native_identity()
            .unwrap();
        assert_eq!(expected.creation_filetime, 456);
        for unknown in [
            raw.replacen('{', "{\"sourceAuthority\":true,", 1),
            raw.replacen('{', "{\"pid\":123,", 1),
            raw.replace("\"456\"", "\"not-native-filetime\""),
        ] {
            assert!(serde_json::from_str::<ExpectedBroker>(&unknown)
                .and_then(|v| v.native_identity().map_err(serde::de::Error::custom))
                .is_err());
        }
        // Decoded data alone has no retained handle or native proof method.
        assert_eq!(expected.executable_id.index, 9);
    }
}
