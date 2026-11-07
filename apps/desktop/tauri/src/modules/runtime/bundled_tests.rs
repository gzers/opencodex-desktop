use super::*;
use serde_json::{json, Value};
use std::os::unix::fs::PermissionsExt;

const VERSION: &str = "2.97.0-desktop023.names.1";
const PACKAGE: &str = "runtime/opencodex/node_modules/@bitkyc08/opencodex";
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    i: PathBuf,
    d: PathBuf,
    archive: PathBuf,
    rows: Vec<(Value, Vec<u8>)>,
    descriptor: Value,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let archive = root.join("runtime-bundle-0.2.23-darwin-arm64.tar.gz");
        let mut rows = Vec::new();
        let mut file = |p: String, data: Vec<u8>, mode| {
            rows.push((json!({"path":p,"kind":"file","bytes":data.len(),"sha256":hash::sha256_hex(&data),"mode":mode}), data));
        };
        // Fake executable bytes are never executed, including for ownership/version detection.
        file(
            "runtime/node/bin/node".into(),
            b"#!/bin/sh\nexit 93\n".to_vec(),
            0o755,
        );
        file(
            "runtime/node/lib/node_modules/npm/bin/npm-cli.js".into(),
            b"throw Error('never execute npm');".to_vec(),
            0o644,
        );
        file(
            "runtime/bun/bun".into(),
            b"#!/bin/sh\nexit 94\n".to_vec(),
            0o755,
        );
        file(
            "runtime/opencodex/package.json".into(),
            br#"{"private":true}"#.to_vec(),
            0o644,
        );
        file(
            "runtime/opencodex/package-lock.json".into(),
            br#"{"lockfileVersion":3}"#.to_vec(),
            0o644,
        );
        file(format!("{PACKAGE}/package.json"), serde_json::to_vec(&json!({"name":OFFICIAL_PACKAGE,
            "version":VERSION,"bin":{"ocx":"./bin/actual.cjs"},"scripts":{"postinstall":"never run"}})).unwrap(), 0o644);
        file(
            format!("{PACKAGE}/bin/actual.cjs"),
            b"#!/usr/bin/env node\nthrow Error('not a version probe');\n".to_vec(),
            0o755,
        );
        file(
            "runtime/opencodex/node_modules/dependency/data.json".into(),
            b"{}".to_vec(),
            0o644,
        );
        file(
            format!("runtime/opencodex/sources/opencodex-{VERSION}.tgz"),
            b"maintenance source tgz fixture".to_vec(),
            0o644,
        );
        rows.push((json!({"path":"runtime/node/bin/npm","kind":"symlink","target":"../lib/node_modules/npm/bin/npm-cli.js"}), vec![]));
        let descriptor = json!({"schemaVersion":1,"platform":"darwin-arm64","runtimeVersion":VERSION,
            "nodeVersion":"26.8.1","bunVersion":"1.4.4",
            "runtimeArchiveSha256":hash::sha256_hex(b"maintenance source tgz fixture"),
            "archiveName":archive.file_name().unwrap().to_str().unwrap(),"archiveSha256":"","archiveBytes":0,"entries":[]});
        let mut f = Self {
            i: root.join("installed"),
            d: root.join("user-a/desktop"),
            root,
            _temp: temp,
            archive,
            rows,
            descriptor,
        };
        f.repack();
        f
    }
    fn repack(&mut self) {
        self.rows
            .sort_by_key(|row| row.0["path"].as_str().unwrap().to_string());
        let rows = self.rows.clone();
        self.pack_rows(&rows);
    }
    fn pack_rows(&mut self, rows: &[(Value, Vec<u8>)]) {
        let gzip = flate2::write::GzEncoder::new(
            File::create(&self.archive).unwrap(),
            flate2::Compression::default(),
        );
        let mut tar = tar::Builder::new(gzip);
        for (row, bytes) in rows {
            let mut header = tar::Header::new_ustar();
            header.set_path(row["path"].as_str().unwrap()).unwrap();
            if row["kind"] == "symlink" {
                header.set_entry_type(tar::EntryType::Symlink);
                header
                    .set_link_name(row["target"].as_str().unwrap())
                    .unwrap();
                header.set_mode(0o777);
                header.set_size(0);
            } else {
                header.set_entry_type(tar::EntryType::Regular);
                header.set_mode(row["mode"].as_u64().unwrap() as u32);
                header.set_size(bytes.len() as u64);
            }
            header.set_cksum();
            tar.append(&header, bytes.as_slice()).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
        self.descriptor["entries"] = Value::Array(self.rows.iter().map(|r| r.0.clone()).collect());
        self.pin_archive();
    }
    fn pin_archive(&mut self) {
        self.descriptor["archiveBytes"] = json!(fs::metadata(&self.archive).unwrap().len());
        self.descriptor["archiveSha256"] = json!(hash::sha256_file(&self.archive).unwrap());
    }
    fn bytes(&self) -> (Vec<u8>, String) {
        let bytes = serde_json::to_vec(&self.descriptor).unwrap();
        let digest = hash::sha256_hex(&bytes);
        (bytes, digest)
    }
    fn provision(&self) -> AppResult<ProvisionResult> {
        let (bytes, digest) = self.bytes();
        provision(&self.i, &self.d, &self.archive, &bytes, &digest)
    }
    fn verify(&self) -> AppResult<BundleVerification> {
        let (bytes, digest) = self.bytes();
        verify_bundle(&self.archive, &bytes, &digest)
    }
    fn change_file(&mut self, suffix: &str, bytes: Vec<u8>) {
        let row = self
            .rows
            .iter_mut()
            .find(|r| r.0["path"].as_str().unwrap().ends_with(suffix))
            .unwrap();
        row.0["sha256"] = json!(hash::sha256_hex(&bytes));
        row.0["bytes"] = json!(bytes.len());
        row.1 = bytes;
        self.repack();
    }
}
fn tree_snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, u32)> {
    let mut snapshot = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            let key = path.strip_prefix(root).unwrap().to_path_buf();
            if meta.is_dir() {
                stack.push(path);
                continue;
            }
            let bytes = if meta.file_type().is_symlink() {
                fs::read_link(&path)
                    .unwrap()
                    .as_os_str()
                    .as_encoded_bytes()
                    .to_vec()
            } else {
                fs::read(&path).unwrap()
            };
            snapshot.insert(key, (bytes, meta.permissions().mode()));
        }
    }
    snapshot
}

