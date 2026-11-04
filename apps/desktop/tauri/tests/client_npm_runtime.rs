//! Filesystem contract tests use a labelled synthetic PE. Ignored public-source
//! tests additionally check the real npm binary and unchanged JS selector; they
//! never execute the cmd, Node-selected executable, or native Codex.
use opencodex_desktop_lib::modules::model_config::cache_refresh::client_npm_runtime::resolve_npm_native;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/ocx021-codex-npm"
);
const X64: &str = "x86_64-pc-windows-msvc";
const ARM64: &str = "aarch64-pc-windows-msvc";

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    modules: PathBuf,
    package: PathBuf,
    launcher: PathBuf,
}

impl Fixture {
    fn new(local: bool) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("codex-npm-")
            .tempdir()
            .unwrap();
        let root = dunce::canonicalize(temp.path()).unwrap();
        let prefix = root.join("npm install 中文");
        let modules = prefix.join("node_modules");
        let package = modules.join("@openai/codex");
        let launcher = if local {
            modules.join(".bin/codex.cmd")
        } else {
            prefix.join("codex.cmd")
        };
        fs::create_dir_all(package.join("bin")).unwrap();
        fs::create_dir_all(launcher.parent().unwrap()).unwrap();
        fs::write(
            package.join("package.json"),
            include_bytes!("fixtures/ocx021-codex-npm/package.json"),
        )
        .unwrap();
        fs::write(
            package.join("bin/codex.js"),
            include_bytes!("fixtures/ocx021-codex-npm/codex-0.166.0.js"),
        )
        .unwrap();
        fs::write(
            &launcher,
            if local {
                include_bytes!("fixtures/ocx021-codex-npm/npm-local-codex.cmd").as_slice()
            } else {
                include_bytes!("fixtures/ocx021-codex-npm/npm-global-codex.cmd").as_slice()
            },
        )
        .unwrap();
        Self {
            _temp: temp,
            root,
            modules,
            package,
            launcher,
        }
    }

    fn optional(&self, nested: bool, cpu: &str) -> PathBuf {
        let root = if nested {
            self.package.join("node_modules")
        } else {
            self.modules.clone()
        };
        let optional = root.join(format!("@openai/codex-win32-{cpu}"));
        let manifest = Path::new(FIXTURES).join(format!("package-win32-{cpu}.json"));
        fs::create_dir_all(&optional).unwrap();
        fs::copy(manifest, optional.join("package.json")).unwrap();
        let triple = if cpu == "x64" { X64 } else { ARM64 };
        let native = optional.join(format!("vendor/{triple}/bin/codex.exe"));
        write_pe(&native, if cpu == "x64" { 0x8664 } else { 0xaa64 });
        native
    }

    fn embedded(&self, cpu: &str) -> PathBuf {
        let triple = if cpu == "x64" { X64 } else { ARM64 };
        let native = self.package.join(format!("vendor/{triple}/bin/codex.exe"));
        write_pe(&native, if cpu == "x64" { 0x8664 } else { 0xaa64 });
        native
    }

    fn resolve(&self) -> PathBuf {
        resolve_npm_native(&self.launcher, "0.166.0").unwrap()
    }
    fn refuses(&self) {
        let error = resolve_npm_native(&self.launcher, "0.166.0").unwrap_err();
        assert!(error.code.starts_with("client_npm_"));
        assert!(!error.message.contains(self.root.to_str().unwrap()));
    }
}

/// Minimal format-only fixture, explicitly not an official executable. Never
/// execute it or treat these assertions as native Codex version verification.
fn write_pe(path: &Path, machine: u16) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = vec![0; 1024];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
    bytes[64..68].copy_from_slice(b"PE\0\0");
    bytes[68..70].copy_from_slice(&machine.to_le_bytes());
    bytes[70..72].copy_from_slice(&1u16.to_le_bytes());
    bytes[84..86].copy_from_slice(&240u16.to_le_bytes());
    bytes[86..88].copy_from_slice(&2u16.to_le_bytes());
    bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
    fs::write(path, bytes).unwrap();
}

