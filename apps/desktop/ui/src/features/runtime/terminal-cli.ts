import { ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { runtimeErrorText } from './errors'
import { useFeedbackStore } from '@/app/feedback/store'
export interface TerminalPreview {
 action:'enable'|'disable'; enabled:boolean; pending:boolean; version:string|null
 commandPath:string;runtimeEntry:string;opencodexHome:string;profilePaths:string[];conflicts:string[];confirmationDigest:string
}
interface Outcome { action:'enable'|'disable';enabled:boolean;commandPath:string;opencodexHome:string }
export const terminalCliApi={
 preview:()=>invoke<TerminalPreview>('runtime_terminal_preview'),
 apply:(request:{action:'enable'|'disable';confirmationDigest:string})=>invoke<Outcome>('runtime_terminal_apply',{request}),
}
export const useTerminalCliStore=defineStore('runtime-terminal-cli',()=>{
 const open=ref(false),reading=ref(false),executing=ref(false),error=ref(''),result=ref('')
 const preview=shallowRef<TerminalPreview|null>(null)
 let generation=0
 async function inspect(){
  if(executing.value){open.value=true;return}
  open.value=true;reading.value=true;error.value='';result.value='';preview.value=null
  const seq=++generation
  try{const value=await terminalCliApi.preview();if(seq===generation&&open.value)preview.value=value}
  catch(cause){if(seq===generation)error.value=runtimeErrorText(cause)}
  finally{if(seq===generation)reading.value=false}
 }
 function close(){open.value=false;generation++;reading.value=false}
 async function apply(){
  const current=preview.value;if(!current||reading.value||executing.value||current.conflicts.length||result.value)return
  executing.value=true;error.value=''
  try{
   const value=await terminalCliApi.apply({action:current.action,confirmationDigest:current.confirmationDigest})
   if(value.action!==current.action||value.enabled!==(current.action==='enable')||value.commandPath!==current.commandPath||value.opencodexHome!==current.opencodexHome)throw new Error('终端入口回执与本次确认不一致，请重新核对。')
   preview.value={...current,enabled:value.enabled,pending:false}
   result.value=value.enabled?'ocx 命令已启用，请打开新终端使用。':'终端 ocx 入口已停用。'
   useFeedbackStore().showToast(result.value)
  }catch(cause){error.value=runtimeErrorText(cause);preview.value=null}
  finally{executing.value=false}
 }
 return {open,reading,executing,error,result,preview,inspect,close,apply}
})
