<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import TaskProgressCard from '@/components/patterns/TaskProgressCard.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useFeedbackStore } from '@/app/feedback/store'
import { errorText, modelApi } from './api'

// 由全局 TaskProgressHost 承载，关闭后不留下任务或待重启状态。
const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const feedback = useFeedbackStore()
const confirming = ref(false)
const busy = ref(false)
const restartButton = ref<InstanceType<typeof UiButton> | null>(null)
let disposed = false
let origin: HTMLElement | null = null

watch(() => props.open, async open => {
  if (!open) { confirming.value = false; return }
  origin = document.activeElement instanceof HTMLElement ? document.activeElement : null
  await nextTick()
  if (!disposed && props.open) restartButton.value?.$el?.focus()
}, { immediate: true })

function later() { if (!busy.value) emit('close') }
function onEscape(event: KeyboardEvent) {
  // UiDialog 自己处理原生 cancel；其它弹窗的 Esc 不消费保存反馈。
  if (event.key !== 'Escape' || !props.open || document.querySelector('dialog[open], [aria-modal="true"]')) return
  event.preventDefault()
  later()
}
onMounted(() => window.addEventListener('keydown', onEscape))
onBeforeUnmount(() => {
  disposed = true
  window.removeEventListener('keydown', onEscape)
  if (origin?.isConnected) origin.focus()
})

async function restart() {
  if (busy.value) return
  busy.value = true
  try {
    const result = await modelApi.refreshCache(true)
    feedback.showToast(result.summary, { kind: result.success ? 'info' : 'error' })
  } catch (cause) {
    feedback.showToast(errorText(cause), { kind: 'error' })
  } finally {
    busy.value = false
    confirming.value = false
    // 新一次保存可能已替换本提示；旧请求不能关闭新反馈。
    if (!disposed) emit('close')
  }
}
</script>

<template>
  <TaskProgressCard
    v-if="open"
    title="已写入"
    message=""
    phase="done"
    :show-track="false"
    :show-collapse="false"
  >
    <template #actions>
      <UiButton ref="restartButton" class="notif-ctl" :disabled="busy" @click="confirming = true">立即重启 Codex</UiButton>
      <UiButton class="notif-ctl" variant="ghost" :disabled="busy" @click="later">稍后</UiButton>
    </template>
  </TaskProgressCard>

  <UiDialog :open="confirming" title="重启 Codex" :busy="busy" @close="later">
    <p>重启会立即结束当前正在进行的 Codex 对话，然后重新打开 Codex。</p>
    <p>如果你还有没保存的对话内容，请先选「取消」。</p>
    <template #actions>
      <UiButton :disabled="busy" @click="confirming = false">取消</UiButton>
      <UiButton variant="primary" :loading="busy" @click="restart">确认重启</UiButton>
    </template>
  </UiDialog>
</template>
