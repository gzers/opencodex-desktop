<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { syncEmbeddedPanel, type PanelBounds } from '@/features/panel/api'
import { useAppStore } from '@/stores/app'
import { usePanelStore } from '@/features/panel/store'
import { useThemeStore } from '@/app/appearance/theme'
import { useEffectsStore } from '@/app/appearance/effects'
import { clampScale } from '@/app/appearance/scale'
import { useOverlaySurfaces } from '@/app/surfaces'
import { useRouteStore } from '@/stores/routes'
import { LIFECYCLE_FALLBACK_MS, PANEL_LOAD_NOTICE_MS } from '@/config/runtimeDefaults'
import { isRuntimeStatusUnknown, RUNTIME_STATUS_UNKNOWN } from '@/lib/labels'

const app = useAppStore()
const theme = useThemeStore()
const effects = useEffectsStore()
const panel = usePanelStore()
const routes = useRouteStore()
// 覆盖表面规则集中在 app（IMP-04 §13.5）：面板只消费派生值，不逐字段判断。
const { requiredOverlays } = useOverlaySurfaces()
const panelFrame = ref<HTMLElement | null>(null)
const panelReady = ref(false)
const panelLoading = ref(true)

// 面板空态（原型 `.panel-empty` / prototype-panel.js S18-02）：进程状态与页面加载是两件事。
// - `load-failed`：OpenCodex 已运行、内嵌页面没加载出来——「重试加载」只重载页面，不碰进程；
// - `not-running`：OpenCodex 没在运行——「返回概览」，由概览启动；
// - `unavailable`：进程在但代理不健康/不可达（原型静态默认文案）——重新判断后再加载；
// - `waiting` / `browser`：等待首个状态快照、浏览器兜底模式，只给说明。
type PanelNoticeKind = 'load-failed' | 'not-running' | 'unavailable' | 'unknown' | 'waiting' | 'browser'
interface PanelNotice { kind: PanelNoticeKind; title: string; message: string }
const NOTICES: Record<PanelNoticeKind, PanelNotice> = {
  'load-failed': { kind: 'load-failed', title: '官方面板加载失败', message: 'OpenCodex 正在运行，内嵌页面未能加载。' },
  'not-running': { kind: 'not-running', title: 'OpenCodex 未运行', message: '请从概览启动 OpenCodex 后再打开面板。' },
  unavailable: { kind: 'unavailable', title: '面板不可用', message: '请先启动 OpenCodex，或检查代理状态后重新加载。' },
  unknown: { kind: 'unknown', title: RUNTIME_STATUS_UNKNOWN, message: '确认 OpenCodex 的运行状态后才能打开面板。' },
  waiting: { kind: 'waiting', title: '', message: '正在等待 OpenCodex 状态；就绪后会自动嵌入官方面板。' },
  browser: { kind: 'browser', title: '', message: '浏览器兜底模式已启用；官方面板不会嵌入主窗口。' },
}
const panelNotice = ref<PanelNotice | null>(null)
// 页面加载失败时为让空态可见而收起了子视图；页面随后仍加载完成时据此重新显示。
let panelHiddenForFailure = false
const panelScale = ref(100)
const syncBusy = ref(false)
let observer: ResizeObserver | null = null
let loadTimeout: number | null = null
let resizeFrame: number | null = null
let statusWaitTimer: number | null = null
let disposed = false
let loadRevision = 0
let commandRevision = 0
let nativeGeneration: number | undefined
let acceptingLoadEvents = false
type PanelNavigation = { value: string; generation: number }
let pendingNavigation: PanelNavigation | null = null
let showInFlight = false
let reloadAfterRuntimeChange = false
let currentTrace = 0
let timedOut = false
let loadInFlight = false
let timedOutBeforeShow = false
let pendingShow: { reload: boolean } | null = null

const embedded = computed(() => app.preferences?.panelMode !== 'browser')
const statusUnknown = computed(() => isRuntimeStatusUnknown(app.runtimeState, app.statusSnapshot, app.statusError))
const waitingForStatus = computed(() => !statusUnknown.value && (
  !app.statusSnapshot ||
  ['starting', 'stopping'].includes(app.runtimeState) ||
  ['loading', 'starting', 'pending', 'stopping'].includes(app.statusSnapshot.matrix.runtime)
))
const runtimeIdentity = computed(() => {
  const snapshot = app.statusSnapshot
  const port = snapshot?.port
  if (statusUnknown.value || waitingForStatus.value || snapshot?.matrix.runtime !== 'running' ||
      snapshot.facts.health === 'unhealthy' || port == null || !Number.isInteger(port) || port < 1 || port > 65535) return null
  return JSON.stringify([snapshot.pid, port])
})

