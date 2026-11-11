<script setup lang="ts">
import { layoutObserver } from '@/lib/layoutObserver'
import { displayPath } from '@/lib/paths'
// 概览：运行 / 环境准备双形态（UI规范 §25）。
//
// - 环境就绪（ready / ready_via_source）：无框居中状态区；Logo → 主线状态 → 必要说明 → 当前操作；
//   环境明细并入运行详情；下方版本升级 / 配置迁移 / WebDAV 同步三卡，最后为最近事件摘要。
// - 环境准备（检查中或真实前置缺失）：三卡与最近事件退出布局，顶部缩短，下方一张准备卡
//   （左纵向检查 / 右当前指引 + 命令 + 动作）。
// - 状态形象只消费既有运行投影（`projectMotion`），不引入新的状态来源；环境走既有 presentation。
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { LifecycleAction } from '@/types/ui'
import { useRouteStore } from '@/stores/routes'
import { useAppStore } from '@/stores/app'
import { useAppController } from '@/composables/useAppController'
import AppTopbar from '@/components/AppTopbar.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import InformationRow from '@/components/patterns/InformationRow.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UpdateCenter from '@/features/updates/UpdateCenter.vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { managerInstallProgress, managerUpdateStatus } from '@/features/updates/update'
import { hasNewerVersion } from '@/features/updates/version'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import EnvironmentGate from '@/features/environment/components/EnvironmentGate.vue'
import { webdavStates } from '@/features/sync/states'
import { useSyncStore } from '@/features/sync/store'
import { remoteVersionHint as versionHint } from '@/features/sync/observation'
import { clockText, syncActivity } from '@/features/sync/activity'
import { runtimeSourceLabel } from '@/lib/labels'
import { useEnvironmentStore } from '@/features/environment/store'
import { buildEnvironmentPresentation, environmentChecksOrdered } from '@/features/environment/presentation'
import { projectMotion } from '@/features/runtime/motion/projection'

const routes = useRouteStore()
const app = useAppStore()
const controller = useAppController()
const syncStore = useSyncStore()

const updateCenter = ref<InstanceType<typeof UpdateCenter> | null>(null)
const updateOpen = ref(false)
const updateTarget = ref<'manager' | 'official' | null>(null)
function openUpdates(target: 'manager' | 'official' | null = null, check = false) {
  updateTarget.value = target
  updateOpen.value = true
  if (check) void updateCenter.value?.check()
}
const importOpen = ref(false);const overviewTransfer=ref<'file'|'webdav'|null>(null)
const detailOpen = ref(false)
const envOpen = ref(false)
const copiedCommand = ref<string | null>(null)
const documentVisible = ref(true)
// 按**应用窗口可用逻辑高度**切换紧凑版（参考阈值 680px，考虑界面缩放），不按外部浏览器尺寸猜测。
const compact = ref(false)

function onVisibilityChange() {
  documentVisible.value = document.visibilityState !== 'hidden'
}
let sizeObserver: ReturnType<typeof layoutObserver> = undefined
function updateSize() {
  const win = document.querySelector('.app-window')
  const measured = win?.getBoundingClientRect().height ?? window.innerHeight
  const zoom = Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom')) || 1
  const logical = measured / zoom
  const next = logical < 680
  if (next !== compact.value) compact.value = next
  const value=next ? 'compact' : 'standard'
  if(document.documentElement.getAttribute('data-overview-size')!==value) document.documentElement.setAttribute('data-overview-size',value)
}

onMounted(() => {
  document.addEventListener('visibilitychange', onVisibilityChange)
  updateSize()
  if (typeof ResizeObserver !== 'undefined') {
    const win = document.querySelector('.app-window')
    if (win) {
      sizeObserver = layoutObserver(updateSize)!
      sizeObserver.observe(win)
    }
  }
})
onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', onVisibilityChange)
  sizeObserver?.disconnect()
  sizeObserver = undefined
  document.documentElement.removeAttribute('data-overview-size')
})

const scenario = controller.scenario

