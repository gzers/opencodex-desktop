//! Debug-only ordinary existing-root qualification admission. It cannot open
//! public Start. Actual source/Owner/lease/custody/typed lifecycle factories
//! remain mandatory after this separate exact task/environment admission.
use super::{bounded::BoundedAccessV1, coordination, v2::routes::RoutePin};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        hash, platform, runtime_executable::SharedRuntimeExecutable,
        windows_official_runtime024 as native,
    },
    modules::{
        data_root::layout::Platform,
        process::ProcessCommand,
        runtime::{
            install,
            official_lifecycle024::ActionWindow,
            program_image,
            program_layout::RuntimeVersionPaths,
            program_selection::{self, CurrentSelection, ImageManifest},
        },
    },
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    os::windows::fs::OpenOptionsExt,
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};
pub(crate) const SCOPE: &str = "isolated-r6-existing-official-qualification";
pub(crate) const BASE: &str = r"C:\Users\marenben\ocx024-r6-existing-20261117-r1";
pub(crate) const PUBLIC: &str = r"C:\Users\Public\ocx-r6-official-existing-20261117-r1";
const BUNDLE_SHA: &str = "14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7";
const OFFICIAL_SHA: &str = "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c";
const CONFIG:&[u8]=b"{\"providers\":{},\"fastRows\":false,\"catalogAutoRefresh\":{\"enabled\":false},\"quotaResetNotify\":{\"enabled\":false},\"clientIntegrations\":{\"codex\":true,\"grok\":true,\"claude-desktop\":false}}\n";
fn invalid(reason: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason.to_string())
}
fn app(e: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_existing_qualification_refused".into(),
        detail: e.to_string(),
    }
}
fn write_new(path: &Path, value: &Value) -> io::Result<()> {
    let mut f = platform::create_new_private_file(path, 0o600)?;
    f.write_all(&serde_json::to_vec_pretty(value).map_err(invalid)?)?;
    f.sync_all()
}
fn digest_file(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut b = [0u8; 65536];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn snapshot(path: &Path, dir: bool, end: Instant) -> io::Result<Value> {
    let proof = BoundedAccessV1::capture(path, dir, end).map_err(invalid)?;
    proof.recheck(end).map_err(invalid)?;
    serde_json::to_value(proof.snapshot_data()?).map_err(invalid)
}
fn absent(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(_) => Err(invalid(
            "setup target already present; no adoption/overwrite/retry",
        )),
    }
}
fn bounded_json(path: &Path) -> io::Result<Value> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1024 * 1024 {
        return Err(invalid("setup JSON bound"));
    }
    serde_json::from_slice(&bytes).map_err(invalid)
}
fn baseline_shape(base: &Path, rows: &[Value]) -> io::Result<()> {
    let profile = Path::new(r"C:\Users\marenben");
    let expected = [
        (base.join("I"), true),
        (base.join("D"), true),
        (base.join("H"), true),
        (profile.join(".codex"), true),
        (profile.join(".grok"), true),
        (
            profile.join(".codex/.opencodex-native-main.claim.sqlite"),
            false,
        ),
        (
            profile.join(".codex/.opencodex-native-main.owner.sqlite"),
            false,
        ),
        (profile.join(".codex/config.toml"), false),
        (profile.join(".codex/auth.json"), false),
        (profile.join(".grok/config.json"), false),
    ];
    if rows.len() != expected.len() {
        return Err(invalid(
            "complete original synthetic baseline set missing/extra",
        ));
    }
    for (row, (path, directory)) in rows.iter().zip(expected) {
        if row["path"].as_str().map(Path::new) != Some(path.as_path())
            || row["directory"] != directory
        {
            return Err(invalid(
                "synthetic baseline locator order/path/kind differs",
            ));
        }
    }
    Ok(())
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn private_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if fs::symlink_metadata(parent).is_err_and(|e| e.kind() == io::ErrorKind::NotFound) {
            private_parent(parent)?;
        }
    }
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            native::create_private_directory(path)?.recheck()
        }
        Err(e) => Err(e),
        Ok(_) => native::PinnedDirectory::open(path)?.recheck(),
    }
}
fn archive_target(name: &str, version: &Path) -> io::Result<PathBuf> {
    if name.contains('\\')
        || name.contains(':')
        || name.len() > 32700
        || name
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
        || Path::new(name)
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err(invalid("noncanonical fixed bundle entry"));
    }
    let relative = if let Some(p) = name.strip_prefix("runtime/opencodex/") {
        PathBuf::from("opencodex").join(p)
    } else if let Some(p) = name.strip_prefix("runtime/node/") {
        PathBuf::from("dependencies/node").join(p)
    } else if name == "runtime/bun/bun.exe" {
        PathBuf::from("dependencies/bun/bun.exe")
    } else {
        return Err(invalid("unexpected pinned bundle member"));
    };
    Ok(version.join(relative))
}
fn copy_private(source: &Path, target: &Path) -> io::Result<()> {
    absent(target)?;
    private_parent(target.parent().ok_or_else(|| invalid("parent absent"))?)?;
    let mut out = platform::create_new_private_file(target, 0o600)?;
    let mut input = File::open(source)?;
    io::copy(&mut input, &mut out)?;
    out.sync_all()
}
fn install_fixed(base: &Path, ledger: &Path) -> io::Result<Value> {
    let bundle = Path::new(PUBLIC).join("input/runtime-bundle-0.2.24-win32-x64.tar.gz");
    let mut pinned_bundle = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(0x200000)
        .open(&bundle)?;
    let mut bundle_hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = pinned_bundle.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bundle_hasher.update(&buffer[..n]);
    }
    if format!("{:x}", bundle_hasher.finalize()) != BUNDLE_SHA {
        return Err(invalid("pinned original bundle digest mismatch"));
    }
    pinned_bundle.seek(SeekFrom::Start(0))?;
    let paths = RuntimeVersionPaths::new(
        &base.join("I/runtime/versions"),
        "2.99.0",
        Platform::Windows,
    )
    .map_err(invalid)?;
    absent(&base.join("I/runtime"))?;
    private_parent(&paths.version_root)?;
    write_new(
        &ledger.join("image-create-intent.json"),
        &json!({"bundleSha256":BUNDLE_SHA,"versionRoot":paths.version_root,"officialArchiveSha256":OFFICIAL_SHA,"setupOnly":true}),
    )?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(pinned_bundle));
    let mut seen = BTreeSet::new();
    let mut bytes = 0u64;
    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            return Err(invalid(
                "fixed bundle must contain regular files only; links/directories/unknown reject",
            ));
        }
        if !entry.header().entry_type().is_file() {
            return Err(invalid("fixed bundle nonregular entry"));
        }
        let source = entry
            .path()?
            .to_str()
            .ok_or_else(|| invalid("bundle path not UTF8"))?
            .to_string();
        let target = archive_target(&source, &paths.version_root)?;
        let len = entry.size();
        bytes = bytes
            .checked_add(len)
            .ok_or_else(|| invalid("bundle size overflow"))?;
        if !seen.insert(source)
            || seen.len() > 100000
            || len > 512 * 1024 * 1024
            || bytes > 2 * 1024 * 1024 * 1024
        {
            return Err(invalid("original complete image bounds"));
        }
        absent(&target)?;
        private_parent(target.parent().ok_or_else(|| invalid("bundle parent"))?)?;
        let mut out = platform::create_new_private_file(&target, 0o600)?;
        if io::copy(&mut entry, &mut out)? != len {
            return Err(invalid("truncated original bundle"));
        }
        out.sync_all()?;
    }
    // Same layout as shared version publisher: npm is an independent exact
    // private copy of the pinned Node bundled npm, not npm install or download.
    let npm = paths
        .node
        .parent()
        .ok_or_else(|| invalid("node parent"))?
        .join("node_modules/npm");
    let mut stack = vec![(npm.clone(), paths.version_root.join("dependencies/npm"))];
    while let Some((src, dst)) = stack.pop() {
        private_parent(&dst)?;
        for e in fs::read_dir(src)? {
            let e = e?;
            let m = e.file_type()?;
            if m.is_symlink() {
                return Err(invalid("npm source link"));
            }
            let dst = dst.join(e.file_name());
            if m.is_dir() {
                stack.push((e.path(), dst));
            } else if m.is_file() {
                copy_private(&e.path(), &dst)?;
            } else {
                return Err(invalid("npm unknown object"));
            }
        }
    }
    let package = paths.package_root.join(install::PACKAGE_SUBPATH);
    let package_value = bounded_json(&package.join("package.json"))?;
    let bin = package_value["bin"]["ocx"]
        .as_str()
        .ok_or_else(|| invalid("fixed package ocx bin absent"))?
        .trim_start_matches("./");
    if package_value["name"] != "@bitkyc08/opencodex" || package_value["version"] != "2.99.0" {
        return Err(invalid("fixed official package identity"));
    }
    let image = ImageManifest {
        schema_version: 1,
        package: "@bitkyc08/opencodex".into(),
        version: "2.99.0".into(),
        platform: "win32-x64".into(),
        official_archive_sha256: OFFICIAL_SHA.into(),
        tree_sha256: program_image::content_digest(&paths.version_root).map_err(invalid)?,
        bin: bin.into(),
        package_sha256: digest_file(&package.join("package.json"))?,
        bin_sha256: digest_file(&package.join(bin))?,
        node_sha256: digest_file(&paths.node)?,
        npm_sha256: digest_file(&paths.npm)?,
        bun_sha256: digest_file(&paths.bun)?,
        bun_launcher_sha256: digest_file(&paths.package_root.join("node_modules/bun/bin/bun.exe"))?,
    };
    let raw = serde_json::to_vec_pretty(&image).map_err(invalid)?;
    {
        let mut f = platform::create_new_private_file(
            &paths.version_root.join(program_selection::IMAGE_MANIFEST),
            0o600,
        )?;
        f.write_all(&raw)?;
        f.sync_all()?;
    }
    let current = CurrentSelection {
        schema_version: 1,
        version: "2.99.0".into(),
        platform: "win32-x64".into(),
        image_sha256: hash::sha256_hex(&raw),
    };
    write_new(
        &base.join("I/runtime/current.json"),
        &serde_json::to_value(&current).map_err(invalid)?,
    )?;
    let stable = base.join("I/runtime/bin/ocx-runtime.cmd");
    private_parent(stable.parent().unwrap())?;
    {
        let mut f = platform::create_new_private_file(&stable, 0o600)?;
        f.write_all(install::entry_script(Some(&paths.node), &package.join(bin)).as_bytes())?;
        f.sync_all()?;
    }
    let selected =
        program_selection::read_complete(&base.join("I"), Platform::Windows, "win32-x64")
            .map_err(invalid)?
            .ok_or_else(|| invalid("exact fixed current absent"))?;
    program_image::verify(&selected).map_err(invalid)?;
    let report = json!({"bundleSha256":BUNDLE_SHA,"officialArchiveSha256":OFFICIAL_SHA,"members":seen.len(),"unpackedBytes":bytes,"current":current,"image":image,"sharedFullImageReadback":true,"setupOnly":true});
    write_new(&ledger.join("image-created-verified.json"), &report)?;
    Ok(report)
}
fn prepare_coordination(
    base: &Path,
    ledger: &Path,
    token: &native::TokenIdentity,
) -> io::Result<Value> {
    let root = coordination::official_runtime_root(token)?;
    let client = Path::new(r"C:\Users\marenben\.codex");
    let canonical = fs::canonicalize(client)?;
    let canonical = dunce::simplified(&canonical);
    if !canonical
        .to_string_lossy()
        .eq_ignore_ascii_case(&client.to_string_lossy())
    {
        return Err(invalid("actual client canonical path differs"));
    }
    let digest = hash::sha256_hex(
        canonical
            .to_str()
            .ok_or_else(|| invalid("canonical path utf8"))?
            .as_bytes(),
    );
    let paths = [
        root.join("native-write-locks")
            .join(format!("{digest}.sqlite")),
        root.join("client-desktop-lifecycle.sqlite"),
        root.join("catalog-write-locks")
            .join(format!("{digest}.sqlite")),
    ];
    // Collect all independent paths first, then mutate only this fresh synthetic
    // account baseline. Existing/unknown files are never overwritten/adopted.
    let state=paths.iter().map(|p|json!({"path":p,"absent":matches!(fs::symlink_metadata(p),Err(e) if e.kind()==io::ErrorKind::NotFound)})).collect::<Vec<_>>();
    write_new(
        &ledger.join("coordination-before.json"),
        &json!({"root":root,"states":state,"priorAbsencePreserved":true,"officialInitializationProven":false}),
    )?;
    if state.iter().any(|s| s["absent"] != true) {
        return Err(invalid(
            "test coordination target already present or unknown",
        ));
    }
    let mut created = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        write_new(
            &ledger.join(format!("coord-{i}-intent.json")),
            &json!({"path":p,"operation":"CreateSyntheticExistingCoordinationFile","officialInitializationProven":false}),
        )?;
        private_parent(p.parent().unwrap())?;
        let mut f = native::PrivateRecord::create(p, b"", 4096)?;
        if !f.read()?.is_empty() {
            return Err(invalid("new lock bytes changed"));
        }
        let row = json!({"path":p,"volume":f.identity().volume,"fileId":f.identity().index.to_string(),"security":snapshot(p,false,Instant::now()+Duration::from_secs(5))?,"bytes":0});
        write_new(&ledger.join(format!("coord-{i}-verified.json")), &row)?;
        created.push(row);
    }
    let result = json!({"statesBefore":state,"objects":created,"base":base,"nativeLeaseGranted":false,"officialInitializationProven":false});
    write_new(&ledger.join("coordination-created.json"), &result)?;
    Ok(result)
}
/// Private native admission. This is NOT a lease, OriginalSession or Closed.
/// No serde/Clone or constructor accepting external receipt/permit booleans.
pub(crate) struct ExistingQualification {
    peer: native::RetainedPeer,
    pins: Vec<native::PinnedDirectory>,
    base: PathBuf,
}
impl ExistingQualification {
    pub(crate) fn prepare() -> io::Result<Self> {
        let base = Path::new(BASE);
        let peer = native::RetainedPeer::current()?;
        let t = &peer.identity().token;
        if t.sid != "S-1-5-21-587408064-856152790-2583328096-1003"
            || t.session_id != 7
            || t.elevated
            || t.elevation_type != 1
            || platform::home_dir().as_deref() != Some(Path::new(r"C:\Users\marenben"))
        {
            return Err(invalid("exact ordinary desktop/real Profile required"));
        }
        peer.no_outer_job()?;
        peer.verify_image(&base.join("I/ocxd.exe"))?;
        let pins = [
            base.to_path_buf(),
            base.join("I"),
            base.join("D"),
            base.join("H"),
        ]
        .iter()
        .map(|p| native::PinnedDirectory::open_private(p))
        .collect::<io::Result<Vec<_>>>()?;
        let baseline = bounded_json(&base.join("preparation/result.json"))?;
        if baseline["preparedExistingBaseline"] != true
            || baseline["actualProductionExecuted"] != false
            || baseline["originalClientRootsBeforePreparation"] != "Absent"
        {
            return Err(invalid("original setup baseline unknown"));
        }
        let rows = baseline["rows"]
            .as_array()
            .ok_or_else(|| invalid("baseline object set"))?;
        baseline_shape(base, rows)?;
        for row in rows {
            let path = Path::new(
                row["path"]
                    .as_str()
                    .ok_or_else(|| invalid("baseline path"))?,
            );
            if snapshot(
                path,
                row["directory"]
                    .as_bool()
                    .ok_or_else(|| invalid("baseline kind"))?,
                Instant::now() + Duration::from_secs(5),
            )? != row["security"]
            {
                return Err(invalid(
                    "original synthetic baseline identity/security drift",
                ));
            }
            if row["directory"] == false {
                let mut raw = Vec::new();
                File::open(path)?.take(4097).read_to_end(&mut raw)?;
                if raw.len() > 4096
                    || row["bytes"].as_u64() != Some(raw.len() as u64)
                    || row["sha256"] != hash::sha256_hex(&raw)
                {
                    return Err(invalid("original credential-free baseline byte drift"));
                }
            }
        }
        let ledger = native::create_private_directory(&base.join("qualification-preparation"))?;
        write_new(
            &ledger.path().join("admission.json"),
            &json!({"sid":t.sid,"session":t.session_id,"elevated":t.elevated,"noOuterJob":true,"base":base,"publicStartOpen":false,"officialInitializationProven":false,"baseline":baseline}),
        )?;
        let image = install_fixed(base, ledger.path())?;
        let coordination = prepare_coordination(base, ledger.path(), t)?;
        let config = base.join("H/config.json");
        absent(&config)?;
        let mut f = platform::create_new_private_file(&config, 0o600)?;
        f.write_all(CONFIG)?;
        f.sync_all()?;
        drop(f);
        let end = Instant::now() + Duration::from_secs(5);
        let mut routes = RoutePin::capture(&base.join("H"), end)?;
        if !routes.declared(&base.join("H"), end)?.is_empty() {
            return Err(invalid("qualification routes must be declared empty"));
        }
        drop(routes);
        write_new(
            &ledger.path().join("prepared.json"),
            &json!({"image":image,"coordination":coordination,"configSha256":hash::sha256_hex(CONFIG),"credentialFreeProviders":true,"autoRefreshDisabled":true,"prepared":true,"actualProductionExecuted":false,"nativeLeaseGranted":false,"officialInitializationProven":false,"startOpen":false}),
        )?;
        Ok(Self {
            peer,
            pins,
            base: base.into(),
        })
    }
    pub(crate) fn admit(
        &self,
        runtime: &SharedRuntimeExecutable,
        command: &ProcessCommand,
        data: &Path,
        window: &ActionWindow,
    ) -> AppResult<()> {
        window.remaining()?;
        before(window.deadline()).map_err(app)?;
        self.peer.no_outer_job().map_err(app)?;
        self.peer
            .verify_image(&self.base.join("I/ocxd.exe"))
            .map_err(app)?;
        for pin in &self.pins {
            pin.recheck().map_err(app)?;
        }
        if runtime.managed_installation_root() != Some(self.base.join("I"))
            || command.working_directory != Path::new(r"C:\Users\marenben")
            || command.environment.opencodex_home != self.base.join("H")
            || data != self.base.join("D")
        {
            return Err(app(
                "qualification current source/real Profile/I-D-H differs",
            ));
        }
        let mut config = Vec::new();
        File::open(self.base.join("H/config.json"))
            .map_err(app)?
            .take(CONFIG.len() as u64 + 1)
            .read_to_end(&mut config)
            .map_err(app)?;
        if config != CONFIG {
            return Err(app("credential-free config/declared empty routes changed"));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_r6_existing_qualification_baseline_shape_rejects_omitted_extra_and_foreign_data() {
        let base = Path::new(BASE);
        assert!(baseline_shape(base, &[]).is_err());
        let forged = vec![json!({"path":r"C:\Users\marenben\daily-secret","directory":false}); 10];
        assert!(baseline_shape(base, &forged).is_err());
        println!("r6 original credential-free synthetic baseline requires exact complete10 ordered locators and kinds before any native source read/setup; omitted/extra/foreign DATA never grants admission");
    }
    #[test]
    fn wl2_r6_existing_qualification_archive_mapping_rejects_escape_and_unknown_source_without_authority(
    ) {
        let v = Path::new("C:/synthetic-version");
        assert_eq!(
            archive_target("runtime/bun/bun.exe", v).unwrap(),
            v.join("dependencies/bun/bun.exe")
        );
        for p in [
            "../escape",
            "runtime/node/../outside",
            "runtime/opencodex/C:escape",
            "runtime/extra/x",
            "runtime/node\\x",
            "/runtime/node/x",
        ] {
            assert!(archive_target(p, v).is_err(), "{p}");
        }
        println!("r6 qualification fixed source map refuses traversal/unknown prefixes/ADS/alternate separator; DATA path mapping creates no qualification, native Owner/lease/Closed or public Start permit");
    }
}
