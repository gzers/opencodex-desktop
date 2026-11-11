//! Read one explicitly selected, immutable runtime image. Missing selection is
//! distinct from damaged selection; neither scans old programs or executes npm.
use super::{
    install,
    program_layout::{RuntimeVersionPaths, CURRENT_FILENAME},
    OFFICIAL_PACKAGE,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::hash,
    modules::{backup::safety, data_root::layout::Platform},
};
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

pub const IMAGE_MANIFEST: &str = ".runtime-image.json";
pub const ACTIVATION_PENDING: &str = ".selection-pending.json";
const MAX_METADATA: u64 = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentSelection {
    pub schema_version: u32,
    pub version: String,
    pub platform: String,
    pub image_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageManifest {
    pub schema_version: u32,
    pub package: String,
    pub version: String,
    pub platform: String,
    pub official_archive_sha256: String,
    pub tree_sha256: String,
    pub bin: String,
    pub package_sha256: String,
    pub bin_sha256: String,
    pub node_sha256: String,
    pub npm_sha256: String,
    pub bun_sha256: String,
    pub bun_launcher_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedProgram {
    pub selection: CurrentSelection,
    pub image: ImageManifest,
    pub paths: RuntimeVersionPaths,
    pub stable_entry: PathBuf,
    pub script: PathBuf,
}

/// Bounded freshness check for a previously fully admitted image. This does
/// not re-admit modified binaries or substitute for action-time full admission.
pub fn verify_current(program_root: &Path, selected: &SelectedProgram) -> AppResult<()> {
    if !root_present(program_root)? {
        return Err(invalid("已选择的程序根已不可确认"));
    }
    let runtime = program_root.join("runtime");
    require_no_removal(program_root)?;
    let pending = runtime.join(ACTIVATION_PENDING);
    safety::check_path(program_root, &pending, true).map_err(|_| invalid("激活路径不可确认"))?;
    match std::fs::symlink_metadata(&pending) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        _ => return Err(invalid("版本激活尚未完成")),
    }
    let current: CurrentSelection =
        serde_json::from_slice(&metadata(program_root, &runtime.join(CURRENT_FILENAME))?)
            .map_err(|_| invalid("程序选择记录已变化"))?;
    if current != selected.selection
        || hash::sha256_hex(&metadata(
            program_root,
            &selected.paths.version_root.join(IMAGE_MANIFEST),
        )?) != selected.selection.image_sha256
    {
        return Err(invalid("所选程序代次已变化，需要重新核对"));
    }
    if metadata(program_root, &selected.stable_entry)?
        != install::entry_script(Some(&selected.paths.node), &selected.script).as_bytes()
    {
        return Err(invalid("稳定入口已变化，需要重新核对"));
    }
    Ok(())
}

fn invalid(detail: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_selection_invalid".into(),
        detail: detail.into(),
    }
}
fn require_no_removal(program_root: &Path) -> AppResult<()> {
    let path = program_root
        .join("runtime")
        .join(super::program_removal::PENDING);
    safety::check_path(program_root, &path, true)?;
    if !matches!(std::fs::symlink_metadata(path),Err(e) if e.kind()==std::io::ErrorKind::NotFound) {
        return Err(AppError::RuntimeManaged {
            code: "runtime_removal_pending".into(),
            detail: "原卸载操作尚未完成，请核对原操作；未重新安装或启动".into(),
        });
    }
    Ok(())
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn root_present(root: &Path) -> AppResult<bool> {
    if !super::program_layout::clean_absolute(root) {
        return Err(invalid("程序根路径无效"));
    }
    let mut path = PathBuf::new();
    for component in root.components() {
        path.push(component);
        #[cfg(windows)]
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(_) => return Err(invalid("程序根状态不可读")),
            Ok(_) => {
                if !safety::inspect(&path)
                    .map_err(|_| invalid("程序根存在重定向或未知对象"))?
                    .is_dir()
                {
                    return Err(invalid("程序根不是目录"));
                }
            }
        }
    }
    Ok(true)
}
fn metadata(root: &Path, path: &Path) -> AppResult<Vec<u8>> {
    safety::check_path(root, path, false).map_err(|_| invalid("运行程序记录路径不可信"))?;
    let file = safety::open_file(path, false).map_err(|_| invalid("无法读取运行程序记录"))?;
    safety::require_single_link(&file).map_err(|_| invalid("运行程序记录不能共享文件链接"))?;
    let mut bytes = Vec::new();
    file.take(MAX_METADATA + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid("读取运行程序记录失败"))?;
    if bytes.len() as u64 > MAX_METADATA {
        return Err(invalid("运行程序记录超过上限"));
    }
    Ok(bytes)
}
fn file_digest(root: &Path, path: &Path) -> AppResult<String> {
    use sha2::{Digest, Sha256};
    safety::check_path(root, path, false).map_err(|_| invalid("运行程序文件路径不可信"))?;
    let mut file = safety::open_file(path, false).map_err(|_| invalid("运行程序文件不可读"))?;
    safety::require_single_link(&file).map_err(|_| invalid("运行程序文件不能共享链接"))?;
    if file
        .metadata()
        .map_err(|_| invalid("运行程序文件信息不可读"))?
        .len()
        > 512 * 1024 * 1024
    {
        return Err(invalid("运行程序文件超过上限"));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    let mut total = 0_u64;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| invalid("运行程序文件读取失败"))?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > 512 * 1024 * 1024 {
            return Err(invalid("运行程序读取期间超过上限"));
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

/// Validate against the host's selected target, never against a guessed record
/// or last observed version. The installer owns publication and rollback.
pub fn read(
    program_root: &Path,
    platform: Platform,
    target: &str,
) -> AppResult<Option<SelectedProgram>> {
    read_inner(program_root, platform, target, false)
}

/// Installer-only final readback while its own serialized journal is present.
/// Ordinary runtime/status consumers must use read(), which refuses that state.
pub(super) fn read_activating(
    program_root: &Path,
    platform: Platform,
    target: &str,
) -> AppResult<Option<SelectedProgram>> {
    read_inner(program_root, platform, target, true)
}

fn read_inner(
    program_root: &Path,
    platform: Platform,
    target: &str,
    activating: bool,
) -> AppResult<Option<SelectedProgram>> {
    let expected_prefix = if platform == Platform::Windows {
        "win32-"
    } else if platform == Platform::Macos {
        "darwin-"
    } else {
        "linux-"
    };
    if !target.starts_with(expected_prefix)
        || !["darwin-arm64", "win32-x64", "linux-x64", "linux-arm64"].contains(&target)
    {
        return Err(invalid("运行程序平台不支持"));
    }
    if !root_present(program_root)? {
        return Ok(None);
    }
    require_no_removal(program_root)?;
    let runtime = program_root.join("runtime");
    if !activating {
        let pending = runtime.join(ACTIVATION_PENDING);
        safety::check_path(program_root, &pending, true)
            .map_err(|_| invalid("版本激活状态路径不可信"))?;
        match std::fs::symlink_metadata(&pending) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => {
                return Err(AppError::RuntimeManaged {
                    code: "runtime_selection_pending".into(),
                    detail: "运行程序版本激活尚未完成，请核对恢复；未采用其它来源".into(),
                })
            }
        }
    }
    let current_path = runtime.join(CURRENT_FILENAME);
    safety::check_path(program_root, &current_path, true)
        .map_err(|_| invalid("运行程序选择路径不可信"))?;
    match std::fs::symlink_metadata(&current_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(invalid("运行程序选择状态不可读")),
        Ok(_) => {}
    }
    let current_bytes = metadata(program_root, &current_path)?;
    let selection: CurrentSelection =
        serde_json::from_slice(&current_bytes).map_err(|_| invalid("运行程序选择记录格式无效"))?;
    if selection.schema_version != 1
        || selection.platform != target
        || !digest(&selection.image_sha256)
    {
        return Err(invalid("运行程序选择记录身份不符"));
    }
    let selected = read_image(program_root, platform, target, &selection)?;
    let image_bytes = metadata(
        program_root,
        &selected.paths.version_root.join(IMAGE_MANIFEST),
    )?;
    if hash::sha256_hex(&image_bytes) != selection.image_sha256 {
        return Err(invalid("校验期间版本清单已变化"));
    }
    let paths = &selected.paths;
    let script = &selected.script;
    let stable_entry = program_root.join(super::program_layout::runtime_entry_relative(platform));
    if metadata(program_root, &stable_entry)?
        != install::entry_script(Some(&paths.node), script).as_bytes()
    {
        return Err(invalid("稳定入口未绑定当前版本"));
    }
    super::paths::validate_executable(&stable_entry).map_err(|_| invalid("稳定入口不可执行"))?;
    if metadata(program_root, &current_path)? != current_bytes
        || metadata(program_root, &paths.version_root.join(IMAGE_MANIFEST))? != image_bytes
    {
        return Err(invalid("校验期间运行程序选择已变化"));
    }
    Ok(Some(selected))
}

/// Internal installer admission of a named immutable image; callers must have
/// established its source through the pinned bundle or verified install path.
/// This never discovers an arbitrary version or modifies the stable launcher.
pub(super) fn read_image(
    program_root: &Path,
    platform: Platform,
    target: &str,
    selection: &CurrentSelection,
) -> AppResult<SelectedProgram> {
    if !matches!(
        (platform, target),
        (Platform::Macos, "darwin-arm64")
            | (Platform::Windows, "win32-x64")
            | (Platform::Unix, "linux-x64" | "linux-arm64")
    ) || !root_present(program_root)?
        || selection.schema_version != 1
        || selection.platform != target
        || !digest(&selection.image_sha256)
    {
        return Err(invalid("候选版本身份或程序根不可确认"));
    }
    let paths = RuntimeVersionPaths::new(
        &program_root.join("runtime/versions"),
        &selection.version,
        platform,
    )?;
    let image_bytes = metadata(program_root, &paths.version_root.join(IMAGE_MANIFEST))?;
    if hash::sha256_hex(&image_bytes) != selection.image_sha256 {
        return Err(invalid("运行程序版本清单已变化"));
    }
    let image: ImageManifest =
        serde_json::from_slice(&image_bytes).map_err(|_| invalid("运行程序版本清单格式无效"))?;
    if image.schema_version != 1
        || image.package != OFFICIAL_PACKAGE
        || image.version != selection.version
        || image.platform != selection.platform
        || [
            &image.official_archive_sha256,
            &image.tree_sha256,
            &image.package_sha256,
            &image.bin_sha256,
            &image.node_sha256,
            &image.npm_sha256,
            &image.bun_sha256,
            &image.bun_launcher_sha256,
        ]
        .into_iter()
        .any(|v| !digest(v))
        || image.bun_launcher_sha256 != image.bun_sha256
        || image.bin.is_empty()
        || image.bin.len() > 4096
        || image.bin.chars().any(char::is_control)
        || image.bin.contains(['\\', ':'])
        || image
            .bin
            .split('/')
            .any(|v| v.is_empty() || v == "." || v == "..")
        || Path::new(&image.bin)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid("运行程序版本清单身份或入口无效"));
    }
    let package = paths.package_root.join(install::PACKAGE_SUBPATH);
    let script = package.join(&image.bin);
    for (path, expected) in [
        (package.join("package.json"), &image.package_sha256),
        (script.clone(), &image.bin_sha256),
        (paths.node.clone(), &image.node_sha256),
        (paths.npm.clone(), &image.npm_sha256),
        (paths.bun.clone(), &image.bun_sha256),
        (
            paths.package_root.join("node_modules/bun/bin/bun.exe"),
            &image.bun_launcher_sha256,
        ),
    ] {
        if file_digest(program_root, &path)? != *expected {
            return Err(invalid("运行程序文件与版本清单不一致"));
        }
    }
    let package_json: serde_json::Value =
        serde_json::from_slice(&metadata(program_root, &package.join("package.json"))?)
            .map_err(|_| invalid("运行程序包信息无效"))?;
    let bin =
        super::archive::resolve_bin(&package_json).ok_or_else(|| invalid("运行程序包缺少入口"))?;
    if package_json["name"] != image.package
        || package_json["version"] != image.version
        || bin.strip_prefix("./").unwrap_or(&bin) != image.bin
    {
        return Err(invalid("实际程序包身份与所选版本不一致"));
    }
    for path in [&paths.node, &paths.bun, &script] {
        super::paths::validate_executable(path).map_err(|_| invalid("所选运行程序不可执行"))?;
    }
    if metadata(program_root, &paths.version_root.join(IMAGE_MANIFEST))? != image_bytes {
        return Err(invalid("校验期间候选版本清单已变化"));
    }

    Ok(SelectedProgram {
        selection: selection.clone(),
        image,
        paths,
        stable_entry: program_root.join(super::program_layout::runtime_entry_relative(platform)),
        script,
    })
}

#[cfg(all(test, unix))]
#[path = "program_selection_tests.rs"]
pub(crate) mod tests;
