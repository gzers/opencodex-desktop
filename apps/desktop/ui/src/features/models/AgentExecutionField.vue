<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ModelPicker,{type ModelChoice} from './ModelPicker.vue'
import {modelApi,errorText,type AgentSettings,type ExecutionDirectory} from './api'
const settings=defineModel<AgentSettings|null>({default:null})
defineProps<{disabled?:boolean;compact?:boolean}>()
const error=ref('');const busy=ref(false);const open=ref(false);const configuring=ref(false)
const candidate=ref<AgentSettings|null>(null);const directory=ref<ExecutionDirectory|null>(null)
const directoryLoaded=ref(false)
const channels=computed(()=>directory.value?.channels??[])
let disposed=false
onBeforeUnmount(()=>{disposed=true})
onMounted(async()=>{busy.value=true;try{const value=await modelApi.agentSettings();if(!disposed)settings.value=value}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}})
const current=computed<ModelChoice[]>(()=>{const value=candidate.value;const channel=channels.value.find(c=>c.providerId===value?.providerId);return value?[{channelId:value.channelId??channel?.id??value.providerId,modelId:value.modelId,...(value.reasoningEffort===null?{}:{reasoningEffort:value.reasoningEffort}),...(value.fast===null?{}:{fast:value.fast})}]:[]})
function configure(){if(busy.value)return;candidate.value=settings.value?{...settings.value}:null;error.value='';configuring.value=true}
async function browse(){if(busy.value)return;open.value=true;if(directoryLoaded.value)return;busy.value=true;error.value='';try{const actual=await modelApi.executionDirectory();if(!disposed){directory.value=actual;directoryLoaded.value=true}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
function choose(picks:ModelChoice[]){const pick=picks[0];if(!pick||pick.nativeModelId)return;const row=directory.value?.rows.find(r=>r.channelId===pick.channelId&&r.modelId===pick.modelId);if(!row||row.unavailableReason||!row.enabled)return;candidate.value={channelId:pick.channelId,connectionDigest:row.connectionDigest,providerId:row.providerId,modelId:pick.modelId!,reasoningEffort:pick.reasoningEffort??null,fast:pick.fast??null};open.value=false}
async function save(){if(!candidate.value||busy.value)return;busy.value=true;error.value='';try{const value=await modelApi.saveAgentSettings({...candidate.value});if(!disposed){settings.value=value;configuring.value=false}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
function summary(value:AgentSettings){return (value.reasoningEffort??'默认推理')+' · '+(value.fast===null?'默认快速模式':value.fast?'快速模式已开启':'快速模式已关闭')}
</script>
<template>
 <div class="model-execution-field" :class="{'model-execution-compact':compact}"><UiButton :disabled="disabled" :loading="busy" @click="configure" :title="settings?settings.providerId+' / '+settings.modelId:undefined">Agent 配置</UiButton><small v-if="settings&&!compact">{{ settings.providerId }} / {{ settings.modelId }} · {{ summary(settings) }}</small><p v-if="error&&!configuring" role="alert" class="model-error">{{ error }}</p>
  <UiDialog :open="configuring" title="Agent 配置" :busy="busy" @close="configuring=false"><p>模型模版助手 · 本机共用</p><UiButton :loading="busy" @click="browse">{{ candidate?candidate.providerId+' / '+candidate.modelId:'选择执行模型' }}</UiButton><p v-if="candidate">{{ summary(candidate) }}</p><p>此设置用于下一次自动填充、持续纠偏和批量模版检查。</p><p v-if="error" role="alert" class="model-error">{{ error }}</p><template #actions><UiButton @click="configuring=false">取消</UiButton><UiButton variant="primary" :disabled="!candidate" :loading="busy" @click="save">保存共用设置</UiButton></template></UiDialog>
  <ModelPicker :open="open" :loading="busy" :error="open?error:undefined" @retry="browse" :rows="directory?.rows??[]" :channels="channels" :current="current" reasoning title="执行模型" @close="open=false" @confirm="choose" />
 </div>
</template>
<style scoped>
.model-execution-compact{width:max-content;max-width:100%}.model-execution-field{display:grid;gap:6px;min-width:0}.model-execution-field small{color:var(--muted);overflow-wrap:anywhere}.model-error{color:var(--red);overflow-wrap:anywhere}
</style>

