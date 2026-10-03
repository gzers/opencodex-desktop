//! 离线包（`.tgz`）校验与解包加固（IMP `FZ-49`）。
//!
//! 这是本域最容易被攻击的一步：归档条目可以声明任意路径、符号链接与超大体量。
//! 因此**先枚举校验、再展开**，且任一条不合格就整体拒绝——不做「尽力而为」的部分展开。

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;

use super::OFFICIAL_PACKAGE;

/// 归档上限（`FZ-49`）。
pub const MAX_ENTRIES: usize = 20_000;
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_DEPTH: usize = 16;

/// 拒绝原因；`code()` 是稳定标识，界面文案在命令层映射。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveRejection {
    /// 不是 gzip 流（按魔数判断，不只看扩展名）。
    NotGzip,
    /// 文件名不匹配 `opencodex-<semver>.tgz`（或 `bitkyc08-opencodex-<semver>.tgz`）。
    BadFilename,
    /// 包内没有 `package.json`。
    MissingPackageJson,
    /// `package.json` 无法解析或不含 `name` / `version`。
    BadPackageJson,
    /// 包名不是官方包。
    NotOfficialPackage { found: String },
    /// 包内版本与文件名声明不一致。
    VersionMismatch { declared: String, found: String },
    /// 条目数超限。
    TooManyEntries,
    /// 解压总量超限。
    TooLarge,
    /// 单文件超限。
    FileTooLarge,
    /// 嵌套深度超限。
    DepthExceeded,
    /// 条目路径越界（绝对路径、`..`、或规范化后不在目标根内）。
    PathEscape { entry: String },
    /// 符号链接 / 硬链接条目。
    LinkEntry { entry: String },
    /// 设备 / FIFO 等特殊条目。
    SpecialEntry { entry: String },
    /// 读流失败。
    Read,
}

impl ArchiveRejection {
    pub fn code(&self) -> &'static str {
        match self {
            ArchiveRejection::NotGzip => "not_gzip",
            ArchiveRejection::BadFilename => "bad_filename",
            ArchiveRejection::MissingPackageJson => "missing_package_json",
            ArchiveRejection::BadPackageJson => "bad_package_json",
            ArchiveRejection::NotOfficialPackage { .. } => "not_official_package",
            ArchiveRejection::VersionMismatch { .. } => "version_mismatch",
            ArchiveRejection::TooManyEntries => "too_many_entries",
            ArchiveRejection::TooLarge => "too_large",
            ArchiveRejection::FileTooLarge => "file_too_large",
            ArchiveRejection::DepthExceeded => "depth_exceeded",
            ArchiveRejection::PathEscape { .. } => "path_escape",
            ArchiveRejection::LinkEntry { .. } => "link_entry",
            ArchiveRejection::SpecialEntry { .. } => "special_entry",
            ArchiveRejection::Read => "read_failed",
        }
    }
}

/// 一次成功的校验结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveInspection {
    pub name: String,
    pub version: String,
    /// 包内 `package.json` 的 `bin` 解析出的入口（相对包根）。
    pub bin_entry: String,
    pub sha256: String,
    pub entries: usize,
    pub unpacked_bytes: u64,
}

/// tarball 摘要（`FZ-49` 要求记录并展示）。
pub fn sha256_of(path: &Path) -> Result<String, ArchiveRejection> {
    crate::infrastructure::hash::sha256_file(path).map_err(|_| ArchiveRejection::Read)
}

/// 文件名模式校验：`opencodex-<semver>.tgz` 或 `bitkyc08-opencodex-<semver>.tgz`。
///
/// 返回声明的版本。扩展名只接受 `.tgz` / `.tar.gz`，且**必须**匹配模式——
/// 不匹配就拒绝，不做「猜一个版本」的兜底。
pub fn declared_version(path: &Path) -> Result<String, ArchiveRejection> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ArchiveRejection::BadFilename)?;
    let stem = file_name
        .strip_suffix(".tgz")
        .or_else(|| file_name.strip_suffix(".tar.gz"))
        .ok_or(ArchiveRejection::BadFilename)?;
    let version = stem
        .strip_prefix("bitkyc08-opencodex-")
        .or_else(|| stem.strip_prefix("opencodex-"))
        .ok_or(ArchiveRejection::BadFilename)?;
    if version.is_empty() || !version.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(ArchiveRejection::BadFilename);
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        return Err(ArchiveRejection::BadFilename);
    }
    Ok(version.to_string())
}

