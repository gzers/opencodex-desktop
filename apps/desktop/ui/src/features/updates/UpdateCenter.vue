<script lang="ts">
import { reactive } from 'vue'
// 界面请求按应用实例保存，路由重挂载不会丢失在途操作；进度事实来自既有 store。
const sessions = new WeakMap<object, ReturnType<typeof createSession>>()
function createSession() {
  return reactive({ checking: false, managerInstalling: false, officialInstalling: false, officialBackingUp: false, officialSucceeded: false,
    managerError: '', managerFailedOperation: '', officialError: '', managerCheckError: '',
    officialCandidate: '', managerCandidate: '', backup: true })
}
function sessionFor(app: object) {
  let session = sessions.get(app)
  if (!session) { session = createSession(); sessions.set(app, session) }
  return session
}
</script>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import { useAppStore } from '@/stores/app'
import { useRuntimeStore } from '@/features/runtime/store'
import { useUpdatesStore } from './store'
import { checkForUpdate, getUpdateStatus, installUpdate, managerInstallProgress, managerUpdateStatus, restartAfterUpdate } from './update'
import { hasNewerVersion } from './version'
import { installPhaseLabel } from '@/features/runtime/presentation'

type Target = 'manager' | 'official'
const props = withDefaults(defineProps<{ open: boolean; target?: Target | null }>(), { target: null })
const emit = defineEmits<{ 'update:open': [value: boolean] }>()
const app = useAppStore()
const runtime = useRuntimeStore()
const session = sessionFor(app)
const selected = ref<Target | null>(props.target)
const dialog = ref<HTMLElement | null>(null)
const titleId = 'update-center-' + useId()
let restoreFocus: HTMLElement | null = null
const status = managerUpdateStatus
const progress = managerInstallProgress
const pendingRestart = computed(() => status.value?.pendingRestart || progress.value?.stage === 'pending_restart')
const managerBusy = computed(() => session.managerInstalling || app.appUpdateBusy || !!(progress.value
  && !['failed', 'pending_restart'].includes(progress.value.stage)
  && !(session.managerError && session.managerFailedOperation === progress.value.operationId)))
