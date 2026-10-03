//! 双端同步的真实链路（无网络）：两个隔离数据根之间走序列化后的传输包。
//!
//! 覆盖：新格式无需额外口令的上传/下载/重复同步/冲突，旧格式需要原口令，
//! 以及载荷损坏与清单篡改的拒绝路径。

use base64::Engine as _;
use chrono::{TimeZone, Utc};
use opencodex_desktop_lib::modules::sync::engine::{
    apply_remote_snapshot, apply_remote_snapshot_batch, build_local_snapshot, open_manifest,
    open_payload, parse_manifest_package, parse_payload_package, seal_legacy_manifest,
    seal_legacy_payload, sync_state_dir, SyncDecision, SyncState,
};

const PREFERENCES: &str = "manager-state/preferences.json";
const EXTENSION: &str = "manager-state/extension-config.json";

fn client_root(name: &str) -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect(name);
    opencodex_desktop_lib::modules::data_root::initialize(temp.path()).expect("initialize");
    temp
}

/// 一端生成快照并"上传"：返回传输层看到的三类字节。
fn upload(files: Vec<(&str, &str)>) -> (Vec<u8>, Vec<Vec<u8>>) {
    let artifacts = files
        .into_iter()
        .map(|(name, body)| (name.to_string(), body.as_bytes().to_vec()))
        .collect::<Vec<_>>();
    let now = Utc.with_ymd_and_hms(2026, 9, 20, 3, 0, 0).unwrap();
    let (manifest, payloads) = build_local_snapshot(artifacts, "client-a", now).expect("snapshot");
    let manifest_bytes = {
        let package =
            opencodex_desktop_lib::modules::sync::engine::seal_manifest(&manifest).expect("seal");
        serde_json::to_vec(&package).expect("manifest json")
    };
    let payload_bytes = payloads
        .iter()
        .map(|(_, envelope)| serde_json::to_vec(envelope).expect("payload json"))
        .collect();
    (manifest_bytes, payload_bytes)
}

#[test]
fn two_isolated_clients_sync_without_any_extra_passphrase() {
    let target = client_root("client-b");
    let (manifest_bytes, payload_bytes) = upload(vec![
        (PREFERENCES, "{\"interface_scale\":130}"),
        (EXTENSION, "{\"config\":{}}"),
    ]);

    let package = parse_manifest_package(&manifest_bytes).expect("parse manifest");
    let manifest = open_manifest(&package, None).expect("open without passphrase");
    assert_eq!(manifest.artifacts.len(), 2);
    assert!(!manifest.snapshot_id.is_empty());

    let mut state = SyncState::new("default");
    let now = Utc.with_ymd_and_hms(2026, 9, 20, 3, 5, 0).unwrap();
    let mut applied: Vec<(std::path::PathBuf, Vec<u8>)> = Vec::new();
    for (artifact, bytes) in manifest.artifacts.iter().zip(payload_bytes.iter()) {
        let payload = parse_payload_package(bytes).expect("parse payload");
        let data = open_payload(&payload, &artifact.sha256).expect("open payload");
        assert_eq!(data.len() as u64, artifact.size);
        assert_eq!(
            opencodex_desktop_lib::modules::sync::engine::sha256_hex(&data),
            artifact.sha256
        );
        applied.push((target.path().join(&artifact.name), data));
    }
    // 同一快照的多个文件必须整批应用：逐文件调用会被误判回放（历史半应用缺陷）。
    let batch: Vec<(&std::path::Path, &[u8])> = applied
        .iter()
        .map(|(path, data)| (path.as_path(), data.as_slice()))
        .collect();
    let decision = apply_remote_snapshot_batch(
        &mut state,
        target.path(),
        &batch,
        &manifest.snapshot_id,
        &manifest.created_at,
        "manifest-hash",
        now,
    )
    .expect("apply");
    assert!(matches!(decision, SyncDecision::Applied { .. }));
    assert!(state.conflicts.is_empty());
    for (path, _) in &applied {
        assert!(path.exists(), "{path:?} was not written");
    }
    assert_eq!(
        std::fs::read_to_string(target.path().join(PREFERENCES)).expect("read"),
        "{\"interface_scale\":130}"
    );
    state.save(target.path()).expect("save state");
    let dir = sync_state_dir(target.path());
    assert!(dir.is_dir());

    // 重复同步同一快照：判定为回放冲突，不再覆盖本地。
    let later = Utc.with_ymd_and_hms(2026, 9, 20, 3, 10, 0).unwrap();
    let decision = apply_remote_snapshot(
        &mut state,
        target.path(),
        &target.path().join(PREFERENCES),
        b"other",
        &manifest.snapshot_id,
        &manifest.created_at,
        "manifest-hash",
        later,
    )
    .expect("replay");
    assert_eq!(decision, SyncDecision::Conflict);
    assert_eq!(state.conflicts.len(), 1);
}

