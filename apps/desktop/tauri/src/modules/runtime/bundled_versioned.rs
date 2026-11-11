//! First supply into the versioned program layout. Archive verification remains
//! the pinned complete-tree verifier; activation never overwrites unknown files.
use super::*;
use crate::modules::runtime::{
    program_layout::RuntimeVersionPaths,
    program_selection::{self, CurrentSelection, ImageManifest, IMAGE_MANIFEST},
};

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

/// Verified file aliases are regular, independent files in version images.
/// Their bytes and mode come from the pinned resolved target, never a live
/// arbitrary link. This also keeps npm/npx independent after directory moves.
fn materialize_links(raw: &Path, scratch: &Path, plan: &Plan) -> AppResult<()> {
    let npm_base = if plan.descriptor.platform == "win32-x64" {
        "runtime/node/node_modules/npm/"
    } else {
        "runtime/node/lib/node_modules/npm/"
    };
    let mut total = plan.total;
    for (name, entry) in &plan.files {
        if name.starts_with(npm_base) {
            let resolved = if entry.kind == "symlink" {
                &plan.files[&plan.targets[name]]
            } else {
                entry
            };
            total = total
                .checked_add(
                    resolved
                        .bytes
                        .ok_or_else(|| invalid("npm链接必须映射普通文件"))?,
                )
                .ok_or_else(|| invalid("文件副本大小溢出"))?;
        }
    }
    for name in plan.targets.values() {
        total = total
            .checked_add(
                plan.files
                    .get(name)
                    .and_then(|e| e.bytes)
                    .ok_or_else(|| invalid("版本镜像只接受普通文件链接映射"))?,
            )
            .ok_or_else(|| invalid("链接副本大小溢出"))?;
    }
    if total > MAX_UNPACKED {
        return Err(invalid("版本落位包含链接副本后超过2GiB"));
    }
    mkdir(scratch)?;
    for (index, (name, target)) in plan.targets.iter().enumerate() {
        let expected = &plan.files[target];
        let path = raw.join(name);
        let meta = fs::symlink_metadata(&path).map_err(io)?;
        if meta.is_file() {
            if hash::sha256_file(&path).map_err(io)? != *expected.sha256.as_ref().unwrap() {
                return Err(invalid("链接映射副本摘要不符"));
            }
            continue;
        }
        if !meta.file_type().is_symlink()
            || fs::read_link(&path).map_err(io)?.as_path()
                != Path::new(plan.files[name].target.as_ref().unwrap())
        {
            return Err(invalid("供应链接在实体化前已变化"));
        }
        let mut input = regular(&raw.join(target))?;
        let staged = scratch.join(index.to_string());
        let mut output = crate::infrastructure::platform::create_new_private_file(
            &staged,
            expected.mode.unwrap(),
        )
        .map_err(io)?;
        let count = std::io::copy(
            &mut std::io::Read::by_ref(&mut input).take(MAX_FILE + 1),
            &mut output,
        )
        .map_err(io)?;
        output.sync_all().map_err(io)?;
        drop(output);
        if count != expected.bytes.unwrap()
            || hash::sha256_file(&staged).map_err(io)? != *expected.sha256.as_ref().unwrap()
            || fs::read_link(&path).map_err(io)?.as_path()
                != Path::new(plan.files[name].target.as_ref().unwrap())
        {
            return Err(invalid("链接实体化读回不符"));
        }
        crate::infrastructure::platform::rename(&staged, &path).map_err(io)?;
    }
    Ok(())
}

/// Existing unrelated files are never read/adopted. Only fresh reserved paths
/// are created, and collisions never authorize overwrite or migration.
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
    supply(
        i,
        d,
        archive,
        descriptor,
        pin,
        inject_activation_failure,
        None,
    )
}

