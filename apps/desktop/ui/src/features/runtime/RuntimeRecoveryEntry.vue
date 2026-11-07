<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import {useRuntimeRecoveryV2Store} from './recovery-v2'
defineProps<{disabled?:boolean}>()
const recovery=useRuntimeRecoveryV2Store()
</script>
<template><div class="runtime-recovery-entry"><UiButton data-testid="runtime-recovery-entry" :disabled="disabled" @click="recovery.start()">{{recovery.executing||recovery.status?.phase==='applying'?'查看恢复进度':'诊断与恢复程序'}}</UiButton><UiButton v-if="recovery.lastOperationId" variant="ghost" @click="recovery.observe()">查看原恢复记录</UiButton><UiButton v-if="recovery.status&&['succeeded','failed','cancelled'].includes(recovery.status.phase)" variant="ghost" :disabled="disabled||recovery.executing" @click="recovery.startNew()">重新诊断与预览</UiButton></div></template>
<style scoped>.runtime-recovery-entry{display:flex;gap:var(--space-2);flex-wrap:wrap}</style>
