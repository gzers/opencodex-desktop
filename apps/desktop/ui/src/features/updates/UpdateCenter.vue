<script lang="ts">
import { reactive } from 'vue'
// 界面请求按应用实例保存，路由重挂载不会丢失在途操作；进度事实来自既有 store。
const sessions = new WeakMap<object, ReturnType<typeof createSession>>()
function createSession() {
  return reactive({ checking: false, managerInstalling: false, officialInstalling: false, officialBackingUp: false, officialSucceeded: false,
    managerError: '', managerFailedOperation: '', officialError: '', officialInstallRequested: false, officialFailureLines: [] as string[], managerCheckError: '',
    officialCandidate: '', managerCandidate: '', officialCurrent: '', managerCurrent: '', backup: true })
}
function sessionFor(app: object) {
  let session = sessions.get(app)
  if (!session) { session = createSession(); sessions.set(app, session) }
  return session
}
</script>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ReleaseNotes from './ReleaseNotes.vue'
import RuntimeVersionFacts from './RuntimeVersionFacts.vue'
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
const versions=useUpdatesStore()
const session = sessionFor(app)
const selected = ref<Target | null>(props.target)
const dialogHost = ref<InstanceType<typeof UiDialog> | null>(null)
const dialog = computed(() => dialogHost.value?.element ?? null)
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
const officialAvailable = computed(() => !versions.versionFactsError&&!versions.versionFactsLoading&&versions.versionFacts?.disk.integrity==='consistent'&&!app.officialRemoteError && hasNewerVersion(versions.versionFacts?.disk.diskVersion??null,app.officialRemote?.version??null)===true)
const officialQueryError = computed(() => versions.versionFactsError|| (versions.versionFacts&&versions.versionFacts.disk.integrity!=='consistent'?'程序来源已变化或尚待核对，请先检查安装配置。':'') || (app.officialRemoteError
  ? app.officialRemoteErrorDetail || '版本查询失败，请检查网络或运行来源后重试。'
  : ''))
