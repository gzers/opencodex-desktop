<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import UiButton from '@/components/ui/UiButton.vue'
import {useEnvironmentStore} from '../store'
import {useRouteStore} from '@/stores/routes'
type Phase='idle'|'checking'|'preparing'|'verifying'|'extracting'|'assembling'|'activating'|'supplied'|'failed'|'removed'|'review'
interface Snapshot{attemptId:string|null;revision:number;phase:Phase;failedPhase:Phase|null;failure:'integrity'|'busy'|'recovery_required'|'unavailable'|null;retryable:boolean}
const emit=defineEmits<{observing:[boolean]}>()
const state=ref<Snapshot|null>(null),error=ref(''),retrying=ref(false),environment=useEnvironmentStore()
let disposed=false,timer:ReturnType<typeof setTimeout>|null=null,epoch=0,lastCompleted=''
const active=(p:Phase)=>['checking','preparing','verifying','extracting','assembling','activating'].includes(p)
const steps:Phase[]=['preparing','verifying','extracting','assembling','activating']
const labels:Partial<Record<Phase,string>>={checking:'核对内置程序状态',preparing:'准备内置运行环境',verifying:'校验运行归档',extracting:'展开运行包与依赖',assembling:'核对程序与固定依赖',activating:'核对程序入口',supplied:'内置程序已就绪',failed:'内置运行环境准备失败',removed:'运行程序已卸载',review:'内置程序需要核对'}
const visible=computed(()=>!!error.value||!!state.value&&state.value.phase!=='idle')
const title=computed(()=>labels[state.value?.phase??'checking'])
const failureText=computed(()=>({integrity:'运行资源校验未通过，原内容已保留。',busy:'已有程序操作正在进行，请完成后重试。',recovery_required:'原程序操作尚待核对，请到安装配置继续原操作。',unavailable:'内置运行资源暂不可用，原内容已保留。'}[state.value?.failure??'unavailable']))
function stepLabel(p:Phase){const s=state.value;if(!s)return '待执行';if(s.phase==='supplied')return '已完成';const current=s.failedPhase??s.phase;const index=steps.indexOf(current);const i=steps.indexOf(p);if(i<index)return '已完成';if(i===index)return s.failure?'未完成':'进行中';return '待执行'}
function settings(){useRouteStore().go('settings',{section:'installation'})}
function accept(value:Snapshot){
 if(!value||!['idle',...steps,'checking','supplied','failed','removed','review'].includes(value.phase)||!Number.isSafeInteger(value.revision))throw Error('preparation response invalid')
 if(state.value&&value.revision<state.value.revision)return
 state.value=value;emit('observing',value.phase!=='idle')
 if(value.phase==='supplied'&&value.attemptId!==lastCompleted){lastCompleted=value.attemptId??'';void environment.refresh()}
}
async function observe(){
 const own=++epoch
 try{const s=await invoke<Snapshot>('runtime_preparation_status');if(!disposed&&own===epoch){accept(s);error.value=''}}
 catch{if(!disposed&&own===epoch){error.value='准备状态暂不可读取，请重新检查。';emit('observing',true)}}
 finally{if(!disposed&&own===epoch&&state.value&&active(state.value.phase))timer=setTimeout(observe,600)}
}
async function retry(){
 const s=state.value;if(!s?.attemptId||!s.retryable||retrying.value)return
 retrying.value=true;error.value='';if(timer)clearTimeout(timer);epoch++
 try{const next=await invoke<Snapshot>('runtime_preparation_retry',{request:{attemptId:s.attemptId}});if(!disposed)accept(next)}
 catch{if(!disposed)error.value='本次准备未开始，请重新核对当前程序状态。'}
 finally{retrying.value=false;if(!disposed)void observe()}
}
onMounted(()=>{void observe()})
onBeforeUnmount(()=>{disposed=true;epoch++;if(timer)clearTimeout(timer)})
</script>
<template>
 <section v-if="visible" class="integrated-preparation" data-testid="integrated-preparation">
  <h2>{{title}}</h2><p>Node.js、npm、Bun 与 OpenCodex 随应用提供，无需另装系统 Node.js。</p>
  <ol v-if="state?.phase!=='removed'&&state?.phase!=='review'" aria-label="内置环境准备阶段"><li v-for="step in steps" :key="step"><span>{{labels[step]}}</span><b>{{stepLabel(step)}}</b></li></ol>
  <p v-if="state?.failure" role="alert">{{failureText}}</p>
  <p v-if="state?.phase==='supplied'" role="status">{{environment.loading?'正在检查内置依赖，尚未启动服务。':environment.report?.gate==='ready'?'内置依赖检查通过；服务运行状态单独显示。':'程序已供应，依赖检查尚未通过。'}}{{environment.error}}</p>
  <p v-if="error" role="alert">{{error}}</p>
  <div class="integrated-preparation-actions"><UiButton v-if="error&&!state" @click="observe">重新检查准备状态</UiButton><UiButton v-if="state?.retryable" :disabled="retrying" @click="retry">{{retrying?'准备中':'重新准备内置环境'}}</UiButton><UiButton v-if="state?.phase==='supplied'&&!environment.loading" @click="environment.refresh()">重新检查</UiButton><UiButton variant="ghost" @click="settings">查看安装配置</UiButton></div>
 </section>
</template>
<style scoped>
.integrated-preparation{display:grid;gap:var(--space-3)}.integrated-preparation h2,.integrated-preparation p{margin:0}.integrated-preparation p{color:var(--muted);line-height:1.6}.integrated-preparation ol{list-style:none;padding:0;margin:0;display:grid;gap:var(--space-2)}.integrated-preparation li{display:flex;justify-content:space-between;gap:var(--space-3);padding-block:var(--space-2);border-bottom:1px solid var(--border-soft)}.integrated-preparation-actions{display:flex;gap:var(--space-2);flex-wrap:wrap}
</style>
