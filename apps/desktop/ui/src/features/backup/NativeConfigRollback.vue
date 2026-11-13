<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useModelsV2Store } from '@/features/models/store-v2'
import { useFeedbackStore } from '@/app/feedback/store'

interface Backup { id: string; createdAt: string; bytes: number; purpose?: string; runtimeVersion?: string | null }
interface Selection { id: string; preview: { currentPort: number | null; backupPort: number; targetPath: string } }
const models = useModelsV2Store()
const feedback = useFeedbackStore()
const open = ref(false)
const backupOpen = ref(false)
const backupError = ref('')
const busy = ref(false)
const backups = ref<Backup[]>([])
const backupId = ref('')
const selected = ref<(Selection & { context: string }) | null>(null)
const error = ref('')
let active = true
onBeforeUnmount(() => { active = false })
watch(backupId, () => { selected.value = null; error.value = '' })

const errors: Record<string, string> = {
  native_rollback_backup_invalid: '无法核验此备份，请核对备份记录和文件。',
  native_import_config_invalid: '备份未通过当前 OpenCodex 的配置校验，无法回退。',
  native_import_port_required: '无法核对本机端口设置，请先到安装配置核对。',
  native_import_draft_active: '当前有未保存的渠道更改，请先保存或还原。',
  native_import_runtime_unavailable: '当前 OpenCodex 运行程序未准备完成，暂时无法校验备份。',
  native_import_runtime_not_stopped: '请先停止 OpenCodex 并确认服务已退出，然后重新核对回退。',
  native_import_backup_failed: '未能完整备份当前配置，原配置已保留。',
  native_import_stale: '备份或本机配置已变化，请重新核对。',
  model_transaction_cas: '本机配置已变化，请重新核对。',
  model_transaction_pending: '上一次配置操作仍需核对，请处理后再回退。',
  native_config_missing: '当前 HOME 尚无配置文件，未生成备份。',
  native_config_backup_failed: '配置备份未能完整核验，原文件已保留。',
  native_config_backup_stale: '配置在备份期间发生变化，请重新备份。',
}
function failure(cause: unknown) {
  const code = typeof cause === 'object' && cause !== null && 'code' in cause ? String(cause.code) : ''
  return errors[code] ?? '未能确认回退结果，请核对当前配置和备份。'
}
function hasEdits() {
  return models.fileBusy || models.fileNeedsReconcile || models.fileDirty || !!models.pendingRows.length
}
function backupLabel(backup: Backup) {
  const date = new Date(backup.createdAt).toLocaleString()
  return date + ' · ' + (backup.runtimeVersion ?? '版本未确认') + ' · ' + backup.bytes.toLocaleString() + ' 字节 · ' + backup.id.slice(-8)
}
async function createBackup() {
  if (busy.value) return
  busy.value = true; backupError.value = ''
  try {
    const result = await invoke<Backup>('model_v2_native_config_backup')
    backupOpen.value = false
    feedback.showToast('完整配置已备份并核验，保存 ' + result.bytes.toLocaleString() + ' 字节。')
  } catch (cause) { backupError.value = failure(cause) }
  finally { busy.value = false }
}
async function show() {
  if (busy.value) return
  selected.value = null; backupId.value = ''; error.value = ''; backups.value = []
  open.value = true; busy.value = true
  try { backups.value = await invoke<Backup[]>('model_v2_native_rollback_list') }
  catch (cause) { error.value = failure(cause) }
  finally { busy.value = false }
}
async function review() {
  if (!backupId.value || busy.value) return
  if (hasEdits()) { error.value = errors.native_import_draft_active!; return }
  busy.value = true; selected.value = null; error.value = ''
  const context = models.fileContext()
  try {
    const choice = await invoke<Selection>('model_v2_native_rollback_select', { backupId: backupId.value })
    if (!active) return
    if (context !== models.fileContext() || hasEdits()) { error.value = errors.native_import_stale!; return }
    selected.value = { ...choice, context }
  } catch (cause) { error.value = failure(cause) }
  finally { busy.value = false }
}
async function rollback() {
  const choice = selected.value
  if (!choice || busy.value) return
  if (choice.context !== models.fileContext() || hasEdits()) {
    selected.value = null; error.value = errors.native_import_stale!; return
  }
  busy.value = true; models.fileBusy = true; error.value = ''
  let changed = false
  try {
    const result = await invoke<{ changed: boolean; backupId: string | null }>('model_v2_native_rollback_apply', { id: choice.id })
    changed = result.changed
    open.value = false
    feedback.showToast(changed
      ? (result.backupId ? '已回退配置，回退前的文件已备份。' : '原配置文件不存在，已从备份恢复。') + '下次启动 OpenCodex 时生效。'
      : '当前配置与所选备份一致。')
  } catch (cause) { error.value = failure(cause) }
  finally { selected.value = null; models.fileBusy = false; busy.value = false }
  if (changed && models.file) await models.reloadSavedFacts().catch(() => undefined)
}
</script>

