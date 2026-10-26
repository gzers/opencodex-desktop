<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { errorText, modelApi } from './api'

/**
 * 保存写入成功之后的一条通知（0.2.12）。
 *
 * 这里刻意**不保留任何状态**：
 * - 点「立即重启」→ 二次确认 → `sync-cache --restart-codex` → Codex 重启，选择器重读；
 * - 点「稍后」/ Esc → 这条提示直接消失，界面不再提「待重启」。
 *
 * 为什么不做「待重启」状态：状态会一直挂着，用户重启一百次也不会自动消失，
 * 而这正是 0.2.11 的死锁观感。用户口径是「写文件 + 重启就肯定生效」，所以只提示一次。
 */
const props = defineProps<{ open: boolean; planId: string | null }>()
const emit = defineEmits<{ close: [] }>()

const confirming = ref(false)
const busy = ref(false)
const error = ref('')
const restartButton = ref<InstanceType<typeof UiButton> | null>(null)

watch(() => props.open, async open => {
  if (!open) { confirming.value = false; busy.value = false; error.value = ''; return }
  await nextTick()
  // 默认焦点在「立即重启」（用户裁决 2026-10-26）：保存完就该重启，少一次寻找。
  restartButton.value?.$el?.focus?.()
})

// 默认聚焦不等于跳过确认：真重启前仍要一次确认，并写明会结束当前对话。
function askRestart() { if (!busy.value) { error.value = ''; confirming.value = true } }
function later() { if (!busy.value) emit('close') }

async function restart() {
  if (busy.value) return
  busy.value = true; error.value = ''
  try {
    const result = await modelApi.refreshCache(true)
    // 成功或失败都关闭提示：失败原因写在提示里，用户可手动重启，不留状态。
    if (!result.success) error.value = result.summary
    else emit('close')
  } catch (cause) {
    error.value = errorText(cause)
  } finally {
    busy.value = false
    confirming.value = false
  }
}
</script>

<template>
  <section
    v-if="open"
    class="save-result"
    role="status"
    aria-live="polite"
    @keydown.esc.stop="later"
  >
    <header><h3>已写入</h3></header>
    <p class="save-result-detail">配置已写入。重启 Codex 后，模型选择器才会读到这份配置。</p>
    <p v-if="error" class="save-result-error" role="alert">{{ error }}</p>
    <div class="save-result-actions">
      <UiButton ref="restartButton" variant="primary" :disabled="busy" @click="askRestart">立即重启 Codex</UiButton>
      <UiButton variant="ghost" :disabled="busy" @click="later">稍后</UiButton>
    </div>
  </section>

  <UiDialog :open="confirming" title="重启 Codex" :busy="busy" @close="confirming = false">
    <p>重启会立即结束当前正在进行的 Codex 对话，然后重新打开 Codex。</p>
    <p>如果你还有没保存的对话内容，请先选「取消」。</p>
    <p v-if="error" class="save-result-error" role="alert">{{ error }}</p>
    <template #actions>
      <UiButton :disabled="busy" @click="confirming = false">取消</UiButton>
      <UiButton variant="primary" :loading="busy" @click="restart">确认重启</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.save-result{border:1px solid var(--border);border-radius:var(--radius);background:var(--panel);box-shadow:var(--shadow);padding:var(--space-4);max-width:400px}
.save-result header h3{margin:0 0 var(--space-2);font-size:var(--text-subtitle)}
.save-result-detail{margin:0 0 var(--space-3);color:var(--muted);font-size:var(--text-label)}
.save-result-error{margin:0 0 var(--space-3);color:var(--danger);font-size:var(--text-label)}
.save-result-actions{display:flex;gap:var(--space-2);justify-content:flex-end}
</style>

