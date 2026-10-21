import type {ChannelPolicy,ModelPolicy} from './api'
export interface PolicyChange {field:string;before:unknown;after:unknown}
export interface ChannelChangeGroup {id:string;label:string;kind:'added'|'removed'|'changed';changes:PolicyChange[]}
function connection(channel:ChannelPolicy){
 const {credentialRef,importedConfig,sourceReadId,enabled: _enabled,...publicFields}=channel.connection
 return {...publicFields,authentication:credentialRef?'已配置本机认证':'未配置本机认证',...(importedConfig?{portableSource:'已接收的保护配置'}:{}),...(sourceReadId?{source:'已读取官方渠道'}:{})}
}
function fields(channel:ChannelPolicy):Record<string,unknown>{return{'名称':channel.label,'连接与节点':connection(channel),'启用':channel.connection.enabled,'模型公开范围':channel.selection,'型号顺序':channel.modelOrder,'发现清单':channel.discovered,'渠道默认字段':channel.defaults,'族精确采用':channel.templateRefs,'单模型精确采用':channel.modelRefs,'族覆盖':channel.familyOverrides,'单模型覆盖':channel.overrides,'别名与显示名':channel.naming??{}}}
function changed(field:string,before:unknown,after:unknown):PolicyChange[]{return JSON.stringify(before)===JSON.stringify(after)?[]:[{field,before:before??null,after:after??null}]}
export function policyChanges(saved:ModelPolicy,draft:ModelPolicy){
 const global=[...changed('全局族精确采用',saved.templateRefs,draft.templateRefs),...changed('默认渠道',saved.defaultChannelId??null,draft.defaultChannelId??null),...changed('渠道顺序',saved.channels.map(c=>c.id),draft.channels.map(c=>c.id))]
 const channels:ChannelChangeGroup[]=[]
 for(const id of new Set([...saved.channels,...draft.channels].map(c=>c.id))){
  const before=saved.channels.find(c=>c.id===id),after=draft.channels.find(c=>c.id===id),left=before?fields(before):{},right=after?fields(after):{}
  const changes=[...new Set([...Object.keys(left),...Object.keys(right)])].flatMap(key=>changed(key,left[key],right[key]))
  if(before&&after&&before.connection.credentialRef!==after.connection.credentialRef)changes.push({field:'本机认证引用',before:'原本机认证',after:'拟更换本机认证（不展示保护引用）'})
  if(before&&after&&JSON.stringify(before.connection.importedConfig)!==JSON.stringify(after.connection.importedConfig))changes.push({field:'接收的原渠道配置',before:'原保护配置',after:'拟采用另一保护配置（不展示本机引用）'})
  if(before&&after&&before.connection.sourceReadId!==after.connection.sourceReadId)changes.push({field:'本机读取基准',before:'原读取基准',after:'拟使用另一读取基准（不展示本机引用）'})
  if(changes.length)channels.push({id,label:after?.label??before?.label??id,kind:!before?'added':!after?'removed':'changed',changes})
 }
 const removals=(draft.pendingRemovals??[]).map(r=>({channelId:r.channelId,providerId:r.providerId}));const previous=(saved.pendingRemovals??[]).map(r=>({channelId:r.channelId,providerId:r.providerId}))
 global.push(...changed('已读取官方渠道的待移除项',previous,removals),...changed('恢复来源',saved.recoveryId??null,draft.recoveryId??null))
 return{global,channels}
}
