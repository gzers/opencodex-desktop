<script setup lang="ts">
import {computed,onBeforeUnmount,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsStore} from './store'
import {emptyRules,errorText,modelApi,type DiscoveredModel,type ChannelPolicy} from './api'
const props=defineProps<{original?:ChannelPolicy}>()
const original=props.original?JSON.parse(JSON.stringify(props.original)) as ChannelPolicy:undefined
const emit=defineEmits<{close:[];added:[string]}>()
const store=useModelsStore()
const step=ref(1),label=ref(original?.label??''),providerId=ref(original?.connection.providerId??''),protocol=ref(original?.connection.protocol??'openai-responses'),apiKey=ref(''),privateNetwork=ref(original?.connection.allowPrivateNetwork??false)
const nodes=ref<{id:string;name:string;url:string}[]>(original?JSON.parse(JSON.stringify(original.connection.nodes)):[])
const activeNode=ref(original?.connection.activeNodeId??'')
const newUrl=ref('')
const url=computed({get:()=>original?(nodes.value.find(n=>n.id===activeNode.value)?.url??''):newUrl.value,set:value=>{if(original){const node=nodes.value.find(n=>n.id===activeNode.value);if(node)node.url=value}else newUrl.value=value}})
const id=original?.id??crypto.randomUUID(),nodeId=crypto.randomUUID()
const models=ref<DiscoveredModel[]>([]),selected=ref<string[]>([]),query=ref(''),onlySelected=ref(false),manual=ref('')
const ready=ref(true)
const busy=ref(false),discovering=ref(false),error=ref(''),status=ref('')
let generation=0,disposed=false
const needsKey=computed(()=>protocol.value!=='ollama-native')
const rows=computed(()=>models.value.filter(m=>(!onlySelected.value||selected.value.includes(m.id))&&(m.id+' '+m.label).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const allVisible=computed(()=>rows.value.length>0&&rows.value.every(m=>selected.value.includes(m.id)))
const mixedVisible=computed(()=>rows.value.some(m=>selected.value.includes(m.id))&&!allVisible.value)
watch(()=>JSON.stringify([protocol.value,url.value,apiKey.value,privateNetwork.value,nodes.value]),()=>{generation++;discovering.value=false;models.value=[];selected.value=[];status.value='';error.value=''})
onBeforeUnmount(()=>{disposed=true;generation++;apiKey.value=''})
function connectionError(){
 if(!label.value.trim()||label.value.trim().length>256)return '请填写渠道名称，名称不能过长'
 if(!/^[A-Za-z0-9_-]{1,128}$/.test(providerId.value)||['openai','chatgpt','openai-multi','combo'].includes(providerId.value.toLowerCase()))return '运行标识仅支持字母、数字、连字符和下划线，且不能使用原生保留标识'
 const existing=store.channelDraft?.channels??store.snapshot?.policy.channels??[]
 if(existing.filter(c=>c.id!==original?.id).some(c=>c.label.trim().toLowerCase()===label.value.trim().toLowerCase()||c.connection.providerId.toLowerCase()===providerId.value.toLowerCase()))return '渠道名称或运行标识已存在'
 try{const parsed=new URL(url.value);if(!['http:','https:'].includes(parsed.protocol)||!parsed.hostname||parsed.username||parsed.password||parsed.search||parsed.hash||/\/(responses|chat\/completions|messages)\/?$/.test(parsed.pathname))return '填写不含凭据、参数或生成接口路径的HTTP/HTTPS服务基地址'}catch{return '请填写有效服务基地址'}
 if(original){
   if(nodes.value.length<1||nodes.value.length>32||nodes.value.some(node=>{try{const u=new URL(node.url);return !['http:','https:'].includes(u.protocol)||!!u.username||!!u.password||!!u.search||!!u.hash||!node.name.trim()}catch{return true}}))return '请补齐每个节点的名称和有效服务基地址'
   const unchanged=protocol.value===original.connection.protocol&&privateNetwork.value===original.connection.allowPrivateNetwork&&JSON.stringify(nodes.value)===JSON.stringify(original.connection.nodes)&&activeNode.value===original.connection.activeNodeId&&!apiKey.value
   if(unchanged)return '连接尚未修改，可关闭返回当前工作区'
 }
 if(needsKey.value&&(!apiKey.value||apiKey.value.length>8192||/[\r\n]/.test(apiKey.value)))return '请输入有效API Key'
 return ''
}
function addEditNode(){nodes.value.push({id:crypto.randomUUID(),name:'',url:''})}
function next(){const message=connectionError();if(message){error.value=message;return}error.value='';step.value=2}
async function discover(){
 if(busy.value||discovering.value)return
 const message=connectionError();if(message){error.value=message;step.value=1;return}
 const sequence=++generation;discovering.value=true;error.value='';status.value=''
 try{const result=await modelApi.previewNewChannel({protocol:protocol.value,url:url.value,apiKey:needsKey.value?apiKey.value:null,credentialRef:null,allowPrivateNetwork:privateNetwork.value})
  if(disposed||sequence!==generation)return
  const previous=models.value;models.value=result.models.map(m=>({...m,label:previous.find(old=>old.id===m.id)?.label??m.label}))
  // Preserve explicitly selected or manually entered IDs temporarily absent from discovery.
  for(const old of previous)if(selected.value.includes(old.id)&&!models.value.some(m=>m.id===old.id))models.value.push(old)
  status.value=result.models.length?'已读取 '+result.models.length+' 个型号；新发现型号默认不选。':'清单为空，可重试、返回修改连接或手动添加精确ID。'
 }catch(cause){if(!disposed&&sequence===generation)error.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
function select(id:string,checked:boolean){selected.value=checked?[...new Set([...selected.value,id])]:selected.value.filter(v=>v!==id)}
function selectVisible(){const visible=new Set(rows.value.map(m=>m.id));selected.value=allVisible.value?selected.value.filter(id=>!visible.has(id)):[...new Set([...selected.value,...visible])]}
function addManual(){const value=manual.value;if(!value||value.trim()!==value||value.length>2048||/[\u0000-\u001f]/.test(value)){error.value='请输入完整且不含首尾空白的精确模型ID';return}if(models.value.some(m=>m.id===value)){error.value='此精确模型ID已在清单中';return}models.value.push({id:value,label:value,rules:emptyRules()});manual.value='';error.value=''}
async function join(){
 if(busy.value||discovering.value)return
 const message=connectionError();if(message){error.value=message;step.value=1;return}
 if(!selected.value.length){error.value='请选择至少一个精确型号';return}
 busy.value=true;store.busy=true;error.value=''
 try{
  await store.persistDraft();if(store.channelError)throw {message:store.channelError}
  if(!store.snapshot)throw {message:'请先读取本机渠道工作区'}
  const channel:ChannelPolicy={id,label:label.value.trim(),templateRefs:[],modelRefs:{},connection:{providerId:providerId.value,protocol:protocol.value,nodes:[{id:nodeId,name:'默认节点',url:url.value}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:privateNetwork.value},selection:{mode:'whitelist',models:[...selected.value]},modelOrder:models.value.map(m=>m.id),discovered:JSON.parse(JSON.stringify(models.value)),defaults:emptyRules(),overrides:{},familyOverrides:{}}
  if(original){
   if(JSON.stringify(store.channelDraft?.channels.find(c=>c.id===original.id))!==JSON.stringify(original))throw {code:'connection_edit',message:'原渠道草稿已变化，请关闭后重新编辑连接'}
   Object.assign(channel,{templateRefs:original.templateRefs,modelRefs:original.modelRefs,defaults:original.defaults,overrides:original.overrides,familyOverrides:original.familyOverrides,naming:original.naming})
   channel.connection={...channel.connection,nodes:JSON.parse(JSON.stringify(nodes.value)),activeNodeId:activeNode.value,enabled:original.connection.enabled}
   channel.modelOrder=[...new Set([...original.modelOrder,...original.discovered.map(m=>m.id),...channel.modelOrder])]
  }
  if(!store.channelDraft)throw {message:'请先读取当前配置'}
  if(original)store.channelDraft.channels=store.channelDraft.channels.map(c=>c.id===original.id?channel:c)
  else store.channelDraft.channels.push(channel)
  if(apiKey.value)store.credentialEdits[channel.id]={channelId:channel.id,action:'replace',value:apiKey.value}
  apiKey.value='';emit('added',channel.id);emit('close')
 }catch(cause){error.value=errorText(cause)}
 finally{busy.value=false;store.busy=false}
}
function close(){if(busy.value)return;generation++;apiKey.value='';emit('close')}
</script>
<template>
 <UiDialog :open="true" wide :title="original?'编辑渠道连接':'新增渠道'" :busy="busy" @close="close">
  <p role="status">{{ step===1?'1 / 2 · 连接配置':'2 / 2 · 选择模型' }}</p>
  <template v-if="step===1">
   <div class="model-columns"><label>渠道名称<input v-model="label" class="input" maxlength="256" :disabled="!!original" /></label><label>运行渠道标识<input v-model="providerId" class="input" maxlength="128" :disabled="!!original" autocapitalize="off" spellcheck="false" /></label></div>
   <label>默认协议<UiSelectField v-model="protocol" label="新渠道默认协议" :options="['openai-responses','openai-chat','anthropic','google','ollama-native'].map(value=>({value,label:value}))" /></label>
   <template v-if="original"><label>当前节点<UiSelectField v-model="activeNode" label="编辑连接的当前节点" :options="nodes.map(n=>({value:n.id,label:n.name||'未命名节点'}))" /></label><div v-for="node in nodes" :key="node.id" class="model-columns"><label>节点名称<input v-model="node.name" class="input" /></label><label>节点服务基地址<input v-model="node.url" class="input" type="url" :aria-label="node.name+'编辑地址'" /></label><UiButton size="sm" variant="danger" :disabled="nodes.length<2||node.id===activeNode" @click="nodes=nodes.filter(n=>n.id!==node.id)">删除节点</UiButton></div><UiButton size="sm" :disabled="nodes.length>=32" @click="addEditNode">新增节点</UiButton><p>修改连接后重新获取并选择型号；请重新输入用于这些节点的Key，不会把原Key自动发到新地址。</p></template>
   <label v-else>服务基地址<input v-model="url" class="input" type="url" autocapitalize="off" spellcheck="false" placeholder="https://api.example.com/v1" /></label>
   <label v-if="needsKey">API Key<input v-model="apiKey" class="input" type="password" autocomplete="new-password" autocapitalize="off" spellcheck="false" /></label>
   <label><input v-model="privateNetwork" type="checkbox" />允许访问可信私有网络</label>
  </template>
  <template v-else>
   <div class="model-field-title"><strong>{{ label }}</strong><UiButton :loading="discovering" :disabled="busy" @click="discover">获取模型列表</UiButton><UiButton v-if="discovering" @click="generation++;discovering=false">取消等待</UiButton></div>
   <div class="model-inline"><input v-model="query" class="input" type="search" aria-label="新渠道型号搜索" placeholder="搜索型号或名称" /><label><input v-model="onlySelected" type="checkbox" :disabled="discovering" />仅已选</label></div>
   <p>已选 {{ selected.length }} / {{ models.length }} 项<span v-if="query||onlySelected"> · 当前显示 {{ rows.length }} 项</span></p>
   <label><input type="checkbox" :checked="allVisible" :indeterminate="mixedVisible" :disabled="discovering||!rows.length" @change="selectVisible" />选择当前结果</label>
   <div class="new-channel-models"><label v-for="model in rows" :key="model.id"><input type="checkbox" :checked="selected.includes(model.id)" :disabled="discovering" :aria-label="'选择新渠道型号 '+model.id" @change="select(model.id,($event.target as HTMLInputElement).checked)" /><span>{{ model.label||model.id }}<code v-if="model.label!==model.id">{{ model.id }}</code></span></label><p v-if="!rows.length">暂无型号；可获取清单或手动添加精确ID。</p></div>
   <details><summary>手动添加精确ID</summary><label>原始模型ID<input v-model="manual" class="input" :disabled="discovering" autocapitalize="off" spellcheck="false" /></label><UiButton size="sm" :disabled="discovering||!manual" @click="addManual">添加ID</UiButton></details>
  </template>
  <p v-if="status" role="status" class="model-muted">{{ status }}</p><p v-if="error" role="alert" class="model-error">{{ error }}</p>
  <template #actions><UiButton :disabled="busy" @click="close">关闭</UiButton><UiButton v-if="step===2" :disabled="busy" @click="generation++;discovering=false;step=1">上一步</UiButton><UiButton v-if="step===1" variant="primary" :disabled="!ready" @click="next">下一步</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="discovering||!selected.length" @click="join">{{ original?'确认连接并加入草稿':'加入草稿' }}</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.new-channel-models{max-height:320px;overflow:auto;min-width:0}.new-channel-models label{display:flex;gap:var(--space-2);align-items:flex-start;padding-block:var(--space-2)}.new-channel-models span,.new-channel-models code{min-width:0;overflow-wrap:anywhere}.new-channel-models code{display:block;font-family:var(--font-code);color:var(--muted)}
</style>
