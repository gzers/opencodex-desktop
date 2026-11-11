//! Build a separate official runtime image, then use the shared selection
//! transaction. Node/npm are fixed by the currently admitted image, never PATH.
use super::*;
use crate::errors::AppResult;
use crate::modules::{
    backup::safety,
    runtime::{
        program_activation::Mutation,
        program_layout::{self, RuntimeVersionPaths},
        program_selection::{
            self, CurrentSelection, ImageManifest, SelectedProgram, IMAGE_MANIFEST,
        },
    },
};
use std::io::{Read, Write};

const MAX_ENTRIES: usize = 100_000;
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;

/// The npm bun wrapper's install script only selects/copies the platform Bun.
/// Do that documented placement with bounded file I/O; official OpenCodex files
/// remain untouched and arbitrary dependency lifecycle scripts remain opt-in.
pub(super) fn materialize_bun(prefix: &Path) -> Result<(), InstallError> {
    let (target, _) = program_layout::host_target().map_err(InstallError::from)?;
    let name = match target {
        "darwin-arm64" => "bun-darwin-aarch64",
        "win32-x64" => "bun-windows-x64",
        "linux-arm64" => "bun-linux-aarch64",
        "linux-x64" => "bun-linux-x64",
        _ => return Err(bad("不支持的Bun平台")),
    };
    let wrapper = prefix.join("node_modules/bun");
    let native = prefix.join("node_modules/@oven").join(name);
    let info = |path: &Path| -> Result<serde_json::Value, InstallError> {
        safety::check_path(prefix, path, false).map_err(InstallError::from)?;
        serde_json::from_slice(&safety::read_file(path, 64 * 1024).map_err(InstallError::from)?)
            .map_err(|_| bad("Bun依赖包信息无效"))
    };
    let w = info(&wrapper.join("package.json"))?;
    let n = info(&native.join("package.json"))?;
    if w["name"] != "bun"
        || n["name"] != format!("@oven/{name}")
        || !w["version"].is_string()
        || n["version"] != w["version"]
    {
        return Err(bad("Bun平台包与依赖声明不一致"));
    }
    let source = native.join(if target == "win32-x64" {
        "bin/bun.exe"
    } else {
        "bin/bun"
    });
    safety::check_path(prefix, &source, false).map_err(InstallError::from)?;
    paths::validate_executable(&source)?;
    let destination = wrapper.join("bin/bun.exe");
    safety::check_path(prefix, &destination, true).map_err(InstallError::from)?;
    match std::fs::symlink_metadata(&destination) {
        Ok(_) => {
            let opened = safety::open_file(&destination, false).map_err(InstallError::from)?;
            safety::require_single_link(&opened).map_err(InstallError::from)?;
            let current = digest(&destination)?;
            if digest(&source)? != current {
                // Official bun@1.4.4 ships a 450-byte error placeholder even
                // with scripts disabled. This digest was checked against its
                // sha512-locked npm tarball, not guessed from filename/content.
                if w["version"] != "1.4.4"
                    || current != "3877c58dda42c0c5ce4f33d21bfde4733c8e6906e8268e136cfdd6ea522df040"
                {
                    return Err(bad("Bun入口既不是已核验的占位文件，也不是对应平台二进制"));
                }
                let stage = tempfile::Builder::new()
                    .prefix(".bun-placement-")
                    .tempdir_in(destination.parent().unwrap())
                    .map_err(|e| bad(&e.to_string()))?;
                let prepared = stage.path().join("bun.exe");
                file_copy(&source, &prepared, &mut CopyBudget::default())?;
                if digest(&destination)? != current {
                    return Err(bad("Bun占位文件在准备期间变化"));
                }
                crate::infrastructure::platform::rename(&prepared, &destination)
                    .map_err(|e| bad(&e.to_string()))?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            file_copy(&source, &destination, &mut CopyBudget::default())?
        }
        Err(e) => return Err(bad(&e.to_string())),
    }
    if digest(&source)? != digest(&destination)? {
        return Err(bad("Bun入口读回不符"));
    }
    Ok(())
}

pub struct Outcome {
    pub installation: InstallOutcome,
    pub changed: bool,
}
#[derive(Default)]
struct CopyBudget {
    entries: usize,
    bytes: u64,
}
fn bad(detail: &str) -> InstallError {
    recovery_error("versioned_install_invalid", detail)
}
pub(super) fn pin_archive(
    temp: &Path,
    source: &Path,
) -> Result<(PathBuf, String, String), InstallError> {
    let observed = archive::inspect(source, None)?;
    let path = temp
        .join("sources")
        .join(format!("opencodex-{}.tgz", observed.version));
    file_copy(source, &path, &mut CopyBudget::default())?;
    let pinned = archive::inspect(&path, None)?;
    if pinned.sha256 != observed.sha256 || pinned.version != observed.version {
        return Err(bad("官方源归档在固定期间变化"));
    }
    let spec = format!("file:sources/opencodex-{}.tgz", pinned.version);
    Ok((path, spec, pinned.sha256))
}
pub(super) fn verify_source_after_install(
    temp: &Path,
    path: &Path,
    expected: &str,
) -> Result<(), InstallError> {
    if digest(path)? != expected {
        return Err(bad("官方源归档在安装期间变化"));
    }
    let check = tempfile::Builder::new()
        .prefix(".official-source-check-")
        .tempdir_in(temp)
        .map_err(|e| bad(&e.to_string()))?;
    let original = check.path().join("package");
    archive::extract(path, None, &original)?;
    if tree_digest(&original)? != tree_digest(&temp.join(PACKAGE_SUBPATH))? {
        return Err(bad("安装后的OpenCodex文件与官方原包不一致"));
    }
    Ok(())
}
fn file_copy(source: &Path, target: &Path, budget: &mut CopyBudget) -> Result<(), InstallError> {
    let mut input = safety::open_file(source, false).map_err(InstallError::from)?;
    safety::require_single_link(&input).map_err(InstallError::from)?;
    let before = input.metadata().map_err(|e| bad(&e.to_string()))?;
    if before.len() > MAX_FILE {
        return Err(bad("候选文件超过上限"));
    }
    budget.bytes = budget
        .bytes
        .checked_add(before.len())
        .ok_or_else(|| bad("候选大小溢出"))?;
    if budget.bytes > MAX_BYTES {
        return Err(bad("候选版本总量超过2GiB"));
    }
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        if before.permissions().mode() & 0o111 != 0 {
            0o755
        } else {
            0o644
        }
    };
    #[cfg(not(unix))]
    let mode = 0o644;
    crate::infrastructure::platform::create_private_dir_all(
        target.parent().ok_or_else(|| bad("缺少候选父目录"))?,
        0o755,
    )
    .map_err(|e| bad(&e.to_string()))?;
    let mut output = crate::infrastructure::platform::create_new_private_file(target, mode)
        .map_err(|e| bad(&e.to_string()))?;
    let copied = std::io::copy(
        &mut std::io::Read::by_ref(&mut input).take(MAX_FILE + 1),
        &mut output,
    )
    .map_err(|e| bad(&e.to_string()))?;
    output.sync_all().map_err(|e| bad(&e.to_string()))?;
    if copied != before.len()
        || input.metadata().map_err(|e| bad(&e.to_string()))?.len() != before.len()
    {
        return Err(bad("复制期间候选文件发生变化"));
    }
    Ok(())
}
fn copy_tree(
    source: &Path,
    target: &Path,
    budget: &mut CopyBudget,
    skip_downloads: bool,
) -> Result<(), InstallError> {
    if !safety::inspect(source)
        .map_err(InstallError::from)?
        .is_dir()
    {
        return Err(bad("运行依赖根不是普通目录"));
    }
    let source = source.canonicalize().map_err(|e| bad(&e.to_string()))?;
    if !safety::inspect(&source)
        .map_err(InstallError::from)?
        .is_dir()
    {
        return Err(bad("运行依赖不是目录"));
    }
    let mut stack = vec![(source.clone(), target.to_path_buf(), 0usize)];
    while let Some((directory, destination, depth)) = stack.pop() {
        if depth > 64 {
            return Err(bad("候选目录过深"));
        }
        safety::check_path(&source, &directory, false).map_err(InstallError::from)?;
        crate::infrastructure::platform::create_private_dir_all(&destination, 0o755)
            .map_err(|e| bad(&e.to_string()))?;
        for item in std::fs::read_dir(&directory).map_err(|e| bad(&e.to_string()))? {
            let item = item.map_err(|e| bad(&e.to_string()))?;
            // npm's archive scratch is not part of the selected runtime image.
            // The pinned source lives under sources/ with a stable relative spec.
            if skip_downloads && depth == 0 && item.file_name() == ".downloads" {
                continue;
            }
            budget.entries += 1;
            if budget.entries > MAX_ENTRIES {
                return Err(bad("候选条目超过上限"));
            }
            if item
                .file_name()
                .to_string_lossy()
                .starts_with(".npmrc-proxy-")
            {
                return Err(bad("安装代理临时凭据尚未清理，未发布候选"));
            }
            let path = item.path();
            let destination = destination.join(item.file_name());
            let meta = std::fs::symlink_metadata(&path).map_err(|e| bad(&e.to_string()))?;
            if meta.file_type().is_symlink() {
                let link = std::fs::read_link(&path).map_err(|e| bad(&e.to_string()))?;
                if link.is_absolute() {
                    return Err(bad("候选包含绝对链接"));
                }
                let resolved = path.canonicalize().map_err(|e| bad(&e.to_string()))?;
                if !resolved.starts_with(&source)
                    || !safety::inspect(&resolved)
                        .map_err(InstallError::from)?
                        .is_file()
                {
                    return Err(bad("候选链接越界或不是文件"));
                }
                safety::check_path(&source, &resolved, false).map_err(InstallError::from)?;
                file_copy(&resolved, &destination, budget)?;
            } else if safety::inspect(&path).map_err(InstallError::from)?.is_dir() {
                stack.push((path, destination, depth + 1));
            } else {
                file_copy(&path, &destination, budget)?;
            }
        }
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "install_versioned_tests.rs"]
mod tests;
fn digest(path: &Path) -> Result<String, InstallError> {
    crate::infrastructure::hash::sha256_file(path).map_err(|e| bad(&e.to_string()))
}
fn tree_digest(root: &Path) -> Result<String, InstallError> {
    let mut rows = std::collections::BTreeMap::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut budget = CopyBudget::default();
    while let Some((dir, depth)) = stack.pop() {
        if depth > 64 {
            return Err(bad("候选目录过深"));
        }
        safety::check_path(root, &dir, false).map_err(InstallError::from)?;
        for item in std::fs::read_dir(&dir).map_err(|e| bad(&e.to_string()))? {
            let item = item.map_err(|e| bad(&e.to_string()))?;
            budget.entries += 1;
            if budget.entries > MAX_ENTRIES {
                return Err(bad("候选条目超过上限"));
            }
            let path = item.path();
            let meta = safety::inspect(&path).map_err(InstallError::from)?;
            let name = path
                .strip_prefix(root)
                .map_err(|_| bad("候选文件路径越界"))?
                .to_str()
                .ok_or_else(|| bad("候选路径不是有效文本"))?
                .replace('\\', "/");
            if meta.is_dir() {
                rows.insert(name, "directory".to_string());
                stack.push((path, depth + 1));
            } else {
                if meta.len() > MAX_FILE {
                    return Err(bad("候选文件超过上限"));
                }
                budget.bytes = budget
                    .bytes
                    .checked_add(meta.len())
                    .ok_or_else(|| bad("候选大小溢出"))?;
                if budget.bytes > MAX_BYTES {
                    return Err(bad("候选版本总量超过2GiB"));
                }
                let file = safety::open_file(&path, false).map_err(InstallError::from)?;
                safety::require_single_link(&file).map_err(InstallError::from)?;
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;
                rows.insert(name, format!("{}:{executable}", digest(&path)?));
            }
        }
    }
    Ok(crate::infrastructure::hash::sha256_hex(
        &serde_json::to_vec(&rows).map_err(|_| bad("候选文件树无法编码"))?,
    ))
}

impl RuntimeInstaller<'_> {
    /// Caller holds Desktop's install/storage reservations. This method never
    /// replaces an existing immutable version directory or falls back to flat I.
    pub fn install_versioned(
        &self,
        program_root: &Path,
        current: &SelectedProgram,
        request: &InstallRequest,
        require_stopped: impl Fn() -> AppResult<()>,
    ) -> Result<Outcome, InstallError> {
        let (target, platform) = program_layout::host_target().map_err(InstallError::from)?;
        super::super::bundled::roots(program_root, self.data_root).map_err(InstallError::from)?;
        super::super::program_image::verify(current).map_err(InstallError::from)?;
        let mutation = Mutation::acquire(program_root).map_err(InstallError::from)?;
        let actual =
            program_selection::read(program_root, platform, target).map_err(InstallError::from)?;
        if actual.as_ref() != Some(current)
            || request.prefix != current.paths.package_root
            || request.node_path.as_ref() != Some(&current.paths.node)
            || request.npm_path.as_ref() != Some(&current.paths.npm)
        {
            return Err(bad("安装输入与当前已核验的程序来源不符"));
        }
        require_stopped()
            .map_err(|_| bad("请先停止当前OpenCodex并确认服务已退出，再安装运行包"))?;
        self.check_cancel()?;
        self.emit(InstallPhase::Preparing, 2, None);
        let stage = tempfile::Builder::new()
            .prefix(".runtime-install-stage-")
            .tempdir_in(program_root)
            .map_err(|e| bad(&e.to_string()))?;
        let raw = stage.path().join("npm-output");
        std::fs::create_dir(&raw).map_err(|e| bad(&e.to_string()))?;
        let filled = match &request.source {
            InstallSource::Offline { archive } => {
                self.fill_offline(request, &raw, archive, true)?
            }
            InstallSource::Registry { version, proxy } => {
                self.fill_registry(request, &raw, version, proxy.as_ref(), true)?
            }
        };
        let expected = match &request.source {
            InstallSource::Registry { version, .. } if version != default_version() => {
                Some(version.as_str())
            }
            _ => None,
        };
        let verified = verify_package(&raw, expected, self.probe, Some(&current.paths.node))?;
        let destination = RuntimeVersionPaths::new(
            &program_root.join("runtime/versions"),
            &verified.version,
            platform,
        )
        .map_err(InstallError::from)?;
        let candidate =
            RuntimeVersionPaths::new(&stage.path().join("images"), &verified.version, platform)
                .map_err(InstallError::from)?;
        let mut budget = CopyBudget::default();
        copy_tree(&raw, &candidate.package_root, &mut budget, true)?;
        copy_tree(
            &current.paths.version_root.join("dependencies/node"),
            &candidate.version_root.join("dependencies/node"),
            &mut budget,
            false,
        )?;
        copy_tree(
            &current.paths.version_root.join("dependencies/npm"),
            &candidate.version_root.join("dependencies/npm"),
            &mut budget,
            false,
        )?;
        let bun = candidate.package_root.join("node_modules/bun/bin/bun.exe");
        safety::check_path(&candidate.version_root, &bun, false).map_err(InstallError::from)?;
        file_copy(&bun, &candidate.bun, &mut budget)?;
        #[cfg(unix)]
        crate::infrastructure::platform::set_mode(&candidate.bun, 0o755)
            .map_err(|e| bad(&e.to_string()))?;
        if digest(&candidate.node)? != current.image.node_sha256
            || digest(&candidate.npm)? != current.image.npm_sha256
        {
            return Err(bad("复制后的固定运行依赖与当前来源不符"));
        }
        verify_package(
            &candidate.package_root,
            Some(&verified.version),
            self.probe,
            Some(&candidate.node),
        )?;
        let manifest = ImageManifest {
            tree_sha256: super::super::program_image::content_digest(&candidate.version_root)
                .map_err(InstallError::from)?,
            schema_version: 1,
            package: OFFICIAL_PACKAGE.into(),
            version: verified.version.clone(),
            platform: target.into(),
            official_archive_sha256: filled.tarball_sha256.clone(),
            bin: verified.bin_entry.clone(),
            package_sha256: digest(
                &candidate
                    .package_root
                    .join(PACKAGE_SUBPATH)
                    .join("package.json"),
            )?,
            bin_sha256: digest(
                &candidate
                    .package_root
                    .join(PACKAGE_SUBPATH)
                    .join(&verified.bin_entry),
            )?,
            node_sha256: digest(&candidate.node)?,
            npm_sha256: digest(&candidate.npm)?,
            bun_sha256: digest(&candidate.bun)?,
            bun_launcher_sha256: digest(&bun)?,
        };
        let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| bad(&e.to_string()))?;
        let mut file = crate::infrastructure::platform::create_new_private_file(
            &candidate.version_root.join(IMAGE_MANIFEST),
            0o600,
        )
        .map_err(|e| bad(&e.to_string()))?;
        file.write_all(&bytes).map_err(|e| bad(&e.to_string()))?;
        file.sync_all().map_err(|e| bad(&e.to_string()))?;
        drop(file);
        self.check_cancel()?;
        if program_selection::read(program_root, platform, target)
            .map_err(InstallError::from)?
            .as_ref()
            != Some(current)
        {
            return Err(bad("候选准备期间当前运行程序已变化"));
        }
        let outcome = InstallOutcome {
            package: OFFICIAL_PACKAGE.into(),
            version: verified.version.clone(),
            target: destination.package_root.clone(),
            entry: current.stable_entry.clone(),
            tarball_sha256: filled.tarball_sha256,
            source: filled.source,
            scripts_enabled: filled.scripts_enabled,
            npm_path: filled.npm_path,
            installed_at: now_rfc3339(),
            proxy_used: matches!(
                request.source,
                InstallSource::Registry { proxy: Some(_), .. }
            ),
        };
        let same_core = ImageManifest {
            tree_sha256: current.image.tree_sha256.clone(),
            ..manifest.clone()
        } == current.image;
        if same_core {
            super::super::program_image::verify(current).map_err(InstallError::from)?;
            self.emit(
                InstallPhase::Done,
                100,
                Some("已核验当前版本，无需切换".into()),
            );
            return Ok(Outcome {
                installation: outcome,
                changed: false,
            });
        }
        safety::check_path(program_root, &destination.version_root, true)
            .map_err(InstallError::from)?;
        let source_tree = tree_digest(&candidate.version_root)?;
        let already_published = match std::fs::symlink_metadata(&destination.version_root) {
            Ok(_) => {
                if tree_digest(&destination.version_root)? != source_tree {
                    return Err(bad("该版本目录与本次完整候选不一致，未覆盖"));
                }
                true
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(bad(&e.to_string())),
        };
        require_stopped().map_err(|_| bad("候选准备后未能确认服务已停止，未切换版本"))?;
        let protection =
            crate::modules::model_config::config_file::v2::native_import::backups::protect_upgrade(
                self.data_root,
                self.config_backup
                    .as_ref()
                    .ok_or_else(|| bad("缺少实际HOME的配置保护上下文"))?,
            )
            .map_err(config_protection_error)?;
        self.check_cancel()?;
        if !already_published {
            super::super::bundled::publish(&candidate.version_root, &destination.version_root)
                .map_err(InstallError::from)?;
        }
        if tree_digest(&destination.version_root)? != source_tree {
            return Err(bad("发布后的版本文件树与候选不一致，未激活"));
        }
        safety::sync_directories(program_root, &program_root.join("runtime/versions"))
            .map_err(InstallError::from)?;
        verify_package(
            &destination.package_root,
            Some(&verified.version),
            self.probe,
            Some(&destination.node),
        )?;
        let selection = CurrentSelection {
            schema_version: 1,
            version: verified.version,
            platform: target.into(),
            image_sha256: crate::infrastructure::hash::sha256_hex(&bytes),
        };
        mutation
            .activate_install(
                self.data_root,
                Some(&current.selection),
                &selection,
                &outcome.history_entry("succeeded", None),
                || {
                    self.check_cancel().map_err(AppError::from)?;
                    require_stopped()?;
                    protection
                        .verify()
                        .map_err(|e| AppError::from(config_protection_error(e)))
                },
            )
            .map_err(InstallError::from)?;
        self.emit(InstallPhase::Done, 100, None);
        Ok(Outcome {
            installation: outcome,
            changed: true,
        })
    }
}
