/* 账户同步原型能力登记。纯资料/内存判断，不读取授权，不刷新或写账户。 */
(function(root){
 'use strict';
 const platforms={chatgpt:'ChatGPT / Codex',antigravity:'Google Antigravity',kiro:'Kiro',muse:'Muse','claude-code':'Claude Code','gemini-cli':'Gemini CLI',opencode:'OpenCode'};
 const entries=[
  {accountPlatform:'chatgpt',owner:'opencodex',provider:'chatgpt',authKind:'oauth',info:'supported',login:'blocked',reason:'上游手工导入未开放；当前仅账户信息，目标设备需登录'},
  {accountPlatform:'antigravity',owner:'opencodex',provider:'google-antigravity',authKind:'oauth',info:'supported',login:'unverified',reason:'有来源专用导入流程；本软件尚无跨设备和过期刷新证明'},
  {accountPlatform:'kiro',owner:'opencodex',provider:'kiro',authKind:'device',info:'supported',login:'unverified',reason:'需要设备/客户端专用适配，不能按普通OAuth复制'},
  {accountPlatform:'muse',owner:'opencodex',provider:'muse',authKind:'oauth',info:'supported',login:'unverified',reason:'专用适配与跨设备验证未完成'},
  {accountPlatform:'claude-code',owner:'platform-native',provider:'anthropic',authKind:'oauth',info:'unavailable',login:'unavailable',reason:'Claude Code原生账户适配未纳入当前已支持能力'},
  {accountPlatform:'gemini-cli',owner:'platform-native',provider:'google',authKind:'oauth',info:'unavailable',login:'unavailable',reason:'Gemini CLI原生账户适配未纳入当前已支持能力'},
  {accountPlatform:'opencode',owner:'platform-native',provider:'provider-specific',authKind:'oauth',info:'unavailable',login:'unavailable',reason:'OpenCode原生账户适配未纳入当前已支持能力'}
 ].map(Object.freeze);
 function describe(account){
  const value=entries.find(entry=>['accountPlatform','owner','provider','authKind'].every(key=>entry[key]===account[key]));
  return value||{accountPlatform:account.accountPlatform||'unknown',owner:account.owner||'unknown',provider:account.provider||'unknown',authKind:account.authKind||'unknown',info:'unavailable',login:'unavailable',reason:'未登记此账户平台、管理方和认证类型的适配器，不能选择同步'};
 }
 function canSelect(account){return describe(account).info==='supported'}
 // Only a reviewed implementation with the complete evidence gate can add a
 // supported login entry. A raw file, an import endpoint or a fixture cannot.
 function canTransferLogin(account){const value=describe(account);return value.info==='supported'&&value.login==='supported'&&value.evidenceGate==='verified'}
 function identity(account){return JSON.stringify([account.accountPlatform||'unknown',account.owner||'unknown',account.provider||'unknown',account.subject||null,account.tenant||null,account.sourceAccountId||account.id]);}
 const api=Object.freeze({platforms:Object.freeze(platforms),entries:Object.freeze(entries),describe,canSelect,canTransferLogin,identity});
 if(typeof module==='object'&&module.exports)module.exports=api;
 if(root)root.SyncAccountCapabilities=api;
})(typeof window==='object'?window:null);