// 面板路由下的通知/确认表面必须完整可见：原生子 WebView 与主 WebView 不是同一合成层，
// 管理器 DOM 的浮层会被官方面板盖住（IMP FZ-43.4 / 原型 `.notif-host` 覆盖在面板之上）。
// 这里采用 FZ-43.4 允许的「协调显示」：存在可见表面时先隐藏面板子视图，
// 表面消失后复用原生视图；Show 会报告视图是否已经加载完成。
let panelHiddenForSurface = false

function clearLoadTimeout() {
  if (loadTimeout !== null) window.clearTimeout(loadTimeout)
  loadTimeout = null
}

function stopStatusWait() {
  if (statusWaitTimer !== null) window.clearInterval(statusWaitTimer)
  statusWaitTimer = null
}

function showNotice(kind: PanelNoticeKind) {
  panelLoading.value = false
  panelReady.value = false
  panel.setLoaded(false)
  panelNotice.value = NOTICES[kind]
  if (kind !== 'waiting') {
    clearLoadTimeout()
    panel.finishLoad(currentTrace, kind)
  }
}

// 依据当前快照说明面板为何不能嵌入；不把「页面没加载」与「进程没启动」混为一谈。
function unavailableKind(): PanelNoticeKind {
  if (statusUnknown.value) return 'unknown'
  if (waitingForStatus.value) return 'waiting'
  const snapshot = app.statusSnapshot
  const runtime = snapshot?.matrix.runtime
  if (runtime === 'running' || runtime === 'unreachable' || runtime === 'external_takeover') return 'unavailable'
  return 'not-running'
}

// 嵌入失败时按进程事实分流：进程仍在运行 ⇒ 页面加载失败，不改报启动失败、不重启服务。
function reportLoadFailure() {
  clearLoadTimeout()
  showNotice(runtimeIdentity.value ? 'load-failed' : unavailableKind())
  // 原生子视图盖在主 WebView 之上：页面没加载出来时先收起，空态说明才可见。
  panelHiddenForFailure = true
  panel.finishLoad(currentTrace, timedOut ? 'timeout' : 'failed')
  void syncEmbeddedPanel({ action: 'hide' }).catch(() => {})
}

// 状态事件流未就绪时快照可能一直为空：按节奏主动补取，拿到运行态后由 watch 自动嵌入，
// 否则会永远停在「正在等待 OpenCodex 状态」。
function startStatusWait() {
  if (statusWaitTimer !== null) return
  statusWaitTimer = window.setInterval(() => {
    if (!waitingForStatus.value) {
      stopStatusWait()
      return
    }
    if (!app.statusLoading) void app.loadStatusSnapshot()
  }, LIFECYCLE_FALLBACK_MS)
}

function scheduleLoadTimeout() {
  clearLoadTimeout()
  loadTimeout = window.setTimeout(() => {
    if (!panelReady.value) {
      timedOut = true
      timedOutBeforeShow = syncBusy.value || loadInFlight
      loadRevision++
      commandRevision++
      syncBusy.value = false
      loadInFlight = false
      showInFlight = false
      pendingNavigation = null
      stopStatusWait()
      reportLoadFailure()
    }
  }, PANEL_LOAD_NOTICE_MS)
}

function getBounds(): PanelBounds | null {
  const element = panelFrame.value
  if (!element) return null
  const rect = element.getBoundingClientRect()
  if (rect.width < 1 || rect.height < 1) return null
  // 主壳带 CSS `zoom`，这里拿到的是缩放后的窗口坐标（侧栏渲染宽度 = 64 * 缩放）。
  // 原生子视图也按同一坐标系摆放，故直接上报；边界校验按 `scale` 在
  // `commands/panel.rs` 里折算，两处不重复缩放。
  return { x: rect.left, y: rect.top, width: rect.width, height: rect.height }
}