const officialBusy = computed(() => session.officialInstalling || app.officialUpdateBusy)
const busy = computed(() => managerBusy.value || officialBusy.value || session.checking)
const managerAvailable = computed(() => !session.managerCheckError && !status.value?.error && hasNewerVersion(status.value?.currentVersion ?? null, status.value?.availableVersion ?? null) === true)
const officialAvailable = computed(() => !app.officialRemoteError && !app.officialProjectError && hasNewerVersion(app.officialProject?.version ?? null, app.officialRemote?.version ?? null) === true)
const managerResult = computed(() => {
  if (managerBusy.value) return '更新进行中'
  if (pendingRestart.value) return '更新已准备就绪'
  if (session.managerError || progress.value?.stage === 'failed') return '安装失败 · 可重试'
  if (session.managerCheckError || status.value?.error) return '检查失败'
  if (session.checking) return '正在检查'
  if (managerAvailable.value) return '发现新版本'
  if (!status.value?.lastCheckedAt) return '尚未检查'
  return status.value.availableVersion && hasNewerVersion(status.value.currentVersion, status.value.availableVersion) === null ? '无法比较版本' : '已是最新版本'
})
const officialResult = computed(() => {
  if (officialBusy.value) return '更新进行中'
  if (session.officialError || app.officialUpdateError) return '安装失败 · 可重试'
  if (app.officialRemoteLoading || app.officialProjectLoading) return '正在检查'
  if (app.officialRemoteError || app.officialProjectError) return '检查失败'
  if (officialAvailable.value) return '发现新版本'
  if (!app.officialRemote) return '尚未检查'
  return hasNewerVersion(app.officialProject?.version ?? null, app.officialRemote.version) === false ? '已是最新版本' : '无法比较版本'
})
const managerStage = computed(() => {
  if (pendingRestart.value) return '更新已准备就绪；请确认重启并应用。'
  const labels = { backup: '正在备份', checking: '正在检查更新包', downloading: '正在下载', installing: '正在安装', pending_restart: '更新已准备就绪', failed: '安装失败' }
  return progress.value ? labels[progress.value.stage] : '正在提交更新请求，等待更新器进度…'
})
const downloadPercent = computed(() => {
  const p = progress.value
  if (p?.stage !== 'downloading' || p.totalBytes === null || !Number.isFinite(p.totalBytes) || p.totalBytes <= 0 || !Number.isFinite(p.downloadedBytes) || p.downloadedBytes < 0) return undefined
  return Math.min(100, Math.max(0, p.downloadedBytes / p.totalBytes * 100))
})
function bytes(value: number) {
  if (value < 1024) return value + ' B'
  if (value < 1024 * 1024) return (value / 1024).toFixed(1) + ' KB'
  return (value / (1024 * 1024)).toFixed(1) + ' MB'
}
function safeHttps(value: string | null | undefined): string | null {
  if (!value || !/^https:\/\//i.test(value) || /[\s\\]/.test(value)) return null
  try {
    const url = new URL(value)
    return url.protocol === 'https:' && url.hostname && !url.username && !url.password ? url.href : null
  } catch { return null }
}
const releaseUrl = computed(() => safeHttps(status.value?.releaseUrl))
async function openRelease(url: string | null) {
  const safe = safeHttps(url)
  if (!safe) return
  if (!await app.openExternalLink(safe)) app.showToast('网页详情打开失败，请稍后重试。')
}
async function check() {
  if (busy.value) return
  session.checking = true
  session.managerCheckError = ''
  try {
    await Promise.allSettled([
      checkForUpdate().catch(() => { session.managerCheckError = '更新服务不可用，请检查网络后重试。' }),
      app.loadOfficialProject(), app.loadOfficialRemoteLatest(),
    ])
  } finally { session.checking = false }
}
async function installManager() {
  const snapshot = status.value
  if (busy.value || pendingRestart.value || !managerAvailable.value || !snapshot?.availableVersion) return
  const candidateVersion = snapshot.availableVersion
  const channel = snapshot.channel
  const backup = session.backup
  session.managerInstalling = true
  session.managerCandidate = candidateVersion
  session.managerError = ''
  try { await installUpdate({ candidateVersion, channel, backup }) }
  catch (error) {
    session.managerFailedOperation = progress.value?.operationId ?? ''
    session.managerError = error instanceof Error ? error.message : '更新安装失败，请重新检查后重试。'
  }
  finally { session.managerInstalling = false }
}
async function installOfficial() {
  const candidateVersion = app.officialRemote?.version
  if (busy.value || app.runtimeInstalling || app.upgradeBackupBusy || !officialAvailable.value || !candidateVersion) return
  session.officialInstalling = true
  session.officialBackingUp = true
  session.officialSucceeded = false
  session.officialCandidate = candidateVersion
  session.officialError = ''
  try {
    // 每次面板更新前备份，失败时阻断安装，不把旧备份冒充本次备份。
    const backup = await useUpdatesStore().createUpgradeBackup()
    if (!backup) { session.officialError = app.upgradeBackupError || '升级前备份失败，尚未安装更新。'; return }
    session.officialBackingUp = false
    const ok = await app.applyOfficialUpdate(candidateVersion)
    session.officialSucceeded = ok
    if (!ok) session.officialError = app.officialUpdateError || '官方更新安装失败，请重试。'
  } catch (error) { session.officialError = error instanceof Error ? error.message : '官方更新安装失败，请重试。' }
  finally { session.officialInstalling = false; session.officialBackingUp = false }
}
const restarting = ref(false)
async function restart() {
  if (!pendingRestart.value || restarting.value) return
  restarting.value = true
  try { await restartAfterUpdate() }
  catch { session.managerError = '重启失败，更新仍待应用；请重试。' }
  finally { restarting.value = false }
}
function close() { emit('update:open', false) }
function focusables() {
  return Array.from(dialog.value?.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), [href], [tabindex="0"]') ?? [])
}
function onKeydown(event: KeyboardEvent) {
  if (!props.open) return
  if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); close() }
  if (event.key !== 'Tab') return
  const list = focusables()
  const first = list[0], last = list[list.length - 1]
  const inside = dialog.value?.contains(document.activeElement) === true
  if (!first || document.activeElement === dialog.value || (event.shiftKey ? !inside || document.activeElement === first : !inside || document.activeElement === last)) {
    event.preventDefault()
    ;(event.shiftKey ? last : first)?.focus()
  }
}
watch(() => props.target, target => { selected.value = target })
watch(() => props.open, async open => {
  if (open) {
    selected.value = props.target
    restoreFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
    window.addEventListener('keydown', onKeydown, true)
    // 打开详情只读缓存，反复进入路由不会隐式联网检查或备份。
    void getUpdateStatus().catch(() => {})
    await nextTick()
    dialog.value?.focus()
  } else {
    window.removeEventListener('keydown', onKeydown, true)
    restoreFocus?.focus()
    restoreFocus = null
  }
}, { immediate: true })
onBeforeUnmount(() => { window.removeEventListener('keydown', onKeydown, true); restoreFocus?.focus() })
defineExpose({ check })
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-mask update-center-mask" data-testid="update-center" @click.self="close">
      <div ref="dialog" class="modal modal-wide update-center" role="dialog" aria-modal="true" :aria-labelledby="titleId" tabindex="-1">
        <header class="update-center-head">
          <div><span class="update-kicker">软件更新</span><h3 :id="titleId">{{ selected === 'manager' ? '桌面管理器更新' : selected === 'official' ? 'OpenCodex 面板更新' : '更新中心' }}</h3><p>检查两个更新对象，查看说明后选择安装。</p></div>
          <UiButton variant="ghost" aria-label="关闭更新中心" @click="close">关闭</UiButton>
        </header>
        <div class="update-center-body">
          <div v-if="selected" class="update-back"><UiButton variant="ghost" @click="selected = null">← 全部更新</UiButton></div>
          <UiCard v-if="!selected || selected === 'manager'" class="update-object" data-testid="manager-update-card">
            <button class="update-object-head" type="button" aria-label="查看桌面管理器更新详情" :aria-expanded="selected === 'manager'" @click="selected = 'manager'">
              <span class="update-object-icon" aria-hidden="true">↗</span><span><strong>桌面管理器</strong><small>OpenCodex Desktop · {{ status?.channel === 'beta' ? '测试' : '稳定' }}通道</small></span><span class="tag" :class="{ danger: managerResult.includes('失败'), ok: pendingRestart }">{{ managerResult }}</span>
            </button>
            <div class="update-versions"><span>当前 <b>{{ status?.currentVersion ?? app.aboutApp?.version ?? '未知' }}</b></span><span aria-hidden="true">→</span><span>可用 <b>{{ status?.availableVersion ?? '—' }}</b></span></div>
            <p v-if="session.managerCheckError || status?.error" class="update-error" role="status">{{ session.managerCheckError || status?.error }}</p>
            <div v-if="selected === 'manager'" class="update-detail">
              <div class="update-detail-head"><h4>更新说明</h4><time v-if="status?.publishedAt">{{ status.publishedAt }}</time></div>
              <pre class="update-notes" tabindex="0" aria-label="桌面管理器更新说明">{{ status?.notes || '发行方尚未提供更新说明。可打开网页详情查看发行信息。' }}</pre>
              <UiButton v-if="releaseUrl" variant="ghost" @click="openRelease(releaseUrl)">查看网页详情 ↗</UiButton>
              <p class="update-hint">更新包通过签名校验后准备应用；准备完成后可选择重启时机。</p>
            </div>
            <div v-if="managerBusy || pendingRestart || progress?.stage === 'failed'" class="update-progress" aria-live="polite">
              <strong>{{ managerStage }}</strong><small v-if="progress?.candidateVersion || session.managerCandidate">目标版本 {{ progress?.candidateVersion || session.managerCandidate }}</small>
              <template v-if="managerBusy">
                <progress aria-label="管理器更新进度" :value="downloadPercent" max="100"></progress>
                <small v-if="progress?.stage === 'downloading'">{{ bytes(progress.downloadedBytes) }}<template v-if="progress.totalBytes !== null"> / {{ bytes(progress.totalBytes) }}</template><template v-if="downloadPercent !== undefined"> · {{ Math.floor(downloadPercent) }}%</template></small>
              </template>
              <p v-if="pendingRestart" class="update-hint">可先隐藏此窗口，稍后回来重启。</p>
            </div>
            <p v-if="session.managerError" class="update-error" role="alert">{{ session.managerError }}</p>
            <div class="update-object-actions">
              <label v-if="!pendingRestart" class="update-backup"><input v-model="session.backup" type="checkbox" :disabled="managerBusy">更新前备份配置（推荐）</label>
              <UiButton v-if="pendingRestart" variant="primary" :loading="restarting" @click="restart">重启并应用更新</UiButton>
              <UiButton v-else variant="primary" :disabled="busy || !managerAvailable" :loading="managerBusy" data-testid="install-manager-update" @click="installManager">{{ managerBusy ? '更新中' : session.managerError || progress?.stage === 'failed' ? '重试安装' : '安装管理器更新' }}</UiButton>
            </div>
          </UiCard>
          <UiCard v-if="!selected || selected === 'official'" class="update-object" data-testid="official-update-card">
            <button class="update-object-head" type="button" aria-label="查看 OpenCodex 面板更新详情" :aria-expanded="selected === 'official'" @click="selected = 'official'">
              <span class="update-object-icon" aria-hidden="true">⌘</span><span><strong>OpenCodex 面板</strong><small>官方项目 · @bitkyc08/opencodex</small></span><span class="tag" :class="{ danger: officialResult.includes('失败') }">{{ officialResult }}</span>
            </button>
            <div class="update-versions"><span>当前 <b>{{ app.officialProject?.version ?? '未知' }}</b></span><span aria-hidden="true">→</span><span>可用 <b>{{ app.officialRemote?.version ?? '—' }}</b></span></div>
            <p v-if="app.officialRemoteError || app.officialProjectError" class="update-error" role="status">版本查询失败，请检查网络或运行来源后重试。</p>
            <div v-if="selected === 'official'" class="update-detail">
              <h4>更新说明</h4><pre class="update-notes" tabindex="0" aria-label="OpenCodex 面板更新说明">当前版本查询未提供发布说明。请查看官方网页详情，确认版本变化后再安装。</pre>
              <UiButton variant="ghost" @click="openRelease('https://www.npmjs.com/package/@bitkyc08/opencodex?activeTab=versions')">查看官方网页详情 ↗</UiButton>
            </div>
            <p class="update-hint">面板升级前必须备份配置；备份成功后才会安装选定版本。更新完成后如需重启运行中的代理，将另行提示。</p>
            <div v-if="officialBusy || session.officialSucceeded && runtime.installOutcome?.version === session.officialCandidate" class="update-progress" aria-live="polite">
              <strong>{{ session.officialBackingUp || app.upgradeBackupBusy ? '正在备份配置' : officialBusy ? installPhaseLabel(runtime.install?.phase ?? 'preparing') : '面板更新已完成' }}</strong>
              <small>目标版本 {{ session.officialCandidate || app.officialRemote?.version }}</small>
              <progress v-if="officialBusy" aria-label="面板更新进度"></progress>
              <pre v-if="!session.officialBackingUp && runtime.installLines.length && session.officialCandidate" class="update-console" tabindex="0" aria-label="面板安装输出">{{ runtime.installLines.join('\n') }}</pre>
              <p v-if="!officialBusy && runtime.installOutcome?.needsRestart" class="update-hint">面板更新已安装；请在运行操作中重启代理后生效。</p>
              <p v-if="!officialBusy && runtime.installOutcome?.protectionReconciliationPending" class="update-hint" role="status">安装已完成；保护备份完整性核对未完成，备份仍保留。请查看诊断日志。</p>
            </div>
            <p v-if="session.officialError || app.officialUpdateError" class="update-error" role="alert">{{ session.officialError || app.officialUpdateError }}</p>
            <div class="update-object-actions"><small v-if="app.upgradeLastBackup">最近备份 {{ app.upgradeLastBackup.backupId }}</small><UiButton variant="primary" :disabled="busy || app.runtimeInstalling || app.upgradeBackupBusy || !officialAvailable" :loading="officialBusy" data-testid="install-official-update" @click="installOfficial">{{ officialBusy ? '更新中' : session.officialError || app.officialUpdateError ? '重试面板更新' : '备份并更新面板' }}</UiButton></div>
          </UiCard>
        </div>
        <footer class="update-center-foot"><p>{{ managerBusy || officialBusy ? '隐藏窗口后更新继续，可重新打开查看进度。' : '更新准备完成后由你选择重启时机。' }}</p><div><UiButton variant="ghost" @click="close">{{ managerBusy || officialBusy ? '隐藏' : '稍后' }}</UiButton><UiButton :disabled="busy" :loading="session.checking" @click="check">{{ session.checking ? '检查中' : '重新检查更新' }}</UiButton></div></footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.update-center.modal { width: min(820px, 100%); max-height: calc(100% - 24px); padding: 0; display: flex; flex-direction: column; overflow: hidden; }
