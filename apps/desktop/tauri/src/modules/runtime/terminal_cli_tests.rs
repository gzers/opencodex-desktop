use super::*;
use crate::modules::{data_root, runtime::program_selection::tests::Fixture as ProgramFixture};
use std::os::unix::fs::PermissionsExt;
pub(crate) struct Fixture {
    _program: ProgramFixture,
    _root: tempfile::TempDir,
    pub(crate) c: Context,
}
impl Fixture {
    pub(crate) fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().canonicalize().unwrap();
        let profile = base.join("user space ' literal $name");
        fs::create_dir(&profile).unwrap();
        let exe = base.join("Applications/App name.app/Contents/MacOS/desktop");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        let program = program_layout::ProgramLocations::resolve(
            &exe,
            &profile,
            data_root::layout::Platform::Macos,
        )
        .unwrap()
        .program_root;
        fs::create_dir_all(&program).unwrap();
        let d = profile.join(".codex-desktop/desktop");
        data_root::initialize(&d).unwrap();
        let home = data_root::resolve_opencodex_home(&data_root::load_runtime_config(&d).unwrap());
        fs::create_dir_all(&home).unwrap();
        data_root::startup_v2::set_location(&d, d.parent().unwrap()).unwrap();
        let source = ProgramFixture::new();
        fs::rename(source.root.join("runtime"), program.join("runtime")).unwrap();
        let (_, platform) = program_layout::host_target().unwrap();
        let selection: program_selection::CurrentSelection =
            serde_json::from_slice(&fs::read(program.join("runtime/current.json")).unwrap())
                .unwrap();
        let selected =
            program_selection::read_image(&program, platform, "darwin-arm64", &selection).unwrap();
        fs::write(
            &selected.stable_entry,
            super::super::install::entry_script(Some(&selected.paths.node), &selected.script),
        )
        .unwrap();
        fs::set_permissions(&selected.stable_entry, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(&exe, b"#!/bin/sh\nprintf '%s\\n' \"$@\"\n").unwrap();
        fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            _program: source,
            _root: root,
            c: Context {
                user_profile: profile,
                desktop_root: d,
                opencodex_home: home,
                program_root: program,
                manager_executable: exe,
            },
        }
    }
    fn preview(&self, a: Action) -> Preview {
        preview(&self.c, a, "/bin/zsh", &[]).unwrap()
    }
    pub(crate) fn change(&self, a: Action) {
        let p = self.preview(a);
        apply(&self.c, a, "/bin/zsh", &[], &p.confirmation_digest).unwrap();
    }
}
#[test]
fn enable_is_explicit_new_zsh_resolves_command_and_disable_preserves_user_content() {
    let f = Fixture::new();
    let profile = f.c.user_profile.join(".zshrc");
    let original = b"# user content without newline";
    fs::write(&profile, original).unwrap();
    let p = f.preview(Action::Enable);
    assert!(!p.enabled);
    assert!(!f.c.command().exists());
    assert!(!f.c.record().exists());
    apply(
        &f.c,
        Action::Enable,
        "/bin/zsh",
        &[],
        &p.confirmation_digest,
    )
    .unwrap();
    assert!(inspect(&f.c, "/bin/zsh", &[]).unwrap().enabled);
    assert_eq!(fs::read(f.c.command()).unwrap(), scripts(&f.c).unwrap().0);
    let output = Command::new("/bin/zsh")
        .args([
            "-i",
            "-c",
            "command -v ocx; ocx 'arg space' 'literal $text' '--version'",
        ])
        .env_clear()
        .env("HOME", &f.c.user_profile)
        .env("PATH", "/usr/bin:/bin")
        .env("ZDOTDIR", &f.c.user_profile)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with(&f.c.command().to_string_lossy().to_string()));
    assert!(text.contains("--ocxd-runtime-cli\narg space\nliteral $text\n--version"));
    let mut content = fs::read(&profile).unwrap();
    content.extend_from_slice(b"# later user content\n");
    fs::write(&profile, &content).unwrap();
    f.change(Action::Disable);
    assert!(!f.c.command().exists());
    assert!(!f.c.bridge().exists());
    assert!(!f.c.record().exists());
    let mut expected = original.to_vec();
    expected.extend_from_slice(b"\n# later user content\n");
    assert_eq!(fs::read(profile).unwrap(), expected);
}
#[test]
fn fresh_profile_cancel_and_repeated_enable_do_not_duplicate_path_block() {
    let f = Fixture::new();
    let before = fs::read_dir(&f.c.user_profile).unwrap().count();
    let _ = f.preview(Action::Enable);
    assert_eq!(fs::read_dir(&f.c.user_profile).unwrap().count(), before);
    f.change(Action::Enable);
    let profile = f.c.user_profile.join(".zshrc");
    let first = fs::read(&profile).unwrap();
    f.change(Action::Enable);
    assert_eq!(fs::read(&profile).unwrap(), first);
    f.change(Action::Disable);
    assert!(!profile.exists());
    assert!(!f.c.pending().exists());
}
#[test]
fn existing_commands_foreign_paths_and_stale_preview_never_overwrite() {
    for kind in ["command", "path", "source", "profile", "symlink"] {
        let f = Fixture::new();
        let p = f.preview(Action::Enable);
        let other = f.c.user_profile.join("other-bin");
        fs::create_dir(&other).unwrap();
        let search = vec![other.clone()];
        match kind {
            "command" => {
                fs::create_dir_all(f.c.command().parent().unwrap()).unwrap();
                fs::write(f.c.command(), b"foreign command").unwrap();
            }
            "path" => {
                fs::write(other.join("ocx"), b"existing path command").unwrap();
            }
            "source" => {
                fs::write(f.c.program_root.join("runtime/current.json"), b"changed").unwrap();
            }
            "profile" => {
                fs::write(f.c.user_profile.join(".zshrc"), b"external edit").unwrap();
            }
            "symlink" => {
                std::os::unix::fs::symlink(&other, f.c.user_profile.join(".local")).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            apply(
                &f.c,
                Action::Enable,
                "/bin/zsh",
                &search,
                &p.confirmation_digest
            )
            .is_err(),
            "{kind}"
        );
        assert!(!f.c.record().exists());
        assert!(!f.c.pending().exists());
        if kind == "path" {
            assert_eq!(
                fs::read(other.join("ocx")).unwrap(),
                b"existing path command"
            );
        }
    }
}
#[test]
fn disable_refuses_modified_owned_script_and_can_restore_an_interrupted_enable() {
    let f = Fixture::new();
    let (t, _) = plan(&f.c, Action::Enable, "/bin/zsh", &[]).unwrap();
    fs::write(f.c.pending(), encode(&t).unwrap()).unwrap();
    let c = &t.changes[0];
    let (_, path, mode) = target(&f.c, &c.target).unwrap();
    atomic_write(&path, c.after.as_ref().unwrap(), mode).unwrap();
    let p = f.preview(Action::Enable);
    assert!(p.pending);
    apply(
        &f.c,
        Action::Enable,
        "/bin/zsh",
        &[],
        &p.confirmation_digest,
    )
    .unwrap();
    assert!(!f.c.pending().exists());
    fs::write(f.c.command(), b"external replacement").unwrap();
    assert!(preview(&f.c, Action::Disable, "/bin/zsh", &[]).is_err());
    assert_eq!(fs::read(f.c.command()).unwrap(), b"external replacement");
    assert!(f.c.record().exists());
}
#[test]
fn bash_profiles_new_terminal_use_same_entry_and_absent_files_are_removed() {
    let f = Fixture::new();
    for name in [".bash_profile", ".bashrc"] {
        assert!(!f.c.user_profile.join(name).exists());
    }
    let p = preview(&f.c, Action::Enable, "/bin/bash", &[]).unwrap();
    assert_eq!(p.profile_paths.len(), 2);
    apply(
        &f.c,
        Action::Enable,
        "/bin/bash",
        &[],
        &p.confirmation_digest,
    )
    .unwrap();
    let output = Command::new("/bin/bash")
        .args(["--noprofile", "-i", "-c", "ocx --version"])
        .env_clear()
        .env("HOME", &f.c.user_profile)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("--ocxd-runtime-cli\n--version"));
    let p = preview(&f.c, Action::Disable, "/bin/bash", &[]).unwrap();
    apply(
        &f.c,
        Action::Disable,
        "/bin/bash",
        &[],
        &p.confirmation_digest,
    )
    .unwrap();
    for name in [".bash_profile", ".bashrc"] {
        assert!(!f.c.user_profile.join(name).exists());
    }
}

