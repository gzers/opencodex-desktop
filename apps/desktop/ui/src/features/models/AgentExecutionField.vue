<script setup lang="ts">
import { computed,onBeforeUnmount,onMounted,ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import ModelPicker,{type ModelChoice} from './ModelPicker.vue'
import { modelApi,errorText,type AgentSettings,type DraftProjection,type ChannelPolicy } from './api'
import { mergeUpstream } from './upstream'
const settings=defineModel<AgentSettings|null>({default:null})
defineProps<{disabled?:boolean}>()
const error=ref('');const busy=ref(false);const open=ref(false);const channels=ref<ChannelPolicy[]>([]);const directory=ref<DraftProjection|null>(null)
let disposed=false
onBeforeUnmount(()=>{disposed=true})
onMounted(async()=>{try{const value=await modelApi.agentSettings();if(!disposed)settings.value=value}catch(cause){if(!disposed)error.value=errorText(cause)}})
const current=computed<ModelChoice[]>(()=>{const value=settings.value;const channel=channels.value.find(c=>c.connection.providerId===value?.providerId);return value&&channel?[{channelId:channel.id,modelId:value.modelId,...(value.reasoningEffort===null?{}:{reasoningEffort:value.reasoningEffort}),...(value.fast===null?{}:{fast:value.fast})}]:[]})
async function configure(){if(busy.value)return;busy.value=true;error.value='';try{const snapshot=await modelApi.upstream();const values=mergeUpstream(snapshot,[]);const preview=await modelApi.project({schemaVersion:1,templateRefs:[],channels:values});if(!disposed){channels.value=values;directory.value=preview;open.value=true}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
async function save(picks:ModelChoice[]){const pick=picks[0];const channel=channels.value.find(c=>c.id===pick?.channelId);if(!pick||!channel||busy.value)return;busy.value=true;error.value='';try{const value=await modelApi.saveAgentSettings({providerId:channel.connection.providerId,modelId:pick.modelId,reasoningEffort:pick.reasoningEffort??null,fast:pick.fast??null});if(!disposed){settings.value=value;open.value=false}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
</script>
<template>
 <div class="model-execution-field"><UiButton :disabled="disabled" :loading="busy" @click="configure">{{ settings?settings.providerId+' / '+settings.modelId:'配置执行模型' }}</UiButton><small v-if="settings">{{ settings.reasoningEffort??'默认推理' }} · {{ settings.fast===null?'默认Fast':settings.fast?'Fast开':'Fast关' }}</small><p v-if="error" role="alert" class="model-error">{{ error }}</p><ModelPicker :open="open" :rows="directory?.rows??[]" :channels="channels" :current="current" reasoning title="执行模型" @close="open=false" @confirm="save" /></div>
</template>
<style scoped>
.model-execution-field{display:grid;gap:6px;min-width:0}.model-execution-field small{color:var(--muted)}.model-error{color:var(--red);overflow-wrap:anywhere}
</style>
