use opencodex_desktop_lib::modules::model_config::{
    recovery::Baseline, runtime, schema::*, store::{ModelStore, sha256},
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from("/private/tmp/ocx020-history-native-34a/data");
    assert!(root.is_dir());
    let actual_config = root.join("opencodex-home/config.json");
    let original = fs::read(&actual_config).unwrap();
    let store = ModelStore::new(&root);
    let mut policy = Policy::default();
    let node = ServiceNode { id: uuid::Uuid::new_v4().to_string(), name:"owned history fixture node".into(), url:"https://fixture.example.test/v1".into() };
    let channel = ChannelPolicy {
        naming: Default::default(), id:uuid::Uuid::new_v4().to_string(), label:"历史草稿原生校验".into(),
        template_refs:vec![],model_refs:BTreeMap::new(),
        connection:ChannelConnection { auth_mode:None,source_read_id:None,imported_config:None,
            provider_id:"owned_history_fixture".into(),protocol:"openai-chat".into(),
            active_node_id:node.id.clone(),nodes:vec![node],credential_ref:None,enabled:false,allow_private_network:false },
        selection:ModelSelection::All,model_order:vec![],discovered:vec![],defaults:Default::default(),overrides:Default::default(),family_overrides:Default::default(),
    };
    policy.channels.push(channel);
    policy.channels[0].defaults.context_window = Field::Custom(128000);
    let local=store.load().unwrap();
    let before=store.commit_runtime_policy(&local.library_digest,&local.policy_digest,&local.workspace_digest,policy.clone()).unwrap();
    let recovery=Baseline::capture(&before).unwrap();
    policy.channels[0].defaults.context_window=Field::Custom(64000);
    let current=store.commit_runtime_policy(&before.library_digest,&before.policy_digest,&before.workspace_digest,policy.clone()).unwrap();
    let id=uuid::Uuid::new_v4().to_string();
    let plan=json!({
        "id":id,"phase":"committed","expectedLibrary":current.library_digest,"expectedPolicy":current.policy_digest,"expectedWorkspace":current.workspace_digest,
        "policy":policy,"before":{"version":"owned-legacy-history-fixture","providers":{"providers":{}},"models":[],"order":{"pickerOrder":[]},"catalog":{"models":[]}},
        "nextProviders":{"providers":{}},"nextOrder":[],"expectedSlugs":[],"runtimeExecutable":"never-executed-owned-fixture",
        "error":null,"createdAt":"2026-11-03T18:00:00Z","ownership":[],"previousOwnership":[],"disabledBefore":[],"observedProviders":{"providers":{}},"observedOrder":[],"observedDisabled":[],
        "recovery":recovery
    });
    let directory=root.join("manager-state/model-save-plans");fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join(format!("{id}.json")),serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
    let history=runtime::history(&root).unwrap();assert_eq!(history.len(),1);assert!(history[0].recoverable);
    assert_eq!(fs::read(&actual_config).unwrap(),original);
    fs::write(root.parent().unwrap().join("seed-receipt.json"),serde_json::to_vec_pretty(&json!({
        "result":"pass","sourceId":id,"businessConfigSha256":sha256(&original),"canonicalBeforeContextWindow":128000,"canonicalCurrentContextWindow":64000,
        "method":"Actual ModelStore canonical local commits plus explicitly seeded validated legacy successful-plan catalog. No runtime CLI, provider/key changes or generation. Native consumer will prepare/accept via real IPC.",
        "businessConfigUnchanged":true,"legacyHistoryCatalogSeeded":true,"actualRuntimeSaveClaimed":false
    })).unwrap()).unwrap();
    println!("owned history consumer baseline prepared {id}");
}