async function syncPanel(action: 'show' | 'layout' | 'hide', reload = false) {
  const identity = runtimeIdentity.value
  if (action !== 'hide' && (disposed || timedOut || !embedded.value || !identity)) return
  if (syncBusy.value && action !== 'hide') {
    // Hide can still be returning when the last overlay closes. Preserve the
    // latest desired Show rather than losing the only restoration request.
    if (action === 'show') pendingShow = { reload: reload || !!pendingShow?.reload }
    return
  }
  const bounds = action === 'hide' ? undefined : getBounds()
  if (action !== 'hide' && !bounds) return
  const expectedUrl = action === 'show'
    ? new URL(`http://127.0.0.1:${app.statusSnapshot?.port}/web`).href
    : null
  const revision = ++commandRevision
  if (action === 'show') {
    reload ||= reloadAfterRuntimeChange
    pendingShow = null
    showInFlight = true
    pendingNavigation = null
  } else if (action === 'hide') {
    showInFlight = false
    pendingNavigation = null
  }
  syncBusy.value = true
  if (action === 'show') panel.markLoad(currentTrace, 'native-request')
  try {
    const result = await syncEmbeddedPanel({
      action,
      bounds: bounds ?? undefined,
      reload,
      scale: panelScale.value,
      // `theme` 只驱动管理器注入浮层的主题；`themeSetting` 按官方面板自己的
      // `ocx-theme` 存储键同步明暗，两者都不改写官方页面样式。
      theme: theme.resolved === 'dark' ? 'dark' : 'light',
      themeSetting: theme.setting,
      effects: effects.setting,
    })
    if (disposed || revision !== commandRevision || timedOut ||
        (action !== 'hide' && (identity !== runtimeIdentity.value || !embedded.value))) return
    if (action === 'show') {
      showInFlight = false
      const navigation = pendingNavigation as PanelNavigation | null
      pendingNavigation = null
      nativeGeneration = result.generation
      const ownsView = result.visible && result.panelUrl === expectedUrl && nativeGeneration !== undefined
      acceptingLoadEvents = ownsView
      const observedReady = navigation?.value === 'ready' && navigation.generation === nativeGeneration
      for (const phase of result.timings ?? []) panel.markLoad(currentTrace, 'native:' + phase.phase + ':' + phase.elapsedMs.toFixed(1))
      panelHiddenForFailure = false
      if (!ownsView) {
        // 原生视图不存在且未创建：停止「正在嵌入」并按进程事实给出显式失败，不留下悬空的加载态。
        reportLoadFailure()
      } else if (observedReady || result.ready) {
        // 视图被复用且本轮已成功加载过：不再等待页面事件，也不启动加载超时。
        clearLoadTimeout()
        panelNotice.value = null
        panelLoading.value = false
        panelReady.value = true
        panel.setLoaded(true)
        panel.finishLoad(currentTrace, 'visible-ready')
      } else {
        panelNotice.value = null
        panelLoading.value = true
        if (loadTimeout === null) scheduleLoadTimeout()
      }
      if (ownsView) reloadAfterRuntimeChange = false
      if (requiredOverlays.value) {
        panelHiddenForSurface = true
        await syncPanel('hide')
      }
    }
  } catch {
    // 后端拒绝或建视图失败：原因已写入运行日志（commands/panel.rs），这里只按进程事实分流。
    // 布局同步失败不改变已展示的面板。
    if (action === 'show' && !disposed && revision === commandRevision && !timedOut) reportLoadFailure()
  } finally {
    if (revision === commandRevision) {
      if (action === 'show') showInFlight = false
      syncBusy.value = false
      if (pendingShow && !disposed && !timedOut && !requiredOverlays.value && !panelNotice.value) {
        const pending = pendingShow
        pendingShow = null
        void syncPanel('show', pending.reload)
      }
    }
  }
}

// 「重试加载」：只重新加载内嵌页面，不发起任何启停动作。
function retryLoad() {
  if (!runtimeIdentity.value) {
    void loadPanel()
    return
  }
  panelNotice.value = null
  panelLoading.value = true
  timedOut = false
  timedOutBeforeShow = false
  acceptingLoadEvents = false
  commandRevision++
  syncBusy.value = false
  currentTrace = panel.beginLoad()
  scheduleLoadTimeout()
  usePanelStore().setLoaded(false)
  void nextTick(() => syncPanel('show', true))
}