// 状态形象：主线 / 新鲜度 / 持续问题 → 一种内部结构。只读投影，不新建状态机。
const ongoingProblem = computed(() =>
  ['external_takeover', 'unreachable'].includes(app.runtimeState),
)
// 事实新鲜度接线：**最近一次采集失败**（`statusError`）且手上是**此前真实观测到的**快照
// （`source === 'live'`）时，保留旧事实但标记为「过期」；fixture / 未配置来源不冒充过期，
// 进行中的观测/操作也不受旧事实顶替（见 projection 的 IN_FLIGHT）。不伪造新鲜，也不伪造过期。
const fresh = computed(() => !(app.statusError && app.statusSnapshot?.source === 'live'))
const motion = computed(() =>
  projectMotion({ runtimeState: app.runtimeState, fresh: fresh.value, problem: ongoingProblem.value }),
)
// 只在概览可见且窗口在前台时逐帧更新。
const motionActive = computed(() => routes.current === 'overview' && documentVisible.value)

function recheckEnvironment() {
  void controller.refreshEnvironment()
}

const knownInstall = computed(() => !['loading', 'not_found'].includes(app.runtimeState))
const version = computed(() =>
  knownInstall.value
    ? (app.statusSnapshot?.facts.version_drift ?? app.officialProject?.version ?? '—')
    : '—',
)
// 版本行按原型「管理器 / 面板 + 版本 + 阶段 + ›」三段呈现（index.html:1877-1878；
// 运行时阶段文案见 maintenance-ui.js:91 的 labels）。阶段只由已有更新事实推导，不补假状态。
type UpdatePhase = 'idle' | 'checking' | 'available' | 'latest' | 'failed' | 'applying' | 'restart-required' | 'unknown'
const updatePhaseLabels: Record<UpdatePhase, string> = {
  idle: '尚未检查',
  checking: '检查中…',
  available: '有可用更新',
  latest: '已是最新',
  failed: '失败，可重试',
  applying: '更新中…',
  'restart-required': '等待重启',
  unknown: '—',
}
const managerVersion = computed(() => managerUpdateStatus.value?.currentVersion ?? app.aboutApp?.version ?? '—')
const managerPhase = computed<UpdatePhase>(() => {
  const stage = managerInstallProgress.value?.stage
  if (stage === 'failed') return 'failed'
  if (stage === 'pending_restart' || managerUpdateStatus.value?.pendingRestart) return 'restart-required'
  if (stage) return 'applying'
  const status = managerUpdateStatus.value
  if (status?.error) return 'failed'
  if (!status) return 'idle'
  // 与 UpdateCenter 同一口径：检查完成但没有候选版本就是「已是最新」，不退回破折号。
  const newer = status.availableVersion ? hasNewerVersion(status.currentVersion, status.availableVersion) : false
  if (newer === true) return 'available'
  if (!status.lastCheckedAt) return 'idle'
  return newer === false ? 'latest' : 'unknown'
})
const officialPhase = computed<UpdatePhase>(() => {
  if (app.officialUpdateBusy) return 'applying'
  if (app.officialProjectLoading || app.officialRemoteLoading) return 'checking'
  if (app.officialUpdateError || app.officialProjectError || app.officialRemoteError) return 'failed'
  if (!app.officialRemote) return 'idle'
  const newer = hasNewerVersion(app.officialProject?.version ?? null, app.officialRemote.version)
  return newer === true ? 'available' : newer === false ? 'latest' : 'unknown'
})
// 阶段无法判定时，退到数据里已有的通道名（如 stable / beta）；没有通道才用「—」。
const managerPhaseLabel = computed(() =>
  managerPhase.value === 'unknown' ? (managerUpdateStatus.value?.channel ?? '—') : updatePhaseLabels[managerPhase.value],
)
const officialPhaseLabel = computed(() => updatePhaseLabels[officialPhase.value])
// S18-09：0.2.18 原型 maintenance-ui.js:7-12/95-101 概览版本行＝左对象名，右侧「阶段词（仅检查中/失败/更新中/待重启）+ 版本 + 状态点 + ›」。
// 有候选/更新中/待重启显示候选版本，已是最新显示当前版本，其余显示「—」且不带点；尚未检查与进行中的行不可点。
const ROW_DOTS: Partial<Record<UpdatePhase, string>> = { idle: 'idle', checking: 'info', available: 'ok', latest: 'idle', failed: 'error', applying: 'info', 'restart-required': 'info' }
const ROW_WORDS: Partial<Record<UpdatePhase, string>> = { checking: '检查中', failed: '失败', applying: '更新中', 'restart-required': '待重启' }
function versionRow(key: 'manager' | 'official', label: string, name: string, phase: UpdatePhase, current: string | null | undefined, candidate: string | null | undefined) {
  const useCandidate = ['available', 'applying', 'restart-required'].includes(phase)
  const shown = useCandidate ? candidate : phase === 'latest' ? current : null
  const version = shown || '—'
  return {
    key, label, phase, version,
    word: ROW_WORDS[phase] ?? '',
    dot: shown ? (ROW_DOTS[phase] ?? '') : '',
    disabled: phase === 'idle' || phase === 'checking' || phase === 'applying',
    aria: '查看' + name + '更新详情（' + (key === 'manager' ? managerPhaseLabel.value : officialPhaseLabel.value) + '）',
  }
}
const versionRows = computed(() => [
  versionRow('manager', '管理器', '桌面管理器', managerPhase.value, managerVersion.value === '—' ? null : managerVersion.value, managerInstallProgress.value?.candidateVersion ?? managerUpdateStatus.value?.availableVersion),
  versionRow('official', '面板', ' OpenCodex 面板', officialPhase.value, app.officialProject?.version, app.officialRemote?.version),
])
const runtime = computed(() =>
  knownInstall.value && app.statusSnapshot?.facts.runtime_label
    ? `Codex 运行时（${runtimeSourceLabel(app.statusSnapshot.facts.runtime_label)}）`
    : '—',
)
const managerDataRoot = computed(() => app.dataRootConfig?.currentDataRoot || app.dataRootPath || '')
const managerHome = computed(() => app.dataRootConfig?.currentOpencodexHome || '')
const dataRoot = computed(() => displayPath(managerDataRoot.value || app.statusSnapshot?.facts.data_root) || '—')
const home = computed(() => displayPath(managerHome.value || app.statusSnapshot?.facts.opencodex_home) || '—')

