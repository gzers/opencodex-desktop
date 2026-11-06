//! Explicit loopback TLS fixture; no real endpoint or daily configuration.
#![cfg(feature = "integration-test")]
use opencodex_desktop_lib::{
    infrastructure::webdav_config_v2::FileWebDavTransport, modules::model_config::sync_v2::types::*,
};

#[tokio::test]
#[ignore = "requires explicitly started owned loopback TLS WebDAV fixture"]
async fn v2_depth_one_and_precise_file_transport_over_verified_tls() {
    let url = std::env::var("OCXD_V2_FIXTURE_URL").expect("owned loopback fixture URL");
    assert!(url.starts_with("https://localhost:") || url.starts_with("https://127.0.0.1:"));
    let ca = std::fs::read(std::env::var_os("OCXD_V2_FIXTURE_CA").expect("fixture CA")).unwrap();
    let transport = FileWebDavTransport::with_test_root_certificate(&ca).unwrap();
    let handle = ConnectionHandle {
        id: "owned-fixture".into(),
        binding_id: format!("sha256:{}", "a".repeat(64)),
        revision: format!("sha256:{}", "b".repeat(64)),
    };
    transport
        .bind(
            handle.clone(),
            &url,
            "v2-owned",
            "fixture".into(),
            "fixture-only-password".into(),
        )
        .unwrap();
    let list = ListLimits {
        max_entries: 100,
        max_xml_bytes: 1024 * 1024,
        timeout_ms: 3000,
    };
    assert!(transport
        .list_children(&handle, list)
        .await
        .unwrap()
        .entries
        .is_empty());
    let generation = uuid::Uuid::new_v4().to_string();
    let operation = uuid::Uuid::new_v4().to_string();
    let generation_path = SyncRelativePath::generation(&generation).unwrap();
    let payload_path = SyncRelativePath::data(&generation, &operation).unwrap();
    let commit_path = SyncRelativePath::commit(&generation, &operation).unwrap();
    let write = WriteLimits {
        max_bytes: 1024,
        timeout_ms: 3000,
    };
    let read = ReadLimits {
        max_bytes: 1024,
        timeout_ms: 3000,
    };
    for (path, bytes) in [
        (&generation_path, b"generation".as_slice()),
        (&payload_path, b"payload".as_slice()),
        (&commit_path, b"commit".as_slice()),
    ] {
        assert!(matches!(
            transport
                .put_file(&handle, path, bytes, write)
                .await
                .unwrap()
                .http_status,
            200 | 201 | 204
        ));
        assert_eq!(
            transport.get_file(&handle, path, read).await.unwrap().body,
            bytes
        );
    }
    let result = transport.list_children(&handle, list).await.unwrap();
    assert_eq!(result.entries.len(), 3);
    assert!(result.entries.iter().all(|e| !e.is_collection));
    let too_small = ReadLimits {
        max_bytes: 2,
        timeout_ms: 3000,
    };
    assert!(matches!(
        transport
            .get_file(&handle, &payload_path, too_small)
            .await
            .err()
            .unwrap()
            .kind,
        TransportKind::TooLarge
    ));
    let missing = SyncRelativePath::generation(&uuid::Uuid::new_v4().to_string()).unwrap();
    assert!(matches!(
        transport
            .get_file(&handle, &missing, read)
            .await
            .err()
            .unwrap()
            .kind,
        TransportKind::NotFound
    ));
    let other = ConnectionHandle {
        id: "bad-password".into(),
        ..handle.clone()
    };
    transport
        .bind(
            other.clone(),
            &url,
            "v2-owned",
            "fixture".into(),
            "wrong".into(),
        )
        .unwrap();
    assert!(matches!(
        transport
            .list_children(&other, list)
            .await
            .err()
            .unwrap()
            .kind,
        TransportKind::Unauthorized
    ));
}
