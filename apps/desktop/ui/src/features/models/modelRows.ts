import {emptyRules,type ChannelPolicy,type ModelNaming,type ModelRules,type TemplateRef} from './api'
export interface ModelRowFields {rules:ModelRules;naming:ModelNaming;reference:TemplateRef|null}
export interface ModelRowBuffer extends ModelRowFields {base:ModelRowFields;error:string}
export interface ModelRowsState {selected:string[];expanded:string[];buffers:Record<string,ModelRowBuffer>}
export const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
export function createModelRowsState():ModelRowsState{return{selected:[],expanded:[],buffers:Object.create(null) as Record<string,ModelRowBuffer>}}
export function own<T>(values:Record<string,T>|undefined,id:string):T|undefined{return values&&Object.hasOwn(values,id)?values[id]:undefined}
export function rowFields(channel:ChannelPolicy,id:string):ModelRowFields{return copy({rules:own(channel.overrides,id)??emptyRules(),naming:own(channel.naming,id)??{},reference:own(channel.modelRefs,id)??null})}
export function rowPending(buffer:ModelRowBuffer):boolean{return JSON.stringify({rules:buffer.rules,naming:buffer.naming,reference:buffer.reference})!==JSON.stringify(buffer.base)}
export function pendingModelRows(state:ModelRowsState):string[]{return Object.entries(state.buffers).filter(([,value])=>rowPending(value)).map(([id])=>id)}
export function writeRow(channel:ChannelPolicy,id:string,fields:ModelRowFields){
 if(Object.values(fields.rules).every(value=>!value||value.mode==='inherit'))delete channel.overrides[id];else channel.overrides={...channel.overrides,[id]:copy(fields.rules)}
 if(fields.reference)channel.modelRefs={...channel.modelRefs,[id]:copy(fields.reference)}
 else delete channel.modelRefs[id]
 channel.naming??={};if(Object.keys(fields.naming).length)channel.naming={...channel.naming,[id]:copy(fields.naming)}
 else delete channel.naming[id]
}
