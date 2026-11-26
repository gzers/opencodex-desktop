<script setup lang="ts">
import { computed, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useRouteStore } from '@/stores/routes'
import { usePreparationStore, type PreparationPhase } from '../preparation'
import { useEnvironmentStore } from '../store'

const props = defineProps<{ inSettings?: boolean; actionsDisabled?: boolean }>()
const preparation = usePreparationStore()
const environment = useEnvironmentStore()
const routes = useRouteStore()
const steps: PreparationPhase[] = ['preparing', 'verifying', 'extracting', 'assembling', 'activating']
const labels: Record<PreparationPhase, string> = {
  idle: '', checking: '核对内置程序状态', preparing: '准备内置运行环境', verifying: '校验运行归档',
  downloading: '下载运行环境',
  extracting: '展开运行包与依赖', assembling: '核对程序与固定依赖', activating: '核对程序入口',
  supplied: '内置程序已供应', failed: '内置运行环境准备失败', removed: '运行程序已卸载', review: '内置程序需要核对',
}
const failures = {
  integrity: '运行资源校验未通过，原内容已保留。',
  busy: '已有程序操作正在进行，请完成后重试。',
  recovery_required: '原程序操作尚待核对，请到安装配置继续原操作。',
  unavailable: '内置运行资源暂不可用，原内容已保留。',
  missing_archive: '缺少运行包。你可以联网下载，或稍后处理。',
}
const state = computed(() => preparation.snapshot)
const confirmationAttempt = ref<string | null>(null)
const mayDownload = computed(() => !props.actionsDisabled && !preparation.retrying &&
  preparation.needsDownloadConsent && state.value?.attemptId === confirmationAttempt.value)
function confirmDownload() {
  const attemptId = confirmationAttempt.value
  if (!mayDownload.value || !attemptId) return
  confirmationAttempt.value = null
  void preparation.retry(attemptId)
}
const title = computed(() => preparation.error ? '内置程序准备状态未知' : labels[state.value?.phase ?? 'idle'])
const visible = computed(() => preparation.error || (state.value && state.value.phase !== 'idle'))
const showSteps = computed(() => !preparation.error && (preparation.active || state.value?.phase === 'failed'))
function stepLabel(index: number) {
  const snapshot = state.value!
  const current = steps.indexOf(snapshot.failedPhase ?? snapshot.phase)
  if (index < current) return '已完成'
  if (index === current) return snapshot.failure ? '未完成' : '进行中'
  return '待执行'
}
</script>

<template>
  <section v-if="visible" class="integrated-preparation" data-testid="integrated-preparation">
    <h2>{{ title }}</h2>
    <p v-if="state?.phase === 'downloading' && !preparation.error">正在联网下载运行包。</p>
    <p v-else-if="preparation.active && !preparation.error">正在处理本地运行资源，无需联网。</p>
    <ol v-if="showSteps" aria-label="内置环境准备阶段">
      <li v-for="(step, index) in steps" :key="step"><span>{{ labels[step] }}</span><b>{{ stepLabel(index) }}</b></li>
    </ol>
    <p v-if="preparation.error" role="alert">{{ preparation.error }}</p>
    <p v-else-if="state?.failure" role="alert">{{ failures[state.failure] }}</p>
    <p v-else-if="state?.phase === 'supplied'" role="status">{{ environment.ready ? '内置依赖检查通过。' : '等待内置依赖检查结果。' }}</p>
    <p v-if="preparation.retryError" role="alert">{{ preparation.retryError }}</p>
    <div class="integrated-preparation-actions">
      <UiButton v-if="preparation.error" @click="preparation.observe()">重新检查准备状态</UiButton>
      <UiButton v-if="preparation.needsDownloadConsent" :disabled="actionsDisabled || preparation.retrying" @click="confirmationAttempt = state?.attemptId ?? null">下载运行环境</UiButton>
      <UiButton v-else-if="preparation.canRetry" :disabled="actionsDisabled || preparation.retrying" @click="preparation.retry()">{{ preparation.retrying ? '准备中' : '重新准备内置环境' }}</UiButton>
      <UiButton v-if="!inSettings" variant="ghost" @click="routes.go('settings', { section: 'installation' })">查看安装配置</UiButton>
      <slot />
    </div>
    <UiDialog :open="confirmationAttempt !== null" title="下载运行环境" initial-focus="dialog" @close="confirmationAttempt = null">
      <p>下载需要联网。运行包通过校验后将在本机安装。</p>
      <template #actions>
        <UiButton variant="ghost" @click="confirmationAttempt = null">稍后处理</UiButton>
        <UiButton variant="primary" :disabled="!mayDownload" @click="confirmDownload">确认下载</UiButton>
      </template>
    </UiDialog>
  </section>
</template>

<style scoped>
.integrated-preparation{display:grid;gap:var(--space-3)}.integrated-preparation h2,.integrated-preparation p{margin:0}.integrated-preparation p{color:var(--muted);line-height:1.6}.integrated-preparation ol{list-style:none;padding:0;margin:0;display:grid;gap:var(--space-2)}.integrated-preparation li{display:flex;justify-content:space-between;gap:var(--space-3);padding-block:var(--space-2);border-bottom:1px solid var(--border-soft)}.integrated-preparation-actions{display:flex;gap:var(--space-2);flex-wrap:wrap}
</style>
