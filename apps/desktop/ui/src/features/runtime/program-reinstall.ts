import {defineStore} from 'pinia'
import {ref,shallowRef} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {runtimeErrorText} from './errors'
export interface BuiltinReview {version:string;resume:boolean;programRoot:string;configPath:string;confirmationDigest:string}
export interface BuiltinOutcome {version:string;configurationPreserved:boolean}
export const builtinApi={preview:()=>invoke<BuiltinReview>('runtime_builtin_preview'),
 apply:(confirmationDigest:string)=>invoke<BuiltinOutcome>('runtime_builtin_reinstall',{request:{confirmationDigest}})}
export const useProgramReinstallStore=defineStore('runtime-program-reinstall',()=>{
 const open=ref(false),reading=ref(false),executing=ref(false),error=ref('')
 const review=shallowRef<BuiltinReview|null>(null),outcome=shallowRef<BuiltinOutcome|null>(null)
 let sequence=0
 async function inspect(){
  open.value=true;if(executing.value)return
  const seq=++sequence;reading.value=true;review.value=null;outcome.value=null;error.value=''
  try{const r=await builtinApi.preview();if(seq===sequence&&open.value)review.value=r}
  catch(cause){if(seq===sequence)error.value=runtimeErrorText(cause)}
  finally{if(seq===sequence)reading.value=false}
 }
 function close(){open.value=false;sequence++;reading.value=false}
 async function apply(){
  const original=review.value;if(!original||reading.value||executing.value)return false
  executing.value=true;error.value=''
  try{const r=await builtinApi.apply(original.confirmationDigest);if(r.version!==original.version||r.configurationPreserved!==true)throw new Error('重装回执与原候选不一致，请核对原操作。');outcome.value=r;review.value=null;return true}
  catch(cause){error.value=runtimeErrorText(cause);review.value=null;return false}
  finally{executing.value=false}
 }
 return{open,reading,executing,error,review,outcome,inspect,close,apply}
})
