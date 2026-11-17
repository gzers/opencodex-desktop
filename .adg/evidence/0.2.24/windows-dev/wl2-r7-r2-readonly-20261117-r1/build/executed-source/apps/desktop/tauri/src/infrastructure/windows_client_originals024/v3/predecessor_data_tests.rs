use super::*;
use crate::infrastructure::windows_official_runtime024::{
    create_private_directory, PrivateRecord, RetainedPeer,
};
use std::{
    fs::OpenOptions,
    os::windows::fs::OpenOptionsExt,
    time::{Duration, Instant},
};
struct Setup {
    fixture: crate::infrastructure::windows_client_originals024::tree::tests::Fixture,
    binding: Binding,
    windows: PinnedDirectory,
    refs: Vec<Predecessor>,
    _records: Vec<PrivateRecord>,
    _pins: [PinnedDirectory; 4],
}
fn setup() -> Setup {
    let (fixture, _plan, profile, _) = super::super::tests::plan();
    let pins = [
        PinnedDirectory::open_private(&profile).unwrap(),
        create_private_directory(&fixture.path().join("D")).unwrap(),
        create_private_directory(&fixture.path().join("H")).unwrap(),
        create_private_directory(&fixture.path().join("I")).unwrap(),
    ];
    let official = create_private_directory(&pins[1].path().join("official024")).unwrap();
    let windows = create_private_directory(&official.path().join("windows")).unwrap();
    let binding = Binding::from_native(
        &RetainedPeer::current().unwrap(),
        &pins,
        "a".repeat(64),
        "b".repeat(64),
    )
    .unwrap();
    let binding_digest = hash::sha256_hex(&encoded(&binding).unwrap());
    let mut refs = Vec::new();
    let mut records = Vec::new();
    for _ in 0..2 {
        let session = uuid::Uuid::new_v4().to_string();
        let dir = create_private_directory(&windows.path().join(&session)).unwrap();
        let file = PrivateRecord::create(
            &dir.path().join("current.json"),
            b"synthetic locator DATA only, no original closed proof",
            4096,
        )
        .unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        let df = OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(1 | 2)
            .custom_flags(0x2200000)
            .open(dir.path())
            .unwrap();
        let cf = OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(1 | 2)
            .custom_flags(0x200000)
            .open(dir.path().join("current.json"))
            .unwrap();
        let ds =
            crate::infrastructure::windows_client_originals024::bounded::native_stamp(&df, end)
                .unwrap();
        let cs =
            crate::infrastructure::windows_client_originals024::bounded::native_stamp(&cf, end)
                .unwrap();
        let raw = std::fs::read(dir.path().join("current.json")).unwrap();
        refs.push(Predecessor {
            schema: "windows-anchored-closed-predecessor-v3".into(),
            session_id: session,
            binding_digest: binding_digest.clone(),
            directory: dir.path().into(),
            volume: dir.identity().volume,
            directory_id: dir.identity().index.to_string(),
            directory_security_digest: hash::sha256_hex(&encoded(&ds).unwrap()),
            current_id: file.identity().index.to_string(),
            current_security_digest: hash::sha256_hex(&encoded(&cs).unwrap()),
            current_sha256: hash::sha256_hex(&raw),
            manifest_digest: "c".repeat(64),
            sequence: 123,
            event_digest: "d".repeat(64),
            content_digest: "e".repeat(64),
            route_close_digest: "f".repeat(64),
            closed_digest: "a".repeat(64),
            native_completion_digest: "b".repeat(64),
            prior_set_digest: set_digest(&refs).unwrap(),
        });
        records.push(file);
    }
    Setup {
        fixture,
        binding,
        windows,
        refs,
        _records: records,
        _pins: pins,
    }
}
fn copy(refs: &[Predecessor]) -> Vec<Predecessor> {
    serde_json::from_slice(&encoded(&refs).unwrap()).unwrap()
}
#[test]
fn wl2_r6_predecessors_data_exact_order_scope_ids_and_complete_prior_prefix_refuse_unknown() {
    let mut s = setup();
    validate_set(&s.refs, &s.binding).unwrap();
    let good = serde_json::to_value(&s.refs).unwrap();
    let mut values = Vec::new();
    for field in [
        "schema",
        "sessionId",
        "bindingDigest",
        "directoryId",
        "currentId",
        "directorySecurityDigest",
        "currentSecurityDigest",
        "currentSha256",
        "manifestDigest",
        "eventDigest",
        "contentDigest",
        "routeCloseDigest",
        "closedDigest",
        "nativeCompletionDigest",
        "priorSetDigest",
    ] {
        let mut v = good.clone();
        v[0][field] = "foreign".into();
        values.push(v);
    }
    let mut v = good.clone();
    v.as_array_mut().unwrap().swap(0, 1);
    values.push(v);
    let mut v = good.clone();
    v.as_array_mut().unwrap().remove(0);
    values.push(v);
    let mut v = good.clone();
    v[1] = v[0].clone();
    values.push(v);
    let mut v = good.clone();
    v[1]["directory"] = "C:/outside".into();
    values.push(v);
    let mut v = good.clone();
    v[1]["sequence"] = 0.into();
    values.push(v);
    let mut v = good.clone();
    v[1]["sequence"] = 4097.into();
    values.push(v);
    let mut v = good.clone();
    v[1]["currentId"] = v[0]["currentId"].clone();
    values.push(v);
    let mut v = good.clone();
    v[1]["nativeAuthority"] = true.into();
    values.push(v);
    for v in values {
        assert!(serde_json::from_value::<Vec<Predecessor>>(v)
            .map(|refs| validate_set(&refs, &s.binding).is_err())
            .unwrap_or(true));
    }
    let raw = String::from_utf8(encoded(&good).unwrap())
        .unwrap()
        .replacen(
            "\"sessionId\":",
            "\"sessionId\":\"foreign\",\"sessionId\":",
            1,
        );
    assert!(serde_json::from_str::<Vec<Predecessor>>(&raw).is_err());
    println!("r6 predecessor DATA exact ordered prefix/same broker binding/session directory/nativeIDs/all8/current/closure hashes/bounds rejects23malformed sets and duplicate fields; synthetic locator records do not construct NativePredecessors or completed native histories");
    s.fixture.retain();
}
#[test]
fn wl2_r6_predecessors_native_namespace_exact_membership_new_active_and_foreign_current_hash_refuse(
) {
    let mut s = setup();
    let end = Instant::now() + Duration::from_secs(10);
    check_namespace(&s.windows, &s.refs, &s.binding, None, end).unwrap();
    assert!(
        check_namespace(&s.windows, &s.refs[..1], &s.binding, None, end).is_err(),
        "unknown extra directory not adopted by Closed text"
    );
    let active =
        create_private_directory(&s.windows.path().join(uuid::Uuid::new_v4().to_string())).unwrap();
    check_namespace(&s.windows, &s.refs, &s.binding, Some(&active), end).unwrap();
    assert!(check_namespace(&s.windows, &s.refs, &s.binding, None, end).is_err());
    let mut wrong = copy(&s.refs);
    wrong[1].current_sha256 = "e".repeat(64);
    assert!(
        check_namespace(&s.windows, &wrong, &s.binding, Some(&active), end).is_err(),
        "valid-form hash cannot substitute original bytes"
    );
    let mut wrong = copy(&s.refs);
    wrong[1].directory_id = "999".into();
    assert!(check_namespace(&s.windows, &wrong, &s.binding, Some(&active), end).is_err());
    let mut wrong = copy(&s.refs);
    wrong[1].directory_security_digest = "e".repeat(64);
    assert!(check_namespace(&s.windows, &wrong, &s.binding, Some(&active), end).is_err());
    let foreign = create_private_directory(&s.windows.path().join("old-v2-text-Closed")).unwrap();
    assert!(check_namespace(&s.windows, &s.refs, &s.binding, Some(&active), end).is_err());
    assert!(foreign.path().is_dir());
    println!("r6 native namespace pins reobserve exact parent/current directory IDs/all8 digests/current bytes/streams and BOTH membership barriers; known new active allowed only explicitly, missing/extra/foreigntextClosed/wrongID/security/hash refuse. Synthetic records DATA only, no native history adoption");
    s.fixture.retain();
}
