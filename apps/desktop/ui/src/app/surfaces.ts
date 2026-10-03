// 覆盖表面（overlay surfaces）协调接口（IMP-04 §13.5）。
//
// 原生子 WebView 与主 WebView 不是同一合成层：管理器 DOM 的浮层（通知中心 / Toast / 任务卡 / 模态）
// 会被官方子面板盖住。这里把「是否存在必须在原生面板之上显示的表面」集中成**一个** UI 状态来源，
// 供面板路由决定是否临时隐藏子 WebView；规则只表达 UI 状态，不反向引用 notifications/runtime 的 store。
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'

export function useOverlaySurfaces() {
  const app = useAppStore()
  // 多个表面重叠时，只要还有任一必需表面可见，就继续让面板避让；
  // 全部关闭后（派生值自然回落为 false）才恢复子 WebView——不逐表面计数，避免关早/关晚。
  const requiredOverlays = computed(
    () =>
      app.notificationPanelOpen ||
      app.toast !== '' ||
      app.processProgressOpen ||
      app.modal !== null,
  )
  return { requiredOverlays }
}
