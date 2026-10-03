//! 双端同步的真实链路（需要一台真实 TLS WebDAV 服务器）。
//!
//! 与 `sync_plaintext.rs`（内存里搬运字节）不同，这里把 `modules::sync::runner`
//! 完整跑起来：真实配置、真实 `WebDavClient::production()`、真实服务器上的
//! latest 指针 / 清单 / 载荷，验证「一端上传，另一端下载并应用」「重复同步不覆盖」
//! 以及失败映射。仍不触碰 Keychain——口令由测试直接注入。
//!
//! 未设置 `OCX_TEST_WEBDAV_URL` 时用例打印跳过原因并返回（视为未验证）。
//! `OCX_TEST_WEBDAV_DIR` 指向该服务器的本地根目录，用于在每个用例前清空远端，
//! 使用例彼此独立、与执行顺序无关；缺失时同样跳过。
//!
//! 运行方式（共享一台远端，必须串行）：
//! `SSL_CERT_FILE=<ca> OCX_TEST_WEBDAV_URL=<url> OCX_TEST_WEBDAV_DIR=<dir> \
//!  cargo test --test webdav_sync_real -- --test-threads=1`

use opencodex_desktop_lib::infrastructure::webdav_client::manifest_remote_path;
use opencodex_desktop_lib::infrastructure::webdav_client::WebDavConfig;
use opencodex_desktop_lib::modules::preferences::PreferencesStore;
use opencodex_desktop_lib::modules::sync::config::{
    CredentialRef, SyncEndpointConfig, TLS_POLICY_VERIFY_REQUIRED,
};
use opencodex_desktop_lib::modules::sync::engine::{build_local_snapshot, seal_manifest};
use opencodex_desktop_lib::modules::sync::runner::{self, SyncRunFailure};

fn endpoint_config() -> Option<SyncEndpointConfig> {
    let base_url = std::env::var("OCX_TEST_WEBDAV_URL").ok()?;
    Some(SyncEndpointConfig {
        endpoint_id: "default".to_string(),
        url: base_url,
        remote_path: std::env::var("OCX_TEST_WEBDAV_PATH").unwrap_or_else(|_| "ocx-test".into()),
        username: std::env::var("OCX_TEST_WEBDAV_USER").unwrap_or_else(|_| "davuser".into()),
        credential_ref: CredentialRef {
            ref_id: "cred_12345678-1234-1234-1234-123456789abc".to_string(),
            backend: "keychain".to_string(),
            service_name: "OpenCodex Desktop".to_string(),
            account_key: "ocx.dav.cred_12345678-1234-1234-1234-123456789abc".to_string(),
            purpose: "webdav_credential".to_string(),
            created_at: "2026-09-20T00:00:00Z".to_string(),
            updated_at: "2026-09-20T00:00:00Z".to_string(),
        },
        tls_policy: TLS_POLICY_VERIFY_REQUIRED.to_string(),
        legacy_encryption: String::new(),
        conflict_policy: "ask".to_string(),
    })
}

fn webdav(endpoint: &SyncEndpointConfig) -> WebDavConfig {
    WebDavConfig {
        base_url: endpoint.url.clone(),
        remote_path: endpoint.remote_path.clone(),
        username: endpoint.username.clone(),
        password: std::env::var("OCX_TEST_WEBDAV_PASS").unwrap_or_else(|_| "dav-pass-123".into()),
    }
}

/// 建立隔离数据根并写入给定 `interface_scale`（同步会传输偏好文件）。
fn isolated_root(name: &str, interface_scale: i32) -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect(name);
    opencodex_desktop_lib::modules::data_root::initialize(temp.path()).expect("init root");
    let store = PreferencesStore::new(temp.path());
    let mut preferences = store.load().expect("load defaults");
    preferences.interface_scale = interface_scale;
    // 冷同步会直接跳过同步，测试必须显式关闭。
    preferences.cold_sync = false;
    store.save(&preferences).expect("save preferences");
    temp
}

fn scale_of(root: &std::path::Path) -> i32 {
    PreferencesStore::new(root)
        .load()
        .expect("load preferences")
        .interface_scale
}

