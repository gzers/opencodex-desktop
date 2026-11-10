// Diagnostic appended only to exact v0.2.23 test module.
#[test]
fn predev024_actual_new_and_rename_name_chain() {
 let out=std::path::PathBuf::from(std::env::var("NAME_CHAIN_OUT").unwrap());
 let cases=[("Krill-New",true),("中文渠道",false),("Team One",false),("团队/海外",false),("Name_" ,false)];
 let mut results=vec![];
 for (index,(name,exact)) in cases.into_iter().enumerate(){
  for create in [false,true]{
   let (_t,root,home)=fixture(raw());let started=begin(&root,&home);let (draft,id)=if create {
    let staged=stage_connection(&root,&home,StageConnectionV2{expected:Some(started.draft.as_ref().unwrap().base.clone()),expected_source:started.source_digest.clone(),expected_library:started.library_digest.clone(),channel_id:None,name:name.into(),url:"https://fixture.invalid/v1".into(),nodes:None,active_node_id:None,auth_mode:"none".into(),api_key:None,raw_model_ids:vec!["gpt-6-sol".into()],selected_model_ids:vec!["gpt-6-sol".into()]}).unwrap();
    let c=staged.draft.as_ref().unwrap().policy.channels.iter().find(|c|c.label==name).unwrap();let id=c.id.clone();(staged,id)
   }else{let id=started.draft.as_ref().unwrap().policy.channels[0].id.clone();(started,id)};
   let before=std::fs::read(home.join("config.json")).unwrap();let mode=if create{"create"}else{"rename"};
   let provider_before=draft.draft.as_ref().unwrap().policy.channels.iter().find(|c|c.id==id).unwrap().connection.provider_id.clone();
   let mut intents=vec![ChannelIntentV2::SetEnabled{channel_id:id.clone(),enabled:true}];if !create{intents.push(ChannelIntentV2::Rename{channel_id:id.clone(),name:name.into()});}
   let saved=save(&root,&home,SaveRequestV2{expected:draft.draft.unwrap().base,edits:vec![],channel_intents:intents,scope_channel_id:None}).unwrap();
   let channel=saved.saved_policy.channels.iter().find(|c|c.id==id).unwrap();assert_eq!(channel.label,name);assert_eq!(channel.connection.provider_id==name,exact);
   if !exact {assert_eq!(channel.connection.provider_id,format!("channel-{}",id.replace('-',"")));}
   let bytes=std::fs::read(home.join("config.json")).unwrap();let raw:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(raw["providers"][&channel.connection.provider_id]["name"],name);
   if !create{assert!(raw["providers"].get("Krill-1").is_none());assert_eq!(raw["defaultProvider"],channel.connection.provider_id);assert_eq!(raw["defaultModel"],format!("{}/vendor/custom",channel.connection.provider_id));assert_eq!(raw["providers"][&channel.connection.provider_id]["apiKey"],"synthetic-only-never-return");}
   let reopened=snapshot(&root,&home).unwrap();assert_eq!(reopened.saved_policy,saved.saved_policy);
   let path=format!("{index}-{mode}");std::fs::write(out.join(format!("{path}-saved-config.json")),&bytes).unwrap();std::fs::write(out.join(format!("{path}-before-config.json")),&before).unwrap();
   results.push(json!({"case":path,"mode":mode,"name":name,"channelId":id,"providerBefore":provider_before,"providerAfter":channel.connection.provider_id,"providerEqualsName":exact,"nameFieldWritten":true,"oldReferencesRewritten":!create,"savedPolicyAndReopenEqual":true}));
  }
 }
 std::fs::write(out.join("save-chain-results.json"),serde_json::to_vec_pretty(&results).unwrap()).unwrap();println!("NAME_CHAIN {}",json!({"passed":results.len(),"results":results}));
}
