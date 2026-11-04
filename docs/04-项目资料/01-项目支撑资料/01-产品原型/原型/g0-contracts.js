/* 0.2.21 G0：纯内存的校验与判新合同；不访问命令、网络或配置。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.G0Contracts=api;})(typeof window==='object'?window:typeof globalThis==='object'?globalThis:this,function(){
 'use strict';
 const reserved=new Set(['native','openai','chatgpt','openai-multi','combo','policy','__proto__','constructor','prototype']);
 function providerError(channel,others=[],occupied=[]){
  const id=channel.providerId??channel.id,base=channel.originalProviderId??channel.id;
  if(channel.id==='native'||['oauth','forward'].includes(channel.connection?.auth)||channel.managed){return id!==base?'官方或受保护来源不能改运行标识':'';}
  if(!/^[A-Za-z0-9](?:[A-Za-z0-9_-]{0,62}[A-Za-z0-9])?$/.test(id))return '运行标识需为 1–64 位字母、数字、连字符或下划线，并以字母或数字开头和结尾；请明确填写合法目标';
  if(reserved.has(id.toLowerCase()))return '该运行标识由官方保留，请换一个标识';
  const keys=p=>[p.providerId??p.id,...(p.derivedProviderIds||[])].map(x=>x.toLowerCase());
  const targets=keys(channel);
  if(others.filter(p=>p.id!==channel.id).some(p=>keys(p).some(k=>targets.includes(k)))||occupied.some(k=>targets.includes(k.toLowerCase())))return '运行标识或派生标识已被其它配置占用，不能覆盖';
  return '';
 }
 function decideSavedContent(local,remote){
  const result=(status,source,reason)=>({status,source:source||(status==='same'?'unchanged':'undetermined'),reason});
  if(local?.readError||remote?.readError)return result('unknown',null,'内容读取失败，不能当作空内容');
  if(local?.exists===false&&remote?.exists===false)return result('same',null,'两端没有所选内容，无需更新');
  if(remote?.exists===false&&local?.exists)return result('update','local','云端尚无所选内容，上传新增');
  if(local?.exists===false&&remote?.exists)return result('update','remote','本机尚无所选内容，下载新增');
  if(!local||!remote||!local.scopeKey||local.scopeKey!==remote.scopeKey)return result('unknown',null,'保存版本与当前所选范围不匹配');
  if(!local.contentDigest||!remote.contentDigest)return result('unknown',null,'内容摘要缺失，无法确认保存版本');
  if(local.contentDigest===remote.contentDigest)return result('same',null,'所选内容一致，无需更新');
  const reliable=v=>v.timeKind==='saved'&&v.bound===true&&!v.clockSkew&&typeof v.savedAt==='string'&&/(Z|[+-]\d{2}:\d{2})$/.test(v.savedAt)&&Number.isFinite(Date.parse(v.savedAt));
  if(!reliable(local)||!reliable(remote))return result('unknown',null,'缺少与内容绑定的可靠保存时间（未知、历史 mtime 或时钟异常）');
  const a=Date.parse(local.savedAt),b=Date.parse(remote.savedAt);
  if(a===b)return result('unknown',null,'保存时间相同但内容不同，无法判定新版');
  return result('update',a>b?'local':'remote',a>b?'本机已保存内容较新，上传云端':'云端已保存内容较新，下载本机');
 }
 return Object.freeze({providerError,decideSavedContent});
});
