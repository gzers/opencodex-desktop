//! Opt-in platform credential-store evidence, using only owned random fixture
//! accounts and the explicit Sandbox service. No runtime/account/network calls.
#![cfg(any(target_os = "macos", target_os = "windows"))]
use opencodex_desktop_lib::{
    infrastructure::{keychain, runtime_executable::FixedRuntimeExecutable},
    modules::{
        model_config::{provider_auth, provider_fields, provider_import, schema::*},
        test_sandbox,
    },
    state::ProcessContext,
};
use serde_json::json;
use std::path::{Path, PathBuf};
struct Cleanup {
    root: PathBuf,
    accounts: Vec<(String, String)>,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let directory = self.root.join("manager-state/provider-auth-backups");
        if let Ok(files) = std::fs::read_dir(directory) {
            for entry in files.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("ocxdconf") {
                    continue;
                }
                let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if uuid::Uuid::parse_str(id).is_ok() {
                    self.accounts.push((
                        keychain::KEYCHAIN_SANDBOX_SERVICE_NAME.into(),
                        format!("model-provider-protection:{id}"),
                    ));
                }
            }
        }
        self.accounts.sort();
        self.accounts.dedup();
        for (service, account) in &self.accounts {
            let entry = keyring::Entry::new(service, account).expect("owned cleanup entry");
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(_) => panic!("owned test credential cleanup failed"),
            }
            assert!(
                matches!(entry.get_password(), Err(keyring::Error::NoEntry)),
                "owned credential must be absent after cleanup"
            );
        }
    }
}
fn hash_id(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))[..8].into()
}
fn install(cleanup: &mut Cleanup, service: &str, account: &str, value: &str) {
    let entry = keyring::Entry::new(service, account).unwrap();
    assert!(
        matches!(entry.get_password(), Err(keyring::Error::NoEntry)),
        "random fixture entry unexpectedly exists; never overwrite"
    );
    cleanup.accounts.push((service.into(), account.into()));
    entry.set_password(value).unwrap();
    assert_eq!(entry.get_password().unwrap(), value);
}
fn context(root: &Path) -> ProcessContext {
    ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(root.join("unused-official")),
        working_directory: root.into(),
        opencodex_home: root.into(),
    }
}
#[test]
#[ignore = "requires explicit OPENCODEX_SANDBOX=1; writes/deletes only random owned native credential fixtures"]
fn native_protection_reference_resolution_rebind_and_missing_key_refusal() {
    assert!(test_sandbox::enabled(), "explicit Sandbox mode required");
    assert_eq!(
        keychain::keychain_service_name(),
        keychain::KEYCHAIN_SANDBOX_SERVICE_NAME
    );
    let root = tempfile::tempdir().unwrap();
    let mut cleanup = Cleanup {
        root: root.path().into(),
        accounts: vec![],
    };
    let provider = format!("ocxd-native-fixture-{}", uuid::Uuid::new_v4().simple());
    let first = format!("fixture-secret-{}", uuid::Uuid::new_v4());
    let second = format!("fixture-secret-{}", uuid::Uuid::new_v4());
    let first_id = hash_id(&first);
    let second_id = hash_id(&second);
    let first_account = format!("{provider}/{first_id}");
    let second_account = format!("{provider}/{second_id}");
    install(
        &mut cleanup,
        "opencodex.provider-api-key.v1",
        &first_account,
        &first,
    );
    install(
        &mut cleanup,
        "opencodex.provider-api-key.v1",
        &second_account,
        &second,
    );
    let raw = json!({"adapter":"openai-chat","authMode":"key","baseUrl":"https://native-fixture.example.test/v1","models":["a"],"apiKey":format!("keychain:{second_account}"),"apiKeyPool":[{"id":first_id,"key":format!("keychain:{first_account}"),"label":"first","addedAt":100},{"id":second_id,"key":format!("keychain:{second_account}"),"label":"second","addedAt":200}],"headers":{"X-Fixture":"fixture-header"}});
    std::fs::write(
        root.path().join("config.json"),
        serde_json::to_vec(&json!({"providers":{&provider:&raw}})).unwrap(),
    )
    .unwrap();
    let public = provider_fields::public_from_raw(&raw).unwrap();
    let portable = provider_auth::export_portable(root.path(), &provider, &public).unwrap();
    let serialized = zeroize::Zeroizing::new(serde_json::to_string(&portable).unwrap());
    assert!(serialized.contains(&first) && serialized.contains(&second));
    assert!(!serialized.contains("keychain:"));
    let channel = ChannelPolicy {
        naming: Default::default(),
        id: uuid::Uuid::new_v4().to_string(),
        label: "native fixture".into(),
        template_refs: vec![],
        model_refs: Default::default(),
        connection: ChannelConnection {
            provider_id: format!("ocxd-target-{}", uuid::Uuid::new_v4().simple()),
            protocol: "openai-chat".into(),
            nodes: vec![ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "owned".into(),
                url: "https://native-fixture.example.test/v1".into(),
            }],
            ..Default::default()
        },
        selection: ModelSelection::None,
        model_order: vec![],
        discovered: vec![],
        defaults: Default::default(),
        overrides: Default::default(),
        family_overrides: Default::default(),
    };
    let reference =
        provider_import::stage(root.path(), &channel, &public, Some(&portable)).unwrap();
    let staged = provider_import::load(root.path(), &channel, &reference).unwrap();
    let first_backup = staged.authorization.as_ref().unwrap();
    provider_auth::check_backup(root.path(), first_backup).unwrap();
    let bound = provider_import::authorization(root.path(), &context(root.path()), &staged)
        .unwrap()
        .unwrap();
    provider_auth::check_backup(root.path(), &bound).unwrap();
    for backup in [first_backup, &bound] {
        let ciphertext = std::fs::read(
            root.path()
                .join("manager-state/provider-auth-backups")
                .join(format!("{}.ocxdconf", backup.id)),
        )
        .unwrap();
        assert!(!String::from_utf8_lossy(&ciphertext).contains(&first));
        assert!(!String::from_utf8_lossy(&ciphertext).contains(&second));
    }
    let account = format!("model-provider-protection:{}", bound.id);
    keyring::Entry::new(keychain::KEYCHAIN_SANDBOX_SERVICE_NAME, &account)
        .unwrap()
        .delete_credential()
        .unwrap();
    assert_eq!(
        provider_auth::check_backup(root.path(), &bound)
            .unwrap_err()
            .code,
        "provider_auth_keychain"
    );
    drop(cleanup);
    println!("native-platform={} sandbox-only=true original-reference-resolved=true protected-stage-and-runtime-rebind=true missing-key-refused=true owned-entries-cleaned=true network=false",std::env::consts::OS);
}