#[test]
fn shell_path_conflicts_added_after_enable_remain_authoritative() {
    let f = Fixture::new();
    let other = f.c.user_profile.join("other path");
    fs::create_dir(&other).unwrap();
    fs::write(other.join("ocx"), b"#!/bin/sh\nprintf 'other command\\n'\n").unwrap();
    fs::set_permissions(other.join("ocx"), fs::Permissions::from_mode(0o755)).unwrap();
    f.change(Action::Enable);
    let output = Command::new("/bin/zsh")
        .args(["-i", "-c", "ocx"])
        .env_clear()
        .env("HOME", &f.c.user_profile)
        .env("ZDOTDIR", &f.c.user_profile)
        .env(
            "PATH",
            std::env::join_paths([other, PathBuf::from("/usr/bin"), PathBuf::from("/bin")])
                .unwrap(),
        )
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "other command\n");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("another ocx command"));
}

#[test]
fn corrupted_pending_cannot_publish_another_script_or_change_unrelated_profile_bytes() {
    for kind in ["script", "profile", "record", "target"] {
        let f = Fixture::new();
        let (mut t, _) = plan(&f.c, Action::Enable, "/bin/zsh", &[]).unwrap();
        match kind {
            "script" => t.changes[0].after = Some(b"unrecognized executable".to_vec()),
            "profile" => t.changes[2].after = Some(b"erase original shell config".to_vec()),
            "record" => {
                let mut i: Installed =
                    decode(t.changes.last().unwrap().after.as_ref().unwrap()).unwrap();
                i.profiles[0].block = "arbitrary removal".into();
                t.changes.last_mut().unwrap().after = Some(encode(&i).unwrap());
            }
            "target" => t.changes[2].target = Target::Profile(".bashrc".into()),
            _ => unreachable!(),
        }
        let bytes = encode(&t).unwrap();
        fs::write(f.c.pending(), &bytes).unwrap();
        assert!(
            preview(&f.c, Action::Enable, "/bin/zsh", &[]).is_err(),
            "{kind}"
        );
        assert!(!f.c.command().exists());
        assert!(!f.c.bridge().exists());
        assert!(!f.c.user_profile.join(".zshrc").exists());
        assert_eq!(fs::read(f.c.pending()).unwrap(), bytes);
    }
}

