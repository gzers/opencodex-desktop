<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsStore} from './store'
import {emptyRules,errorText,modelApi,type DiscoveredModel,type ChannelPolicy,type NewChannelOutcome} from './api'
const emit=defineEmits<{close:[];added:[string]}>()
const store=useModelsStore()
const step=ref(1),label=ref(''),providerId=ref(''),protocol=ref('openai-responses'),url=ref(''),apiKey=ref(''),privateNetwork=ref(false)
const id=crypto.randomUUID(),nodeId=crypto.randomUUID()
const models=ref<DiscoveredModel[]>([]),selected=ref<string[]>([]),query=ref(''),onlySelected=ref(false),manual=ref('')
const ready=ref(false)
const busy=ref(false),discovering=ref(false),frozen=ref(false),operationId=ref(''),error=ref(''),status=ref('')
let generation=0,disposed=false
const needsKey=computed(()=>protocol.value!=='ollama-native')
const rows=computed(()=>models.value.filter(m=>(!onlySelected.value||selected.value.includes(m.id))&&(m.id+' '+m.label).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const allVisible=computed(()=>rows.value.length>0&&rows.value.every(m=>selected.value.includes(m.id)))
const mixedVisible=computed(()=>rows.value.some(m=>selected.value.includes(m.id))&&!allVisible.value)
watch([protocol,url,apiKey,privateNetwork],()=>{generation++;discovering.value=false;models.value=[];selected.value=[];status.value='';error.value=''})
onBeforeUnmount(()=>{disposed=true;generation++;apiKey.value=''})
onMounted(async()=>{try{const previous=await modelApi.latestNewChannel();if(!disposed&&previous){operationId.value=previous.id;frozen.value=true;step.value=2;label.value=previous.channel.label;status.value='上次加入结果待核对，请沿原操作继续。';error.value=previous.error?errorText(previous.error):''}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{ready.value=true}})
function connectionError(){
 if(!label.value.trim()||label.value.trim().length>256)return '请填写渠道名称，名称不能过长'
 if(!/^[A-Za-z0-9_-]{1,128}$/.test(providerId.value)||['openai','chatgpt','openai-multi','combo'].includes(providerId.value.toLowerCase()))return '运行标识仅支持字母、数字、连字符和下划线，且不能使用原生保留标识'
 const existing=store.channelDraft?.channels??store.snapshot?.policy.channels??[]
 if(existing.some(c=>c.label.trim().toLowerCase()===label.value.trim().toLowerCase()||c.connection.providerId.toLowerCase()===providerId.value.toLowerCase()))return '渠道名称或运行标识已存在'
 try{const parsed=new URL(url.value);if(!['http:','https:'].includes(parsed.protocol)||!parsed.hostname||parsed.username||parsed.password||parsed.search||parsed.hash||/\/(responses|chat\/completions|messages)\/?$/.test(parsed.pathname))return '填写不含凭据、参数或生成接口路径的HTTP/HTTPS服务基地址'}catch{return '请填写有效服务基地址'}
 if(needsKey.value&&(!apiKey.value||apiKey.value.length>8192||/[\r\n]/.test(apiKey.value)))return '请输入有效API Key'
 return ''
}
function next(){const message=connectionError();if(message){error.value=message;return}error.value='';step.value=2}
async function discover(){
 if(busy.value||frozen.value||discovering.value)return
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
function outcome(result:NewChannelOutcome){
 if(result.phase==='committed'&&result.snapshot){store.acceptTransfer(result.snapshot);apiKey.value='';emit('added',result.channel.id);emit('close');return}
 error.value=result.error?errorText(result.error):'加入结果待核对'
 if(result.phase==='failed'){frozen.value=false;operationId.value='';status.value='原加入未完成；输入保留，临时凭据已清理。'}
 else{frozen.value=true;status.value='请核对原加入结果；关闭窗口不会撤回可能已经加入的渠道。'}
}
async function join(){
 if(busy.value||discovering.value)return
 if(frozen.value){await check();return}
 const message=connectionError();if(message){error.value=message;step.value=1;return}
 if(!selected.value.length){error.value='请选择至少一个精确型号';return}
 busy.value=true;store.busy=true;error.value=''
 try{
  await store.persistDraft();if(store.channelError)throw {message:store.channelError}
  if(!store.snapshot)throw {message:'请先读取本机渠道工作区'}
  const channel:ChannelPolicy={id,label:label.value.trim(),templateRefs:[],modelRefs:{},connection:{providerId:providerId.value,protocol:protocol.value,nodes:[{id:nodeId,name:'默认节点',url:url.value}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:privateNetwork.value},selection:{mode:'whitelist',models:[...selected.value]},modelOrder:models.value.map(m=>m.id),discovered:JSON.parse(JSON.stringify(models.value)),defaults:emptyRules(),overrides:{},familyOverrides:{}}
  operationId.value=crypto.randomUUID();frozen.value=true
  outcome(await modelApi.addNewChannel({id:operationId.value,expectedPolicy:store.snapshot.policyDigest,expectedWorkspace:store.snapshot.workspaceDigest,channel,apiKey:needsKey.value?apiKey.value:null}))
 }catch(cause){error.value=errorText(cause);const code=typeof cause==='object'&&cause&&'code'in cause?String(cause.code):'';if(['stale_workspace','new_channel_boundary','new_channel_duplicate','new_channel_selection','credential','channel','provider','node_url','nodes','active_node','model_order','model_identity','new_channel_limit'].includes(code)){frozen.value=false;operationId.value=''}if(code==='new_channel_unfinished'){const previous=await modelApi.latestNewChannel();if(previous){operationId.value=previous.id;frozen.value=true}}}
 finally{busy.value=false;store.busy=false}
}
async function check(){if(busy.value||!operationId.value)return;busy.value=true;store.busy=true;error.value='';try{outcome(await modelApi.newChannelResult(operationId.value))}catch(cause){error.value=errorText(cause);if(typeof cause==='object'&&cause&&'code'in cause&&cause.code==='new_channel_not_found'){frozen.value=false;operationId.value=''}}finally{busy.value=false;store.busy=false}}
function close(){if(busy.value)return;generation++;apiKey.value='';emit('close')}
</script>
<template>
 <UiDialog :open="true" wide title="新增渠道" :busy="busy" @close="close">
  <p role="status">{{ frozen?'原加入结果待核对':step===1?'1 / 2 · 连接配置':'2 / 2 · 选择模型' }}</p>
  <template v-if="!frozen&&step===1">
   <div class="model-columns"><label>渠道名称<input v-model="label" class="input" maxlength="256" /></label><label>运行渠道标识<input v-model="providerId" class="input" maxlength="128" autocapitalize="off" spellcheck="false" /></label></div>
   <label>默认协议<UiSelectField v-model="protocol" label="新渠道默认协议" :options="['openai-responses','openai-chat','anthropic','google','ollama-native'].map(value=>({value,label:value}))" /></label>
   <label>服务基地址<input v-model="url" class="input" type="url" autocapitalize="off" spellcheck="false" placeholder="https://api.example.com/v1" /></label>
   <label v-if="needsKey">API Key<input v-model="apiKey" class="input" type="password" autocomplete="new-password" autocapitalize="off" spellcheck="false" /></label>
   <label><input v-model="privateNetwork" type="checkbox" />允许访问可信私有网络</label>
  </template>
  <template v-else-if="!frozen">
   <div class="model-field-title"><strong>{{ label }}</strong><UiButton :loading="discovering" :disabled="busy" @click="discover">获取模型列表</UiButton><UiButton v-if="discovering" @click="generation++;discovering=false">取消等待</UiButton></div>
   <div class="model-inline"><input v-model="query" class="input" type="search" aria-label="新渠道型号搜索" placeholder="搜索型号或名称" /><label><input v-model="onlySelected" type="checkbox" :disabled="discovering" />仅已选</label></div>
   <p>已选 {{ selected.length }} / {{ models.length }} 项<span v-if="query||onlySelected"> · 当前显示 {{ rows.length }} 项</span></p>
   <label><input type="checkbox" :checked="allVisible" :indeterminate="mixedVisible" :disabled="discovering||!rows.length" @change="selectVisible" />选择当前结果</label>
   <div class="new-channel-models"><label v-for="model in rows" :key="model.id"><input type="checkbox" :checked="selected.includes(model.id)" :disabled="discovering" :aria-label="'选择新渠道型号 '+model.id" @change="select(model.id,($event.target as HTMLInputElement).checked)" /><span>{{ model.label||model.id }}<code v-if="model.label!==model.id">{{ model.id }}</code></span></label><p v-if="!rows.length">暂无型号；可获取清单或手动添加精确ID。</p></div>
   <details><summary>手动添加精确ID</summary><label>原始模型ID<input v-model="manual" class="input" :disabled="discovering" autocapitalize="off" spellcheck="false" /></label><UiButton size="sm" :disabled="discovering||!manual" @click="addManual">添加ID</UiButton></details>
  </template>
  <p v-if="status" role="status" class="model-muted">{{ status }}</p><p v-if="error" role="alert" class="model-error">{{ error }}</p>
  <template #actions><UiButton :disabled="busy" @click="close">关闭</UiButton><UiButton v-if="!frozen&&step===2" :disabled="busy" @click="generation++;discovering=false;step=1">上一步</UiButton><UiButton v-if="!frozen&&step===1" variant="primary" :disabled="!ready" @click="next">下一步</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="discovering||(!frozen&&!selected.length)" @click="join">{{ frozen?'核对原加入结果':'加入草稿' }}</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.new-channel-models{max-height:320px;overflow:auto;min-width:0}.new-channel-models label{display:flex;gap:var(--space-2);align-items:flex-start;padding-block:var(--space-2)}.new-channel-models span,.new-channel-models code{min-width:0;overflow-wrap:anywhere}.new-channel-models code{display:block;font-family:var(--font-code);color:var(--muted)}
</style>
