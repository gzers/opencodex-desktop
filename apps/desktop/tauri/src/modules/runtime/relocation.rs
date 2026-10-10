//! Prepare runtime state inside an unpublished copy. Never execute packages,
//! use the network, write external paths or change source bindings.
use super::{
    install, RuntimeSourceKind, RuntimeSourceRecord, MANAGED_ENTRY_RELATIVE, MANIFEST_FILENAME,
    RUNTIME_SCHEMA_VERSION, RUNTIME_SOURCE_FILENAME,
};
use crate::errors::{AppError, AppResult};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

fn failure(message: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "prepare migrated runtime".into(),
        detail: message.into(),
    }
}
fn io(error: std::io::Error) -> AppError {
    failure(error.to_string())
}

fn read(path: &Path, limit: u64) -> AppResult<Vec<u8>> {
    if !fs::symlink_metadata(path).map_err(io)?.is_file() {
        return Err(failure(
            "runtime metadata or launcher is not a regular file",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(io)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 > limit {
        return Err(failure("runtime metadata exceeds migration limit"));
    }
    Ok(bytes)
}

// Resolve aliases (including Windows verbatim paths), allowing absent old
// history targets. Relative/parent paths are ambiguous and are refused.
fn resolved_path(path: &Path) -> AppResult<PathBuf> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(failure(
            "runtime record contains a non-absolute or parent path",
        ));
    }
    let mut ancestor = path;
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| failure("runtime path has no existing ancestor"))?;
            }
            Err(e) => return Err(io(e)),
        }
    }
    Ok(ancestor.canonicalize().map_err(io)?.join(
        path.strip_prefix(ancestor)
            .map_err(|_| failure("invalid runtime path"))?,
    ))
}

fn relocate(path: &Path, source: &Path, target: &Path) -> AppResult<PathBuf> {
    let resolved = resolved_path(path)?;
    match resolved.strip_prefix(source) {
        Ok(relative) => Ok(target.join(relative)),
        Err(_) => Ok(path.to_path_buf()), // External Node and custom prefixes stay external.
    }
}

fn strict_record(bytes: &[u8]) -> AppResult<(RuntimeSourceRecord, serde_json::Value)> {
    let json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| failure(e.to_string()))?;
    let object = json
        .as_object()
        .ok_or_else(|| failure("runtime record is not an object"))?;
    let known = [
        "schema_version",
        "source",
        "path",
        "resolved_version",
        "resolved_at",
        "history",
    ];
    if object.keys().any(|key| !known.contains(&key.as_str())) {
        return Err(failure(
            "unknown runtime fields require an explicit relocation contract",
        ));
    }
    if let Some(history) = json.get("history") {
        for entry in history
            .as_array()
            .ok_or_else(|| failure("invalid runtime history"))?
        {
            let known = [
                "action", "result", "package", "version", "target", "at", "reason",
            ];
            if entry
                .as_object()
                .ok_or_else(|| failure("invalid history entry"))?
                .keys()
                .any(|key| !known.contains(&key.as_str()))
            {
                return Err(failure(
                    "unknown history fields require a relocation contract",
                ));
            }
        }
    }
    let record: RuntimeSourceRecord =
        serde_json::from_value(json.clone()).map_err(|e| failure(e.to_string()))?;
    if record.schema_version != RUNTIME_SCHEMA_VERSION {
        return Err(failure(
            "unsupported runtime schema; no lossy default or history truncation",
        ));
    }
    Ok((record, json))
}

