<script setup lang="ts">
import {computed,onBeforeUnmount,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsStore} from './store'
import {useModelsV2Store} from './store-v2'
import {errorText,type ChannelPolicy} from './api'
import type {ChannelConnectionEditIntentV2,TemporaryDiscoveryKind} from './api-v2'
const props=defineProps<{original?:ChannelPolicy}>()
const original=props.original?JSON.parse(JSON.stringify(props.original)) as ChannelPolicy:undefined
const emit=defineEmits<{close:[];added:[string]}>()
const savedView=useModelsStore(),store=useModelsV2Store()
const step=ref(1),label=ref(original?.label??''),authMode=ref<'key'|'none'>(original?.connection.authMode==='none'?'none':'key'),apiKey=ref(''),keyVisible=ref(false)
const discoveryKind=ref<TemporaryDiscoveryKind|''>(''),discoveryReadId=ref<string|null>(null)
const nodes=ref<{id:string;name:string;url:string}[]>(original?JSON.parse(JSON.stringify(original.connection.nodes)):[])
const activeNode=ref(original?.connection.activeNodeId??''),newUrl=ref('')
const url=computed({get:()=>original?(nodes.value.find(n=>n.id===activeNode.value)?.url??''):newUrl.value,set:value=>{if(original){const node=nodes.value.find(n=>n.id===activeNode.value);if(node)node.url=value}else newUrl.value=value}})
const id=original?.id??crypto.randomUUID(),nodeId=crypto.randomUUID()
const models=ref<{id:string;label:string}[]>([]),selected=ref<string[]>([]),query=ref(''),onlySelected=ref(false),manual=ref('')
const busy=ref(false),discovering=ref(false),error=ref(''),status=ref('')
let generation=0,disposed=false
const needsKey=computed(()=>authMode.value==='key')
const rows=computed(()=>models.value.filter(m=>(!onlySelected.value||selected.value.includes(m.id))&&(m.id+' '+m.label).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const allVisible=computed(()=>rows.value.length>0&&rows.value.every(m=>selected.value.includes(m.id)))
const mixedVisible=computed(()=>rows.value.some(m=>selected.value.includes(m.id))&&!allVisible.value)
watch(()=>JSON.stringify([authMode.value,url.value,apiKey.value,nodes.value,activeNode.value]),()=>{generation++;discovering.value=false;models.value=[];selected.value=[];discoveryReadId.value=null;status.value='';error.value=''})
watch(discoveryKind,()=>{generation++;discovering.value=false;discoveryReadId.value=null;error.value=''})
onBeforeUnmount(()=>{disposed=true;generation++;apiKey.value='';keyVisible.value=false})
function connectionError(){
 if(!label.value.trim()||[...label.value.trim()].length>80||new TextEncoder().encode(label.value.trim()).length>256)return '请填写渠道名称，名称不能过长'
 const existing=savedView.channelDraft?.channels??savedView.snapshot?.policy.channels??[]
 if(existing.filter(c=>c.id!==original?.id).some(c=>c.label.trim().toLowerCase()===label.value.trim().toLowerCase()))return '渠道名称已存在'
 try{const parsed=new URL(url.value);if(!['http:','https:'].includes(parsed.protocol)||!parsed.hostname||parsed.username||parsed.password||parsed.search||parsed.hash||/\/(responses|chat\/completions|messages)\/?$/.test(parsed.pathname))return '填写不含凭据、参数或生成接口路径的 HTTP/HTTPS 服务基地址'}catch{return '请填写有效服务基地址'}
 if(original){
   if(nodes.value.length<1||nodes.value.length>32||nodes.value.some(node=>{try{const u=new URL(node.url);return !['http:','https:'].includes(u.protocol)||!!u.username||!!u.password||!!u.search||!!u.hash||!node.name.trim()}catch{return true}}))return '请补齐每个节点的名称和有效服务基地址'
   const unchanged=JSON.stringify(nodes.value)===JSON.stringify(original.connection.nodes)&&activeNode.value===original.connection.activeNodeId&&authMode.value===(original.connection.authMode==='none'?'none':'key')&&!apiKey.value
   if(unchanged)return '连接尚未修改，可关闭返回当前工作区'
 }
 if(needsKey.value&&(!apiKey.value||apiKey.value.length>8192||/[\r\n]/.test(apiKey.value)))return '请输入有效 API Key'
 return ''
}
function addEditNode(){nodes.value.push({id:crypto.randomUUID(),name:'',url:''})}
function next(){const message=connectionError();if(message){error.value=message;return}error.value='';step.value=2}
async function discover(){
 if(busy.value||discovering.value)return
 const message=connectionError();if(message){error.value=message;step.value=1;return}
 if(!discoveryKind.value){error.value='请选择本次清单获取方式，或手动添加精确 ID。';return}
 const sequence=++generation;discovering.value=true;error.value='';status.value=''
 try{const result=await store.discover({kind:discoveryKind.value,url:url.value,authMode:authMode.value,apiKey:needsKey.value?apiKey.value:null})
  if(disposed||sequence!==generation)return
  const previous=models.value;models.value=result.models.map(m=>({id:m.id,label:previous.find(old=>old.id===m.id)?.label??m.label}));discoveryReadId.value=result.readId
  for(const old of previous)if(selected.value.includes(old.id)&&!models.value.some(m=>m.id===old.id))models.value.push(old)
  status.value=result.models.length?'已读取 '+result.models.length+' 个型号；新发现型号默认不选。':'清单为空，可重试、返回修改连接或手动添加精确 ID。'
 }catch(cause){if(!disposed&&sequence===generation)error.value=errorText(cause)}finally{if(sequence===generation)discovering.value=false}
}
function select(id:string,checked:boolean){selected.value=checked?[...new Set([...selected.value,id])]:selected.value.filter(v=>v!==id)}
function selectVisible(){const visible=new Set(rows.value.map(m=>m.id));selected.value=allVisible.value?selected.value.filter(id=>!visible.has(id)):[...new Set([...selected.value,...visible])]}
function addManual(){const value=manual.value;if(!value||value.trim()!==value||new TextEncoder().encode(value).length>2048||/[\u0000-\u001f]/.test(value)){error.value='请输入完整且不含首尾空白的精确模型 ID';return}if(models.value.some(m=>m.id===value)){error.value='此精确模型 ID 已在清单中';return}models.value.push({id:value,label:value});manual.value='';error.value=''}
async function join(){
 if(busy.value||discovering.value)return
 const message=connectionError();if(message){error.value=message;step.value=1;return}
 if(!selected.value.length){error.value='请选择至少一个精确型号';return}
 busy.value=true;error.value=''
 try{
  if(original&&JSON.stringify(savedView.channelDraft?.channels.find(c=>c.id===original.id))!==JSON.stringify(original))throw new Error('原渠道草稿已变化，请关闭后重新编辑连接。')
  const intent:ChannelConnectionEditIntentV2={channelId:id,kind:original?'edit_connection':'create',label:label.value.trim(),nodes:original?JSON.parse(JSON.stringify(nodes.value)):[{id:nodeId,name:'默认节点',url:url.value}],activeNodeId:original?activeNode.value:nodeId,authMode:authMode.value,selectedModelIds:[...selected.value],modelOrder:[...new Set([...(original?.modelOrder??[]),...models.value.map(m=>m.id)])],modelLabels:Object.fromEntries(models.value.map(m=>[m.id,m.label])),discoveryReadId:discoveryReadId.value}
  const result=await store.stageConnection(intent,needsKey.value?{action:'replace',value:apiKey.value}:{action:'unchanged'})
  if(disposed)return
  apiKey.value='';emit('added',result.channelId);emit('close')
 }catch(cause){if(!disposed)error.value=errorText(cause)}finally{busy.value=false}
}
function close(){if(busy.value)return;generation++;apiKey.value='';keyVisible.value=false;emit('close')}
</script>
<template>
 <UiDialog :open="true" wide :title="original?'编辑渠道连接':'新增渠道'" :busy="busy" @close="close">
  <p role="status">{{ step===1?'1 / 2 · 连接配置':'2 / 2 · 选择模型' }}</p>
  <div v-if="step===1" class="connection-fields">
   <label>渠道名称<input v-model="label" class="input" maxlength="80" :disabled="!!original" /></label>
   <template v-if="original"><label>当前节点<UiSelectField v-model="activeNode" label="编辑连接的当前节点" :options="nodes.map(n=>({value:n.id,label:n.name||'未命名节点'}))" /></label><div v-for="node in nodes" :key="node.id" class="model-columns"><label>节点名称<input v-model="node.name" class="input" /></label><label>节点服务基地址<input v-model="node.url" class="input" type="url" :aria-label="node.name+'编辑地址'" /></label><UiButton size="sm" variant="danger" :disabled="nodes.length<2||node.id===activeNode" @click="nodes=nodes.filter(n=>n.id!==node.id)">删除节点</UiButton></div><UiButton size="sm" :disabled="nodes.length>=32" @click="addEditNode">新增节点</UiButton><p>修改连接后重新获取并选择型号；请重新输入用于这些节点的Key，不会把原Key自动发到新地址。</p></template>
   <label v-else>服务基地址<input v-model="url" class="input" type="url" autocapitalize="off" spellcheck="false" placeholder="https://api.example.com/v1" /></label>
   <label>认证方式<UiSelectField v-model="authMode" label="渠道认证方式" :options="[{value:'key',label:'API Key'},{value:'none',label:'无需认证'}]" /></label>
   <label v-if="needsKey">API Key<input v-model="apiKey" aria-label="API Key" class="input" :type="keyVisible?'text':'password'" autocomplete="new-password" autocapitalize="off" spellcheck="false" /></label>
   <div v-if="needsKey" class="model-inline"><UiButton size="sm" :aria-pressed="keyVisible" @click="keyVisible=!keyVisible">{{keyVisible?'隐藏 Key':'显示 Key'}}</UiButton><UiButton size="sm" :disabled="!apiKey" @click="apiKey=''">清空 Key</UiButton></div>
   <p class="model-muted">填写服务基地址，不包含 responses、chat/completions 或 messages 等生成接口路径。</p>
  </div>
  <template v-else>
   <div class="model-field-title"><strong>{{ label }}</strong><UiButton :loading="discovering" :disabled="busy||!store.canDiscover" @click="discover">获取模型列表</UiButton><UiButton v-if="discovering" @click="generation++;discovering=false">取消等待</UiButton></div>
   <label>本次清单获取方式<UiSelectField v-model="discoveryKind" label="本次清单获取方式" :disabled="busy||discovering" :options="[{value:'',label:'请选择本次获取方式'},{value:'compatible_models',label:'兼容模型清单'},{value:'google_models',label:'Google 模型清单'},{value:'ollama_models',label:'Ollama 模型清单'}]" /></label>
   <p class="model-muted">只影响本次清单读取，不保存为渠道或模型请求协议。未选择时仍可手动添加精确 ID。</p>
   <p v-if="!store.canDiscover" role="status" class="model-muted">清单获取暂不可用，可以继续手动填写精确 ID。</p>
   <div class="model-inline"><input v-model="query" class="input" type="search" aria-label="新渠道型号搜索" placeholder="搜索型号或名称" /><label><input v-model="onlySelected" type="checkbox" :disabled="discovering" />仅已选</label></div>
   <p>已选 {{ selected.length }} / {{ models.length }} 项<span v-if="query||onlySelected"> · 当前显示 {{ rows.length }} 项</span></p>
   <label><input type="checkbox" :checked="allVisible" :indeterminate="mixedVisible" :disabled="discovering||!rows.length" @change="selectVisible" />选择当前结果</label>
   <div class="new-channel-models"><label v-for="model in rows" :key="model.id"><input type="checkbox" :checked="selected.includes(model.id)" :disabled="discovering" :aria-label="'选择新渠道型号 '+model.id" @change="select(model.id,($event.target as HTMLInputElement).checked)" /><span>{{ model.label||model.id }}<code v-if="model.label!==model.id">{{ model.id }}</code></span></label><p v-if="!rows.length">暂无型号；可获取清单或手动添加精确ID。</p></div>
   <details><summary>手动添加精确ID</summary><label>原始模型ID<input v-model="manual" class="input" :disabled="discovering" autocapitalize="off" spellcheck="false" /></label><UiButton size="sm" :disabled="discovering||!manual" @click="addManual">添加ID</UiButton></details>
  </template>
  <p v-if="step===2&&!store.canStageConnection" class="model-muted" role="status">渠道草稿服务暂不可用，当前输入已保留。</p>
  <p v-if="status" role="status" class="model-muted">{{ status }}</p><p v-if="error" role="alert" class="model-error">{{ error }}</p>
  <template #actions="{ close: requestClose }"><UiButton :disabled="busy" @click="requestClose">取消</UiButton><UiButton v-if="step===2" :disabled="busy" @click="generation++;discovering=false;step=1">上一步</UiButton><UiButton v-if="step===1" variant="primary" :disabled="busy" @click="next">下一步</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="discovering||!selected.length||!store.canStageConnection" @click="join">{{ original?'确认连接并加入草稿':'加入草稿' }}</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.connection-fields{display:grid;grid-template-columns:minmax(0,1fr);gap:var(--space-3)}.connection-fields .model-columns{grid-template-columns:minmax(0,1fr)}.connection-fields>label{display:grid;gap:var(--space-2);min-width:0}.connection-fields>.input,.connection-fields input{width:100%;min-width:0;box-sizing:border-box}.new-channel-models{max-height:320px;overflow:auto;min-width:0}.new-channel-models label{display:flex;gap:var(--space-2);align-items:flex-start;padding-block:var(--space-2)}.new-channel-models span,.new-channel-models code{min-width:0;overflow-wrap:anywhere}.new-channel-models code{display:block;font-family:var(--font-code);color:var(--muted)}
</style>