struct Resupply<'a> {
    home: &'a Path,
    confirmation_digest: &'a str,
    unused: &'a dyn Fn() -> AppResult<()>,
}
pub struct ReinstallRequest<'a> {
    pub home: &'a Path,
    pub confirmation_digest: &'a str,
}
/// An explicit product action, never called by ordinary startup. The same
/// pinned archive verification and image builder are used for every attempt.
pub fn reinstall(
    i: &Path,
    d: &Path,
    request: ReinstallRequest<'_>,
    archive: &Path,
    descriptor: &[u8],
    pin: &str,
    unused: impl Fn() -> AppResult<()>,
) -> AppResult<ProvisionResult> {
    supply(
        i,
        d,
        archive,
        descriptor,
        pin,
        false,
        Some(Resupply {
            home: request.home,
            confirmation_digest: request.confirmation_digest,
            unused: &unused,
        }),
    )
}
fn supply(
    i: &Path,
    d: &Path,
    archive: &Path,
    descriptor: &[u8],
    pin: &str,
    inject_activation_failure: bool,
    resupply: Option<Resupply<'_>>,
) -> AppResult<ProvisionResult> {
    let (target, platform) = crate::modules::runtime::program_layout::host_target()?;
    #[cfg(target_os = "macos")]
    crate::modules::data_root::layout::validate_macos_installation_root(i, None)?;
    roots(i, d)?;
    // Existing selected programs are read-only inputs. Another OS user or a
    // normal launch from a protected installation must not need a write lock.
    if resupply.is_none() {
        if super::super::program_removal::intentionally_removed(i, d)? {
            return Err(fail(
                "runtime_intentionally_removed",
                "运行程序已由用户卸载，请显式重新安装内置资源",
            ));
        }
        if let Some(selected) = program_selection::read(i, platform, target)? {
            persist_entry(d, &selected.stable_entry, &selected.selection.version)?;
            return Ok(ProvisionResult::AlreadyInstalled {
                version: selected.selection.version,
            });
        }
    }
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
    let mutation = super::super::program_activation::Mutation::acquire(i)?;
    if let Some(resupply) = &resupply {
        super::super::program_reinstall::preflight(i, d, resupply.home)?;
        let review = super::super::program_reinstall::review(
            i,
            d,
            resupply.home,
            pin,
            &plan.descriptor.runtime_version,
        )?;
        if review.confirmation_digest != resupply.confirmation_digest {
            return Err(invalid("内置重装预览已过期，请重新核对原操作"));
        }
        (resupply.unused)()?;
    }
    if resupply.is_none() {
        if super::super::program_removal::intentionally_removed(i, d)? {
            return Err(fail(
                "runtime_intentionally_removed",
                "运行程序已由用户卸载，请显式重新安装内置资源",
            ));
        }
        if let Some(selected) = program_selection::read(i, platform, target)? {
            persist_entry(d, &selected.stable_entry, &selected.selection.version)?;
            return Ok(ProvisionResult::AlreadyInstalled {
                version: selected.selection.version,
            });
        }
    }
    let runtime = i.join("runtime");
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
    materialize_links(&raw, &stage.path().join("link-copies"), &plan)?;
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
        tree_sha256: super::super::program_image::content_digest(&image)?,
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
    mkdir(&runtime)?;
    mkdir(&runtime.join("versions"))?;
    mkdir(&runtime.join("bin"))?;
    if let Some(resupply) = resupply {
        let outcome = super::super::program_reinstall::install(
            super::super::program_reinstall::Candidate {
                program: i,
                desktop: d,
                home: resupply.home,
                staged_image: &image,
                selection: &selection,
                descriptor_pin: pin,
                source_sha256: &manifest.official_archive_sha256,
            },
            &mutation,
            resupply.unused,
            |_| Ok(()),
        )?;
        return Ok(ProvisionResult::Installed {
            version: outcome.version,
        });
    }
    // Publish only the verified immutable image. Selection/current/entry and D's
    // source mirror are committed by the shared recoverable transaction.
    publish(&image, &version.version_root)?;
    safety::sync_directories(i, &runtime.join("versions"))?;
    super::super::program_catalog::register(i, &selection, &manifest.official_archive_sha256)?;
    mkdir(d)?;
    mkdir(&d.join("manager-state"))?;
    if inject_activation_failure {
        mutation.activate_inner(
            d,
            None,
            &selection,
            || Ok(()),
            |phase| {
                if phase == super::super::program_activation::Checkpoint::Entry {
                    Err(fail("bundle_activation_injected", "测试注入版本激活中断"))
                } else {
                    Ok(())
                }
            },
        )?;
    } else {
        mutation.activate(d, None, &selection, || Ok(()))?;
    }
    Ok(ProvisionResult::Installed {
        version: manifest.version,
    })
}