#[test]
fn complete_tree_install_reopen_and_second_user_preserve_program_and_home() {
    let f = Fixture::new();
    let home = f.root.join("opencodex/home");
    mkdir(&home).unwrap();
    fs::write(home.join("sentinel"), b"user config must survive").unwrap();
    assert_eq!(f.verify().unwrap().version, VERSION);
    assert!(!f.i.exists(), "preview must not create installation root");
    assert_eq!(
        f.provision().unwrap(),
        ProvisionResult::Installed {
            version: VERSION.into()
        }
    );
    let snapshot = tree_snapshot(&f.i);
    assert_eq!(
        f.provision().unwrap(),
        ProvisionResult::AlreadyInstalled {
            version: VERSION.into()
        }
    );
    let second = f.root.join("user-b/desktop");
    let (bytes, pin) = f.bytes();
    fs::set_permissions(&f.i, fs::Permissions::from_mode(0o500)).unwrap();
    let result = provision(&f.i, &second, &f.archive, &bytes, &pin);
    fs::set_permissions(&f.i, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        result.unwrap(),
        ProvisionResult::AlreadyInstalled {
            version: VERSION.into()
        }
    );
    assert_eq!(snapshot, tree_snapshot(&f.i));
    assert_eq!(
        fs::read(home.join("sentinel")).unwrap(),
        b"user config must survive"
    );
    assert!(!f.d.join("runtime").exists());
    for d in [&f.d, &second] {
        let record = RuntimeStore::new(d).load();
        assert_eq!(record.source, RuntimeSourceKind::Managed);
        assert_eq!(record.resolved_version.as_deref(), Some(VERSION));
        assert_eq!(
            record.path,
            Some(f.i.join(MANAGED_ENTRY_RELATIVE).to_string_lossy().into())
        );
        assert_eq!(
            fs::metadata(RuntimeStore::new(d).path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let manifest: install::RuntimeManifest = serde_json::from_slice(
        &fs::read(f.i.join(MANAGED_PREFIX_RELATIVE).join(MANIFEST_FILENAME)).unwrap(),
    )
    .unwrap();
    assert!(!manifest.scripts_enabled);
    assert_eq!(manifest.source, install::InstallSourceKind::Offline);
    assert_eq!(
        manifest.tarball_sha256,
        f.descriptor["runtimeArchiveSha256"]
    );
    assert_eq!(
        manifest.npm_path,
        install::managed_dependency_paths(&f.i).1.to_string_lossy()
    );
    assert!(manifest.activation.is_some());
    install::validate_managed_installation(&f.i, Some(&install::managed_dependency_paths(&f.i).0))
        .unwrap();
    assert_eq!(
        fs::metadata(f.i.join("runtime"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
}

#[test]
fn pinned_descriptor_and_archive_tamper_rejected_without_program_or_store() {
    let mut f = Fixture::new();
    let (bytes, pin) = f.bytes();
    assert!(provision(&f.i, &f.d, &f.archive, &bytes, &"f".repeat(64)).is_err());
    assert!(verify_bundle(&f.archive, &bytes, &format!("sha256:{pin}")).is_err());
    f.descriptor["runtimeVersion"] = json!("evil");
    assert!(verify_bundle(
        &f.archive,
        &serde_json::to_vec(&f.descriptor).unwrap(),
        &pin
    )
    .is_err());
    let mut bytes_archive = fs::read(&f.archive).unwrap();
    *bytes_archive.last_mut().unwrap() ^= 1;
    fs::write(&f.archive, bytes_archive).unwrap();
    assert!(f.provision().is_err());
    assert!(!f.i.join("runtime").exists());
    assert!(!RuntimeStore::new(&f.d).path().exists());
}

#[test]
fn tar_missing_extra_duplicate_and_mode_or_hash_drift_rejected() {
    for case in 0..5 {
        let mut f = Fixture::new();
        let mut rows = f.rows.clone();
        match case {
            0 => {
                rows.pop();
            }
            1 => {
                let mut e = rows[0].clone();
                e.0["path"] = json!("runtime/extra");
                rows.push(e);
            }
            2 => rows.push(rows[0].clone()),
            3 => {
                let e = rows.iter_mut().find(|e| e.0["kind"] == "file").unwrap();
                e.0["mode"] = json!(0o600);
            }
            _ => {
                let e = rows.iter_mut().find(|e| !e.1.is_empty()).unwrap();
                e.1[0] ^= 1;
            }
        }
        f.pack_rows(&rows);
        assert!(f.verify().is_err(), "case {case}");
        assert!(f.provision().is_err(), "case {case}");
        assert!(!f.i.join("runtime").exists());
        assert!(!f.d.exists());
    }
}

#[test]
fn unsafe_links_cycles_missing_targets_and_parent_link_paths_rejected() {
    for target in [
        "/etc/passwd",
        "../../escape",
        "../absent",
        "../bin/npm",
        "C:\\escape",
        "../bin/./node",
        "missing-directory/../node",
    ] {
        let mut f = Fixture::new();
        let row = f
            .rows
            .iter_mut()
            .find(|e| e.0["kind"] == "symlink")
            .unwrap();
        row.0["target"] = json!(target);
        f.repack();
        assert!(f.verify().is_err(), "target {target}");
    }
    let mut f = Fixture::new();
    f.rows.push((json!({"path":"runtime/node/bin/npm/child","kind":"file","bytes":0,"sha256":hash::sha256_hex(b""),"mode":0o644}), vec![]));
    f.repack();
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    f.rows.push((
        json!({"path":"runtime/node/alias","kind":"symlink","target":".."}),
        vec![],
    ));
    let row = f
        .rows
        .iter_mut()
        .find(|e| e.0["path"] == "runtime/node/bin/npm")
        .unwrap();
    row.0["target"] = json!("../alias/../node/bin/node");
    f.repack();
    assert!(
        f.verify().is_err(),
        "a/.. must not traverse a link before lexical collapse"
    );
}

#[test]
fn descriptor_duplicate_dot_absolute_hardlink_special_and_reserved_entries_rejected() {
    for path in [
        "runtime/../escape",
        "runtime/./dot",
        "/runtime/absolute",
        "runtime//double",
        "runtime/opencodex/.runtime-manifest.json",
        "runtime/bin/ocx",
        "runtime/NUL",
        "runtime/trailing.",
    ] {
        let mut f = Fixture::new();
        f.descriptor["entries"][0]["path"] = json!(path);
        assert!(f.verify().is_err(), "path {path}");
    }
    for kind in ["hardlink", "directory", "fifo", "device"] {
        let mut f = Fixture::new();
        f.descriptor["entries"][0]["kind"] = json!(kind);
        assert!(f.verify().is_err());
    }
    let mut f = Fixture::new();
    let row = f.descriptor["entries"][0].clone();
    f.descriptor["entries"].as_array_mut().unwrap().push(row);
    assert!(f.verify().is_err());
}

#[test]
fn compressed_single_file_total_and_count_limits_fail_without_truncation() {
    let mut f = Fixture::new();
    f.descriptor["archiveBytes"] = json!(MAX_ARCHIVE + 1);
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    f.descriptor["entries"][0]["bytes"] = json!(MAX_FILE + 1);
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    for row in f.descriptor["entries"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|r| r["kind"] == "file")
        .take(5)
    {
        row["bytes"] = json!(MAX_FILE);
    }
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    f.descriptor["entries"] = json!(vec![f.descriptor["entries"][0].clone(); MAX_ENTRIES + 1]);
    assert!(f.verify().is_err());
}

#[test]
fn package_name_version_and_bin_identity_must_match_without_execution() {
    for package in [
        json!({"name":"wrong","version":VERSION,"bin":"bin/actual.cjs"}),
        json!({"name":OFFICIAL_PACKAGE,"version":"0.0.1","bin":"bin/actual.cjs"}),
        json!({"name":OFFICIAL_PACKAGE,"version":VERSION,"bin":"../outside"}),
        json!({"name":OFFICIAL_PACKAGE,"version":VERSION,"bin":"bin/missing"}),
    ] {
        let mut f = Fixture::new();
        f.change_file(
            "@bitkyc08/opencodex/package.json",
            serde_json::to_vec(&package).unwrap(),
        );
        assert!(f.verify().is_err());
        assert!(f.provision().is_err());
        assert!(!f.i.join("runtime").exists());
    }
}

#[test]
fn existing_valid_version_retained_even_if_bundle_is_unavailable_or_older() {
    let f = Fixture::new();
    f.provision().unwrap();
    let before = tree_snapshot(&f.i);
    fs::remove_file(&f.archive).unwrap();
    assert_eq!(
        provision(&f.i, &f.d, &f.archive, b"invalid", "bad").unwrap(),
        ProvisionResult::AlreadyInstalled {
            version: VERSION.into()
        }
    );
    assert_eq!(tree_snapshot(&f.i), before);
}

#[test]
fn existing_unknown_empty_corrupt_and_missing_manifest_are_preserved() {
    for case in 0..4 {
        let f = Fixture::new();
        if case < 2 {
            mkdir(&f.i.join("runtime")).unwrap();
            if case == 1 {
                fs::write(f.i.join("runtime/unknown"), b"preserve").unwrap();
            }
        } else {
            f.provision().unwrap();
            let manifest = f.i.join(MANAGED_PREFIX_RELATIVE).join(MANIFEST_FILENAME);
            if case == 2 {
                fs::write(manifest, b"broken").unwrap();
            } else {
                fs::remove_file(manifest).unwrap();
            }
        }
        let before = tree_snapshot(&f.i);
        assert!(f.provision().is_err(), "case {case}");
        assert_eq!(tree_snapshot(&f.i), before);
    }
}

#[test]
fn activation_failure_compensates_only_own_install_and_allows_retry() {
    let f = Fixture::new();
    let (bytes, pin) = f.bytes();
    assert!(provision_inner(&f.i, &f.d, &f.archive, &bytes, &pin, true, None).is_err());
    assert!(!f.i.join("runtime").exists());
    assert!(!f.i.join(PENDING).exists());
    assert!(!f.d.exists());
    let failure: Value = serde_json::from_slice(&fs::read(f.i.join(FAILURE)).unwrap()).unwrap();
    assert_eq!(failure["compensated"], true);
    assert!(failure["before"].is_null());
    assert!(matches!(
        f.provision().unwrap(),
        ProvisionResult::Installed { .. }
    ));
}

#[test]
fn interrupted_pending_checks_only_named_stage_and_never_adopts_tree() {
    let f = Fixture::new();
    f.provision().unwrap();
    let stage = ".runtime-bundle-stage-interrupted";
    mkdir(&f.i.join(stage)).unwrap();
    fs::write(f.i.join(stage).join("sentinel"), b"preserve").unwrap();
    save_pending(
        &f.i,
        &Pending {
            schema_version: 1,
            attempt_id: uuid::Uuid::new_v4().to_string(),
            stage: stage.into(),
            descriptor_sha256: "a".repeat(64),
            phase: "activating".into(),
            before: None,
            at: now_rfc3339(),
        },
    )
    .unwrap();
    let before = tree_snapshot(&f.i);
    let error = f.provision().unwrap_err().to_string();
    assert!(error.contains(stage));
    assert_eq!(tree_snapshot(&f.i), before);
}

#[test]
fn separated_absolute_roots_reject_overlap_traversal_and_symlinks() {
    let f = Fixture::new();
    for (i, d) in [
        (f.i.clone(), f.i.clone()),
        (f.i.clone(), f.i.join("data")),
        (f.d.join("program"), f.d.clone()),
        (PathBuf::from("relative"), f.d.clone()),
        (f.root.join("foo/../installed"), f.d.clone()),
        (f.root.join("./installed"), f.d.clone()),
    ] {
        let (bytes, pin) = f.bytes();
        assert!(provision(&i, &d, &f.archive, &bytes, &pin).is_err());
    }
    mkdir(&f.root.join("actual")).unwrap();
    std::os::unix::fs::symlink(f.root.join("actual"), f.root.join("alias")).unwrap();
    let (bytes, pin) = f.bytes();
    assert!(provision(
        &f.root.join("alias/program"),
        &f.d,
        &f.archive,
        &bytes,
        &pin
    )
    .is_err());
}

#[test]
fn hardlinked_archive_lock_or_installed_file_refused_without_changing_alias() {
    let f = Fixture::new();
    let alias = f.root.join("hardlink.tgz");
    fs::hard_link(&f.archive, &alias).unwrap();
    assert!(f.provision().is_err());
    fs::remove_file(alias).unwrap();
    mkdir(&f.i).unwrap();
    fs::remove_file(f.i.join(".runtime-bundle.lock")).unwrap();
    fs::write(f.root.join("sentinel"), b"keep").unwrap();
    fs::hard_link(f.root.join("sentinel"), f.i.join(".runtime-bundle.lock")).unwrap();
    assert!(f.provision().is_err());
    assert_eq!(fs::read(f.root.join("sentinel")).unwrap(), b"keep");
    fs::remove_file(f.i.join(".runtime-bundle.lock")).unwrap();
    f.provision().unwrap();
    fs::hard_link(f.i.join("runtime/node/bin/node"), f.root.join("node-alias")).unwrap();
    let before = tree_snapshot(&f.i);
    assert!(f.provision().is_err());
    assert_eq!(tree_snapshot(&f.i), before);
}

#[test]
fn publish_never_replaces_even_an_empty_unknown_directory() {
    let f = Fixture::new();
    let src = f.root.join("source");
    let dst = f.root.join("target");
    mkdir(&src).unwrap();
    mkdir(&dst).unwrap();
    assert!(publish(&src, &dst).is_err());
    assert!(src.is_dir() && dst.is_dir());
}

#[test]
fn corrupt_user_store_preserved_and_program_can_be_adopted_after_explicit_fix() {
    let f = Fixture::new();
    mkdir(&f.d.join("manager-state")).unwrap();
    let path = RuntimeStore::new(&f.d).path().to_path_buf();
    fs::write(&path, b"corrupt preserve").unwrap();
    assert!(f.provision().is_err());
    install::validate_managed_installation(&f.i, Some(&install::managed_dependency_paths(&f.i).0))
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"corrupt preserve");
    fs::remove_file(path).unwrap();
    assert!(matches!(
        f.provision().unwrap(),
        ProvisionResult::AlreadyInstalled { .. }
    ));
}

fn rewrite_raw_tar(f: &mut Fixture, mutate: impl FnOnce(&mut Vec<u8>)) {
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(File::open(&f.archive).unwrap())
        .read_to_end(&mut bytes)
        .unwrap();
    mutate(&mut bytes);
    let mut gzip = flate2::write::GzEncoder::new(
        File::create(&f.archive).unwrap(),
        flate2::Compression::default(),
    );
    gzip.write_all(&bytes).unwrap();
    gzip.finish().unwrap();
    f.pin_archive();
}
#[test]
fn raw_tar_hardlink_fifo_device_pax_gnu_and_dot_paths_refused() {
    for kind in *b"1634xgLKS" {
        let mut f = Fixture::new();
        rewrite_raw_tar(&mut f, |bytes| {
            let mut h = tar::Header::new_ustar();
            h.as_mut_bytes().copy_from_slice(&bytes[..512]);
            h.as_mut_bytes()[156] = kind;
            h.set_cksum();
            bytes[..512].copy_from_slice(h.as_bytes());
        });
        assert!(f.verify().is_err(), "raw type {}", kind as char);
        assert!(f.provision().is_err());
        assert!(!f.i.join("runtime").exists());
    }
    for path in [
        "/runtime/bun/bun",
        "runtime/./bun/bun",
        "runtime/../outside",
        "runtime//bun/bun",
    ] {
        let mut f = Fixture::new();
        rewrite_raw_tar(&mut f, |bytes| {
            let mut h = tar::Header::new_ustar();
            h.as_mut_bytes().copy_from_slice(&bytes[..512]);
            h.as_mut_bytes()[..100].fill(0);
            h.as_mut_bytes()[..path.len()].copy_from_slice(path.as_bytes());
            h.set_cksum();
            bytes[..512].copy_from_slice(h.as_bytes());
        });
        assert!(f.verify().is_err(), "raw path {path}");
    }
}

#[test]
fn hidden_tar_after_end_marker_and_broken_gzip_crc_refused() {
    let mut f = Fixture::new();
    rewrite_raw_tar(&mut f, |bytes| {
        let hidden = bytes[..512].to_vec();
        bytes.extend_from_slice(&hidden);
    });
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    let mut bytes = fs::read(&f.archive).unwrap();
    let n = bytes.len();
    bytes[n - 8] ^= 1;
    fs::write(&f.archive, bytes).unwrap();
    f.pin_archive();
    assert!(f.verify().is_err());
}

#[test]
fn unexpected_published_content_prevents_compensation_and_is_preserved() {
    let f = Fixture::new();
    f.provision().unwrap();
    let stage = tempfile::Builder::new()
        .prefix(".runtime-bundle-stage-")
        .tempdir_in(&f.i)
        .unwrap();
    let (bytes, pin) = f.bytes();
    let plan = load(&bytes, &pin).unwrap();
    let owner = read(&f.i.join("runtime").join(OWNER), 64 * 1024).unwrap();
    let generated = BTreeMap::from([(format!("runtime/{OWNER}"), owner)]);
    fs::write(f.i.join("runtime/unknown-new-content"), b"preserve").unwrap();
    let before = tree_snapshot(&f.i.join("runtime"));
    assert!(compensate(
        &f.i,
        stage.path(),
        Some(directory_identity(&f.i.join("runtime")).unwrap()),
        &generated,
        &plan
    )
    .is_err());
    assert_eq!(before, tree_snapshot(&f.i.join("runtime")));
}

#[test]
fn staging_must_be_exact_before_atomic_publication() {
    let f = Fixture::new();
    let stage = tempfile::tempdir().unwrap();
    let stage_path = stage.path().canonicalize().unwrap();
    let (bytes, pin) = f.bytes();
    let plan = load(&bytes, &pin).unwrap();
    unpack(&f.archive, &plan, Some(&stage_path)).unwrap();
    verify_staged(&stage_path, &plan).unwrap();
    fs::write(stage_path.join("runtime/unexpected"), b"x").unwrap();
    assert!(verify_staged(&stage_path, &plan).is_err());
    fs::remove_file(stage_path.join("runtime/unexpected")).unwrap();
    fs::write(stage_path.join("runtime/node/bin/node"), b"changed").unwrap();
    assert!(verify_staged(&stage_path, &plan).is_err());
}

#[test]
fn descriptor_maintenance_source_hash_must_match_the_copied_tgz() {
    let mut f = Fixture::new();
    f.descriptor["runtimeArchiveSha256"] = json!("b".repeat(64));
    assert!(f.verify().is_err());
    let mut f = Fixture::new();
    f.rows
        .retain(|r| !r.0["path"].as_str().unwrap().contains("/sources/"));
    f.repack();
    assert!(f.verify().is_err());
}

#[test]
fn exclusive_installation_lock_refuses_another_first_installer() {
    let f = Fixture::new();
    mkdir(&f.i).unwrap();
    let lock = safety::open_file(&f.i.join(".runtime-bundle.lock"), true).unwrap();
    lock.try_lock().unwrap();
    assert!(f.provision().is_err());
    assert!(!f.i.join("runtime").exists());
    drop(lock);
    assert!(matches!(
        f.provision().unwrap(),
        ProvisionResult::Installed { .. }
    ));
}

#[test]
fn windows_descriptor_preview_uses_real_builder_paths_and_preserves_file_modes() {
    let mut f = Fixture::new();
    f.descriptor["platform"] = json!("win32-x64");
    f.rows.retain(|r| r.0["kind"] != "symlink");
    for (row, _) in &mut f.rows {
        let path = row["path"].as_str().unwrap().to_string();
        if path == "runtime/node/bin/node" {
            row["path"] = json!("runtime/node/node.exe");
            row["mode"] = json!(0o644);
        }
        if path == "runtime/node/lib/node_modules/npm/bin/npm-cli.js" {
            row["path"] = json!("runtime/node/node_modules/npm/bin/npm-cli.js");
        }
        if path == "runtime/bun/bun" {
            row["path"] = json!("runtime/bun/bun.exe");
            row["mode"] = json!(0o644);
        }
        if path.ends_with("bin/actual.cjs") {
            row["mode"] = json!(0o644);
        }
    }
    f.repack();
    assert_eq!(f.verify().unwrap().platform, "win32-x64");
    assert!(
        f.provision().is_err(),
        "foreign platform must not be deployed"
    );
    assert!(!f.i.exists());
}

#[test]
fn actual_activation_proof_failure_after_launcher_is_compensated() {
    let mut f = Fixture::new();
    f.change_file("bin/actual.cjs", vec![b'x'; 16 * 1024 * 1024 + 1]);
    assert!(
        f.provision().is_err(),
        "existing activation proof has a 16MiB script limit"
    );
    assert!(!f.i.join("runtime").exists());
    assert!(!f.i.join(PENDING).exists());
    assert!(!f.d.exists());
    let failure: Value = serde_json::from_slice(&fs::read(f.i.join(FAILURE)).unwrap()).unwrap();
    assert_eq!(failure["compensated"], true);
}

#[test]
fn installed_unsafe_links_and_record_hardlinks_are_preserved_and_refused() {
    let f = Fixture::new();
    f.provision().unwrap();
    std::os::unix::fs::symlink("../../outside", f.i.join("runtime/unsafe-link")).unwrap();
    let before = tree_snapshot(&f.i);
    assert!(f.provision().is_err());
    assert_eq!(tree_snapshot(&f.i), before);
    fs::remove_file(f.i.join("runtime/unsafe-link")).unwrap();
    let path = RuntimeStore::new(&f.d).path().to_path_buf();
    fs::hard_link(&path, f.root.join("record-alias")).unwrap();
    let before = fs::read(&path).unwrap();
    assert!(f.provision().is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn safe_link_chains_and_in_runtime_directory_targets_remain_valid() {
    let mut f = Fixture::new();
    f.rows.push((
        json!({"path":"runtime/node/bin/npm-alias","kind":"symlink","target":"npm"}),
        vec![],
    ));
    f.rows.push((
        json!({"path":"runtime/node/library-alias","kind":"symlink","target":"lib"}),
        vec![],
    ));
    f.repack();
    f.verify().unwrap();
    f.provision().unwrap();
    assert!(matches!(
        f.provision().unwrap(),
        ProvisionResult::AlreadyInstalled { .. }
    ));
    assert_eq!(
        f.i.join("runtime/node/bin/npm-alias")
            .canonicalize()
            .unwrap(),
        f.i.join("runtime/node/lib/node_modules/npm/bin/npm-cli.js")
    );
}

#[cfg(target_os = "macos")]
#[test]
fn mac_container_rejected_before_archive_lock_stage_runtime_or_store_write() {
    let f = Fixture::new();
    let user = f.root.join("isolated-profile");
    let applications = user.join("Applications");
    let old_app = applications.join("OpenCodeX Desktop.app/Contents/MacOS/app");
    mkdir(old_app.parent().unwrap()).unwrap();
    fs::write(&old_app, b"old app stays").unwrap();
    let config = user.join("config-sentinel");
    fs::write(&config, b"old config stays").unwrap();
    let before = tree_snapshot(&user);
    let entries: BTreeSet<_> = fs::read_dir(&applications)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let (bytes, pin) = f.bytes();
    for i in [&applications, &user] {
        // Valid pinned bundle: fail only on the real installation boundary.
        let error =
            provision_inner(i, &f.d, &f.archive, &bytes, &pin, false, Some(&user)).unwrap_err();
        assert!(matches!(error, AppError::RuntimeManaged { ref code, .. }
            if code == "macos_install_container"));
        assert_eq!(tree_snapshot(&user), before);
        assert!(!f.d.exists());
    }
    assert_eq!(
        fs::read_dir(&applications)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<BTreeSet<_>>(),
        entries
    );
    assert!(!applications.join(".runtime-bundle.lock").exists());
    assert!(!applications.join(PENDING).exists());
    assert!(!applications.join(FAILURE).exists());
    // Public entry also refuses a real container BEFORE reading any archive.
    let error = provision(
        Path::new("/Applications"),
        &f.d,
        &f.root.join("missing-archive"),
        b"",
        "",
    )
    .unwrap_err();
    assert!(matches!(error, AppError::RuntimeManaged { ref code, .. }
        if code == "macos_install_container"));
    assert!(!f.d.exists());
}

#[cfg(target_os = "macos")]
#[test]
fn mac_whole_folder_under_personal_applications_installs_and_reopens_read_only() {
    let mut f = Fixture::new();
    let user = f.root.join("isolated-profile");
    f.i = user.join("Applications/OpenCodeX Desktop");
    let (bytes, pin) = f.bytes();
    let first = provision_inner(&f.i, &f.d, &f.archive, &bytes, &pin, false, Some(&user)).unwrap();
    assert!(matches!(first, ProvisionResult::Installed { .. }));
    let before = tree_snapshot(&f.i);
    let entries: BTreeSet<_> = fs::read_dir(&f.i)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    fs::set_permissions(&f.i, fs::Permissions::from_mode(0o555)).unwrap();
    let second = f.root.join("second-user/desktop");
    // Public production entry takes the read-only existing path, with no install-target
    // write probe and no need to reopen/create a lock in I.
    let result = provision(&f.i, &second, &f.root.join("missing-archive"), b"", "");
    fs::set_permissions(&f.i, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        result.unwrap(),
        ProvisionResult::AlreadyInstalled { .. }
    ));
    assert_eq!(tree_snapshot(&f.i), before);
    assert_eq!(
        fs::read_dir(&f.i)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<BTreeSet<_>>(),
        entries
    );
    assert_eq!(
        RuntimeStore::new(&second).load().source,
        RuntimeSourceKind::Managed
    );
}
