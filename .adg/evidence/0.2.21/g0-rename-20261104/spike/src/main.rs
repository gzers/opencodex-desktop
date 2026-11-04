//! G0 executable probe. All reads/writes use explicit tempfile roots. The
//! production crate is an unchanged Cargo path dependency, not a patched copy.
mod memory;
mod transaction;
use memory::Memory;
use transaction::*;
use opencodex_desktop_lib::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::{self as model, config_file::{self, SaveRequest, SaveScope}, schema::*, store::sha256},
    state::ProcessContext,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}, process::Command};

const OLD: &str="Krill-1";
const NEW: &str="Krill";
const UUID_REF: &str="20000000-0000-4000-8000-000000000001";
const GROUP_ID: &str="30000000-0000-4000-8000-000000000001";
const GROUP_PROVIDER: &str="ocxd-30000000000040008000000000000001";
const KEY_A: &str="fixture-first-value";
const KEY_B: &str="fixture-second-value";
fn account_id(key:&str)->String {sha256(key.as_bytes())[7..15].into()}
fn fixture_root()->PathBuf {PathBuf::from(std::env::var_os("G0_RUNTIME_ROOT").expect("explicit temporary runtime root"))}

struct Fixture { _temp:tempfile::TempDir, root:PathBuf, home:PathBuf, id:String, original:Value, layout:Value }
impl Fixture {
    fn new(memory:&Memory)->Self {
        memory.reset();
        let temp=tempfile::Builder::new().prefix("case-").tempdir_in(fixture_root()).unwrap();
        let root=temp.path().join("manager"); let home=temp.path().join("official");
        fs::create_dir_all(&root).unwrap(); fs::create_dir_all(&home).unwrap();
        let a=account_id(KEY_A);let b=account_id(KEY_B);
        for (account,value) in [(format!("{OLD}/{a}"),KEY_A),(format!("{OLD}/{b}"),KEY_B),(format!("{OLD}/header"),"fixture-header".into()),(format!("{OLD}/nested"),"fixture-nested".into())] {put(&account,value).unwrap();}
        keyring::Entry::new("OpenCodex Desktop Sandbox",&format!("model_channel:{UUID_REF}")).unwrap().set_password("fixture-uuid-owned").unwrap();
        let original=json!({
            "providers":{
                OLD:{"name":OLD,"adapter":"openai-responses","authMode":"key","baseUrl":"https://fixture.invalid/v1",
                    "apiKey":format!("keychain:{OLD}/{b}"),
                    "apiKeyPool":[{"id":a,"key":format!("keychain:{OLD}/{a}"),"label":"first","addedAt":100,"futureSlot":7},
                                  {"id":b,"key":format!("keychain:{OLD}/{b}"),"label":"second","addedAt":200}],
                    "headers":{"X-Fixture":format!("keychain:{OLD}/header"),"X-Env":"$G0_FIXTURE_HEADER"},
                    "desktopExecutor":{"env":{"token":format!("keychain:{OLD}/nested")}},
                    "models":["vendor/a:8b","vendor-a:8b","Mixed_Case/V2"],
                    "selectedModels":["vendor/a:8b","vendor-a:8b","Mixed_Case/V2"],
                    "modelAliases":{"vendor/a:8b":"exact-alias"},
                    "modelContextWindows":{"vendor/a:8b":128000},
                    "futureProvider":{"literal":"Krill-1 text stays","keep":42}},
                "Krill-10":{"adapter":"openai-chat","authMode":"key","baseUrl":"https://neighbor.invalid/v1","apiKey":"fixture-neighbor","models":["untouched"]},
                "openai":{"adapter":"openai-responses","authMode":"oauth","baseUrl":"https://official.invalid/v1","models":["native"]},
                GROUP_PROVIDER:{"adapter":"anthropic","authMode":"key","baseUrl":"https://fixture.invalid/v1","models":["group-model"],"note":format!("OpenCodeX channel={{channel}} protocol=anthropic")}
            },
            "defaultProvider":OLD,
            "modelPickerOrder":["native-head",format!("{OLD}/vendor/a:8b"),"Krill-10/untouched","opaque-middle",format!("{OLD}/vendor-a:8b"),format!("{OLD}/opaque/alias"),format!("{GROUP_PROVIDER}/group-model"),"native-tail"],
            "disabledModels":[format!("{OLD}/Mixed_Case/V2"),"Krill-10/not-this",format!("{OLD}/unknown/native/row")],
            "futureRoot":{"literal":"Krill-1/vendor/a:8b","keep":true}
        });
        write(&home.join("config.json"),&original);
        let snapshot=config_file::snapshot(&root,&home).unwrap();
        let mut policy=snapshot.policy;
        let (template,revision)={let t=&snapshot.library.templates[0];(t.id.clone(),t.revisions[0].id.clone())};
        let c=policy.channels.iter_mut().find(|c|c.connection.provider_id==OLD).unwrap();
        let id=c.id.clone();
        c.template_refs=vec![TemplateRef{template_id:template,revision_id:revision}];
        c.connection.credential_ref=Some(UUID_REF.into());
        // Preserve an existing UUID-owned reference in the stored layout. The
        // current file DTO intentionally omits this legacy credential_ref.
        let layout=serde_json::to_value(&policy).unwrap(); write(&root.join(LAYOUT),&layout);
        write(&root.join(PREFS),&json!({"version":1,"modelTemplateAgent":{"providerId":OLD,"modelId":"vendor/a:8b","reasoningEffort":null,"fast":null,"channelId":id,"connectionDigest":"stale-before-rename"},"other":{"keep":99}}));
        write(&root.join(IDS),&json!({OLD:id,"Krill-10":policy.channels.iter().find(|c|c.connection.provider_id=="Krill-10").unwrap().id}));
        write(&root.join(GROUPS),&json!([{"id":GROUP_ID,"channelId":id,"protocol":"anthropic","providerId":GROUP_PROVIDER}]));
        Self{_temp:temp,root,home,id,original,layout}
    }
    fn rename(&self)->Rename {Rename{id:self.id.clone(),label:NEW.into(),target:NEW.into()}}
    fn plan(&self,scope:Option<&str>)->Plan {prepare(&self.root,&self.home,&[self.rename()],scope,&self.context()).unwrap()}
    fn bytes(&self)->BTreeMap<String,Vec<u8>> {
        let mut map=BTreeMap::new();
        for (key,path) in [("config",self.home.join("config.json")),("layout",self.root.join(LAYOUT)),("prefs",self.root.join(PREFS)),("ids",self.root.join(IDS)),("groups",self.root.join(GROUPS))] {map.insert(key.into(),fs::read(path).unwrap());}
        map
    }
    fn snapshot(&self)->Snapshot {config_file::snapshot(&self.root,&self.home).unwrap()}
    fn context(&self)->ProcessContext {ProcessContext{runtime:FixedRuntimeExecutable::resolved(self._temp.path().join("never-executed-ocx")),working_directory:self._temp.path().into(),opencodex_home:self.home.clone()}}
}
fn receipt(id:&str,evidence:Value) {println!("{}",json!({"case":id,"status":"PASS","evidence":evidence}));}
fn old_rules(memory:&Memory) {
    for scoped in [true,false] {
        let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let writes=memory.writes();
        let snapshot=f.snapshot();let mut policy=snapshot.policy;
        let c=policy.channels.iter_mut().find(|c|c.id==f.id).unwrap();c.label=NEW.into();c.connection.provider_id=NEW.into();
        let request=SaveRequest{expected_file:snapshot.policy_digest,policy,credentials:vec![],scope:scoped.then(||SaveScope{channel_id:f.id.clone()})};
        let failure=config_file::save(&f.root,&f.home,request).err().expect("old implementation must reject provider rename");
        assert_eq!(failure.code,"provider_identity");assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);assert_eq!(memory.writes(),writes);
        receipt(if scoped{"OLD-SCOPED"}else{"OLD-MAPPER"},json!({"code":failure.code,"message":failure.message,"five_files_byte_identical":true,"credential_mutations":0}));
    }
    let f=Fixture::new(memory);let snapshot=f.snapshot();let mut policy=snapshot.policy;
    policy.channels.iter_mut().find(|c|c.id==f.id).unwrap().label=NEW.into();
    let result=config_file::save(&f.root,&f.home,SaveRequest{expected_file:snapshot.policy_digest,policy,credentials:vec![],scope:Some(SaveScope{channel_id:f.id.clone()})}).unwrap();
    assert!(!result.changed);assert!(read(&f.home.join("config.json"))["providers"].get(OLD).is_some());
    assert_eq!(result.snapshot.policy.channels.iter().find(|c|c.id==f.id).unwrap().connection.provider_id,OLD);
    receipt("OLD-LABEL-ONLY",json!({"changed":false,"persisted_label":NEW,"runtime_provider_still":OLD}));

    let f=Fixture::new(memory);let mut raw=f.original.clone();let row=raw["providers"].as_object_mut().unwrap().remove(OLD).unwrap();raw["providers"][NEW]=row;write(&f.home.join("config.json"),&raw);
    let id=f.snapshot().policy.channels.iter().find(|c|c.connection.provider_id==NEW).unwrap().id.clone();assert_ne!(id,f.id);
    let failure=config_file::credential(&f.home,NEW).err().expect("stale provider keychain ref must fail before reading");assert_eq!(failure.code,"credential");
    receipt("OLD-RAW-KEY-ONLY",json!({"stable_id_lost":true,"credential_code":failure.code,"default_still":raw["defaultProvider"]}));
    // Exercise provider_auth's independent owner check against the unmodified
    // library. Use a separate known-field fixture so future-field validation
    // cannot hide the stale keychain owner error.
    raw["providers"][NEW].as_object_mut().unwrap().remove("futureProvider");
    for slot in raw["providers"][NEW]["apiKeyPool"].as_array_mut().unwrap(){slot.as_object_mut().unwrap().remove("futureSlot");}
    write(&f.home.join("config.json"),&raw);
    let public=model::provider_fields::public_from_raw(&raw["providers"][NEW]).unwrap();
    let failure=model::provider_auth::export_portable(&f.home,NEW,&public).err().expect("auth exporter must reject old account owner");
    assert_eq!(failure.code,"provider_auth_reference");
    receipt("OLD-AUTH-OWNER",json!({"code":failure.code,"message":failure.message}));
    raw["providers"][NEW].as_object_mut().unwrap().remove("apiKeyPool");write(&f.home.join("config.json"),&raw);
    let failure=model::provider_auth::export_portable(&f.home,NEW,&public).err().expect("independent owner check");assert_eq!(failure.code,"provider_auth_reference");
    receipt("OLD-AUTH-ACCOUNT-OWNER",json!({"code":failure.code,"message":failure.message}));
}
fn upstream(raw:&Value)->model::upstream::UpstreamSnapshot {
    let models=raw["providers"].as_object().unwrap().iter().flat_map(|(provider,p)|p["models"].as_array().unwrap().iter().map(move |id|json!({"id":id,"provider":provider,"namespaced":format!("{provider}/{}",id.as_str().unwrap()),"name":format!("{provider}/{}",id.as_str().unwrap())}))).collect::<Vec<_>>();
    model::upstream::UpstreamSnapshot{version:"isolated-fixture".into(),providers:raw.clone(),models:json!(models),order:json!({}),catalog:json!({})}
}
fn assert_saved(f:&Fixture,memory:&Memory,keys_before:&BTreeMap<(String,String),Vec<u8>>) {
    let raw=read(&f.home.join("config.json"));assert!(raw["providers"].get(OLD).is_none());assert_eq!(raw["defaultProvider"],NEW);
    let mut expected=f.original.clone();let mut source=expected["providers"].as_object_mut().unwrap().remove(OLD).unwrap();
    // Construct the expected result independently using fixture accounts and
    // exact expected arrays, rather than calling the transaction rewriter.
    let a=account_id(KEY_A);let b=account_id(KEY_B);
    source["name"]=json!(NEW);source["apiKey"]=json!(format!("keychain:{NEW}/{b}"));
    source["apiKeyPool"][0]["key"]=json!(format!("keychain:{NEW}/{a}"));source["apiKeyPool"][1]["key"]=json!(format!("keychain:{NEW}/{b}"));
    source["headers"]["X-Fixture"]=json!(format!("keychain:{NEW}/header"));source["desktopExecutor"]["env"]["token"]=json!(format!("keychain:{NEW}/nested"));
    expected["providers"][NEW]=source;expected["defaultProvider"]=json!(NEW);
    expected["modelPickerOrder"]=json!(["native-head","Krill/vendor/a:8b","Krill-10/untouched","opaque-middle","Krill/vendor-a:8b","Krill/opaque/alias",format!("{GROUP_PROVIDER}/group-model"),"native-tail"]);
    expected["disabledModels"]=json!(["Krill/Mixed_Case/V2","Krill-10/not-this","Krill/unknown/native/row"]);
    assert_eq!(raw,expected);
    let snapshot=f.snapshot();let c=snapshot.policy.channels.iter().find(|c|c.id==f.id).unwrap();
    assert_eq!(c.label,NEW);assert_eq!(c.connection.provider_id,NEW);
    let old_c=f.layout["channels"].as_array().unwrap().iter().find(|c|c["id"]==f.id).unwrap();
    assert_eq!(serde_json::to_value(&c.connection.nodes).unwrap(),old_c["connection"]["nodes"]);
    assert_eq!(serde_json::to_value(&c.template_refs).unwrap(),old_c["templateRefs"]);
    let layout=read(&f.root.join(LAYOUT));let next_c=layout["channels"].as_array().unwrap().iter().find(|c|c["id"]==f.id).unwrap();
    assert_eq!(next_c["connection"]["credentialRef"],UUID_REF);
    let view=config_file::credential(&f.home,NEW).unwrap();assert!(view.viewable);assert_eq!(view.value,KEY_B);
    let after=memory.snapshot();assert_eq!(after.len(),keys_before.len());
    for ((service,account),secret) in keys_before {
        let key=if service==SERVICE && account.starts_with("Krill-1/"){(service.clone(),format!("Krill/{}",&account[8..]))}else{(service.clone(),account.clone())};
        assert_eq!(after.get(&key),Some(secret));
    }
    assert!(after.keys().all(|(s,a)|s!=SERVICE || !a.starts_with("Krill-1/")));
    let context=f.context();let up=upstream(&raw);
    let directory=model::execution_directory::bound_directory(&f.root,&context,&up).unwrap();
    let row=directory.rows.iter().find(|r|r.model.provider_id==NEW&&r.model.model_id=="vendor/a:8b").unwrap();
    assert_eq!(row.model.channel_id,f.id);assert_eq!(row.model.label,"Krill/vendor/a:8b");
    let settings=model::agent::settings(&f.root).unwrap().unwrap();assert_eq!(settings.provider_id,NEW);assert_eq!(settings.connection_digest,row.connection_digest);
    let bound=model::execution_directory::bind(&f.root,&context,&up,settings).unwrap();assert!(bound.connection_digest.is_some());assert_eq!(bound.channel_id.as_deref(),Some(f.id.as_str()));
    assert_eq!(read(&f.root.join(PREFS))["other"],json!({"keep":99}));
    assert_eq!(read(&f.root.join(IDS))[NEW],f.id);
    let groups=read(&f.root.join(GROUPS));assert_eq!(groups[0]["id"],GROUP_ID);assert_eq!(groups[0]["channelId"],f.id);assert_eq!(groups[0]["providerId"],GROUP_PROVIDER);
    assert_eq!(model::discovery::credential(UUID_REF).unwrap().as_str(),"fixture-uuid-owned");
}
fn positives(memory:&Memory) {
    for scoped in [true,false] {
        let f=Fixture::new(memory);let keys=memory.snapshot();let plan=f.plan(scoped.then_some(f.id.as_str()));
        assert_eq!(plan.images.len(),4);assert_eq!(plan.moves.len(),4);
        assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::Committed);assert_saved(&f,memory,&keys);
        // A fresh OS process reads only the persisted config/layout through the
        // real snapshot consumer. This is not an official-client restart.
        let child=Command::new(std::env::current_exe().unwrap()).arg("readback").arg(&f.root).arg(&f.home).arg(&f.id).output().unwrap();
        assert!(child.status.success(),"{}",String::from_utf8_lossy(&child.stderr));
        receipt(if scoped{"TX-SCOPED"}else{"TX-ALL-PAGE"},json!({"files_written":4,"keychain_accounts_rebound":4,"stable_channel_node_template_uuid":true,"pool_order_ids_labels_active_key_unchanged":true,"unknown_and_adjacent_prefix_preserved":true,"fresh_process_snapshot":true,"production_directory_label":"Krill/vendor/a:8b","agent_rebind":true,"protocol_group_ids_preserved":true}));
    }
    let f=Fixture::new(memory);let mut ignored=f.rename();ignored.id=f.snapshot().policy.channels.iter().find(|c|c.connection.provider_id=="Krill-10").unwrap().id.clone();ignored.target="invalid/provider".into();ignored.label.clear();
    let plan=prepare(&f.root,&f.home,&[f.rename(),ignored],Some(&f.id),&f.context()).unwrap();assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::Committed);
    assert_eq!(read(&f.home.join("config.json"))["providers"]["Krill-10"],f.original["providers"]["Krill-10"]);
    receipt("TX-SCOPE-IGNORES-OTHER-DRAFT",json!({"invalid_other_channel_draft_ignored":true,"other_provider_byte_semantics_preserved":true}));

    let f=Fixture::new(memory);let neighbor=f.snapshot().policy.channels.iter().find(|c|c.connection.provider_id=="Krill-10").unwrap().id.clone();
    let batch=[f.rename(),Rename{id:neighbor.clone(),label:"Neighbor".into(),target:"Neighbor".into()}];
    let plan=prepare(&f.root,&f.home,&batch,None,&f.context()).unwrap();assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::Committed);
    let saved=f.snapshot();let c=saved.policy.channels.iter().find(|c|c.id==neighbor).unwrap();assert_eq!(c.connection.provider_id,"Neighbor");assert_eq!(c.label,"Neighbor");
    assert_eq!(read(&f.home.join("config.json"))["providers"]["Neighbor"]["apiKey"],"fixture-neighbor");
    receipt("TX-ALL-PAGE-BATCH",json!({"renamed_channels":2,"stable_ids":true,"inline_neighbor_key_unchanged":true}));

    let f=Fixture::new(memory);for p in [PREFS,IDS,GROUPS] {fs::remove_file(f.root.join(p)).unwrap();}
    let plan=f.plan(None);assert_eq!(plan.images.len(),2);assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::Committed);
    for p in [PREFS,IDS,GROUPS] {assert!(!f.root.join(p).exists());}
    receipt("TX-MINIMUM-FILES",json!({"files_written":2,"absent_optional_files_not_created":true}));
}