<template>
  <article class="card">
    <div class="card-head">
      <div><h2>OpenCodex 配置备份与恢复</h2><p>从完整备份恢复配置，包括备份中的渠道、密钥和运行端口。</p></div>
      <div class="native-backup-actions">
        <UiButton :disabled="busy" @click="backupError = ''; backupOpen = true">备份完整配置</UiButton>
        <UiButton :disabled="busy" @click="show">查看备份与回退</UiButton>
      </div>
    </div>
  </article>
  <UiDialog :open="backupOpen" title="备份完整 OpenCodex 配置" :busy="busy" @close="backupOpen = false">
    <p>备份当前 HOME 中已保存的全部配置，包括渠道、模型、配置内密钥和运行端口。草稿、外部账户文件及系统凭据不在备份范围内。</p>
    <p>备份保存在当前用户的数据目录，生成后核对完整性与文件权限。</p>
    <p v-if="backupError" class="model-error" role="alert">{{ backupError }}</p>
    <template #actions="{ close }">
      <UiButton :disabled="busy" @click="close">取消</UiButton>
      <UiButton variant="primary" :disabled="busy" :loading="busy" @click="createBackup">生成完整备份</UiButton>
    </template>
  </UiDialog>
  <UiDialog :open="open" title="整文件回退" :busy="busy" @close="open = false">
    <p v-if="!busy && !backups.length && !error">当前用户尚无可用于整文件回退的原生配置备份。</p>
    <label v-if="backups.length" class="native-rollback-picker">
      配置备份
      <select v-model="backupId" class="input" :disabled="busy">
        <option value="" disabled>选择备份</option>
        <option v-for="backup in backups" :key="backup.id" :value="backup.id">{{ backupLabel(backup) }}</option>
      </select>
    </label>
    <template v-if="selected">
      <p>写入目标：<code>{{ selected.preview.targetPath }}</code></p>
      <p>运行端口：{{ selected.preview.currentPort ?? '未指定或无法读取' }} → {{ selected.preview.backupPort || '未指定' }}</p>
      <p>确认后先备份当前配置，再恢复整份文件。端口模式记录、外部账户文件和系统凭据不在回退范围内。</p>
    </template>
    <p v-if="error" class="model-error" role="alert">{{ error }}</p>
    <p v-if="busy" role="status">正在核对配置与备份…</p>
    <template #actions="{ close }">
      <UiButton :disabled="busy" @click="close">取消</UiButton>
      <UiButton v-if="selected" variant="danger" :disabled="busy" @click="rollback">备份当前配置并回退</UiButton>
      <UiButton v-else-if="backups.length" variant="primary" :disabled="busy || !backupId" @click="review">查看整文件回退预览</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.native-rollback-picker { display: grid; gap: var(--space-2); }
.native-backup-actions { display: flex; gap: var(--space-2); flex-wrap: wrap; }
code { overflow-wrap: anywhere; }
</style>
