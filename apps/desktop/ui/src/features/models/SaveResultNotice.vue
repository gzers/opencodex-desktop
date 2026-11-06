<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'
import TaskProgressCard from '@/components/patterns/TaskProgressCard.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useFeedbackStore } from '@/app/feedback/store'
import { createCountdown } from '@/app/feedback/countdown'
import { TERMINAL_TTL_MS } from '@/app/feedback/tasks'
import { cancelLeave, leave } from '@/app/feedback/motion'
import { errorText, modelApi } from './api'

// 由全局 TaskProgressHost 承载，关闭后不留下任务或待重启状态。
const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const feedback = useFeedbackStore()
const confirming = ref(false)
const busy = ref(false)
const restartError = ref('')
let disposed = false
// 原型 scheduleTaskCollapse：「已写入」卡与其它终态任务卡同为 8 秒成功倒计时，到点收起到铃铛。
// 每次保存由宿主按 revision 重建本组件，倒计时随实例独立。
const NOTICE_KEY = 'model-save:' + useId()
const countdown = createCountdown({
  durationMs: TERMINAL_TTL_MS,
  tone: 'success',
  onElapsed: () => leave(NOTICE_KEY, () => { if (!disposed && !busy.value) emit('close') }),
})
// 重启确认框打开或重启进行中时不允许自动收起，否则会连同确认框一起关掉。
watch([confirming, busy], async ([open, running], [wasOpen]) => {
  if (open || running) { countdown.pause('dialog'); return }
  // 等公共弹窗完成焦点归还；自动归还不算主动进入通知（G0 releaseProgrammaticFocus）。
  await nextTick(); await nextTick()
  if (disposed || confirming.value || busy.value) return
  if (wasOpen) countdown.resume('focus')
  countdown.resume('dialog')
})

// 成功通知保留用户当前编辑焦点；主动进入卡片的焦点由公共倒计时登记暂停。
watch(() => props.open, open => { if (!open) confirming.value = false })

function later() { if (!busy.value) emit('close') }
function onEscape(event: KeyboardEvent) {
  // UiDialog 自己处理原生 cancel；其它弹窗的 Esc 不消费保存反馈。
  if (event.defaultPrevented || event.key !== 'Escape' || !props.open || confirming.value || document.querySelector('dialog[open], :not(dialog)[aria-modal="true"]')) return
  event.preventDefault()
  later()
}
onMounted(() => window.addEventListener('keydown', onEscape))
onBeforeUnmount(() => {
  disposed = true
  countdown.cancel()
  cancelLeave(NOTICE_KEY)
  window.removeEventListener('keydown', onEscape)
})

async function restart() {
  if (busy.value) return
  busy.value = true
  restartError.value = ''
  countdown.resume('restart-error')
  let completed = false
  try {
    const result = await modelApi.refreshCache(true)
    if (disposed) return
    completed = result.success && result.restarted
    if (completed) feedback.showToast(result.summary, { kind: 'info' })
    else restartError.value = result.summary || '重启结果尚未确认；已保存配置保持，可检查服务状态后重试。'
  } catch (cause) {
    if (!disposed) restartError.value = errorText(cause)
  } finally {
    if (restartError.value) countdown.pause('restart-error')
    busy.value = false
    if (completed) confirming.value = false
    // 新一次保存可能已替换本提示；旧请求不能关闭新反馈。
    if (!disposed && completed) emit('close')
  }
}
</script>

<template>
  <TaskProgressCard
    v-if="open"
    title="已写入"
    :message="restartError ? '配置已保存；重启尚未完成，仍可重试。' : ''"
    phase="done"
    :show-track="false"
    :show-collapse="false"
    :notification-key="NOTICE_KEY"
    :countdown="countdown"
    @keydown="countdown.pause('focus')"
  >
    <template #actions>
      <UiButton class="notif-ctl" :disabled="busy" @click="confirming = true">立即重启 Codex</UiButton>
      <UiButton class="notif-ctl" variant="ghost" :disabled="busy" @click="later">稍后</UiButton>
    </template>
  </TaskProgressCard>

  <UiDialog :open="confirming" title="重启 Codex" :busy="busy" @close="confirming = false">
    <p>重启会立即结束当前正在进行的 Codex 对话，然后重新打开 Codex。</p>
    <p>如果你还有没保存的对话内容，请先选「取消」。</p>
    <p v-if="restartError" role="alert">{{ restartError }}</p>
    <template #actions="{ close }">
      <UiButton :disabled="busy" @click="close">取消</UiButton>
      <UiButton variant="primary" :loading="busy" @click="restart">{{ restartError ? '重试重启' : '确认重启' }}</UiButton>
    </template>
  </UiDialog>
</template>
