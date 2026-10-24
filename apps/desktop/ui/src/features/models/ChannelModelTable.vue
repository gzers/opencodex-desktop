<script setup lang="ts">
import {computed,onBeforeUnmount,reactive,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {useModalStore} from '@/app/modal/store'
import {reorderSlots} from './ordering'
import {usePointerDrag} from './pointerDrag'
import {copy,createModelRowsState,pendingModelRows,rowFields,rowPending,writeRow,type ModelRowFields,type ModelRowBuffer,type ModelRowsState} from './modelRows'
import {emptyRules,errorText,modelApi,type ChannelPolicy,type ModelPolicy,type ModelProjection,type Template,type ModelRules} from './api'
const props=defineProps<{channel:ChannelPolicy;policy?:ModelPolicy;models:ChannelPolicy['discovered'];templates:Template[];projection?:ModelProjection[];projectionBusy?:boolean;disabled?:boolean;filtered?:boolean;state?:ModelRowsState}>()
const host=ref<HTMLElement|null>(null)
const state=props.state??reactive(createModelRowsState()),modal=useModalStore()
const error=ref(''),validating=ref(false),dragged=ref(''),dragOver=ref(''),detailId=ref(''),detail=ref<ModelRowFields>(),detailEnabled=ref(false),detailBase=ref('')
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
function expand(id:string){buffer(id);state.expanded=state.expanded.includes(id)?state.expanded.filter(value=>value!==id):[id]}
function detailOpen(id:string){const value=buffer(id);detailId.value=id;detail.value=copy({rules:value.rules,naming:value.naming,reference:value.reference});detailEnabled.value=chosen(id);detailBase.value=JSON.stringify(value)}
async function acceptDetail(){const id=detailId.value;if(!detail.value)return;if(JSON.stringify(buffer(id))!==detailBase.value){error.value='行缓冲已变化，请关闭明细后重新核对';return}if(await accept(id,detail.value,detailEnabled.value))detailId.value=''}
function effective(id:string){return props.projectionBusy?undefined:projectionById.value.get(id)}
// 摘要文案与原型 .mp-model-meta 一致：推理 / 快速 前缀，不用自造的 Fast未知 写法。
function summary(id:string){const row=effective(id);if(!row)return{protocol:'协议待解析',context:'上下文未知',reasoning:'推理 未知',fast:'快速 未知'};const fields=row.rules;return{protocol:fields.protocol.mode==='custom'?fields.protocol.value:props.channel.connection.protocol,context:fields.contextWindow.mode==='custom'?fields.contextWindow.value.toLocaleString()+' tokens':'上下文未知',reasoning:'推理 '+(fields.noReasoning?.mode==='custom'&&fields.noReasoning.value?'不传推理参数':fields.reasoningEfforts.mode==='custom'?fields.reasoningEfforts.value.join(' / ')||'不显示档位':'未知'),fast:'快速 '+(fields.fast.mode==='custom'?(fields.fast.value?'支持':'不支持'):'未知')}}
function templateText(id:string){const ref=effective(id)?.templateRef;if(!ref)return'未采用模版匹配';return(props.templates.find(t=>t.id===ref.templateId)?.label??ref.templateId)+' · '+ref.revisionId.slice(0,4)}
const templateOptions=computed(()=>[{value:'',label:'沿用匹配结果'},...props.templates.flatMap(t=>t.revisions.map(r=>({value:JSON.stringify([t.id,r.id]),label:t.label+' · '+r.createdAt.slice(0,10)+' · '+r.id.slice(0,4)})))])
function reference(fields:ModelRowFields,value:string){if(!value)fields.reference=null;else{const [templateId,revisionId]=JSON.parse(value) as [string,string];fields.reference={templateId,revisionId}}}
function reorder(id:string,target:string){if(!canSort.value)return;const order=[...new Set([...props.channel.modelOrder,...props.channel.discovered.map(m=>m.id)])];props.channel.modelOrder=reorderSlots(order,props.models.map(m=>m.id),id,target)}
function move(id:string,delta:number){const index=props.models.findIndex(m=>m.id===id),target=props.models[index+delta];if(target)reorder(id,target.id)}
// 行排序用指针事件而非 HTML5 拖放：WKWebView 里 drop 常不触发，排序会静默失败并顺带框选文字。
const pointerDrag=usePointerDrag({
 rowSelector:'tr[data-model]',itemAttr:'data-model',
 enabled:()=>canSort.value,
 onReorder:(id,target)=>reorder(id,target),
 onActive:id=>{dragged.value=id??''},
 onOver:id=>{dragOver.value=id??''},
})
function dragStart(event:PointerEvent,id:string){dragOver.value='';pointerDrag.start(event,id)}
function sorted(value:string){if(!canSort.value||!value)return;const indexed=props.models.map((model,index)=>({model,index}));const compare=(a:typeof indexed[number],b:typeof indexed[number])=>{const text=(id:string)=>id;const context=(id:string)=>{const f=effective(id)?.rules.contextWindow;return f?.mode==='custom'?f.value:undefined};switch(value){case'nameDesc':return text(b.model.id).localeCompare(text(a.model.id));case'alias':return(props.channel.naming?.[a.model.id]?.alias||a.model.id).localeCompare(props.channel.naming?.[b.model.id]?.alias||b.model.id);case'protocol':return summary(a.model.id).protocol.localeCompare(summary(b.model.id).protocol);case'contextAsc':case'contextDesc':{const av=context(a.model.id),bv=context(b.model.id);if(av===undefined||bv===undefined)return av===undefined?(bv===undefined?0:1):-1;return value==='contextAsc'?av-bv:bv-av}case'enabled':return Number(chosen(b.model.id))-Number(chosen(a.model.id));default:return text(a.model.id).localeCompare(text(b.model.id))}};indexed.sort((a,b)=>compare(a,b)||a.index-b.index);const eligible=new Set(props.models.map(m=>m.id));const order=[...new Set([...props.channel.modelOrder,...props.channel.discovered.map(m=>m.id)])];let cursor=0;props.channel.modelOrder=order.map(id=>eligible.has(id)?indexed[cursor++]!.model.id:id)}
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
 <div class="model-toolbar-actions"><UiButton :disabled="disabled" @click="state.expanded=[...new Set([...state.expanded,...models.map(m=>m.id)])];models.forEach(m=>buffer(m.id))">全部展开</UiButton><UiButton @click="state.expanded=[]">全部收起</UiButton><span class="model-toolbar-sort"><UiSelectField model-value="" label="模型快捷排序" :disabled="!canSort" :options="[{value:'',label:'快捷排序'},...Object.entries({nameAsc:'名称 A → Z',nameDesc:'名称 Z → A',alias:'别名 A → Z',protocol:'协议分组',contextDesc:'上下文大 → 小',contextAsc:'上下文小 → 大',enabled:'启用优先'}).map(([value,label])=>({value,label}))]" @update:model-value="sorted" /></span></div>
 <p v-if="pending.length" role="status">{{ pending.length }} 项未确认编辑；请加入草稿或还原。<UiButton size="sm" variant="ghost" @click="state.expanded=[...new Set([...state.expanded,...pending])];host?.querySelector('[data-pending=true]')?.scrollIntoView({block:'center'})">定位未确认项</UiButton></p>
 <p v-if="error" class="model-error" role="alert">{{ error }}</p>
 <div v-if="selected.length" class="model-inline"><span>已选 {{ selected.length }} 项<span v-if="hiddenSelected.length"> · 筛选外 {{ hiddenSelected.length }} 项</span></span><UiButton size="sm" :disabled="disabled||validating" @click="batch('enable')">批量启用</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch('disable')">批量停用</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch()">批量编辑</UiButton><UiButton size="sm" :disabled="disabled||validating" @click="batch('inherit')">恢复字段继承</UiButton></div>
 <div class="model-table-scroll"><table class="model-table"><thead><tr><th><input type="checkbox" :checked="allSelected" :indeterminate="!allSelected&&models.some(m=>selected.includes(m.id))" aria-label="选择当前型号结果" :disabled="disabled||!models.length" @change="pickVisible" /></th><th>模型标识 / 别名与配置</th><th>启停</th><th>编辑与排序</th></tr></thead><tbody>
 <template v-for="(model,index) in models" :key="model.id"><tr :data-model="model.id" :data-pending="state.buffers[model.id]&&rowPending(state.buffers[model.id]!)" :class="{dragging:dragged===model.id,'drag-over':dragOver===model.id}" @pointerdown="dragStart($event,model.id)">
  <td><div class="model-row-pick"><input type="checkbox" :checked="selected.includes(model.id)" :aria-label="'操作选择 '+model.id" :disabled="disabled" @change="pick(model.id,($event.target as HTMLInputElement).checked)" /><span class="model-grip" :aria-hidden="true" title="拖动排序">↕</span></div></td>
  <td><div class="model-identity"><div class="model-identity-head"><strong :title="model.id" class="model-raw-id">{{ channel.label }}/{{ model.id }}</strong><label class="model-alias"><span class="sr-only">路由别名</span><input class="input" :value="state.buffers[model.id]?.naming.alias??channel.naming?.[model.id]?.alias??''" maxlength="128" :aria-label="model.id+'路由别名'" :disabled="disabled||validating" placeholder="路由别名（可选）" @input="buffer(model.id).naming.alias=($event.target as HTMLInputElement).value" @blur="acceptAlias(model.id)" /></label></div><small class="model-row-summary" :title="templateText(model.id)+' · '+summary(model.id).protocol"><span class="model-row-summary-line"><span>{{ summary(model.id).protocol }}</span><span>{{ summary(model.id).context }}</span></span><span class="model-row-summary-line"><span>{{ summary(model.id).reasoning }}</span><span>{{ summary(model.id).fast }}</span></span></small><span v-if="state.buffers[model.id]?.error" role="alert" class="model-error">{{ state.buffers[model.id]!.error }}</span></div></td>
  <td><div class="model-enabled-cell"><UiSwitch :model-value="chosen(model.id)" :label="model.id+'启用状态'" :disabled="disabled||validating" @update:model-value="toggleVisibility(model.id,$event)" /><small>{{ chosen(model.id)?'启用':'停用' }}</small></div></td>
  <td><div class="model-row-actions"><UiButton size="sm" variant="ghost" :aria-expanded="state.expanded.includes(model.id)" @click="expand(model.id)">{{ state.expanded.includes(model.id)?'收起编辑':'展开编辑' }}</UiButton><UiButton size="sm" variant="ghost" :disabled="disabled||validating" @click="detailOpen(model.id)">编辑明细</UiButton><span class="model-order"><button class="model-move" type="button" :aria-label="model.id+'上移'" :disabled="!canSort||index===0" @click="move(model.id,-1)">↑</button><button class="model-move" type="button" :aria-label="model.id+'下移'" :disabled="!canSort||index===models.length-1" @click="move(model.id,1)">↓</button></span></div></td>
 </tr><tr v-if="state.expanded.includes(model.id)"><td :colspan="4"><div class="model-row-editor" :aria-label="model.id+'行编辑'"><ModelRuleFields basic source-caption="已加入渠道草稿的有效值（不含本行待确认编辑）" :model-value="buffer(model.id).rules" :effective="effective(model.id)" :disabled="disabled||validating" @update:model-value="buffer(model.id).rules=$event" /><UiSelectField :model-value="buffer(model.id).reference?JSON.stringify([buffer(model.id).reference!.templateId,buffer(model.id).reference!.revisionId]):''" :options="templateOptions" :label="model.id+'采用版本'" :disabled="disabled||validating" @update:model-value="reference(buffer(model.id),$event)" /><div class="model-inline"><UiButton size="sm" :disabled="disabled||validating" @click="restore(model.id)">还原本行</UiButton><UiButton size="sm" variant="primary" :loading="validating" :disabled="disabled||!rowPending(buffer(model.id))" @click="accept(model.id,copy(buffer(model.id)))">加入草稿</UiButton></div></div></td></tr></template>
 </tbody></table></div><p v-if="!models.length">没有匹配的型号。</p>
 <UiDialog :open="!!detailId" title="模型编辑明细" wide :busy="validating" @close="detailId=''">
 <template v-if="detail"><p class="model-raw-id">{{ detailId }} · {{ templateText(detailId) }}</p><p class="model-display-name">{{ channel.label }}/{{ detailId }}<small>显示名称本轮固定为 渠道名/原始模型ID，不提供友好名称编辑（UI 规范 §29.1）。</small></p><label>路由别名<input v-model="detail.naming.alias" class="input" maxlength="128" :disabled="disabled||validating" /></label><div class="model-inline"><UiButton size="sm" @click="delete detail.naming.alias">沿用原别名</UiButton><UiButton size="sm" @click="detail.naming.alias=''">移除自定义别名</UiButton></div><label><input v-model="detailEnabled" type="checkbox" :disabled="disabled||validating" />启用此型号（关闭将使用明确白名单）</label><ModelRuleFields v-model="detail.rules" :effective="effective(detailId)" :disabled="disabled||validating" /><UiSelectField :model-value="detail.reference?JSON.stringify([detail.reference.templateId,detail.reference.revisionId]):''" :options="templateOptions" label="明细采用版本" :disabled="disabled||validating" @update:model-value="reference(detail!,$event)" /><p v-if="error||state.buffers[detailId]?.error" role="alert" class="model-error">{{ error||state.buffers[detailId]?.error }}</p></template>
 <template #actions><UiButton :disabled="validating" @click="detailId=''">取消</UiButton><UiButton variant="primary" :disabled="disabled" :loading="validating" @click="acceptDetail">加入草稿</UiButton></template></UiDialog>
 <UiDialog :open="batchOpen" title="确认批量型号修改" wide :busy="validating" @close="batchOpen=false"><p>{{ channel.label }} · {{ batchIds.length }} 项</p><details><summary>完整型号范围</summary><ul><li v-for="id in batchIds" :key="id" class="model-raw-id">{{ id }}</li></ul></details><UiSelectField v-model="batchMode" label="批量启停" :disabled="validating||disabled" :options="[{value:'keep',label:'保持启停'},{value:'enable',label:'启用'},{value:'disable',label:'停用'}]" /><UiSelectField v-model="batchNaming" label="批量别名" :disabled="validating||disabled" :options="[{value:'keep',label:'保持别名'},{value:'clear',label:'清空自定义别名'},{value:'generate',label:'规则生成：前缀＋序号'}]" /><label v-if="batchNaming==='generate'">别名前缀<input v-model="batchPrefix" class="input" :disabled="validating||disabled" /></label><div class="model-batch-fields"><label v-for="(name,key) in fieldNames" :key="key">{{ name }}<UiSelectField v-model="batchFields[key]" :label="'批量'+name+'处理'" :disabled="validating||disabled" :options="[{value:'keep',label:'不修改'},{value:'inherit',label:'恢复继承'},{value:'custom',label:'使用自定义值'}]" /></label></div><ModelRuleFields v-if="Object.values(batchFields).includes('custom')" v-model="batchRules" :disabled="disabled||validating" /><p>任一冲突整批不改；启停改变将使用明确白名单，原始ID和顺序保持。</p><p v-if="error" role="alert" class="model-error">{{ error }}</p><template #actions><UiButton :disabled="validating" @click="batchOpen=false">取消</UiButton><UiButton variant="primary" :disabled="disabled" :loading="validating" @click="applyBatch">整批加入草稿</UiButton></template></UiDialog>
</section>
</template>
<style scoped>
.channel-model-table> .model-table-scroll>table>thead>tr>th:first-child,.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:first-child{width:36px;min-width:36px;padding-inline:0;white-space:nowrap;text-align:center}
.model-row-pick{display:flex;align-items:center;justify-content:center;gap:var(--space-1);flex-wrap:nowrap}
.model-row-pick .model-grip{margin-inline-start:0}
.channel-model-table{display:grid;gap:var(--space-3);min-width:0;container-type:inline-size}.model-raw-id{display:block;white-space:normal;overflow-wrap:anywhere;font-family:var(--font-mono,monospace)}
/* WP-47：36 / 弹性 / 62 / 182；首行同轴，两行摘要留在下方。
   auto 布局会按内容重算列宽，必须固定布局才能让三列严格落在 36 / 62 / 182。 */
.channel-model-table> .model-table-scroll>table{table-layout:fixed;width:100%}
.channel-model-table> .model-table-scroll>table>thead>tr>th:nth-child(3),.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:nth-child(3){width:62px;min-width:62px}
.channel-model-table> .model-table-scroll>table>thead>tr>th:nth-child(4),.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:nth-child(4){width:182px;min-width:182px}
.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td{vertical-align:middle;padding-block:var(--space-3)}
/* 原型 .mp-model-row 为 padding:12px 0：启停与操作列不占横向内距，182px 列宽全部留给操作组。 */
.channel-model-table> .model-table-scroll>table>thead>tr>th:nth-child(3),.channel-model-table> .model-table-scroll>table>thead>tr>th:nth-child(4),.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:nth-child(3),.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:nth-child(4){padding-inline:0}
.model-identity{display:grid;gap:3px;align-content:center;min-width:0}
.model-identity-head{display:grid;grid-template-columns:minmax(0,1fr) minmax(90px,140px);align-items:center;gap:var(--space-2);min-width:0}
.model-alias{min-width:0}
.model-alias .input{width:100%;min-height:var(--control-sm);box-sizing:border-box;font-size:var(--text-caption)}
.model-display-name{color:var(--muted);font-size:var(--text-caption);overflow-wrap:anywhere}
.model-row-actions{display:flex;align-items:center;justify-content:flex-end;gap:var(--space-1);min-height:var(--control-sm);flex-wrap:nowrap}
.model-row-actions .btn{white-space:nowrap}
.model-order{display:grid;gap:1px;flex:none}
.model-move{display:grid;place-items:center;width:22px;min-height:18px;padding:0;border:0;border-radius:5px;background:transparent;color:var(--muted);font:inherit;font-size:var(--text-label);line-height:1;cursor:pointer}
.model-move:hover:not(:disabled){background:var(--raised);color:var(--text)}
.model-move:disabled{opacity:.35;cursor:default}
.model-move:focus-visible{outline:2px solid var(--accent);outline-offset:1px}
/* 启停列与原型一致：开关在上、状态小字在下，整列居中。 */
.model-enabled-cell{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:var(--space-1)}
.model-enabled-cell small{font-size:var(--text-caption);line-height:1.35;color:var(--muted);white-space:nowrap}
.model-toolbar-actions{display:flex;align-items:center;gap:var(--space-2);min-width:0;flex-wrap:nowrap}
.model-toolbar-actions .btn{white-space:nowrap}
.model-toolbar-sort{flex:none;min-width:130px}
/* WP-57：操作区与拖拽手柄不参与文本选择；输入、正文保持可复制。 */
.model-row-actions,.model-order,.model-toolbar-actions,.model-grip{-webkit-user-select:none;user-select:none}
/* 指针拖拽期间整体禁选：拖过一行不会顺手框住一路文字；松手即恢复，正文仍可复制。 */
:global(html.ocx-pointer-drag),:global(html.ocx-pointer-drag *){-webkit-user-select:none!important;user-select:none!important}
.model-row-pick .model-grip{cursor:grab}
:global(html.ocx-pointer-drag) .model-grip{cursor:grabbing}
.model-grip{display:inline-grid;place-items:center;width:16px;color:var(--muted);cursor:grab;font-size:var(--text-caption)}
.channel-model-table :deep(.model-table tbody>tr.dragging){opacity:.45}
.channel-model-table :deep(.model-table tbody>tr.drag-over){outline:2px solid var(--accent);outline-offset:-2px}
.model-identity input,.model-row-summary,.model-display-name{-webkit-user-select:text;user-select:text}
.model-row-editor{padding:var(--space-4);display:grid;gap:var(--space-3)}.model-batch-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-3)}small{display:block;color:var(--muted)}
/* 摘要与原型一致：两条分组行，行内小间距 4px、项间距 12px；模版名进 title，不再挤成一行。 */
.model-row-summary{display:grid;gap:4px;margin-top:3px;line-height:1.5}
.model-row-summary-line{display:flex;flex-wrap:wrap;align-items:baseline;gap:4px 12px;min-width:0}
.model-row-summary-line>span{min-width:0;overflow-wrap:anywhere}
label{display:grid;gap:var(--space-1)}
@container(max-width:680px){.model-identity-head{grid-template-columns:minmax(0,1fr)}.model-alias .input{max-width:220px}}
@container(max-width:540px){.model-row-actions{justify-content:flex-start;flex-wrap:wrap}.channel-model-table> .model-table-scroll>table>thead>tr>th:nth-child(4),.channel-model-table> .model-table-scroll>table>tbody>tr[data-model]>td:nth-child(4){width:auto;min-width:0}}
@media(max-width:650px){.model-batch-fields{grid-template-columns:minmax(0,1fr)}}
</style>
