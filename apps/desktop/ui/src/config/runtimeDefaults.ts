// 版本固化运行策略的前端只读子集（H-24/H-25/H-26）。
//
// 单一事实源是 `apps/desktop/tauri/config/runtime.defaults.json`；首屏等待与提示窗口
// 必须在后端偏好返回前就可用，因此在构建期直接由 Vite 内联该文件的 ui.* 子集。
// 只暴露允许前端读取的等待/提示窗口，不泄漏机器路径、凭据或后端策略。
import defaults from '../../../tauri/config/runtime.defaults.json'

const ui = defaults.ui as {
  startup_snapshot_wait_ms: number
  startup_preferences_wait_ms: number
  lifecycle_fallback_ms: number
  lifecycle_notice_stop_ms: number
  lifecycle_notice_start_ms: number
  lifecycle_late_observe_ms: number
  panel_load_notice_ms: number
  tray_poll_ms: number
}

export const STARTUP_SNAPSHOT_WAIT_MS = ui.startup_snapshot_wait_ms
export const STARTUP_PREFERENCES_WAIT_MS = ui.startup_preferences_wait_ms
export const LIFECYCLE_FALLBACK_MS = ui.lifecycle_fallback_ms
export const LIFECYCLE_NOTICE_STOP_MS = ui.lifecycle_notice_stop_ms
export const LIFECYCLE_NOTICE_START_MS = ui.lifecycle_notice_start_ms
export const LIFECYCLE_LATE_OBSERVE_MS = ui.lifecycle_late_observe_ms
export const PANEL_LOAD_NOTICE_MS = ui.panel_load_notice_ms
export const TRAY_POLL_MS = ui.tray_poll_ms

