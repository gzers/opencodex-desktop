<script setup lang="ts">
import { onMounted, ref } from 'vue'
import BackupFiles from './BackupFiles.vue'
import { createBackup, executeCleanup, getBackupPolicy, listBackups, openBackupFile, pinBackup, previewCleanup, restoreBackup, saveBackupPolicy, type BackupFileNode, type BackupPolicy, type PreferencesBackup } from './api'
import { useAppStore } from '@/stores/app'
import { getUpdateStatus } from '@/features/updates/update'
const app = useAppStore()
const files = ref<InstanceType<typeof BackupFiles> | null>(null)
const records = ref<PreferencesBackup[]>([])
const policy = ref<BackupPolicy | null>(null)
const mode = ref<'manual' | 'automatic'>('manual')
const busy = ref(false)
const loading = ref(false)
const error = ref('')
const feedback = ref('')
let loadSequence = 0
async function refresh() {
  const sequence = ++loadSequence
  loading.value = true
  error.value = ''
  try {
    const [items, savedPolicy] = await Promise.all([listBackups(), getBackupPolicy()])
    if (sequence !== loadSequence) return
    records.value = items
    policy.value = savedPolicy
    mode.value = savedPolicy.mode
  } catch {
    if (sequence === loadSequence) error.value = '无法读取备份或清理策略。请检查目录与清单后重试。'
  } finally { if (sequence === loadSequence) loading.value = false }
}
async function transact(action: () => Promise<string>) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  feedback.value = ''
  try {
    feedback.value = await action()
    await refresh()
    await files.value?.refresh()
  } catch { error.value = '操作未完成。请检查目录权限、备份完整性及在途更新，并刷新记录确认当前状态。' }
  finally { busy.value = false }
}
function generate() {
  void transact(async () => {
    const result = await createBackup()
    return result.cleanupError ? '备份已生成；自动清理未完成，可预览后重试。' : '管理器偏好备份已生成。'
  })
}
function applyPolicy() {
  if (!policy.value) return
  void transact(async () => {
    await saveBackupPolicy({ ...policy.value!, mode: mode.value })
    return '清理策略已保存；此次保存不删除备份。'
  })
}
function setPinned(record: PreferencesBackup) {
  void transact(async () => {
    await pinBackup(record.id, !record.pinned)
    return record.pinned ? '已取消固定保留。' : '已固定保留，不占最近份数。'
  })
}
async function cleanup() {
  if (busy.value) return
  busy.value = true
  error.value = ''
  try {
    const preview = await previewCleanup()
    if (!preview.candidateIds.length) { feedback.value = '没有可清理的备份。'; return }
    app.openModal({
      title: '确认清理备份',
      body: '将删除 ' + preview.candidateIds.length + ' 份备份（' + bytes(preview.candidateBytes) + '）。最近 ' + preview.keepRecent + ' 份或 ' + preview.keepDays + ' 天内的备份、固定项与受保护项会保留。执行时重新校验预览；有变化则中止。',
      confirmLabel: '清理这些备份',
      onConfirm: () => { void transact(async () => { const removed = await executeCleanup(preview); return '已清理 ' + removed.length + ' 份备份。' }) },
    })
  } catch { error.value = '无法生成清理预览。未删除任何备份。' }
  finally { busy.value = false }
}
function restore(record: PreferencesBackup) {
  app.openModal({
    title: '恢复管理器偏好',
    body: '将恢复此备份中的管理器偏好；恢复前先生成当前偏好的保护备份，校验失败会中止。此操作不恢复面板配置、会话或登录态。',
    confirmLabel: '校验并恢复',
    onConfirm: () => { void transact(async () => {
      const result = await restoreBackup(record.id)
      if (result?.refreshRequired) return '偏好已恢复；运行状态刷新未完成，请重启管理器。恢复前保护备份已保留。'
      try {
        await app.loadPreferences()
        await getUpdateStatus()
      } catch {
        return '偏好已恢复；界面刷新未完成，请重新加载。恢复前保护备份已保留。'
      }
      return '偏好已恢复；恢复前保护备份已保留。'
    }) },
  })
}
function openFile(node: BackupFileNode) {
  app.openModal({ title: node.directory ? '打开备份目录' : '打开备份文件',
    body: '备份文件可能包含配置与敏感信息。请勿直接修改或删除，以免影响恢复。将在系统应用中打开所选项目。',
    confirmLabel: '继续打开',
    onConfirm: () => { void openBackupFile(node.id).catch(() => app.showToast('打开失败，请检查文件是否存在及目录权限。')) },
  })
}
function bytes(value: number) { return value < 1024 ? value + ' B' : value < 1048576 ? (value / 1024).toFixed(1) + ' KB' : (value / 1048576).toFixed(1) + ' MB' }
function actionName(action: string) { return ({ 'manual-preferences': '手动', upgrade: '升级前', 'restore-protection': '恢复前保护', 'preferences-protection': '配置写入前保护' } as Record<string, string>)[action] || '事务备份' }
function date(value: string) { const d = new Date(value); return Number.isNaN(d.valueOf()) ? '时间未知' : d.toLocaleString() }
onMounted(() => { void refresh() })
</script>