#[test]
fn forwarding_binds_current_config_program_and_arguments_and_refuses_pending() {
    let f = Fixture::new();
    f.change(Action::Enable);
    let (_, platform) = program_layout::host_target().unwrap();
    let selected = program_selection::read(&f.c.program_root, platform, "darwin-arm64")
        .unwrap()
        .unwrap();
    let command =
        forwarding_command(&f.c, vec!["--version".into(), "literal $name".into()]).unwrap();
    assert_eq!(command.get_program(), selected.paths.node.as_os_str());
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        vec![
            selected.script.as_os_str(),
            std::ffi::OsStr::new("--version"),
            std::ffi::OsStr::new("literal $name")
        ]
    );
    let env = command
        .get_envs()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        env[std::ffi::OsStr::new("OPENCODEX_HOME")],
        Some(f.c.opencodex_home.as_os_str())
    );
    assert!(!env.contains_key(std::ffi::OsStr::new("HOME")));
    assert!(!env.contains_key(std::ffi::OsStr::new("USERPROFILE")));
    fs::write(
        f.c.program_root.join("runtime/.selection-pending.json"),
        b"pending program operation",
    )
    .unwrap();
    assert!(forwarding_command(&f.c, vec!["--version".into()]).is_err());
    f.change(Action::Disable);
    assert!(!f.c.command().exists());
}

