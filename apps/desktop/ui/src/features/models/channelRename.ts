import type {ChannelPolicy} from './api'
import {protectedChannel} from './channelActions'

export interface ChannelRenameIntent {providerExplicit:boolean}
export function channelRenameIntent(channel:ChannelPolicy,saved?:ChannelPolicy):ChannelRenameIntent {
 return {providerExplicit:!!saved&&channel.connection.providerId!==saved.connection.providerId&&channel.connection.providerId!==channel.label.trim()}
}
export function renameChannelLabel(channel:ChannelPolicy,label:string,intent:ChannelRenameIntent){
 channel.label=label
 if(!intent.providerExplicit)channel.connection.providerId=label.trim()
}
export function renameChannelProvider(channel:ChannelPolicy,provider:string,intent:ChannelRenameIntent){
 channel.connection.providerId=provider;intent.providerExplicit=true
}
/** Saved identities remain occupied even when another draft deletes/renames
 * them. The backend also checks hidden protocol groups and raw configuration. */
export function channelRenameError(channel:ChannelPolicy,saved:readonly ChannelPolicy[],draft:readonly ChannelPolicy[]=saved):string {
 const original=saved.find(c=>c.id===channel.id),provider=channel.connection.providerId,name=channel.label.trim()
 if(original&&protectedChannel(original)){
  return original.label!==channel.label||original.connection.providerId!==provider?'官方认证或转发渠道不能直接改名':''
 }
 if(!name||[...name].length>80)return '请填写 1–80 字的渠道名称'
 const others=[...saved,...draft].filter(c=>c.id!==channel.id)
 if(others.some(c=>c.label.trim().toLowerCase()===name.toLowerCase()))return '渠道名称已存在，请使用其它名称'
 if(!/^[A-Za-z0-9](?:[A-Za-z0-9_-]{0,62}[A-Za-z0-9])?$/.test(provider))return '运行标识须为 1–64 位字母、数字、连字符或下划线，以字母或数字开头和结尾'
 if(['openai','chatgpt','openai-multi','combo','policy','__proto__','constructor','prototype'].includes(provider.toLowerCase()))return '此运行标识由官方管理，请使用其它标识'
 if(others.some(c=>c.connection.providerId===provider))return '运行标识已存在，请使用其它标识'
 return ''
}
export function channelRenamePreview(channel:ChannelPolicy,saved?:ChannelPolicy){
 if(!saved||saved.label===channel.label&&saved.connection.providerId===channel.connection.providerId)return null
 const model=channel.discovered.find(m=>channel.selection.mode==='all'||channel.selection.mode==='whitelist'&&channel.selection.models.includes(m.id))?.id??channel.modelOrder[0]??'原始模型ID'
 return {fromLabel:saved.label,toLabel:channel.label,fromProvider:saved.connection.providerId,toProvider:channel.connection.providerId,selector:channel.connection.providerId+'/'+model}
}