// 环境结论（就绪态并入运行详情；准备态由准备卡承载）。
const environment = computed(() =>
  buildEnvironmentPresentation(app.environment, app.environmentLoading, app.runtimeSource?.kind ?? null, app.aboutApp?.platform, useEnvironmentStore().error),
)
// 按检查顺序呈现：检查中只亮当前项，其后为「待检查」。
const envChecks = computed(() =>
  environmentChecksOrdered(environment.value.checks, environment.value.state === 'checking'),
)

// 双形态：环境未就绪（检查中或真实前置缺失）走「环境准备」，其余走「环境就绪」（UI规范 §25.1）。
// `blocked` 只覆盖真实前置缺失；检查中由 `state === 'checking'` 一并纳入准备态。
const gateBlocked = computed(() => environment.value.state === 'checking' || environment.value.blocked)
const mode = computed<'ready' | 'setup'>(() => (gateBlocked.value ? 'setup' : 'ready'))
// 准备态的主线说明与入口按 §25 调整：不充当无效详情入口。
const caption = computed(() => {
  if (app.clientRecoveryRequired) return '客户端配置恢复尚未完成，请查看诊断。'
  if (!gateBlocked.value) return motion.value.caption
  return environment.value.state === 'checking' ? '正在确认启动条件' : '启动条件未满足 · 请按下方指引处理'
})
// 副标题按双形态改写（§25.1 / 原型 overview-dual.js）。
const pageSubtitle = computed(() =>
  gateBlocked.value ? '完成启动前的环境准备，再继续运行操作。' : '运行状态与常用操作；环境明细收进运行详情。',
)

// 最近事件：标准显示最近两条，紧凑一条；完整历史保留在诊断。
const recentEvents = computed(() => app.recentEvents.slice(0, compact.value ? 1 : 2))