#[test]
#[ignore = "requires explicit freshly built Desktop executable; launches only an isolated CLI fixture"]
fn real_desktop_dispatch_enters_cli_before_gui_and_preserves_home_args_exit_status() {
    let binary = PathBuf::from(
        std::env::var_os("OCXD_TEST_MANAGER_BINARY").expect("explicit current Desktop binary"),
    );
    assert!(binary.is_absolute());
    let f = Fixture::new();
    fs::copy(&binary, &f.c.manager_executable).unwrap();
    fs::set_permissions(&f.c.manager_executable, fs::Permissions::from_mode(0o755)).unwrap();
    let (_, platform) = program_layout::host_target().unwrap();
    let selected = program_selection::read(&f.c.program_root, platform, "darwin-arm64")
        .unwrap()
        .unwrap();
    fs::write(
        &selected.paths.node,
        b"#!/bin/sh\nprintf '%s\\n' \"$OPENCODEX_HOME\" \"$HOME\" \"$@\"\nexit 23\n",
    )
    .unwrap();
    let manifest = selected
        .paths
        .version_root
        .join(program_selection::IMAGE_MANIFEST);
    let mut image: program_selection::ImageManifest =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    image.node_sha256 = hash::sha256_file(&selected.paths.node).unwrap();
    image.tree_sha256 =
        super::super::program_image::content_digest(&selected.paths.version_root).unwrap();
    let bytes = encode(&image).unwrap();
    fs::write(&manifest, &bytes).unwrap();
    let mut choice = selected.selection.clone();
    choice.image_sha256 = hash::sha256_hex(&bytes);
    fs::write(
        f.c.program_root.join("runtime/current.json"),
        encode(&choice).unwrap(),
    )
    .unwrap();
    f.change(Action::Enable);
    let output = Command::new(f.c.command())
        .args(["--version", "literal $arg", "two words"])
        .env_clear()
        .env("HOME", &f.c.user_profile)
        .env("TMPDIR", f._root.path())
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(23),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    assert_eq!(
        lines,
        vec![
            f.c.opencodex_home.to_str().unwrap(),
            f.c.user_profile.to_str().unwrap(),
            selected.script.to_str().unwrap(),
            "--version",
            "literal $arg",
            "two words"
        ]
    );
    assert!(output.stderr.is_empty());
    println!(
        "REAL_TERMINAL_DISPATCH {}",
        serde_json::json!({"exitCode":23,"sameHome":true,"exactArguments":true,"guiStarted":false})
    );
    f.change(Action::Disable);
    assert!(!f.c.command().exists());
}

#[test]
fn directory_switch_must_first_disable_cli_and_keeps_registration_on_refusal() {
    let f = Fixture::new();
    ensure_binding_change_allowed(&f.c.desktop_root).unwrap();
    f.change(Action::Enable);
    let registered = fs::read(f.c.record()).unwrap();
    assert!(ensure_binding_change_allowed(&f.c.desktop_root).is_err());
    assert_eq!(fs::read(f.c.record()).unwrap(), registered);
    f.change(Action::Disable);
    ensure_binding_change_allowed(&f.c.desktop_root).unwrap();
    fs::write(f.c.pending(), b"unreadable pending").unwrap();
    assert!(ensure_binding_change_allowed(&f.c.desktop_root).is_err());
    assert_eq!(fs::read(f.c.pending()).unwrap(), b"unreadable pending");
}