#[test]
fn tampered_payload_and_manifest_are_rejected() {
    let (manifest_bytes, payload_bytes) = upload(vec![(PREFERENCES, "{\"a\":1}")]);
    let package = parse_manifest_package(&manifest_bytes).expect("parse manifest");
    let manifest = open_manifest(&package, None).expect("open");
    let artifact = &manifest.artifacts[0];

    // 载荷被改动：完整性不符，拒绝。
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&payload_bytes[0]).expect("payload json");
    tampered["data_b64"] =
        serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(b"{\"a\":2}"));
    let payload = parse_payload_package(&serde_json::to_vec(&tampered).unwrap()).expect("parse");
    assert!(open_payload(&payload, &artifact.sha256).is_err());

    // 清单被改动：完整性不符，拒绝。
    let mut manifest_value: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).expect("manifest json");
    manifest_value["manifest"]["device_name"] = serde_json::Value::String("evil".to_string());
    let package =
        parse_manifest_package(&serde_json::to_vec(&manifest_value).unwrap()).expect("parse");
    assert!(open_manifest(&package, None).is_err());
}

#[test]
fn legacy_transport_packages_need_the_original_passphrase() {
    let (_, payload_bytes) = upload(vec![(PREFERENCES, "{\"a\":1}")]);
    let now = Utc.with_ymd_and_hms(2026, 9, 20, 3, 0, 0).unwrap();
    let legacy_manifest = {
        let artifacts = vec![opencodex_desktop_lib::modules::sync::SyncArtifact {
            name: PREFERENCES.to_string(),
            size: 7,
            sha256: opencodex_desktop_lib::modules::sync::engine::sha256_hex(b"{\"a\":1}"),
        }];
        let manifest = opencodex_desktop_lib::modules::sync::SyncManifest::new(
            "snap_20260920030000_aaaaaaaaaaaa",
            now,
            "client-a",
            artifacts,
        )
        .expect("manifest");
        seal_legacy_manifest(&manifest, b"legacy-pass").expect("seal legacy")
    };
    let package =
        parse_manifest_package(&serde_json::to_vec(&legacy_manifest).unwrap()).expect("parse");
    // 没有原口令：明确拒绝，而不是静默按新格式处理。
    assert!(open_manifest(&package, None).is_err());
    assert!(open_manifest(&package, Some(b"wrong")).is_err());
    assert!(open_manifest(&package, Some(b"legacy-pass")).is_ok());

    // 旧版载荷同样需要按清单摘要派生的密钥材料。
    let digest = opencodex_desktop_lib::modules::sync::engine::sha256_hex(b"{\"a\":1}");
    let legacy_payload = seal_legacy_payload(b"{\"a\":1}", &digest).expect("seal legacy payload");
    let parsed =
        parse_payload_package(&serde_json::to_vec(&legacy_payload).unwrap()).expect("parse");
    assert_eq!(open_payload(&parsed, &digest).expect("open"), b"{\"a\":1}");
    // 新格式载荷不受影响（对照）。
    let payload = parse_payload_package(&payload_bytes[0]).expect("parse new payload");
    assert_eq!(
        open_payload(&payload, &digest).expect("open new"),
        b"{\"a\":1}"
    );
}