// 动作：主操作直接露出，低频动作收进「更多」（UI规范 §19 / §20）。
// 动作层级对齐已确认原型 `actionsByState`（运行中＝打开面板/停止/重启/查看日志，全部内联）。
// 既有「刷新状态」不在原型主线动作内，但仍保留出口：收进「更多」，不删既有功能。
const PRIMARY = new Set(['start', 'panel'])
const SECONDARY = new Set(['stop', 'restart', 'logs', 'advice'])
const primaryAction = computed(() => scenario.value.actions.find(action => PRIMARY.has(action)) ?? null)
const secondaryActions = computed(() => scenario.value.actions.filter(action => SECONDARY.has(action)))
const moreActions = computed(() => scenario.value.actions.filter(action => !PRIMARY.has(action) && !SECONDARY.has(action)))

const lifecycleActions = new Set(['start', 'stop', 'restart'])
function isLifecycleAction(action: string) {
  return lifecycleActions.has(action)
}
function actionBlocked(action: string) {
  return isLifecycleAction(action) && (app.processActionBusy || app.processActionBlocked(action as LifecycleAction))
}
function actionLabel(action: string) {
  if (app.processActionBusy && app.processAction === action) {
    return action === 'start' ? '启动中…' : action === 'stop' ? '停止中…' : '重启中…'
  }
  // 与原型一致（overview.html `render()`）：启动失败时启动动作读作「重试启动」，
  // 其余状态沿用同一份动作文案（`actionLabels`）。
  if (action === 'start' && app.runtimeState === 'starting_failed') return '重试启动'
  return controller.actionLabels[action]
}

function openRuntimeDetail() {
  if (gateBlocked.value) return
  detailOpen.value = true
}
function openEnvironment() {
  envOpen.value = true
}

async function copyValue(value: string, event?: MouseEvent) {
  try {
    if (!navigator.clipboard) {
      app.showToast('剪贴板不可用；请手动复制该值。')
      return
    }
    await navigator.clipboard.writeText(value)
    app.showToast(`已复制：${value}`)
  } catch {
    app.showToast('复制失败；请手动复制该值。')
  } finally {
    if (event) (event.currentTarget as HTMLButtonElement | null)?.blur()
  }
}

async function copyCommand(command: string) {
  try {
    if (!navigator.clipboard) {
      app.showToast(`剪贴板不可用；不会执行 ${command}，请手动运行。`)
      return
    }
    await navigator.clipboard.writeText(command)
    copiedCommand.value = command
    window.setTimeout(() => {
      if (copiedCommand.value === command) copiedCommand.value = null
    }, 1400)
    app.showToast(`已复制 ${command}；不会代你执行，请手动运行。`)
  } catch {
    app.showToast(`复制失败；不会执行 ${command}，请手动运行。`)
  }
}

function installManaged(source: 'registry' | 'offline') {
  app.requestInstallModal(source)
  envOpen.value = false
}

function guide() {
  envOpen.value = false
  routes.go('settings', { section: 'installation' })
}

