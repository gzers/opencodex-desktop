<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { syncEmbeddedPanel, type PanelBounds } from '@/features/panel/api'
import { useAppStore } from '@/stores/app'
import { usePanelStore } from '@/features/panel/store'
import { useThemeStore } from '@/app/appearance/theme'
import { clampScale } from '@/app/appearance/scale'
import { useOverlaySurfaces } from '@/app/surfaces'

const app = useAppStore()
const theme = useThemeStore()
const panel = usePanelStore()
// 覆盖表面规则集中在 app（IMP-04 §13.5）：面板只消费派生值，不逐字段判断。
const { requiredOverlays } = useOverlaySurfaces()
const panelFrame = ref<HTMLElement | null>(null)
const panelReady = ref(false)
const panelLoading = ref(true)
const panelError = ref('')
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
  }, 1500)
}

function scheduleLoadTimeout() {
  clearLoadTimeout()
  loadTimeout = window.setTimeout(() => {
    if (!panelReady.value) {
      panelLoading.value = false
      panelError.value = '官方面板加载超时；主界面仍可继续操作，请重试或在浏览器打开。'
    }
  }, 8000)
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

// 后端错误是 `{code, message}` 形状的 IPC 载荷；这里取可读文案，避免把失败原因
// 笼统说成「需要先启动服务」而误导排障。
function describePanelError(error: unknown): string {
  if (typeof error === 'string' && error) return error
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message) return message
  }
  if (error instanceof Error && error.message) return error.message
  return '未返回原因'
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
    })
    if (action === 'show') {
      panelError.value = ''
      if (!result.visible) {
        // 原生视图不存在且未创建：停止「正在嵌入」并显示显式失败，不留下悬空的加载态。
        clearLoadTimeout()
        panelLoading.value = false
        panelError.value = '官方面板未能嵌入主窗口。'
      } else if (app.panelLoaded) {
        // 视图被复用且本轮已成功加载过：不再等待页面事件，也不启动加载超时。
        clearLoadTimeout()
        panelLoading.value = false
        panelReady.value = true
      } else {
        panelLoading.value = true
        scheduleLoadTimeout()
      }
    }
  } catch (error) {
    if (action !== 'hide') {
      panelLoading.value = false
      panelError.value =
        app.panelError || `官方面板加载失败：${describePanelError(error)}。`
    }
  } finally {
    syncBusy.value = false
  }
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
      panelError.value = ''
    } else {
      // 新的页面加载开始（含重载与端口变化后的重建）：重新等待本次加载。
      usePanelStore().setLoaded(false)
      panelReady.value = false
      panelLoading.value = true
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
    panelLoading.value = false
    panelError.value = '浏览器兜底模式已启用；官方面板不会嵌入主窗口。'
    return
  }
  if (!app.statusSnapshot) await app.loadStatusSnapshot()
  if (!await app.loadPanelUrl()) {
    panelLoading.value = false
    pendingStatusRetry = waitingForStatus.value
    // 快照未就绪时 `loadPanelUrl` 也会给出「请先启动服务」，这里要先于它判断。
    panelError.value = waitingForStatus.value
      ? '正在等待 OpenCodex 状态；就绪后会自动嵌入官方面板。'
      : app.panelError || '官方面板需要 OpenCodex 处于运行且健康可用状态。'
    if (waitingForStatus.value) startStatusWait()
    return
  }
  pendingStatusRetry = false
  stopStatusWait()
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

// 协调显示：表面出现/消失时收敛面板子视图的可见性。
watch(requiredOverlays, visible => {
  if (!embedded.value) return
  if (visible && !panelHiddenForSurface) {
    panelHiddenForSurface = true
    void syncPanel('hide')
  } else if (!visible && panelHiddenForSurface) {
    panelHiddenForSurface = false
    void syncPanel('show')
  }
})
</script>

<template>
  <section class="panel-shell show">
    <div ref="panelFrame" class="panel-frame">
      <div v-if="panelLoading || panelError" class="panel-state" role="status" aria-live="polite">
        <p v-if="panelLoading">正在将官方面板嵌入主窗口…</p>
        <p v-else>{{ panelError }}</p>
        <div v-if="panelError" class="panel-state-actions">
          <button class="btn" type="button" @click="void loadPanel()">重试</button>
          <button class="btn" type="button" @click="void openBrowserFallback()">在浏览器打开</button>
        </div>
      </div>
      <p v-if="panelReady" class="panel-embedded-note">官方面板已嵌入主窗口</p>
    </div>
  </section>
</template>