fn edit_json(path: &Path, edit: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

#[test]
fn standard_local_and_global_shims_resolve_only_their_linked_package() {
    for local in [false, true] {
        for nested in [false, true] {
            let fixture = Fixture::new(local);
            let native = fixture.optional(nested, "x64");
            let before = fs::read(&fixture.launcher).unwrap();
            assert_eq!(fixture.resolve(), native);
            assert_eq!(fs::read(&fixture.launcher).unwrap(), before);
        }
    }
}

#[test]
fn embedded_vendor_is_used_only_when_the_optional_dependency_is_absent() {
    let fixture = Fixture::new(true);
    let embedded = fixture.embedded("x64");
    assert_eq!(fixture.resolve(), embedded);
    let optional = fixture.optional(false, "x64");
    assert_eq!(fixture.resolve(), optional);
    fs::remove_file(optional).unwrap();
    fixture.refuses();
}

#[test]
fn arbitrary_wrappers_extra_commands_flags_and_escaping_targets_are_rejected() {
    for rewrite in [
        "custom",
        "append",
        "prepend",
        "argument",
        "escape",
        "different_package",
        "duplicate_target",
        "bom",
    ] {
        let fixture = Fixture::new(true);
        fixture.optional(false, "x64");
        let original = fs::read_to_string(&fixture.launcher).unwrap();
        let rewritten = match rewrite {
            "custom" => "@echo off\r\nnode arbitrary.js %*\r\n".to_owned(),
            "append" => format!("{original}\r\nwhoami\r\n"),
            "prepend" => format!("@echo injected\r\n{original}"),
            "argument" => original.replace("%*", "--config model=Injected %*"),
            "escape" => original.replace("..\\@openai", "..\\..\\external\\@openai"),
            "different_package" => original.replace("@openai\\codex", "@attacker\\codex"),
            "duplicate_target" => format!("{original}{original}"),
            "bom" => format!("\u{feff}{original}"),
            _ => unreachable!(),
        };
        fs::write(&fixture.launcher, rewritten).unwrap();
        fixture.refuses();
    }
}

#[test]
fn unsupported_versions_and_inconsistent_package_bin_alias_or_selector_refuse() {
    for mutation in [
        "version",
        "name",
        "bin",
        "extra_bin",
        "alias",
        "exports",
        "javascript",
    ] {
        let fixture = Fixture::new(true);
        fixture.optional(false, "x64");
        if mutation == "javascript" {
            fs::write(
                fixture.package.join("bin/codex.js"),
                "// real selector below\nmalicious()\n",
            )
            .unwrap();
        } else {
            edit_json(
                &fixture.package.join("package.json"),
                |value| match mutation {
                    "version" => value["version"] = json!("0.167.0"),
                    "name" => value["name"] = json!("@unknown/codex"),
                    "bin" => value["bin"]["codex"] = json!("../outside.js"),
                    "extra_bin" => value["bin"]["other"] = json!("bin/other.js"),
                    "alias" => {
                        value["optionalDependencies"]["@openai/codex-win32-x64"] =
                            json!("npm:@openai/codex@0.165.0-win32-x64")
                    }
                    "exports" => value["exports"] = json!({"./package.json":"../external.json"}),
                    _ => unreachable!(),
                },
            );
        }
        fixture.refuses();
    }
    let fixture = Fixture::new(false);
    fixture.optional(false, "x64");
    for version in [
        "",
        "latest",
        "^0.166.0",
        "v0.166.0",
        "0.166.0-beta",
        "0.0166.0",
        "0.165.0",
    ] {
        assert!(resolve_npm_native(&fixture.launcher, version).is_err());
    }
}

#[test]
fn stable_version_updates_with_the_same_selector_do_not_need_a_desktop_hash_update() {
    let fixture = Fixture::new(true);
    fixture.optional(false, "x64");
    edit_json(&fixture.package.join("package.json"), |value| {
        value["version"] = json!("0.167.0");
        for cpu in ["x64", "arm64"] {
            value["optionalDependencies"][format!("@openai/codex-win32-{cpu}")] =
                json!(format!("npm:@openai/codex@0.167.0-win32-{cpu}"));
        }
    });
    edit_json(
        &fixture.modules.join("@openai/codex-win32-x64/package.json"),
        |value| value["version"] = json!("0.167.0-win32-x64"),
    );
    assert!(resolve_npm_native(&fixture.launcher, "0.167.0").is_ok());
}

#[test]
fn optional_version_os_cpu_exports_and_native_format_must_match() {
    for mutation in [
        "version",
        "name",
        "os",
        "cpu",
        "exports",
        "missing",
        "script",
        "wrong_arch",
        "dll",
    ] {
        let fixture = Fixture::new(false);
        let native = fixture.optional(false, "x64");
        match mutation {
            "missing" => fs::remove_file(&native).unwrap(),
            "script" => fs::write(&native, b"@echo injected\r\n").unwrap(),
            "wrong_arch" => write_pe(&native, 0xaa64),
            "dll" => {
                let mut bytes = fs::read(&native).unwrap();
                bytes[86..88].copy_from_slice(&0x2002u16.to_le_bytes());
                fs::write(&native, bytes).unwrap();
            }
            _ => edit_json(
                &fixture.modules.join("@openai/codex-win32-x64/package.json"),
                |value| match mutation {
                    "version" => value["version"] = json!("0.165.0-win32-x64"),
                    "name" => value["name"] = json!("@unknown/codex"),
                    "os" => value["os"] = json!(["linux"]),
                    "cpu" => value["cpu"] = json!(["arm64"]),
                    "exports" => value["exports"] = json!({"./package.json":"./other.json"}),
                    _ => unreachable!(),
                },
            ),
        }
        fixture.refuses();
    }
}

#[test]
fn architecture_ambiguity_and_adjacent_non_x64_node_are_refused() {
    let fixture = Fixture::new(true);
    fixture.optional(false, "arm64");
    fixture.refuses();
    let fixture = Fixture::new(true);
    fixture.optional(false, "x64");
    fixture.optional(false, "arm64");
    fixture.refuses();
    let fixture = Fixture::new(false);
    fixture.optional(false, "x64");
    fixture.optional(true, "x64");
    fixture.refuses();
    let fixture = Fixture::new(true);
    fixture.embedded("arm64");
    fixture.refuses();
    let fixture = Fixture::new(true);
    fixture.embedded("x64");
    fixture.embedded("arm64");
    fixture.refuses();
    let fixture = Fixture::new(true);
    fixture.optional(false, "x64");
    let node = fixture.launcher.parent().unwrap().join("node.exe");
    write_pe(&node, 0xaa64);
    fixture.refuses();
    write_pe(&node, 0x8664);
    assert!(fixture.resolve().is_file());
}

#[test]
fn shadowing_and_external_optional_packages_are_not_used_as_fallbacks() {
    for shadow in [
        "bin/node_modules/@openai/codex-win32-x64",
        "../node_modules/@openai/codex-win32-x64",
    ] {
        let fixture = Fixture::new(true);
        fixture.optional(false, "x64");
        fs::create_dir_all(fixture.package.join(shadow)).unwrap();
        fixture.refuses();
    }
    let fixture = Fixture::new(true);
    fixture.embedded("x64");
    fs::create_dir_all(fixture.root.join("node_modules/@openai/codex-win32-x64")).unwrap();
    fixture.refuses();
}

#[test]
fn relative_non_cmd_missing_and_parent_traversal_inputs_are_rejected() {
    let fixture = Fixture::new(true);
    fixture.optional(false, "x64");
    for path in [
        PathBuf::from("codex.cmd"),
        fixture.launcher.with_extension("exe"),
        fixture.launcher.parent().unwrap().join("../.bin/codex.cmd"),
    ] {
        assert!(resolve_npm_native(&path, "0.166.0").is_err());
    }
    assert!(resolve_npm_native(&fixture.launcher, &"1".repeat(100_000)).is_err());
}

#[cfg(windows)]
#[test]
fn network_device_and_alternate_stream_paths_are_refused_before_io() {
    for path in [
        r"\\server\share\codex.cmd",
        r"\\.\pipe\codex.cmd",
        r"C:\npm\codex.cmd:stream",
    ] {
        assert_eq!(
            resolve_npm_native(Path::new(path), "0.166.0")
                .unwrap_err()
                .code,
            "client_npm_path"
        );
    }
}

#[test]
fn text_reads_are_bounded_and_malformed_metadata_does_not_escape() {
    for target in [
        "shim",
        "manifest",
        "selector",
        "invalid_json",
        "invalid_utf8",
    ] {
        let fixture = Fixture::new(true);
        fixture.optional(false, "x64");
        let path = match target {
            "shim" => fixture.launcher.clone(),
            "selector" => fixture.package.join("bin/codex.js"),
            _ => fixture.package.join("package.json"),
        };
        let bytes = match target {
            "invalid_json" => b"{broken".to_vec(),
            "invalid_utf8" => vec![0xff],
            _ => vec![b'x'; 65_537],
        };
        fs::write(path, bytes).unwrap();
        fixture.refuses();
    }
}

#[cfg(unix)]
#[test]
fn symlinked_shim_package_manifest_selector_optional_and_native_are_rejected() {
    for target in [
        "shim", "package", "manifest", "selector", "optional", "native",
    ] {
        let fixture = Fixture::new(true);
        let native = fixture.optional(false, "x64");
        let path = match target {
            "shim" => fixture.launcher.clone(),
            "package" => fixture.package.clone(),
            "manifest" => fixture.package.join("package.json"),
            "selector" => fixture.package.join("bin/codex.js"),
            "optional" => fixture.modules.join("@openai/codex-win32-x64"),
            _ => native,
        };
        let outside = fixture.root.join("escaped-source");
        fs::rename(&path, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &path).unwrap();
        fixture.refuses();
    }
}

#[cfg(unix)]
#[test]
fn non_regular_input_cannot_block_open_on_a_fifo() {
    use std::os::unix::ffi::OsStrExt;
    let fixture = Fixture::new(true);
    fixture.optional(false, "x64");
    fs::remove_file(&fixture.launcher).unwrap();
    let path = std::ffi::CString::new(fixture.launcher.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    fixture.refuses();
}

#[test]
#[ignore = "requires explicit verified public package root; never executes Codex"]
fn actual_public_binary_matches_the_unmodified_official_js_selector() {
    use sha2::{Digest, Sha256};
    let public = PathBuf::from(
        std::env::var_os("OCX_TEST_CODEX_NPM_PUBLIC").expect("public package root required"),
    );
    let node =
        PathBuf::from(std::env::var_os("OCX_TEST_CODEX_NPM_NODE").expect("explicit Node required"));
    assert!(public.is_absolute() && node.is_absolute());
    let actual = public.join(format!("codex-win32-x64/vendor/{X64}/bin/codex.exe"));
    let mut sha = Sha256::new();
    {
        use std::io::Read;
        let mut file = fs::File::open(&actual).unwrap();
        let mut buffer = [0; 65_536];
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            sha.update(&buffer[..count]);
        }
    }
    assert_eq!(
        format!("{:x}", sha.finalize()),
        "9d563ee4e0441ff9d59d612013e0460ddf8e5709735c2d9f6b98b7ef8ca4eb42"
    );
    let mut results = Vec::new();
    for (local, nested, embedded) in [
        (true, false, false),
        (false, false, false),
        (true, true, false),
        (false, false, true),
    ] {
        let fixture = Fixture::new(local);
        let native = if embedded {
            fixture.embedded("x64")
        } else {
            fixture.optional(nested, "x64")
        };
        fs::remove_file(&native).unwrap();
        fs::hard_link(&actual, &native)
            .or_else(|_| fs::copy(&actual, &native).map(|_| ()))
            .unwrap();
        let resolved = fixture.resolve();
        assert_eq!(resolved, native);
        // Run the exact, unmodified public JS with only platform/arch and its
        // child_process spawn intercepted. No native program/tools are run.
        let probe = r#"import {pathToFileURL} from 'node:url';
import {EventEmitter} from 'node:events';
import childProcess from 'node:child_process';
import {syncBuiltinESMExports} from 'node:module';
Object.defineProperty(process,'platform',{value:'win32'});
Object.defineProperty(process,'arch',{value:'x64'});
process.execve=undefined;
childProcess.spawn=(file)=>{console.log(JSON.stringify({selected:file}));const child=new EventEmitter();queueMicrotask(()=>child.emit('exit',0,null));return child;};
syncBuiltinESMExports();
await import(pathToFileURL(process.argv[1]).href);"#;
        let output = std::process::Command::new(&node)
            .args(["--input-type=module", "--eval", probe])
            .arg(fixture.package.join("bin/codex.js"))
            .env_clear()
            .env("HOME", &fixture.root)
            .env("PATH", "/usr/bin:/bin")
            .current_dir(&fixture.root)
            .output()
            .unwrap();
        assert!(output.status.success(), "selector probe failed");
        let selection: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            PathBuf::from(selection["selected"].as_str().unwrap()),
            resolved
        );
        results.push(json!({"local":local,"nested":nested,"embedded":embedded,"officialJsSelectedSameNative":true,"actualNativeSha256":"9d563ee4e0441ff9d59d612013e0460ddf8e5709735c2d9f6b98b7ef8ca4eb42"}));
    }
    if let Some(evidence) = std::env::var_os("OCX_TEST_CODEX_NPM_EVIDENCE") {
        fs::write(PathBuf::from(evidence).join("actual-public-selector.json"), serde_json::to_vec_pretty(&json!({"passed":true,"nativeExecuted":false,"platform":"win32","arch":"x64","cases":results})).unwrap()).unwrap();
    }
}