/// Decode only our generated launcher. The complete text must match a fresh
/// rendering before paths can change; arbitrary shell text is never executed.
fn recorded_node(content: &str, script: &Path) -> AppResult<Option<PathBuf>> {
    if content == install::entry_script(None, script) {
        return Ok(None);
    }
    #[cfg(not(windows))]
    let node = {
        let exec = content
            .lines()
            .find_map(|line| line.strip_prefix("exec "))
            .ok_or_else(|| failure("unrecognized managed launcher"))?;
        let mut rest = exec
            .strip_prefix('\'')
            .ok_or_else(|| failure("unquoted launcher node"))?;
        let mut decoded = String::new();
        loop {
            let close = rest
                .find('\'')
                .ok_or_else(|| failure("unterminated launcher node"))?;
            decoded.push_str(&rest[..close]);
            rest = &rest[close + 1..];
            if let Some(next) = rest.strip_prefix("\\''") {
                decoded.push('\'');
                rest = next;
            } else {
                break;
            }
        }
        PathBuf::from(decoded)
    };
    #[cfg(windows)]
    let node = {
        let line = content
            .lines()
            .nth(3)
            .ok_or_else(|| failure("missing launcher command"))?;
        let rest = line
            .strip_prefix('"')
            .ok_or_else(|| failure("unquoted launcher node"))?;
        let close = rest
            .find('"')
            .ok_or_else(|| failure("unterminated launcher node"))?;
        PathBuf::from(rest[..close].replace("%%", "%"))
    };
    if !node.is_absolute() || content != install::entry_script(Some(&node), script) {
        return Err(failure(
            "launcher was modified or does not match the recorded package",
        ));
    }
    Ok(Some(node))
}

fn package_script(prefix: &Path) -> AppResult<PathBuf> {
    let manifest: install::RuntimeManifest =
        serde_json::from_slice(&read(&prefix.join(MANIFEST_FILENAME), 1024 * 1024)?)
            .map_err(|e| failure(e.to_string()))?;
    let package_root = prefix.join(install::PACKAGE_SUBPATH);
    let package: serde_json::Value =
        serde_json::from_slice(&read(&package_root.join("package.json"), 1024 * 1024)?)
            .map_err(|e| failure(e.to_string()))?;
    if manifest.package != super::OFFICIAL_PACKAGE
        || package["name"] != super::OFFICIAL_PACKAGE
        || manifest.version.is_empty()
        || package["version"] != manifest.version
    {
        return Err(failure(
            "installed official package and manifest do not agree",
        ));
    }
    let bin = super::archive::resolve_bin(&package)
        .ok_or_else(|| failure("package has no launcher bin"))?;
    let bin = Path::new(&bin);
    if bin.as_os_str().is_empty()
        || bin.is_absolute()
        || bin
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(failure("package bin escapes package root"));
    }
    let script = package_root.join(bin);
    if !script
        .canonicalize()
        .map_err(io)?
        .starts_with(package_root.canonicalize().map_err(io)?)
        || !fs::metadata(&script).map_err(io)?.is_file()
    {
        return Err(failure("package bin is missing or escapes package root"));
    }
    Ok(script)
}

