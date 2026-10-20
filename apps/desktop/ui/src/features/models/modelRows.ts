import {emptyRules,type ChannelPolicy,type ModelNaming,type ModelRules,type TemplateRef} from './api'
export interface ModelRowFields {rules:ModelRules;naming:ModelNaming;reference:TemplateRef|null}
export interface ModelRowBuffer extends ModelRowFields {base:ModelRowFields;error:string}
export interface ModelRowsState {selected:string[];expanded:string[];buffers:Record<string,ModelRowBuffer>}
export const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
export function createModelRowsState():ModelRowsState{return{selected:[],expanded:[],buffers:{}}}
export function rowFields(channel:ChannelPolicy,id:string):ModelRowFields{return copy({rules:channel.overrides[id]??emptyRules(),naming:channel.naming?.[id]??{},reference:channel.modelRefs[id]??null})}
export function rowPending(buffer:ModelRowBuffer):boolean{return JSON.stringify({rules:buffer.rules,naming:buffer.naming,reference:buffer.reference})!==JSON.stringify(buffer.base)}
export function pendingModelRows(state:ModelRowsState):string[]{return Object.entries(state.buffers).filter(([,value])=>rowPending(value)).map(([id])=>id)}
export function writeRow(channel:ChannelPolicy,id:string,fields:ModelRowFields){
 if(Object.values(fields.rules).every(value=>!value||value.mode==='inherit'))delete channel.overrides[id];else channel.overrides[id]=copy(fields.rules)
 if(fields.reference)channel.modelRefs[id]=copy(fields.reference);else delete channel.modelRefs[id]
 channel.naming??={};if(Object.keys(fields.naming).length)channel.naming[id]=copy(fields.naming);else delete channel.naming[id]
}