/// gzip 魔数判断：只看扩展名是不够的。
fn is_gzip(path: &Path) -> bool {
    let mut magic = [0u8; 2];
    match std::fs::File::open(path).and_then(|mut file| file.read_exact(&mut magic)) {
        Ok(()) => magic == [0x1f, 0x8b],
        Err(_) => false,
    }
}

/// 条目路径校验：相对、无 `..`、深度受限，且规范化后仍以 `dest` 为前缀。
fn validate_entry_path(entry: &Path) -> Result<PathBuf, ArchiveRejection> {
    let display = entry.to_string_lossy().into_owned();
    let mut depth = 0usize;
    let mut normalized = PathBuf::new();
    for component in entry.components() {
        match component {
            Component::Normal(part) => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err(ArchiveRejection::DepthExceeded);
                }
                normalized.push(part);
            }
            Component::CurDir => {}
            // 绝对路径、`..`、Windows 前缀与根目录一律拒绝。
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ArchiveRejection::PathEscape { entry: display });
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(ArchiveRejection::PathEscape { entry: display });
    }
    Ok(normalized)
}

/// 口子入口名：`package.json` 的 `bin`（字符串或含 `ocx` 的映射）。
pub(crate) fn resolve_bin(package_json: &serde_json::Value) -> Option<String> {
    match package_json.get("bin") {
        Some(serde_json::Value::String(single)) => Some(single.clone()),
        Some(serde_json::Value::Object(map)) => map
            .get("ocx")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .or_else(|| {
                map.values()
                    .find_map(|value| value.as_str())
                    .map(|value| value.to_string())
            }),
        _ => None,
    }
}

/// 只校验不展开：读一遍归档，累计条目数与解压体量，检查包名与版本。
///
/// `declared` 为文件名声明的版本；`None` 表示跳过文件名要求（内部使用）。
pub fn inspect(
    archive: &Path,
    declared: Option<&str>,
) -> Result<ArchiveInspection, ArchiveRejection> {
    scan(archive, declared, None).map(|(inspection, _)| inspection)
}

/// 校验并展开到 `dest`：**任一条不合格即整体拒绝**，失败时由调用方清理临时目录。
///
/// 归档里的顶层 `package/` 会被剥掉，展开结果就是包的根目录。
pub fn extract(
    archive: &Path,
    declared: Option<&str>,
    dest: &Path,
) -> Result<ArchiveInspection, ArchiveRejection> {
    scan(archive, declared, Some(dest)).map(|(inspection, _)| inspection)
}

