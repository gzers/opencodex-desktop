import {computed,onScopeDispose,ref,watch} from 'vue'
import {errorText,type ChannelPolicy,type NodeMeasurement} from './api'
import {modelV2Api,type ChannelV2,type DiscoveryKindV2,type ReadBaseV2} from './api-v2'
const nodeErrors:Record<string,string>={discovery_html:'服务返回网页而非模型清单',discovery_path:'未找到模型清单接口',discovery_json:'模型清单不是有效 JSON',discovery_dns:'地址解析失败',discovery_auth:'服务拒绝了这次请求，请核对凭据',credential_unconfirmed:'该节点没有明确认可的凭据',discovery_timeout:'请求超时',discovery_failed:'连接或清单请求失败',discovery_pagination:'分页未完整',discovery_shape:'服务未返回模型清单',discovery_limit:'响应超过上限',discovery_protocol:'请选择本次清单获取方式',node_url:'节点地址无效',credential_store:'系统凭据不可用',model_save_stale:'草稿基准已变化，请重新核对'}
export function nodeMeasurementError(code:string):string{return nodeErrors[code]??'检测未完成，请核对连接与本次清单方式'}
export interface NodeMeasurementAdmissionV2 {expected:()=>ReadBaseV2|null;discoveryKind:()=>DiscoveryKindV2|'';enabled?:()=>boolean}
export function useNodeMeasurements(channel:()=>ChannelPolicy|ChannelV2,_key:()=>string=()=>'',admission?:NodeMeasurementAdmissionV2){
 const run=ref(''),pending=ref<string[]>([]),results=ref<Record<string,NodeMeasurement>>({}),error=ref(''),stopped=ref(false)
 let sequence=0,stoppedReceipt:{id:string;sequence:number}|null=null
 const isStoppedReceipt=(id:string,current:number)=>stoppedReceipt?.id===id&&stoppedReceipt.sequence===current
 const binding=computed(()=>JSON.stringify({channelId:channel().id,providerId:channel().connection.providerId,credentialRef:channel().connection.credentialRef,nodes:channel().connection.nodes.map(n=>({id:n.id,url:n.url})),expected:admission?.expected(),discoveryKind:admission?.discoveryKind()}))
 function invalidate(){sequence++;stoppedReceipt=null;const id=run.value;if(id)void modelV2Api.stopMeasurement(id).catch(()=>{});run.value='';pending.value=[];results.value={};error.value='';stopped.value=false}
 watch(binding,invalidate,{flush:'sync'})
 onScopeDispose(invalidate)
 async function measure(ids:string[]){
  if(run.value)return
  if(!admission||admission.enabled?.()===false){error.value='节点检测服务尚未就绪，未发送请求。';return}
  const expected=admission.expected(),discoveryKind=admission.discoveryKind()
  if(!expected){error.value='请先取得当前可信渠道草稿。';return}
  if(!discoveryKind){error.value='请选择本次模型清单获取方式。';return}
  const value=channel();if(!('models'in value)){error.value='节点检测仅接受新版可信渠道，未从旧协议字段推断。';return}
  const nodeIds=[...new Set(ids)].filter(id=>value.connection.nodes.some(n=>n.id===id));if(!nodeIds.length)return
  const current=++sequence,id=crypto.randomUUID();stoppedReceipt=null;run.value=id;pending.value=nodeIds;error.value='';stopped.value=false
  try{const response=await modelV2Api.measureNodes({expected,channelId:value.id,nodeIds,discoveryKind,runId:id});const completedStop=isStoppedReceipt(id,current)&&response.stopped;if(current!==sequence&&!completedStop||response.runId!==id)return;for(const result of response.results)if(nodeIds.includes(result.nodeId))results.value[result.nodeId]=result;stopped.value=stopped.value||response.stopped}
  catch(cause){if(current===sequence)error.value=errorText(cause)}
  finally{if(current===sequence){run.value='';pending.value=[]}}
 }
 async function stop(){const id=run.value,current=sequence;if(!id)return;try{await modelV2Api.stopMeasurement(id);if(current===sequence&&run.value===id){stoppedReceipt={id,sequence:current};sequence++;run.value='';pending.value=[];stopped.value=true}}catch(cause){if(current===sequence)error.value=errorText(cause)}}
 return {run,pending,results,error,stopped,measure,stop}
}
