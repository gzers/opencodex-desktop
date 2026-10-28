/* 对话策略上下文状态：仅内存意图、编译预览与固定观察，不触发真实接口。 */
(function(root,factory){if(typeof module==='object'&&module.exports)module.exports=factory();else root.PrototypeContextPolicy=factory();})(typeof window==='undefined'?this:window,function(){
 'use strict';
 const clone=v=>JSON.parse(JSON.stringify(v));
 const modes={inherit:'跟随默认',regular:'常规压缩',background:'后台压缩',notes:'历史与笔记'};
 const protocols={inherit:'跟随 Provider 默认',local:'客户端压缩流程',v2:'原生远程压缩'};
 const targets=[{id:'opencodex',name:'OpenCodex 本地代理'},{id:'krill',name:'Krill 直连'}];
 const blank=()=>({mode:'inherit',autoTokens:'',backgroundPercent:'',postTurnPercent:'',providers:Object.fromEntries(targets.map(t=>[t.id,{protocol:'inherit'}]))});
 function create(){
  let saved=blank(),draft=clone(saved),applied=clone(saved),selected='opencodex',failSave=false,observations={};
  const dirty=()=>JSON.stringify(draft)!==JSON.stringify(saved),pending=()=>JSON.stringify(saved)!==JSON.stringify(applied);
  function error(){for(const [field,label,min,max] of [['autoTokens','自动压缩阈值',1,Number.MAX_SAFE_INTEGER],['backgroundPercent','后台启动比例',1,100],['postTurnPercent','轮次结束压缩比例',0,100]]){if(field==='backgroundPercent'&&draft.mode!=='background'||field==='postTurnPercent'&&draft.mode==='notes')continue;const v=draft[field];if(v!==''&&(!/^\d+$/.test(v)||!Number.isSafeInteger(Number(v))||Number(v)<min||Number(v)>max))return label+'请输入'+min+'–'+(max===Number.MAX_SAFE_INTEGER?'安全范围内的':max)+'整数，或留空跟随默认';}return '';}
  function edit(field,value){if(field==='mode'&&Object.hasOwn(modes,value))draft.mode=value;else if(field==='protocol'&&Object.hasOwn(protocols,value))draft.providers[selected].protocol=value;else if(['autoTokens','backgroundPercent','postTurnPercent'].includes(field))draft[field]=String(value).trim();}
  function preview(){const out={features:{},model_providers:{}};const m=draft.mode;if(m!=='inherit'){out.features.async_compaction={enabled:m==='background'};out.features.context_management={experimental_mode:m==='notes'};out.features.token_budget={enabled:m==='notes',use_history_notes_extension:m==='notes'};}
   if(draft.autoTokens!=='')out.model_auto_compact_token_limit=Number(draft.autoTokens);
   if(draft.backgroundPercent!==''&&m==='background')out.features.async_compaction.model_async_compact_threshold_percent=Number(draft.backgroundPercent);
   if(draft.postTurnPercent!==''&&m!=='notes')out.model_post_turn_compact_threshold_percent=Number(draft.postTurnPercent);
   for(const target of targets){const p=draft.providers[target.id].protocol;if(p!=='inherit')out.model_providers[target.id]={capabilities:{remote_compaction:p==='local'?'unsupported':'v2'}};}
   if(!Object.keys(out.features).length)delete out.features;if(!Object.keys(out.model_providers).length)delete out.model_providers;return out;
  }
  function save(){const issue=error();if(issue)return {ok:false,error:issue};if(failSave)return {ok:false,error:'保存未完成，本次设置与原配置均已保留'};saved=clone(draft);return {ok:true,pending:pending()};}
  function restart(){applied=clone(saved);observations={};}
  function observe(scene,key='thread:config'){
   const fixtures={summary:{strategy:'后台压缩',processor:'代理摘要',history:'未使用',notes:'未使用',reason:'当前第三方代理路径生成摘要，未执行原生远程压缩'},native:{strategy:'常规压缩',processor:'原生远程压缩 · 直连示例',history:'未使用',notes:'未使用',reason:'已观察原生压缩返回；不代表所有代理路径均支持'},notes:{strategy:'历史与笔记',processor:'按需读取历史',history:'已调用 · 直连示例',notes:'已写入 · 直连示例',reason:'仅为此对话的固定观察，不是人工笔记管理或远端同步'},unavailable:{strategy:'历史与笔记',processor:'未执行',history:'链路未接通',notes:'未调用',reason:'当前 OpenCodex 第三方上下文入口尚未接通'}};
   if(scene==='unknown'){observations={};return;}if(fixtures[scene])observations[key]=clone(fixtures[scene]);
  }
  return {edit,save,restart,preview,error,observe,discard(){draft=clone(saved);},select(id){if(targets.some(t=>t.id===id))selected=id;},setSaveFailure(v){failSave=!!v;},observation:key=>clone(observations[key]||null),snapshot:()=>clone({modes,protocols,targets,selected,draft,saved,applied,dirty:dirty(),pending:pending(),failSave,observations})};
 }
 return {create};
});
