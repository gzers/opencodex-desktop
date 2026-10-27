<script setup lang="ts">
import { computed, reactive, ref, watch, onBeforeUnmount, toRef } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiIconButton from '@/components/ui/UiIconButton.vue'
import DetailWorkspace from '@/components/layout/DetailWorkspace.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import UiStatus from '@/components/ui/UiStatus.vue'
import type {StatusTone} from '@/lib/saveStates'
import ModelRuleFields from './ModelRuleFields.vue'
import ChannelModelTable from './ChannelModelTable.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {createModelRowsState,pendingModelRows,type ModelRowsState} from './modelRows'
import NodePicker from './NodePicker.vue'
import {useModalStore} from '@/app/modal/store'
import {protectedChannel} from './channelActions'
import { nodeMeasurementError, useNodeMeasurements } from './useNodeMeasurements'
import {useModelsStore} from './store'
import { emptyRules, modelApi, errorText, type ChannelPolicy, type Template, type TemplateRef, type ModelProjection, type ModelPolicy } from './api'
const props = defineProps<{ channel: ChannelPolicy; templates: Template[]; busy?: boolean;existing?:boolean;globalRefs?:TemplateRef[];projection?:ModelProjection[];projectionBusy?:boolean;projectionError?:string;policy?:ModelPolicy;rowState?:ModelRowsState;navigation?:{collapsed:boolean;activeId:string};channels?:ChannelPolicy[];saveDisabled?:boolean;dirty?:boolean;saveStatus?:string;saveTone?:StatusTone;activeCount?:number;viewState?:{search:string;selectedOnly:boolean;family:string} }>()
const emit = defineEmits<{ close: [];switch:[string];save:[] }>()
const nav=props.navigation??reactive({collapsed:false,activeId:'channel-config'})
const sections=[{id:'channel-config',label:'渠道配置'},{id:'channel-nodes',label:'服务节点'},{id:'channel-defaults',label:'模型默认'},{id:'channel-models',label:'模型列表'}]
const defaultsOpen=ref(false);
const apiKey=ref('');const discovering=ref(false);const discoveryError=ref('');let generation=0
const models=useModelsStore()
const keyOriginal=ref(''),keyVisible=ref(false),keyLoading=ref(false),keySource=ref(''),keyError=ref('')
const keyExplicit=ref(false)
let keySequence=0
watch([()=>props.channel.id,()=>props.channel.connection.providerId,()=>models.snapshot?.policyDigest],async()=>{
 const sequence=++keySequence,channelId=props.channel.id
 const restoreEdit=(savedValue='')=>{const edit=models.credentialEdits[channelId];apiKey.value=edit?(edit.value??''):savedValue;keyExplicit.value=edit?.action==='replace'}
 keyVisible.value=false;keyError.value='';keySource.value='';keyLoading.value=true;keyOriginal.value='';restoreEdit()
 try{
  const saved=await modelApi.credential(props.channel.connection.providerId)
  if(sequence!==keySequence)return
  keyOriginal.value=saved.value;keySource.value=saved.source
  restoreEdit(saved.value)
  if(saved.configured&&!saved.viewable)keyError.value='已配置；当前凭据来源暂无法读取明文，未修改时保留原值。'
 }catch(cause){
  if(sequence===keySequence){restoreEdit();if(props.existing)keyError.value=errorText(cause)}
 }finally{if(sequence===keySequence)keyLoading.value=false}
},{immediate:true})
function keyChanged(){keyExplicit.value=true;models.editCredential(props.channel.id,apiKey.value,keyOriginal.value)}
function clearKey(){apiKey.value='';keyExplicit.value=false;keyVisible.value=false;models.clearCredential(props.channel.id)}
// Displaying a stored secret is not new consent to send it to an edited URL.
// Untouched credentials use the backend's file/provider origin binding.
const probeKey=computed(()=>authMode.value==='none'?'':keyExplicit.value?apiKey.value:'')
const keyIntent=computed(()=>models.credentialEdits[props.channel.id]?.action==='clear'?'保存后清空 Key':models.credentialEdits[props.channel.id]?'保存后替换 Key':'保持原 Key')
const authOptions=[{value:'key',label:'API Key'},{value:'none',label:'无需认证'}]
const authMode=computed({get:()=>['none','local'].includes(props.channel.connection.authMode??'')?'none':'key',set:(value:'key'|'none')=>{props.channel.connection.authMode=value==='none'?'local':'key'}})
watch(()=>[props.channel.id,props.channel.connection.protocol,props.channel.connection.activeNodeId,props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId)?.url],()=>{generation++;discovering.value=false;discoveryError.value=''})
onBeforeUnmount(()=>{generation++;keySequence++;apiKey.value='';keyOriginal.value=''})
async function discover(){
 const node=props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId);if(!node)return
 const sequence=++generation;discovering.value=true;discoveryError.value=''
 try{const result=await modelApi.discover({protocol:props.channel.connection.protocol,url:node.url,apiKey:probeKey.value||null,credentialRef:props.channel.connection.credentialRef,allowPrivateNetwork:props.channel.connection.allowPrivateNetwork,providerId:props.channel.connection.providerId})
  if(sequence!==generation){if(result.credentialRef&&result.credentialRef!==props.channel.connection.credentialRef)void modelApi.forgetCredential(result.credentialRef);return}
  const previousModels=props.channel.discovered
  props.channel.discovered=result.models.map(model=>{const previous=props.channel.discovered.find(item=>item.id===model.id);return previous?{...model,label:previous.label}:model})
  for(const previous of previousModels)if(chosen(previous.id)&&!props.channel.discovered.some(m=>m.id===previous.id))props.channel.discovered.push(previous)
  for(const model of result.models)if(!props.channel.modelOrder.includes(model.id))props.channel.modelOrder.push(model.id)
 }catch(cause){if(sequence===generation)discoveryError.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
const selectionMode = computed(() => props.channel.selection.mode)
const measurement=reactive(useNodeMeasurements(()=>props.channel,()=>probeKey.value))
/** 已选型号数；与原型 selected(p).length 同口径（all 模式取全部发现项）。 */
const selectedCount=computed(()=>props.channel.selection.mode==='all'?props.channel.discovered.length:props.channel.selection.mode==='whitelist'?props.channel.selection.models.length:0)
/**
 * 详情头部状态徽标（原型 model-config.js:449 channelHeadingStatusHTML）：
 * 当前渠道保存状态 + 启停 + 连接可用性 + 清单获取，末尾单独一个「已选 N / 发现 M」文本。
 * 连接可用性只有节点检测结果这一个来源，未检测即「未验证」（原型同口径）。
 */
const connectionState=computed(():{label:string;tone:StatusTone}=>{
 if(props.channel.connection.authMode==='none')return {label:'无需认证',tone:'muted'}
 const results=Object.values(measurement.results)
 if(!results.length)return {label:'未验证',tone:'muted'}
 const failed=results.find(result=>result.error)
 if(failed)return {label:nodeMeasurementError(failed.error??''),tone:'bad'}
 return {label:'连接检查通过',tone:'running'}
})
const discoveryState=computed(():{label:string;tone:StatusTone}|null=>discovering.value?{label:'正在获取清单',tone:'starting'}:discoveryError.value?{label:'获取清单失败',tone:'bad'}:null)

function mode(value:string){
 if(props.busy||pendingModelRows(rowsState.value).length){discoveryError.value='请先处理未确认型号编辑';return}
 if(value===props.channel.selection.mode)return
 const channel=props.channel,state=rowsState.value,baseline=JSON.stringify(channel),current=channel.selection
 const next=value==='whitelist'?{mode:'whitelist' as const,models:current.mode==='all'?props.channel.discovered.map(m=>m.id):[]}: {mode:value==='all'?'all' as const:'none' as const}
 useModalStore().open({title:'调整模型公开范围',body:value==='all'?'全部当前和未来发现型号将公开，保存时仍核对全局预算。':value==='none'?'当前和未来型号全部隐藏；已有规则和顺序保留。':'改为明确白名单，保留当前已知的启用项，未来新型号不自动公开。',confirmLabel:'确认范围',onConfirm:()=>{if(props.busy||props.channel!==channel||rowsState.value!==state||pendingModelRows(rowsState.value).length||JSON.stringify(props.channel)!==baseline){discoveryError.value='范围确认已过期，请重新核对';return}channel.selection=next}})
}
function chosen(id:string){return props.channel.selection.mode==='all'||props.channel.selection.mode==='whitelist'&&props.channel.selection.models.includes(id)}
function addNode(){props.channel.connection.nodes.push({id:crypto.randomUUID(),name:'',url:''})}
const manualOpen=ref(false),manualId=ref(''),manualError=ref('')
function addModel(){manualId.value='';manualError.value='';manualOpen.value=true}
function addExactModel(){const id=manualId.value;if(!id||id.trim()!==id||id.length>2048||/[\u0000-\u001f]/.test(id)){manualError.value='请填写不含首尾空白或控制字符的完整原始ID';return}if(props.channel.discovered.some(m=>m.id===id)){manualError.value='此原始ID已经存在';return}if(props.channel.selection.mode==='all')props.channel.selection={mode:'whitelist',models:props.channel.discovered.map(m=>m.id)};props.channel.discovered.push({id,label:id,rules:emptyRules()});if(!props.channel.modelOrder.includes(id))props.channel.modelOrder.push(id);manualOpen.value=false}
const localRows=new Map<string,ModelRowsState>()
const rowsState=computed(()=>{
 if(props.rowState)return props.rowState
 if(!localRows.has(props.channel.id))localRows.set(props.channel.id,reactive(createModelRowsState()))
 return localRows.get(props.channel.id)!
})
// ChannelModelTable captures its row buffer on mount; replace it when ownership changes.
const rowsVersion=ref(0)
watch(rowsState,()=>{rowsVersion.value++},{flush:'sync'})
const orderedModels=computed(()=>{const rank=new Map(props.channel.modelOrder.map((id,index)=>[id,index]));return [...props.channel.discovered].sort((a,b)=>(rank.get(a.id)??Number.MAX_SAFE_INTEGER)-(rank.get(b.id)??Number.MAX_SAFE_INTEGER))})
const filters=props.viewState??reactive({search:'',selectedOnly:false,family:'all'})
const modelSearch=toRef(filters,'search');const selectedOnly=toRef(filters,'selectedOnly');const familyFilter=toRef(filters,'family')
const adoptedFamilies=computed(()=>props.templates.filter(t=>(props.globalRefs??[]).some(r=>r.templateId===t.id)||props.channel.templateRefs.some(r=>r.templateId===t.id)||Object.values(props.channel.modelRefs).some(r=>r.templateId===t.id)||t.id in props.channel.familyOverrides))
const projectionById=computed(()=>new Map((props.projection??[]).map(row=>[row.modelId,row])))
const familyOptions=computed(()=>[{value:'all',label:'全部族'},{value:'unmatched',label:'未采用模版匹配'},...adoptedFamilies.value.map(t=>({value:t.id,label:t.label}))])
const visibleModels=computed(()=>orderedModels.value.filter(m=>(familyFilter.value==='all'||(projectionById.value.has(m.id)&&(familyFilter.value==='unmatched'?!projectionById.value.get(m.id)?.templateRef:projectionById.value.get(m.id)?.templateRef?.templateId===familyFilter.value)))&&(!selectedOnly.value||chosen(m.id))&&(m.id+' '+m.label+' '+(props.channel.naming?.[m.id]?.alias??'')+' '+(props.channel.naming?.[m.id]?.displayName??'')).toLocaleLowerCase().includes(modelSearch.value.trim().toLocaleLowerCase())))
function adopt(t: Template, revision: string) {
  props.channel.templateRefs = props.channel.templateRefs.filter(r => r.templateId !== t.id)
  if (revision) props.channel.templateRefs.push({ templateId: t.id, revisionId: revision })
}
</script>
<template>
  <DetailWorkspace v-model:collapsed="nav.collapsed" v-model:active-id="nav.activeId" label="渠道导航" :sections="sections">
    <!--
      详情头部（原型 model-config.js:868 .mp-editor-head：32px 返回键 / 弹性标题 / 下拉）：
      标题下挂一行状态徽标，顺序与 channelHeadingStatusHTML 一致——当前渠道保存状态、启停、
      连接可用性、清单获取，最后是「已选 N / 发现 M」纯文本。旧版把四项压成一个计数标签，
      用户看不出保存状态与连接状态，故拆回四枚徽标 + 计数。
    -->
    <template #header><span class="channel-detail-back"><UiButton size="sm" variant="ghost" aria-label="返回渠道列表" title="返回渠道列表" @click="emit('close')"><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m14 6-6 6 6 6" /></svg></UiButton></span><div class="channel-detail-heading"><strong class="channel-detail-name">{{ channel.label || '新渠道' }}</strong><div class="channel-detail-status" role="status"><UiStatus :tone="saveTone ?? 'muted'" :label="saveStatus" data-status-scope="current-channel" :hint="'当前渠道的保存状态'" /><UiStatus v-if="protectedChannel(channel)" tone="muted" label="官方管理" data-channel-managed /><template v-else><UiStatus :tone="channel.connection.enabled?'running':'muted'" :label="channel.connection.enabled?'已启用':'已停用'" :data-channel-enabled="channel.connection.enabled" /><UiStatus :tone="connectionState.tone" :label="connectionState.label" :data-channel-availability="connectionState.label" /></template><UiStatus v-if="discoveryState" :tone="discoveryState.tone" :label="discoveryState.label" /><span class="channel-detail-count">已选 {{ selectedCount }} / 发现 {{ channel.discovered.length }}</span></div></div><UiSelectField v-if="channels?.length" :model-value="channel.id" :options="channels.map(c=>({value:c.id,label:c.label||'未命名渠道'}))" label="切换渠道" :disabled="busy" @update:model-value="emit('switch',$event)" /></template>
    <template #primary="{collapsed}"><UiButton size="sm" variant="primary" :disabled="busy||saveDisabled||dirty===false" title="保存当前渠道，成功后返回渠道列表；其它渠道修改保留" data-save-scope="current-channel" aria-label="保存渠道草稿" @click="emit('save')"><svg v-if="collapsed" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 3h13l3 3v15H4zM8 3v6h8V3M8 21v-8h8v8"/></svg><template v-else>保存</template></UiButton></template>
    <div class="model-form">
    <section id="channel-config" tabindex="-1"><h2>渠道配置</h2><div class="channel-config-grid"><label>渠道名称<input class="input" v-model="channel.label" maxlength="80" :disabled="busy" /></label><label>渠道标识<input class="input" :value="channel.connection.providerId" readonly aria-label="渠道标识" /><span class="model-muted">标识固定，名称可修改。</span></label><label>默认接口协议<UiSelectField :model-value="channel.connection.protocol" label="默认接口协议" :disabled="busy" :options="[{value:'openai-responses',label:'openai-responses'},{value:'openai-chat',label:'openai-chat'},{value:'anthropic',label:'anthropic'},{value:'ollama-native',label:'ollama-native'}]" @update:model-value="channel.connection.protocol=$event" /></label><label>认证方式<UiSelectField :model-value="authMode" label="认证方式" :options="authOptions" :disabled="busy" @update:model-value="authMode=$event as 'key'|'none'" /></label><label v-if="authMode==='key'" class="channel-key">API Key<span class="channel-key-input"><input class="input" :type="keyVisible?'text':'password'" autocomplete="off" v-model="apiKey" aria-label="API Key" :placeholder="keyLoading?'正在读取…':'API Key'" :disabled="busy||keyLoading||protectedChannel(channel)" @input="keyChanged" /><UiIconButton :label="keyVisible?'隐藏 API Key':'显示 API Key'" :disabled="keyLoading||!apiKey" :aria-pressed="keyVisible" @click="keyVisible=!keyVisible"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/><path v-if="keyVisible" d="M3 3l18 18"/></svg></UiIconButton><UiButton size="sm" :disabled="busy||keyLoading||protectedChannel(channel)" @click="clearKey">清空 Key</UiButton></span><span class="model-muted">{{ keyIntent }}</span><span v-if="keySource" class="model-muted">{{ keySource }}</span><span v-if="keyError" class="model-muted" role="status">{{ keyError }}</span></label></div>
      <p class="model-muted">认证与 Key 由全部服务节点共用；地址在服务节点中维护。</p>
      <label class="model-inline"><input v-model="channel.connection.enabled" type="checkbox" :disabled="busy" />启用渠道</label>
    </section>
    <section id="channel-nodes" tabindex="-1" aria-label="服务节点"><NodePicker v-model:nodes="channel.connection.nodes" v-model:active-id="channel.connection.activeNodeId" :disabled="busy" :measuring="!!measurement.run" :measurement-disabled="discovering" :pending="measurement.pending" :results="measurement.results" :error="measurement.error" :stopped="measurement.stopped" @measure="measurement.measure" @stop="measurement.stop" @add="addNode" /></section>
    <section id="channel-defaults" tabindex="-1"><div class="model-field-title"><h2>模型默认</h2><UiButton size="sm" :aria-expanded="defaultsOpen" @click="defaultsOpen=!defaultsOpen">{{ defaultsOpen?'收起默认配置':'编辑默认配置' }}</UiButton></div><p class="model-muted">作用于本渠道全部型号；未覆盖字段继续继承模版或上游声明。</p><dl v-if="!defaultsOpen" class="channel-default-summary"><template v-for="item in [['上下文窗口',channel.defaults.contextWindow],['输入模态',channel.defaults.inputModalities],['可选推理档位',channel.defaults.reasoningEfforts],['默认推理档位',channel.defaults.defaultEffort],['Fast支持',channel.defaults.fast]]" :key="String(item[0])"><dt>{{ item[0] }}</dt><dd>{{ (item[1] as {mode:string;value?:unknown}).mode==='inherit'?'继承':(item[1] as {value?:unknown}).value }}</dd></template></dl><div v-show="defaultsOpen"><ModelRuleFields v-model="channel.defaults" hide-protocol :disabled="busy" />
    <h3>采用模版</h3><label v-for="t in templates" :key="t.id">{{ t.label }}<UiSelectField :model-value="channel.templateRefs.find(r => r.templateId === t.id)?.revisionId ?? ''" :label="`采用${t.label}版本`" :options="[{ value: '', label: '不设置渠道引用' }, ...t.revisions.map(r => ({ value: r.id, label: `${r.createdAt} · ${r.id.slice(0,4)}` }))]" :disabled="busy" @update:model-value="adopt(t, $event)" /></label>
    <div v-if="adoptedFamilies.length"><h3>渠道族覆盖</h3><details v-for="family in adoptedFamilies" :key="family.id"><summary>{{ family.label }}</summary><ModelRuleFields :model-value="channel.familyOverrides[family.id]??emptyRules()" :disabled="busy" @update:model-value="channel.familyOverrides[family.id]=$event" /><UiButton size="sm" variant="ghost" :disabled="busy||!channel.familyOverrides[family.id]" @click="delete channel.familyOverrides[family.id]">恢复此族继承</UiButton></details></div></div></section>
    <section id="channel-models" tabindex="-1"><div class="model-field-title"><h2>模型列表</h2><div class="model-inline"><UiButton :disabled="busy||discovering" :loading="discovering" @click="discover">获取模型列表</UiButton><UiButton v-if="discovering" @click="generation++; discovering=false">停止等待</UiButton><UiButton :disabled="busy" @click="addModel">手动添加ID</UiButton></div></div><p v-if="discoveryError" class="model-error" role="alert">{{ discoveryError }}</p><div class="model-toolbar"><UiSelectField :model-value="selectionMode" label="模型公开范围" :options="[{ value: 'none', label: '全部隐藏' }, { value: 'all', label: '全部模型（含未来发现）' }, { value: 'whitelist', label: '仅所选模型' }]" :disabled="busy" @update:model-value="mode" />
      <input v-model="modelSearch" class="input" type="search" aria-label="搜索渠道内型号" placeholder="搜索型号或名称" /><UiSelectField v-model="familyFilter" :options="familyOptions" label="渠道内型号族筛选" :disabled="projectionBusy||!!projectionError" /><label class="ui-inline-check"><input v-model="selectedOnly" type="checkbox" />仅启用</label><span class="model-muted">{{ visibleModels.length }} / {{ orderedModels.length }} 项</span></div><p v-if="modelSearch.trim()||familyFilter!=='all'||selectedOnly" class="model-muted">清除全部搜索和筛选后可调整型号顺序。</p>
      <p v-if="projectionBusy" role="status" class="model-muted">正在解析当前草稿，筛选暂用上次结果…</p><p v-else-if="projectionError" role="alert" class="model-error">{{ projectionError }}；有效值暂不可确定，筛选暂用上次结果。</p>
      <ChannelModelTable :key="channel.id+':'+rowsVersion" :channel="channel" :policy="policy" :models="visibleModels" :templates="templates" :projection="projection" :projection-busy="projectionBusy||!!projectionError" :disabled="busy||discovering" :filtered="!!modelSearch.trim()||familyFilter!=='all'||selectedOnly" :state="rowsState" />
      <p class="model-footer-note">{{ channel.connection.enabled?'复选框仅用于批量选择；启停独立控制。别名不改变上游 ID，行编辑加入草稿后统一保存。':'渠道已停用，模型配置仍可编辑；启用渠道后恢复计划选择器。' }}</p>
      <UiDialog :open="manualOpen" title="手动添加精确型号" :busy="busy" @close="manualOpen=false"><label>原始模型ID<input v-model="manualId" class="input" autocapitalize="off" spellcheck="false" /></label><p>新增型号保持未启用，能力未知；当前为全部模型时改为保留已知型号的明确白名单。添加不会发送生成请求。</p><p v-if="manualError" role="alert" class="model-error">{{ manualError }}</p><template #actions><UiButton @click="manualOpen=false">取消</UiButton><UiButton variant="primary" :disabled="busy||!manualId" @click="addExactModel">添加型号</UiButton></template></UiDialog>
    </section>
  </div>
  </DetailWorkspace>
</template>

<style scoped>
.channel-key-input{display:flex;align-items:center;gap:var(--space-2);min-width:0}.channel-key-input .input{flex:1;min-width:0}.channel-key-input svg{width:16px;height:16px}

.channel-default-summary{display:grid;grid-template-columns:max-content 1fr;gap:var(--space-2);font-size:var(--text-control)}.channel-default-summary dt{color:var(--muted)}.channel-default-summary dd{margin:0;overflow-wrap:anywhere}
/* 详情头部（原型 model-config.css:221/230/231/232）：标题在上、徽标行在下，行内垂直居中不换行挤压。 */
/* 原型 iconButton 是 .mp-editor-head 第一列的 32px 图标键；这里只约束占位宽度，按钮度量交给 .btn。 */
.channel-detail-back{flex:none;display:inline-flex;width:var(--control-md);min-width:var(--control-md)}
.channel-detail-back :deep(.btn){flex:1;justify-content:center}
.channel-detail-back svg{flex:none;width:16px;height:16px}
.channel-detail-heading{flex:1;min-width:0;display:grid;gap:var(--space-1);align-content:center}
.channel-detail-name{min-width:0;font-size:var(--text-subtitle);line-height:1.3;overflow-wrap:anywhere}
.channel-detail-status{display:flex;align-items:center;gap:6px;flex-wrap:wrap;min-width:0;font-size:var(--text-caption);line-height:18px}
.channel-detail-count{min-width:0;color:var(--muted);white-space:nowrap;font-variant-numeric:tabular-nums}
/* WP-53：公开范围 / 搜索 / 族筛选 / 仅启用 / 计数同处 34px 行；搜索吃剩余宽度。 */
.model-toolbar{display:flex;align-items:center;gap:var(--space-2);flex-wrap:wrap;min-width:0}
.model-toolbar>.input{flex:1 1 160px;min-height:var(--control-md);min-width:120px;box-sizing:border-box}
.model-toolbar :deep(.ui-select-field),.model-toolbar :deep(.btn){flex:none}
.model-toolbar>*{min-width:0}
.model-toolbar .ui-inline-check{display:inline-flex;align-items:center;gap:var(--space-1);min-height:var(--control-md);white-space:nowrap}
.model-toolbar .model-muted{white-space:nowrap;font-variant-numeric:tabular-nums}
/* WP-48：连接字段在页面内直接编辑，Key 跨两列。 */
.channel-config-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-3);min-width:0}
.channel-config-grid>label{display:grid;gap:var(--space-1-5);min-width:0;font-size:var(--text-label);color:var(--muted)}
.channel-config-grid>label>.input,.channel-config-grid>label :deep(.ui-select-field){width:100%}
.channel-config-grid .channel-key{grid-column:1/-1;max-width:calc(50% - var(--space-3) / 2)}
@container(max-width:680px){.channel-config-grid{grid-template-columns:minmax(0,1fr)}.channel-config-grid .channel-key{max-width:none}}
@container(max-width:680px){.model-toolbar{gap:var(--space-1-5)}.model-toolbar>.input{flex-basis:100%}}
</style>
