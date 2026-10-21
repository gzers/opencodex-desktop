<script setup lang="ts">
import {computed,onBeforeUnmount,reactive,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {useModalStore} from '@/app/modal/store'
import {reorderSlots} from './ordering'
import {copy,createModelRowsState,pendingModelRows,rowFields,rowPending,writeRow,type ModelRowFields,type ModelRowBuffer,type ModelRowsState} from './modelRows'
import {emptyRules,errorText,modelApi,type ChannelPolicy,type ModelPolicy,type ModelProjection,type Template,type ModelRules} from './api'
const props=defineProps<{channel:ChannelPolicy;policy?:ModelPolicy;models:ChannelPolicy['discovered'];templates:Template[];projection?:ModelProjection[];projectionBusy?:boolean;disabled?:boolean;filtered?:boolean;state?:ModelRowsState}>()
const host=ref<HTMLElement|null>(null)
const state=props.state??reactive(createModelRowsState()),modal=useModalStore()
const error=ref(''),validating=ref(false),dragged=ref(''),detailId=ref(''),detail=ref<ModelRowFields>(),detailEnabled=ref(false),detailBase=ref('')
let sequence=0;onBeforeUnmount(()=>{sequence++})
const pending=computed(()=>pendingModelRows(state)),projectionById=computed(()=>new Map((props.projection??[]).map(row=>[row.modelId,row])))
const selected=computed(()=>state.selected.filter(id=>props.channel.discovered.some(m=>m.id===id)))
const hiddenSelected=computed(()=>selected.value.filter(id=>!props.models.some(m=>m.id===id)))
const allSelected=computed(()=>!!props.models.length&&props.models.every(m=>selected.value.includes(m.id)))
const canSort=computed(()=>!props.disabled&&!validating.value&&!props.filtered&&!pending.value.length)
const fieldNames:Record<keyof ModelRules,string>={protocol:'协议',contextWindow:'上下文窗口',inputModalities:'输入模态',reasoningEfforts:'可选推理档位',defaultEffort:'默认推理档位',fast:'Fast支持',pinnedEffort:'强制推理档位',effortMap:'推理标签映射',noReasoning:'不传推理参数',reasoningAuthoritative:'推理列表权威',fastWire:'Fast wire'}
function buffer(id:string):ModelRowBuffer{if(!state.buffers[id]){const fields=rowFields(props.channel,id);state.buffers[id]={...copy(fields),base:copy(fields),error:''}}return state.buffers[id]!}
watch(()=>JSON.stringify([props.channel.overrides,props.channel.naming,props.channel.modelRefs]),()=>{for(const id of Object.keys(state.buffers)){const value=state.buffers[id]!;if(!rowPending(value)){const fields=rowFields(props.channel,id);Object.assign(value,copy(fields),{base:copy(fields)})}}})
function chosen(id:string){return props.channel.selection.mode==='all'||props.channel.selection.mode==='whitelist'&&props.channel.selection.models.includes(id)}
function pick(id:string,checked:boolean){if(props.disabled)return;state.selected=checked?[...new Set([...state.selected,id])]:state.selected.filter(value=>value!==id)}
function pickVisible(){const ids=new Set(props.models.map(m=>m.id));state.selected=allSelected.value?state.selected.filter(id=>!ids.has(id)):[...new Set([...state.selected,...ids])]}
function setVisibility(channel:ChannelPolicy,ids:string[],enabled:boolean){
 const values=channel.selection.mode==='all'?channel.discovered.map(m=>m.id):channel.selection.mode==='whitelist'?[...channel.selection.models]:[]
 const next=enabled?[...new Set([...values,...ids])]:values.filter(id=>!ids.includes(id));channel.selection=next.length?{mode:'whitelist',models:next}:{mode:'none'}
}
function toggleVisibility(id:string,enabled:boolean){
 if(props.disabled||validating.value)return
 const candidate=copy(props.channel);setVisibility(candidate,[id],enabled)
 const apply=()=>{props.channel.selection=copy(candidate.selection)}
 if(props.channel.selection.mode==='all'||candidate.selection.mode==='none'){
  const baseline=JSON.stringify(props.channel);modal.open({title:candidate.selection.mode==='none'?'隐藏全部型号':'改为仅公开所选型号',body:candidate.selection.mode==='none'?'此渠道及未来发现型号将全部隐藏，其他配置与顺序保留。':'使用明确白名单保留其他已知型号；未来发现的新型号默认不公开。',confirmLabel:'确认修改',onConfirm:()=>{if(JSON.stringify(props.channel)!==baseline){error.value='选择确认已过期，请重新核对';return}apply()}})
 }else apply()
}
async function validate(candidate:ChannelPolicy,baseline:string):Promise<boolean>{
 if(props.disabled||validating.value)return false
 const request=++sequence;validating.value=true;error.value=''
 const policy=props.policy?copy(props.policy):{schemaVersion:1,channels:[copy(props.channel)],templateRefs:[]}
 const policyBaseline=JSON.stringify(policy);const index=policy.channels.findIndex(c=>c.id===candidate.id);if(index<0){validating.value=false;error.value='渠道目标已不存在';return false}policy.channels[index]=candidate
 try{await modelApi.project(policy);if(request!==sequence)return false;if(JSON.stringify(props.channel)!==baseline||props.policy&&JSON.stringify(props.policy)!==policyBaseline)throw {message:'配置基准已变化，请重新核对本行'};return true}
 catch(cause){if(request===sequence)error.value=errorText(cause);return false}
 finally{if(request===sequence)validating.value=false}
}
async function accept(id:string,proposed:ModelRowFields,enabled?:boolean){
 const fields=copy(proposed),value=buffer(id);if(fields.rules.reasoningEfforts.mode==='custom'&&!fields.rules.reasoningEfforts.value.length&&!(fields.rules.noReasoning?.mode==='custom'&&fields.rules.noReasoning.value)){value.error='自定义推理档位至少保留一项；隐藏档位请在编辑明细中明确设置';return false}if(JSON.stringify(rowFields(props.channel,id))!==JSON.stringify(value.base)){value.error='本行已在其他位置修改，请还原本行后重新编辑';return false}
 const candidate=copy(props.channel),baseline=JSON.stringify(props.channel),bufferBaseline=JSON.stringify(value);writeRow(candidate,id,fields);if(enabled!==undefined)setVisibility(candidate,[id],enabled)
 if(!await validate(candidate,baseline)){value.error=error.value;return false}
 if(JSON.stringify(value)!==bufferBaseline){value.error='校验期间行缓冲已变化，请重新确认';return false}
 writeRow(props.channel,id,fields);if(enabled!==undefined)props.channel.selection=candidate.selection
 const saved=rowFields(props.channel,id);Object.assign(value,copy(saved),{base:copy(saved),error:''});return true
}
async function acceptAlias(id:string){
 const value=buffer(id);if(value.naming.alias===value.base.naming.alias)return
 const attempted=value.naming.alias;const fields={...copy(value.base),naming:{...copy(value.base.naming),alias:attempted}}
 const candidate=copy(props.channel),baseline=JSON.stringify(props.channel);writeRow(candidate,id,fields)
 if(JSON.stringify(rowFields(props.channel,id))!==JSON.stringify(value.base)){value.error='本行已改变，请还原后编辑';return}
 if(!await validate(candidate,baseline)){if(value.naming.alias===attempted){if(value.base.naming.alias===undefined)delete value.naming.alias;else value.naming.alias=value.base.naming.alias}value.error=error.value;return}
 if(value.naming.alias!==attempted){value.error='别名校验期间输入已变化，请再次离开输入框确认';return}
 props.channel.naming??={};props.channel.naming={...props.channel.naming,[id]:copy(fields.naming)};value.base.naming=copy(fields.naming);value.error=''
}
function restore(id:string){const fields=rowFields(props.channel,id);state.buffers[id]={...copy(fields),base:copy(fields),error:''}}
function expand(id:string){buffer(id);state.expanded=state.expanded.includes(id)?state.expanded.filter(value=>value!==id):[...state.expanded,id]}
function detailOpen(id:string){const value=buffer(id);detailId.value=id;detail.value=copy({rules:value.rules,naming:value.naming,reference:value.reference});detailEnabled.value=chosen(id);detailBase.value=JSON.stringify(value)}
async function acceptDetail(){const id=detailId.value;if(!detail.value)return;if(JSON.stringify(buffer(id))!==detailBase.value){error.value='行缓冲已变化，请关闭明细后重新核对';return}if(await accept(id,detail.value,detailEnabled.value))detailId.value=''}
function effective(id:string){return props.projectionBusy?undefined:projectionById.value.get(id)}
function summary(id:string){const row=effective(id);if(!row)return{protocol:'协议待解析',context:'上下文未知',reasoning:'推理未知',fast:'Fast未知'};const fields=row.rules;return{protocol:fields.protocol.mode==='custom'?fields.protocol.value:props.channel.connection.protocol,context:fields.contextWindow.mode==='custom'?fields.contextWindow.value.toLocaleString()+' tokens':'上下文未知',reasoning:fields.noReasoning?.mode==='custom'&&fields.noReasoning.value?'不传推理参数':fields.reasoningEfforts.mode==='custom'?fields.reasoningEfforts.value.join(' / ')||'不显示档位':'推理未知',fast:fields.fast.mode==='custom'?(fields.fast.value?'Fast支持':'Fast不支持'):'Fast未知'}}
function templateText(id:string){const ref=effective(id)?.templateRef;if(!ref)return'未采用模版匹配';return(props.templates.find(t=>t.id===ref.templateId)?.label??ref.templateId)+' · '+ref.revisionId.slice(0,4)}
const templateOptions=computed(()=>[{value:'',label:'沿用匹配结果'},...props.templates.flatMap(t=>t.revisions.map(r=>({value:JSON.stringify([t.id,r.id]),label:t.label+' · '+r.createdAt.slice(0,10)+' · '+r.id.slice(0,4)})))])
function reference(fields:ModelRowFields,value:string){if(!value)fields.reference=null;else{const [templateId,revisionId]=JSON.parse(value) as [string,string];fields.reference={templateId,revisionId}}}
function reorder(id:string,target:string){if(!canSort.value)return;const order=[...new Set([...props.channel.modelOrder,...props.channel.discovered.map(m=>m.id)])];props.channel.modelOrder=reorderSlots(order,props.models.map(m=>m.id),id,target)}
function move(id:string,delta:number){const index=props.models.findIndex(m=>m.id===id),target=props.models[index+delta];if(target)reorder(id,target.id)}
function drag(event:DragEvent,id:string){if(!canSort.value){event.preventDefault();return}dragged.value=id;event.dataTransfer?.setData('text/plain',id)}
function sorted(value:string){if(!canSort.value||!value)return;const indexed=props.models.map((model,index)=>({model,index}));const compare=(a:typeof indexed[number],b:typeof indexed[number])=>{const text=(id:string)=>props.channel.naming?.[id]?.displayName||props.channel.discovered.find(m=>m.id===id)?.label||id;const context=(id:string)=>{const f=effective(id)?.rules.contextWindow;return f?.mode==='custom'?f.value:undefined};switch(value){case'nameDesc':return text(b.model.id).localeCompare(text(a.model.id));case'alias':return(props.channel.naming?.[a.model.id]?.alias||a.model.id).localeCompare(props.channel.naming?.[b.model.id]?.alias||b.model.id);case'protocol':return summary(a.model.id).protocol.localeCompare(summary(b.model.id).protocol);case'contextAsc':case'contextDesc':{const av=context(a.model.id),bv=context(b.model.id);if(av===undefined||bv===undefined)return av===undefined?(bv===undefined?0:1):-1;return value==='contextAsc'?av-bv:bv-av}case'enabled':return Number(chosen(b.model.id))-Number(chosen(a.model.id));default:return text(a.model.id).localeCompare(text(b.model.id))}};indexed.sort((a,b)=>compare(a,b)||a.index-b.index);const eligible=new Set(props.models.map(m=>m.id));const order=[...new Set([...props.channel.modelOrder,...props.channel.discovered.map(m=>m.id)])];let cursor=0;props.channel.modelOrder=order.map(id=>eligible.has(id)?indexed[cursor++]!.model.id:id)}
const batchOpen=ref(false),batchIds=ref<string[]>([]),batchBaseline=ref(''),batchMode=ref('keep'),batchNaming=ref('keep'),batchPrefix=ref('model-'),batchRules=ref(emptyRules()),batchFields=reactive<Record<string,string>>({})
function batch(action='edit'){
 if(pending.value.some(id=>selected.value.includes(id))){error.value='所选模型有未确认编辑，请先加入草稿或还原';return}
 batchIds.value=[...selected.value];if(!batchIds.value.length)return;batchBaseline.value=JSON.stringify(props.channel);batchMode.value=action==='enable'?'enable':action==='disable'?'disable':'keep';batchNaming.value='keep';batchRules.value=emptyRules();for(const key of Object.keys(fieldNames))batchFields[key]=action==='inherit'?'inherit':'keep';batchOpen.value=true;error.value=''
}
async function applyBatch(){
 if(JSON.stringify(props.channel)!==batchBaseline.value){error.value='批量确认已过期，请关闭后重新选择';return}
 const candidate=copy(props.channel)
 for(const [index,id] of batchIds.value.entries()){
  const fields=rowFields(candidate,id)
  for(const key of Object.keys(fieldNames) as (keyof ModelRules)[]){const mode=batchFields[key];if(mode==='custom'&&(!batchRules.value[key]||batchRules.value[key]?.mode==='inherit')){error.value='请先填写 '+fieldNames[key]+' 的自定义值';return}if(mode==='inherit')Object.assign(fields.rules,{[key]:{mode:'inherit'}});else if(mode==='custom')Object.assign(fields.rules,{[key]:copy(batchRules.value[key]??{mode:'inherit'})})}
  if(batchNaming.value==='clear')fields.naming.alias='';if(batchNaming.value==='generate')fields.naming.alias=batchPrefix.value+(index+1)
  writeRow(candidate,id,fields)
 }
 if(batchMode.value!=='keep')setVisibility(candidate,batchIds.value,batchMode.value==='enable')
 if(!await validate(candidate,batchBaseline.value))return
 props.channel.overrides=candidate.overrides;props.channel.naming=candidate.naming;props.channel.modelRefs=candidate.modelRefs;props.channel.selection=candidate.selection
 for(const id of batchIds.value)restore(id);batchOpen.value=false
}
</script>
<template>
<section ref="host" class="channel-model-table" :aria-busy="validating?'true':undefined">
 <div class="model-inline"><UiButton size="sm" :disabled="disabled" @click="state.expanded=[...new Set([...state.expanded,...models.map(m=>m.id)])];models.forEach(m=>buffer(m.id))">全部展开</UiButton><UiButton size="sm" @click="state.expanded=[]">全部收起</UiButton><UiSelectField model-value="" label="模型快捷排序" :disabled="!canSort" :options="[{value:'',label:'快捷排序'},...Object.entries({nameAsc:'名称 A → Z',nameDesc:'名称 Z → A',alias:'别名 A → Z',protocol:'协议分组',contextDesc:'上下文大 → 小',contextAsc:'上下文小 → 大',enabled:'启用优先'}).map(([value,label])=>({value,label}))]" @update:model-value="sorted" /></div>
 <p v-if="pending.length" role="status">{{ pending.length }} 项未确认编辑；请加入草稿或还原。<UiButton size="sm" variant="ghost" @click="state.expanded=[...new Set([...state.expanded,...pending])];host?.querySelector('[data-pending=true]')?.scrollIntoView({block:'center'})">定位未确认项</UiButton></p>
 <p v-if="error" class="model-error" role="alert">{{ error }}</p>
 <div v-if="selected.length" class="model-inline"><span>已选 {{ selected.length }} 项<span v-if="hiddenSelected.length"> · 筛选外 {{ hiddenSelected.length }} 项</span></span><UiButton size="sm" :disabled="disabled||validating" @click="batch('enable')">批量启用</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch('disable')">批量停用</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch()">批量编辑</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch('inherit')">恢复字段继承</UiButton></div>
 <div class="model-table-scroll"><table class="model-table"><thead><tr><th><input type="checkbox" :checked="allSelected" :indeterminate="!allSelected&&models.some(m=>selected.includes(m.id))" aria-label="选择当前型号结果" :disabled="disabled||!models.length" @change="pickVisible" /></th><th>模型标识 / 别名与配置</th><th>启停</th><th>编辑与排序</th></tr></thead><tbody>
 <template v-for="(model,index) in models" :key="model.id"><tr :data-model="model.id" :data-pending="state.buffers[model.id]&&rowPending(state.buffers[model.id]!)" @dragover.prevent @drop.prevent="reorder(dragged,model.id);dragged=''">
  <td><input type="checkbox" :checked="selected.includes(model.id)" :aria-label="'操作选择 '+model.id" :disabled="disabled" @change="pick(model.id,($event.target as HTMLInputElement).checked)" /><UiButton size="sm" variant="ghost" :draggable="canSort" :aria-label="model.id+'拖动排序'" :disabled="!canSort" @dragstart="drag($event,model.id)" @dragend="dragged=''">↕</UiButton></td>
  <td><strong :title="model.id" class="model-raw-id">{{ model.id }}</strong><span v-if="(channel.naming?.[model.id]?.displayName||model.label)!==model.id">{{ channel.naming?.[model.id]?.displayName||model.label }}</span><label><span class="sr-only">路由别名</span><input class="input" :value="state.buffers[model.id]?.naming.alias??channel.naming?.[model.id]?.alias??''" maxlength="128" :aria-label="model.id+'路由别名'" :disabled="disabled||validating" placeholder="路由别名（可选）" @input="buffer(model.id).naming.alias=($event.target as HTMLInputElement).value" @blur="acceptAlias(model.id)" /></label><small class="model-row-summary"><span>{{ summary(model.id).protocol }}</span><span>{{ summary(model.id).context }}</span><span>{{ summary(model.id).reasoning }}</span><span>{{ summary(model.id).fast }}</span><span>{{ templateText(model.id) }}</span></small><span v-if="state.buffers[model.id]?.error" role="alert" class="model-error">{{ state.buffers[model.id]!.error }}</span></td>
  <td><div class="model-inline"><UiSwitch :model-value="chosen(model.id)" :label="model.id+'启用状态'" :disabled="disabled||validating" @update:model-value="toggleVisibility(model.id,$event)" /><span>{{ chosen(model.id)?'启用':'停用' }}</span></div></td>
  <td><div class="model-inline"><UiButton size="sm" :aria-expanded="state.expanded.includes(model.id)" @click="expand(model.id)">{{ state.expanded.includes(model.id)?'收起编辑':'展开编辑' }}</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="detailOpen(model.id)">编辑明细</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'上移'" :disabled="!canSort||index===0" @click="move(model.id,-1)">↑</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'下移'" :disabled="!canSort||index===models.length-1" @click="move(model.id,1)">↓</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'置顶'" :disabled="!canSort||index===0" @click="reorder(model.id,models[0]!.id)">置顶</UiButton></div></td>
 </tr><tr v-if="state.expanded.includes(model.id)"><td :colspan="4"><div class="model-row-editor" :aria-label="model.id+'行编辑'"><ModelRuleFields basic source-caption="已加入渠道草稿的有效值（不含本行待确认编辑）" :model-value="buffer(model.id).rules" :effective="effective(model.id)" :disabled="disabled||validating" @update:model-value="buffer(model.id).rules=$event" /><UiSelectField :model-value="buffer(model.id).reference?JSON.stringify([buffer(model.id).reference!.templateId,buffer(model.id).reference!.revisionId]):''" :options="templateOptions" :label="model.id+'采用版本'" :disabled="disabled||validating" @update:model-value="reference(buffer(model.id),$event)" /><div class="model-inline"><UiButton size="sm" :disabled="disabled||validating" @click="restore(model.id)">还原本行</UiButton><UiButton size="sm" variant="primary" :loading="validating" :disabled="disabled||!rowPending(buffer(model.id))" @click="accept(model.id,copy(buffer(model.id)))">加入草稿</UiButton></div></div></td></tr></template>
 </tbody></table></div><p v-if="!models.length">没有匹配的型号。</p>
 <UiDialog :open="!!detailId" title="模型编辑明细" wide :busy="validating" @close="detailId=''">
 <template v-if="detail"><p class="model-raw-id">{{ detailId }} · {{ templateText(detailId) }}</p><label>显示名称<input :value="detail.naming.displayName??channel.discovered.find(m=>m.id===detailId)?.label??detailId" @input="detail.naming.displayName=($event.target as HTMLInputElement).value" class="input" maxlength="128" placeholder="保持上游显示名；留空清除" :disabled="disabled||validating" /></label><label>路由别名<input v-model="detail.naming.alias" class="input" maxlength="128" :disabled="disabled||validating" /></label><div class="model-inline"><UiButton size="sm" @click="delete detail.naming.alias">沿用原别名</UiButton><UiButton size="sm" @click="detail.naming.alias=''">移除自定义别名</UiButton></div><label><input v-model="detailEnabled" type="checkbox" :disabled="disabled||validating" />启用此型号（关闭将使用明确白名单）</label><ModelRuleFields v-model="detail.rules" :effective="effective(detailId)" :disabled="disabled||validating" /><UiSelectField :model-value="detail.reference?JSON.stringify([detail.reference.templateId,detail.reference.revisionId]):''" :options="templateOptions" label="明细采用版本" :disabled="disabled||validating" @update:model-value="reference(detail!,$event)" /><p v-if="error||state.buffers[detailId]?.error" role="alert" class="model-error">{{ error||state.buffers[detailId]?.error }}</p></template>
 <template #actions><UiButton :disabled="validating" @click="detailId=''">取消</UiButton><UiButton variant="primary" :disabled="disabled" :loading="validating" @click="acceptDetail">加入草稿</UiButton></template></UiDialog>
 <UiDialog :open="batchOpen" title="确认批量型号修改" wide :busy="validating" @close="batchOpen=false"><p>{{ channel.label }} · {{ batchIds.length }} 项</p><details><summary>完整型号范围</summary><ul><li v-for="id in batchIds" :key="id" class="model-raw-id">{{ id }}</li></ul></details><UiSelectField v-model="batchMode" label="批量启停" :disabled="validating||disabled" :options="[{value:'keep',label:'保持启停'},{value:'enable',label:'启用'},{value:'disable',label:'停用'}]" /><UiSelectField v-model="batchNaming" label="批量别名" :disabled="validating||disabled" :options="[{value:'keep',label:'保持别名'},{value:'clear',label:'清空自定义别名'},{value:'generate',label:'规则生成：前缀＋序号'}]" /><label v-if="batchNaming==='generate'">别名前缀<input v-model="batchPrefix" class="input" :disabled="validating||disabled" /></label><div class="model-batch-fields"><label v-for="(name,key) in fieldNames" :key="key">{{ name }}<UiSelectField v-model="batchFields[key]" :label="'批量'+name+'处理'" :disabled="validating||disabled" :options="[{value:'keep',label:'不修改'},{value:'inherit',label:'恢复继承'},{value:'custom',label:'使用自定义值'}]" /></label></div><ModelRuleFields v-if="Object.values(batchFields).includes('custom')" v-model="batchRules" :disabled="disabled||validating" /><p>任一冲突整批不改；启停改变将使用明确白名单，原始ID和顺序保持。</p><p v-if="error" role="alert" class="model-error">{{ error }}</p><template #actions><UiButton :disabled="validating" @click="batchOpen=false">取消</UiButton><UiButton variant="primary" :disabled="disabled" :loading="validating" @click="applyBatch">整批加入草稿</UiButton></template></UiDialog>
</section>
</template>
<style scoped>
.channel-model-table> .model-table-scroll>table>thead>tr>th:first-child,.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:first-child{width:64px;min-width:64px;white-space:nowrap}
.channel-model-table{display:grid;gap:var(--space-3);min-width:0}.model-raw-id{display:block;white-space:normal;overflow-wrap:anywhere;font-family:var(--font-mono,monospace)}td{vertical-align:middle}td:nth-child(2){min-width:260px}td:nth-child(3){min-width:120px}td:last-child{min-width:210px}.model-row-editor{padding:var(--space-4);display:grid;gap:var(--space-3)}.model-batch-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-3)}small{display:block;color:var(--muted)}.model-row-summary{display:flex;flex-wrap:wrap;gap:var(--space-1) var(--space-2);font-size:var(--text-caption)}label{display:grid;gap:var(--space-1)}@media(max-width:650px){.model-batch-fields{grid-template-columns:minmax(0,1fr)}}
</style>
