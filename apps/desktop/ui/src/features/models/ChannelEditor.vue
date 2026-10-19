<script setup lang="ts">
import { computed, reactive, ref, watch, onBeforeUnmount } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import { useNodeMeasurements } from './useNodeMeasurements'
import { emptyRules, modelApi, errorText, type ChannelPolicy, type Template } from './api'
const props = defineProps<{ channel: ChannelPolicy; templates: Template[]; busy?: boolean;existing?:boolean }>()
const emit = defineEmits<{ close: [] }>()
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
const measurementErrors:Record<string,string>={discovery_dns:'地址解析失败',discovery_private:'请先确认允许可信私有网络',discovery_auth:'认证失败',discovery_timeout:'请求超时',discovery_failed:'连接或清单请求失败',discovery_pagination:'分页未完整',discovery_shape:'服务未返回模型清单',discovery_limit:'响应超过上限',discovery_protocol:'该协议需专用连接',node_url:'节点地址无效',credential_store:'系统凭据不可用'}
function mode(value: string) { props.channel.selection = value === 'whitelist' ? { mode: 'whitelist', models: [] } : { mode: value === 'all' ? 'all' : 'none' } }
function select(id: string) {
  const current = props.channel.selection
  if (current.mode !== 'whitelist') { props.channel.selection = { mode: 'whitelist', models: [id] }; return }
  current.models = current.models.includes(id) ? current.models.filter(v => v !== id) : [...current.models, id]
  if (!current.models.length) props.channel.selection = { mode: 'none' }
}
function chosen(id: string) { return props.channel.selection.mode === 'all' || (props.channel.selection.mode === 'whitelist' && props.channel.selection.models.includes(id)) }
function addNode() { props.channel.connection.nodes.push({ id: crypto.randomUUID(), name: '', url: '' }) }
function addModel() { props.channel.discovered.push({ id: '', label: '', rules: emptyRules() }) }
const orderedModels=computed(()=>{const rank=new Map(props.channel.modelOrder.map((id,index)=>[id,index]));return [...props.channel.discovered].sort((a,b)=>(rank.get(a.id)??Number.MAX_SAFE_INTEGER)-(rank.get(b.id)??Number.MAX_SAFE_INTEGER))})
function moveModel(id:string,direction:number){const order=orderedModels.value.map(m=>m.id);const index=order.indexOf(id);const target=index+direction;if(index<0||target<0||target>=order.length)return;const [value]=order.splice(index,1);order.splice(target,0,value!);props.channel.modelOrder=order}
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
    <section><div class="model-field-title"><h2>服务节点</h2><div class="model-inline"><UiButton size="sm" :disabled="busy||!!measurement.run||discovering" @click="measurement.measure(channel.connection.nodes.map(n=>n.id))">检测全部节点</UiButton><UiButton v-if="measurement.run" size="sm" @click="measurement.stop">取消检测</UiButton><UiButton size="sm" :disabled="busy" @click="addNode">新增节点</UiButton></div></div>
      <p class="model-muted">检测模型清单请求的耗时；当前节点由你手动选择。</p><p v-if="measurement.error" class="model-error" role="alert">{{ measurement.error }}</p><p v-if="measurement.stopped" class="model-muted">检测已取消，已完成结果保留。</p>
      <article v-for="node in channel.connection.nodes" :key="node.id" class="model-variant"><div class="model-columns"><label>名称<input class="input" v-model="node.name" :disabled="busy" /></label><label>服务基地址<input class="input" v-model="node.url" type="url" :disabled="busy" placeholder="https://api.example.com/v1" /></label></div><p v-if="measurement.results[node.id]" class="model-muted">{{ measurement.results[node.id]!.error ? (measurementErrors[measurement.results[node.id]!.error!]??"检测未完成，请检查连接") : measurement.results[node.id]!.elapsedMs+" ms · "+measurement.results[node.id]!.modelCount+" 个模型" }}</p><div class="model-inline"><UiButton size="sm" :loading="measurement.pending.includes(node.id)" :disabled="busy||!!measurement.run||discovering" @click="measurement.measure([node.id])">检测节点</UiButton><UiButton size="sm" :disabled="busy || channel.connection.activeNodeId === node.id" @click="channel.connection.activeNodeId = node.id">{{ channel.connection.activeNodeId === node.id ? '当前节点' : '设为当前' }}</UiButton><UiButton size="sm" variant="danger" :disabled="busy || channel.connection.nodes.length === 1 || channel.connection.activeNodeId === node.id" @click="channel.connection.nodes = channel.connection.nodes.filter(n => n.id !== node.id)">删除</UiButton></div></article>
    </section>
    <section><h2>模型默认</h2><ModelRuleFields v-model="channel.defaults" hide-protocol :disabled="busy" /></section>
    <section><h2>采用模版</h2><label v-for="t in templates" :key="t.id">{{ t.label }}<UiSelectField :model-value="channel.templateRefs.find(r => r.templateId === t.id)?.revisionId ?? ''" :label="`采用${t.label}版本`" :options="[{ value: '', label: '不设置渠道引用' }, ...t.revisions.map(r => ({ value: r.id, label: `${r.createdAt} · ${r.id.slice(0,4)}` }))]" :disabled="busy" @update:model-value="adopt(t, $event)" /></label></section>
    <section><div class="model-field-title"><h2>模型列表</h2><UiButton size="sm" :disabled="busy" @click="addModel">手动添加ID</UiButton></div><UiSelectField :model-value="selectionMode" label="模型公开范围" :options="[{ value: 'none', label: '全部隐藏' }, { value: 'all', label: '全部模型（含未来发现）' }, { value: 'whitelist', label: '仅所选模型' }]" :disabled="busy" @update:model-value="mode" />
      <article v-for="(model, i) in orderedModels" :key="model.id || i" class="model-variant"><div class="model-columns"><label>原始模型ID<input class="input" v-model="model.id" :disabled="busy || existingModelIds.has(model.id)" /></label><label>显示名称<input class="input" v-model="model.label" maxlength="128" placeholder="留空使用原模型ID" :disabled="busy" /></label></div><label class="model-inline"><input type="checkbox" :checked="chosen(model.id)" :disabled="busy || !model.id" @change="select(model.id)" />公开此模型</label><details><summary>上游能力声明</summary><ModelRuleFields :model-value="model.rules" disabled /></details><details><summary>单模型覆盖</summary><ModelRuleFields :model-value="channel.overrides[model.id]??emptyRules()" :disabled="busy||!model.id" @update:model-value="channel.overrides[model.id]=$event" /></details><label>采用版本<UiSelectField :model-value="channel.modelRefs[model.id]?JSON.stringify([channel.modelRefs[model.id]!.templateId,channel.modelRefs[model.id]!.revisionId]):''" :options="modelTemplateOptions" :label="model.id+'采用版本'" :disabled="busy||!model.id" @update:model-value="adoptModel(model.id,$event)" /></label><div class="model-inline"><UiButton size="sm" variant="ghost" :disabled="busy||i===0||!model.id" @click="moveModel(model.id,-1)">上移</UiButton><UiButton size="sm" variant="ghost" :disabled="busy||i===orderedModels.length-1||!model.id" @click="moveModel(model.id,1)">下移</UiButton><UiButton size="sm" variant="danger" :disabled="busy" @click="removeRecord(model.id)">移除本机记录</UiButton></div></article>
    </section>
  </div>
</template>