// 顺序与文案对齐 0.2.0 原型 index.html:1866/1885/1905：版本升级 → 文件同步 → WebDAV 同步。
// 「最近事件」在原型里排在模块区之前（index.html:1853），见模板顺序。
function activityText(entry: { time: string; count: number } | undefined, empty: string) {
  const time = clockText(entry?.time)
  return entry && time ? time + ' · ' + entry.count + ' 项' : empty
}
// Last observations belong to the saved endpoint revision, never to a draft form.
const webdavView = computed(() => webdavStates[app.webdavState])
const remoteVersionTime = computed(() => clockText(syncStore.observation?.remoteTime ?? undefined) || '—')
const remoteVersionHint = computed(() => versionHint(syncStore.observation, syncStore.checkingVersion))
const remoteVersionTitle = computed(() => {
  const observation = syncStore.observation
  if (!observation) return '尚未读取当前连接的云端版本'
  return '观察时间：' + new Date(observation.remoteObservedAt ?? observation.observedAt).toLocaleString('zh-CN') + (observation.remoteRevision ? '；快照：' + observation.remoteRevision : '') + (observation.remoteTime ? '；保存版本：' + new Date(observation.remoteTime).toLocaleString('zh-CN') : '')
})
const webdavConnectionText = computed(() => {
  const time = clockText(syncStore.observation?.observedAt)
  const label = syncStore.checkingVersion ? '正在检测' : syncStore.observation?.comparison === 'same' && syncStore.observation.source === 'sync-preview' ? '最近检查一致' : webdavView.value.conn
  return label + (time && !['unconfigured', 'configured', 'unknown', 'connecting', 'syncing'].includes(app.webdavState) ? ' · ' + time : '')
})
const mods = computed(() => [
  {
    key: 'upgrade',
    title: '版本升级',
    // 0.2.18 原型 index.html:747 隐藏版本卡与 WebDAV 卡头部标签：状态已在正文逐行给出。
    tag: '',
    rows: [],
    // 原型 maintenance-ui.js:103：更新中 / 待重启时在检查更新后追加对应对象的「查看…进度」。
    actions: [
      { label: '检查更新', cls: 'btn primary', disabled: versionRows.value.some(row => row.phase === 'checking' || row.phase === 'applying'), run: () => openUpdates(null, true) },
      ...versionRows.value.filter(row => row.phase === 'applying' || row.phase === 'restart-required').map(row => ({ label: '查看' + row.label + '进度', cls: 'btn', run: () => openUpdates(row.key) })),
    ],
  },
  {
    key: 'migration',
    // 原型 index.html:1891 标题是「文件同步」，不是「配置迁移」。
    title: '文件同步',
    // S18-01：0.2.18 原型 index.html:1930-1946 无标签，只显示最近一次导入 / 导出（时间 + 项数）。
    tag: '',
    rows: [['最近导入', activityText(syncActivity.value.import, '尚未导入')], ['最近导出', activityText(syncActivity.value.export, '尚未导出')]],
    // 导入在概览原位打开；导出进入文件同步页按记住的范围直接准备导出包。
    actions: [
      { label: '导入', cls: 'btn', run: () => { importOpen.value = true } },
      { label: '导出', cls: 'btn', run: () => { overviewTransfer.value='file';syncStore.requestAction('export') } },
    ],
  },
  { key: 'sync', title: 'WebDAV 同步', tag: '', rows: [['连接状态', webdavConnectionText.value]], actions: [...webdavView.value.actions.map(a => ({ label: a.label, cls: a.cls, disabled: !!a.disabled, run: () => { if(a.act==='result')syncStore.openResult();else if(a.act==='connection')syncStore.openConnection();else{overviewTransfer.value='webdav';syncStore.requestAction(a.act)} } })), ...(syncStore.config?.endpoint && !['syncing','connecting'].includes(app.webdavState) ? [{ label: '检测版本', cls: 'btn ghost', disabled: syncStore.checkingVersion || ['syncing', 'connecting'].includes(app.webdavState), run: () => { void syncStore.inspectRemoteVersion() } }] : [])] },
])
</script>

