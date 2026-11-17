//! Actual ordinary desktop baseline preparation, not an official initialization,
//! lease, mutation/restore permit, native Owner or Closed production proof.
use super::bounded::BoundedAccessV1;
use crate::infrastructure::{hash, platform, windows_official_runtime024 as native};
use crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_CODEX_STABLE_FILES;
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const SID: &str = "S-1-5-21-587408064-856152790-2583328096-1003";
const PROFILE: &str = r"C:\Users\marenben";
const PUBLIC: &str = r"C:\Users\Public\ocx-r6-profile-preflight-20261117-r1";
const BASE: &str = "ocx024-r6-existing-20261117-r1";
fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}
fn locator_matches(sid: &str, session: u32, elevated: bool, profile: &Path, public: &Path) -> bool {
    sid == SID
        && session == 7
        && !elevated
        && profile == Path::new(PROFILE)
        && public == Path::new(PUBLIC)
}
fn write_new(path: &Path, value: &Value) -> io::Result<()> {
    let raw = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    let mut f = platform::create_new_private_file(path, 0o600)?;
    f.write_all(&raw)?;
    f.sync_all()
}
fn stamp(path: &Path, directory: bool, end: Instant) -> io::Result<Value> {
    let proof = BoundedAccessV1::capture(path, directory, end).map_err(io::Error::other)?;
    let data = proof.snapshot_data()?;
    proof.recheck(end).map_err(io::Error::other)?;
    serde_json::to_value(data).map_err(io::Error::other)
}
fn observation(path: &Path, end: Instant) -> Value {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            json!({"path":path,"state":"Absent","nativeAuthority":false})
        }
        Err(e) => {
            json!({"path":path,"state":"Unknown","raw":e.raw_os_error(),"error":e.to_string()})
        }
        Ok(m) => match stamp(path, m.is_dir(), end) {
            Ok(s) => json!({"path":path,"state":"Present","directory":m.is_dir(),"security":s}),
            Err(e) => {
                json!({"path":path,"state":"Unknown","raw":e.raw_os_error(),"error":e.to_string()})
            }
        },
    }
}
fn create_dir(
    path: &Path,
    ledger: &Path,
    index: &mut usize,
    rows: &mut Vec<Value>,
) -> io::Result<()> {
    write_new(
        &ledger.join(format!("prepare-{index:03}-intent.json")),
        &json!({"operation":"CreateSyntheticDirectory","path":path,"notOfficialInitialization":true}),
    )?;
    let pin = native::create_private_directory(path)?;
    let end = Instant::now() + Duration::from_secs(5);
    let row = json!({"path":path,"directory":true,"volume":pin.identity().volume,"fileId":pin.identity().index.to_string(),"security":stamp(path,true,end)?});
    pin.recheck()?;
    write_new(
        &ledger.join(format!("prepare-{index:03}-result.json")),
        &row,
    )?;
    rows.push(row);
    *index += 1;
    Ok(())
}
fn create_file(
    path: &Path,
    bytes: &[u8],
    ledger: &Path,
    index: &mut usize,
    rows: &mut Vec<Value>,
) -> io::Result<()> {
    write_new(
        &ledger.join(format!("prepare-{index:03}-intent.json")),
        &json!({"operation":"CreateSyntheticFile","path":path,"bytes":bytes.len(),"sha256":hash::sha256_hex(bytes),"notOfficialInitialization":true}),
    )?;
    let mut file = native::PrivateRecord::create(path, bytes, 4096)?;
    let end = Instant::now() + Duration::from_secs(5);
    let row = json!({"path":path,"directory":false,"volume":file.identity().volume,"fileId":file.identity().index.to_string(),"bytes":bytes.len(),"sha256":hash::sha256_hex(bytes),"security":stamp(path,false,end)?});
    if file.read()? != bytes {
        return Err(invalid("fresh synthetic bytes changed"));
    }
    write_new(
        &ledger.join(format!("prepare-{index:03}-result.json")),
        &row,
    )?;
    rows.push(row);
    *index += 1;
    Ok(())
}
fn prepare(public: &Path, profile: &Path) -> io::Result<Value> {
    let current = native::RetainedPeer::current()?;
    let token = &current.identity().token;
    if !locator_matches(
        &token.sid,
        token.session_id,
        token.elevated || token.elevation_type == 2,
        profile,
        public,
    ) {
        return Err(invalid(
            "exact ordinary desktop SID/session/KnownFolder required",
        ));
    }
    // Independent facts collected before any task/client preparation. The
    // original r4 eight projections remain required, audit SACL unknown.
    let nojob = current.no_outer_job();
    let profile_pin = native::PinnedDirectory::open(profile);
    let end = Instant::now() + Duration::from_secs(5);
    let profile_security = stamp(profile, true, end);
    let base = profile.join(BASE);
    let paths = [profile.join(".codex"), profile.join(".grok"), base.clone()];
    let roots = paths
        .iter()
        .map(|p| observation(p, end))
        .collect::<Vec<_>>();
    let facts = json!({"sid":token.sid,"session":token.session_id,"elevated":token.elevated,"elevationType":token.elevation_type,"actualKnownFolderProfile":profile,"outerJobAbsent":nojob.is_ok(),"outerJobError":nojob.as_ref().err().map(|e|e.to_string()),"profilePin":profile_pin.is_ok(),"profileSecurity":profile_security.as_ref().ok(),"profileSecurityError":profile_security.as_ref().err().map(|e|e.to_string()),"roots":roots,"actualProductionExecuted":false,"startOpen":false});
    write_new(&public.join("out01/prerequisites.json"), &facts)?;
    nojob?;
    let profile_pin = profile_pin?;
    profile_security?;
    if roots.iter().any(|r| r["state"] != "Absent") {
        return Err(invalid(
            "preexisting or unknown client/task root refused; no adoption/overwrite",
        ));
    }
    current.verify()?;
    profile_pin.recheck()?;
    let base_pin = native::create_private_directory(&base)?;
    let ledger = native::create_private_directory(&base.join("preparation"))?;
    write_new(&ledger.path().join("initial-absence.json"), &facts)?;
    let mut rows = Vec::new();
    let mut index = 0;
    let result = (|| {
        for name in ["I", "D", "H"] {
            create_dir(&base.join(name), ledger.path(), &mut index, &mut rows)?;
        }
        for root in [&paths[0], &paths[1]] {
            create_dir(root, ledger.path(), &mut index, &mut rows)?;
        }
        for name in ANCHORED_CODEX_STABLE_FILES {
            create_file(
                &paths[0].join(name),
                b"",
                ledger.path(),
                &mut index,
                &mut rows,
            )?;
        }
        create_file(
            &paths[0].join("config.toml"),
            b"# credential-free synthetic existing-root baseline\n",
            ledger.path(),
            &mut index,
            &mut rows,
        )?;
        create_file(
            &paths[0].join("auth.json"),
            b"{}\n",
            ledger.path(),
            &mut index,
            &mut rows,
        )?;
        create_file(
            &paths[1].join("config.json"),
            b"{}\n",
            ledger.path(),
            &mut index,
            &mut rows,
        )?;
        profile_pin.recheck()?;
        base_pin.recheck()?;
        ledger.recheck()?;
        current.no_outer_job()?;
        current.verify()?;
        Ok(())
    })();
    // Preserve all created prefixes on any failure. No implicit cleanup or
    // retry; setup records are DATA and never native lifecycle permissions.
    let final_value = json!({"base":base,"baseline":"synthetic-credential-free-existing-roots","originalClientRootsBeforePreparation":"Absent","preparedExistingBaseline":result.is_ok(),"rows":rows,"firstError":result.as_ref().err().map(|e:&io::Error|json!({"raw":e.raw_os_error(),"error":e.to_string()})),"actualProductionExecuted":false,"officialInitializationProven":false,"originalAbsentRetirementProven":false,"nativeAuthority":false,"typedClosed":false,"startOpen":false,"cleanup":"all preparation objects retained for upcoming exact original-owner test"});
    write_new(&ledger.path().join("result.json"), &final_value)?;
    write_new(&public.join("out01/preparation-result.json"), &final_value)?;
    result?;
    Ok(final_value)
}
#[test]
fn wl2_r6_desktop_preflight_locator_data_refuses_other_account_session_profile_or_scope() {
    let p = Path::new(PROFILE);
    let public = Path::new(PUBLIC);
    assert!(locator_matches(SID, 7, false, p, public));
    for (sid, session, elevated, profile, path) in [
        ("S-1-5-18", 7, false, p, public),
        (SID, 2, false, p, public),
        (SID, 7, true, p, public),
        (SID, 7, false, Path::new("C:/synthetic-profile"), public),
        (SID, 7, false, p, Path::new("C:/Users/Public/foreign")),
    ] {
        assert!(!locator_matches(sid, session, elevated, profile, path));
    }
    println!("r6 exact desktop preflight locator DATA rejects foreign/elevated/profile/session/path; no token, Profile, native capability or client setup produced by locator comparisons");
}
#[test]
#[ignore = "only user-started marenben ordinary interactive desktop, exact prepared fresh input/output"]
fn isolated_r6_existing_root_preflight() {
    let public = PathBuf::from(
        std::env::var_os("OCXD024_R6_PREFLIGHT_PUBLIC").expect("exact desktop input required"),
    );
    assert_eq!(public, Path::new(PUBLIC));
    let profile = platform::home_dir().expect("actual KnownFolder Profile");
    let result = prepare(&public, &profile);
    if let Err(e) = &result {
        let _ = write_new(
            &public.join("out01/native-failure.json"),
            &json!({"raw":e.raw_os_error(),"error":e.to_string(),"actualProductionExecuted":false,"startOpen":false,"createdObjectsRetained":true}),
        );
    }
    println!(
        "r6 ordinary desktop preparation result={}",
        result
            .as_ref()
            .map(|_| "prepared synthetic existing baseline only")
            .unwrap_or("refused; exact prefixes retained")
    );
    result.unwrap();
}
