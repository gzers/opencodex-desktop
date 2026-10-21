<script setup lang="ts">
import {computed,onMounted,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {errorText,modelApi,type ModelClientState} from './api'
const props=defineProps<{refreshKey?:string}>()
const emit=defineEmits<{change:[state:ModelClientState]}>()
const state=ref<ModelClientState|null>(null);const busy=ref(false);const error=ref('');const opened=ref(false);const confirmed=ref(false)
const label=computed(()=>({not_required:'本次无需重启',restart_required:'已写入 · 待重启 Codex',pending_verification:'运行目录一致 · 待客户端验证',user_confirmed:'用户已确认客户端结果',runtime_changed:'运行配置已变化 · 待重新比较'}[state.value?.status??'not_required']))
async function refresh(){try{state.value=await modelApi.clientStatus()}catch(cause){error.value=errorText(cause)}}
async function check(confirm=false){if(!state.value?.planId||busy.value)return;busy.value=true;error.value='';try{state.value=await modelApi.checkClient(state.value.planId,confirm?state.value.checkId??undefined:undefined);confirmed.value=false}catch(cause){error.value=errorText(cause)}finally{busy.value=false}}
watch(()=>props.refreshKey,()=>void refresh())
watch(state,value=>{if(value)emit('change',value)})
onMounted(()=>void refresh())
</script>
<template>
 <div v-if="state?.planId||error" class="model-client-status"><span class="tag" :data-tone="state?.status==='runtime_changed'?'danger':'neutral'">{{ label }}</span><UiButton size="sm" @click="opened=true">客户端加载核对</UiButton></div>
 <UiDialog :open="opened" title="核对 Codex 客户端加载" :busy="busy" @close="opened=false">
  <p>{{ label }}</p><p>保存完成后，请先结束或保留当前会话，再手动退出并重新打开 Codex 客户端；管理器不会自动关闭客户端。</p><p>重新打开模型选择器，检查本次模型、顺序、上下文、推理和快速模式。下方检查只验证官方运行配置与目录，客户端所见结果需单独确认。</p>
  <p v-if="state?.checkedAt">最近运行核对：{{ new Date(state.checkedAt).toLocaleString() }}</p>
  <p v-if="state?.confirmedAt">用户确认：{{ new Date(state.confirmedAt).toLocaleString() }}</p>
  <label v-if="state?.status==='pending_verification'"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已重新打开 Codex，并在模型选择器确认本次配置显示正确</label>
  <p v-if="error" class="model-error" role="alert">{{ error }}</p>
  <template #actions><UiButton @click="opened=false">稍后</UiButton><UiButton :loading="busy" :disabled="!state?.planId" @click="check()">我已重启，重新检查</UiButton><UiButton v-if="state?.status==='pending_verification'" variant="primary" :disabled="!confirmed" :loading="busy" @click="check(true)">记录用户确认</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.model-client-status{display:flex;align-items:center;flex-wrap:wrap;gap:10px;margin:12px 0}.model-error{color:var(--red)}label{display:flex;align-items:flex-start;gap:8px}
</style>
