import {emptyRules,type ChannelPolicy,type ModelNaming,type ModelRules,type TemplateRef} from './api'
import type {ChannelV2,ChannelIntentV2,ModelEditV2,LibraryV2,RefV2} from './api-v2'
export interface ModelRowFields {rules:ModelRules;naming:ModelNaming;reference:TemplateRef|null}
export interface ModelRowBuffer extends ModelRowFields {base:ModelRowFields;error:string}
export interface ModelRowsState {selected:string[];expanded:string[];buffers:Record<string,ModelRowBuffer>}
export const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
export function createModelRowsState():ModelRowsState{return{selected:[],expanded:[],buffers:Object.create(null) as Record<string,ModelRowBuffer>}}
export function own<T>(values:Record<string,T>|undefined,id:string):T|undefined{return values&&Object.hasOwn(values,id)?values[id]:undefined}
export function rowFields(channel:ChannelPolicy,id:string):ModelRowFields{return copy({rules:own(channel.overrides,id)??emptyRules(),naming:own(channel.naming,id)??{},reference:own(channel.modelRefs,id)??null})}
export function rowPending(buffer:ModelRowBuffer):boolean{return JSON.stringify({rules:buffer.rules,naming:buffer.naming,reference:buffer.reference})!==JSON.stringify(buffer.base)}
export function pendingModelRows(state:ModelRowsState):string[]{return Object.entries(state.buffers).filter(([,value])=>rowPending(value)).map(([id])=>id)}
/** V2 buffers hold explicit intentions only, never source/effective/automatic bindings. */
export function rowFieldsV2(channel:ChannelV2,id:string,rules?:ModelRules):ModelRowFields{const model=channel.models.find(m=>m.rawModelId===id);return copy({rules:rules??model?.explicitOverrides??emptyRules(),naming:own(channel.naming,id)??{},reference:model?.binding?.kind==='manual'?model.binding.reference:null})}
export function rowChangesV2(channel:ChannelV2,id:string,before:ModelRowFields,next:ModelRowFields,library:LibraryV2){
 const edits:ModelEditV2[]=[],intents:ChannelIntentV2[]=[]
 for(const field of Object.keys({...before.rules,...next.rules}) as (keyof ModelRules)[]){const value=next.rules[field];if(JSON.stringify(before.rules[field]??{mode:'inherit'})===JSON.stringify(value??{mode:'inherit'}))continue;edits.push(!value||value.mode==='inherit'?{kind:'clear_override',channelId:channel.id,rawModelId:id,field}:{kind:'set_override',channelId:channel.id,rawModelId:id,field,value:copy(value.value)})}
 if(JSON.stringify(before.naming)!==JSON.stringify(next.naming))intents.push({kind:'set_naming',channelId:channel.id,rawModelId:id,alias:next.naming.alias||null,displayName:next.naming.displayName||null})
 if(JSON.stringify(before.reference)!==JSON.stringify(next.reference)){if(!next.reference)edits.push({kind:'clear_manual_binding',channelId:channel.id,rawModelId:id});else{const ref=next.reference as Partial<RefV2>,revision=library.templates.find(t=>t.id===ref.templateId)?.revisions.find(r=>r.id===ref.revisionId);if(!revision||ref.digest&&ref.digest!==revision.digest)throw new Error('精确模版版本已变化，请重新选择。');edits.push({kind:'set_manual_binding',channelId:channel.id,rawModelId:id,reference:{templateId:ref.templateId!,revisionId:revision.id,digest:revision.digest}})}}
 return {edits,intents}
}
export function writeRow(channel:ChannelPolicy,id:string,fields:ModelRowFields){
 if(Object.values(fields.rules).every(value=>!value||value.mode==='inherit'))delete channel.overrides[id];else channel.overrides={...channel.overrides,[id]:copy(fields.rules)}
 if(fields.reference)channel.modelRefs={...channel.modelRefs,[id]:copy(fields.reference)}
 else delete channel.modelRefs[id]
 channel.naming??={};if(Object.keys(fields.naming).length)channel.naming={...channel.naming,[id]:copy(fields.naming)}
 else delete channel.naming[id]
}
