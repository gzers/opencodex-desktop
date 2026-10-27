//! Explicitly opted-in verification. Live source config is read-only; all writes
//! and backups use a separate temporary data root. No key/response is printed.
use opencodex_desktop_lib::modules::model_config::{config_file, measurements, ServiceNode};
use serde_json::{json, Value};
use std::path::PathBuf;
fn official_home() -> PathBuf {
    assert_eq!(
        std::env::var("OCX_LIVE_MODEL_VERIFY").as_deref(),
        Ok("1"),
        "explicit live opt-in required"
    );
    PathBuf::from(std::env::var_os("OCX_TEST_OFFICIAL_HOME").expect("explicit test home"))
}

#[test]
#[ignore = "requires explicit live source; writes only an isolated temporary clone"]
fn installed_configuration_roundtrips_as_an_isolated_file() {
    let source = official_home();
    let original = std::fs::read(source.join("config.json")).unwrap();
    let t = tempfile::tempdir().unwrap();
    let home = t.path().join("official");
    let root = t.path().join("manager");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(home.join("config.json"), &original).unwrap();
    let mut current = config_file::snapshot(&root, &home).unwrap();
    let preview = opencodex_desktop_lib::modules::model_config::compiler::project(
        &current.library,
        &current.policy,
    )
    .expect("the actual file must also open in save preview");
    assert!(preview.notices.iter().all(|n| !n.contains("待核对")));
    let target = current
        .policy
        .channels
        .iter_mut()
        .find(|c| c.connection.provider_id == "GZGG-NewApi")
        .expect("target fixture channel");
    target.connection.enabled = !target.connection.enabled;
    let target_id = target.id.clone();
    let provider = target.connection.provider_id.clone();
    let expected = target.connection.enabled;
    let result = config_file::save(
        &root,
        &home,
        config_file::SaveRequest {
            expected_file: current.policy_digest,
            policy: current.policy,
            credentials: vec![config_file::CredentialEdit {
                channel_id: target_id,
                action: "replace".into(),
                value: Some("fixture-isolated-replacement".into()),
            }],
        },
    )
    .unwrap();
    assert!(result.changed);
    let reopened = config_file::snapshot(&root, &home).unwrap();
    assert_eq!(
        reopened
            .policy
            .channels
            .iter()
            .find(|c| c.connection.provider_id == provider)
            .unwrap()
            .connection
            .enabled,
        expected
    );
    assert_eq!(
        config_file::credential(&home, &provider).unwrap().value,
        "fixture-isolated-replacement"
    );
    let after: Value =
        serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();
    let before: Value = serde_json::from_slice(&original).unwrap();
    for (name, row) in before["providers"].as_object().unwrap() {
        if name != &provider {
            assert!(
                after["providers"].get(name) == Some(row),
                "an unrelated provider changed"
            );
        }
    }
    assert!(
        std::fs::read(source.join("config.json")).unwrap() == original,
        "live file must stay unchanged"
    );
    let official_validated = if let Some(entry) = std::env::var_os("OCX_TEST_OFFICIAL_ENTRY") {
        let node = std::env::var_os("OCX_TEST_OFFICIAL_NODE").expect("explicit official Node");
        let result = std::process::Command::new(node)
            .arg(entry)
            .args(["config", "validate"])
            .env("OPENCODEX_HOME", &home)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "isolated saved file failed official config validation"
        );
        true
    } else {
        false
    };
    println!(
        "LIVE_FILE_ROUNDTRIP {}",
        json!({"providers":reopened.policy.channels.len(),"isolatedWrite":true,"keyOnlySupported":true,"liveSourceUnchanged":true,"officialValidated":official_validated})
    );
}

#[tokio::test]
#[ignore = "requires explicit live opt-in; only model-list GET requests"]
async fn real_model_measurements_use_the_backend_without_altering_config() {
    let home = official_home();
    let original = std::fs::read(home.join("config.json")).unwrap();
    let config: Value = serde_json::from_slice(&original).unwrap();
    for name in ["GZGG-NewApi", "FeiMao"] {
        let row = &config["providers"][name];
        assert!(row.is_object(), "explicit verification channel absent");
        let result = measurements::measure(measurements::Request {
            run_id: uuid::Uuid::new_v4().to_string(),
            protocol: row["adapter"].as_str().unwrap().into(),
            nodes: vec![ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "验证节点".into(),
                url: row["baseUrl"].as_str().unwrap().into(),
            }],
            api_key: None,
            credential_ref: None,
            allow_private_network: true,
            provider_id: Some(name.into()),
            home: Some(home.clone()),
        })
        .await
        .unwrap();
        let measured = &result.results[0];
        println!(
            "LIVE_NODE_MEASUREMENT {}",
            json!({"provider":name,"elapsedMs":measured.elapsed_ms,"modelCount":measured.model_count,"error":measured.error})
        );
        assert!(
            measured.error.is_none(),
            "live model-list request failed; see non-secret error code above"
        );
        assert!(measured.model_count.is_some());
        assert!(measured.elapsed_ms.is_some());
    }
    assert!(
        std::fs::read(home.join("config.json")).unwrap() == original,
        "live source changed during the read-only test"
    );
}
