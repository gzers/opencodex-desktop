<script setup lang="ts">
import { computed, reactive, ref, watch, onBeforeUnmount } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import NodePicker from './NodePicker.vue'
import { useNodeMeasurements } from './useNodeMeasurements'
import {reorderSlots} from './ordering'
import {useModalStore} from '@/app/modal/store'
import { emptyRules, modelApi, errorText, type ChannelPolicy, type Template } from './api'
const props = defineProps<{ channel: ChannelPolicy; templates: Template[]; busy?: boolean;existing?:boolean }>()
const emit = defineEmits<{ close: [] }>()
const modal=useModalStore()
const existingModelIds=new Set(props.channel.discovered.map(model=>model.id).filter(Boolean))
const apiKey=ref('');const discovering=ref(false);const discoveryError=ref('');let generation=0
watch(()=>[props.channel.id,props.channel.connection.protocol,props.channel.connection.activeNodeId,props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId)?.url],()=>{generation++;discovering.value=false;discoveryError.value=''})
onBeforeUnmount(()=>{generation++;apiKey.value=''})
async function discover(){
 const node=props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId);if(!node)return
 const sequence=++generation;discovering.value=true;discoveryError.value=''
 try{const result=await modelApi.discover({protocol:props.channel.connection.protocol,url:node.url,apiKey:apiKey.value||null,credentialRef:props.channel.connection.credentialRef,allowPrivateNetwork:props.channel.connection.allowPrivateNetwork})
  if(sequence!==generation){if(result.credentialRef&&result.credentialRef!==props.channel.connection.credentialRef)void modelApi.forgetCredential(result.credentialRef);return}
  props.channel.connection.credentialRef=result.credentialRef;apiKey.value='';props.channel.discovered=result.models.map(model=>{existingModelIds.add(model.id);const previous=props.channel.discovered.find(item=>item.id===model.id);return previous?{...model,label:previous.label}:model})
  for(const model of result.models)if(!props.channel.modelOrder.includes(model.id))props.channel.modelOrder.push(model.id)
 }catch(cause){if(sequence===generation)discoveryError.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
const selectionMode = computed(() => props.channel.selection.mode)
const measurement=reactive(useNodeMeasurements(()=>props.channel,()=>apiKey.value))

function mode(value: string) { props.channel.selection = value === 'whitelist' ? { mode: 'whitelist', models: [] } : { mode: value === 'all' ? 'all' : 'none' } }
function select(id: string, event?:Event) {
  if(event?.target instanceof HTMLInputElement)event.target.checked=chosen(id)
  if(props.busy||discovering.value)return
  const current = props.channel.selection
  const next = current.mode==='all'?props.channel.discovered.map(m=>m.id).filter(value=>value!==id):current.mode==='whitelist'?(current.models.includes(id)?current.models.filter(v=>v!==id):[...current.models,id]):[id]
  const apply=()=>{props.channel.selection=next.length?{mode:'whitelist',models:next}:{mode:'none'}}
  if(current.mode==='all'||!next.length){
    const baseline=JSON.stringify(props.channel)
    modal.open({title:next.length?'改为仅公开所选型号':'隐藏全部型号',body:next.length?'将取消此型号并改为明确白名单，未来发现的新型号不会自动公开。':'当前渠道与未来发现型号均不再进入运行选择器。修改仍需保存才生效。',confirmLabel:next.length?'确认修改':'全部隐藏',onConfirm:()=>{if(props.busy||JSON.stringify(props.channel)!==baseline){discoveryError.value='型号选择确认已过期，请重新核对';return}apply()}})
  }else apply()
}
function chosen(id: string) { return props.channel.selection.mode === 'all' || (props.channel.selection.mode === 'whitelist' && props.channel.selection.models.includes(id)) }
function addNode() { props.channel.connection.nodes.push({ id: crypto.randomUUID(), name: '', url: '' }) }
function addModel() { props.channel.discovered.push({ id: '', label: '', rules: emptyRules() }) }
const orderedModels=computed(()=>{const rank=new Map(props.channel.modelOrder.map((id,index)=>[id,index]));return [...props.channel.discovered].sort((a,b)=>(rank.get(a.id)??Number.MAX_SAFE_INTEGER)-(rank.get(b.id)??Number.MAX_SAFE_INTEGER))})
const modelSearch=ref('');const selectedOnly=ref(false);const draggedModel=ref('')
const visibleModels=computed(()=>orderedModels.value.filter(m=>(!selectedOnly.value||chosen(m.id))&&(m.id+' '+m.label).toLocaleLowerCase().includes(modelSearch.value.trim().toLocaleLowerCase())))
const canReorder=computed(()=>!props.busy&&!discovering.value&&!modelSearch.value.trim())
function reorderModel(id:string,target:string){
 if(!canReorder.value)return
 const order=[...new Set([...props.channel.modelOrder,...orderedModels.value.map(m=>m.id).filter(Boolean)])]
 props.channel.modelOrder=reorderSlots(order,visibleModels.value.map(m=>m.id),id,target)
}
function moveModel(id:string,direction:number){const order=visibleModels.value.map(m=>m.id);const target=order[order.indexOf(id)+direction];if(target)reorderModel(id,target)}
function dragModel(event:DragEvent,id:string){if(!canReorder.value){event.preventDefault();return}draggedModel.value=id;event.dataTransfer?.setData('text/plain',id);if(event.dataTransfer)event.dataTransfer.effectAllowed='move'}
function dropModel(target:string){if(draggedModel.value)reorderModel(draggedModel.value,target);draggedModel.value=''}
function adoptModel(id:string,value:string){if(!value){delete props.channel.modelRefs[id];return}const [templateId,revisionId]=JSON.parse(value) as [string,string];props.channel.modelRefs[id]={templateId,revisionId}}
const modelTemplateOptions=computed(()=>[{value:'',label:'沿用匹配结果'},...props.templates.flatMap(t=>t.revisions.map(r=>({value:JSON.stringify([t.id,r.id]),label:t.label+' · '+r.createdAt.slice(0,10)+' · '+r.id.slice(0,4)})))])
function removeRecord(id:string){const model=props.channel.discovered.find(m=>m.id===id);if(!model)return;props.channel.discovered=props.channel.discovered.filter(m=>m!==model);props.channel.modelOrder=props.channel.modelOrder.filter(value=>value!==id);delete props.channel.modelRefs[id];delete props.channel.overrides[id];if(props.channel.selection.mode==='whitelist'){const models=props.channel.selection.models.filter(value=>value!==id);props.channel.selection=models.length?{mode:'whitelist',models}:{mode:'none'}}}
function adopt(t: Template, revision: string) {
  props.channel.templateRefs = props.channel.templateRefs.filter(r => r.templateId !== t.id)
  if (revision) props.channel.templateRefs.push({ templateId: t.id, revisionId: revision })
}
</script>
<template>
  <div class="model-form">
    <header class="model-field-title"><UiButton @click="emit('close')">返回渠道列表</UiButton><strong>{{ channel.label || '新渠道' }}</strong></header>
    <section><h2>渠道配置</h2><div class="model-columns"><label>名称<input class="input" v-model="channel.label" :disabled="busy" /></label><label>运行渠道标识<input class="input" v-model="channel.connection.providerId" :disabled="busy || existing" /></label></div>
      <label>默认协议<UiSelectField v-model="channel.connection.protocol" label="渠道默认协议" :options="['openai-responses','openai-chat','anthropic','google','ollama-native'].map(value => ({ value, label: value }))" :disabled="busy" /></label>
      <label>API Key<input v-model="apiKey" class="input" type="password" autocomplete="new-password" autocapitalize="off" spellcheck="false" :disabled="busy || discovering" :placeholder="channel.connection.credentialRef ? '已安全配置；留空保持' : '无需认证可留空'" /></label><label class="model-inline"><input v-model="channel.connection.allowPrivateNetwork" type="checkbox" :disabled="busy" />允许此渠道访问可信私有网络</label><div class="model-inline"><UiButton :loading="discovering" :disabled="busy" @click="discover">获取并确认模型</UiButton><UiButton v-if="discovering" @click="generation++; discovering = false">停止等待</UiButton></div><p v-if="discoveryError" class="model-error" role="alert">{{ discoveryError }}</p>
      <label class="model-inline"><input v-model="channel.connection.enabled" type="checkbox" :disabled="busy" />启用渠道</label>
    </section>
    <NodePicker v-model:nodes="channel.connection.nodes" v-model:active-id="channel.connection.activeNodeId" :disabled="busy" :measuring="!!measurement.run" :measurement-disabled="discovering" :pending="measurement.pending" :results="measurement.results" :error="measurement.error" :stopped="measurement.stopped" @measure="measurement.measure" @stop="measurement.stop" @add="addNode" />
    <section><h2>模型默认</h2><ModelRuleFields v-model="channel.defaults" hide-protocol :disabled="busy" /></section>
    <section><h2>采用模版</h2><label v-for="t in templates" :key="t.id">{{ t.label }}<UiSelectField :model-value="channel.templateRefs.find(r => r.templateId === t.id)?.revisionId ?? ''" :label="`采用${t.label}版本`" :options="[{ value: '', label: '不设置渠道引用' }, ...t.revisions.map(r => ({ value: r.id, label: `${r.createdAt} · ${r.id.slice(0,4)}` }))]" :disabled="busy" @update:model-value="adopt(t, $event)" /></label></section>
    <section><div class="model-field-title"><h2>模型列表</h2><UiButton size="sm" :disabled="busy" @click="addModel">手动添加ID</UiButton></div><UiSelectField :model-value="selectionMode" label="模型公开范围" :options="[{ value: 'none', label: '全部隐藏' }, { value: 'all', label: '全部模型（含未来发现）' }, { value: 'whitelist', label: '仅所选模型' }]" :disabled="busy" @update:model-value="mode" />
      <div class="model-inline"><input v-model="modelSearch" class="input" type="search" aria-label="搜索渠道内型号" placeholder="搜索型号或名称" /><label><input v-model="selectedOnly" type="checkbox" />仅已选</label></div><p v-if="modelSearch.trim()" class="model-muted">清除搜索后可排序；完整仅已选视图可直接调整顺序。</p><p class="model-muted" role="status">显示 {{ visibleModels.length }} / {{ orderedModels.length }} 项</p>
      <article v-for="(model, i) in visibleModels" :key="model.id || i" class="model-variant" @dragover.prevent @drop.prevent="dropModel(model.id)"><div class="model-columns"><label>原始模型ID<input class="input" v-model="model.id" :disabled="busy || existingModelIds.has(model.id)" /></label><label>显示名称<input class="input" v-model="model.label" maxlength="128" placeholder="留空使用原模型ID" :disabled="busy" /></label></div><label class="model-inline"><input type="checkbox" :checked="chosen(model.id)" :disabled="busy || !model.id" @change="select(model.id,$event)" />公开此模型</label><details><summary>上游能力声明</summary><ModelRuleFields :model-value="model.rules" disabled /></details><details><summary>单模型覆盖</summary><ModelRuleFields :model-value="channel.overrides[model.id]??emptyRules()" :disabled="busy||!model.id" @update:model-value="channel.overrides[model.id]=$event" /></details><label>采用版本<UiSelectField :model-value="channel.modelRefs[model.id]?JSON.stringify([channel.modelRefs[model.id]!.templateId,channel.modelRefs[model.id]!.revisionId]):''" :options="modelTemplateOptions" :label="model.id+'采用版本'" :disabled="busy||!model.id" @update:model-value="adoptModel(model.id,$event)" /></label><div class="model-inline"><UiButton size="sm" variant="ghost" :draggable="canReorder" :disabled="!canReorder||!model.id" :aria-label="model.id+'拖动排序'" @dragstart="dragModel($event,model.id)" @dragend="draggedModel=''">↕</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'上移'" :disabled="!canReorder||i===0||!model.id" @click="moveModel(model.id,-1)">上移</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'下移'" :disabled="!canReorder||i===visibleModels.length-1||!model.id" @click="moveModel(model.id,1)">下移</UiButton><UiButton size="sm" variant="ghost" :aria-label="model.id+'置顶'" :disabled="!canReorder||i===0||!model.id" @click="reorderModel(model.id,visibleModels[0]!.id)">置顶</UiButton><UiButton size="sm" variant="danger" :disabled="busy" @click="removeRecord(model.id)">移除本机记录</UiButton></div></article>
    </section>
  </div>
</template>