.update-center-head, .update-center-foot { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); padding: var(--space-5) var(--space-6); }
.update-center-head { border-bottom: 1px solid var(--border); }
.update-center-head h3 { margin: var(--space-1) 0; }
.update-center-head p, .update-center-foot p { margin: 0; font-size: var(--text-label); color: var(--muted); }
.update-kicker { color: var(--muted); font-size: var(--text-caption); }
.update-center-body { overflow: auto; min-height: 0; padding: var(--space-5) var(--space-6); display: grid; gap: var(--space-4); }
.update-object { min-width: 0; padding: var(--space-5); }
.update-object-head { display: flex; align-items: center; gap: var(--space-3); width: 100%; padding: 0; color: var(--text); background: transparent; border: 0; text-align: left; cursor: pointer; font: inherit; }
.update-object-head > span:nth-child(2) { flex: 1; min-width: 0; }
.update-object-head strong { display: block; font-size: var(--text-subtitle); }
.update-object-head small { display: block; margin-top: var(--space-1); color: var(--muted); overflow-wrap: anywhere; }
.update-object-icon { display: grid; place-items: center; width: 40px; height: 40px; flex: none; border: 1px solid var(--border); border-radius: var(--radius); background: var(--raised); font-size: var(--text-title); }
.update-versions { display: flex; flex-wrap: wrap; gap: var(--space-3); padding: var(--space-4) 0; font-size: var(--text-label); color: var(--muted); }
.update-versions b { color: var(--text); font-family: var(--font-code); overflow-wrap: anywhere; }
.update-detail { border-top: 1px solid var(--border-soft); margin-bottom: var(--space-4); padding-top: var(--space-4); }
.update-detail-head { display: flex; flex-wrap: wrap; justify-content: space-between; gap: var(--space-2); }
.update-detail h4 { margin: 0 0 var(--space-3); font-size: var(--text-control); }
.update-detail time { color: var(--muted); font-size: var(--text-caption); }
.update-notes, .update-console { max-height: 220px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; padding: var(--space-4); background: var(--raised); border: 1px solid var(--border-soft); border-radius: var(--radius-sm); font-size: var(--text-control); line-height: 1.7; }
.update-notes { font-family: inherit; margin: 0 0 var(--space-3); }
.update-console { max-height: 150px; margin: 0; }
.update-hint { color: var(--muted); font-size: var(--text-label); line-height: 1.7; margin: var(--space-3) 0; }
.update-error { color: var(--red); font-size: var(--text-control); overflow-wrap: anywhere; }
.update-object-actions { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: var(--space-3); margin-top: var(--space-4); }
.update-object-actions > .btn { margin-left: auto; }
.update-backup { display: flex; gap: var(--space-2); align-items: center; font-size: var(--text-label); }
.update-progress { display: grid; gap: var(--space-2); margin-top: var(--space-4); }
.update-progress progress { width: 100%; height: 6px; accent-color: var(--accent); }
.update-progress small, .update-object-actions small { color: var(--muted); overflow-wrap: anywhere; }
.update-center-foot { border-top: 1px solid var(--border); }
.update-center-foot > div { display: flex; gap: var(--space-2); flex: none; }
@media (max-width: 600px) {
  .update-center-head, .update-center-foot { padding: var(--space-4); flex-wrap: wrap; }
  .update-center-body { padding: var(--space-4); }
  .update-object { padding: var(--space-4); }
  .update-object-head { flex-wrap: wrap; }
  .update-center-foot > div { margin-left: auto; }
}
</style>