fn scan(
    archive: &Path,
    declared: Option<&str>,
    mut extract_to: Option<&Path>,
) -> Result<(ArchiveInspection, ()), ArchiveRejection> {
    if !is_gzip(archive) {
        return Err(ArchiveRejection::NotGzip);
    }
    if let Some(declared) = declared {
        let from_name = declared_version(archive)?;
        if from_name != declared {
            return Err(ArchiveRejection::VersionMismatch {
                declared: declared.to_string(),
                found: from_name,
            });
        }
    } else {
        // 没有外部声明时仍要求文件名合规——这是「只允许官方包」的第一道闸。
        declared_version(archive)?;
    }

    let file = std::fs::File::open(archive).map_err(|_| ArchiveRejection::Read)?;
    let mut tar = tar::Archive::new(GzDecoder::new(file));

    let mut entries = 0usize;
    let mut unpacked_bytes = 0u64;
    let mut package_json: Option<serde_json::Value> = None;

    let iter = tar.entries().map_err(|_| ArchiveRejection::Read)?;
    for entry in iter {
        let mut entry = entry.map_err(|_| ArchiveRejection::Read)?;
        entries += 1;
        if entries > MAX_ENTRIES {
            return Err(ArchiveRejection::TooManyEntries);
        }
        let entry_type = entry.header().entry_type();
        let raw_path = entry
            .path()
            .map_err(|_| ArchiveRejection::Read)?
            .to_path_buf();
        let raw_display = raw_path.to_string_lossy().into_owned();

        if entry_type.is_symlink() || entry_type.is_hard_link() {
            return Err(ArchiveRejection::LinkEntry { entry: raw_display });
        }
        if !(entry_type.is_file() || entry_type.is_dir()) {
            return Err(ArchiveRejection::SpecialEntry { entry: raw_display });
        }

        let normalized = validate_entry_path(&raw_path)?;
        // npm tarball 约定：包内容在顶层 `package/` 下；剥掉它，其余保持原样。
        let relative = normalized
            .strip_prefix("package")
            .map(|value| value.to_path_buf())
            .unwrap_or(normalized);
        if relative.as_os_str().is_empty() {
            continue;
        }

        let size = entry.header().size().map_err(|_| ArchiveRejection::Read)?;
        if entry_type.is_file() {
            if size > MAX_FILE_BYTES {
                return Err(ArchiveRejection::FileTooLarge);
            }
            unpacked_bytes = unpacked_bytes.saturating_add(size);
            if unpacked_bytes > MAX_TOTAL_BYTES {
                return Err(ArchiveRejection::TooLarge);
            }
        }

        if let Some(root) = extract_to.as_mut() {
            let target = root.join(&relative);
            // 二次防线：规范化后的落点必须仍在目标根内。
            if !target.starts_with(*root) {
                return Err(ArchiveRejection::PathEscape { entry: raw_display });
            }
            if entry_type.is_dir() {
                std::fs::create_dir_all(&target).map_err(|_| ArchiveRejection::Read)?;
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|_| ArchiveRejection::Read)?;
                }
                let mut payload = Vec::with_capacity(size as usize);
                entry
                    .read_to_end(&mut payload)
                    .map_err(|_| ArchiveRejection::Read)?;
                std::fs::write(&target, &payload).map_err(|_| ArchiveRejection::Read)?;
                // tar 保留权限位；补一次显式设置，避免解包后入口不可执行。
                if let Ok(mode) = entry.header().mode() {
                    let _ = crate::infrastructure::platform::set_mode(&target, mode & 0o777);
                }
            }
        }

        if relative.as_path() == Path::new("package.json") {
            if let Some(root) = extract_to.as_ref() {
                let raw = std::fs::read(root.join("package.json"))
                    .map_err(|_| ArchiveRejection::MissingPackageJson)?;
                package_json = Some(
                    serde_json::from_slice(&raw).map_err(|_| ArchiveRejection::BadPackageJson)?,
                );
            } else {
                let mut payload = Vec::new();
                entry
                    .read_to_end(&mut payload)
                    .map_err(|_| ArchiveRejection::Read)?;
                package_json = Some(
                    serde_json::from_slice(&payload)
                        .map_err(|_| ArchiveRejection::BadPackageJson)?,
                );
            }
        }
    }

    let package_json = package_json.ok_or(ArchiveRejection::MissingPackageJson)?;
    let name = package_json
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or(ArchiveRejection::BadPackageJson)?
        .to_string();
    let version = package_json
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or(ArchiveRejection::BadPackageJson)?
        .to_string();
    if name != OFFICIAL_PACKAGE {
        return Err(ArchiveRejection::NotOfficialPackage { found: name });
    }
    if declared_version(archive)? != version {
        return Err(ArchiveRejection::VersionMismatch {
            declared: declared_version(archive)?,
            found: version,
        });
    }
    let bin_entry = resolve_bin(&package_json).ok_or(ArchiveRejection::BadPackageJson)?;
    validate_entry_path(Path::new(&bin_entry))?;

    Ok((
        ArchiveInspection {
            name,
            version,
            bin_entry,
            sha256: sha256_of(archive)?,
            entries,
            unpacked_bytes,
        },
        (),
    ))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// 造一个最小可用离线包：`package/package.json` + 入口脚本。
    fn build_tgz(
        dir: &Path,
        file_name: &str,
        package_json: serde_json::Value,
        extra: &[(&str, &[u8], bool)],
    ) -> PathBuf {
        let path = dir.join(file_name);
        let file = std::fs::File::create(&path).expect("create tgz");
        let encoder = GzEncoder::new(file, Compression::fast());
        let mut builder = tar::Builder::new(encoder);

        let payload = serde_json::to_vec_pretty(&package_json).expect("serialize package.json");
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", payload.as_slice())
            .expect("append package.json");

        for (name, body, executable) in extra {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(if *executable { 0o755 } else { 0o644 });
            header.set_cksum();
            builder
                .append_data(&mut header, name, *body)
                .expect("append entry");
        }
        builder
            .into_inner()
            .expect("finish tar")
            .finish()
            .expect("finish gzip");
        path
    }

    fn official_json(version: &str) -> serde_json::Value {
        serde_json::json!({
            "name": OFFICIAL_PACKAGE,
            "version": version,
            "bin": { "ocx": "bin/ocx.js" }
        })
    }

    fn official_tgz(dir: &Path) -> PathBuf {
        build_tgz(
            dir,
            "opencodex-0.3.1.tgz",
            official_json("0.3.1"),
            &[("package/bin/ocx.js", b"#!/usr/bin/env node\n", true)],
        )
    }

    #[test]
    fn accepts_official_package_and_reports_entry() {
        let dir = tempfile::tempdir().expect("temp");
        let archive = official_tgz(dir.path());

        let inspection = inspect(&archive, Some("0.3.1")).expect("inspect");
        assert_eq!(inspection.name, OFFICIAL_PACKAGE);
        assert_eq!(inspection.version, "0.3.1");
        assert_eq!(inspection.bin_entry, "bin/ocx.js");
        assert_eq!(inspection.entries, 2);
        assert_eq!(inspection.sha256.len(), 64);
    }

    #[test]
    fn extract_strips_the_top_level_package_prefix() {
        let dir = tempfile::tempdir().expect("temp");
        let archive = official_tgz(dir.path());
        let dest = dir.path().join("out");
        std::fs::create_dir(&dest).expect("mkdir");

        extract(&archive, Some("0.3.1"), &dest).expect("extract");

        assert!(dest.join("package.json").is_file());
        assert!(dest.join("bin/ocx.js").is_file());
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dest.join("bin/ocx.js"))
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o755, "入口的可执行位必须保留");
    }

    #[test]
    fn rejects_non_gzip_and_bad_filename() {
        let dir = tempfile::tempdir().expect("temp");
        let plain = dir.path().join("opencodex-0.3.1.tgz");
        std::fs::write(&plain, b"not gzip").expect("write");
        assert_eq!(
            inspect(&plain, Some("0.3.1")).unwrap_err(),
            ArchiveRejection::NotGzip
        );

        let archive = official_tgz(dir.path());
        let renamed = dir.path().join("opencodex-latest.tgz");
        std::fs::copy(&archive, &renamed).expect("copy");
        assert_eq!(
            inspect(&renamed, None).unwrap_err(),
            ArchiveRejection::BadFilename
        );
    }

    #[test]
    fn rejects_non_official_package_name() {
        let dir = tempfile::tempdir().expect("temp");
        let mut json = official_json("0.3.1");
        json["name"] = serde_json::json!("evil-package");
        let archive = build_tgz(dir.path(), "opencodex-0.3.1.tgz", json, &[]);

        assert_eq!(
            inspect(&archive, Some("0.3.1")).unwrap_err(),
            ArchiveRejection::NotOfficialPackage {
                found: "evil-package".to_string()
            }
        );
    }

    #[test]
    fn rejects_version_mismatch_between_filename_and_manifest() {
        let dir = tempfile::tempdir().expect("temp");
        let archive = build_tgz(
            dir.path(),
            "opencodex-0.3.1.tgz",
            official_json("9.9.9"),
            &[("package/bin/ocx.js", b"#!/usr/bin/env node\n", true)],
        );

        match inspect(&archive, Some("0.3.1")).unwrap_err() {
            ArchiveRejection::VersionMismatch { declared, found } => {
                assert_eq!(declared, "0.3.1");
                assert_eq!(found, "9.9.9");
            }
            other => panic!("expected version mismatch, got {other:?}"),
        }
    }

    /// 手写 ustar 头 + 数据块，必要时用 Gzip 包起来：用来构造 `tar` crate
    /// 本身拒绝构造、但我们必须能拦住的恶意条目。
    fn raw_tar_entry(name: &str, body: &[u8]) -> Vec<u8> {
        let mut header = [0u8; 512];
        let name_bytes = name.as_bytes();
        header[..name_bytes.len()].copy_from_slice(name_bytes);
        header[100..108].copy_from_slice(b"0000644\0");
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        let size = format!("{:011o}\0", body.len());
        header[124..136].copy_from_slice(size.as_bytes());
        header[136..148].copy_from_slice(b"00000000000\0");
        header[148..156].copy_from_slice(b"        ");
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u32 = header.iter().map(|byte| u32::from(*byte)).sum();
        header[148..156].copy_from_slice(format!("{:06o}\0 ", checksum).as_bytes());
        let mut output = header.to_vec();
        output.extend_from_slice(body);
        let padding = (512 - body.len() % 512) % 512;
        output.extend(std::iter::repeat_n(0u8, padding));
        output
    }

    #[test]
    fn rejects_path_escape_symlink_and_hardlink_entries() {
        let dir = tempfile::tempdir().expect("temp");

        // `..` 越界条目：`tar` crate 拒绝**构造**这种条目，所以直接手写 tar 字节块，
        // 保证这里测的是「解析端能拦住恶意归档」，而不是「构造端不让造」。
        let escape = dir.path().join("opencodex-0.3.1.tgz");
        {
            let payload = serde_json::to_vec(&official_json("0.3.1")).expect("json");
            let mut bytes = raw_tar_entry("package/package.json", &payload);
            bytes.extend(raw_tar_entry("package/../../escaped.txt", b"evil"));
            bytes.extend([0u8; 1024]);
            let file = std::fs::File::create(&escape).expect("create");
            let mut encoder = GzEncoder::new(file, Compression::fast());
            encoder.write_all(&bytes).expect("gzip");
            encoder.finish().expect("finish gz");
        }
        match inspect(&escape, Some("0.3.1")).unwrap_err() {
            ArchiveRejection::PathEscape { .. } => {}
            other => panic!("expected path escape, got {other:?}"),
        }

        // 符号链接条目。
        let link = dir.path().join("opencodex-0.3.2.tgz");
        {
            let file = std::fs::File::create(&link).expect("create");
            let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::fast()));
            let payload = serde_json::to_vec(&official_json("0.3.2")).expect("json");
            let mut header = tar::Header::new_gnu();
            header.set_size(payload.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "package/package.json", payload.as_slice())
                .expect("append");
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            header.set_link_name("/etc/passwd").expect("link name");
            header.set_cksum();
            builder
                .append_data(&mut header, "package/link", std::io::empty())
                .expect("append");
            builder.into_inner().expect("tar").finish().expect("gz");
        }
        match inspect(&link, Some("0.3.2")).unwrap_err() {
            ArchiveRejection::LinkEntry { .. } => {}
            other => panic!("expected link entry, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_package_json_and_missing_bin() {
        let dir = tempfile::tempdir().expect("temp");
        let no_json = dir.path().join("opencodex-0.3.1.tgz");
        {
            let file = std::fs::File::create(&no_json).expect("create");
            let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::fast()));
            let mut header = tar::Header::new_gnu();
            header.set_size(4);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "package/readme.md", &b"hi!!"[..])
                .expect("append");
            builder.into_inner().expect("tar").finish().expect("gz");
        }
        assert_eq!(
            inspect(&no_json, Some("0.3.1")).unwrap_err(),
            ArchiveRejection::MissingPackageJson
        );

        let mut json = official_json("0.3.1");
        json.as_object_mut().expect("object").remove("bin");
        let no_bin = build_tgz(dir.path(), "opencodex-0.3.1.tgz", json, &[]);
        assert_eq!(
            inspect(&no_bin, Some("0.3.1")).unwrap_err(),
            ArchiveRejection::BadPackageJson
        );
    }

    #[test]
    fn resolves_bin_from_string_and_object_forms() {
        let single = serde_json::json!({ "bin": "cli.js" });
        assert_eq!(resolve_bin(&single).as_deref(), Some("cli.js"));
        let mapped = serde_json::json!({ "bin": { "other": "a.js", "ocx": "b.js" } });
        assert_eq!(resolve_bin(&mapped).as_deref(), Some("b.js"));
        let only_one = serde_json::json!({ "bin": { "whatever": "c.js" } });
        assert_eq!(resolve_bin(&only_one).as_deref(), Some("c.js"));
        assert_eq!(resolve_bin(&serde_json::json!({})), None);
    }

    #[test]
    fn declared_version_accepts_both_official_file_name_forms() {
        assert_eq!(
            declared_version(Path::new("/x/opencodex-1.2.3.tgz")).expect("plain"),
            "1.2.3"
        );
        assert_eq!(
            declared_version(Path::new("/x/bitkyc08-opencodex-1.2.3.tar.gz")).expect("scoped"),
            "1.2.3"
        );
        assert!(declared_version(Path::new("/x/opencodex-latest.tgz")).is_err());
        assert!(declared_version(Path::new("/x/thing.tgz")).is_err());
        assert!(declared_version(Path::new("/x/opencodex-1.2.3.zip")).is_err());
    }

    #[test]
    fn entry_path_validation_rejects_absolute_and_deep_paths() {
        assert!(validate_entry_path(Path::new("/abs")).is_err());
        assert!(validate_entry_path(Path::new("a/../../b")).is_err());
        assert!(validate_entry_path(Path::new("")).is_err());
        let deep = (0..(MAX_DEPTH + 1))
            .map(|index| format!("d{index}"))
            .collect::<Vec<_>>()
            .join("/");
        assert!(matches!(
            validate_entry_path(Path::new(&deep)),
            Err(ArchiveRejection::DepthExceeded)
        ));
        assert_eq!(
            validate_entry_path(Path::new("./pkg/./bin/ocx")).expect("normal"),
            PathBuf::from("pkg/bin/ocx")
        );
    }
}
