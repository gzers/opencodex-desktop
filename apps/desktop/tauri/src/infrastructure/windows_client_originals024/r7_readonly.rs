//! Task-specific DATA observations only: no Owner, lease, Job or repair grant.
use super::{bounded::native_stamp, tree};
use crate::{
    infrastructure::{hash, platform, windows_official_runtime024 as native},
    modules::{data_root::layout::Platform, runtime::program_selection},
};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
    time::{Duration, Instant},
};
const LOCATORS: &[u8] =
    include_bytes!("../../../../../../test/windows024-native/r7-scene-locators.json");
const PUBLIC: &str = r"C:\Users\Public\ocx-r7-r2-readonly-20261117-r1";
const BASE: &str = r"C:\Users\marenben\ocx024-r6-existing-20261117-r2";
fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}
fn observe(path: &Path, directory: bool, end: Instant) -> io::Result<Value> {
    native::validate_local_path(path)?;
    let mut file = OpenOptions::new()
        .access_mode(if directory { 0x20080 } else { 0x80020000 })
        .share_mode(7)
        .custom_flags(0x200000 | if directory { 0x2000000 } else { 0 })
        .open(path)?;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, GetFinalPathNameByHandleW,
        BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || info.dwFileAttributes & 0x400 != 0
        || (info.dwFileAttributes & 0x10 != 0) != directory
        || !directory && info.nNumberOfLinks != 1
    {
        return Err(invalid("reparse/multiple links/wrong kind refused"));
    }
    let mut final_path = vec![0u16; 32768];
    let n = unsafe {
        GetFinalPathNameByHandleW(file.as_raw_handle(), final_path.as_mut_ptr(), 32768, 0)
    };
    if n == 0 || n >= 32768 {
        return Err(invalid("native final path unavailable"));
    }
    let actual =
        String::from_utf16(&final_path[..n as usize]).map_err(|_| invalid("native path UTF16"))?;
    let actual = actual
        .strip_prefix(r"\\?\")
        .ok_or_else(|| invalid("local native path required"))?;
    if !actual.eq_ignore_ascii_case(&path.to_string_lossy().replace('/', r"\")) {
        return Err(invalid("native final path differs"));
    }
    tree::streams(path, directory)?;
    let security = native_stamp(&file, end).map_err(io::Error::other)?;
    let mut children = Vec::new();
    let mut bytes = Vec::new();
    if directory {
        for entry in fs::read_dir(path)? {
            if children.len() >= 20000 {
                return Err(invalid("children bound"));
            }
            children.push(
                entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("child UTF16"))?,
            );
        }
        children.sort();
    } else {
        file.by_ref().take(4194305).read_to_end(&mut bytes)?;
        if bytes.len() > 4194304 {
            return Err(invalid("critical record bound"));
        }
    }
    if native_stamp(&file, end).map_err(io::Error::other)? != security {
        return Err(invalid("native object/security drift"));
    }
    tree::streams(path, directory)?;
    Ok(
        json!({"path":path,"directory":directory,"security":security,"children":children,"bytes":bytes.len(),"sha256":hash::sha256_hex(&bytes)}),
    )
}
fn compare(expected: &Value, actual: &Value) -> io::Result<()> {
    if expected
        .get("security")
        .is_some_and(|s| s != &actual["security"])
        || expected
            .get("fileId")
            .is_some_and(|v| v != &actual["security"]["fileId"])
        || expected
            .get("volume")
            .is_some_and(|v| v != &actual["security"]["volume"])
        || expected
            .get("children")
            .is_some_and(|v| v != &actual["children"])
        || expected.get("bytes").is_some_and(|v| v != &actual["bytes"])
        || expected
            .get("sha256")
            .is_some_and(|v| v != &actual["sha256"])
    {
        return Err(invalid(
            "exact original expected object/bytes/8/children differ",
        ));
    }
    Ok(())
}
#[cfg(test)]
#[test]
fn wl2_r7_readonly_comparison_refuses_identity_content_security_and_membership_drift() {
    let original = json!({"security":{"fileId":"1","volume":1,"projections":[1,2,4,16,32,64,128,256]},"children":["a"],"bytes":3,"sha256":"exact"});
    compare(&original, &original).unwrap();
    for key in ["security", "children", "bytes", "sha256"] {
        let mut changed = original.clone();
        changed[key] = Value::Null;
        assert!(compare(&original, &changed).is_err(), "{key}");
    }
}
pub(crate) fn observe_current_r2(phase: &str) -> io::Result<()> {
    if !matches!(phase, "before" | "after") {
        return Err(invalid("exact version1 readonly phase required"));
    }
    let peer = native::RetainedPeer::current()?;
    let token = &peer.identity().token;
    if token.sid != "S-1-5-21-587408064-856152790-2583328096-1003"
        || token.session_id != 7
        || token.elevated
        || token.elevation_type != 1
        || platform::home_dir().as_deref() != Some(Path::new(r"C:\Users\marenben"))
    {
        return Err(invalid("exact ordinary SID1003/session7/Profile required"));
    }
    peer.no_outer_job()?;
    let expected: Value = serde_json::from_slice(LOCATORS).unwrap();
    let end = Instant::now() + Duration::from_secs(120);
    let started = Instant::now();
    let mut rows = Vec::new();
    // Independent rows are all attempted. Errors are not success or authority.
    for entry in expected["objects"].as_array().unwrap() {
        let path = Path::new(entry["path"].as_str().unwrap());
        let result = observe(path, entry["directory"].as_bool().unwrap(), end);
        let validation = result
            .as_ref()
            .map_err(|e| e.to_string())
            .and_then(|actual| {
                compare(entry, actual).map_err(|e| e.to_string())?;
                if let Some(parent) = entry.get("parentId") {
                    let actual_parent =
                        observe(path.parent().unwrap(), true, end).map_err(|e| e.to_string())?;
                    if parent != &actual_parent["security"]["fileId"] {
                        return Err("parent native ID differs".into());
                    }
                }
                Ok(())
            });
        rows.push(json!({"expected":entry,"actual":result.as_ref().ok(),"passed":validation.is_ok(),"error":validation.err(),"raw":result.as_ref().err().and_then(io::Error::raw_os_error)}));
    }
    for record in expected["records"].as_array().unwrap() {
        let path = Path::new(record["path"].as_str().unwrap());
        let result = observe(path, false, end);
        let validation = result
            .as_ref()
            .map_err(|e| e.to_string())
            .and_then(|actual| compare(record, actual).map_err(|e| e.to_string()));
        rows.push(json!({"expected":record,"actual":result.as_ref().ok(),"passed":validation.is_ok(),"error":validation.err(),"raw":result.as_ref().err().and_then(io::Error::raw_os_error)}));
    }
    let installation = Path::new(BASE).join("I");
    let selection = program_selection::read_complete(&installation, Platform::Windows, "win32-x64");
    let selection_ok = selection.as_ref().is_ok_and(|s| {
        s.as_ref().is_some_and(|s| {
            s.selection.image_sha256 == expected["sourceDigest"] && s.selection.version == "2.99.0"
        })
    });
    let current = observe(&installation.join("runtime/current.json"), false, end);
    let current_ok = current
        .as_ref()
        .is_ok_and(|c| c["sha256"] == expected["currentSelectionSha256"]);
    let passed = rows.iter().all(|r| r["passed"] == true) && selection_ok && current_ok;
    let report = json!({"schema":"windows-r7-readonly-observation-v1","phase":phase,"sid":token.sid,"session":token.session_id,"ordinaryNoJob":true,"rows":rows,"fullSourcePassed":selection_ok,"sourceError":selection.as_ref().err().map(|e|e.to_string()),"current":current.as_ref().ok(),"currentError":current.as_ref().err().map(|e|e.to_string()),"currentPassed":current_ok,"passed":passed,"elapsedMs":started.elapsed().as_millis(),"observationLimitSeconds":120,"isActionWindow":false,"executionReady":false,"restoreExecuted":false,"forced":false,"normalStop":false,"closed":false,"publicStartOpen":false});
    let output = Path::new(PUBLIC).join(format!("out01/native-{phase}.json"));
    use std::io::Write;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    f.write_all(&serde_json::to_vec_pretty(&report).map_err(io::Error::other)?)?;
    f.sync_all()?;
    peer.verify()?;
    if !passed {
        return Err(invalid(
            "readonly scene has conflicts; consult complete per-item report",
        ));
    }
    Ok(())
}
