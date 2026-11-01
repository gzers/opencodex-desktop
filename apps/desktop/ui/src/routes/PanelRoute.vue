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
type PanelNoticeKind = 'load-failed' | 'not-running' | 'unavailable' | 'waiting' | 'browser'
interface PanelNotice { kind: PanelNoticeKind; title: string; message: string }
const NOTICES: Record<PanelNoticeKind, PanelNotice> = {
  'load-failed': { kind: 'load-failed', title: '官方面板加载失败', message: 'OpenCodex 正在运行，内嵌页面未能加载。' },
  'not-running': { kind: 'not-running', title: 'OpenCodex 未运行', message: '请从概览启动 OpenCodex 后再打开面板。' },
  unavailable: { kind: 'unavailable', title: '面板不可用', message: '请先启动 OpenCodex，或检查代理状态后重新加载。' },
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
let pendingStatusRetry = false
let statusWaitTimer: number | null = null

const embedded = computed(() => app.preferences?.panelMode !== 'browser')

// 冷启动可能先落在面板路由、状态快照还没到。此时既不该说「请先启动服务」，
// 也不该让用户手点「重试」：状态一旦变为运行就自动嵌入。
const waitingForStatus = computed(
  () =>
    !app.statusSnapshot ||
    ['loading', 'starting'].includes(app.statusSnapshot.matrix.runtime),
)

// 面板路由下的通知/确认表面必须完整可见：原生子 WebView 与主 WebView 不是同一合成层，
// 管理器 DOM 的浮层会被官方面板盖住（IMP FZ-43.4 / 原型 `.notif-host` 覆盖在面板之上）。
// 这里采用 FZ-43.4 允许的「协调显示」：存在可见表面时先隐藏面板子视图，
// 表面消失后立即恢复——面板此前已加载完成，恢复不会再重新计时（见 panelLoaded）。
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
  panelNotice.value = NOTICES[kind]
}

// 依据当前快照说明面板为何不能嵌入；不把「页面没加载」与「进程没启动」混为一谈。
function unavailableKind(): PanelNoticeKind {
  const snapshot = app.statusSnapshot
  const runtime = snapshot?.matrix.runtime
  if (runtime === 'running' && snapshot?.port != null) return 'unavailable'
  if (runtime === 'unreachable' || runtime === 'pending' || runtime === 'external_takeover') return 'unavailable'
  return 'not-running'
}

// 嵌入失败时按进程事实分流：进程仍在运行 ⇒ 页面加载失败，不改报启动失败、不重启服务。
function reportLoadFailure() {
  clearLoadTimeout()
  const running = app.statusSnapshot?.matrix.runtime === 'running'
  showNotice(running ? 'load-failed' : unavailableKind())
  // 原生子视图盖在主 WebView 之上：页面没加载出来时先收起，空态说明才可见。
  panelHiddenForFailure = true
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
    void app.loadStatusSnapshot()
  }, LIFECYCLE_FALLBACK_MS)
}

