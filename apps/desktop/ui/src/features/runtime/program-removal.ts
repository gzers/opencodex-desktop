import {ref,shallowRef} from 'vue'
import {defineStore} from 'pinia'
import {invoke} from '@tauri-apps/api/core'
import {runtimeErrorText} from './errors'
export interface RemovalReview {
 id:string;resume:boolean;commandPath:string|null;shellPaths:string[];confirmationDigest:string
 removal:{operationId:string;versions:string[];programRoot:string;preservedHome:string;files:number;bytes:number;confirmationDigest:string}
}
export interface RemovalOutcome {operationId:string;removedVersions:string[];configurationPreserved:boolean}
export const programRemovalApi={
 preview:()=>invoke<RemovalReview|null>('runtime_removal_preview'),
 apply:(request:{id:string;confirmationDigest:string})=>invoke<RemovalOutcome>('runtime_removal_apply',{request}),
}
export const useProgramRemovalStore=defineStore('runtime-program-removal',()=>{
 const open=ref(false),reading=ref(false),executing=ref(false),checked=ref(false),error=ref('')
 const review=shallowRef<RemovalReview|null>(null),outcome=shallowRef<RemovalOutcome|null>(null)
 let generation=0
 async function inspect(){
  if(executing.value){open.value=true;return}
  open.value=true;reading.value=true;checked.value=false;error.value='';review.value=null;outcome.value=null
  const seq=++generation
  try{const value=await programRemovalApi.preview();if(seq!==generation||!open.value)return;review.value=value;checked.value=true}
  catch(cause){if(seq===generation)error.value=runtimeErrorText(cause)}
  finally{if(seq===generation)reading.value=false}
 }
 function close(){open.value=false;generation++;reading.value=false}
 async function apply():Promise<boolean>{
  const selected=review.value;if(!selected||executing.value||reading.value)return false
  executing.value=true;error.value=''
  try{
   const result=await programRemovalApi.apply({id:selected.id,confirmationDigest:selected.confirmationDigest})
   const sorted=(v:string[])=>JSON.stringify([...v].sort())
   if(result.operationId!==selected.removal.operationId||!result.configurationPreserved||sorted(result.removedVersions)!==sorted(selected.removal.versions))throw new Error('卸载回执与本次范围不一致，请核对原操作。')
   outcome.value=result;review.value=null;checked.value=false;return true
  }catch(cause){error.value=runtimeErrorText(cause);review.value=null;checked.value=false;return false}
  finally{executing.value=false}
 }
 return {open,reading,executing,checked,error,review,outcome,inspect,close,apply}
})
