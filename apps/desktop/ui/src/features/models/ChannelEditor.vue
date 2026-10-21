<script setup lang="ts">
import { computed, reactive, ref, watch, onBeforeUnmount, toRef } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import DetailWorkspace from '@/components/layout/DetailWorkspace.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import ChannelModelTable from './ChannelModelTable.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {createModelRowsState,pendingModelRows,type ModelRowsState} from './modelRows'
import NodePicker from './NodePicker.vue'
import {useModalStore} from '@/app/modal/store'
import NewChannelDialog from './NewChannelDialog.vue'
import { useNodeMeasurements } from './useNodeMeasurements'
import { emptyRules, modelApi, errorText, type ChannelPolicy, type Template, type TemplateRef, type ModelProjection, type ModelPolicy } from './api'
const props = defineProps<{ channel: ChannelPolicy; templates: Template[]; busy?: boolean;existing?:boolean;globalRefs?:TemplateRef[];projection?:ModelProjection[];projectionBusy?:boolean;projectionError?:string;policy?:ModelPolicy;rowState?:ModelRowsState;navigation?:{collapsed:boolean;activeId:string};channels?:ChannelPolicy[];saveDisabled?:boolean;activeCount?:number;viewState?:{search:string;selectedOnly:boolean;family:string} }>()
const emit = defineEmits<{ close: [];switch:[string];preview:[];save:[] }>()
const nav=props.navigation??reactive({collapsed:false,activeId:'channel-config'})
const sections=[{id:'channel-config',label:'渠道配置'},{id:'channel-nodes',label:'服务节点'},{id:'channel-defaults',label:'模型默认'},{id:'channel-models',label:'模型列表'}]
const connectionOpen=ref(false);const connectionOriginal=ref<ChannelPolicy>();
function editConnection(){connectionOriginal.value=JSON.parse(JSON.stringify(props.channel));connectionOpen.value=true}
const apiKey=ref('');const discovering=ref(false);const discoveryError=ref('');let generation=0
watch(()=>[props.channel.id,props.channel.connection.protocol,props.channel.connection.activeNodeId,props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId)?.url],()=>{generation++;discovering.value=false;discoveryError.value=''})
onBeforeUnmount(()=>{generation++;apiKey.value=''})
async function discover(){
 const node=props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId);if(!node)return
 const sequence=++generation;discovering.value=true;discoveryError.value=''
 try{const result=await modelApi.discover({protocol:props.channel.connection.protocol,url:node.url,apiKey:apiKey.value||null,credentialRef:props.channel.connection.credentialRef,allowPrivateNetwork:props.channel.connection.allowPrivateNetwork})
  if(sequence!==generation){if(result.credentialRef&&result.credentialRef!==props.channel.connection.credentialRef)void modelApi.forgetCredential(result.credentialRef);return}
  const previousModels=props.channel.discovered
  props.channel.connection.credentialRef=result.credentialRef;apiKey.value='';props.channel.discovered=result.models.map(model=>{const previous=props.channel.discovered.find(item=>item.id===model.id);return previous?{...model,label:previous.label}:model})
  for(const previous of previousModels)if(chosen(previous.id)&&!props.channel.discovered.some(m=>m.id===previous.id))props.channel.discovered.push(previous)
  for(const model of result.models)if(!props.channel.modelOrder.includes(model.id))props.channel.modelOrder.push(model.id)
 }catch(cause){if(sequence===generation)discoveryError.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
const selectionMode = computed(() => props.channel.selection.mode)
const measurement=reactive(useNodeMeasurements(()=>props.channel,()=>apiKey.value))

function mode(value:string){
 if(props.busy||pendingModelRows(rowsState).length){discoveryError.value='请先处理未确认型号编辑';return}
 if(value===props.channel.selection.mode)return
 const baseline=JSON.stringify(props.channel),current=props.channel.selection
 const next=value==='whitelist'?{mode:'whitelist' as const,models:current.mode==='all'?props.channel.discovered.map(m=>m.id):[]}: {mode:value==='all'?'all' as const:'none' as const}
 useModalStore().open({title:'调整模型公开范围',body:value==='all'?'全部当前和未来发现型号将公开，保存时仍核对全局预算。':value==='none'?'当前和未来型号全部隐藏；已有规则和顺序保留。':'改为明确白名单，保留当前已知的启用项，未来新型号不自动公开。',confirmLabel:'确认范围',onConfirm:()=>{if(JSON.stringify(props.channel)!==baseline){discoveryError.value='范围确认已过期，请重新核对';return}props.channel.selection=next}})
}
function chosen(id:string){return props.channel.selection.mode==='all'||props.channel.selection.mode==='whitelist'&&props.channel.selection.models.includes(id)}
function addNode(){props.channel.connection.nodes.push({id:crypto.randomUUID(),name:'',url:''})}
const manualOpen=ref(false),manualId=ref(''),manualError=ref('')
function addModel(){manualId.value='';manualError.value='';manualOpen.value=true}
function addExactModel(){const id=manualId.value;if(!id||id.trim()!==id||id.length>2048||/[\u0000-\u001f]/.test(id)){manualError.value='请填写不含首尾空白或控制字符的完整原始ID';return}if(props.channel.discovered.some(m=>m.id===id)){manualError.value='此原始ID已经存在';return}if(props.channel.selection.mode==='all')props.channel.selection={mode:'whitelist',models:props.channel.discovered.map(m=>m.id)};props.channel.discovered.push({id,label:id,rules:emptyRules()});if(!props.channel.modelOrder.includes(id))props.channel.modelOrder.push(id);manualOpen.value=false}
const rowsState=props.rowState??reactive(createModelRowsState())
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
    <template #header><UiButton @click="emit('close')">返回渠道列表</UiButton><strong class="channel-detail-name">{{ channel.label || '新渠道' }}</strong><span class="tag">{{ channel.connection.enabled?'启用':'停用' }} · {{ channel.selection.mode==='all'?channel.discovered.length:channel.selection.mode==='whitelist'?channel.selection.models.length:0 }} / {{ channel.discovered.length }}</span><UiSelectField v-if="channels?.length" :model-value="channel.id" :options="channels.map(c=>({value:c.id,label:c.label||'未命名渠道'}))" label="切换渠道" :disabled="busy" @update:model-value="emit('switch',$event)" /></template>
    <template #actions="{collapsed}"><UiButton size="sm" title="预览选择器" aria-label="预览选择器" @click="emit('preview')"><svg v-if="collapsed" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></svg><template v-else>预览选择器</template></UiButton><span :title="'草稿激活数 '+(activeCount??'未知')+' / 100'">{{ activeCount??'—' }}<template v-if="!collapsed"> / 100</template></span><UiButton size="sm" :disabled="busy||saveDisabled" title="保存渠道草稿" aria-label="保存渠道草稿" @click="emit('save')"><svg v-if="collapsed" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 3h13l3 3v15H4zM8 3v6h8V3M8 21v-8h8v8"/></svg><template v-else>保存</template></UiButton></template>
    <div class="model-form">
    <section id="channel-config" tabindex="-1"><h2>渠道配置</h2><div class="model-columns"><label>名称<input class="input" v-model="channel.label" :disabled="busy" /></label><label>运行渠道标识<input class="input" v-model="channel.connection.providerId" :disabled="busy || existing" /></label></div>
      <div class="model-inline"><span>默认协议：{{ channel.connection.protocol }}</span><UiButton :disabled="busy||discovering" @click="editConnection">编辑连接</UiButton><UiButton :loading="discovering" :disabled="busy" @click="discover">获取并确认模型</UiButton><UiButton v-if="discovering" @click="generation++; discovering=false">停止等待</UiButton></div><p v-if="discoveryError" class="model-error" role="alert">{{ discoveryError }}</p>
      <label class="model-inline"><input v-model="channel.connection.enabled" type="checkbox" :disabled="busy" />启用渠道</label>
    </section>
    <section id="channel-nodes" tabindex="-1" aria-label="服务节点"><NodePicker readonly v-model:nodes="channel.connection.nodes" v-model:active-id="channel.connection.activeNodeId" :disabled="busy" :measuring="!!measurement.run" :measurement-disabled="discovering" :pending="measurement.pending" :results="measurement.results" :error="measurement.error" :stopped="measurement.stopped" @measure="measurement.measure" @stop="measurement.stop" @add="addNode" /></section>
    <NewChannelDialog v-if="connectionOpen&&connectionOriginal" :original="connectionOriginal" @close="connectionOpen=false" />
    <section id="channel-defaults" tabindex="-1"><h2>模型默认</h2><ModelRuleFields v-model="channel.defaults" hide-protocol :disabled="busy" />
    <h3>采用模版</h3><label v-for="t in templates" :key="t.id">{{ t.label }}<UiSelectField :model-value="channel.templateRefs.find(r => r.templateId === t.id)?.revisionId ?? ''" :label="`采用${t.label}版本`" :options="[{ value: '', label: '不设置渠道引用' }, ...t.revisions.map(r => ({ value: r.id, label: `${r.createdAt} · ${r.id.slice(0,4)}` }))]" :disabled="busy" @update:model-value="adopt(t, $event)" /></label>
    <div v-if="adoptedFamilies.length"><h3>渠道族覆盖</h3><details v-for="family in adoptedFamilies" :key="family.id"><summary>{{ family.label }}</summary><ModelRuleFields :model-value="channel.familyOverrides[family.id]??emptyRules()" :disabled="busy" @update:model-value="channel.familyOverrides[family.id]=$event" /><UiButton size="sm" variant="ghost" :disabled="busy||!channel.familyOverrides[family.id]" @click="delete channel.familyOverrides[family.id]">恢复此族继承</UiButton></details></div></section>
    <section id="channel-models" tabindex="-1"><div class="model-field-title"><h2>模型列表</h2><UiButton size="sm" :disabled="busy" @click="addModel">手动添加ID</UiButton></div><UiSelectField :model-value="selectionMode" label="模型公开范围" :options="[{ value: 'none', label: '全部隐藏' }, { value: 'all', label: '全部模型（含未来发现）' }, { value: 'whitelist', label: '仅所选模型' }]" :disabled="busy" @update:model-value="mode" />
      <div class="model-inline"><input v-model="modelSearch" class="input" type="search" aria-label="搜索渠道内型号" placeholder="搜索型号或名称" /><UiSelectField v-model="familyFilter" :options="familyOptions" label="渠道内型号族筛选" :disabled="projectionBusy||!!projectionError" /><label><input v-model="selectedOnly" type="checkbox" />仅启用</label></div><p v-if="modelSearch.trim()||familyFilter!=='all'||selectedOnly" class="model-muted">清除全部搜索和筛选后可调整型号顺序。</p><p class="model-muted" role="status">显示 {{ visibleModels.length }} / {{ orderedModels.length }} 项</p>
      <p v-if="projectionBusy" role="status" class="model-muted">正在解析当前草稿，筛选暂用上次结果…</p><p v-else-if="projectionError" role="alert" class="model-error">{{ projectionError }}；有效值暂不可确定，筛选暂用上次结果。</p>
      <ChannelModelTable :channel="channel" :policy="policy" :models="visibleModels" :templates="templates" :projection="projection" :projection-busy="projectionBusy||!!projectionError" :disabled="busy||discovering" :filtered="!!modelSearch.trim()||familyFilter!=='all'||selectedOnly" :state="rowsState" />
      <UiDialog :open="manualOpen" title="手动添加精确型号" :busy="busy" @close="manualOpen=false"><label>原始模型ID<input v-model="manualId" class="input" autocapitalize="off" spellcheck="false" /></label><p>新增型号保持未启用，能力未知；当前为全部模型时改为保留已知型号的明确白名单。添加不会发送生成请求。</p><p v-if="manualError" role="alert" class="model-error">{{ manualError }}</p><template #actions><UiButton @click="manualOpen=false">取消</UiButton><UiButton variant="primary" :disabled="busy||!manualId" @click="addExactModel">添加型号</UiButton></template></UiDialog>
    </section>
  </div>
  </DetailWorkspace>
</template>

<style scoped>
.channel-detail-name{flex:1;min-width:80px;overflow-wrap:anywhere}
</style>