function scheduleLoadTimeout() {
  clearLoadTimeout()
  loadTimeout = window.setTimeout(() => {
    if (!panelReady.value) reportLoadFailure()
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
  if (action !== 'hide' && !embedded.value) return
  if (syncBusy.value && action !== 'hide') return
  const bounds = action === 'hide' ? undefined : getBounds()
  if (action !== 'hide' && !bounds) return
  syncBusy.value = true
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
    if (action === 'show') {
      panelHiddenForFailure = false
      if (!result.visible) {
        // 原生视图不存在且未创建：停止「正在嵌入」并按进程事实给出显式失败，不留下悬空的加载态。
        reportLoadFailure()
      } else if (app.panelLoaded) {
        // 视图被复用且本轮已成功加载过：不再等待页面事件，也不启动加载超时。
        clearLoadTimeout()
        panelNotice.value = null
        panelLoading.value = false
        panelReady.value = true
      } else {
        panelNotice.value = null
        panelLoading.value = true
        scheduleLoadTimeout()
      }
    }
  } catch {
    // 后端拒绝或建视图失败：原因已写入运行日志（commands/panel.rs），这里只按进程事实分流。
    // 布局同步失败不改变已展示的面板。
    if (action === 'show') reportLoadFailure()
  } finally {
    syncBusy.value = false
  }
}

// 「重试加载」：只重新加载内嵌页面，不发起任何启停动作。
function retryLoad() {
  if (app.statusSnapshot?.matrix.runtime !== 'running') {
    void loadPanel()
    return
  }
  panelNotice.value = null
  panelLoading.value = true
  usePanelStore().setLoaded(false)
  void nextTick(() => syncPanel('show', true))
}

function onPanelEvent(event: Event) {
  const detail = (event as CustomEvent<{ kind?: string; value?: string }>).detail
  if (!detail) return
  if (detail.kind === 'load') {
    if (detail.value === 'ready') {
      clearLoadTimeout()
      usePanelStore().setLoaded(true)
      panelReady.value = true
      panelLoading.value = false
      panelNotice.value = null
      // 超时后页面仍加载完成：把为空态收起的子视图重新显示出来。
      if (panelHiddenForFailure && !panelHiddenForSurface) void syncPanel('show')
    } else {
      // 新的页面加载开始（含重载与端口变化后的重建）：重新等待本次加载。
      usePanelStore().setLoaded(false)
      panelReady.value = false
      panelLoading.value = true
      panelNotice.value = null
    }
    return
  }
  if (detail.kind !== 'action') return
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
  await app.loadPreferences()
  panelScale.value = app.preferences?.interfaceScale ?? 100
  if (!embedded.value) {
    showNotice('browser')
    return
  }
  if (!app.statusSnapshot) await app.loadStatusSnapshot()
  if (!await app.loadPanelUrl()) {
    pendingStatusRetry = waitingForStatus.value
    // 快照未就绪时 `loadPanelUrl` 也会给出「请先启动服务」，这里要先于它判断。
    showNotice(waitingForStatus.value ? 'waiting' : unavailableKind())
    if (waitingForStatus.value) startStatusWait()
    return
  }
  pendingStatusRetry = false
  stopStatusWait()
  panelNotice.value = null
  if (!panelReady.value) panelLoading.value = true
  await nextTick()
  await syncPanel('show')
  if (requiredOverlays.value && !panelHiddenForSurface) {
    // 进入面板路由时已经有表面在显示：直接收敛，避免表面被原生面板盖住。
    panelHiddenForSurface = true
    await syncPanel('hide')
  }
}

async function openBrowserFallback() {
  if (!app.panelUrl) await app.loadPanelUrl()
  if (!app.panelUrl) {
    app.showToast(app.panelError || '官方面板地址不可用。')
    return
  }
  const opened = await app.openPanelInBrowser()
  app.showToast(opened ? '已在系统浏览器打开官方面板。' : '浏览器打开失败；已保留当前窗口。')
}

function scheduleLayout() {
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame)
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = null
    void syncPanel('layout')
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
  clearLoadTimeout()
  stopStatusWait()
  observer?.disconnect()
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame)
  window.removeEventListener('ocxd-panel', onPanelEvent)
  void syncEmbeddedPanel({ action: 'hide' })
})

watch(() => app.preferences?.interfaceScale, value => {
  if (typeof value === 'number' && value !== panelScale.value) {
    panelScale.value = value
    void syncPanel('layout')
  }
})

// 冷启动进入面板路由时快照可能还没到：状态变成可嵌入后自动补一次，不需要用户手点重试。
watch(
  () => app.statusSnapshot?.matrix.runtime,
  runtime => {
    if (!pendingStatusRetry) return
    if (runtime !== 'running') return
    pendingStatusRetry = false
    void loadPanel()
  },
)

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
        <UiButton v-if="panelNotice.kind === 'load-failed' || panelNotice.kind === 'browser'" variant="ghost" type="button" @click="void openBrowserFallback()">浏览器打开</UiButton>
      </div>
      <div v-else-if="panelLoading" class="panel-empty" role="status" aria-live="polite">
        <span>正在将官方面板嵌入主窗口…</span>
      </div>
      <p v-if="panelReady" class="panel-embedded-note">官方面板已嵌入主窗口</p>
    </div>
  </section>
</template>
