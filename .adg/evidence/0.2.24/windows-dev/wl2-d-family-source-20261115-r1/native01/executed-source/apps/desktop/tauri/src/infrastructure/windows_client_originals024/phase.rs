//! Canonical pending-record architecture. Deserializing a complete phase chain
//! only discovers recovery work; it never recreates native restoration proofs.
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
    binding: serde_json::Value,
    phase: Phase,
    originals_manifest_digest: String,
    previous_record_id: Option<String>,
}
pub(super) struct PendingOnly {
    _directory: PinnedDirectory,
    _records: Vec<PrivateRecord>,
    pub(super) terminal_record_observed: bool,
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original phase chain unknown; preserve pending custody/Required",
    )
}
fn digest(s: &str) -> bool {
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
        if r.schema != "windows-client-originals-v1"
            || uuid::Uuid::parse_str(&r.session_id).is_err()
            || r.sequence != i as u32 + 1
            || r.previous_digest != last_digest
            || !digest(&r.originals_manifest_digest)
            || !r.binding.is_object()
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
    let closed = check_prefix(&records)?;
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
                binding: serde_json::json!({"testOnlySyntheticScope":true,"nativeAuthority":false}),
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
        let dir = create_private_directory(&root.path().join("session")).unwrap();
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
}
