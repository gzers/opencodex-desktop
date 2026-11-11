<script setup lang="ts">
import {onBeforeUnmount,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import {checkNetworkProxy,type NetworkProbeResult} from './api'
const props=defineProps<{policyKey:string}>()
const busy=ref(false),results=ref<NetworkProbeResult[]>([]),error=ref('')
let epoch=0,disposed=false
const labels:Record<NetworkProbeResult['state'],string>={ready:'连接与来源核验通过',timeout:'请求超时',rate_limited:'请求受限流',http_error:'服务返回错误',network:'连接失败',identity_mismatch:'响应来源不匹配',too_large:'响应超过大小限制',invalid_proxy:'代理或网络客户端配置无效'}
function label(row:NetworkProbeResult){return labels[row.state]+(row.httpStatus?' · HTTP '+row.httpStatus:'')+(row.retryAfterSeconds!==null?' · 建议 '+row.retryAfterSeconds+' 秒后重试':'')}
function time(value:string){const date=new Date(value);return Number.isNaN(date.getTime())?'检查时间未知':date.toLocaleString()}
watch(()=>props.policyKey,()=>{epoch++;results.value=[];error.value=''})
onBeforeUnmount(()=>{disposed=true;epoch++})
async function check(){
 if(busy.value)return
 const own=++epoch;busy.value=true;error.value='';results.value=[]
 try{
  const value=await checkNetworkProxy()
  if(own!==epoch||disposed)return
  if(!Array.isArray(value.results)||value.results.length!==2||new Set(value.results.map(r=>r.target)).size!==2||value.results.some(r=>!['github','npm'].includes(r.target)||!(r.state in labels)))throw Error('invalid diagnostic response')
  results.value=value.results
 }catch{if(own===epoch&&!disposed)error.value='检查结果尚未确认，请重新检查。'}
 finally{if(!disposed)busy.value=false}
}
</script>
<template>
 <div class="setting-row" data-testid="setting-network-probe"><div><div class="setting-title">检查连接</div><div class="setting-desc">分别检查 GitHub 和 npm，使用当前已保存的应用代理设置。</div></div><div class="controls"><UiButton variant="ghost" :disabled="busy" @click="check">{{busy?'检查中':'检查连接'}}</UiButton></div></div>
 <p v-if="error" class="model-error" role="alert">{{error}}</p>
 <div v-if="results.length" class="network-probe-results" role="status" aria-label="网络连接检查结果">
  <div v-for="row in results" :key="row.target" class="network-probe-row"><strong>{{row.target==='github'?'GitHub':'npm'}}</strong><span :class="{'model-error':row.state!=='ready'}">{{label(row)}}</span><time :datetime="row.checkedAt">{{time(row.checkedAt)}}</time></div>
 </div>
</template>
<style scoped>
.network-probe-results{display:grid;gap:var(--space-3);padding-block:var(--space-3)}.network-probe-row{display:grid;grid-template-columns:5em minmax(0,1fr);gap:var(--space-2) var(--space-3)}.network-probe-row time{grid-column:2;color:var(--muted);font-size:var(--text-label)}.network-probe-row span{overflow-wrap:anywhere}
</style>