<template>
  <section class="backup-management" aria-label="管理器偏好备份">
    <div class="backup-summary">
      <p>备份管理器偏好与清单；面板完整配置、会话及登录态由对应事务另行管理。</p>
      <button class="btn" :disabled="busy || loading" @click="generate">{{ busy ? '处理中…' : '生成备份' }}</button>
    </div>
    <BackupFiles ref="files" @open="openFile" />
    <div class="backup-policy">
      <div><h3>保留与清理</h3><p>保留最近 10 份或 30 天内的备份；满足任一条件即保留。固定项与受保护项额外保留。</p></div>
      <div class="controls">
        <label for="backup-cleanup-mode">清理方式</label>
        <select id="backup-cleanup-mode" v-model="mode" class="input" :disabled="busy || loading || !policy">
          <option value="manual">手动确认（默认）</option>
          <option value="automatic">新备份成功后自动清理</option>
        </select>
        <button class="btn ghost" :disabled="busy || loading || !policy || mode === policy.mode" @click="applyPolicy">保存策略</button>
        <button class="btn ghost" :disabled="busy || loading" @click="cleanup">预览清理</button>
      </div>
    </div>
    <div class="backup-list-head"><h3>备份记录</h3><button class="btn ghost" :disabled="busy || loading" @click="refresh">刷新记录</button></div>
    <p v-if="error" class="backup-error" role="alert">{{ error }}</p>
    <p v-if="feedback" role="status">{{ feedback }}</p>
    <p v-if="loading" role="status">正在读取备份清单…</p>
    <p v-else-if="!error && !records.length" class="backup-empty">尚无备份。生成一份后可在这里固定保留或恢复。</p>
    <ul v-if="records.length" class="backup-records">
      <li v-for="record in records" :key="record.id">
        <div class="backup-record-info"><strong>{{ actionName(record.action) }} · {{ date(record.createdAt) }}</strong><code>{{ record.id }}</code>
          <span>{{ bytes(record.bytes) }} · {{ record.integrityVerified ? '已校验' : '恢复前重新校验' }}{{ record.pinned ? ' · 已固定' : '' }}{{ record.protected ? ' · 受保护' : '' }}</span>
        </div>
        <div class="controls"><button class="btn ghost" :disabled="busy || loading || !record.preferencesCandidate" @click="setPinned(record)">{{ record.pinned ? '取消固定' : '固定保留' }}</button><button class="btn ghost" :disabled="busy || loading || !record.preferencesCandidate" @click="restore(record)">恢复</button></div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.backup-summary,.backup-list-head { display:flex; align-items:center; justify-content:space-between; gap:16px; }
.backup-summary p,.backup-policy p,.backup-empty { color:var(--muted); font-size:12px; line-height:1.6; }
.backup-policy { border-top:1px solid var(--line); margin-top:20px; padding-top:16px; }
h3 { margin:0; font-size:13px; }
.backup-policy .controls { flex-wrap:wrap; margin:12px 0 20px; }
.backup-policy select { width:auto; max-width:100%; }
.backup-records { list-style:none; margin:10px 0 0; padding:0; }
.backup-records li { display:flex; align-items:center; justify-content:space-between; gap:16px; padding:14px 0; border-top:1px solid var(--line); }
.backup-record-info { display:flex; flex-direction:column; gap:5px; min-width:0; font-size:12px; }
.backup-record-info code { font-size:11px; overflow-wrap:anywhere; }
.backup-record-info span { color:var(--muted); font-size:11px; }
.backup-error { color:var(--danger); }
@media(max-width:650px) { .backup-records li,.backup-summary { align-items:flex-start; flex-direction:column; } }
</style>
