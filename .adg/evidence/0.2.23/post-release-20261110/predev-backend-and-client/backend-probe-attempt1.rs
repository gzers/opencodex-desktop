// Appended only to isolated exact-tag config_file_v2_tests.rs. No production patch.
fn predev_save_request(s:&FileSnapshotV2) -> SaveRequestV2 {
 let c=&s.draft.as_ref().unwrap().policy.channels[0];
 SaveRequestV2{expected:s.draft.as_ref().unwrap().base.clone(),edits:vec![],channel_intents:vec![ChannelIntentV2::SetEnabled{channel_id:c.id.clone(),enabled:false}],scope_channel_id:None}
}
fn predev_conflict(case:&str,mutation:impl FnOnce(&mut Value),format_only:bool){
 let (_tmp,root,home)=fixture(raw());let started=begin(&root,&home);
 let draft_path=root.join(store_v2::DRAFT_PATH);let original_draft=std::fs::read(&draft_path).unwrap();
 let mut document:Value=serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();mutation(&mut document);
 let changed=if format_only{serde_json::to_vec_pretty(&document).unwrap()}else{serde_json::to_vec(&document).unwrap()};std::fs::write(home.join("config.json"),&changed).unwrap();
 let req=predev_save_request(&started);let error=save(&root,&home,req).unwrap_err();assert_eq!(error.code,"model_save_stale", "{case}");
 assert_eq!(std::fs::read(home.join("config.json")).unwrap(),changed);assert_eq!(std::fs::read(&draft_path).unwrap(),original_draft);
 // Actual snapshot sees new raw source while retaining server draft's original source.
 let fresh=snapshot(&root,&home).unwrap();assert_ne!(fresh.source_digest,started.source_digest);assert_eq!(fresh.draft.as_ref().unwrap().base,started.draft.as_ref().unwrap().base);
 let retry=save(&root,&home,predev_save_request(&fresh)).unwrap_err();assert_eq!(retry.code,"model_save_stale");
 let read=read_existing(&root,&home,request(&fresh));
 let read_status=match read {Ok(_)=>"read-success".to_owned(),Err(e)=>e.code};
 println!("PREDEV_CASE {}",serde_json::to_string(&json!({"case":case,"firstSave":"model_save_stale","retry":"model_save_stale","filePreserved":true,"draftPreserved":true,"freshSnapshotRetainsOldDraftBase":true,"explicitRead":read_status})).unwrap());
}
#[test] fn predev024_format_only(){predev_conflict("json-format-only", |_|{},true)}
#[test] fn predev024_claude_bookkeeping(){predev_conflict("claude-appliedAt-only",|d|d["claudeCode"]=json!({"desktopProfile":{"appliedAt":"synthetic-later","appliedFingerprint":"synthetic"}}),false)}
#[test] fn predev024_discovery_bookkeeping(){predev_conflict("discovery-knownModels-only",|d|d["modelDiscovery"]=json!({"knownModels":{"Krill-1":["gpt-6-sol"]},"recentArrivals":{}}),false)}
#[test] fn predev024_real_node_change(){predev_conflict("actual-provider-baseUrl-change",|d|d["providers"]["Krill-1"]["baseUrl"]=json!("https://changed.invalid/v1"),false)}
#[test] fn predev024_real_key_change(){predev_conflict("actual-provider-apiKey-change",|d|d["providers"]["Krill-1"]["apiKey"]=json!("synthetic-external-new-value"),false)}
#[test] fn predev024_unknown_global_change(){predev_conflict("unknown-global-change",|d|d["newUnknownGlobal"]=json!({"keep":true}),false)}
#[test] fn predev024_current_source_normal_save(){
 let (_tmp,root,home)=fixture(raw());let started=begin(&root,&home);let result=save(&root,&home,predev_save_request(&started)).unwrap();assert!(result.draft.is_none());let fresh=snapshot(&root,&home).unwrap();assert_eq!(fresh.source_digest,result.source_digest);assert!(!fresh.saved_policy.channels[0].connection.enabled);println!("PREDEV_CASE {}",json!({"case":"normal-save-and-reopen","pass":true}));
}
#[test] fn predev024_staged_key_intent_preserved_on_bookkeeping_conflict(){
 use std::cell::RefCell;
 #[derive(Default)] struct Vault(RefCell<BTreeMap<String,String>>);
 impl DraftCredentialStoreV2 for Vault {
  fn put(&self,r:&str,v:&str)->Result<()>{self.0.borrow_mut().insert(r.into(),v.into());Ok(())}
  fn get(&self,r:&str)->Result<Zeroizing<String>>{Ok(Zeroizing::new(self.0.borrow()[r].clone()))}
  fn remove(&self,r:&str)->Result<()>{self.0.borrow_mut().remove(r);Ok(())}
 }
 let (_tmp,root,home)=fixture(raw());let started=begin(&root,&home);let channel_id=started.draft.as_ref().unwrap().policy.channels[0].id.clone();let vault=Vault::default();
 let staged=stage_credential_with_store(&root,&home,StageCredentialV2{expected:started.draft.as_ref().unwrap().base.clone(),channel_id,action:"replace".into(),value:Some("synthetic-staged-key-never-log".into())},&vault).unwrap();
 let draft_path=root.join(store_v2::DRAFT_PATH);let draft_bytes=std::fs::read(&draft_path).unwrap();let mut document:Value=serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();document["claudeCode"]=json!({"desktopProfile":{"appliedAt":"synthetic-new-time"}});let changed=serde_json::to_vec(&document).unwrap();std::fs::write(home.join("config.json"),&changed).unwrap();
 let e=save(&root,&home,predev_save_request(&staged)).unwrap_err();assert_eq!(e.code,"model_save_stale");assert_eq!(std::fs::read(&draft_path).unwrap(),draft_bytes);assert_eq!(vault.0.borrow().len(),1);assert_eq!(std::fs::read(home.join("config.json")).unwrap(),changed);println!("PREDEV_CASE {}",json!({"case":"staged-credential-conflict-keeps-private-intent","pass":true}));
}