fn skip(name: &str) -> bool {
    eprintln!("[webdav_sync_real] 跳过 {name}：未设置 OCX_TEST_WEBDAV_URL / OCX_TEST_WEBDAV_DIR");
    true
}

/// 清空远端目录，保证用例之间互不干扰。
fn reset_remote(remote_path: &str) -> Option<std::path::PathBuf> {
    let root = std::env::var("OCX_TEST_WEBDAV_DIR").ok()?;
    let dir = std::path::Path::new(&root).join(remote_path.trim_matches('/'));
    std::fs::create_dir_all(&dir).expect("create remote dir");
    for entry in std::fs::read_dir(&dir).expect("read remote dir") {
        let entry = entry.expect("entry");
        if entry.path().is_file() {
            std::fs::remove_file(entry.path()).expect("clean remote file");
        }
    }
    Some(dir)
}

/// 直接把内容写到远端（用于故障注入：只写清单、不写载荷）。
async fn put_raw(env: &WebDavConfig, name: &str, body: Vec<u8>) {
    let url = format!(
        "{}/{}/{}",
        env.base_url.trim_end_matches('/'),
        env.remote_path.trim_matches('/'),
        name
    );
    let response = reqwest::Client::new()
        .put(&url)
        .basic_auth(&env.username, Some(&env.password))
        .body(body)
        .send()
        .await
        .expect("PUT to real server");
    assert!(
        response.status().is_success(),
        "PUT {name} -> {}",
        response.status()
    );
}

/// 一端上传，另一端下载并应用，然后重复同步不得覆盖本地。
#[tokio::test]
async fn two_isolated_roots_sync_through_a_real_server() {
    let Some(endpoint) = endpoint_config() else {
        skip("two_isolated_roots_sync_through_a_real_server");
        return;
    };
    let Some(_remote) = reset_remote(&endpoint.remote_path) else {
        skip("two_isolated_roots_sync_through_a_real_server");
        return;
    };
    let config = webdav(&endpoint);
    let source = isolated_root("sync-a", 137);
    let target = isolated_root("sync-b", 65);

    // A 上传自己的偏好。
    let uploaded = runner::run_sync(source.path(), source.path(), &endpoint, &config)
        .await
        .expect("上传端同步应成功");
    assert!(uploaded.applied.is_empty(), "首次同步没有远端内容可应用");
    // 测试数据根只有偏好文件；缺失的扩展配置不得被当成空载荷上传。
    assert_eq!(uploaded.uploaded.len(), 1);
    assert!(uploaded
        .uploaded
        .iter()
        .any(|name| name == "manager-state/preferences.json"));

    // B 下载并应用 A 的快照：本机 42 必须变成 137。
    let applied = runner::run_sync(target.path(), target.path(), &endpoint, &config)
        .await
        .expect("下载端同步应成功");
    assert!(
        applied
            .applied
            .iter()
            .any(|name| name == "manager-state/preferences.json"),
        "应如实报告应用了远端偏好，实际 {:?}",
        applied.applied
    );
    assert_eq!(scale_of(target.path()), 137, "远端偏好必须真实落盘");

    // 覆盖前必须先备份：B 原本的 65 必须能在 backups/ 下找到记录。
    let backups_root = target.path().join("backups");
    let records = opencodex_desktop_lib::modules::backup::list_action_records(
        &backups_root,
        "sync-overwrite",
    )
    .expect("list sync-overwrite backups");
    assert!(
        !records.is_empty(),
        "覆盖远端内容前必须留下 sync-overwrite 备份"
    );
    assert!(records
        .iter()
        .any(|record| record.manifest.target_path.ends_with("preferences.json")));

    // B 再同步一次：远端 latest 就是 B 刚上传的快照，属回放，不得覆盖本地。
    let replay = runner::run_sync(target.path(), target.path(), &endpoint, &config)
        .await
        .expect("重复同步应成功返回");
    assert!(
        replay.applied.is_empty(),
        "回放冲突不得声称已应用远端，实际 {:?}",
        replay.applied
    );
    assert!(
        replay.conflicted,
        "回放必须标记为冲突，否则「同步冲突提醒」无从触发"
    );
    assert_eq!(scale_of(target.path()), 137, "回放不得改动本地");
}

