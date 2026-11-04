//! Opt-in native credential-store validation for the actual channel save owner.
//! Random owned accounts only; no daily configuration, runtime or network calls.
#![cfg(any(target_os = "macos", target_os = "windows"))]
use opencodex_desktop_lib::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::{model_config::config_file, test_sandbox},
    state::ProcessContext,
};
use serde_json::json;
use sha2::{Digest, Sha256};

struct Cleanup(Vec<String>);
impl Drop for Cleanup {
    fn drop(&mut self) {
        for account in &self.0 {
            let entry = keyring::Entry::new("opencodex.provider-api-key.v1", account).unwrap();
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(_) => panic!("owned rename credential cleanup failed"),
            }
            assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
        }
    }
}
#[test]
#[ignore = "requires explicit OPENCODEX_SANDBOX=1; uses only random owned native credential accounts"]
fn actual_channel_rename_rebinds_native_key_pool_and_preserves_current_identity() {
    assert!(test_sandbox::enabled(), "explicit Sandbox mode required");
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("manager");
    let home = temp.path().join("official");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    let old = format!("ocxd-rename-old-{}", uuid::Uuid::new_v4().simple());
    let new = format!("ocxd-rename-new-{}", uuid::Uuid::new_v4().simple());
    let values = [
        format!("owned-a-{}", uuid::Uuid::new_v4()),
        format!("owned-b-{}", uuid::Uuid::new_v4()),
    ];
    let ids: Vec<String> = values
        .iter()
        .map(|s| format!("{:x}", Sha256::digest(s.as_bytes()))[..8].into())
        .collect();
    let mut cleanup = Cleanup(vec![]);
    for provider in [&old, &new] {
        for id in &ids {
            let account = format!("{provider}/{id}");
            let entry = keyring::Entry::new("opencodex.provider-api-key.v1", &account).unwrap();
            assert!(
                matches!(entry.get_password(), Err(keyring::Error::NoEntry)),
                "random test account occupied; never overwrite"
            );
            cleanup.0.push(account);
        }
    }
    for (id, value) in ids.iter().zip(&values) {
        let entry =
            keyring::Entry::new("opencodex.provider-api-key.v1", &format!("{old}/{id}")).unwrap();
        entry.set_password(value).unwrap();
        assert_eq!(&entry.get_password().unwrap(), value);
    }
    let raw = json!({"providers":{&old:{"adapter":"openai-chat","authMode":"key","baseUrl":"https://rename-fixture.invalid/v1","models":["vendor/a","model-b"],"selectedModels":["vendor/a"],"apiKey":format!("keychain:{old}/{}",ids[1]),"apiKeyPool":[{"id":ids[0],"key":format!("keychain:{old}/{}",ids[0]),"label":"first","addedAt":100},{"id":ids[1],"key":format!("keychain:{old}/{}",ids[1]),"label":"active","addedAt":200}],"futureField":{"keep":true}}},"defaultProvider":old,"modelPickerOrder":[format!("{old}/vendor/a")],"disabledModels":[format!("{old}/model-b")],"futureRoot":7});
    std::fs::write(home.join("config.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
    let before = config_file::snapshot(&root, &home).unwrap();
    let mut policy = before.policy.clone();
    let channel = policy
        .channels
        .iter_mut()
        .find(|c| c.connection.provider_id == old)
        .unwrap();
    let stable_id = channel.id.clone();
    channel.label = new.clone();
    channel.connection.provider_id = new.clone();
    let context = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(temp.path().join("unused-official")),
        working_directory: temp.path().into(),
        opencodex_home: home.clone(),
    };
    let saved = config_file::save_with_context(
        &root,
        &context,
        config_file::SaveRequest {
            expected_file: before.policy_digest,
            policy,
            credentials: vec![],
            scope: Some(config_file::SaveScope {
                channel_id: stable_id.clone(),
            }),
        },
    )
    .unwrap();
    assert!(saved.changed);
    let readback = config_file::snapshot(&root, &home).unwrap();
    let channel = readback
        .policy
        .channels
        .iter()
        .find(|c| c.id == stable_id)
        .unwrap();
    assert_eq!(channel.connection.provider_id, new);
    assert_eq!(channel.label, new);
    let actual: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();
    assert!(actual["providers"].get(&old).is_none());
    assert_eq!(actual["defaultProvider"], new);
    assert_eq!(
        actual["modelPickerOrder"],
        json!([format!("{new}/vendor/a")])
    );
    assert_eq!(actual["disabledModels"], json!([format!("{new}/model-b")]));
    assert_eq!(
        actual["providers"][&new]["futureField"],
        json!({"keep":true})
    );
    assert_eq!(actual["futureRoot"], 7);
    for (index, (id, value)) in ids.iter().zip(&values).enumerate() {
        assert!(matches!(
            keyring::Entry::new("opencodex.provider-api-key.v1", &format!("{old}/{id}"))
                .unwrap()
                .get_password(),
            Err(keyring::Error::NoEntry)
        ));
        assert_eq!(
            &keyring::Entry::new("opencodex.provider-api-key.v1", &format!("{new}/{id}"))
                .unwrap()
                .get_password()
                .unwrap(),
            value
        );
        assert_eq!(actual["providers"][&new]["apiKeyPool"][index]["id"], *id);
    }
    assert_eq!(
        actual["providers"][&new]["apiKey"],
        format!("keychain:{new}/{}", ids[1])
    );
}
