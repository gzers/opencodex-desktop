#[path = "../../../../../../apps/desktop/tauri/src/infrastructure/windows_client_originals024/r7_readonly/objects.rs"]
mod objects;
const LOCATORS: &[u8] = include_bytes!("../../../../r7-scene-locators.json");
use crate::{backend as native, infrastructure::hash, modules::runtime::program_image};
use serde_json::{json, Value};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const PUBLIC: &str = r"C:\Users\Public\ocx-r7-r2-readonly-20261117-r1";
const BASE: &str = r"C:\Users\marenben\ocx024-r6-existing-20261117-r2";
fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}
fn known_profile() -> io::Result<PathBuf> {
    #[repr(C)]
    struct Guid {
        a: u32,
        b: u16,
        c: u16,
        d: [u8; 8],
    }
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(
            id: *const Guid,
            flags: u32,
            token: *mut std::ffi::c_void,
            path: *mut *mut u16,
        ) -> i32;
    }
    let id = Guid {
        a: 0x5e6c858f,
        b: 0x0e22,
        c: 0x4760,
        d: [0x9a, 0xfe, 0xea, 0x33, 0x17, 0xb6, 0x71, 0x73],
    };
    let mut raw = std::ptr::null_mut();
    let code = unsafe { SHGetKnownFolderPath(&id, 0, std::ptr::null_mut(), &mut raw) };
    if code != 0 || raw.is_null() {
        return Err(invalid(&format!("OS Profile query failed HRESULT={code}")));
    }
    struct Alloc(*mut u16);
    impl Drop for Alloc {
        fn drop(&mut self) {
            unsafe {
                CoTaskMemFree(self.0.cast());
            }
        }
    }
    let _allocated = Alloc(raw);
    // CoTaskMemFree is an ole32 API, declared locally to avoid feature changes.
    let mut len = 0;
    while len < 32768 && unsafe { *raw.add(len) } != 0 {
        len += 1;
    }
    if len == 32768 {
        return Err(invalid("OS Profile string exceeds bound"));
    }
    let value = String::from_utf16(unsafe { std::slice::from_raw_parts(raw, len) })
        .map_err(|_| invalid("OS Profile UTF16"));
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(value: *mut std::ffi::c_void);
    }
    value.map(PathBuf::from)
}
fn context() -> io::Result<native::RetainedPeer> {
    let peer = native::RetainedPeer::current()?;
    let token = &peer.identity().token;
    if token.sid != "S-1-5-21-587408064-856152790-2583328096-1003"
        || token.session_id != 7
        || token.elevated
        || token.elevation_type != 1
        || known_profile()? != Path::new(r"C:\Users\marenben")
    {
        return Err(invalid(
            "exact ordinary SID1003/session7/OS Profile required",
        ));
    }
    peer.no_outer_job()?;
    peer.verify()?;
    Ok(peer)
}
fn source_at(root: &Path, current: &Path, expected: &Value, end: Instant) -> io::Result<Value> {
    let before = objects::observe(&root.join(".runtime-image.json"), false, end)?;
    if before["sha256"] != expected["sourceDigest"] {
        return Err(invalid("exact original image manifest digest differs"));
    }
    let started = Instant::now();
    let rows = program_image::inventory(&root).map_err(io::Error::other)?;
    let digest = program_image::inventory_digest(&rows).map_err(io::Error::other)?;
    if digest != "306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55" {
        return Err(invalid(
            "complete original source bytes/membership digest differs",
        ));
    }
    let after = objects::observe(&root.join(".runtime-image.json"), false, end)?;
    if before != after {
        return Err(invalid(
            "native image manifest changed during complete scan",
        ));
    }
    let current = objects::observe(current, false, end)?;
    if current["sha256"] != expected["currentSelectionSha256"] {
        return Err(invalid("exact original current digest differs"));
    }
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    let bytes = rows
        .values()
        .map(|r| match r {
            program_image::Entry::File { bytes, .. } => *bytes,
            _ => 0,
        })
        .sum::<u64>();
    Ok(
        json!({"passed":true,"treeSha256":digest,"entries":rows.len(),"fileBytes":bytes,"elapsedMs":started.elapsed().as_millis(),"manifestBefore":before,"manifestAfter":after,"current":current,"scope":"actual unchanged shared inventory algorithm section; fixed original manifest/current expected hashes; read-only source observation, no selection/admission/Owner grant"}),
    )
}
pub fn run(phase: &str) -> io::Result<()> {
    if !matches!(phase, "before" | "after") {
        return Err(invalid("version1 exact phase before/after required"));
    }
    let peer = context()?;
    let expected: Value = serde_json::from_slice(LOCATORS).map_err(io::Error::other)?;
    let started = Instant::now();
    let end = started + Duration::from_secs(120);
    let mut rows = Vec::new();
    for kind in ["objects", "records"] {
        for entry in expected[kind]
            .as_array()
            .ok_or_else(|| invalid("fixed locator members"))?
        {
            let path = Path::new(
                entry["path"]
                    .as_str()
                    .ok_or_else(|| invalid("fixed locator path"))?,
            );
            let result = objects::observe(path, entry["directory"].as_bool().unwrap_or(false), end);
            let validation = result
                .as_ref()
                .map_err(|e| e.to_string())
                .and_then(|actual| {
                    objects::compare(entry, actual).map_err(|e| e.to_string())?;
                    if let Some(parent) = entry.get("parentId") {
                        let actual =
                            objects::observe(path.parent().ok_or("parent absent")?, true, end)
                                .map_err(|e| e.to_string())?;
                        if parent != &actual["security"]["fileId"] {
                            return Err("exact native parent ID differs".into());
                        }
                    }
                    Ok(())
                });
            rows.push(json!({"expected":entry,"actual":result.as_ref().ok(),"passed":validation.is_ok(),"error":validation.err(),"raw":result.as_ref().err().and_then(io::Error::raw_os_error)}));
        }
    }
    let source = source_at(
        &Path::new(BASE).join("I/runtime/versions/2.99.0"),
        &Path::new(BASE).join("I/runtime/current.json"),
        &expected,
        end,
    );
    let passed = source.is_ok() && rows.iter().all(|row| row["passed"] == true);
    let report = json!({"schema":"windows-r7-readonly-observation-v1","collector":"standalone actual native query + unchanged full inventory section; not Tauri product","phase":phase,"sid":peer.identity().token.sid,"session":peer.identity().token.session_id,"ordinaryNoJob":true,"rows":rows,"fullSourcePassed":source.is_ok(),"source":source.as_ref().ok(),"sourceError":source.as_ref().err().map(|e|e.to_string()),"passed":passed,"elapsedMs":started.elapsed().as_millis(),"observationLimitSeconds":120,"isActionWindow":false,"executionReady":false,"restoreExecuted":false,"forced":false,"normalStop":false,"closed":false,"publicStartOpen":false});
    peer.verify()?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(PUBLIC).join(format!("out01/native-{phase}.json")))?;
    output.write_all(&serde_json::to_vec_pretty(&report).map_err(io::Error::other)?)?;
    output.sync_all()?;
    if !passed {
        return Err(invalid(
            "read-only scene conflicts; consult all native rows",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r7_collector_refuses_foreign_controller_and_unknown_version_phase_before_any_scene_access() {
        assert!(run("resume").is_err());
        let current = native::current_identity().unwrap();
        if current.sid != "S-1-5-21-587408064-856152790-2583328096-1003" {
            assert!(run("before")
                .unwrap_err()
                .to_string()
                .contains("exact ordinary"));
        }
    }
    #[test]
    fn r7_collector_actual_native_objects_keep_8_bytes_children_and_reject_work_drift() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("owned-original.txt");
        std::fs::write(&file, b"original no credential").unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        let first = objects::observe(&file, false, end).unwrap();
        objects::compare(&first, &objects::observe(&file, false, end).unwrap()).unwrap();
        assert_eq!(
            first["security"]["projections"].as_array().unwrap().len(),
            8
        );
        let tree = objects::observe(root.path(), true, end).unwrap();
        assert_eq!(tree["children"], json!(["owned-original.txt"]));
        std::fs::write(&file, b"modified no credential").unwrap();
        assert!(objects::compare(&first, &objects::observe(&file, false, end).unwrap()).is_err());
        std::fs::write(root.path().join("unexpected"), b"x").unwrap();
        assert!(
            objects::compare(&tree, &objects::observe(root.path(), true, end).unwrap()).is_err()
        );
    }
    #[test]
    fn r7_collector_native_stream_query_rejects_ads_without_changing_default_bytes() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("owned.txt");
        std::fs::write(&file, b"default data").unwrap();
        super::super::tree::streams(&file, false).unwrap();
        let ads = PathBuf::from(format!("{}:r7-owned-ads", file.display()));
        std::fs::write(&ads, b"alternate data").unwrap();
        assert!(super::super::tree::streams(&file, false).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), b"default data");
        std::fs::remove_file(ads).unwrap();
        super::super::tree::streams(&file, false).unwrap();
    }
    #[test]
    #[ignore = "explicit complete fixed owned copy source observation; no private scene/Owner"]
    fn r7_collector_full_source_native_manifest_current_and_full_inventory_are_bound() {
        let root = PathBuf::from(
            std::env::var_os("OCXD024_R7_FIXED_SCAN_ROOT").expect("explicit fixed copied source"),
        );
        assert!(root
            .to_string_lossy()
            .contains("r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd"));
        let current = root
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("current.json");
        let expected: Value = serde_json::from_slice(LOCATORS).unwrap();
        let result = source_at(
            &root,
            &current,
            &expected,
            Instant::now() + Duration::from_secs(120),
        )
        .unwrap();
        assert_eq!(result["passed"], true);
        println!("R7_COLLECTOR_SOURCE {result}");
    }
}
