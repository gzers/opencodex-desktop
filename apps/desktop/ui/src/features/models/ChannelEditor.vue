<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import { emptyRules, modelApi, errorText, type ChannelPolicy, type Template } from './api'
const props = defineProps<{ channel: ChannelPolicy; templates: Template[]; busy?: boolean }>()
const emit = defineEmits<{ close: [] }>()
const apiKey=ref('');const discovering=ref(false);const discoveryError=ref('');let generation=0
watch(()=>[props.channel.id,props.channel.connection.protocol,props.channel.connection.activeNodeId,props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId)?.url],()=>{generation++;discovering.value=false;discoveryError.value=''})
onBeforeUnmount(()=>{generation++;apiKey.value=''})
async function discover(){
 const node=props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId);if(!node)return
 const sequence=++generation;discovering.value=true;discoveryError.value=''
 try{const result=await modelApi.discover({protocol:props.channel.connection.protocol,url:node.url,apiKey:apiKey.value||null,credentialRef:props.channel.connection.credentialRef,allowPrivateNetwork:props.channel.connection.allowPrivateNetwork})
  if(sequence!==generation){if(result.credentialRef&&result.credentialRef!==props.channel.connection.credentialRef)void modelApi.forgetCredential(result.credentialRef);return}
  props.channel.connection.credentialRef=result.credentialRef;apiKey.value='';props.channel.discovered=result.models
  for(const model of result.models)if(!props.channel.modelOrder.includes(model.id))props.channel.modelOrder.push(model.id)
 }catch(cause){if(sequence===generation)discoveryError.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
const selectionMode = computed(() => props.channel.selection.mode)
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
function adopt(t: Template, revision: string) {
  props.channel.templateRefs = props.channel.templateRefs.filter(r => r.templateId !== t.id)
  if (revision) props.channel.templateRefs.push({ templateId: t.id, revisionId: revision })
}
</script>
<template>
  <div class="model-form">
    <header class="model-field-title"><UiButton @click="emit('close')">返回渠道列表</UiButton><strong>{{ channel.label || '新渠道' }}</strong></header>
    <section><h2>渠道配置</h2><div class="model-columns"><label>名称<input class="input" v-model="channel.label" :disabled="busy" /></label><label>运行渠道标识<input class="input" v-model="channel.connection.providerId" :disabled="busy" /></label></div>
      <label>默认协议<UiSelectField v-model="channel.connection.protocol" label="渠道默认协议" :options="['openai-responses','openai-chat','anthropic','google','ollama-native'].map(value => ({ value, label: value }))" :disabled="busy" /></label>
      <label>API Key<input v-model="apiKey" class="input" type="password" autocomplete="new-password" autocapitalize="off" spellcheck="false" :disabled="busy || discovering" :placeholder="channel.connection.credentialRef ? '已安全配置；留空保持' : '无需认证可留空'" /></label><label class="model-inline"><input v-model="channel.connection.allowPrivateNetwork" type="checkbox" :disabled="busy" />允许此渠道访问可信私有网络</label><div class="model-inline"><UiButton :loading="discovering" :disabled="busy" @click="discover">获取并确认模型</UiButton><UiButton v-if="discovering" @click="generation++; discovering = false">停止等待</UiButton></div><p v-if="discoveryError" class="model-error" role="alert">{{ discoveryError }}</p>
      <label class="model-inline"><input v-model="channel.connection.enabled" type="checkbox" :disabled="busy" />启用渠道</label>
    </section>
    <section><div class="model-field-title"><h2>服务节点</h2><UiButton size="sm" :disabled="busy" @click="addNode">新增节点</UiButton></div>
      <article v-for="node in channel.connection.nodes" :key="node.id" class="model-variant"><div class="model-columns"><label>名称<input class="input" v-model="node.name" :disabled="busy" /></label><label>服务基地址<input class="input" v-model="node.url" type="url" :disabled="busy" placeholder="https://api.example.com/v1" /></label></div><div class="model-inline"><UiButton size="sm" :disabled="busy || channel.connection.activeNodeId === node.id" @click="channel.connection.activeNodeId = node.id">{{ channel.connection.activeNodeId === node.id ? '当前节点' : '设为当前' }}</UiButton><UiButton size="sm" variant="danger" :disabled="busy || channel.connection.nodes.length === 1 || channel.connection.activeNodeId === node.id" @click="channel.connection.nodes = channel.connection.nodes.filter(n => n.id !== node.id)">删除</UiButton></div></article>
    </section>
    <section><h2>模型默认</h2><ModelRuleFields v-model="channel.defaults" hide-protocol :disabled="busy" /></section>
    <section><h2>采用模版</h2><label v-for="t in templates" :key="t.id">{{ t.label }}<UiSelectField :model-value="channel.templateRefs.find(r => r.templateId === t.id)?.revisionId ?? ''" :label="`采用${t.label}版本`" :options="[{ value: '', label: '不设置渠道引用' }, ...t.revisions.map(r => ({ value: r.id, label: `${r.createdAt} · ${r.id.slice(0,4)}` }))]" :disabled="busy" @update:model-value="adopt(t, $event)" /></label></section>
    <section><div class="model-field-title"><h2>模型列表</h2><UiButton size="sm" :disabled="busy" @click="addModel">手动添加ID</UiButton></div><UiSelectField :model-value="selectionMode" label="模型公开范围" :options="[{ value: 'none', label: '全部隐藏' }, { value: 'all', label: '全部模型（含未来发现）' }, { value: 'whitelist', label: '仅所选模型' }]" :disabled="busy" @update:model-value="mode" />
      <article v-for="(model, i) in channel.discovered" :key="i" class="model-variant"><div class="model-columns"><label>原始模型ID<input class="input" v-model="model.id" :disabled="busy" /></label><label>显示名称<input class="input" v-model="model.label" :disabled="busy" /></label></div><label class="model-inline"><input type="checkbox" :checked="chosen(model.id)" :disabled="busy || !model.id" @change="select(model.id)" />公开此模型</label><details><summary>能力声明</summary><ModelRuleFields v-model="model.rules" :disabled="busy" /></details><UiButton size="sm" variant="danger" :disabled="busy" @click="channel.discovered.splice(i,1)">移除手动记录</UiButton></article>
    </section>
  </div>
</template>