fn negative_preflight(memory:&Memory) {
    for mode in ["invalid-provider","provider-too-long","empty-label","occupied","reserved-target","oauth-source","forward-source","reserved-source","derived-occupied","foreign-account","missing-account","credential-target-occupied","shared-reference","agent-mismatch"] {
        let f=Fixture::new(memory);let mut rename=f.rename();
        let mut raw=read(&f.home.join("config.json"));
        let expected=match mode {
            "invalid-provider"=>{rename.target="bad/name".into();"provider"},
            "provider-too-long"=>{rename.target="a".repeat(129);"provider"},
            "empty-label"=>{rename.label="".into();"channel"},
            "occupied"=>{rename.target="Krill-10".into();"provider_occupied"},
            "reserved-target"=>{rename.target="openai".into();"protected_provider"},
            "oauth-source"=>{raw["providers"][OLD]["authMode"]=json!("oauth");"protected_provider"},
            "forward-source"=>{raw["providers"][OLD]["authMode"]=json!("forward");"protected_provider"},
            "reserved-source"=>{rename.id=f.snapshot().policy.channels.iter().find(|c|c.connection.provider_id=="openai").unwrap().id.clone();"protected_provider"},
            "derived-occupied"=>{raw["providers"].as_object_mut().unwrap().remove(GROUP_PROVIDER);rename.target=GROUP_PROVIDER.into();"derived_provider_occupied"},
            "foreign-account"=>{raw["providers"][OLD]["apiKey"]=json!("keychain:Krill-10/foreign");"credential_owner"},
            "missing-account"=>{raw["providers"][OLD]["apiKey"]=json!("keychain:Krill-1/missing");"credential_missing"},
            "credential-target-occupied"=>{put(&format!("{NEW}/{}",account_id(KEY_A)),"fixture-external-target").unwrap();"credential_target_occupied"},
            "shared-reference"=>{raw["providers"]["Krill-10"]["headers"]=json!({"X-Shared":format!("keychain:{OLD}/{}",account_id(KEY_A))});"credential_shared_reference"},
            "agent-mismatch"=>{let mut prefs=read(&f.root.join(PREFS));prefs["modelTemplateAgent"]["channelId"]=json!("99999999-0000-4000-8000-000000000001");write(&f.root.join(PREFS),&prefs);"agent_binding"},
            _=>unreachable!(),
        };
        write(&f.home.join("config.json"),&raw);let before=f.bytes();let keys=memory.snapshot();let writes=memory.writes();
        let failure=prepare(&f.root,&f.home,&[rename],None,&f.context()).err().expect("invalid preflight must fail");assert_eq!(failure,expected,"{mode}");
        assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);assert_eq!(memory.writes(),writes);
        receipt(&format!("PREFLIGHT-{mode}"),json!({"code":failure,"business_files_unchanged":true,"credential_mutations":0}));
    }
    let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let writes=memory.writes();
    let plan=f.plan(Some(&f.id));drop(plan);
    assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);assert_eq!(memory.writes(),writes);
    receipt("TX-CANCEL",json!({"prepared_plan_dropped":true,"business_files_unchanged":true,"credential_mutations":0,"journal_created":false}));

    let f=Fixture::new(memory);let before=f.bytes();let snap=f.snapshot();let mut policy=snap.policy;
    let c=policy.channels.iter_mut().find(|c|c.id==f.id).unwrap();c.label=NEW.into();c.connection.provider_id=NEW.into();
    c.connection.imported_config=Some(ImportedProviderRef{id:"40000000-0000-4000-8000-000000000001".into(),digest:format!("sha256:{}","0".repeat(64))});
    let failure=config_file::save(&f.root,&f.home,SaveRequest{expected_file:snap.policy_digest,policy,credentials:vec![],scope:None}).err().unwrap();
    assert_eq!(failure.code,"provider_import_scope");assert_eq!(f.bytes(),before);
    receipt("OLD-STALE-IMPORT",json!({"code":failure.code,"stale_receipt_rejected_before_load":true,"business_files_unchanged":true}));
}
fn failure_matrix(memory:&Memory) {
    for point in ["key:0","key:1","key:2","key:3","file:0","file:1","file:2","file:3","readback"] {
        let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let plan=f.plan(None);
        let outcome=execute(&plan,&Fault{fail:Some(point.into()),..Default::default()}).unwrap();
        assert_eq!(outcome,Outcome::RolledBack);assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);assert!(!plan.journal.exists());
        receipt(&format!("ROLLBACK-{point}"),json!({"outcome":outcome,"five_files_restored_byte_for_byte":true,"credential_map_exactly_restored":true}));
    }
    for index in 0..4 {
        let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let plan=f.plan(None);
        let outcome=execute(&plan,&Fault{real_write_fail:Some(index),..Default::default()}).unwrap();
        assert_eq!(outcome,Outcome::RolledBack);assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);
        receipt(&format!("FS-WRITE-FAILURE-{index}"),json!({"actual_atomic_replace_error":true,"outcome":outcome,"files_and_credentials_restored":true}));
    }
    // Each index is relative to execute(): four source/target baseline reads,
    // followed by the first source read, destination read, put, and readback.
    for (label,index) in [("source-read",1),("stage-put",11),("stage-readback",12)] {
        let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let plan=f.plan(None);memory.fail_after(index);
        let result=execute(&plan,&Fault::default());memory.clear_fault();
        if label=="source-read"{assert_eq!(result.unwrap_err(),"credential_read");}else{assert_eq!(result.unwrap(),Outcome::RolledBack);}
        assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);
        receipt(&format!("CREDENTIAL-FAILURE-{label}"),json!({"injected_in_memory_credential_api":true,"files_and_credentials_restored":true}));
    }
    let f=Fixture::new(memory);let plan=f.plan(None);
    let outcome=execute(&plan,&Fault{fail:Some("cleanup".into()),..Default::default()}).unwrap();assert_eq!(outcome,Outcome::CleanupPending);
    assert!(read(&f.home.join("config.json"))["providers"].get(NEW).is_some());assert!(plan.journal.exists());let after=f.bytes();
    assert_eq!(recover(&plan.journal).unwrap(),Outcome::Committed);assert_eq!(f.bytes(),after);assert!(get(&format!("{OLD}/{}",account_id(KEY_A))).unwrap().is_none());
    receipt("CLEANUP-PENDING",json!({"commit_kept":true,"pending_cleanup_reported":true,"replay_removed_unreferenced_old_accounts":true,"files_not_rolled_back_after_commit":true}));

    let f=Fixture::new(memory);let plan=f.plan(None);memory.fail_after(35);
    assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::CleanupPending);memory.clear_fault();
    assert_eq!(recover(&plan.journal).unwrap(),Outcome::Committed);
    receipt("CREDENTIAL-FAILURE-cleanup-delete",json!({"actual_memory_delete_api_failed":true,"durable_commit_recovered_forward":true}));

    let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let plan=f.plan(None);
    assert_eq!(execute(&plan,&Fault{fail:Some("file:1".into()),rollback_fail:true,..Default::default()}).unwrap(),Outcome::RecoveryRequired);
    assert_ne!(f.bytes(),before);assert!(plan.journal.exists());let journal=plan.journal.clone();drop(plan);
    assert_eq!(recover(&journal).unwrap(),Outcome::RolledBack);assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);
    assert_eq!(recover(&journal).unwrap(),Outcome::NoPending);
    receipt("ROLLBACK-FAILURE-REPLAY",json!({"pending_recovery_reported":true,"new_transaction_object_replayed_disk_journal":true,"five_files_and_credentials_restored":true,"replay_again_noop":true}));
}
fn drift_and_replay(memory:&Memory) {
    for mode in ["raw","layout","preferences","protocol-mapping","credential","target-credential"] {
        let f=Fixture::new(memory);let plan=f.plan(None);
        let expected=match mode {
            "raw"=>{let mut v=read(&f.home.join("config.json"));v["externalWriter"]=json!(1);write(&f.home.join("config.json"),&v);"file_changed"},
            "layout"=>{let mut v=read(&f.root.join(LAYOUT));v["channels"][0]["label"]=json!("external");write(&f.root.join(LAYOUT),&v);"file_changed"},
            "preferences"=>{let mut v=read(&f.root.join(PREFS));v["other"]=json!({"external":1});write(&f.root.join(PREFS),&v);"file_changed"},
            "protocol-mapping"=>{let mut v=read(&f.root.join(GROUPS));v[0]["protocol"]=json!("google");write(&f.root.join(GROUPS),&v);"guard_changed"},
            "credential"=>{put(&format!("{OLD}/{}",account_id(KEY_A)),"fixture-external-newer").unwrap();"credential_changed"},
            "target-credential"=>{put(&format!("{NEW}/{}",account_id(KEY_A)),"fixture-occupied-newer").unwrap();"credential_target_occupied"},
            _=>unreachable!(),
        };
        let changed=f.bytes();let keys=memory.snapshot();let writes=memory.writes();
        assert_eq!(execute(&plan,&Fault::default()).unwrap_err(),expected,"{mode}");assert_eq!(f.bytes(),changed);assert_eq!(memory.snapshot(),keys);assert_eq!(memory.writes(),writes);
        receipt(&format!("DRIFT-{mode}"),json!({"code":expected,"external_changes_preserved":true,"transaction_credential_writes":0}));
    }
    let f=Fixture::new(memory);let plan=f.plan(None);let config=f.home.join("config.json");
    assert_eq!(execute(&plan,&Fault{drift:Some(config.clone()),..Default::default()}).unwrap(),Outcome::RecoveryRequired);
    assert_eq!(read(&config),json!({"externalWriter":"newer-do-not-overwrite"}));assert!(plan.journal.exists());
    assert!(get(&format!("{NEW}/{}",account_id(KEY_A))).unwrap().is_some());
    receipt("DRIFT-DURING-READBACK",json!({"external_file_not_overwritten":true,"recovery_required":true,"staged_credentials_retained":true}));

    for point in ["key:0","key:3","file:0","file:1","file:2","file:3","readback","commit"] {
        let f=Fixture::new(memory);let before=f.bytes();let keys=memory.snapshot();let plan=f.plan(None);
        assert_eq!(execute(&plan,&Fault{crash:Some(point.into()),..Default::default()}).unwrap(),Outcome::SimulatedProcessLoss);
        let journal=plan.journal.clone();let after=f.bytes();drop(plan);
        let outcome=recover(&journal).unwrap();
        if point=="commit"{assert_eq!(outcome,Outcome::Committed);assert_eq!(f.bytes(),after);assert_saved(&f,memory,&keys);}else{assert_eq!(outcome,Outcome::RolledBack);assert_eq!(f.bytes(),before);assert_eq!(memory.snapshot(),keys);}
        receipt(&format!("JOURNAL-REPLAY-{point}"),json!({"outcome":outcome,"disk_journal_reloaded":true,"process_loss_simulated":true,"native_crash_test":false}));
    }
}
fn portable_consumer(memory:&Memory) {
    let f=Fixture::new(memory);let plan=f.plan(None);assert_eq!(execute(&plan,&Fault::default()).unwrap(),Outcome::Committed);
    // The transfer subsystem rejects unknown raw fields by contract. The main
    // transaction above preserved them; use an explicit known-field fixture
    // here to exercise the existing portable consumer rather than weakening it.
    let mut raw=read(&f.home.join("config.json"));raw["providers"][NEW].as_object_mut().unwrap().remove("futureProvider");
    for slot in raw["providers"][NEW]["apiKeyPool"].as_array_mut().unwrap(){slot.as_object_mut().unwrap().remove("futureSlot");}
    write(&f.home.join("config.json"),&raw);
    let public=model::provider_fields::public_from_raw(&raw["providers"][NEW]).unwrap();
    let auth=model::provider_auth::export_portable(&f.home,NEW,&public).unwrap();
    let auth=serde_json::to_value(&auth).unwrap();
    assert_eq!(auth["fields"]["apiKeyPool"][0]["key"],KEY_A);assert_eq!(auth["fields"]["apiKeyPool"][1]["key"],KEY_B);
    assert_eq!(auth["activeId"],account_id(KEY_B));
    let scope=model::transfer::ExportScope{channel_families:BTreeMap::new(),channel_models:BTreeMap::new(),include_policy:true,channel_ids:vec![f.id.clone()],template_refs:vec![],include_global:false};
    let package=model::portable_channels::export_file(&f.root,&f.home,scope).unwrap();
    assert_eq!(package.payload.policy.as_ref().unwrap().channels[0].id,f.id);
    let wire=serde_json::to_value(&package).unwrap();let connection=&wire["payload"]["policy"]["channels"][0]["connection"];
    assert!(connection.get("providerId").is_none());assert!(connection.get("credentialRef").is_none());
    assert!(!serde_json::to_string(&wire).unwrap().contains("keychain:"));
    receipt("PORTABLE-AUTH-CONSUMERS",json!({"stable_channel_id_exported":true,"source_provider_id_not_in_connection_wire":true,"uuid_credential_not_exported":true,"production_auth_export_resolves_new_owner":true,"active_key_pool_contents_unchanged":true,"cross_device_forced_provider_rename":false}));
}
fn readback(args:&[String]) {
    let root=Path::new(&args[2]);let home=Path::new(&args[3]);let id=&args[4];
    assert!(root.starts_with(fixture_root()));assert!(home.starts_with(fixture_root()));
    let snap=config_file::snapshot(root,home).unwrap();let c=snap.policy.channels.iter().find(|c|&c.id==id).unwrap();assert_eq!(c.connection.provider_id,NEW);assert_eq!(c.label,NEW);
    receipt("FRESH-PROCESS-READBACK",json!({"stable_id":id,"provider":c.connection.provider_id}));
}
fn main() {
    let memory=Memory::default();memory.install();
    let args=std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|s|s=="readback"){readback(&args);return;}
    assert!(fixture_root().is_absolute());assert_eq!(std::env::var("OPENCODEX_SANDBOX").as_deref(),Ok("1"));
    old_rules(&memory);positives(&memory);negative_preflight(&memory);failure_matrix(&memory);drift_and_replay(&memory);portable_consumer(&memory);
    receipt("COMPLETE",json!({"native_keychain":false,"network":false,"production_source_patched":false,"daily_client_restarted":false}));
}