/// Caller owns both roots and checks inventories before/after. Interpret every
/// path before writing. Only target runtime.json and its launcher may change.
/// Historical backup payloads and their recorded absolute paths stay intact.
pub(crate) fn prepare(source: &Path, target: &Path) -> AppResult<()> {
    let record_path = source.join("manager-state").join(RUNTIME_SOURCE_FILENAME);
    let entry = source.join(MANAGED_ENTRY_RELATIVE);
    let has_record = match fs::symlink_metadata(&record_path) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(io(e)),
    };
    let has_entry = match fs::symlink_metadata(&entry) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(io(e)),
    };
    if !has_record {
        return if has_entry {
            Err(failure("managed launcher has no runtime record"))
        } else {
            Ok(())
        };
    }
    let (record, mut json) = strict_record(&read(&record_path, 1024 * 1024)?)?;
    if record.source == RuntimeSourceKind::Managed {
        let path = record
            .path
            .as_ref()
            .ok_or_else(|| failure("managed runtime has no recorded path"))?;
        if resolved_path(Path::new(path))? != entry.canonicalize().map_err(io)? {
            return Err(failure("managed runtime path does not match its launcher"));
        }
    }
    if record.source == RuntimeSourceKind::Managed && !has_entry {
        return Err(failure("managed runtime has no launcher"));
    }
    let launcher = if has_entry {
        let prefix = record.installed_prefix(source);
        let script = package_script(&prefix)?;
        let content = String::from_utf8(read(&entry, 64 * 1024)?)
            .map_err(|_| failure("launcher is not UTF-8"))?;
        let node = recorded_node(&content, &script)?
            .map(|path| relocate(&path, source, target))
            .transpose()?;
        let relocated_script = relocate(&script, source, target)?;
        if !fs::metadata(&relocated_script).map_err(io)?.is_file() {
            return Err(failure("relocated package bin is missing"));
        }
        Some(install::entry_script(node.as_deref(), &relocated_script))
    } else {
        None
    };
    if let Some(path) = record.path.as_ref() {
        json["path"] = relocate(Path::new(path), source, target)?
            .to_string_lossy()
            .into_owned()
            .into();
    }
    if let Some(history) = json.get_mut("history").and_then(|h| h.as_array_mut()) {
        for entry in history {
            let path = entry["target"]
                .as_str()
                .ok_or_else(|| failure("invalid history target"))?;
            if !path.is_empty() {
                entry["target"] = relocate(Path::new(path), source, target)?
                    .to_string_lossy()
                    .into_owned()
                    .into();
            }
        }
    }
    let bytes = serde_json::to_vec_pretty(&json).map_err(|e| failure(e.to_string()))?;
    let destination = target.join("manager-state").join(RUNTIME_SOURCE_FILENAME);
    crate::infrastructure::atomic_write::atomic_write(&destination, &bytes, 0o600)?;
    if read(&destination, 1024 * 1024)? != bytes {
        return Err(failure("relocated runtime record failed readback"));
    }
    if let Some(content) = launcher {
        let destination = target.join(MANAGED_ENTRY_RELATIVE);
        crate::infrastructure::atomic_write::atomic_write(&destination, content.as_bytes(), 0o755)?;
        if read(&destination, 64 * 1024)? != content.as_bytes() {
            return Err(failure("relocated launcher failed readback"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::data_root::{self, migration};
    use serde_json::json;

    fn fixture(
        external: bool,
    ) -> (
        tempfile::TempDir,
        data_root::DataRootRuntimeConfig,
        PathBuf,
        PathBuf,
        PathBuf,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source 中文 ' percent%");
        data_root::initialize(&source).unwrap();
        let source = source.canonicalize().unwrap();
        let prefix = if external {
            temp.path()
                .canonicalize()
                .unwrap()
                .join("external prefix ' %")
        } else {
            source.join("runtime/opencodex")
        };
        let node = if external {
            temp.path()
                .canonicalize()
                .unwrap()
                .join("external node ' %")
        } else {
            source.join("runtime/node ' %")
        };
        let package = prefix.join(install::PACKAGE_SUBPATH);
        fs::create_dir_all(&package).unwrap();
        let script = package.join("cli.js");
        fs::write(&script, b"// not executed").unwrap();
        fs::write(&node, b"not executed").unwrap();
        fs::write(
            package.join("package.json"),
            serde_json::to_vec(&json!({
                "name": super::super::OFFICIAL_PACKAGE, "version": "2.50.0", "bin": "cli.js"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(prefix.join(MANIFEST_FILENAME), serde_json::to_vec(&json!({
            "package": super::super::OFFICIAL_PACKAGE, "version":"2.50.0", "tarball_sha256":"fixture",
            "installed_at":"2026-10-10T00:00:00Z", "npm_path":"external", "scripts_enabled":false, "source":"registry"
        })).unwrap()).unwrap();
        fs::create_dir_all(source.join("runtime/bin")).unwrap();
        fs::write(
            source.join(MANAGED_ENTRY_RELATIVE),
            install::entry_script(Some(&node), &script),
        )
        .unwrap();
        fs::write(source.join("manager-state/runtime.json"), serde_json::to_vec(&json!({
            "schema_version":1, "source":"managed", "path":source.join(MANAGED_ENTRY_RELATIVE),
            "resolved_version":"2.50.0", "resolved_at":"2026-10-10T00:00:00Z", "history":[{
                "action":"install", "result":"succeeded", "package":super::super::OFFICIAL_PACKAGE,
                "version":"2.50.0", "target":prefix, "at":"2026-10-10T00:00:00Z"
            }]
        })).unwrap()).unwrap();
        let config = data_root::load_runtime_config(&source).unwrap();
        let target = temp.path().join("target 中文 ' %");
        (temp, config, target, script, node)
    }

    #[test]
    fn managed_paths_relocate_but_source_and_backups_remain_identical() {
        let (_temp, config, target, script, node) = fixture(false);
        let source = &config.active_data_root;
        fs::write(
            source.join("backups/historical.json"),
            br#"{"path":"old absolute path"}"#,
        )
        .unwrap();
        let original = fs::read(source.join("manager-state/runtime.json")).unwrap();
        let mut copy = migration::copy_verified(&config, &target).unwrap();
        let original_inventory = copy.receipt.entries.clone();
        copy.prepare_runtime().unwrap();
        let target = target.canonicalize().unwrap();
        assert_eq!(copy.receipt.phase, "runtime_prepared");
        assert_eq!(copy.receipt.entries, original_inventory);
        assert_eq!(
            fs::read(source.join("manager-state/runtime.json")).unwrap(),
            original
        );
        assert_eq!(
            fs::read(target.join("backups/historical.json")).unwrap(),
            fs::read(source.join("backups/historical.json")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(target.join(MANAGED_ENTRY_RELATIVE)).unwrap(),
            install::entry_script(
                Some(&target.join(node.strip_prefix(source).unwrap())),
                &target.join(script.strip_prefix(source).unwrap())
            )
        );
        let record: RuntimeSourceRecord =
            serde_json::from_slice(&fs::read(target.join("manager-state/runtime.json")).unwrap())
                .unwrap();
        assert_eq!(
            record.path,
            Some(
                target
                    .join(MANAGED_ENTRY_RELATIVE)
                    .to_string_lossy()
                    .into_owned()
            )
        );
        assert_eq!(
            record.history[0].target,
            target.join("runtime/opencodex").to_string_lossy()
        );
        assert!(migration::require_published(&target).is_err());
        assert!(copy.prepare_runtime().is_err());
    }

    #[test]
    fn external_prefix_and_node_remain_external_without_writes() {
        let (_temp, config, target, script, node) = fixture(true);
        let old = fs::read(config.active_data_root.join(MANAGED_ENTRY_RELATIVE)).unwrap();
        let mut copy = migration::copy_verified(&config, &target).unwrap();
        copy.prepare_runtime().unwrap();
        assert_eq!(fs::read(target.join(MANAGED_ENTRY_RELATIVE)).unwrap(), old);
        assert_eq!(fs::read(&node).unwrap(), b"not executed");
        assert_eq!(fs::read(&script).unwrap(), b"// not executed");
        let record: RuntimeSourceRecord =
            serde_json::from_slice(&fs::read(target.join("manager-state/runtime.json")).unwrap())
                .unwrap();
        assert_eq!(
            record.installed_prefix(&target),
            script.ancestors().nth(4).unwrap()
        );
    }

    #[test]
    fn histories_are_not_truncated_and_absent_old_internal_paths_relocate() {
        let (_temp, config, target, _script, _node) = fixture(false);
        let record_path = config.active_data_root.join("manager-state/runtime.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();
        let last = value["history"][0].clone();
        let mut history = Vec::new();
        for index in 0..60 {
            let mut entry = last.clone();
            entry["target"] = config
                .active_data_root
                .join(format!("runtime/absent-{index}"))
                .to_string_lossy()
                .into_owned()
                .into();
            entry["result"] = "failed".into();
            history.push(entry);
        }
        history.push(last);
        value["history"] = history.into();
        fs::write(&record_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let mut copy = migration::copy_verified(&config, &target).unwrap();
        copy.prepare_runtime().unwrap();
        let changed: serde_json::Value =
            serde_json::from_slice(&fs::read(target.join("manager-state/runtime.json")).unwrap())
                .unwrap();
        assert_eq!(changed["history"].as_array().unwrap().len(), 61);
        assert_eq!(changed["resolved_at"], value["resolved_at"]);
        assert_eq!(
            changed["history"][0]["target"],
            target
                .canonicalize()
                .unwrap()
                .join("runtime/absent-0")
                .to_string_lossy()
                .as_ref()
        );
    }

    #[test]
    fn invalid_records_or_launchers_do_not_write_target_runtime_files() {
        for case in [
            "schema", "unknown", "corrupt", "edited", "path", "package", "escape",
        ] {
            let (_temp, config, target, script, _node) = fixture(false);
            let path = config.active_data_root.join("manager-state/runtime.json");
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match case {
                "schema" => value["schema_version"] = 2.into(),
                "unknown" => value["future"] = true.into(),
                "corrupt" => {}
                "path" => value["path"] = script.to_string_lossy().into_owned().into(),
                "edited" => {
                    let entry = config.active_data_root.join(MANAGED_ENTRY_RELATIVE);
                    let mut content = fs::read_to_string(&entry).unwrap();
                    content.push_str("echo injected\n");
                    fs::write(entry, content).unwrap();
                }
                "package" | "escape" => {
                    let package = script.parent().unwrap().join("package.json");
                    let mut value: serde_json::Value =
                        serde_json::from_slice(&fs::read(&package).unwrap()).unwrap();
                    if case == "package" {
                        value["version"] = "9.9.9".into();
                    } else {
                        value["bin"] = "../outside.js".into();
                    }
                    fs::write(package, serde_json::to_vec(&value).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            fs::write(
                &path,
                if case == "corrupt" {
                    b"{".to_vec()
                } else {
                    serde_json::to_vec(&value).unwrap()
                },
            )
            .unwrap();
            let mut copy = migration::copy_verified(&config, &target).unwrap();
            let before = fs::read(target.join("manager-state/runtime.json")).unwrap();
            let launcher = fs::read(target.join(MANAGED_ENTRY_RELATIVE)).unwrap();
            assert!(copy.prepare_runtime().is_err(), "{case}");
            assert_eq!(
                fs::read(target.join("manager-state/runtime.json")).unwrap(),
                before,
                "{case}"
            );
            assert_eq!(
                fs::read(target.join(MANAGED_ENTRY_RELATIVE)).unwrap(),
                launcher,
                "{case}"
            );
            assert_eq!(copy.receipt.phase, "verified_copy");
            assert!(migration::require_published(&target).is_err());
        }
    }

    #[test]
    fn no_runtime_is_noop_and_changed_inventory_or_marker_refuses_preparation() {
        for case in [
            "none",
            "source_changed",
            "target_changed",
            "marker_missing",
            "receipt_changed",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source");
            data_root::initialize(&source).unwrap();
            let config = data_root::load_runtime_config(&source).unwrap();
            let target = temp.path().join("target");
            let mut copy = migration::copy_verified(&config, &target).unwrap();
            match case {
                "source_changed" => {
                    fs::write(source.join("manager-state/new"), b"changed").unwrap()
                }
                "target_changed" => {
                    fs::write(target.join("manager-state/new"), b"changed").unwrap()
                }
                "marker_missing" => {
                    fs::remove_file(target.join(migration::UNPUBLISHED_MARKER)).unwrap()
                }
                "receipt_changed" => {
                    fs::write(target.join(migration::RECEIPT_PATH), b"changed").unwrap()
                }
                _ => {}
            }
            assert_eq!(copy.prepare_runtime().is_ok(), case == "none", "{case}");
        }
    }

    #[test]
    fn generated_launcher_fallback_and_quoted_paths_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let node = temp
            .path()
            .canonicalize()
            .unwrap()
            .join("node 中文 ' % ! space");
        let script = temp
            .path()
            .canonicalize()
            .unwrap()
            .join("cli 中文 ' % ! space.js");
        assert_eq!(
            recorded_node(&install::entry_script(Some(&node), &script), &script).unwrap(),
            Some(node)
        );
        assert_eq!(
            recorded_node(&install::entry_script(None, &script), &script).unwrap(),
            None
        );
        assert!(recorded_node("malformed", &script).is_err());
    }
}
