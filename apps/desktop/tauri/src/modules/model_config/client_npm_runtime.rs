//! Read-only resolution of the two standard npm Codex shims. Never run a shell
//! or discover Node/PATH. Unknown package/launcher contracts fail closed.
use crate::modules::model_config::schema::{invalid, ModelError, ModelResult};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
};

// Public 0.166.0 npm archives, checked against run-41802736384's SHA-512
// integrities. Pin the entire selector, not a substring of arbitrary JavaScript.
const SELECTOR_SHA256: &str = "e14fc83739c13c1074b12bd1acce4010c893c7f6fbfc1593b4a31d74cabb57af";
const MAX_TEXT: u64 = 64 * 1024;
const MAX_ANCESTORS: usize = 64;

struct WindowsTarget {
    cpu: &'static str,
    triple: &'static str,
    machine: u16,
}

const TARGETS: [WindowsTarget; 2] = [
    WindowsTarget {
        cpu: "x64",
        triple: "x86_64-pc-windows-msvc",
        machine: 0x8664,
    },
    WindowsTarget {
        cpu: "arm64",
        triple: "aarch64-pc-windows-msvc",
        machine: 0xaa64,
    },
];

/// Resolve an already bound, absolute npm codex.cmd to the native executable
/// selected by its own verified @openai/codex package. Supports local
/// node_modules/.bin and Windows global-prefix shims, with one unambiguous
/// x64 optional package (nested/hoisted) or an embedded vendor fallback.
///
/// Production scope is Windows x64 with x64 Node. An adjacent node.exe is
/// checked; absent one, the caller must maintain that supported host contract.
/// ARM64 and multiple Windows targets are refused, never inferred from PATH.
/// The selector contract is pinned, not the release version or EXE digest: a
/// newer stable package with the same selector and consistent versions works.
/// Unknown selector contracts fail closed. Original .exe bindings stay with the
/// existing owner. No redaction expansion, shell/Node execution, PATH lookup,
/// user configuration, network, directory mutation or secret reads occur here.
pub fn resolve_npm_native(launcher: &Path, expected_version: &str) -> ModelResult<PathBuf> {
    if !stable_version(expected_version) {
        return Err(failure("version"));
    }
    if cfg!(all(windows, not(target_arch = "x86_64"))) {
        return Err(failure("platform"));
    }
    let launcher = checked_path(launcher)?;
    if launcher.file_name() != Some(std::ffi::OsStr::new("codex.cmd")) {
        return Err(failure("shim"));
    }
    let parent = launcher.parent().ok_or_else(|| failure("path"))?;
    let local = parent.file_name() == Some(std::ffi::OsStr::new(".bin"));
    let modules = if local {
        let modules = parent.parent().ok_or_else(|| failure("path"))?;
        if modules.file_name() != Some(std::ffi::OsStr::new("node_modules")) {
            return Err(failure("shim"));
        }
        modules.to_path_buf()
    } else {
        parent.join("node_modules")
    };
    let shim = text(&launcher)?;
    if shim.replace("\r\n", "\n") != standard_shim(local) {
        return Err(failure("shim"));
    }
    let package = checked_path(&modules.join("@openai/codex"))?;
    let manifest = json(&package.join("package.json"))?;
    if manifest["name"] != "@openai/codex"
        || manifest["version"] != expected_version
        || manifest["type"] != "module"
        || manifest["bin"] != serde_json::json!({"codex":"bin/codex.js"})
        || manifest.get("exports").is_some()
    {
        return Err(failure("package"));
    }
    let selector = text(&package.join("bin/codex.js"))?;
    if format!("{:x}", Sha256::digest(selector.as_bytes())) != SELECTOR_SHA256 {
        return Err(failure("selector"));
    }
    for target in &TARGETS {
        let alias = format!("@openai/codex-win32-{}", target.cpu);
        if manifest["optionalDependencies"][&alias]
            != format!("npm:@openai/codex@{expected_version}-win32-{}", target.cpu)
        {
            return Err(failure("package"));
        }
    }

    // createRequire(import.meta.url) searches ancestor node_modules in this
    // order. Account for every filesystem candidate before considering the
    // embedded fallback, and refuse ambiguous/shadowing installations.
    let mut optional = Vec::new();
    let search_start = package.join("bin");
    for (depth, ancestor) in search_start.ancestors().enumerate() {
        if depth >= MAX_ANCESTORS {
            return Err(failure("path"));
        }
        if ancestor.file_name() == Some(std::ffi::OsStr::new("node_modules")) {
            continue;
        }
        for (index, target) in TARGETS.iter().enumerate() {
            let candidate =
                ancestor.join(format!("node_modules/@openai/codex-win32-{}", target.cpu));
            if present(&candidate)? {
                let nested =
                    package.join(format!("node_modules/@openai/codex-win32-{}", target.cpu));
                let hoisted = modules.join(format!("@openai/codex-win32-{}", target.cpu));
                if candidate != nested && candidate != hoisted {
                    return Err(failure("boundary"));
                }
                optional.push((checked_path(&candidate)?, index));
            }
        }
    }
    if optional.len() > 1 {
        return Err(failure("ambiguous"));
    }
    let (native, target) = if let Some((optional, index)) = optional.pop() {
        let target = &TARGETS[index];
        if target.cpu != "x64" {
            return Err(failure("platform"));
        }
        let metadata = json(&optional.join("package.json"))?;
        if metadata["name"] != "@openai/codex"
            || metadata["version"] != format!("{expected_version}-win32-{}", target.cpu)
            || metadata["os"] != serde_json::json!(["win32"])
            || metadata["cpu"] != serde_json::json!([target.cpu])
            || metadata.get("exports").is_some()
        {
            return Err(failure("package"));
        }
        // If require.resolve succeeds, the official selector does not fall
        // back when that package has no binary. Preserve this failure behavior.
        (
            optional.join(format!("vendor/{}/bin/codex.exe", target.triple)),
            target,
        )
    } else {
        let mut embedded = Vec::new();
        for target in &TARGETS {
            let native = package.join(format!("vendor/{}/bin/codex.exe", target.triple));
            if present(&native)? {
                embedded.push((native, target));
            }
        }
        if embedded.len() > 1 {
            return Err(failure("ambiguous"));
        }
        let selected = embedded.pop().ok_or_else(|| failure("native"))?;
        if selected.1.cpu != "x64" {
            return Err(failure("platform"));
        }
        selected
    };
    let native = checked_path(&native)?;
    if machine(&native)? != target.machine {
        return Err(failure("native"));
    }
    // npm prefers an adjacent node.exe when it exists. Verify its architecture
    // rather than assuming the Rust process and Node have the same target.
    let node = parent.join("node.exe");
    if present(&node)? && machine(&checked_path(&node)?)? != target.machine {
        return Err(failure("ambiguous"));
    }
    Ok(native)
}