function onPanelEvent(event: Event) {
  const detail = (event as CustomEvent<{ kind?: string; value?: string; generation?: number }>).detail
  if (!detail || disposed || !embedded.value || !runtimeIdentity.value) return
  if (detail.kind === 'load') {
    if (typeof detail.generation !== 'number' || (detail.value !== 'ready' && detail.value !== 'loading')) return
    if (showInFlight) {
      if (detail.generation >= (pendingNavigation?.generation ?? 0)) {
        pendingNavigation = { value: detail.value, generation: detail.generation }
      }
      return
    }
    if (!acceptingLoadEvents || (timedOut && timedOutBeforeShow) || detail.generation !== nativeGeneration) return
    if (timedOut && detail.value === 'ready') timedOut = false
    panel.markLoad(currentTrace, detail.value === 'ready' ? 'navigation-ready' : 'navigation-start')
    if (detail.value === 'ready') {
      clearLoadTimeout()
      usePanelStore().setLoaded(true)
      panelReady.value = true
      panelLoading.value = false
      panelNotice.value = null
      panel.finishLoad(currentTrace, 'visible-ready')
      // 超时后页面仍加载完成：把为空态收起的子视图重新显示出来。
      if (panelHiddenForFailure && !panelHiddenForSurface) void syncPanel('show')
    } else {
      // 新的页面加载开始（含重载与端口变化后的重建）：重新等待本次加载。
      usePanelStore().setLoaded(false)
      panelReady.value = false
      panelLoading.value = true
      panelNotice.value = null
      if (loadTimeout === null) scheduleLoadTimeout()
    }
    return
  }
  if (detail.kind !== 'action' || !panelReady.value || !acceptingLoadEvents || showInFlight || requiredOverlays.value) return
  switch (detail.value) {
    case 'reload':
      void syncPanel('show', true)
      break
    case 'browser':
      void openBrowserFallback()
      break
    case 'zoom-in':
      setScale(panelScale.value + 10)
      break
    case 'zoom-out':
      setScale(panelScale.value - 10)
      break
  }
}

function setScale(next: number) {
  // 面板 hub 与设置页共用同一个「界面缩放」偏好：外壳与内嵌面板始终同一因子，
  // 不会出现两套比例，也不会重复叠加缩放。
  const scale = clampScale(next)
  panelScale.value = scale
  void syncPanel('layout')
  void app.setInterfaceScale(scale)
}

async function loadPanel() {
  if (disposed || loadInFlight) return
  loadInFlight = true
  const revision = ++loadRevision
  timedOut = false
  timedOutBeforeShow = false
  currentTrace = panel.beginLoad()
  panelNotice.value = null
  panelLoading.value = true
  scheduleLoadTimeout()
  try {
    if (statusUnknown.value) {
      showNotice('unknown')
      return
    }
    panel.markLoad(currentTrace, 'preferences-start')
    // Startup already loaded these preferences. Bound the wait if they are missing.
    if (!app.preferences) await app.loadPreferences()
    if (disposed || revision !== loadRevision) return
    panel.markLoad(currentTrace, 'preferences-ready')
    panelScale.value = app.preferences?.interfaceScale ?? 100
    if (!embedded.value) {
      showNotice('browser')
      return
    }
    if (!runtimeIdentity.value) {
      clearLoadTimeout()
      showNotice(unavailableKind())
      if (waitingForStatus.value) {
        startStatusWait()
        if (!app.statusSnapshot && !app.statusLoading) void app.loadStatusSnapshot()
      }
      return
    }
    panel.markLoad(currentTrace, 'status-ready')
    const hasAddress = await app.loadPanelUrl()
    if (disposed || revision !== loadRevision) return
    if (!hasAddress) {
      showNotice(unavailableKind())
      return
    }
    panel.markLoad(currentTrace, 'address-ready')
    stopStatusWait()
    panelNotice.value = null
    if (!panelReady.value) panelLoading.value = true
    await nextTick()
    if (disposed || revision !== loadRevision || timedOut) return
    await syncPanel('show')
    if (disposed || revision !== loadRevision) return
    if (requiredOverlays.value && !panelHiddenForSurface) {
      // Keep the native view below any surface already open on entry.
      panelHiddenForSurface = true
      await syncPanel('hide')
    }
  } catch {
    if (!disposed && revision === loadRevision) reportLoadFailure()
  } finally {
    if (revision === loadRevision) loadInFlight = false
  }
}

async function openBrowserFallback() {
  const identity = runtimeIdentity.value
  const revision = loadRevision
  if (!identity || disposed) return
  const opened = await app.openPanelInBrowser()
  if (disposed || revision !== loadRevision || identity !== runtimeIdentity.value) return
  app.showToast(opened ? '已在系统浏览器打开官方面板。' : '浏览器打开失败；已保留当前窗口。')
}

function scheduleLayout() {
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame)
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = null
    // 首次挂载可能尚无可用尺寸，Show 会在 getBounds 处延后；尺寸就绪后补 Show，
    // 不能仅发 Layout（原生视图尚未创建时 Layout 不会创建它）。失败空态仍由用户重试。
    const awaitingView = panelLoading.value && !panelNotice.value && !!app.panelUrl
    void syncPanel(awaitingView ? 'show' : 'layout')
  })
}

