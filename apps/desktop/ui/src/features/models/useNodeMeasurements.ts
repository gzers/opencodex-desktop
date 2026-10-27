import { computed,onScopeDispose,ref,watch } from 'vue'
import { errorText,modelApi,type ChannelPolicy,type NodeMeasurement } from './api'
const nodeErrors:Record<string,string>={discovery_html:'服务返回了网页，没有模型清单',discovery_path:'未找到模型清单接口',discovery_json:'模型清单响应不是有效 JSON',discovery_dns:'地址解析失败',discovery_private:'请先确认允许可信私有网络',discovery_auth:'服务拒绝了这次请求；请核对 Key 是否有效',credential_unconfirmed:'该节点没有可确认的凭据',discovery_timeout:'请求超时',discovery_failed:'连接或清单请求失败',discovery_pagination:'分页未完整',discovery_shape:'服务未返回模型清单',discovery_limit:'响应超过上限',discovery_protocol:'该协议需专用连接',node_url:'节点地址无效',credential_store:'系统凭据不可用'}
export function nodeMeasurementError(code:string):string{return nodeErrors[code]??'检测未完成，请检查连接'}
export function useNodeMeasurements(channel:()=>ChannelPolicy,key:()=>string){
 const run=ref('');const pending=ref<string[]>([]);const results=ref<Record<string,NodeMeasurement>>({});const error=ref('');const stopped=ref(false)
 let sequence=0
 const binding=computed(()=>JSON.stringify({protocol:channel().connection.protocol,credentialRef:channel().connection.credentialRef,private:channel().connection.allowPrivateNetwork,nodes:channel().connection.nodes.map(n=>({id:n.id,url:n.url})),key:key()}))
 function invalidate(){sequence++;const id=run.value;if(id)void modelApi.stopNodeMeasurement(id).catch(()=>{});run.value='';pending.value=[];results.value={};error.value='';stopped.value=false}
 watch(binding,invalidate)
 onScopeDispose(invalidate)
 async function measure(ids:string[]){
  if(run.value)return
  const value=channel();const nodes=value.connection.nodes.filter(n=>ids.includes(n.id)).map(n=>({...n}));if(!nodes.length)return
  const current=++sequence;const id=crypto.randomUUID();run.value=id;pending.value=nodes.map(n=>n.id);error.value='';stopped.value=false
  try{const response=await modelApi.measureNodes({runId:id,protocol:value.connection.protocol,nodes,apiKey:key()||null,credentialRef:value.connection.credentialRef,allowPrivateNetwork:value.connection.allowPrivateNetwork,providerId:value.connection.providerId});if(current!==sequence||response.runId!==id)return;for(const result of response.results)if(ids.includes(result.nodeId))results.value[result.nodeId]=result;stopped.value=response.stopped}
  catch(cause){if(current===sequence)error.value=errorText(cause)}
  finally{if(current===sequence){run.value='';pending.value=[]}}
 }
 async function stop(){if(run.value)try{await modelApi.stopNodeMeasurement(run.value)}catch(cause){error.value=errorText(cause)}}
 return {run,pending,results,error,stopped,measure,stop}
}
