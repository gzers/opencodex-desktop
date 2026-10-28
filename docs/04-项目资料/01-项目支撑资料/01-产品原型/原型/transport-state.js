/* 连接与传输原型状态；纯内存，不执行配置写入、握手或重启。 */
(function(root,factory){
 if(typeof module==='object'&&module.exports)module.exports=factory();else root.PrototypeTransportState=factory();
})(typeof window==='undefined'?this:window,function(){
 'use strict';
 const copy=v=>JSON.parse(JSON.stringify(v));
 const targets=[{id:'opencodex',name:'OpenCodex 本地代理',proxy:true},{id:'krill',name:'Krill 直连',proxy:false}];
 const policies={inherit:'跟随默认',prefer:'优先 WebSocket',http:'仅 HTTP'};
 const blank=()=>({policy:'inherit',timeout:''});
 function create(){
  const saved=Object.fromEntries(targets.map(t=>[t.id,blank()])),applied=copy(saved),draft=copy(saved);
  let selected='opencodex',mode='local',failSave=false,observations={},handshakes={};
  const valid=id=>targets.some(t=>t.id===id);
  const error=id=>!Object.hasOwn(policies,draft[id]?.policy)?'请选择传输偏好':draft[id].timeout!==''&&(!/^\d+$/.test(String(draft[id].timeout))||!Number.isSafeInteger(Number(draft[id].timeout)))?'连接超时请输入非负整数，或留空沿用客户端':'';
  const pending=id=>JSON.stringify(saved[id])!==JSON.stringify(applied[id]);
  const dirty=id=>JSON.stringify(saved[id])!==JSON.stringify(draft[id]);
  function snapshot(){return copy({targets,policies,selected,mode,saved,applied,draft,observations,handshakes,failSave,pending:targets.filter(t=>pending(t.id)).map(t=>t.id),dirty:targets.filter(t=>dirty(t.id)).map(t=>t.id)});}
  function select(id){if(valid(id))selected=id;}
  function edit(field,value){if(field==='policy'&&Object.hasOwn(policies,value))draft[selected].policy=value;if(field==='timeout')draft[selected].timeout=String(value).trim();}
  function save(){const issue=error(selected);if(issue)return {ok:false,error:issue};if(failSave)return {ok:false,error:'保存未完成，原配置与本次输入均已保留'};saved[selected]=copy(draft[selected]);delete handshakes[selected];return {ok:true,pending:pending(selected)};}
  function discard(){draft[selected]=copy(saved[selected]);}
  // A restart adopts saved intent only; actual transport still requires a request observation.
  function restart(){for(const t of targets)applied[t.id]=copy(saved[t.id]);observations={};handshakes={};}
  function effective(id){const p=applied[id].policy;return p==='prefer'?true:p==='http'?false:null;}
  function probe(){const local=selected==='opencodex',disabled=local&&mode==='link'||local&&effective(selected)===false;handshakes[selected]=disabled?'握手未通过：当前本地入口使用 HTTP':local&&effective(selected)===null?'未确认：当前本地入口沿用已有默认':'握手通过；尚未观察实际请求';return handshakes[selected];}
  function observe(scene,objectKey='thread:config'){
   if(!['partial','end-to-end','fallback'].includes(scene)){observations={};return;}
   // Fixture observations identify their own provider, independent of form selection.
   if(effective('opencodex')===null&&mode!=='link'){observations={};return;}
   const allowed=effective('opencodex')===true&&mode!=='link';
   observations[objectKey]={provider:'opencodex',ingress:allowed?'WebSocket':'HTTP',upstream:allowed&&scene==='end-to-end'?'WebSocket':'HTTP/SSE',reason:mode==='link'?'Link 通道使用 HTTP':!allowed?'当前入口未启用 WebSocket':scene==='partial'?'当前代理未开放第三方上游 WebSocket':scene==='fallback'?'上游连接不可用，已使用 HTTP':'已观察到支持的官方上游使用 WebSocket'};
  }
  function observation(key){return copy(observations[key]||null);}
  function preview(){const value=draft[selected],flag=mode==='link'&&selected==='opencodex'?false:value.policy==='inherit'?undefined:value.policy==='prefer';return {provider:selected,...(flag!==undefined?{supports_websockets:flag,...(selected==='opencodex'&&mode!=='link'?{websockets:flag}:{})}:{}),...(value.timeout!==''&&!error(selected)?{websocket_connect_timeout_ms:Number(value.timeout)}:{})};}
  return {snapshot,select,edit,save,discard,restart,probe,observe,observation,preview,error:()=>error(selected),setMode(value){mode=value==='link'?'link':'local';observations={};handshakes={};},setSaveFailure(value){failSave=!!value;}};
 }
 return {create};
});