const managerResult = computed(() => {
  if (managerBusy.value) return '更新中…'
  if (pendingRestart.value) return '更新已准备就绪'
  if (session.managerError || progress.value?.stage === 'failed') return '安装失败 · 可重试'
  if (session.managerCheckError || status.value?.error) return '检查失败'
  if (session.checking) return '正在检查'
  if (managerAvailable.value) return '有可用更新'
  if (!status.value?.lastCheckedAt) return '尚未检查'
  return status.value.availableVersion && hasNewerVersion(status.value.currentVersion, status.value.availableVersion) === null ? '无法比较版本' : '已是最新版本'
})
const officialResult = computed(() => {
  if (officialBusy.value) return '更新进行中'
  if (session.officialError || app.officialUpdateError) return '安装失败 · 可重试'
  if (app.officialRemoteLoading || versions.versionFactsLoading) return '正在检查'
  if (officialQueryError.value||!versions.versionFacts) return '实际版本待核实'
  if (officialAvailable.value) return '发现新版本'
  if (!app.officialRemote) return '尚未检查'
  return hasNewerVersion(versions.versionFacts?.disk.diskVersion??null, app.officialRemote.version) === false ? '已是最新版本' : '无法比较版本'
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
const officialNotes = computed<string | null>(() => officialAvailable.value ? '查看本版本更新内容与范围，确认后开始更新。' : null)
const showingProgress = computed(() => selected.value === 'manager'
  ? managerBusy.value || pendingRestart.value || !!session.managerError || progress.value?.stage === 'failed'
  : selected.value === 'official' && (!!session.officialCandidate || officialBusy.value || officialFailed.value))
const selectedAvailable = computed(() => selected.value === 'manager' ? managerAvailable.value : officialAvailable.value)
const hasProgressActions = computed(() => selected.value === 'manager'
  ? pendingRestart.value || managerAvailable.value && !managerBusy.value
  : selected.value === 'official' && officialAvailable.value && !officialBusy.value && !session.officialSucceeded)
const selectedTitle = computed(() => (selected.value === 'manager' ? '桌面管理器' : 'OpenCodex 面板') + (showingProgress.value ? '更新进度' : selectedAvailable.value ? '更新详情与确认' : '更新详情'))
const selectedCurrent = computed(() => selected.value === 'manager' ? showingProgress.value && session.managerCurrent || status.value?.currentVersion || app.aboutApp?.version || '未知' : showingProgress.value && session.officialCurrent || versions.versionFacts?.disk.diskVersion || '未知')
const selectedCandidate = computed(() => selected.value === 'manager'
  ? showingProgress.value ? progress.value?.candidateVersion || status.value?.pendingRestart || session.managerCandidate || status.value?.availableVersion : hasNewerVersion(status.value?.currentVersion ?? null, status.value?.availableVersion ?? null) === false ? null : status.value?.availableVersion
  : showingProgress.value ? session.officialCandidate || app.officialRemote?.version : hasNewerVersion(versions.versionFacts?.disk.diskVersion??null,app.officialRemote?.version??null)===false ? null : app.officialRemote?.version)

const selectedSource = computed(() => selected.value === 'manager' ? 'OpenCodex Desktop · ' + (status.value?.channel === 'beta' ? '测试' : '稳定') + '通道' : '官方项目 · @bitkyc08/opencodex')
const managerUnavailableText = computed(() => session.managerError || progress.value?.stage === 'failed' ? '上次安装未完成，可进入详情重试。' : status.value?.availableVersion && hasNewerVersion(status.value.currentVersion, status.value.availableVersion) === null ? '无法比较版本。' : pendingRestart.value ? '更新已准备就绪，请进入详情重启。' : status.value?.lastCheckedAt ? '当前没有待确认的更新。' : '尚未检查。')
const officialUnavailableText = computed(() => session.officialError || app.officialUpdateError ? '上次安装未完成，可进入详情重试。' : officialAvailable.value ? '' : '当前没有待确认的更新。')
const managerStage = computed(() => {
  if (pendingRestart.value) return '安装就绪，等待重启'
  if (session.managerError || progress.value?.stage === 'failed') return '更新失败，当前版本保留'
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
  session.managerCurrent = snapshot.currentVersion
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
  session.officialCurrent = versions.versionFacts?.disk.diskVersion ?? '未知'
  session.officialError = ''
  try {
    const reviewed=versions.versionFacts?.disk.confirmationDigest
    if(!reviewed||!await versions.loadVersions()||versions.versionFacts?.disk.integrity!=='consistent'||versions.versionFacts.disk.confirmationDigest!==reviewed){session.officialError='实际程序在确认前变化，未开始更新，请重新核对。';return}
    // 每次面板更新前备份，失败时阻断安装，不把旧备份冒充本次备份。
    const backup = await useUpdatesStore().createUpgradeBackup()
    if (!backup) { session.officialError = app.upgradeBackupError || '升级前备份失败，尚未安装更新。'; return }
    session.officialBackingUp = false
    session.officialInstallRequested = true
    const ok = await app.applyOfficialUpdate(candidateVersion,reviewed)
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
    if (!officialBusy.value && session.officialSucceeded && session.officialCandidate !== app.officialRemote?.version) { session.officialCandidate = ''; session.officialSucceeded = false }
    selected.value = props.target
    // 打开详情只读缓存，反复进入路由不会隐式联网检查或备份。
    void getUpdateStatus().catch(() => {})
    void versions.loadVersions()
  }
}, { immediate: true })
defineExpose({ check })
</script>

<template>
  <UiDialog ref="dialogHost" :open="open" :title="selected ? selectedTitle : '检查更新结果'" wide panel-class="update-center" data-testid="update-center" initial-focus="dialog" @close="close">
        <div class="update-center-body">
          <RuntimeVersionFacts v-if="!selected||selected==='official'" />
          <template v-if="!selected">
            <p class="update-hint">分别查看更新内容与范围，确认后才会开始更新。</p>
            <div class="update-result-rows" data-testid="update-result-rows">
              <section v-for="target in (['manager','official'] as const)" :key="target" class="update-result-row" :data-testid="target + '-update-row'">
                <div class="update-result-main">
                  <div class="update-result-title"><strong>{{ target === 'manager' ? '桌面管理器' : 'OpenCodex 面板' }}</strong><span class="tag">{{ target === 'manager' ? managerResult : officialResult }}</span></div>
                  <div class="update-result-version"><span>{{ target === 'manager' ? status?.currentVersion ?? app.aboutApp?.version ?? '未知' : versions.versionFacts?.disk.diskVersion ?? '未知' }}</span><template v-if="target === 'manager' ? managerAvailable : officialAvailable"><span aria-hidden="true">→</span><b>{{ target === 'manager' ? status?.availableVersion : app.officialRemote?.version }}</b></template></div>
                  <p class="update-result-source">{{ target === 'manager' ? 'OpenCodex Desktop · ' + (status?.channel ?? 'stable') : '官方项目 · @bitkyc08/opencodex' }}</p>
                  <p v-if="target === 'manager' && managerAvailable && summaryOf(status?.notes)" class="update-result-summary">{{ summaryOf(status?.notes) }}</p>
                  <p v-if="target === 'official' && officialAvailable" class="update-result-summary">{{ summaryOf(officialNotes) }}</p>
                  <p v-if="target === 'manager' && (session.managerCheckError || status?.error)" class="update-error" role="status">{{ session.managerCheckError || status?.error }}</p>
                  <p v-else-if="target === 'official' && officialQueryError" class="update-error" role="status">{{ officialQueryError.split('\n')[0] }}</p>
                  <p v-else-if="target === 'manager' ? !managerAvailable : !officialAvailable" class="update-result-empty">{{ target === 'manager' ? managerUnavailableText : officialUnavailableText }}</p>
                </div>
                <div class="update-result-actions"><UiButton :data-testid="target + '-update-detail'" @click="selected = target">查看详情</UiButton></div>
              </section>
            </div>
          </template>
          <section v-else class="update-object" :data-testid="selected + '-update-card'">
            <div v-if="!showingProgress" class="update-result-heading"><span>版本升级</span><span class="tag">{{ selected === 'manager' ? managerResult : officialResult }}</span></div>
            <div class="update-version-hero"><span>{{ selectedCurrent }}</span><template v-if="selectedCandidate"><span aria-hidden="true">→</span><strong>{{ selectedCandidate }}</strong></template></div>
            <p class="update-hint">{{ selectedSource }}<template v-if="!showingProgress && selectedAvailable"> · {{ selected === 'manager' ? '安装后需重启管理器' : '更新已登记的面板包体' }}</template></p>
              <p v-if="selected === 'official' && officialQueryError" class="update-error update-query-error" role="status" data-testid="official-query-error">{{ officialQueryError }}</p>
            <p v-if="selected === 'manager' && (session.managerCheckError || status?.error)" class="update-error update-query-error" role="status">{{ session.managerCheckError || status?.error }}</p>
            <template v-if="!showingProgress">
              <ReleaseNotes v-if="selectedCandidate" :target="selected" :channel="selected === 'manager' ? status?.channel ?? 'stable' : 'official'" :version="selectedCandidate" :updater-notes="selected === 'manager' ? status?.notes : null" />
              <div v-if="selected === 'manager' && managerAvailable" class="update-confirm-scope"><label class="update-backup"><input v-model="session.backup" type="checkbox" :disabled="managerBusy">更新前备份管理器偏好（建议）</label><p class="update-hint">保存到当前数据根的 backups。包含管理器偏好、清单与完整性摘要；不包含面板完整配置、会话、登录态或安装包。</p></div>
              <p v-if="selected === 'official' && officialAvailable" class="update-hint">面板升级前先备份配置；备份成功后才会安装选定版本。</p>
            </template>
            <template v-else-if="selected === 'manager'">
              <div class="update-progress" aria-live="polite" data-testid="manager-update-progress"><strong>{{ managerStage }}</strong><progress v-if="managerBusy" aria-label="管理器更新进度" :value="downloadPercent" max="100"></progress><small v-if="managerBusy && progress?.stage === 'downloading'">{{ bytes(progress.downloadedBytes) }}<template v-if="progress.totalBytes !== null"> / {{ bytes(progress.totalBytes) }}</template><template v-if="downloadPercent !== undefined"> · {{ Math.floor(downloadPercent) }}%</template></small><p v-if="pendingRestart" class="update-hint">当前仍为 {{ selectedCurrent }}。重启后核验 {{ selectedCandidate }} 是否生效。</p><p v-else-if="managerBusy" class="update-hint">可关闭此窗口，稍后从概览或设置继续查看。</p></div>
              <p v-if="session.managerError" class="update-error" role="alert">{{ session.managerError }}</p>
            </template>
            <template v-else>
              <div class="update-progress" aria-live="polite" data-testid="official-update-progress"><strong>{{ session.officialBackingUp || app.upgradeBackupBusy ? '正在备份配置' : officialBusy ? installPhaseLabel(runtime.install?.phase ?? 'preparing') : officialFailed || session.officialError ? '面板更新失败' : session.officialSucceeded ? '面板更新已完成' : '等待安装结果' }}</strong><progress v-if="officialBusy" aria-label="面板更新进度"></progress><pre v-if="!session.officialBackingUp && session.officialInstallRequested && officialOutput.length && session.officialCandidate" class="update-console" tabindex="0" aria-label="面板安装输出">{{ officialOutput.join('\n') }}</pre><p v-if="!officialBusy && session.officialSucceeded && runtime.installOutcome?.needsRestart" class="update-hint">面板更新已安装；请在运行操作中重启代理后生效。</p><p v-if="!officialBusy && session.officialSucceeded && runtime.installOutcome?.protectionReconciliationPending" class="update-hint" role="status">安装已完成；保护备份完整性核对未完成，备份仍保留。请查看诊断日志。</p></div>
              <p v-if="session.officialError || app.officialUpdateError" class="update-error update-query-error" role="alert">{{ session.officialError || app.officialUpdateError }}</p>
            </template>
            <ol v-if="showingProgress" class="update-stage-list" aria-label="更新阶段"><li>准备</li><li>下载</li><li>校验</li><li>{{ selected === 'manager' ? '安装与重启' : '应用与核验' }}</li></ol>
            <div v-if="showingProgress && hasProgressActions" class="update-progress-actions">
              <UiButton v-if="selected === 'manager' && pendingRestart" variant="primary" :loading="restarting" @click="restart">重启并应用更新</UiButton>
              <UiButton v-else-if="selected === 'manager' && managerAvailable && !managerBusy" variant="primary" :disabled="busy" data-testid="install-manager-update" @click="installManager">重试安装</UiButton>
              <UiButton v-if="selected === 'official' && officialAvailable && !officialBusy && !session.officialSucceeded" variant="primary" :disabled="busy || app.runtimeInstalling || app.upgradeBackupBusy" data-testid="install-official-update" @click="installOfficial">重试面板更新</UiButton>
            </div>
          </section>
        </div>
    <template v-if="selected && !showingProgress && selectedAvailable" #actions="{ close: requestClose }">
      <UiButton variant="ghost" @click="requestClose">取消</UiButton>
      <UiButton v-if="selected === 'manager'" variant="primary" :disabled="busy" data-testid="install-manager-update" @click="installManager">确认更新</UiButton>
      <UiButton v-if="selected === 'official'" variant="primary" :disabled="busy || app.runtimeInstalling || app.upgradeBackupBusy" :loading="session.checking" data-testid="install-official-update" @click="installOfficial">备份并更新面板</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
/* §39：结果页每个对象一行、一屏看完；宽度 780px，正文不出现外层滚动套嵌套滚动。 */
:global(dialog.ui-dialog.update-center.wide) { width:min(780px,calc(100vw / var(--ui-zoom) - 40px)); }
.update-center-body { display:grid; gap:var(--space-3); min-width:0; }
.update-result-rows { display: grid; gap: var(--space-2); }
.update-result-row { display: grid; grid-template-columns: minmax(0,1fr) auto; align-items: center; gap: var(--space-4); padding: var(--space-3) var(--space-4); border:0; border-bottom:1px solid var(--border); border-radius:0; background:transparent; }
.update-result-main { display: grid; grid-template-columns: minmax(0,1.1fr) minmax(0,1fr); gap: var(--space-1) var(--space-4); min-width: 0; }
.update-result-title { grid-column: 1; display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-2); min-width: 0; }
.update-result-title strong { overflow-wrap: anywhere; }
.update-result-version { grid-column: 2; display: flex; flex-wrap: wrap; align-items: baseline; gap: var(--space-2); font-size: var(--text-label); color: var(--muted); font-variant-numeric: tabular-nums; }
.update-result-version b { color: var(--text); font-family: var(--font-code); overflow-wrap: anywhere; }
.update-result-source { grid-column: 1 / -1; margin: 0; color: var(--muted); font-size: var(--text-caption); overflow-wrap: anywhere; }
.update-result-summary { grid-column: 1 / -1; margin: 0; color: var(--text); font-size: var(--text-caption); line-height: 1.5; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.update-result-empty { grid-column: 1 / -1; margin: 0; color: var(--muted); font-size: var(--text-caption); }
.update-result-actions { display: flex; justify-content: flex-end; flex: none; }
.update-back { display: flex; }
.update-object { min-width:0; }
.update-result-heading {display:flex;justify-content:space-between;align-items:center;color:var(--muted);font-size:var(--text-label)}
.update-version-hero {display:flex;align-items:baseline;gap:14px;margin:12px 0 8px;font-size:var(--text-title);font-variant-numeric:tabular-nums}
.update-version-hero>span {color:var(--muted)}
.update-detail-head { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: var(--space-2); }
.update-detail-head h4 { margin: 0 0 var(--space-3); font-size: var(--text-control); }
.update-detail-head time { color: var(--muted); font-size: var(--text-caption); }
.update-notes, .update-console { max-height: 180px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; padding: var(--space-3) var(--space-4); background: var(--raised); border: 1px solid var(--border-soft); border-radius: var(--radius-sm); font-size: var(--text-control); line-height: 1.7; }
.update-notes { font-family: inherit; margin: 0 0 var(--space-3); }
.update-console { max-height: 120px; margin: 0; font-family: var(--font-path); }
.update-hint { color: var(--muted); font-size: var(--text-label); line-height: 1.7; margin: var(--space-2) 0; }
.update-error { color: var(--red); font-size: var(--text-control); overflow-wrap: anywhere; margin: var(--space-2) 0; }
.update-query-error { white-space: pre-wrap; font-family: var(--font-path); }
.update-confirm-scope { border-top:1px solid var(--border); margin-top:20px; padding-top:16px; }
.update-object-actions { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: var(--space-3); margin-top: var(--space-3); }
.update-backup { display: flex; gap: var(--space-2); align-items: center; font-size: var(--text-label); }
.update-progress-actions {display:flex;gap:var(--space-2);justify-content:flex-start;margin-bottom:8px}
.update-stage-list {display:flex;flex-wrap:wrap;gap:8px 24px;margin:24px 0;padding-left:20px;color:var(--muted);font-size:var(--text-label)}
.update-progress { display: grid; gap: var(--space-2); margin-top: var(--space-3); }
.update-progress progress { width: 100%; height: 6px; accent-color: var(--accent); }
.update-progress small, .update-object-actions small { color: var(--muted); overflow-wrap: anywhere; }
.update-center-foot { padding:16px 24px; justify-content:flex-end; }
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

