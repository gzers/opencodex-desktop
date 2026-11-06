<script setup lang="ts">
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import {recoveryLabelsV2,type RecoveryActionV2,type RemoteStateV2,type CommitRefV2} from './api-v2'
defineProps<{open:boolean;state:RemoteStateV2;actions:readonly RecoveryActionV2[];heads?:readonly CommitRefV2[];busy?:boolean;error?:string;status?:string}>()
const emit=defineEmits<{close:[];action:[value:RecoveryActionV2];head:[value:CommitRefV2]}>()
</script>
<template>
  <UiDialog :open="open" :title="state==='initialization_ambiguous'?'目录初始化存在歧义':state==='conflict'?'选择并发配置来源':'同步恢复'" wide @close="emit('close')">
    <p v-if="state==='initialization_ambiguous'">发现多个初始化代次，已停止普通上传与下载应用。原目录及各分支材料保留；可查看后端提供的只读恢复方案，再明确选择另一个已核实空目录开始。</p>
    <p v-else-if="state==='conflict'">当前同一代次有多个已观察提交。请选择一个确切来源再查看完整影响，不按时间、名称或顺序选赢家。</p>
    <p v-else>恢复动作来自原操作或本次完整读取结果。读取失败、损坏和列举不完整不会被当成空目录。</p>
    <p v-if="busy" role="status">正在读取恢复信息…</p><p v-if="status" role="status">{{status}}</p><p v-if="error" class="model-error" role="alert">{{error}}</p>
    <section v-for="(head,index) in heads??[]" :key="head.operationId"><h3>并发配置 {{index+1}}</h3><details><summary>确切提交标识</summary><p>{{head.operationId}}</p><p>{{head.digest}}</p></details><UiButton :disabled="busy||state!=='conflict'" @click="emit('head',head)">查看此来源与变化</UiButton></section>
    <p v-if="!actions.length&&!heads?.length">尚无可安全执行的恢复动作；现有材料保持，可重新读取原目录。</p>
    <p>本页不会清空、搬迁或创建远端目录。专用导出或恢复完成后也不会自动上传，不能据此标记普通同步完成。</p>
    <template #actions><UiButton v-for="(action,index) in actions" :key="index" :disabled="busy" @click="emit('action',action)">{{recoveryLabelsV2[action.kind]}}</UiButton></template>
  </UiDialog>
</template>
<style scoped>p{line-height:1.6;overflow-wrap:anywhere}section{padding-block:12px;border-bottom:1px solid var(--border)}h3{font-size:var(--text-body)}.model-error{color:var(--red)}</style>