#[test]
#[ignore = "requires explicit Desktop binary and pinned official 2.99 bundle; read-only CLI only"]
fn real_official_cli_reports_selected_version_and_reads_the_bound_custom_home() {
    let binary =
        PathBuf::from(std::env::var_os("OCXD_TEST_MANAGER_BINARY").expect("explicit manager"));
    let archive =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit bundle"));
    let descriptor = PathBuf::from(
        std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
    );
    let pin = std::env::var("OCXD_TEST_BUNDLE_PIN").expect("trusted pin");
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let profile = base.join("profile");
    fs::create_dir(&profile).unwrap();
    let anchor = profile.join(".codex-desktop/desktop");
    data_root::initialize(&anchor).unwrap();
    let data = base.join("custom aggregate/desktop");
    data_root::initialize(&data).unwrap();
    let home = base.join("custom OpenCodex HOME");
    fs::create_dir(&home).unwrap();
    data_root::set_opencodex_home(&data, data_root::OpenCodexHomeMode::External, Some(&home))
        .unwrap();
    data_root::startup_v2::set_location(&anchor, data.parent().unwrap()).unwrap();
    let original = br#"{ "port":41234, "providers":{} }"#;
    let config = home.join("config.json");
    fs::write(&config, original).unwrap();
    let manager = base.join("Applications/Fixture.app/Contents/MacOS/opencodex-desktop");
    fs::create_dir_all(manager.parent().unwrap()).unwrap();
    fs::copy(binary, &manager).unwrap();
    fs::set_permissions(&manager, fs::Permissions::from_mode(0o755)).unwrap();
    let (_, platform) = program_layout::host_target().unwrap();
    let program = program_layout::ProgramLocations::resolve(&manager, &profile, platform)
        .unwrap()
        .program_root;
    super::super::bundled::versioned::provision(
        &program,
        &data,
        &archive,
        &fs::read(descriptor).unwrap(),
        &pin,
    )
    .unwrap();
    let context = Context {
        user_profile: profile.clone(),
        desktop_root: data.clone(),
        opencodex_home: home.clone(),
        program_root: program.clone(),
        manager_executable: manager,
    };
    let before = program_selection::read(&program, platform, "darwin-arm64")
        .unwrap()
        .unwrap();
    let review = preview(&context, Action::Enable, "/bin/zsh", &[]).unwrap();
    apply(
        &context,
        Action::Enable,
        "/bin/zsh",
        &[],
        &review.confirmation_digest,
    )
    .unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(context.command())
            .args(args)
            .env_clear()
            .env("HOME", &profile)
            .env("PATH", "/usr/bin:/bin")
            .env("TMPDIR", &base)
            .env("CODEX_HOME", profile.join(".codex"))
            .env("CLAUDE_CONFIG_DIR", profile.join(".claude"))
            .env("XDG_CONFIG_HOME", profile.join(".config"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let version = run(&["--version"]);
    assert_eq!(
        super::super::install::extract_semver(&version).as_deref(),
        Some("2.99.0")
    );
    let port = run(&["config", "get", "port", "--json"]);
    assert_eq!(serde_json::from_str::<u16>(port.trim()).unwrap(), 41234);
    assert_eq!(fs::read(&config).unwrap(), original);
    assert_eq!(
        program_selection::read(&program, platform, "darwin-arm64")
            .unwrap()
            .unwrap(),
        before
    );
    super::super::program_image::verify(&before).unwrap();
    assert!(!home.join("runtime-port.json").exists());
    let review = preview(&context, Action::Disable, "/bin/zsh", &[]).unwrap();
    apply(
        &context,
        Action::Disable,
        "/bin/zsh",
        &[],
        &review.confirmation_digest,
    )
    .unwrap();
    assert!(!context.command().exists());
    assert!(!context.bridge().exists());
    assert!(!profile.join(".zshrc").exists());
    println!(
        "REAL_OFFICIAL_TERMINAL {}",
        serde_json::json!({"version":"2.99.0","port":41234,"customHome":true,"configBytesUnchanged":true,"programIdentityUnchanged":true,"entryRemovedAfterDisable":true,"serviceStarted":false})
    );
}
