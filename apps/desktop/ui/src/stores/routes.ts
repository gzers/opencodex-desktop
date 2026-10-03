import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import type { RouteName, SettingsSection } from '@/types/ui'
import {
  normalizeDiagnosticsTab,
  parseHash,
  routeMetadata,
  serializeHash,
  type DiagnosticsTab,
} from '@/navigation'
import OverviewRoute from '@/routes/OverviewRoute.vue'
import PanelRoute from '@/routes/PanelRoute.vue'
import ExtensionsRoute from '@/routes/ExtensionsRoute.vue'
import LogsRoute from '@/routes/LogsRoute.vue'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import TrayRoute from '@/routes/TrayRoute.vue'

export const useRouteStore = defineStore('routes', () => {
  const route = ref<RouteName>(readHash().route)
  const settingsSection = ref<SettingsSection>(readHash().section)
  const extensionTab = ref(readHash().extensionTab)
  const diagnosticsTab = ref<DiagnosticsTab>(readHash().diagnosticsTab)
  const meta = computed(() => routeMetadata[route.value])
  // 收起通知表面时触发铃铛脉冲；只做视觉提示，不改动任何通知状态。
  const bellPulse = ref(0)

  function requestNotificationAttention() {
    bellPulse.value += 1
  }

  const component = computed(() => {
    switch (route.value) {
      case 'overview': return OverviewRoute
      case 'panel': return PanelRoute
      case 'extensions': return ExtensionsRoute
      case 'logs': return LogsRoute
      case 'settings': return SettingsRoute
      case 'tray': return TrayRoute
    }
  })

  function readHash() {
    return parseHash(window.location.hash)
  }

  function go(next: RouteName, params: Record<string, string> = {}) {
    route.value = next
    if (next === 'settings' && params.section) settingsSection.value = params.section as SettingsSection
    if (next === 'extensions' && params.tab) extensionTab.value = params.tab === 'mcp' ? 'mcp' : 'skills'
    if (next === 'logs' && params.tab) diagnosticsTab.value = normalizeDiagnosticsTab(params.tab)
    const hash = serializeHash(next, params)
    if (window.location.hash !== hash) window.location.hash = hash
  }

  function updateFromLocation() {
    const parsed = readHash()
    route.value = parsed.route
    settingsSection.value = parsed.section
    extensionTab.value = parsed.extensionTab
    diagnosticsTab.value = parsed.diagnosticsTab
  }

  window.addEventListener('hashchange', updateFromLocation)

  // 路由名投影到根元素（与原型 `html[data-route=...]` 同源、单一写入点）：
  // 页面级样式（如概览的内距与区块间距）据此按路由生效，不新增第二套状态。
  function syncRouteAttribute() {
    if (typeof document === 'undefined') return
    document.documentElement.setAttribute('data-route', route.value)
  }
  watch(route, syncRouteAttribute)
  syncRouteAttribute()

  return {
    current: route,
    settingsSection,
    extensionTab,
    diagnosticsTab,
    meta,
    component,
    bellPulse,
    requestNotificationAttention,
    go,
    updateFromLocation,
  }
})
