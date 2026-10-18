import { computed,onScopeDispose,ref,watch } from 'vue'
import { errorText,modelApi,type ChannelPolicy,type NodeMeasurement } from './api'
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
  try{const response=await modelApi.measureNodes({runId:id,protocol:value.connection.protocol,nodes,apiKey:key()||null,credentialRef:value.connection.credentialRef,allowPrivateNetwork:value.connection.allowPrivateNetwork});if(current!==sequence||response.runId!==id)return;for(const result of response.results)if(ids.includes(result.nodeId))results.value[result.nodeId]=result;stopped.value=response.stopped}
  catch(cause){if(current===sequence)error.value=errorText(cause)}
  finally{if(current===sequence){run.value='';pending.value=[]}}
 }
 async function stop(){if(run.value)try{await modelApi.stopNodeMeasurement(run.value)}catch(cause){error.value=errorText(cause)}}
 return {run,pending,results,error,stopped,measure,stop}
}
