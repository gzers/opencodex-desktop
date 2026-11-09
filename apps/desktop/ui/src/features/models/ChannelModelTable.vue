<script setup lang="ts">
import {computed,onBeforeUnmount,ref,watch} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {useModelsV2Store} from './store-v2'
import {useModalStore} from '@/app/modal/store'
import {copy,own,createModelRowsState,pendingModelRows,rowPending,rowFieldsV2,rowChangesV2,type ModelRowsState,type ModelRowFields} from './modelRows'
import {reorderSlots} from './ordering'
import {emptyRulesV2,originLabelV2,type ChannelV2,type EffectiveModelV2,type ModelEditV2,type ModelSelectionV2,type ChannelIntentV2} from './api-v2'
import type {ChannelPolicy,ModelPolicy,ModelProjection} from './api'
const props=defineProps<{channel:ChannelV2|ChannelPolicy;effective?:readonly EffectiveModelV2[];disabled?:boolean;query?:string;selectedOnly?:boolean;familyFilter?:string;models?:readonly unknown[];templates?:readonly unknown[];policy?:ModelPolicy;projection?:ModelProjection[];projectionBusy?:boolean;filtered?:boolean;state?:ModelRowsState}>()
const emit=defineEmits<{edit:[value:ModelEditV2];selection:[value:ModelSelectionV2];order:[ids:string[]];intent:[value:ChannelIntentV2]}>()
const store=useModelsV2Store(),fallback=createModelRowsState(),rowBusy=ref<Record<string,boolean>>({}),error=ref('')
const state=computed(()=>props.state??(channel.value?store.rowsFor(channel.value.id):fallback))
const channel=computed(()=>('models'in props.channel?props.channel:null) as ChannelV2|null)
const batchOpen=ref(false),batchBusy=ref(false),batchMode=ref<'rules'|'disable'>('rules'),batchRules=ref(emptyRulesV2()),batchModes=ref<Record<string,'keep'|'inherit'|'custom'>>({}),batchError=ref('')
const detailId=ref(''),detailFields=ref<ModelRowFields|null>(null),detailBusy=ref(false),detailError=ref('')
let disposed=false,batchBaseline='',detailBaseline='',detailBuffer='',sequence=0
const batchFields=[['protocol','请求协议'],['contextWindow','上下文窗口'],['inputModalities','输入模态'],['reasoningEfforts','推理档位'],['defaultEffort','默认档位'],['fast','Fast'],['pinnedEffort','强制档'],['effortMap','档位映射'],['noReasoning','不传推理参数'],['reasoningAuthoritative','推理权威'],['fastWire','Fast wire']] as const
const selected=(id:string)=>channel.value?.selection.mode==='all'||channel.value?.selection.mode==='whitelist'&&channel.value.selection.models.includes(id)
const ordered=computed(()=>{const c=channel.value;if(!c)return [];const rank=new Map(c.modelOrder.map((id,index)=>[id,index]));return [...c.models].sort((a,b)=>(rank.get(a.rawModelId)??c.models.length)-(rank.get(b.rawModelId)??c.models.length))})
const effective=(id:string)=>props.effective?.find(row=>row.channelId===props.channel.id&&row.rawModelId===id)
const rows=computed(()=>ordered.value.filter(m=>(!props.models||props.models.some(value=>!!value&&typeof value==='object'&&('rawModelId'in value?value.rawModelId:'id'in value?value.id:null)===m.rawModelId))&&(!props.selectedOnly||selected(m.rawModelId))&&(!props.familyFilter||effective(m.rawModelId)?.binding?.reference.templateId===props.familyFilter)&&(!props.query||(m.rawModelId+' '+m.label+' '+(channel.value?.naming[m.rawModelId]?.alias??'')+' '+(channel.value?.naming[m.rawModelId]?.displayName??'')).toLowerCase().includes(props.query.toLowerCase()))))
const pending=computed(()=>pendingModelRows(state.value)),hiddenSelected=computed(()=>state.value.selected.filter(id=>!rows.value.some(m=>m.rawModelId===id)).length)
function actualFields(id:string){const fields=rowFieldsV2(channel.value!,id,store.explicitRules(props.channel.id,id));const binding=store.modelEdits.find(e=>e.channelId===props.channel.id&&e.rawModelId===id&&(e.kind==='set_manual_binding'||e.kind==='clear_manual_binding'));if(binding?.kind==='set_manual_binding')fields.reference=copy(binding.reference);else if(binding?.kind==='clear_manual_binding')fields.reference=null;return fields}
function buffer(id:string){let value=own(state.value.buffers,id);if(!value){const fields=channel.value?actualFields(id):{rules:emptyRulesV2(),naming:{},reference:null};value={...copy(fields),base:copy(fields),error:''};state.value.buffers={...state.value.buffers,[id]:value}}return value}
function fieldsOf(id:string):ModelRowFields {const b=buffer(id);return copy({rules:b.rules,naming:b.naming,reference:b.reference})}
function resetRow(id:string){if(rowBusy.value[id])return;const fields=actualFields(id);state.value.buffers={...state.value.buffers,[id]:{...copy(fields),base:copy(fields),error:''}}}
function baseline(){return JSON.stringify({channel:props.channel,context:store.fileContext(),state:state.value})}
function problem(cause:unknown){return cause instanceof Error?cause.message:typeof cause==='object'&&cause!==null&&'code'in cause?String(cause.code):'校验未完成，当前输入保持。'}
function emitChanges(edits:ModelEditV2[],intents:ChannelIntentV2[]){for(const edit of edits)emit('edit',edit);for(const intent of intents)emit('intent',intent)}
async function applyRow(id:string,fields:ModelRowFields,namingOnly?:'alias'|'displayName'){
 if(props.disabled||rowBusy.value[id]||!channel.value||!store.file)return false
 const owner=state.value,b=buffer(id),before=actualFields(id);b.error='';const input=JSON.stringify(fieldsOf(id)),guard=baseline(),seq=sequence
 rowBusy.value={...rowBusy.value,[id]:true}
 try{
  if(JSON.stringify(b.base)!==JSON.stringify(before))throw new Error('配置基准已变化，请先核对或还原此行；当前输入保持。')
  const changes=rowChangesV2(channel.value,id,before,fields,store.file.library)
  const result=await store.projectFile(changes.edits,changes.intents,channel.value.id)
  if(disposed||seq!==sequence||state.value!==owner||guard!==baseline()||input!==JSON.stringify(fieldsOf(id)))throw new Error('配置基准已变化，当前输入保持。')
  store.adoptProjection(result,changes.edits,changes.intents);emitChanges(changes.edits,changes.intents)
  if(namingOnly){const value=fields.naming[namingOnly];if(value===undefined)delete b.naming[namingOnly];else b.naming[namingOnly]=value;b.base.naming=copy(fields.naming)}else state.value.buffers={...state.value.buffers,[id]:{...copy(fields),base:copy(fields),error:''}}
  return true
 }catch(cause){if(!disposed&&seq===sequence&&state.value===owner){if(namingOnly&&input===JSON.stringify(fieldsOf(id))){b.naming={...b.naming};const old=before.naming[namingOnly];if(old===undefined)delete b.naming[namingOnly];else b.naming[namingOnly]=old}b.error=problem(cause)}return false}
 finally{if(!disposed)rowBusy.value={...rowBusy.value,[id]:false}}
}
function namingBlur(id:string,field:'alias'|'displayName'){if(buffer(id).naming[field]===buffer(id).base.naming[field])return;const fields=actualFields(id);fields.naming={...fields.naming,[field]:buffer(id).naming[field]};if(!fields.naming[field])delete fields.naming[field];void applyRow(id,fields,field)}
function referenceValue(fields:ModelRowFields){return fields.reference?fields.reference.templateId+'|'+fields.reference.revisionId:''}
function setReference(fields:ModelRowFields,value:string){if(!value){fields.reference=null;return}for(const t of store.file?.library.templates??[]){const r=t.revisions.find(r=>t.id+'|'+r.id===value);if(r){fields.reference={templateId:t.id,revisionId:r.id,digest:r.digest} as ModelRowFields['reference'];return}}}
const referenceOptions=computed(()=>[{value:'',label:'不作人工指定'},...(store.file?.library.templates??[]).flatMap(t=>t.revisions.map(r=>({value:t.id+'|'+r.id,label:t.label+' · '+r.id.slice(0,8)})))])
function expand(id:string){state.value.expanded=state.value.expanded.includes(id)?state.value.expanded.filter(v=>v!==id):[...state.value.expanded,id]}
function selectVisible(checked:boolean){const visible=new Set(rows.value.map(m=>m.rawModelId));state.value.selected=checked?[...new Set([...state.value.selected,...visible])]:state.value.selected.filter(id=>!visible.has(id))}
function toggle(id:string,checked:boolean){
 if(props.disabled||!channel.value)return
 const ids=ordered.value.filter(m=>selected(m.rawModelId)).map(m=>m.rawModelId),next={mode:'whitelist' as const,models:checked?[...new Set([...ids,id])]:ids.filter(value=>value!==id)}
 if(channel.value.selection.mode!=='all'){emit('selection',next);return}
 if(pending.value.length){error.value='请先处理未确认型号编辑。';return}
 const guard=baseline(),owner=state.value;useModalStore().open({title:'改为仅公开所选型号',body:'将从全部当前与以后型号改为明确白名单，保存配置后生效。',onConfirm:()=>{if(props.disabled||owner!==state.value||guard!==baseline()){error.value='范围确认已过期，当前输入保持。';return}emit('selection',next)}})
}
function toggleCheckbox(id:string,event:Event){const input=event.target as HTMLInputElement;toggle(id,input.checked);input.checked=!!selected(id)}
function move(id:string,delta:number){if(props.disabled||props.query||props.selectedOnly||props.filtered||props.familyFilter||!channel.value)return;const ids=ordered.value.map(m=>m.rawModelId),to=ids.indexOf(id)+delta;if(to<0||to>=ids.length)return;emit('order',reorderSlots(channel.value.modelOrder,ids,id,ids[to]!))}
function protocolText(id:string){const p=effective(id)?.protocol;return !p?'尚未核对':p.kind==='resolved'?p.value:p.kind==='missing'?'待配置':p.value+' · 暂不支持'}
function openDetail(id:string){if(props.disabled)return;detailFields.value=fieldsOf(id);detailId.value=id;detailError.value='';detailBaseline=baseline();detailBuffer=JSON.stringify(fieldsOf(id))}
function closeDetail(){if(!detailBusy.value){detailId.value='';detailFields.value=null}}
async function applyDetail(){if(detailBusy.value||!detailId.value||!detailFields.value)return;if(detailBaseline!==baseline()||detailBuffer!==JSON.stringify(fieldsOf(detailId.value))){detailError.value='配置基准已变化，当前输入保持。';return}detailBusy.value=true;const id=detailId.value;try{if(await applyRow(id,copy(detailFields.value))){detailId.value='';detailFields.value=null}else detailError.value=buffer(id).error}finally{detailBusy.value=false}}
function openBatch(mode:'rules'|'disable'='rules'){if(props.disabled||!state.value.selected.length)return;if(pending.value.length){error.value='请先处理未确认型号编辑，未应用批量修改。';return}batchMode.value=mode;batchModes.value=Object.fromEntries(batchFields.map(([key])=>[key,'keep']));batchRules.value=emptyRulesV2();batchError.value='';batchBaseline=baseline();batchOpen.value=true}
async function applyBatch(){
 if(props.disabled||batchBusy.value||!channel.value||!store.file)return
 batchBusy.value=true;batchError.value='';const owner=state.value,seq=sequence
 try{
  if(batchBaseline!==baseline())throw new Error('批量确认已过期，请重新选择；当前输入保持。')
  const edits:ModelEditV2[]=[],intents:ChannelIntentV2[]=[],ids=[...state.value.selected]
  if(ids.some(id=>!channel.value!.models.some(m=>m.rawModelId===id)))throw new Error('所选型号已变化，未应用本次批量。')
  if(batchMode.value==='disable'){const remaining=(channel.value.selection.mode==='all'?ordered.value.map(m=>m.rawModelId):channel.value.selection.mode==='whitelist'?channel.value.selection.models:[]).filter(id=>!ids.includes(id));intents.push({kind:'set_selection',channelId:channel.value.id,selection:remaining.length?{mode:'whitelist',models:remaining}:{mode:'none'}})}
  else for(const id of ids)for(const [field,label]of batchFields){const mode=batchModes.value[field]??'keep';if(mode==='keep')continue;if(mode==='inherit')edits.push({kind:'clear_override',channelId:channel.value.id,rawModelId:id,field});else{const value=batchRules.value[field];if(!value||value.mode!=='custom')throw new Error('请填写 '+label+' 的明确自定义值，未应用任何项。');edits.push({kind:'set_override',channelId:channel.value.id,rawModelId:id,field,value:copy(value.value)})}}
  if(!edits.length&&!intents.length)throw new Error('未选择需要修改的字段。')
  const result=await store.projectFile(edits,intents,channel.value.id)
  if(disposed||seq!==sequence||owner!==state.value||batchBaseline!==baseline())throw new Error('批量确认已过期，当前输入保持。')
  store.adoptProjection(result,edits,intents);emitChanges(edits,intents);for(const id of ids)resetRow(id);batchOpen.value=false
 }catch(cause){if(!disposed&&seq===sequence)batchError.value=problem(cause)}finally{batchBusy.value=false}
}
watch([()=>props.channel.id,()=>props.state],()=>{sequence++;detailId.value='';detailFields.value=null;batchOpen.value=false;error.value='';if(props.state)store.modelRows[props.channel.id]=props.state},{immediate:true})
onBeforeUnmount(()=>{disposed=true;sequence++})
</script>
<template>
 <div class="channel-model-table-v2">
  <p v-if="!channel" role="alert" class="model-error">渠道格式暂不支持，请重新读取新版渠道；未转换旧配置。</p>
  <template v-else>
   <p class="model-muted">勾选控制公开范围；操作选择用于批量配置。行编辑经后端只读校验后加入本地草稿，保存配置后生效。</p>
   <p v-if="pending.length" role="status">未确认编辑 {{pending.length}} 项；收起、筛选或返回保留输入，请加入草稿或还原后保存。</p><p v-if="error" role="alert" class="model-error">{{error}}</p>
   <div class="model-inline"><label><input type="checkbox" aria-label="选择当前型号结果" :checked="rows.length>0&&rows.every(m=>state.selected.includes(m.rawModelId))" :indeterminate="rows.some(m=>state.selected.includes(m.rawModelId))&&!rows.every(m=>state.selected.includes(m.rawModelId))" :disabled="disabled" @change="selectVisible(($event.target as HTMLInputElement).checked)" />操作选择</label><span>已选 {{state.selected.length}} 项<span v-if="hiddenSelected"> · 筛选外 {{hiddenSelected}} 项</span></span><UiButton size="sm" :disabled="disabled||!state.selected.length" @click="openBatch()">批量配置所选模型</UiButton><UiButton size="sm" :disabled="disabled||!state.selected.length" @click="openBatch('disable')">批量停用</UiButton><UiButton size="sm" :disabled="!state.selected.length" @click="state.selected=[]">清空操作选择</UiButton><UiButton size="sm" @click="state.expanded=[]">全部收起</UiButton></div>
   <div class="model-table-scroll"><table class="model-table"><thead><tr><th>操作选择</th><th>公开</th><th>模型</th><th>请求协议与来源</th><th>配置与顺序</th></tr></thead><tbody>
    <template v-for="(model,index) in rows" :key="model.rawModelId">
     <tr :data-model="model.rawModelId"><td><input v-model="state.selected" type="checkbox" :value="model.rawModelId" :aria-label="'操作选择 '+model.rawModelId" :disabled="disabled" /></td><td><input type="checkbox" :aria-label="'公开模型 '+model.rawModelId" :checked="selected(model.rawModelId)" :disabled="disabled" @change="toggleCheckbox(model.rawModelId,$event)" /></td><td><strong>{{model.label||model.rawModelId}}</strong><code>{{model.rawModelId}}</code><small>{{model.observed?'已观察到':'手工/已保存 ID，未要求先发现'}}</small><label>路由别名<input class="input" v-model="buffer(model.rawModelId).naming.alias" maxlength="128" :aria-label="model.rawModelId+'路由别名'" :disabled="disabled||rowBusy[model.rawModelId]||!store.channelCapabilities.naming" @blur="namingBlur(model.rawModelId,'alias')" /></label><p v-if="buffer(model.rawModelId).error" role="alert" class="model-error">{{buffer(model.rawModelId).error}}</p></td><td><span>{{protocolText(model.rawModelId)}}</span><small>{{originLabelV2(effective(model.rawModelId)?.fieldOrigins.protocol)}}</small><small v-if="effective(model.rawModelId)?.binding">{{effective(model.rawModelId)!.binding!.kind==='manual'?'人工指定模版':'读取自动匹配'}} · {{effective(model.rawModelId)!.binding!.reference.revisionId}}</small></td><td><div class="model-inline"><UiButton size="sm" :aria-expanded="state.expanded.includes(model.rawModelId)" @click="expand(model.rawModelId)">{{state.expanded.includes(model.rawModelId)?'收起编辑':'展开编辑'}}</UiButton><UiButton size="sm" :disabled="disabled" @click="openDetail(model.rawModelId)">编辑明细</UiButton><UiButton size="sm" :aria-label="model.rawModelId+'上移'" :disabled="disabled||!!query||selectedOnly||filtered||!!familyFilter||index===0" @click="move(model.rawModelId,-1)">上移</UiButton><UiButton size="sm" :aria-label="model.rawModelId+'下移'" :disabled="disabled||!!query||selectedOnly||filtered||!!familyFilter||index===rows.length-1" @click="move(model.rawModelId,1)">下移</UiButton></div></td></tr>
     <tr v-if="state.expanded.includes(model.rawModelId)"><td colspan="5"><h3>单模型人工设置 · {{model.rawModelId}}</h3><label>显示名称<input class="input" v-model="buffer(model.rawModelId).naming.displayName" :aria-label="model.rawModelId+'显示名称'" :disabled="disabled||rowBusy[model.rawModelId]" /></label><UiSelectField :model-value="referenceValue(buffer(model.rawModelId))" :label="model.rawModelId+'人工模版'" :options="referenceOptions" :disabled="disabled||rowBusy[model.rawModelId]" @update:model-value="setReference(buffer(model.rawModelId),$event)" /><ModelRuleFields v-model="buffer(model.rawModelId).rules" :effective-v2="effective(model.rawModelId)" source-caption="上次核对的有效值与来源" :disabled="disabled||rowBusy[model.rawModelId]" /><div class="model-inline"><UiButton :disabled="disabled||rowBusy[model.rawModelId]||!rowPending(buffer(model.rawModelId))" @click="applyRow(model.rawModelId,fieldsOf(model.rawModelId))">加入草稿</UiButton><UiButton :disabled="disabled||rowBusy[model.rawModelId]" @click="resetRow(model.rawModelId)">还原此行</UiButton></div><details><summary>实际来源证据与精确模版引用</summary><dl><template v-for="(field,key) in model.source.fields" :key="key"><dt>{{key}}</dt><dd>{{field.evidence.kind}} · {{field.evidence.pointer}} · {{field.evidence.sourceDigest}}</dd></template></dl><p v-if="model.binding">{{model.binding.reference.templateId}} / {{model.binding.reference.revisionId}} / {{model.binding.reference.digest}}</p><p v-else>未引用保存模版。</p></details></td></tr>
    </template>
   </tbody></table></div>
   <UiDialog :open="!!detailId" title="型号编辑明细" panel-class="model-detail-dialog" :busy="detailBusy" wide @close="closeDetail"><template v-if="detailFields"><p>{{detailId}} · 只编辑临时明细，取消保留原有行输入。</p><label>路由别名<input v-model="detailFields.naming.alias" class="input" maxlength="128" :disabled="detailBusy" /></label><label>显示名称<input v-model="detailFields.naming.displayName" class="input" :disabled="detailBusy" /></label><UiSelectField :model-value="referenceValue(detailFields)" label="明细人工模版" :options="referenceOptions" :disabled="detailBusy" @update:model-value="setReference(detailFields!,$event)" /><ModelRuleFields v-model="detailFields.rules" :effective-v2="effective(detailId)" source-caption="上次核对的有效值与来源" :disabled="detailBusy" /></template><p v-if="detailError" role="alert" class="model-error">{{detailError}}</p><template #actions="{close}"><UiButton :disabled="detailBusy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="detailBusy||disabled" @click="applyDetail">加入草稿</UiButton></template></UiDialog>
   <UiDialog :open="batchOpen" :title="batchMode==='disable'?'批量停用所选型号':'批量配置明确字段'" panel-class="model-batch-dialog" :busy="batchBusy" wide @close="!batchBusy&&(batchOpen=false)"><p>本次只修改以下 {{state.selected.length}} 个精确 ID；未选模型、原始 ID、来源与顺序保持。</p><p>{{state.selected.join('、')}}</p><template v-if="batchMode==='rules'"><div class="model-columns"><label v-for="[field,label] in batchFields" :key="field">{{label}}<UiSelectField :model-value="batchModes[field]??'keep'" :label="'批量'+label+'处理方式'" :disabled="disabled||batchBusy" :options="[{value:'keep',label:'保持原值'},{value:'inherit',label:'恢复继承'},{value:'custom',label:'明确自定义'}]" @update:model-value="batchModes[field]=$event as 'keep'|'inherit'|'custom'" /></label></div><ModelRuleFields v-model="batchRules" :disabled="disabled||batchBusy" /></template><p v-else>从当前公开范围移除所选型号，配置及身份保持。全部公开会改为明确白名单。</p><p v-if="batchError" role="alert" class="model-error">{{batchError}}</p><template #actions="{close}"><UiButton :disabled="batchBusy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="disabled||batchBusy" @click="applyBatch">整批加入草稿</UiButton></template></UiDialog>
   <p v-if="!rows.length" class="model-muted">当前筛选下没有模型；不会清空原始 ID 或配置。</p>
  </template>
 </div>
</template>
<style scoped>
.channel-model-table-v2{min-width:0}.channel-model-table-v2 .model-table-scroll{overflow:auto}.channel-model-table-v2 table{width:100%;min-width:580px}.channel-model-table-v2 th,.channel-model-table-v2 td{padding:10px;text-align:left;vertical-align:top;border-bottom:1px solid var(--border)}.channel-model-table-v2 code,.channel-model-table-v2 small{display:block;overflow-wrap:anywhere;color:var(--muted)}.channel-model-table-v2 dd{margin:0;overflow-wrap:anywhere}.channel-model-table-v2 h3{font-size:var(--text-body)}.channel-model-table-v2 input:not([type=checkbox]){min-width:0;width:100%;box-sizing:border-box}
</style>
