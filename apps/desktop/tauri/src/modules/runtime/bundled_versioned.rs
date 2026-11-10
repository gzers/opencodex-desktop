//! First supply into the versioned program layout. Archive verification remains
//! the pinned complete-tree verifier; activation never overwrites unknown files.
use super::*;
use crate::modules::{
    data_root::layout::Platform,
    runtime::{
        program_layout::RuntimeVersionPaths,
        program_selection::{
            self, CurrentSelection, ImageManifest, ACTIVATION_PENDING, IMAGE_MANIFEST,
        },
    },
};

fn host_target() -> (&'static str, Platform) {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        ("darwin-arm64", Platform::Macos)
    } else if cfg!(all(windows, target_arch = "x86_64")) {
        ("win32-x64", Platform::Windows)
    } else {
        ("unsupported", Platform::Unix)
    }
}

fn create_bytes(path: &Path, bytes: &[u8], mode: u32) -> AppResult<()> {
    let mut file =
        crate::infrastructure::platform::create_new_private_file(path, mode).map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)
}

fn copy_npm(raw: &Path, image: &Path, plan: &Plan) -> AppResult<()> {
    let base = if plan.descriptor.platform == "win32-x64" {
        "runtime/node/node_modules/npm/"
    } else {
        "runtime/node/lib/node_modules/npm/"
    };
    let target = image.join("dependencies/npm");
    let mut count = 0;
    let mut extra_bytes = 0_u64;
    let mut extra_dirs = BTreeSet::new();
    for (name, entry) in plan.files.iter().filter(|(name, _)| name.starts_with(base)) {
        let suffix = name.strip_prefix(base).unwrap();
        if suffix.is_empty() {
            return Err(invalid("npm目录条目无效"));
        }
        let resolved = if entry.kind == "symlink" {
            &plan.files[&plan.targets[name]]
        } else {
            entry
        };
        extra_bytes = extra_bytes
            .checked_add(resolved.bytes.unwrap())
            .ok_or_else(|| invalid("npm复制体积溢出"))?;
        if plan
            .total
            .checked_add(extra_bytes)
            .is_none_or(|total| total > MAX_UNPACKED)
        {
            return Err(invalid("版本落位总量超过2GiB"));
        }
        let source = raw.join(&resolved.path);
        let destination = target.join(suffix);
        let mut parent = destination.parent();
        while let Some(dir) = parent.filter(|p| p.starts_with(&target)) {
            extra_dirs.insert(dir.to_path_buf());
            parent = dir.parent();
        }
        if plan.files.len() + plan.directories.len() + count + 1 + extra_dirs.len() > MAX_ENTRIES {
            return Err(invalid("版本落位文件与目录总数超过上限"));
        }
        mkdir(destination.parent().unwrap())?;
        let mut input = regular(&source)?;
        let mut output = crate::infrastructure::platform::create_new_private_file(
            &destination,
            resolved.mode.unwrap(),
        )
        .map_err(io)?;
        let bytes = std::io::copy(&mut input, &mut output).map_err(io)?;
        output.sync_all().map_err(io)?;
        if bytes != resolved.bytes.unwrap()
            || hash::sha256_file(&destination).map_err(io)? != *resolved.sha256.as_ref().unwrap()
        {
            return Err(invalid("npm落位读回不符"));
        }
        count += 1;
    }
    if count == 0 {
        return Err(invalid("固定运行资源缺少npm"));
    }
    Ok(())
}

/// An existing valid selected image is preserved. Missing current.json never
/// authorizes adoption or migration of a flat/foreign runtime tree.
pub fn provision(
    program_root: &Path,
    desktop_root: &Path,
    archive: &Path,
    descriptor: &[u8],
    pin: &str,
) -> AppResult<ProvisionResult> {
    provision_inner(program_root, desktop_root, archive, descriptor, pin, false)
}