/// 错误口令必须映射为「认证失败」，本地内容不变。
#[tokio::test]
async fn wrong_password_over_real_server_is_unauthorized() {
    let Some(endpoint) = endpoint_config() else {
        skip("wrong_password_over_real_server_is_unauthorized");
        return;
    };
    let mut config = webdav(&endpoint);
    config.password = "definitely-wrong".to_string();
    let root = isolated_root("sync-auth", 88);
    let failure = runner::run_sync(root.path(), root.path(), &endpoint, &config)
        .await
        .expect_err("错误口令不得成功");
    assert!(
        matches!(
            failure,
            SyncRunFailure::WebDav(
                opencodex_desktop_lib::infrastructure::webdav_client::WebDavOperation::Download,
                opencodex_desktop_lib::infrastructure::webdav_client::WebDavError::Unauthorized
            )
        ),
        "预期认证失败，实际 {failure:?}"
    );
    assert_eq!(scale_of(root.path()), 88, "失败不得改动本地");
}

/// 故障注入：远端指针指向一个「有清单、无载荷」的快照时，
/// 下载端必须如实失败（NotFound），并且绝不改动本地内容。
#[tokio::test]
async fn missing_payload_on_a_valid_manifest_fails_without_touching_local() {
    let Some(endpoint) = endpoint_config() else {
        skip("missing_payload_on_a_valid_manifest_fails_without_touching_local");
        return;
    };
    let Some(_remote) = reset_remote(&endpoint.remote_path) else {
        skip("missing_payload_on_a_valid_manifest_fails_without_touching_local");
        return;
    };
    let config = webdav(&endpoint);
    let now = chrono::Utc::now();
    let (manifest, payloads) = build_local_snapshot(
        vec![(
            "manager-state/preferences.json".to_string(),
            br#"{"interface_scale":150}"#.to_vec(),
        )],
        "Injected Device",
        now,
    )
    .expect("build manifest");
    let sealed = seal_manifest(&manifest).expect("seal manifest");
    // 只写清单与指针，故意不写载荷。
    put_raw(
        &config,
        &manifest_remote_path(&manifest.snapshot_id),
        serde_json::to_vec(&sealed).expect("manifest json"),
    )
    .await;
    put_raw(
        &config,
        "latest.txt",
        format!("{}\n", manifest.snapshot_id).into_bytes(),
    )
    .await;
    let _ = payloads;

    let target = isolated_root("sync-missing-payload", 71);
    let failure = runner::run_sync(target.path(), target.path(), &endpoint, &config)
        .await
        .expect_err("载荷缺失不得伪装成功");
    assert!(
        matches!(
            failure,
            SyncRunFailure::WebDav(
                _,
                opencodex_desktop_lib::infrastructure::webdav_client::WebDavError::NotFound
            )
        ),
        "预期载荷缺失被如实报告，实际 {failure:?}"
    );
    assert_eq!(scale_of(target.path()), 71, "失败不得改动本地");
    assert!(
        !target
            .path()
            .join("manager-state/preferences.json")
            .exists()
            || scale_of(target.path()) != 150,
        "远端未成功应用，本地不得变成远端值"
    );
}

/// 同一 endpoint 的并发同步必须被互斥挡住（FZ-32）。
#[tokio::test]
async fn concurrent_sync_is_blocked_by_endpoint_lock() {
    let Some(endpoint) = endpoint_config() else {
        skip("concurrent_sync_is_blocked_by_endpoint_lock");
        return;
    };
    let config = webdav(&endpoint);
    let left = isolated_root("sync-concurrent-a", 120);
    let right = isolated_root("sync-concurrent-b", 121);
    let left_fut = runner::run_sync(left.path(), left.path(), &endpoint, &config);
    let right_fut = runner::run_sync(right.path(), right.path(), &endpoint, &config);
    let (left_result, right_result) = tokio::join!(left_fut, right_fut);
    let locked = [&left_result, &right_result]
        .iter()
        .filter(|result| matches!(result, Err(SyncRunFailure::TargetLocked)))
        .count();
    assert_eq!(
        locked, 1,
        "并发同步必须恰好有一个被互斥挡住，实际 {left_result:?} / {right_result:?}"
    );
}
