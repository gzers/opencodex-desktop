//! Canonical pending-record architecture. Deserializing a complete phase chain
//! only discovers recovery work; it never recreates native restoration proofs.
use super::binding::Binding;
use crate::infrastructure::{
    hash,
    windows_official_runtime024::{create_private_directory, PinnedDirectory, PrivateRecord},
};
use serde::{Deserialize, Serialize};
use std::{io, path::Path};
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Phase {
    Captured,
    Ready,
    PreparedRestore,
    OfficialTeardown,
    Restored,
    Closed,
    Aborted,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Record {
    schema: String,
    session_id: String,
    sequence: u32,
    previous_digest: String,
    binding: Binding,
    phase: Phase,
    originals_manifest_digest: String,
    previous_record_id: Option<String>,
}
pub(super) struct PendingOnly {
    _directory: PinnedDirectory,
    _records: Vec<PrivateRecord>,
    pub(super) terminal_record_observed: bool,
}
pub(super) fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original phase chain unknown; preserve pending custody/Required",
    )
}
pub(super) fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn next(a: &Phase, b: &Phase) -> bool {
    matches!(
        (a, b),
        (Phase::Captured, Phase::Ready | Phase::Aborted)
            | (Phase::Ready, Phase::PreparedRestore | Phase::Aborted)
            | (Phase::PreparedRestore, Phase::OfficialTeardown)
            | (Phase::OfficialTeardown, Phase::Restored)
            | (Phase::Restored, Phase::Closed)
    )
}
fn check_prefix(bytes: &[Vec<u8>]) -> io::Result<bool> {
    check_prefix_for_sid(bytes, None)
}
fn check_prefix_for_sid(bytes: &[Vec<u8>], expected_sid: Option<&str>) -> io::Result<bool> {
    if bytes.is_empty() || bytes.len() > 512 {
        return Err(invalid());
    }
    let mut previous: Option<Record> = None;
    let mut last_digest = String::new();
    for (i, raw) in bytes.iter().enumerate() {
        if raw.len() > 65536 {
            return Err(invalid());
        }
        let r: Record = serde_json::from_slice(raw).map_err(|_| invalid())?;
        let session = uuid::Uuid::parse_str(&r.session_id).map_err(|_| invalid())?;
        r.binding.validate(expected_sid)?;
        if r.schema != "windows-client-originals-v1"
            || session.is_nil()
            || session.to_string() != r.session_id
            || r.sequence != i as u32 + 1
            || r.previous_digest != last_digest
            || !digest(&r.originals_manifest_digest)
        {
            return Err(invalid());
        }
        match &previous {
            None if r.phase != Phase::Captured || r.previous_record_id.is_some() => {
                return Err(invalid())
            }
            Some(p)
                if r.session_id != p.session_id
                    || r.binding != p.binding
                    || r.originals_manifest_digest != p.originals_manifest_digest
                    || r.previous_record_id.as_deref()
                        != Some(&format!("phase-{:08}.json", p.sequence))
                    || !next(&p.phase, &r.phase) =>
            {
                return Err(invalid())
            }
            _ => {}
        }
        last_digest = hash::sha256_hex(raw);
        previous = Some(r);
    }
    Ok(previous.is_some_and(|r| r.phase == Phase::Closed))
}
/// Read canonical create-only records, retain their exact native handles and
/// refuse unknown extras or gaps. Even a textual Closed record is pending until
/// the original host's native restore/source/security/exit proofs are available.
pub(super) fn read_pending(session_dir: &Path) -> io::Result<PendingOnly> {
    let dir = PinnedDirectory::open_private(session_dir)?;
    let mut names = std::fs::read_dir(session_dir)?
        .map(|e| e.map(|e| e.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    if names.is_empty() || names.len() > 512 {
        return Err(invalid());
    }
    let mut handles = vec![];
    let mut records = vec![];
    for (i, name) in names.iter().enumerate() {
        if name.to_str() != Some(&format!("phase-{:08}.json", i + 1)) {
            return Err(invalid());
        }
        let mut file = PrivateRecord::open(&session_dir.join(name), 65536)?;
        records.push(file.read()?);
        handles.push(file);
    }
    let sid = crate::infrastructure::windows_official_runtime024::current_identity()?.sid;
    let closed = check_prefix_for_sid(&records, Some(&sid))?;
    let first: Record = serde_json::from_slice(&records[0]).map_err(|_| invalid())?;
    if session_dir.file_name().and_then(|name| name.to_str()) != Some(&first.session_id) {
        return Err(invalid());
    }
    dir.recheck()?;
    Ok(PendingOnly {
        _directory: dir,
        _records: handles,
        terminal_record_observed: closed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn synthetic_binding() -> Binding {
        // Synthetic declared paths/IDs only; no native identity is reconstructed.
        serde_json::from_value(serde_json::json!({
            "sid": crate::infrastructure::windows_official_runtime024::current_identity().unwrap().sid,
            "profile":{"path":"C:/task/Profile", "volume":1,"fileId":"1"},
            "data":{"path":"D:/task/D", "volume":2,"fileId":"2"},
            "home":{"path":"C:/task/H", "volume":1,"fileId":"3"},
            "installation":{"path":"C:/task/I", "volume":1,"fileId":"4"},
            "sourceDigest":"a".repeat(64),"currentDigest":"b".repeat(64),
            "owner":{"pid":1234,"creationFiletime":"123456789"}
        })).unwrap()
    }
    fn chain(phases: &[Phase]) -> Vec<Vec<u8>> {
        let mut result = vec![];
        let session = uuid::Uuid::new_v4().to_string();
        for (i, p) in phases.iter().enumerate() {
            let r = Record {
                schema: "windows-client-originals-v1".into(),
                session_id: session.clone(),
                sequence: i as u32 + 1,
                previous_digest: result
                    .last()
                    .map_or(String::new(), |b: &Vec<u8>| hash::sha256_hex(b)),
                binding: synthetic_binding(),
                phase: serde_json::from_value(serde_json::to_value(p).unwrap()).unwrap(),
                originals_manifest_digest: "a".repeat(64),
                previous_record_id: if i == 0 {
                    None
                } else {
                    Some(format!("phase-{i:08}.json"))
                },
            };
            result.push(serde_json::to_vec(&r).unwrap());
        }
        result
    }
    #[test]
    fn wl2_r3_every_phase_prefix_remains_pending_no_native_closed_authority() {
        let bytes = chain(&[
            Phase::Captured,
            Phase::Ready,
            Phase::PreparedRestore,
            Phase::OfficialTeardown,
            Phase::Restored,
            Phase::Closed,
        ]);
        for n in 1..=bytes.len() {
            assert_eq!(check_prefix(&bytes[..n]).unwrap(), n == 6);
        }
        let root = tempfile::tempdir().unwrap();
        let first: Record = serde_json::from_slice(&bytes[0]).unwrap();
        let dir = create_private_directory(&root.path().join(first.session_id)).unwrap();
        for (i, b) in bytes.iter().enumerate() {
            let path = dir.path().join(format!("phase-{:08}.json", i + 1));
            let mut record = PrivateRecord::create(&path, b, 65536).unwrap();
            assert_eq!(record.read().unwrap(), *b);
            assert!(PrivateRecord::create(&path, b, 65536).is_err());
            drop(record);
            let pending = read_pending(dir.path()).unwrap();
            assert_eq!(pending.terminal_record_observed, i == 5);
            // No ClosedEligibility or NativeNormalExit constructor exists here.
            drop(pending);
        }
        drop(dir);
    }
    #[test]
    fn wl2_r3_phase_gaps_replays_unknown_schema_and_rollback_refuse() {
        let good = chain(&[Phase::Captured, Phase::Ready, Phase::PreparedRestore]);
        assert!(check_prefix(&[good[0].clone(), good[2].clone()]).is_err());
        assert!(check_prefix(&[good[0].clone(), good[1].clone(), good[1].clone()]).is_err());
        for field in [
            "schema",
            "sessionId",
            "sequence",
            "previousDigest",
            "originalsManifestDigest",
            "previousRecordId",
            "binding",
            "phase",
        ] {
            let mut bad = good.clone();
            let mut value: serde_json::Value = serde_json::from_slice(&bad[1]).unwrap();
            value[field] = if field == "sequence" {
                serde_json::json!(999)
            } else {
                serde_json::json!("invalid-drift")
            };
            bad[1] = serde_json::to_vec(&value).unwrap();
            assert!(check_prefix(&bad).is_err(), "field drift accepted {field}");
        }
        assert!(check_prefix(&chain(&[Phase::Captured, Phase::Closed])).is_err());
        assert!(check_prefix(&chain(&[
            Phase::Captured,
            Phase::Ready,
            Phase::Aborted,
            Phase::Captured
        ]))
        .is_err());
    }
    #[test]
    fn wl2_originals_binding_requires_every_identity_field_and_disjoint_scope() {
        let good = chain(&[Phase::Captured]);
        let value: serde_json::Value = serde_json::from_slice(&good[0]).unwrap();
        assert!(check_prefix(&good).is_ok());
        let mut empty = value.clone();
        empty["binding"] = serde_json::json!({});
        assert!(check_prefix(&[serde_json::to_vec(&empty).unwrap()]).is_err());
        for field in [
            "sid",
            "profile",
            "data",
            "home",
            "installation",
            "sourceDigest",
            "currentDigest",
            "owner",
        ] {
            let mut missing = value.clone();
            missing["binding"].as_object_mut().unwrap().remove(field);
            assert!(
                check_prefix(&[serde_json::to_vec(&missing).unwrap()]).is_err(),
                "missing {field}"
            );
        }
        for sid in [
            "S-01-5-21-1",
            "S-1-05-21-1",
            "S-1-5",
            "S-1-5-4294967296",
            "foreign",
        ] {
            let mut bad = value.clone();
            bad["binding"]["sid"] = serde_json::json!(sid);
            assert!(check_prefix(&[serde_json::to_vec(&bad).unwrap()]).is_err());
        }
        assert!(check_prefix_for_sid(&good, Some("S-1-5-21-9-9-9-9")).is_err());
        for path in [
            "relative",
            "C:/task/../D",
            "C:/task/D:stream",
            "C:/task/CON",
            "C:/task/D.",
            "C:/task/H/inside",
            "C:/task/h",
            "C:/task/Profile/.codex/inside",
            "C:/task/Profile/.grok",
            "C:/task/Profile",
        ] {
            let mut bad = value.clone();
            bad["binding"]["data"]["path"] = serde_json::json!(path);
            assert!(
                check_prefix(&[serde_json::to_vec(&bad).unwrap()]).is_err(),
                "path {path}"
            );
        }
        for field in ["fileId", "path", "volume"] {
            let mut bad = value.clone();
            bad["binding"]["home"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(check_prefix(&[serde_json::to_vec(&bad).unwrap()]).is_err());
        }
        for decimal in ["0", "00", "+1", "01", "-1", "1e3", "18446744073709551616"] {
            for field in ["fileId", "creationFiletime"] {
                let mut bad = value.clone();
                if field == "fileId" {
                    bad["binding"]["data"][field] = serde_json::json!(decimal);
                } else {
                    bad["binding"]["owner"][field] = serde_json::json!(decimal);
                }
                assert!(check_prefix(&[serde_json::to_vec(&bad).unwrap()]).is_err());
            }
        }
        let mut same_id = value.clone();
        same_id["binding"]["home"]["fileId"] = serde_json::json!("1");
        assert!(check_prefix(&[serde_json::to_vec(&same_id).unwrap()]).is_err());
        let mut owner = value.clone();
        owner["binding"]["owner"]["pid"] = serde_json::json!(0);
        assert!(check_prefix(&[serde_json::to_vec(&owner).unwrap()]).is_err());
        for name in ["sourceDigest", "currentDigest"] {
            let mut bad = value.clone();
            bad["binding"][name] = serde_json::json!("F".repeat(64));
            assert!(check_prefix(&[serde_json::to_vec(&bad).unwrap()]).is_err());
        }
    }
    #[test]
    fn wl2_originals_nested_duplicates_unknowns_and_wrong_session_directory_refuse() {
        let good = chain(&[Phase::Captured]);
        let text = String::from_utf8(good[0].clone()).unwrap();
        for marker in ["\"binding\":{", "\"profile\":{", "\"owner\":{"] {
            let bad = text.replacen(marker, &format!("{marker}\"unknownAuthority\":true,"), 1);
            assert_ne!(bad, text);
            assert!(check_prefix(&[bad.into_bytes()]).is_err());
        }
        for marker in ["\"sid\":", "\"fileId\":", "\"pid\":"] {
            let inserted = if marker == "\"pid\":" {
                "1234,"
            } else {
                "\"1\","
            };
            let bad = text.replacen(marker, &format!("{marker}{inserted}{marker}"), 1);
            assert!(check_prefix(&[bad.into_bytes()]).is_err());
        }
        let mut nil: serde_json::Value = serde_json::from_str(&text).unwrap();
        nil["sessionId"] = serde_json::json!(uuid::Uuid::nil().to_string());
        assert!(check_prefix(&[serde_json::to_vec(&nil).unwrap()]).is_err());
        let root = tempfile::tempdir().unwrap();
        let wrong =
            create_private_directory(&root.path().join(uuid::Uuid::new_v4().to_string())).unwrap();
        let record =
            PrivateRecord::create(&wrong.path().join("phase-00000001.json"), &good[0], 65536)
                .unwrap();
        assert!(read_pending(wrong.path()).is_err());
        drop(record);
        drop(wrong);
        // No textual chain, even structurally Closed, creates native custody
        // or a restoration/successor capability in this pending-only decoder.
    }
}
