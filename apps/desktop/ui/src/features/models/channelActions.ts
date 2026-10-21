import type {ChannelPolicy} from './api'
export function protectedChannel(channel:ChannelPolicy){
 return ['openai','chatgpt','openai-multi','combo'].includes(channel.connection.providerId)||['oauth','forward'].includes(channel.connection.authMode??'')
}
export function copyChannel(source:ChannelPolicy,label:string,reuseKey:boolean):ChannelPolicy{
 if(protectedChannel(source))throw {message:'原生、OAuth或转发来源不能复制为普通Key渠道'}
 if(!label.trim())throw {message:'请填写副本名称'}
 const value=structuredClone(source);value.id=crypto.randomUUID();value.label=label.trim();value.connection.providerId='ocxd_'+value.id.replaceAll('-','');value.connection.enabled=false;value.connection.sourceReadId=null;delete value.connection.importedConfig
 if(!reuseKey)value.connection.credentialRef=null
 const oldActive=source.connection.activeNodeId
 value.connection.nodes=value.connection.nodes.map(node=>({...node,id:crypto.randomUUID()}));value.connection.activeNodeId=value.connection.nodes[source.connection.nodes.findIndex(n=>n.id===oldActive)]?.id??value.connection.nodes[0]?.id??''
 return value
}