pub(super) fn provision_inner(
    i: &Path,
    d: &Path,
    archive: &Path,
    descriptor: &[u8],
    pin: &str,
    inject_activation_failure: bool,
) -> AppResult<ProvisionResult> {
    let (target, platform) = host_target();
    #[cfg(target_os = "macos")]
    crate::modules::data_root::layout::validate_macos_installation_root(i, None)?;
    roots(i, d)?;
    let plan = load(descriptor, pin)?;
    if plan.descriptor.platform != target {
        return Err(invalid("运行资源平台与本机不一致"));
    }
    let version = RuntimeVersionPaths::new(
        &i.join("runtime/versions"),
        &plan.descriptor.runtime_version,
        platform,
    )?;
    mkdir(i)?;
    let lock_path = i.join(".runtime-program.lock");
    safety::check_path(i, &lock_path, true)?;
    let lock = safety::open_file(&lock_path, true)?;
    safety::require_single_link(&lock)?;
    lock.try_lock()
        .map_err(|_| fail("bundle_locked", "运行程序正在操作，请稍后重试"))?;
    if let Some(selected) = program_selection::read(i, platform, target)? {
        persist(d, i, &selected.selection.version)?;
        return Ok(ProvisionResult::AlreadyInstalled {
            version: selected.selection.version,
        });
    }
    let runtime = i.join("runtime");
    if runtime.exists()
        && fs::read_dir(&runtime)
            .map_err(io)?
            .next()
            .transpose()
            .map_err(io)?
            .is_some()
    {
        return Err(fail(
            "runtime_unselected_content",
            "运行目录已有未选择内容，未覆盖或自动迁移",
        ));
    }
    let stage = tempfile::Builder::new()
        .prefix(".runtime-version-stage-")
        .tempdir_in(i)
        .map_err(io)?;
    let raw = stage.path().join("raw");
    mkdir(&raw)?;
    let copy = stage.path().join("archive.tar.gz");
    let input = regular(archive)?;
    if input.metadata().map_err(io)?.len() != plan.descriptor.archive_bytes {
        return Err(invalid("运行归档体积不符"));
    }
    let mut output =
        crate::infrastructure::platform::create_new_private_file(&copy, 0o600).map_err(io)?;
    let count = std::io::copy(&mut input.take(MAX_ARCHIVE + 1), &mut output).map_err(io)?;
    output.sync_all().map_err(io)?;
    drop(output);
    if count != plan.descriptor.archive_bytes {
        return Err(invalid("归档读取期间变化"));
    }
    check_archive(&copy, &plan)?;
    unpack(&copy, &plan, Some(&raw))?;
    verify_staged(&raw, &plan)?;
    let bin = package_bin(&raw, &plan)?;
    let image = stage.path().join("image");
    mkdir(&image)?;
    mkdir(&image.join("dependencies"))?;
    copy_npm(&raw, &image, &plan)?;
    for (from, to) in [
        ("runtime/opencodex", "opencodex"),
        ("runtime/node", "dependencies/node"),
        ("runtime/bun", "dependencies/bun"),
    ] {
        fs::rename(raw.join(from), image.join(to)).map_err(io)?;
    }
    let local_paths = RuntimeVersionPaths::new(
        &stage.path().join("unused"),
        &plan.descriptor.runtime_version,
        platform,
    )?;
    // Final paths are fixed before publishing; only hashes are read from stage.
    let relative_node = local_paths
        .node
        .strip_prefix(&local_paths.version_root)
        .unwrap();
    let relative_npm = local_paths
        .npm
        .strip_prefix(&local_paths.version_root)
        .unwrap();
    let relative_bun = local_paths
        .bun
        .strip_prefix(&local_paths.version_root)
        .unwrap();
    let package = image.join("opencodex").join(install::PACKAGE_SUBPATH);
    let manifest = ImageManifest {
        schema_version: 1,
        package: OFFICIAL_PACKAGE.into(),
        version: plan.descriptor.runtime_version.clone(),
        platform: target.into(),
        official_archive_sha256: plan.descriptor.runtime_archive_sha256.clone(),
        bin: bin.clone(),
        package_sha256: hash::sha256_file(&package.join("package.json")).map_err(io)?,
        bin_sha256: hash::sha256_file(&package.join(&bin)).map_err(io)?,
        node_sha256: hash::sha256_file(&image.join(relative_node)).map_err(io)?,
        npm_sha256: hash::sha256_file(&image.join(relative_npm)).map_err(io)?,
        bun_sha256: hash::sha256_file(&image.join(relative_bun)).map_err(io)?,
        bun_launcher_sha256: hash::sha256_file(
            &image.join("opencodex/node_modules/bun/bin/bun.exe"),
        )
        .map_err(io)?,
    };
    let image_bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|_| invalid("版本清单序列化失败"))?;
    create_bytes(&image.join(IMAGE_MANIFEST), &image_bytes, 0o600)?;
    let selection = CurrentSelection {
        schema_version: 1,
        version: manifest.version.clone(),
        platform: target.into(),
        image_sha256: hash::sha256_hex(&image_bytes),
    };
    let current_bytes =
        serde_json::to_vec_pretty(&selection).map_err(|_| invalid("版本选择序列化失败"))?;
    mkdir(&runtime)?;
    mkdir(&runtime.join("versions"))?;
    mkdir(&runtime.join("bin"))?;
    let pending = runtime.join(ACTIVATION_PENDING);
    let journal = serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"attemptId":uuid::Uuid::new_v4().to_string(),"selection":selection,"before":null})).map_err(|_| invalid("激活记录序列化失败"))?;
    create_bytes(&pending, &journal, 0o600)?;
    // No-replace publication preserves even an unknown empty version directory.
    publish(&image, &version.version_root)?;
    safety::sync_directories(i, &runtime.join("versions"))?;
    let entry = runtime.join(if platform == Platform::Windows {
        "bin/ocx.cmd"
    } else {
        "bin/ocx"
    });
    let script = version
        .package_root
        .join(install::PACKAGE_SUBPATH)
        .join(&bin);
    create_bytes(
        &entry,
        install::entry_script(Some(&version.node), &script).as_bytes(),
        0o755,
    )?;
    if inject_activation_failure {
        return Err(fail("bundle_activation_injected", "测试注入版本激活中断"));
    }
    create_bytes(
        &runtime.join(super::super::program_layout::CURRENT_FILENAME),
        &current_bytes,
        0o600,
    )?;
    safety::sync_directories(i, &runtime)?;
    let selected = program_selection::read_activating(i, platform, target)?
        .ok_or_else(|| invalid("版本激活读回缺失"))?;
    if selected.selection != selection || read(&pending, MAX_METADATA_FOR_PENDING)? != journal {
        return Err(invalid("激活读回或操作身份已变化"));
    }
    fs::remove_file(&pending).map_err(io)?;
    safety::sync_directories(i, &runtime)?;
    persist(d, i, &manifest.version)?;
    Ok(ProvisionResult::Installed {
        version: manifest.version,
    })
}
const MAX_METADATA_FOR_PENDING: u64 = 64 * 1024;