<template>
  <ModelTransfer import-only v-model:import-open="importOpen" />
  <ModelTransfer v-if="overviewTransfer" :key="overviewTransfer" action-only :remote="overviewTransfer==='webdav'" @close="overviewTransfer=null" />
  <UpdateCenter ref="updateCenter" v-model:open="updateOpen" :target="updateTarget" />
  <section class="route-section active ov" :data-mode="mode">
    <AppTopbar :subtitle-override="pageSubtitle" />

    <section class="motion-overview" data-testid="overview-motion">
      <div class="motion-fixed">
        <div class="motion-stage">
          <RuntimeMotionMark :state="motion.state" :active="motionActive" />
        </div>
        <div class="motion-identity">
          <button
            class="motion-mainline"
            type="button"
            :disabled="gateBlocked"
            :title="gateBlocked ? '当前运行状态；请先完成环境准备' : '查看运行详情（含运行环境）'"
            :aria-label="gateBlocked ? `${motion.mainline}，环境准备中` : '查看运行详情，包含运行环境'"
            @click="openRuntimeDetail"
          >
            <span>{{ motion.mainline }}</span>
            <svg v-if="!gateBlocked" viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path d="m9 5 7 7-7 7" /></svg>
          </button>
          <p class="motion-caption" :title="caption">{{ caption }}</p>
        </div>
        <div v-if="!gateBlocked" class="motion-actions">
          <UiButton
            v-if="primaryAction"
            variant="primary"
            :disabled="actionBlocked(primaryAction)"
            :loading="isLifecycleAction(primaryAction) && app.processActionBusy && app.processAction === primaryAction"
            @click="controller.runAction(primaryAction)"
          >{{ actionLabel(primaryAction) }}</UiButton>
          <UiButton
            v-for="action in secondaryActions"
            :key="action"
            :variant="action === 'stop' ? 'danger' : (action === 'logs' || action === 'advice') ? 'ghost' : 'default'"
            :disabled="app.processActionBusy || actionBlocked(action)"
            :loading="app.processActionBusy && app.processAction === action"
            @click="controller.runAction(action)"
          >{{ actionLabel(action) }}</UiButton>
          <details v-if="moreActions.length" class="motion-more">
            <summary>更多</summary>
            <div class="motion-more-menu">
              <UiButton
                v-for="action in moreActions"
                :key="action"
                type="button"
                class="btn ghost"
                :disabled="actionBlocked(action)"
                @click="controller.runAction(action)"
              >{{ actionLabel(action) }}</UiButton>
            </div>
          </details>
        </div>
      </div>
    </section>

    <EnvironmentGate v-if="gateBlocked" :on-recheck="recheckEnvironment" />

    <div v-if="!gateBlocked" class="mods">
      <UiCard v-for="mod in mods" :key="mod.key" class="mod">
        <div class="mod-head">
          <h3>{{ mod.title }}</h3>
          <span v-if="mod.tag" class="tag">{{ mod.tag }}</span>
          <button
            class="mod-go"
            type="button"
            :aria-label="`进入「${mod.title}」设置`"
            @click="routes.go('settings', { section: mod.key })"
          ><svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5.2" cy="12" r="1.7"/><circle cx="12" cy="12" r="1.7"/><circle cx="18.8" cy="12" r="1.7"/></svg></button>
        </div>
        <div class="mod-status">
          <template v-if="mod.key === 'upgrade'">
            <button
              v-for="row in versionRows"
              :key="row.key"
              class="mnt-version-row"
              type="button"
              :data-phase="row.dot"
              :disabled="row.disabled"
              :aria-label="row.aria"
              @click="openUpdates(row.key)"
            ><span>{{ row.label }}</span><span class="mnt-version-side"><span v-if="row.word" class="mnt-status" :class="{ danger: row.phase === 'failed' }">{{ row.word }}</span><b class="mnt-version-latest">{{ row.version }}</b><span v-if="row.dot" class="mnt-dot" :class="'is-' + row.dot" aria-hidden="true"></span><span aria-hidden="true">›</span></span></button>
          </template>
          <template v-else><InformationRow v-for="row in mod.rows" :key="row[0]" :label="row[0]" :value="row[1]" /></template>
          <div v-if="mod.key === 'sync'" class="info-row ov-remote-row" data-testid="overview-remote-version" :title="remoteVersionTitle">
            <span class="info-row-label">线上版本</span>
            <span class="ov-state-side">
              <span v-if="webdavView.remote" class="ov-state-dot" :class="'is-' + webdavView.remote.dot" aria-hidden="true"></span>
              <b>{{ remoteVersionTime }}</b>
              <span class="ov-state-sep" aria-hidden="true">·</span><span class="ov-state-text">{{ remoteVersionHint }}</span>
            </span>
          </div>
        </div>
        <div class="mod-actions">
          <UiButton
            v-for="action in mod.actions"
            :key="action.label"
            :variant="action.cls.includes('primary') ? 'primary' : action.cls.includes('ghost') ? 'ghost' : 'default'"
            :disabled="(mod.key === 'upgrade' && action.label === '检查更新' && app.officialProjectLoading) || ('disabled' in action && !!action.disabled)"
            @click="action.run?.()"
          >{{ action.label }}</UiButton>
        </div>
      </UiCard>
    </div>

    <UiCard v-if="!gateBlocked" class="recent-events-card" data-testid="overview-recent-events">
      <div class="recent-events-head">
        <h2>最近事件</h2>
        <UiButton size="sm" class="btn ghost" type="button" @click="routes.go('logs')">诊断中心</UiButton>
      </div>
      <ul class="recent-events-list">
        <li v-if="!recentEvents.length" class="recent-events-empty">暂无事件记录。</li>
        <li v-for="(event, index) in recentEvents" :key="`${event.time}-${index}`">
          <time>{{ event.time }}</time>
          <span>{{ event.message }}</span>
        </li>
      </ul>
    </UiCard>

    <!-- 观察窗口复用公共模态；退出不停止运行或环境检查。 -->
    <UiDialog :open="detailOpen" title="运行详情" wide panel-class="motion-runtime-modal overview-runtime-dialog" @close="detailOpen = false">
          <div class="motion-runtime-grid">
            <section aria-labelledby="runtimeProcessTitle">
              <h4 class="motion-section-title" id="runtimeProcessTitle">进程与就绪</h4>
              <dl class="motion-facts">
                <div><dt>运行状态</dt><dd>{{ motion.mainline }}</dd></div>
                <div><dt>健康</dt><dd>{{ scenario.health }}</dd></div>
                <div><dt>PID / 进程</dt><dd>{{ scenario.pid }}</dd></div>
                <div><dt>端口</dt><dd>{{ scenario.port }}</dd></div>
                <div><dt>当前操作</dt><dd>{{ motion.caption || '无' }}</dd></div>
              </dl>
            </section>
            <section class="motion-runtime-environment" aria-labelledby="runtimeEnvironmentTitle">
              <h4 class="motion-section-title" id="runtimeEnvironmentTitle">运行环境</h4>
              <ol class="motion-check-list">
                <li v-for="(check, index) in envChecks" :key="check.name">
                  <span class="motion-step">{{ index + 1 }}</span>
                  <b>{{ check.name }}</b>
                  <span class="motion-check-value" :data-tone="check.tone">{{ check.value }}</span>
                </li>
              </ol>
              <UiButton v-if="gateBlocked" class="btn ghost" type="button" @click="openEnvironment">完整环境指引</UiButton>
            </section>
          </div>
          <section class="motion-runtime-source" aria-labelledby="runtimeSourceTitle">
            <h4 class="motion-section-title" id="runtimeSourceTitle">来源与目录</h4>
            <dl class="motion-runtime-meta">
              <div><dt>版本</dt><dd><span class="motion-runtime-value" :title="version">{{ version }}</span></dd></div>
              <div><dt>运行时</dt><dd><span class="motion-runtime-value" :title="runtime">{{ runtime }}</span></dd></div>
            </dl>
            <dl class="motion-runtime-paths">
              <div>
                <dt>数据目录</dt>
                <dd><span class="motion-runtime-value local-path" :title="dataRoot">{{ dataRoot }}</span><button class="motion-value-copy" type="button" aria-label="复制数据目录" @click="copyValue(dataRoot, $event)">复制</button></dd>
              </div>
              <div>
                <dt>OPENCODEX_HOME</dt>
                <dd><span class="motion-runtime-value local-path" :title="home">{{ home }}</span><button class="motion-value-copy" type="button" aria-label="复制 OPENCODEX_HOME" @click="copyValue(home, $event)">复制</button></dd>
              </div>
            </dl>
          </section>
      <template #actions>
        <UiButton variant="ghost" type="button" @click="app.openOverviewDataRoot()">打开数据目录</UiButton>
        <UiButton variant="ghost" type="button" @click="app.openOverviewOpencodexHome()">打开 OPENCODEX_HOME</UiButton>
      </template>
    </UiDialog>

    <UiDialog :open="envOpen" title="运行环境" panel-class="overview-environment-dialog" @close="envOpen = false">
          <p class="motion-modal-sub">{{ environment.title }}</p>
          <ol class="motion-check-list">
            <li v-for="(check, index) in envChecks" :key="check.name">
              <span class="motion-step">{{ index + 1 }}</span>
              <b>{{ check.name }}</b>
              <span class="motion-check-value" :data-tone="check.tone">{{ check.value }}</span>
            </li>
          </ol>
          <template v-if="environment.state !== 'ready' && environment.state !== 'ready_via_source'">
            <h4 class="motion-section-title">{{ environment.state === 'checking' ? '检查顺序' : '下一步' }}</h4>
            <p class="motion-modal-copy">{{ environment.description }}</p>
            <div v-for="item in environment.commands" :key="item.command" class="motion-command">
              <span>{{ item.label }}</span>
              <button class="cli-copy-btn" :class="{ copied: copiedCommand === item.command }" type="button" @click="copyCommand(item.command)"><code>{{ item.command }}</code></button>
            </div>
          </template>
      <template #actions>
          <template v-if="environment.state === 'missing_ocx'">
            <UiButton class="btn primary" type="button" @click="installManaged('registry')">安装 OpenCodex</UiButton>
            <UiButton class="btn" type="button" @click="installManaged('offline')">导入离线包</UiButton>
          </template>
          <UiButton class="btn ghost" type="button" @click="guide">查看安装指引</UiButton>
          <UiButton class="btn ghost" type="button" @click="recheckEnvironment">重新检查</UiButton>
      </template>
    </UiDialog>
  </section>