fn failure(reason: &str) -> ModelError {
    invalid(
        &format!("client_npm_{reason}"),
        "无法核验当前 npm Codex 入口对应的原生运行文件；请检查官方安装后重试",
    )
}

fn present(path: &Path) -> ModelResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(failure("path")),
    }
}

/// Reject every symlink/junction/reparse component; canonicalization alone
/// would turn an escaped package into an apparently valid different package.
fn checked_path(path: &Path) -> ModelResult<PathBuf> {
    if !path.is_absolute()
        || path.components().count() > MAX_ANCESTORS
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(failure("path"));
    }
    #[cfg(windows)]
    {
        // The supported npm install is on a local disk. Refuse network/device
        // namespaces and NTFS alternate streams before any filesystem access.
        if !matches!(path.components().next(), Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
            || path.components().any(|part| matches!(part, Component::Normal(name) if name.to_string_lossy().contains(':')))
        {
            return Err(failure("path"));
        }
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        let metadata = fs::symlink_metadata(&current).map_err(|_| failure("path"))?;
        if metadata.file_type().is_symlink() {
            return Err(failure("boundary"));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err(failure("boundary"));
            }
        }
    }
    dunce::canonicalize(path).map_err(|_| failure("path"))
}

fn text(path: &Path) -> ModelResult<String> {
    let path = checked_path(path)?;
    if !fs::metadata(&path).map_err(|_| failure("read"))?.is_file() {
        return Err(failure("read"));
    }
    let file = File::open(path).map_err(|_| failure("read"))?;
    if !file.metadata().map_err(|_| failure("read"))?.is_file() {
        return Err(failure("read"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_TEXT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure("read"))?;
    if bytes.len() as u64 > MAX_TEXT {
        return Err(failure("limit"));
    }
    String::from_utf8(bytes).map_err(|_| failure("read"))
}

fn json(path: &Path) -> ModelResult<Value> {
    serde_json::from_str(&text(path)?).map_err(|_| failure("package"))
}

fn machine(path: &Path) -> ModelResult<u16> {
    let metadata = fs::metadata(path).map_err(|_| failure("native"))?;
    if !metadata.is_file() || metadata.len() < 512 {
        return Err(failure("native"));
    }
    let mut file = File::open(path).map_err(|_| failure("native"))?;
    let mut header = [0; 64];
    file.read_exact(&mut header)
        .map_err(|_| failure("native"))?;
    if &header[..2] != b"MZ" {
        return Err(failure("native"));
    }
    let offset = u32::from_le_bytes(header[60..64].try_into().expect("four bytes"));
    if !(64..=64 * 1024).contains(&offset) {
        return Err(failure("native"));
    }
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|_| failure("native"))?;
    let mut coff = [0; 26];
    file.read_exact(&mut coff).map_err(|_| failure("native"))?;
    let sections = u16::from_le_bytes([coff[6], coff[7]]);
    let optional_size = u16::from_le_bytes([coff[20], coff[21]]);
    let flags = u16::from_le_bytes([coff[22], coff[23]]);
    if &coff[..4] != b"PE\0\0"
        || !(1..=96).contains(&sections)
        || !(112..=4096).contains(&optional_size)
        || flags & 0x2002 != 0x0002
        || u16::from_le_bytes([coff[24], coff[25]]) != 0x20b
        || offset as u64 + 24 + optional_size as u64 + sections as u64 * 40 > metadata.len()
    {
        return Err(failure("native"));
    }
    Ok(u16::from_le_bytes([coff[4], coff[5]]))
}

fn stable_version(version: &str) -> bool {
    if version.len() > 64 {
        return false;
    }
    let parts: Vec<_> = version.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part.len() == 1 || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u64>().is_ok()
        })
}

fn standard_shim(local: bool) -> String {
    let target = if local {
        "..\\@openai\\codex\\bin\\codex.js"
    } else {
        "node_modules\\@openai\\codex\\bin\\codex.js"
    };
    format!(
        concat!(
            "@ECHO off\nGOTO start\n:find_dp0\nSET dp0=%~dp0\nEXIT /b\n:start\n",
            "SETLOCAL\nCALL :find_dp0\n\nIF EXIST \"%dp0%\\node.exe\" (\n",
            "  SET \"_prog=%dp0%\\node.exe\"\n) ELSE (\n  SET \"_prog=node\"\n",
            "  SET PATHEXT=%PATHEXT:;.JS;=;%\n)\n\n",
            "endLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & ",
            "\"%_prog%\"  \"%dp0%\\{}\" %*\n"
        ),
        target
    )
}
