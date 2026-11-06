//! Offline provisioning of build_desktop_runtime_bundle.py's flat runtime tree.
//! Only a descriptor digest pinned in the signed Desktop application is trusted.
//! Never executes Node, npm, package scripts, or accesses HOME/network.
use super::{
    archive, install, now_rfc3339, protection, RuntimeSourceKind, RuntimeSourceRecord,
    RuntimeStore, MANAGED_ENTRY_RELATIVE, MANAGED_PREFIX_RELATIVE, MANIFEST_FILENAME,
    OFFICIAL_PACKAGE,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, hash},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::fs::OpenOptions;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const MAX_UNPACKED: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;
const MAX_DESCRIPTOR: usize = 64 * 1024 * 1024;
const PENDING: &str = ".runtime-bundle-pending.json";
const FAILURE: &str = ".runtime-bundle-failure.json";
const OWNER: &str = ".ocxd-bundle-owner.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvisionResult {
    Installed { version: String },
    AlreadyInstalled { version: String },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleVerification {
    pub version: String,
    pub platform: String,
    pub entries: usize,
    pub unpacked_bytes: u64,
}
// Exact external builder interface: directories are implicit, not descriptor/tar entries.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Descriptor {
    schema_version: u32,
    platform: String,
    runtime_version: String,
    node_version: String,
    bun_version: String,
    runtime_archive_sha256: String,
    archive_name: String,
    archive_sha256: String,
    archive_bytes: u64,
    entries: Vec<BundleEntry>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleEntry {
    path: String,
    kind: String,
    bytes: Option<u64>,
    sha256: Option<String>,
    mode: Option<u32>,
    target: Option<String>,
}
struct Plan {
    descriptor: Descriptor,
    files: BTreeMap<String, BundleEntry>,
    directories: BTreeSet<String>,
    targets: BTreeMap<String, String>,
    total: u64,
}
fn fail(code: &str, detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn invalid(detail: impl Into<String>) -> AppError {
    fail("bundle_invalid", detail)
}
fn io(e: std::io::Error) -> AppError {
    fail("bundle_filesystem", e.to_string())
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn relative(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 4096
        && !s.contains(['\\', ':'])
        && !s.chars().any(char::is_control)
        && s.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with([' ', '.'])
                && !p.contains(['*', '?', '"', '<', '>', '|'])
                && !matches!(
                    p.split('.')
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
}
fn absolute(path: &Path) -> AppResult<()> {
    // Inspect spelling: Path::components normalizes embedded dot components away.
    if !path.is_absolute()
        || path.to_str().is_none()
        || path.to_string_lossy().chars().any(char::is_control)
        || path
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|s| s == "." || s == "..")
        || path
            .components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return Err(fail("bundle_paths", "需要无父遍历的绝对路径"));
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(_) => {
                if !safety::inspect(&current)?.is_dir() {
                    return Err(invalid("路径不是目录"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(e)),
        }
    }
    Ok(())
}
fn roots(i: &Path, d: &Path) -> AppResult<()> {
    absolute(i)?;
    absolute(d)?;
    let a = comparison_path(i)?;
    let b = comparison_path(d)?;
    let (a, b) = (a.trim_end_matches('/'), b.trim_end_matches('/'));
    if a == b || a.starts_with(&format!("{b}/")) || b.starts_with(&format!("{a}/")) {
        return Err(fail("bundle_paths", "安装根I与Desktop根D必须完全分离"));
    }
    Ok(())
}
fn comparison_path(path: &Path) -> AppResult<String> {
    // Canonicalize only existing ancestors, without creating missing I/D directories.
    // This catches repeated separators and Windows short-name/case aliases as well.
    let normalized: PathBuf = path.components().collect();
    let mut ancestor = normalized.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(
            ancestor
                .file_name()
                .ok_or_else(|| invalid("路径无父目录"))?,
        );
        ancestor = ancestor.parent().ok_or_else(|| invalid("路径无父目录"))?;
    }
    let mut result = ancestor.canonicalize().map_err(io)?;
    for part in suffix.into_iter().rev() {
        result.push(part);
    }
    Ok(result.to_string_lossy().replace('\\', "/").to_lowercase())
}
fn mkdir(path: &Path) -> AppResult<()> {
    if path.exists() {
        return if safety::inspect(path)?.is_dir() {
            Ok(())
        } else {
            Err(invalid("目录冲突"))
        };
    }
    if let Some(parent) = path.parent() {
        mkdir(parent)?;
    }
    #[cfg(windows)]
    return crate::infrastructure::windows_private_create::create_private_dir_all(path, 0o700)
        .map_err(io);
    #[cfg(not(windows))]
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    #[cfg(not(windows))]
    match builder.create(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if safety::inspect(path)?.is_dir() {
                Ok(())
            } else {
                Err(invalid("目录冲突"))
            }
        }
        Err(e) => Err(io(e)),
    }
}
fn regular(path: &Path) -> AppResult<File> {
    let file = safety::open_file(path, false)?;
    safety::require_single_link(&file)?;
    Ok(file)
}
fn read(path: &Path, max: u64) -> AppResult<Vec<u8>> {
    let file = regular(path)?;
    if file.metadata().map_err(io)?.len() > max {
        return Err(invalid("文件超过读取上限"));
    }
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() as u64 > max {
        return Err(invalid("文件超过读取上限"));
    }
    Ok(bytes)
}
fn load(bytes: &[u8], expected: &str) -> AppResult<Plan> {
    if bytes.len() > MAX_DESCRIPTOR || !digest(expected) || hash::sha256_hex(bytes) != expected {
        return Err(fail(
            "bundle_descriptor_digest",
            "descriptor与应用固定摘要不符（须为小写裸64位）",
        ));
    }
    let d: Descriptor = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if d.schema_version != 1
        || !matches!(d.platform.as_str(), "darwin-arm64" | "win32-x64")
        || !digest(&d.runtime_archive_sha256)
        || !digest(&d.archive_sha256)
        || d.archive_bytes == 0
        || d.archive_bytes > MAX_ARCHIVE
        || !relative(&d.archive_name)
        || d.archive_name.contains('/')
        || [&d.runtime_version, &d.node_version, &d.bun_version]
            .iter()
            .any(|s| {
                s.is_empty()
                    || s.len() > 128
                    || !s
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b".-+_".contains(&c))
            })
        || d.entries.is_empty()
        || d.entries.len() > MAX_ENTRIES
    {
        return Err(invalid("descriptor元数据或数量/压缩体积上限无效"));
    }
    let mut files = BTreeMap::new();
    let mut directories = BTreeSet::from(["runtime".into()]);
    let mut folded = BTreeSet::new();
    let mut total = 0u64;
    for e in &d.entries {
        if !relative(&e.path)
            || !e.path.starts_with("runtime/")
            || e.path.split('/').count() > 128
            || e.path == format!("{MANAGED_PREFIX_RELATIVE}/{MANIFEST_FILENAME}")
            || e.path == format!("runtime/{OWNER}")
            || e.path.starts_with("runtime/bin/")
            || e.path == "runtime/bin"
            || !folded.insert(e.path.to_lowercase())
        {
            return Err(invalid(format!("非法/重复/部署保留路径: {}", e.path)));
        }
        match e.kind.as_str() {
            "file"
                if e.target.is_none()
                    && e.bytes.is_some_and(|n| n <= MAX_FILE)
                    && e.sha256.as_deref().is_some_and(digest)
                    && matches!(e.mode, Some(0o644 | 0o755)) =>
            {
                total = total
                    .checked_add(e.bytes.unwrap())
                    .ok_or_else(|| invalid("解包体积溢出"))?;
                if total > MAX_UNPACKED {
                    return Err(invalid("解包体积超过2GiB"));
                }
            }
            "symlink"
                if e.bytes.is_none()
                    && e.sha256.is_none()
                    && e.mode.is_none()
                    && e.target.is_some() => {}
            _ => return Err(invalid("条目类型/摘要/模式/单文件上限无效")),
        }
        let mut parent = Path::new(&e.path).parent();
        while let Some(p) = parent.filter(|p| !p.as_os_str().is_empty()) {
            directories.insert(p.to_str().unwrap().to_string());
            parent = p.parent();
        }
        files.insert(e.path.clone(), e.clone());
    }
    if files.keys().any(|p| directories.contains(p)) {
        return Err(invalid("文件/链接不能是其它条目的父目录"));
    }
    if files.len() + directories.len() > MAX_ENTRIES {
        return Err(invalid("包含隐式目录的文件树数量超过100000"));
    }
    let mut spellings = BTreeMap::new();
    for p in files.keys().chain(directories.iter()) {
        if spellings
            .insert(p.to_lowercase(), p)
            .is_some_and(|old| old != p)
        {
            return Err(invalid("大小写折叠路径冲突"));
        }
    }
    let mut plan = Plan {
        descriptor: d,
        files,
        directories,
        targets: BTreeMap::new(),
        total,
    };
    for e in plan.files.values().filter(|e| e.kind == "symlink") {
        let target = resolve_target(
            &plan.files,
            &plan.directories,
            &e.path,
            &mut BTreeSet::new(),
        )?;
        if plan.descriptor.platform == "win32-x64"
            && plan.files.get(&target).is_none_or(|e| e.kind != "file")
        {
            return Err(invalid("Windows仅接受经过摘要验证的普通文件symlink映射"));
        }
        plan.targets.insert(e.path.clone(), target);
    }
    let (node, npm, bun) = dependency_names(&plan.descriptor.platform);
    for p in [
        node,
        npm,
        bun,
        "runtime/opencodex/package.json",
        "runtime/opencodex/package-lock.json",
        "runtime/opencodex/node_modules/@bitkyc08/opencodex/package.json",
    ] {
        if plan.files.get(p).is_none_or(|e| e.kind != "file") {
            return Err(invalid(format!("缺少完整树固定文件: {p}")));
        }
    }
    for p in [node, bun] {
        if plan.descriptor.platform != "win32-x64" && plan.files[p].mode != Some(0o755) {
            return Err(invalid("Node/Bun必须可执行"));
        }
    }
    let source = format!(
        "runtime/opencodex/sources/opencodex-{}.tgz",
        plan.descriptor.runtime_version
    );
    if !plan.files.get(&source).is_some_and(|e| {
        e.kind == "file" && e.sha256.as_deref() == Some(&plan.descriptor.runtime_archive_sha256)
    }) {
        return Err(invalid("维护tgz必须存在，且与runtimeArchiveSha256一致"));
    }
    Ok(plan)
}
fn dependency_names(platform: &str) -> (&'static str, &'static str, &'static str) {
    if platform == "win32-x64" {
        (
            "runtime/node/node.exe",
            "runtime/node/node_modules/npm/bin/npm-cli.js",
            "runtime/bun/bun.exe",
        )
    } else {
        (
            "runtime/node/bin/node",
            "runtime/node/lib/node_modules/npm/bin/npm-cli.js",
            "runtime/bun/bun",
        )
    }
}
fn resolve_target(
    files: &BTreeMap<String, BundleEntry>,
    directories: &BTreeSet<String>,
    name: &str,
    visiting: &mut BTreeSet<String>,
) -> AppResult<String> {
    if visiting.len() >= 128 || !visiting.insert(name.into()) {
        return Err(invalid("symlink循环/深度超限"));
    }
    let target = files[name]
        .target
        .as_deref()
        .ok_or_else(|| invalid("缺少symlink目标"))?;
    if target.is_empty()
        || target.len() > 4096
        || target.starts_with('/')
        || target.contains(['\\', ':'])
        || target.chars().any(char::is_control)
    {
        return Err(invalid("非法symlink目标"));
    }
    let mut parts: Vec<&str> = name.split('/').collect();
    parts.pop();
    for part in target.split('/') {
        // Never collapse a link/file followed by /..: kernel traversal differs.
        if !directories.contains(&parts.join("/")) {
            return Err(invalid("symlink目标父分量不是清单内目录"));
        }
        match part {
            "" | "." => return Err(invalid("非规范symlink目标")),
            ".." => {
                if parts.len() <= 1 {
                    return Err(invalid("symlink越界"));
                }
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    let resolved = parts.join("/");
    if !relative(&resolved) || !(files.contains_key(&resolved) || directories.contains(&resolved)) {
        return Err(invalid("symlink必须指向runtime内清单文件/隐式目录"));
    }
    // Descriptor admission forbids any file/link from being another entry's ancestor.
    let result = if files.get(&resolved).is_some_and(|e| e.kind == "symlink") {
        resolve_target(files, directories, &resolved, visiting)?
    } else {
        resolved
    };
    visiting.remove(name);
    Ok(result)
}

fn check_archive(path: &Path, plan: &Plan) -> AppResult<()> {
    if regular(path)?.metadata().map_err(io)?.len() != plan.descriptor.archive_bytes {
        return Err(invalid("压缩归档体积不符"));
    }
    if hash::sha256_file(path).map_err(io)? != plan.descriptor.archive_sha256 {
        return Err(fail(
            "bundle_archive_digest",
            "归档与已固定descriptor摘要不符",
        ));
    }
    Ok(())
}
/// Read-only verification; expected digest is a lowercase bare 64-character SHA-256.
pub fn verify_bundle(
    archive: &Path,
    descriptor_bytes: &[u8],
    expected_descriptor_sha256: &str,
) -> AppResult<BundleVerification> {
    let plan = load(descriptor_bytes, expected_descriptor_sha256)?;
    check_archive(archive, &plan)?;
    unpack(archive, &plan, None)?;
    Ok(BundleVerification {
        version: plan.descriptor.runtime_version,
        platform: plan.descriptor.platform,
        entries: plan.files.len(),
        unpacked_bytes: plan.total,
    })
}
fn unpack(archive: &Path, plan: &Plan, destination: Option<&Path>) -> AppResult<()> {
    let source = regular(archive)?;
    let compressed = flate2::read::MultiGzDecoder::new(source.take(MAX_ARCHIVE + 1));
    // Hard bound includes the worst allowed header/padding overhead, not just file data.
    let cap = MAX_UNPACKED + MAX_ENTRIES as u64 * 1024 + 10240;
    let mut tar = tar::Archive::new(compressed.take(cap + 1));
    let mut seen = BTreeSet::new();
    for member in tar.entries().map_err(io)?.raw(true) {
        let mut member = member.map_err(io)?;
        let name = String::from_utf8(member.path_bytes().into_owned())
            .map_err(|_| invalid("tar路径非UTF8"))?;
        if !relative(&name) || !seen.insert(name.clone()) || seen.len() > MAX_ENTRIES {
            return Err(invalid("tar非法/重复路径或数量超限"));
        }
        let expected = plan
            .files
            .get(&name)
            .ok_or_else(|| invalid(format!("tar额外条目: {name}")))?;
        let kind = member.header().entry_type();
        if expected.kind == "file" {
            if !kind.is_file()
                || member.header().size().map_err(io)? != expected.bytes.unwrap()
                || member.header().mode().map_err(io)? != expected.mode.unwrap()
            {
                return Err(invalid(format!("tar类型/体积/模式不符: {name}")));
            }
            // The same existing hash primitive is used in deployment and read-only verification.
            // Only one bounded file is in memory; aggregate/count limits were checked before I/O.
            let mut bytes = Vec::new();
            (&mut member)
                .take(MAX_FILE + 1)
                .read_to_end(&mut bytes)
                .map_err(io)?;
            if bytes.len() as u64 != expected.bytes.unwrap()
                || hash::sha256_hex(&bytes) != *expected.sha256.as_ref().unwrap()
            {
                return Err(invalid(format!("tar文件摘要/体积不符: {name}")));
            }
            if name
                == format!(
                    "{MANAGED_PREFIX_RELATIVE}/{}/package.json",
                    install::PACKAGE_SUBPATH
                )
            {
                validate_package(&bytes, plan)?;
            }
            if let Some(root) = destination {
                let path = root.join(&name);
                mkdir(path.parent().unwrap())?;
                let mut file =
                    crate::infrastructure::platform::create_new_private_file(&path, 0o600)
                        .map_err(io)?;
                file.write_all(&bytes).map_err(io)?;
                file.sync_all().map_err(io)?;
                drop(file);
                crate::infrastructure::platform::set_mode(&path, expected.mode.unwrap())
                    .map_err(io)?;
            }
        } else if !kind.is_symlink()
            || member.header().mode().map_err(io)? != 0o777
            || member.header().size().map_err(io)? != 0
            || member.link_name_bytes().as_deref() != expected.target.as_deref().map(str::as_bytes)
        {
            return Err(invalid("tar链接类型/目标不符（hardlink/special禁止）"));
        }
    }
    if seen.len() != plan.files.len() {
        return Err(invalid("tar缺少descriptor条目"));
    }
    // Consume gzip CRC and reject hidden concatenated tar entries after the end marker.
    let mut rest = tar.into_inner();
    let mut tail = [0u8; 8192];
    let mut tail_bytes = 0u64;
    loop {
        let n = rest.read(&mut tail).map_err(io)?;
        if n == 0 {
            break;
        }
        tail_bytes += n as u64;
        if tail_bytes > 10240 || tail[..n].iter().any(|b| *b != 0) {
            return Err(invalid("tar尾部隐藏数据/过量padding"));
        }
    }
    if rest.limit() == 0 {
        return Err(invalid("解压数据超限"));
    }
    if let Some(root) = destination {
        for (name, resolved) in &plan.targets {
            let path = root.join(name);
            mkdir(path.parent().unwrap())?;
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(plan.files[name].target.as_ref().unwrap(), path)
                    .map_err(io)?;
                let _ = resolved;
            }
            #[cfg(windows)]
            {
                // Validated regular-file mapping avoids symlink privileges/developer mode.
                let e = plan
                    .files
                    .get(resolved)
                    .filter(|e| e.kind == "file")
                    .ok_or_else(|| invalid("Windows不接受目录symlink映射"))?;
                let mut source = regular(&root.join(resolved))?;
                let mut target =
                    crate::infrastructure::platform::create_new_private_file(&path, 0o600)
                        .map_err(io)?;
                std::io::copy(&mut source, &mut target).map_err(io)?;
                target.sync_all().map_err(io)?;
                drop(target);
                crate::infrastructure::platform::set_mode(&path, e.mode.unwrap()).map_err(io)?;
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    schema_version: u32,
    attempt_id: String,
    stage: String,
    descriptor_sha256: String,
    phase: String,
    before: Option<String>,
    at: String,
}
fn save_pending(i: &Path, pending: &Pending) -> AppResult<()> {
    atomic_write(
        &i.join(PENDING),
        &serde_json::to_vec_pretty(pending).map_err(|e| invalid(e.to_string()))?,
        0o600,
    )?;
    safety::sync_directories(i, i)
}
// Enumerate only the explicitly selected runtime, without following symlinks.
fn tree_entries(runtime: &Path) -> AppResult<BTreeMap<String, String>> {
    if !safety::inspect(runtime)?.is_dir() {
        return Err(invalid("runtime不是目录"));
    }
    let i = runtime.parent().ok_or_else(|| invalid("runtime无父目录"))?;
    let mut stack = vec![runtime.to_path_buf()];
    let mut entries = BTreeMap::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).map_err(io)? {
            if entries.len() >= MAX_ENTRIES * 2 {
                return Err(invalid("已安装树数量超限"));
            }
            let path = entry.map_err(io)?.path();
            let name = path
                .strip_prefix(i)
                .unwrap()
                .to_str()
                .ok_or_else(|| invalid("非UTF8路径"))?
                .replace('\\', "/");
            if !relative(&name) {
                return Err(invalid("已安装树非法路径"));
            }
            let meta = fs::symlink_metadata(&path).map_err(io)?;
            #[cfg(windows)]
            safety::inspect(&path)?; // Reject every reparse point, even links.
            if meta.file_type().is_symlink() {
                entries.insert(name, "symlink".into());
            } else if safety::inspect(&path)?.is_dir() {
                entries.insert(name, "directory".into());
                stack.push(path);
            } else {
                regular(&path)?;
                entries.insert(name, "file".into());
            }
        }
    }
    Ok(entries)
}
fn verify_staged(stage: &Path, plan: &Plan) -> AppResult<()> {
    let actual = tree_entries(&stage.join("runtime"))?;
    if actual.len() + 1 != plan.files.len() + plan.directories.len() {
        return Err(invalid("staging磁盘文件树数量不符"));
    }
    for dir in &plan.directories {
        if dir != "runtime" && actual.get(dir).map(String::as_str) != Some("directory") {
            return Err(invalid("staging目录缺失/类型改变"));
        }
    }
    for (name, e) in &plan.files {
        let path = stage.join(name);
        if e.kind == "file" {
            if actual.get(name).map(String::as_str) != Some("file")
                || regular(&path)?.metadata().map_err(io)?.len() != e.bytes.unwrap()
                || hash::sha256_file(&path).map_err(io)? != *e.sha256.as_ref().unwrap()
            {
                return Err(invalid("staging文件磁盘读回不符"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if fs::metadata(&path).map_err(io)?.permissions().mode() & 0o7777 != e.mode.unwrap()
                {
                    return Err(invalid("staging文件模式读回不符"));
                }
            }
        } else {
            #[cfg(unix)]
            if actual.get(name).map(String::as_str) != Some("symlink")
                || fs::read_link(&path).map_err(io)?.to_str() != e.target.as_deref()
            {
                return Err(invalid("staging链接读回不符"));
            }
            #[cfg(windows)]
            {
                let target = &plan.files[&plan.targets[name]];
                if actual.get(name).map(String::as_str) != Some("file")
                    || hash::sha256_file(&path).map_err(io)? != *target.sha256.as_ref().unwrap()
                {
                    return Err(invalid("staging链接安全映射读回不符"));
                }
            }
        }
    }
    // Every ordinary file has already been synced. Persist all created directory entries.
    for dir in plan.directories.iter().rev() {
        safety::sync_directories(stage, &stage.join(dir))?;
    }
    Ok(())
}
fn validate_tree(runtime: &Path) -> AppResult<()> {
    let entries = tree_entries(runtime)?;
    // Reuse the virtual link resolver before any symlink is followed by install validation.
    let mut files = BTreeMap::new();
    let mut directories = BTreeSet::from(["runtime".into()]);
    for (name, kind) in &entries {
        if kind == "directory" {
            directories.insert(name.clone());
            continue;
        }
        let target = if kind == "symlink" {
            Some(
                fs::read_link(runtime.parent().unwrap().join(name))
                    .map_err(io)?
                    .to_str()
                    .ok_or_else(|| invalid("symlink目标非UTF8"))?
                    .to_string(),
            )
        } else {
            None
        };
        files.insert(
            name.clone(),
            BundleEntry {
                path: name.clone(),
                kind: kind.clone(),
                target,
                bytes: None,
                sha256: None,
                mode: None,
            },
        );
    }
    for (name, kind) in &entries {
        if kind == "symlink" {
            resolve_target(&files, &directories, name, &mut BTreeSet::new())?;
        }
    }
    Ok(())
}
fn existing(i: &Path) -> AppResult<String> {
    validate_tree(&i.join("runtime"))?;
    let (node, _) = install::managed_dependency_paths(i);
    install::validate_managed_installation(i, Some(&node)).map_err(AppError::from)?;
    Ok(install::resolve_managed_installation(i)
        .map_err(AppError::from)?
        .package_version)
}
fn persist(d: &Path, i: &Path, version: &str) -> AppResult<()> {
    absolute(d)?;
    mkdir(d)?;
    let store = RuntimeStore::new(d);
    let parent = store.path().parent().unwrap();
    safety::check_path(d, parent, true)?;
    mkdir(parent)?;
    safety::check_path(d, store.path(), true)?;
    // RuntimeStore::load falls back for corrupt/unknown records; do not erase that evidence.
    let mut record = if fs::symlink_metadata(store.path()).is_ok() {
        let record: RuntimeSourceRecord = serde_json::from_slice(&read(store.path(), 1024 * 1024)?)
            .map_err(|_| fail("bundle_source_record", "已有runtime.json损坏，保留原记录"))?;
        if record.schema_version != super::RUNTIME_SCHEMA_VERSION {
            return Err(invalid("runtime.json结构版本未知"));
        }
        record
    } else {
        RuntimeSourceRecord::default()
    };
    record.source = RuntimeSourceKind::Managed;
    record.path = Some(
        i.join(MANAGED_ENTRY_RELATIVE)
            .to_string_lossy()
            .into_owned(),
    );
    record.resolved_version = Some(version.into());
    record.resolved_at = Some(now_rfc3339());
    store.save(&record)?;
    let reread: RuntimeSourceRecord = serde_json::from_slice(&read(store.path(), 1024 * 1024)?)
        .map_err(|e| invalid(e.to_string()))?;
    if reread != record {
        return Err(invalid("runtime.json写后读回不符"));
    }
    Ok(())
}

/// Install only into an absent I/runtime. An existing version is never replaced/downgraded.
/// Digest must be lowercase bare SHA-256, supplied from the signed application's pin.
pub fn provision(
    installation_root: &Path,
    desktop_root: &Path,
    archive: &Path,
    descriptor_bytes: &[u8],
    expected_descriptor_sha256: &str,
) -> AppResult<ProvisionResult> {
    provision_inner(
        installation_root,
        desktop_root,
        archive,
        descriptor_bytes,
        expected_descriptor_sha256,
        false,
    )
}
fn provision_inner(
    i: &Path,
    d: &Path,
    archive: &Path,
    descriptor_bytes: &[u8],
    expected: &str,
    inject_activation_failure: bool,
) -> AppResult<ProvisionResult> {
    roots(i, d)?;
    // A second user may validate a read-only installed I without acquiring a write lock.
    if fs::symlink_metadata(i.join(PENDING)).is_ok() {
        return pending_error(i);
    }
    if fs::symlink_metadata(i.join("runtime")).is_ok() {
        let version = existing(i).map_err(|e| {
            fail(
                "bundle_existing_recovery",
                format!("已有runtime未覆盖；需要显式恢复: {e}"),
            )
        })?;
        persist(d, i, &version)?;
        return Ok(ProvisionResult::AlreadyInstalled { version });
    }
    let plan = load(descriptor_bytes, expected)?;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let platform = "darwin-arm64";
    #[cfg(all(windows, target_arch = "x86_64"))]
    let platform = "win32-x64";
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(windows, target_arch = "x86_64")
    )))]
    let platform = "unsupported";
    if plan.descriptor.platform != platform {
        return Err(invalid("bundle平台与当前程序平台不一致"));
    }
    mkdir(i)?;
    let lock_path = i.join(".runtime-bundle.lock");
    safety::check_path(i, &lock_path, true)?;
    let lock = safety::open_file(&lock_path, true)?;
    safety::require_single_link(&lock)?;
    lock.try_lock()
        .map_err(|_| fail("bundle_locked", "该安装根正在首装，请稍后重试"))?;
    if fs::symlink_metadata(i.join(PENDING)).is_ok() {
        return pending_error(i);
    }
    if fs::symlink_metadata(i.join("runtime")).is_ok() {
        let version = existing(i)?;
        persist(d, i, &version)?;
        return Ok(ProvisionResult::AlreadyInstalled { version });
    }
    let stage = tempfile::Builder::new()
        .prefix(".runtime-bundle-stage-")
        .tempdir_in(i)
        .map_err(io)?;
    let attempt = uuid::Uuid::new_v4().to_string();
    let mut pending = Pending {
        schema_version: 1,
        attempt_id: attempt.clone(),
        stage: stage.path().file_name().unwrap().to_str().unwrap().into(),
        descriptor_sha256: expected.into(),
        phase: "verifying".into(),
        before: None,
        at: now_rfc3339(),
    };
    save_pending(i, &pending)?;
    let mut published = false;
    let mut identity = None;
    let owner = serde_json::to_vec(&pending).map_err(|e| invalid(e.to_string()))?;
    let mut generated = BTreeMap::from([(format!("runtime/{OWNER}"), owner.clone())]);
    let result = (|| -> AppResult<String> {
        // Snapshot through no-follow/single-link handle; enforce cap before hashing it.
        let source = regular(archive)?;
        if source.metadata().map_err(io)?.len() != plan.descriptor.archive_bytes {
            return Err(invalid("归档体积不符"));
        }
        let copy = stage.path().join("archive.tar.gz");
        let mut output =
            crate::infrastructure::platform::create_new_private_file(&copy, 0o600).map_err(io)?;
        let n = std::io::copy(&mut source.take(MAX_ARCHIVE + 1), &mut output).map_err(io)?;
        if n != plan.descriptor.archive_bytes {
            return Err(invalid("归档超限/读取期间改变"));
        }
        output.sync_all().map_err(io)?;
        drop(output);
        check_archive(&copy, &plan)?;
        unpack(&copy, &plan, Some(stage.path()))?;
        verify_staged(stage.path(), &plan)?;
        let bin = package_bin(stage.path(), &plan)?;
        let staged_runtime = stage.path().join("runtime");
        atomic_write(&staged_runtime.join(OWNER), &owner, 0o600)?;
        identity = Some(directory_identity(&staged_runtime)?);
        pending.phase = "publishing".into();
        save_pending(i, &pending)?;
        // Atomic no-replace also protects an unknown EMPTY directory appearing after checks.
        publish(&staged_runtime, &i.join("runtime"))?;
        published = true;
        pending.phase = "activating".into();
        save_pending(i, &pending)?;
        let (node, npm) = install::managed_dependency_paths(i);
        let prefix = i.join(MANAGED_PREFIX_RELATIVE);
        let entry = i.join(MANAGED_ENTRY_RELATIVE);
        mkdir(entry.parent().unwrap())?;
        let script = prefix.join(install::PACKAGE_SUBPATH).join(&bin);
        let body = install::entry_script(Some(&node), &script).into_bytes();
        generated.insert(MANAGED_ENTRY_RELATIVE.into(), body.clone());
        atomic_write(&entry, &body, 0o755)?;
        if inject_activation_failure {
            return Err(fail("bundle_activation_injected", "测试注入激活失败"));
        }
        let proof = protection::activation_proof(&prefix, &attempt, &bin, &node)?;
        let manifest = install::RuntimeManifest {
            activation: Some(proof),
            package: OFFICIAL_PACKAGE.into(),
            version: plan.descriptor.runtime_version.clone(),
            tarball_sha256: plan.descriptor.runtime_archive_sha256.clone(),
            installed_at: now_rfc3339(),
            npm_path: npm.to_string_lossy().into_owned(),
            scripts_enabled: false,
            source: install::InstallSourceKind::Offline,
        };
        let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| invalid(e.to_string()))?;
        generated.insert(
            format!("{MANAGED_PREFIX_RELATIVE}/{MANIFEST_FILENAME}"),
            bytes.clone(),
        );
        atomic_write(&prefix.join(MANIFEST_FILENAME), &bytes, 0o600)?;
        let version = existing(i)?;
        if version != plan.descriptor.runtime_version {
            return Err(invalid("首装读回版本不符"));
        }
        safety::sync_directories(i, &prefix)?;
        Ok(version)
    })();
    match result {
        Ok(version) => {
            fs::remove_file(i.join(PENDING)).map_err(io)?;
            safety::sync_directories(i, i)?;
            // Program activation succeeded: a user-store error retains the valid program for retry.
            persist(d, i, &version)?;
            Ok(ProvisionResult::Installed { version })
        }
        Err(error) => {
            let compensated =
                !published || compensate(i, stage.path(), identity, &generated, &plan).is_ok();
            let failure = serde_json::json!({"schemaVersion":1,"attemptId":attempt,"before":null,
                "phase":pending.phase,"stage":pending.stage,"compensated":compensated,"error":error.to_string(),"at":now_rfc3339()});
            atomic_write(
                &i.join(FAILURE),
                &serde_json::to_vec_pretty(&failure).map_err(|e| invalid(e.to_string()))?,
                0o600,
            )?;
            if compensated {
                fs::remove_file(i.join(PENDING)).map_err(io)?;
            }
            Err(error)
        }
    }
}
fn package_bin(root: &Path, plan: &Plan) -> AppResult<String> {
    let package = root
        .join(MANAGED_PREFIX_RELATIVE)
        .join(install::PACKAGE_SUBPATH);
    validate_package(&read(&package.join("package.json"), 64 * 1024)?, plan)
}
fn validate_package(bytes: &[u8], plan: &Plan) -> AppResult<String> {
    if bytes.len() > 64 * 1024 {
        return Err(invalid("package.json超过受管校验上限"));
    }
    let json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if json["name"].as_str() != Some(OFFICIAL_PACKAGE)
        || json["version"].as_str() != Some(&plan.descriptor.runtime_version)
    {
        return Err(invalid("包名/version与descriptor不符"));
    }
    let bin = archive::resolve_bin(&json).ok_or_else(|| invalid("缺少package bin"))?;
    let bin = bin
        .trim()
        .strip_prefix("./")
        .unwrap_or(bin.trim())
        .to_string();
    let name = format!(
        "{MANAGED_PREFIX_RELATIVE}/{}/{bin}",
        install::PACKAGE_SUBPATH
    );
    if !relative(&bin)
        || !plan.files.get(&name).is_some_and(|e| {
            e.kind == "file" && (plan.descriptor.platform == "win32-x64" || e.mode == Some(0o755))
        })
    {
        return Err(invalid("package入口必须是清单内可执行普通文件"));
    }
    Ok(bin)
}
fn pending_error<T>(i: &Path) -> AppResult<T> {
    let raw = read(&i.join(PENDING), 64 * 1024)?;
    let p: Pending =
        serde_json::from_slice(&raw).map_err(|_| invalid("pending记录损坏；原样保留"))?;
    if p.schema_version != 1
        || uuid::Uuid::parse_str(&p.attempt_id).is_err()
        || !p.stage.starts_with(".runtime-bundle-stage-")
        || !relative(&p.stage)
        || p.stage.contains('/')
        || p.before.is_some()
        || !digest(&p.descriptor_sha256)
    {
        return Err(invalid("pending记录未知；原样保留"));
    }
    let stage = i.join(&p.stage);
    safety::check_path(i, &stage, true)?;
    Err(fail("bundle_pending_recovery", format!("首装中断，未作为已安装接纳；阶段={}，本机stage={}，stage存在={}。需显式恢复，未扫描其它目录", p.phase, stage.display(), stage.exists())))
}
fn compensate(
    i: &Path,
    stage: &Path,
    identity: Option<(u64, u64, u64)>,
    generated: &BTreeMap<String, Vec<u8>>,
    plan: &Plan,
) -> AppResult<()> {
    let runtime = i.join("runtime");
    safety::check_path(i, &runtime, false)?;
    if Some(directory_identity(&runtime)?) != identity {
        return Err(invalid("runtime身份改变，保留"));
    }
    if read(&runtime.join(OWNER), 64 * 1024)? != generated[&format!("runtime/{OWNER}")] {
        return Err(invalid("无法证明runtime为本次首装，保留"));
    }
    let entries = tree_entries(&runtime)?;
    for (name, kind) in entries {
        let path = i.join(&name);
        if kind == "directory" && (plan.directories.contains(&name) || name == "runtime/bin") {
            continue;
        }
        if let Some(bytes) = generated.get(&name) {
            if kind != "file" || read(&path, 128 * 1024)? != *bytes {
                return Err(invalid("激活文件改变，保留"));
            }
        } else if let Some(e) = plan.files.get(&name) {
            if e.kind == "file" {
                if kind != "file"
                    || hash::sha256_file(&path).map_err(io)? != *e.sha256.as_ref().unwrap()
                {
                    return Err(invalid("runtime包体改变，保留"));
                }
            } else {
                #[cfg(unix)]
                if kind != "symlink"
                    || fs::read_link(&path).map_err(io)?.to_str() != e.target.as_deref()
                {
                    return Err(invalid("runtime链接改变，保留"));
                }
                #[cfg(windows)]
                {
                    let target = &plan.files[&plan.targets[&name]];
                    if kind != "file"
                        || hash::sha256_file(&path).map_err(io)? != *target.sha256.as_ref().unwrap()
                    {
                        return Err(invalid("runtime链接映射改变，保留"));
                    }
                }
            }
        } else {
            return Err(invalid("runtime出现非本次内容，保留"));
        }
    }
    // Return to this call's still-owned stage; TempDir cleans only its own tree.
    publish(&runtime, &stage.join("runtime"))
}
fn directory_identity(path: &Path) -> AppResult<(u64, u64, u64)> {
    if !safety::inspect(path)?.is_dir() {
        return Err(invalid("不是目录"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let m = fs::symlink_metadata(path).map_err(io)?;
        Ok((m.dev(), m.ino(), 0))
    }
    #[cfg(windows)]
    {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        };
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(0x02200000)
            .open(path)
            .map_err(io)?;
        let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: live handle and writable output; read only after the API succeeds.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err(io(std::io::Error::last_os_error()));
        }
        let info = unsafe { info.assume_init() };
        Ok((
            info.dwVolumeSerialNumber as u64,
            info.nFileIndexHigh as u64,
            info.nFileIndexLow as u64,
        ))
    }
}
fn publish(source: &Path, target: &Path) -> AppResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let a = std::ffi::CString::new(source.as_os_str().as_bytes())
            .map_err(|_| invalid("路径含NUL"))?;
        let b = std::ffi::CString::new(target.as_os_str().as_bytes())
            .map_err(|_| invalid("路径含NUL"))?;
        #[cfg(target_os = "macos")]
        // SAFETY: NUL-terminated live paths; RENAME_EXCL prevents replacing any destination.
        let rc = unsafe { libc::renamex_np(a.as_ptr(), b.as_ptr(), libc::RENAME_EXCL) };
        #[cfg(not(target_os = "macos"))]
        let rc = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                a.as_ptr(),
                libc::AT_FDCWD,
                b.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if rc != 0 {
            return Err(io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        fs::rename(source, target).map_err(io)
    }
}

#[cfg(all(test, unix))]
#[path = "bundled_tests.rs"]
mod bundled_tests;
