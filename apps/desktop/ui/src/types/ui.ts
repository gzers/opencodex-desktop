export type {
  ConnectionState,
  HealthState,
  OperationState,
  RuntimeState,
  StatusSource,
} from '@/contracts/runtimeStatus'

export type ThemeSetting = 'light' | 'dark' | 'system'

export type RouteName = 'overview' | 'panel' | 'extensions' | 'logs' | 'settings' | 'tray'

export type SettingsSection =
  | 'general'
  | 'installation'
  | 'backup'
  | 'extensions'
  | 'migration'
  | 'sync'
  | 'cleanup'
  | 'cli'
  | 'upgrade'
  | 'about'

export type ExtensionTab = 'skills' | 'mcp'

export type TrayOs = 'mac' | 'windows'

export type PanelAction = 'show' | 'layout' | 'hide'

import type { RuntimeState } from '@/contracts/runtimeStatus'

export type LifecycleAction = 'start' | 'stop' | 'restart'
export type LifecycleResult = 'started' | 'stopped' | 'cancelled' | 'failed'
export type ProcessLifecycleState =
  | 'stopped'
  | 'starting'
  | 'pending'
  | 'running'
  | 'stopping'
  | 'starting_failed'
  | 'unreachable'

export interface NotificationItem {
  id: string
  kind: 'info' | 'warning' | 'danger'
  category: 'run' | 'sync' | 'update' | 'system'
  title: string
  detail: string
  time: string
  read: boolean
  // 触发来源与可过期条件：领域模型 §16 的独立维度；后端始终下发，旧夹具可省略。
  source?: 'user_action' | 'runtime' | 'sync' | 'update' | 'diagnostic'
  expiresAt?: string | null
  // read / resolved 是互相独立的维度：已读不等于已解决；后端始终下发，旧夹具可省略。
  resolved?: boolean
  resolvedAt?: string | null
  operationId?: string | null
  dedupeKey?: string | null
  // 后端 `NotificationAction` 冻结为 snake_case（见 `types/notifications.rs` 的 DTO 契约测试），
  // 前端必须用同一套取值，否则多词动作（app_update / settings_installation / settings_cleanup）
  // 查不到对应按钮与跳转——「通知详情没有前往处理的按钮」。
  action?: 'restore' | 'upgrade' | 'app_update' | 'sync' | 'logs' | 'settings_installation' | 'settings_cleanup'
  target?: RuntimeState
}

export interface RecentEvent {
  time: string
  message: string
}
