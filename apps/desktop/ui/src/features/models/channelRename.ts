import type {ChannelPolicy} from './api'
import {protectedChannel} from './channelActions'

const reserved=new Set(['native','openai','chatgpt','openai-multi','combo','policy','__proto__','constructor','prototype'])
/** Runtime keys are an implementation detail. User names need not be ASCII. */
export function allocateChannelProvider(name:string,id:string,others:readonly ChannelPolicy[]):string {
 const occupied=new Set(others.filter(c=>c.id!==id).map(c=>c.connection.providerId.toLowerCase()))
 const legal=(value:string)=>/^[A-Za-z0-9](?:[A-Za-z0-9_-]{0,62}[A-Za-z0-9])?$/.test(value)&&!reserved.has(value.toLowerCase())&&!occupied.has(value.toLowerCase())
 const candidate=name.trim();if(legal(candidate))return candidate
 const base='channel-'+id.replace(/[^A-Za-z0-9]/g,'').slice(0,40)
 let value=base,index=0;while(!legal(value))value=base+'-'+(++index)
 return value
}
export function renameChannelLabel(channel:ChannelPolicy,label:string,others:readonly ChannelPolicy[]=[]){
 if(channel.label===label)return
 channel.label=label
 const saved=others.find(c=>c.id===channel.id)
 channel.connection.providerId=saved?.label===label?saved.connection.providerId:allocateChannelProvider(label,channel.id,others)
}
export function channelRenameError(channel:ChannelPolicy,saved:readonly ChannelPolicy[],draft:readonly ChannelPolicy[]=saved):string {
 const original=saved.find(c=>c.id===channel.id),name=channel.label.trim()
 if(original&&protectedChannel(original))return original.label!==channel.label||original.connection.providerId!==channel.connection.providerId?'官方认证或转发渠道不能直接改名':''
 if(!name||[...name].length>80||new TextEncoder().encode(name).length>256)return '请填写有效渠道名称，最多80字'
 if([...saved,...draft].some(c=>c.id!==channel.id&&c.label.trim().toLowerCase()===name.toLowerCase()))return '渠道名称已存在，请使用其它名称'
 return ''
}
export function channelRenamePreview(channel:ChannelPolicy,saved?:ChannelPolicy){
 if(!saved||saved.label===channel.label)return null
 return {fromLabel:saved.label,toLabel:channel.label}
}
