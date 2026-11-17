//! New diagnostic I/D/H, exact unchanged prior synthetic objects only. These
//! setup references never reconstruct a lease, Owner, NormalExit or Closed.
use super::*;
const OLD: &str = r"C:\Users\marenben\ocx024-r6-existing-20261117-r1";
const CLEANUP: &str =
    r"C:\Users\Public\ocx-r6-authorized-cleanup-20261117-r1\out01\cleanup-result.json";
const CLEANUP_SHA: &str = "7a8c90139cf67184853837c699d4182464c9800b1a0999f975166daf7067cf0e";
const BASELINE_SHA: &str = "52ac0a4802fa111d3c40e06147f27fd808cbd89f374972711cd9850c22983021";
const COORD_SHA: &str = "ca8e7264491abb312bfa725a966bbc8bf7601e288ac8ab023c0b26209b39cd67";
fn fixed_json(path: &Path, sha: &str) -> io::Result<Value> {
    let end = Instant::now() + Duration::from_secs(5);
    let proof = BoundedAccessV1::capture(path, false, end).map_err(invalid)?;
    let mut bytes = Vec::new();
    OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(0x200000)
        .open(path)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1024 * 1024 || hash::sha256_hex(&bytes) != sha {
        return Err(invalid("exact prior fixture reference bytes changed"));
    }
    proof.recheck(end).map_err(invalid)?;
    serde_json::from_slice(&bytes).map_err(invalid)
}
fn cleanup_shape(value: &Value) -> io::Result<()> {
    let c = &value["cleanup"];
    if value["sid"] != "S-1-5-21-587408064-856152790-2583328096-1003"
        || value["session"] != 7
        || value["completed"] != true
        || value["dailyPathUnchanged"] != true
        || value["freshObservationHandlesClosed"] != true
        || c["terminatedPids"] != json!([50580, 54692])
        || c["rawExits"] != json!([242, 242])
        || c["before"][0]["filetime"] != "134393577919362322"
        || c["before"][1]["filetime"] != "134393577413203335"
        || c["before"][0]["threadWaitReason"] != "Suspended"
    {
        return Err(invalid(
            "exact authorized old failure cleanup receipt differs",
        ));
    }
    for key in [
        "bothValidated",
        "preconditionsRevalidated",
        "sourcesRecheckedAfter",
        "attempted",
        "completed",
        "forced",
    ] {
        if c[key] != true {
            return Err(invalid("old cleanup incomplete/unknown"));
        }
    }
    for key in [
        "normalStop",
        "closed",
        "filesDeleted",
        "allCompanionsProven",
        "publicStartOpen",
    ] {
        if value[key] != false || c[key] != false {
            return Err(invalid(
                "forced failure cleanup cannot mint normal lifecycle proof",
            ));
        }
    }
    Ok(())
}
fn check_original_row(row: &Value) -> io::Result<()> {
    let path = Path::new(
        row["path"]
            .as_str()
            .ok_or_else(|| invalid("original fixture path missing"))?,
    );
    let directory = row["directory"]
        .as_bool()
        .ok_or_else(|| invalid("original fixture kind missing"))?;
    if snapshot(path, directory, Instant::now() + Duration::from_secs(5))? != row["security"] {
        return Err(invalid("prior synthetic original ID/8-projection changed"));
    }
    if !directory {
        let mut bytes = Vec::new();
        File::open(path)?.take(4097).read_to_end(&mut bytes)?;
        if bytes.len() > 4096
            || row["bytes"].as_u64() != Some(bytes.len() as u64)
            || row["sha256"] != hash::sha256_hex(&bytes)
        {
            return Err(invalid("prior credential-free synthetic bytes changed"));
        }
    }
    Ok(())
}
pub(super) fn prepare_baseline(base: &Path) -> io::Result<Value> {
    cleanup_shape(&fixed_json(Path::new(CLEANUP), CLEANUP_SHA)?)?;
    let old = fixed_json(
        &Path::new(OLD).join("preparation/result.json"),
        BASELINE_SHA,
    )?;
    if old["preparedExistingBaseline"] != true
        || old["actualProductionExecuted"] != false
        || old["originalClientRootsBeforePreparation"] != "Absent"
    {
        return Err(invalid("original synthetic preparation provenance differs"));
    }
    let old_rows = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("original fixture rows absent"))?;
    baseline_shape(Path::new(OLD), old_rows)?;
    // Old directories and client files are observed, never moved/recreated or
    // reclassified as product first-use initialization or a restored old phase.
    for row in old_rows {
        check_original_row(row)?;
    }
    let mut rows = Vec::new();
    for name in ["I", "D", "H"] {
        let path = base.join(name);
        let pin = native::PinnedDirectory::open_private(&path)?;
        rows.push(json!({"path":path,"directory":true,"volume":pin.identity().volume,
            "fileId":pin.identity().index.to_string(),"security":snapshot(&path,true,Instant::now()+Duration::from_secs(5))?}));
        pin.recheck()?;
    }
    rows.extend(old_rows[3..].iter().cloned());
    baseline_shape(base, &rows)?;
    let path = base.join("preparation");
    absent(&path)?;
    let ledger = native::create_private_directory(&path)?;
    let value = json!({"base":base,"preparedExistingBaseline":true,"actualProductionExecuted":false,
        "originalClientRootsBeforePreparation":"PresentOriginalSyntheticBaseline","rows":rows,
        "originalPreparationAbsenceHistoricalOnly":true,"oldBaselineSha256":BASELINE_SHA,
        "externalFailureCleanupReceiptSha256":CLEANUP_SHA,"sameFailedTransactionRetried":false,
        "officialInitializationProven":false,"originalAbsentRetirementProven":false,
        "nativeLeaseGranted":false,"normalStop":false,"closed":false,"startOpen":false});
    write_new(&ledger.path().join("result.json"), &value)?;
    ledger.recheck()?;
    Ok(value)
}
pub(super) fn coordination_baseline(
    base: &Path,
    ledger: &Path,
    token: &native::TokenIdentity,
) -> io::Result<Value> {
    let old = fixed_json(
        &Path::new(OLD).join("qualification-preparation/coordination-created.json"),
        COORD_SHA,
    )?;
    let root = coordination::official_runtime_root(token)?;
    let client = Path::new(r"C:\Users\marenben\.codex");
    let canonical = fs::canonicalize(client)?;
    if !dunce::simplified(&canonical)
        .to_string_lossy()
        .eq_ignore_ascii_case(&client.to_string_lossy())
    {
        return Err(invalid(
            "actual unchanged synthetic client canonical path differs",
        ));
    }
    let digest = hash::sha256_hex(
        dunce::simplified(&canonical)
            .to_str()
            .ok_or_else(|| invalid("client canonical utf8"))?
            .as_bytes(),
    );
    let paths = [
        root.join("native-write-locks")
            .join(format!("{digest}.sqlite")),
        root.join("client-desktop-lifecycle.sqlite"),
        root.join("catalog-write-locks")
            .join(format!("{digest}.sqlite")),
    ];
    let objects = old["objects"]
        .as_array()
        .ok_or_else(|| invalid("old synthetic coordination set absent"))?;
    if objects.len() != paths.len() {
        return Err(invalid("old coordination set incomplete/extra"));
    }
    let states = paths
        .iter()
        .map(
            |p| match snapshot(p, false, Instant::now() + Duration::from_secs(5)) {
                Ok(security) => json!({"path":p,"state":"Present","security":security}),
                Err(e) => {
                    json!({"path":p,"state":"Unknown","error":e.to_string(),"raw":e.raw_os_error()})
                }
            },
        )
        .collect::<Vec<_>>();
    write_new(
        &ledger.join("coordination-before.json"),
        &json!({"states":states,"readOnly":true,"nativeLeaseGranted":false}),
    )?;
    for ((row, path), state) in objects.iter().zip(&paths).zip(&states) {
        if row["path"].as_str().map(Path::new) != Some(path.as_path())
            || row["bytes"] != 0
            || state["state"] != "Present"
            || state["security"] != row["security"]
            || fs::metadata(path)?.len() != 0
        {
            return Err(invalid(
                "exact prior synthetic coordinator ID/8/empty bytes differ",
            ));
        }
        if !native::PrivateRecord::locator_snapshot(path, 4096)?.is_empty() {
            return Err(invalid("synthetic coordinator changed during readback"));
        }
    }
    let result = json!({"base":base,"objects":objects,"priorPreparationSha256":COORD_SHA,
        "createdByCurrentScene":false,"readOnlyExactSyntheticBaseline":true,
        "nativeLeaseGranted":false,"oldLeaseAdopted":false,"officialInitializationProven":false});
    write_new(&ledger.join("coordination-retained.json"), &result)?;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn receipt() -> Value {
        json!({"sid":"S-1-5-21-587408064-856152790-2583328096-1003","session":7,"completed":true,"dailyPathUnchanged":true,"freshObservationHandlesClosed":true,
            "normalStop":false,"closed":false,"filesDeleted":false,"allCompanionsProven":false,"publicStartOpen":false,
            "cleanup":{"terminatedPids":[50580,54692],"rawExits":[242,242],"bothValidated":true,"preconditionsRevalidated":true,"sourcesRecheckedAfter":true,"attempted":true,"completed":true,"forced":true,
                "before":[{"filetime":"134393577919362322","threadWaitReason":"Suspended"},{"filetime":"134393577413203335"}],"normalStop":false,"closed":false,"filesDeleted":false,"allCompanionsProven":false,"publicStartOpen":false}})
    }
    #[test]
    fn wl2_r6_fresh_fixture_forced_receipt_shape_refuses_normal_or_incomplete_facts() {
        assert!(cleanup_shape(&receipt()).is_ok());
        for key in [
            "normalStop",
            "closed",
            "filesDeleted",
            "allCompanionsProven",
            "publicStartOpen",
        ] {
            let mut v = receipt();
            v[key] = json!(true);
            assert!(cleanup_shape(&v).is_err());
            let mut v = receipt();
            v["cleanup"][key] = json!(true);
            assert!(cleanup_shape(&v).is_err());
        }
        for key in [
            "bothValidated",
            "preconditionsRevalidated",
            "sourcesRecheckedAfter",
            "attempted",
            "completed",
            "forced",
        ] {
            let mut v = receipt();
            v["cleanup"][key] = Value::Null;
            assert!(cleanup_shape(&v).is_err());
        }
        for (key, bad) in [
            ("rawExits", json!([0, 0])),
            ("terminatedPids", json!([54692, 50580])),
        ] {
            let mut v = receipt();
            v["cleanup"][key] = bad;
            assert!(cleanup_shape(&v).is_err());
        }
        let mut v = receipt();
        v["cleanup"]["before"][0]["threadWaitReason"] = json!("Unknown");
        assert!(cleanup_shape(&v).is_err());
    }
}
