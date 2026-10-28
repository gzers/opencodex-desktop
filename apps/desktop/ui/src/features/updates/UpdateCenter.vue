<script lang="ts">
import { reactive } from 'vue'
// 界面请求按应用实例保存，路由重挂载不会丢失在途操作；进度事实来自既有 store。
const sessions = new WeakMap<object, ReturnType<typeof createSession>>()
function createSession() {
  return reactive({ checking: false, managerInstalling: false, officialInstalling: false, officialBackingUp: false, officialSucceeded: false,
    managerError: '', managerFailedOperation: '', officialError: '', officialInstallRequested: false, officialFailureLines: [] as string[], managerCheckError: '',
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
import ReleaseNotes from './ReleaseNotes.vue'
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
const officialFailed = computed(() => session.officialInstallRequested && !!(session.officialError || app.officialUpdateError))
const officialOutput = computed(() => officialFailed.value ? session.officialFailureLines : runtime.installLines)
const busy = computed(() => managerBusy.value || officialBusy.value || session.checking)
const managerAvailable = computed(() => !session.managerCheckError && !status.value?.error && hasNewerVersion(status.value?.currentVersion ?? null, status.value?.availableVersion ?? null) === true)
const officialAvailable = computed(() => !app.officialRemoteError && !app.officialProjectError && hasNewerVersion(app.officialProject?.version ?? null, app.officialRemote?.version ?? null) === true)
const officialQueryError = computed(() => app.officialRemoteError
  ? app.officialRemoteErrorDetail || '版本查询失败，请检查网络或运行来源后重试。'
  : app.officialProjectError ? '本地版本读取失败，请检查运行来源后重试。' : '')
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
/** §39.1：结果页右列的一句说明摘要；取首段，CSS 限制两行。 */
function summaryOf(text: string | null | undefined) {
  const value = (text || '').trim()
  if (!value) return ''
  // 按空行分段：用 filter(Boolean) 代替正则，避免转义在不同写入方式下被破坏。
  const paragraphs = value.split(String.fromCharCode(10).repeat(2)).filter(part => part.trim())
  const first = (paragraphs[0] || value).replace(/\s+/g, ' ').trim()
  return first.length > 120 ? first.slice(0, 120) + '…' : first
}
/** 面板更新的说明来自远端版本详情；结果页只读已有事实，不额外请求。 */
const officialNotes = computed<string | null>(() => null)
const selectedTitle = computed(() => selected.value === 'manager' ? '桌面管理器更新' : 'OpenCodex 面板更新')
const selectedSource = computed(() => selected.value === 'manager' ? 'OpenCodex Desktop · ' + (status.value?.channel === 'beta' ? '测试' : '稳定') + '通道' : '官方项目 · @bitkyc08/opencodex')
const managerUnavailableText = computed(() => session.managerError || progress.value?.stage === 'failed' ? '上次安装未完成，可进入详情重试。' : status.value?.availableVersion && hasNewerVersion(status.value.currentVersion, status.value.availableVersion) === null ? '无法比较版本。' : pendingRestart.value ? '更新已准备就绪，请进入详情重启。' : status.value?.lastCheckedAt ? '当前没有待确认的更新。' : '尚未检查。')
const officialUnavailableText = computed(() => session.officialError || app.officialUpdateError ? '上次安装未完成，可进入详情重试。' : officialAvailable.value ? '' : '当前没有待确认的更新。')
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
  session.officialInstallRequested = false
  session.officialFailureLines = []
  session.officialCandidate = candidateVersion
  session.officialError = ''
  try {
    // 每次面板更新前备份，失败时阻断安装，不把旧备份冒充本次备份。
    const backup = await useUpdatesStore().createUpgradeBackup()
    if (!backup) { session.officialError = app.upgradeBackupError || '升级前备份失败，尚未安装更新。'; return }
    session.officialBackingUp = false
    session.officialInstallRequested = true
    const ok = await app.applyOfficialUpdate(candidateVersion)
    session.officialSucceeded = ok
    if (!ok) {
      session.officialFailureLines = runtime.installing ? [] : [...runtime.installLines]
      session.officialError = app.officialUpdateError || '官方更新安装失败，请重试。'
    }
  } catch (error) {
    if (session.officialInstallRequested && !runtime.installing) session.officialFailureLines = [...runtime.installLines]
    session.officialError = error instanceof Error ? error.message : '官方更新安装失败，请重试。'
  }
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
watch(selected, async () => {
  const label = dialog.value?.contains(document.activeElement)
    ? document.activeElement?.getAttribute('aria-label')
    : null
  await nextTick()
  if (!props.open || !dialog.value || dialog.value.contains(document.activeElement)) return
  // Returning to all updates removes its focused back button. Keep modal focus
  // inside, preserving a labelled control if the render replaced that control.
  const replacement = label ? focusables().find(element => element.getAttribute('aria-label') === label) : null
  ;(replacement ?? dialog.value).focus()
})
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
          <div><span class="update-kicker">软件更新</span><h3 :id="titleId">{{ selected ? selectedTitle : '检查更新结果' }}</h3><p v-if="!selected">分别查看更新内容与范围，确认后才会开始更新。</p><p v-else>{{ selectedSource }}</p></div>
          <UiButton variant="ghost" aria-label="关闭更新中心" @click="close">关闭</UiButton>
        </header>
        <div class="update-center-body">
          <div v-if="!selected" class="update-result-rows" data-testid="update-result-rows">
            <section class="update-result-row" data-testid="manager-update-row">
              <div class="update-result-main">
                <div class="update-result-title"><strong>桌面管理器</strong><span class="tag" :class="{ danger: managerResult.includes('失败'), ok: pendingRestart }">{{ managerResult }}</span></div>
                <div class="update-result-version"><span>{{ status?.currentVersion ?? app.aboutApp?.version ?? '未知' }}</span><template v-if="managerAvailable"><span aria-hidden="true">→</span><b>{{ status?.availableVersion }}</b></template></div>
                <p class="update-result-source">OpenCodex Desktop · {{ status?.channel === 'beta' ? '测试' : '稳定' }}通道</p>

                <!-- §39.1 右列：一句说明摘要，最多两行；结果页不提供展开。 -->
                <p v-if="managerAvailable && summaryOf(status?.notes)" class="update-result-summary">{{ summaryOf(status?.notes) }}</p>
                <p v-if="session.managerCheckError || status?.error" class="update-error" role="status">{{ session.managerCheckError || status?.error }}</p>
                <p v-else-if="!managerAvailable" class="update-result-empty">{{ managerUnavailableText }}</p>
              </div>
              <div class="update-result-actions"><UiButton data-testid="manager-update-detail" @click="selected = 'manager'">查看详情</UiButton></div>
            </section>
            <section class="update-result-row" data-testid="official-update-row">
              <div class="update-result-main">
                <div class="update-result-title"><strong>OpenCodex 面板</strong><span class="tag" :class="{ danger: officialResult.includes('失败') }">{{ officialResult }}</span></div>
                <div class="update-result-version"><span>{{ app.officialProject?.version ?? '未知' }}</span><template v-if="officialAvailable"><span aria-hidden="true">→</span><b>{{ app.officialRemote?.version }}</b></template></div>
                <p class="update-result-source">官方项目 · @bitkyc08/opencodex</p>

                <p v-if="officialAvailable" class="update-result-summary">{{ summaryOf(officialNotes) }}</p>
                <p v-if="officialQueryError" class="update-error" role="status">{{ officialQueryError.split('\n')[0] }}</p>
                <p v-else-if="!officialAvailable" class="update-result-empty">{{ officialUnavailableText }}</p>
              </div>
              <div class="update-result-actions"><UiButton data-testid="official-update-detail" @click="selected = 'official'">查看详情</UiButton></div>
            </section>
          </div>
          <template v-else>
            <div class="update-back"><UiButton variant="ghost" @click="selected = null">← 全部更新</UiButton></div>
            <section v-if="selected === 'manager'" class="update-object" data-testid="manager-update-card">
              <div class="update-detail-head"><h4>桌面管理器更新详情与确认</h4><time v-if="status?.lastCheckedAt">{{ status.lastCheckedAt }}</time></div>
              <ReleaseNotes target="manager" :channel="status?.channel ?? 'stable'" :version="status?.availableVersion ?? null" :updater-notes="status?.notes" />
              <div v-if="managerAvailable" class="update-confirm-scope">
                <label class="update-backup"><input v-model="session.backup" type="checkbox" :disabled="managerBusy">更新前备份配置（推荐）</label>
                <p class="update-hint">保存到当前数据根的备份目录：包含管理器偏好、清单与完整性摘要；不含面板完整配置、会话、登录态或安装包。</p>
              </div>
              <div v-if="managerBusy || pendingRestart || progress?.stage === 'failed'" class="update-progress" aria-live="polite">
                <strong>{{ managerStage }}</strong>
                <small v-if="progress?.candidateVersion || session.managerCandidate">目标版本 {{ progress?.candidateVersion || session.managerCandidate }}</small>
                <template v-if="managerBusy">
                  <progress aria-label="管理器更新进度" :value="downloadPercent" max="100"></progress>
                  <small v-if="progress?.stage === 'downloading'">{{ bytes(progress.downloadedBytes) }}<template v-if="progress.totalBytes !== null"> / {{ bytes(progress.totalBytes) }}</template><template v-if="downloadPercent !== undefined"> · {{ Math.floor(downloadPercent) }}%</template></small>
                </template>
                <p v-if="pendingRestart" class="update-hint">可先隐藏此窗口，稍后回来重启。</p>
              </div>
              <p v-if="session.managerError" class="update-error" role="alert">{{ session.managerError }}</p>
              <div v-if="pendingRestart || managerAvailable" class="update-object-actions">
                <UiButton v-if="pendingRestart" variant="primary" :loading="restarting" @click="restart">重启并应用更新</UiButton>
                <UiButton v-else variant="primary" :disabled="busy || !managerAvailable" :loading="managerBusy || session.checking" data-testid="install-manager-update" @click="installManager">{{ managerBusy ? '更新中' : session.managerError || progress?.stage === 'failed' ? '重试安装' : '确认更新' }}</UiButton>
              </div>
            </section>
            <section v-else class="update-object" data-testid="official-update-card">
              <div class="update-detail-head"><h4>OpenCodex 面板更新详情与确认</h4></div>
              <p v-if="officialQueryError" class="update-error update-query-error" role="status" data-testid="official-query-error">{{ officialQueryError }}</p>
              <ReleaseNotes target="official" channel="official" :version="app.officialRemote?.version ?? null" />
              <p class="update-hint">面板升级前必须备份配置；备份成功后才会安装选定版本。更新完成后如需重启运行中的代理，将另行提示。</p>
              <div v-if="officialBusy || officialFailed || session.officialSucceeded && runtime.installOutcome?.version === session.officialCandidate" class="update-progress" aria-live="polite">
                <strong>{{ session.officialBackingUp || app.upgradeBackupBusy ? '正在备份配置' : officialBusy ? installPhaseLabel(runtime.install?.phase ?? 'preparing') : officialFailed ? '面板更新失败' : '面板更新已完成' }}</strong>
                <small>目标版本 {{ session.officialCandidate || app.officialRemote?.version }}</small>
                <progress v-if="officialBusy" aria-label="面板更新进度"></progress>
                <pre v-if="!session.officialBackingUp && session.officialInstallRequested && officialOutput.length && session.officialCandidate" class="update-console" tabindex="0" aria-label="面板安装输出">{{ officialOutput.join('\n') }}</pre>
                <p v-if="!officialBusy && session.officialSucceeded && runtime.installOutcome?.needsRestart" class="update-hint">面板更新已安装；请在运行操作中重启代理后生效。</p>
                <p v-if="!officialBusy && session.officialSucceeded && runtime.installOutcome?.protectionReconciliationPending" class="update-hint" role="status">安装已完成；保护备份完整性核对未完成，备份仍保留。请查看诊断日志。</p>
              </div>
              <p v-if="session.officialError || app.officialUpdateError" class="update-error update-query-error" role="alert">{{ session.officialError || app.officialUpdateError }}</p>
              <div v-if="officialAvailable || session.officialError || app.officialUpdateError" class="update-object-actions">
                <small v-if="app.upgradeLastBackup">最近备份 {{ app.upgradeLastBackup.backupId }}</small>
                <UiButton variant="primary" :disabled="busy || app.runtimeInstalling || app.upgradeBackupBusy || !officialAvailable" :loading="officialBusy || session.checking" data-testid="install-official-update" @click="installOfficial">{{ officialBusy ? '更新中' : session.officialError || app.officialUpdateError ? '重试面板更新' : '备份并更新面板' }}</UiButton>
              </div>
            </section>
          </template>
        </div>
        <footer class="update-center-foot"><p>{{ managerBusy || officialBusy ? '隐藏窗口后更新继续，可重新打开查看进度。' : '更新准备完成后由你选择重启时机。' }}</p><div><UiButton variant="ghost" @click="close">{{ managerBusy || officialBusy ? '隐藏' : '关闭' }}</UiButton></div></footer>
      </div>
    </div>
  </Teleport>
</template>


<style scoped>
/* §39：结果页每个对象一行、一屏看完；宽度 780px，正文不出现外层滚动套嵌套滚动。 */
.update-center.modal { width: min(780px, 100%); max-height: calc(100% - 24px); padding: 0; display: flex; flex-direction: column; overflow: hidden; }
.update-center-head, .update-center-foot { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); padding: var(--space-5) var(--space-6); }
.update-center-head { border-bottom: 1px solid var(--border); }
.update-center-head h3 { margin: var(--space-1) 0; }
.update-center-head p, .update-center-foot p { margin: 0; font-size: var(--text-label); color: var(--muted); }
.update-kicker { color: var(--muted); font-size: var(--text-caption); }
.update-center-body { overflow: auto; min-height: 0; padding: var(--space-4) var(--space-6); display: grid; gap: var(--space-3); }
.update-result-rows { display: grid; gap: var(--space-2); }
.update-result-row { display: grid; grid-template-columns: minmax(0,1fr) auto; align-items: center; gap: var(--space-4); padding: var(--space-3) var(--space-4); border: 1px solid var(--border); border-radius: var(--radius); background: color-mix(in srgb, var(--raised) 62%, transparent); }
.update-result-main { display: grid; grid-template-columns: minmax(150px,1.1fr) minmax(150px,1fr); gap: var(--space-1) var(--space-4); min-width: 0; }
.update-result-title { grid-column: 1; display: flex; align-items: center; gap: var(--space-2); min-width: 0; }
.update-result-title strong { overflow-wrap: anywhere; }
.update-result-version { grid-column: 2; display: flex; align-items: baseline; gap: var(--space-2); font-size: var(--text-label); color: var(--muted); font-variant-numeric: tabular-nums; }
.update-result-version b { color: var(--text); font-family: var(--font-code); overflow-wrap: anywhere; }
.update-result-source { grid-column: 1 / -1; margin: 0; color: var(--muted); font-size: var(--text-caption); overflow-wrap: anywhere; }
.update-result-summary { grid-column: 1 / -1; margin: 0; color: var(--text); font-size: var(--text-caption); line-height: 1.5; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.update-result-empty { grid-column: 1 / -1; margin: 0; color: var(--muted); font-size: var(--text-caption); }
.update-result-actions { display: flex; justify-content: flex-end; flex: none; }
.update-back { display: flex; }
.update-object { min-width: 0; padding: var(--space-2) 0 var(--space-4); border-bottom: 1px solid var(--border-soft); }
.update-detail-head { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: var(--space-2); }
.update-detail-head h4 { margin: 0 0 var(--space-3); font-size: var(--text-control); }
.update-detail-head time { color: var(--muted); font-size: var(--text-caption); }
.update-notes, .update-console { max-height: 180px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; padding: var(--space-3) var(--space-4); background: var(--raised); border: 1px solid var(--border-soft); border-radius: var(--radius-sm); font-size: var(--text-control); line-height: 1.7; }
.update-notes { font-family: inherit; margin: 0 0 var(--space-3); }
.update-console { max-height: 120px; margin: 0; font-family: var(--font-path); }
.update-hint { color: var(--muted); font-size: var(--text-label); line-height: 1.7; margin: var(--space-2) 0; }
.update-error { color: var(--red); font-size: var(--text-control); overflow-wrap: anywhere; margin: var(--space-2) 0; }
.update-query-error { white-space: pre-wrap; font-family: var(--font-path); }
.update-confirm-scope { margin: var(--space-3) 0; }
.update-object-actions { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: var(--space-3); margin-top: var(--space-3); }
.update-backup { display: flex; gap: var(--space-2); align-items: center; font-size: var(--text-label); }
.update-progress { display: grid; gap: var(--space-2); margin-top: var(--space-3); }
.update-progress progress { width: 100%; height: 6px; accent-color: var(--accent); }
.update-progress small, .update-object-actions small { color: var(--muted); overflow-wrap: anywhere; }
.update-center-foot { border-top: 1px solid var(--border); }
.update-center-foot > div { display: flex; gap: var(--space-2); flex: none; }
@media (max-width: 600px) {
  .update-center-head, .update-center-foot { padding: var(--space-4); flex-wrap: wrap; }
  .update-center-body { padding: var(--space-3) var(--space-4); }
  .update-result-row { grid-template-columns: minmax(0,1fr); gap: var(--space-2); }
  .update-result-main { grid-template-columns: minmax(0,1fr); }
  .update-result-version { grid-column: 1; }
  .update-result-actions { justify-content: flex-start; }
}
</style>