onMounted(() => {
  window.addEventListener('ocxd-panel', onPanelEvent)
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(scheduleLayout)
    if (panelFrame.value) observer.observe(panelFrame.value)
  }
  void loadPanel()
})

// 顶栏「刷新状态」在面板路由下的第二层动作：重载内嵌的官方面板（IMP-06）。
watch(() => panel.reloadToken, () => {
  void loadPanel()
})

onBeforeUnmount(() => {
  disposed = true
  pendingShow = null
  loadRevision++
  commandRevision++
  clearLoadTimeout()
  stopStatusWait()
  observer?.disconnect()
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame)
  window.removeEventListener('ocxd-panel', onPanelEvent)
  void syncEmbeddedPanel({ action: 'hide' }).catch(() => {})
})

watch(() => app.preferences?.interfaceScale, value => {
  if (typeof value === 'number' && value !== panelScale.value) {
    panelScale.value = value
    void syncPanel('layout')
  }
})

// A fresh runtime identity owns its own Show and navigation callbacks.
const unavailableNotice = computed(() => runtimeIdentity.value ? null : unavailableKind())
watch([runtimeIdentity, unavailableNotice, embedded], ([identity], [previousIdentity, , wasEmbedded]) => {
  loadRevision++
  commandRevision++
  loadInFlight = false
  syncBusy.value = false
  showInFlight = false
  pendingShow = null
  pendingNavigation = null
  acceptingLoadEvents = false
  nativeGeneration = undefined
  panelHiddenForFailure = false
  timedOut = false
  timedOutBeforeShow = false
  clearLoadTimeout()
  stopStatusWait()
  panelReady.value = false
  panel.setLoaded(false)
  panel.url = ''
  if (previousIdentity && identity !== previousIdentity) reloadAfterRuntimeChange = true
  if (previousIdentity && wasEmbedded) void syncPanel('hide')
  void loadPanel()
})

// 主题变化分两种：
// - 管理器的明暗**设置**变了：面板要按新的 `ocx-theme` 重新进入首屏（官方页面只在首屏前读这个键，
//   运行时改属性会被它自己的状态覆盖回去），同时把新的浮层配色一并下发。
// - 只有解析结果变了（设置是「跟随系统」、系统外观翻转）：面板自己就跟随系统，只需同步浮层配色。
watch(
  () => [theme.setting, theme.resolved] as const,
  ([setting], [previousSetting]) => {
    if (!panelReady.value) return
    if (setting !== previousSetting) void syncPanel('show', true)
    else void syncPanel('layout')
  },
)
watch(() => effects.setting, () => { if (panelReady.value) void syncPanel('layout') })

// 协调显示：表面出现/消失时收敛面板子视图的可见性。
watch(requiredOverlays, visible => {
  if (!embedded.value) return
  if (visible && !panelHiddenForSurface) {
    panelHiddenForSurface = true
    void syncPanel('hide')
  } else if (!visible && panelHiddenForSurface) {
    panelHiddenForSurface = false
    // 页面加载失败时子视图本就收起：表面消失后保持空态，不重新露出未加载的视图。
    if (!panelNotice.value) void syncPanel('show')
  }
})
</script>

<template>
  <section class="panel-shell show">
    <div ref="panelFrame" class="panel-frame">
      <div v-if="panelNotice" class="panel-empty" role="status" aria-live="polite">
        <strong v-if="panelNotice.title">{{ panelNotice.title }}</strong>
        <span>{{ panelNotice.message }}</span>
        <UiButton v-if="panelNotice.kind === 'load-failed'" type="button" @click="retryLoad()">重试加载</UiButton>
        <UiButton v-else-if="panelNotice.kind === 'unavailable'" type="button" @click="void loadPanel()">重试加载</UiButton>
        <UiButton v-else-if="panelNotice.kind === 'not-running'" type="button" @click="routes.go('overview')">返回概览</UiButton>
        <UiButton v-else-if="panelNotice.kind === 'unknown'" type="button" @click="routes.go('logs', { tab: 'doctor' })">查看诊断</UiButton>
        <UiButton v-if="runtimeIdentity && (panelNotice.kind === 'load-failed' || panelNotice.kind === 'browser')" variant="ghost" type="button" @click="void openBrowserFallback()">浏览器打开</UiButton>
      </div>
      <div v-else-if="panelLoading" class="panel-empty" role="status" aria-live="polite">
        <span>正在将官方面板嵌入主窗口…</span>
      </div>
      <p v-if="panelReady" class="panel-embedded-note">官方面板已嵌入主窗口</p>
    </div>
  </section>
</template>