</template>

<style scoped>
:global(dialog.ui-dialog.overview-runtime-dialog.wide) { width:min(760px,calc(100vw / var(--ui-zoom) - 32px)); }
:global(dialog.ui-dialog.overview-environment-dialog) { width:min(580px,calc(100vw / var(--ui-zoom) - 32px)); }

/* 原型 .mnt-version-row / .mnt-status：版本、阶段与箭头同一行垂直居中，窄卡也不拆行。 */
/* S18-09：原型 maintenance-ui.css 版本行左名右值同字号同字重400、同 --text。 */
.mnt-version-row { display: flex; align-items: center; justify-content: space-between; gap: 10px; width: 100%; min-width: 0; min-height: 18px; padding: 0; border: 0; background: transparent; color: var(--text); font: inherit; font-weight: 400; line-height: inherit; text-align: left; white-space: nowrap; cursor: pointer; }
.mnt-version-row > span:first-child { flex: none; }
.mnt-version-row:hover:not(:disabled) { color: var(--accent); }
.mnt-version-row:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; border-radius: 4px; }
.mnt-version-row:disabled { cursor: default; }
.mnt-version-side { display: inline-flex; align-items: center; justify-content: flex-end; gap: 6px; flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; color: var(--text); font-weight: 400; }
.mnt-version-latest { font-weight: 400; color: var(--text); }
.mnt-status.danger { color: var(--red); }
.mnt-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--muted); flex: none; }
.mnt-dot.is-ok { background: var(--green); }
.mnt-dot.is-info { background: var(--blue); }
.mnt-dot.is-warn { background: var(--amber); }
.mnt-dot.is-error { background: var(--red); }
.mnt-dot.is-idle { background: var(--muted); }
/* S18-09：原型 .mod-status 行 min-height/line-height 18px、无内边距，名与值同为 400、--text。 */
.mod-status :deep(.info-row) { min-height: 18px; padding: 0; border: 0; border-radius: 0; line-height: 18px; }
.mod-status :deep(.info-row-label) { color: var(--text); font-weight: 400; }
.mod-status :deep(.info-row-value) { font-weight: 400; }
/* S18-01：0.2.18 原型 index.html:768-780 线上版本状态点与方向词。 */
.ov-remote-row .ov-state-side { margin-inline-start: auto; }
.ov-remote-row b { color: var(--text); font-weight: 400; }
.ov-state-dot{width:7px;height:7px;border-radius:50%;flex:none;background:var(--muted)}
.ov-state-dot.is-ok{background:var(--green)}
.ov-state-dot.is-info{background:var(--blue)}
.ov-state-dot.is-warn{background:var(--amber)}
.ov-state-dot.is-error{background:var(--red)}
.ov-state-text{font-weight:500;color:var(--text);white-space:nowrap}
.ov-state-text.is-ok{color:var(--green)}
.ov-state-text.is-info{color:var(--blue)}
.ov-state-text.is-warn{color:var(--amber)}
.ov-state-text.is-error{color:var(--red)}
.ov-state-text.is-idle{color:var(--muted)}
.ov-state-side{display:inline-flex;align-items:center;justify-content:flex-end;gap:6px;flex:none;white-space:nowrap;font-variant-numeric:tabular-nums}
.ov-state-sep{color:var(--border)}
/* 最近事件是列表行：时间与消息同行垂直居中，消息换行时不把时间挤偏。 */
.recent-events-list li { align-items: center; }
</style>
