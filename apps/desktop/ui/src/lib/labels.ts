// 统一的中文文案映射：运行时/健康/连接/操作/运行时来源。
//
// 官方 status 与后端 DTO 使用英文枚举（`running`、`healthy`、`bundled`…）。
// 界面**不得**直接渲染这些原始值；所有展示都必须经过本模块，保证中文一致、
// 且新增枚举值不会悄悄漏成英文。

import type {
  ConnectionState,
  HealthState,
  OperationState,
  RuntimeState,
} from '@/contracts/runtimeStatus'

export const RUNTIME_LABELS: Record<RuntimeState, string> = {
  loading: '加载中',
  not_found: '未发现安装',
  stopped: '未运行',
  starting: '启动中',
  pending: '等待就绪',
  running: '运行中',
  stopping: '停止中',
  starting_failed: '启动失败',
  at_risk: '存在风险',
  external_takeover: '外部接管',
  unreachable: '不可达',
}

export const HEALTH_LABELS: Record<HealthState, string> = {
  healthy: '正常',
  degraded: '降级',
  unhealthy: '异常',
  unknown: '未知',
}

export const CONNECTION_LABELS: Record<ConnectionState, string> = {
  conflict: '冲突待处理',
  syncing: '同步中',
  connecting: '连接中',
  failed: '连接失败',
  disconnected: '未连接',
  unconfigured: '未配置',
  synced: '已同步',
}

export const OPERATION_LABELS: Record<OperationState, string> = {
  rolling_back: '回滚中',
  applying: '应用中',
  backing_up: '备份中',
  validating: '校验中',
  failed: '失败',
  succeeded: '已完成',
  cancelled: '已取消',
  idle: '空闲',
}

/// 官方运行时来源（`runtime.source`）。未知值回退 `未知`，绝不展示原始英文。
const RUNTIME_SOURCE_LABELS: Record<string, string> = {
  bundled: '内置',
  system: '系统',
  global: '全局',
  local: '本地',
  npx: 'npx',
  managed: '受管',
}

function lookup(map: Record<string, string>, value: unknown, fallback: string): string {
  if (typeof value === 'string' && map[value]) return map[value]
  return fallback
}

export function runtimeLabel(value: unknown, fallback = '未知'): string {
  return lookup(RUNTIME_LABELS as unknown as Record<string, string>, value, fallback)
}

// 托盘专用标签（2026-09-24 用户决策）：**托盘只讲进程事实，不写风险结论**。
// `at_risk` 在应用内是「存在风险」，托盘这一层如实显示「未运行」；风险结论与成因
// 留在概览 / 通知中心。与后端 `modules/tray::runtime_label` 保持一致，避免两处漂移。
export const TRAY_RUNTIME_LABELS: Record<string, string> = {
  at_risk: '未运行',
}

export function trayRuntimeLabel(value: unknown, fallback = '未知'): string {
  const tray = TRAY_RUNTIME_LABELS[String(value)]
  return tray ?? runtimeLabel(value, fallback)
}

export function healthLabel(value: unknown, fallback = '未知'): string {
  return lookup(HEALTH_LABELS as unknown as Record<string, string>, value, fallback)
}

export function connectionLabel(value: unknown, fallback = '未知'): string {
  return lookup(CONNECTION_LABELS as unknown as Record<string, string>, value, fallback)
}

export function operationLabel(value: unknown, fallback = '未知'): string {
  return lookup(OPERATION_LABELS as unknown as Record<string, string>, value, fallback)
}

export function runtimeSourceLabel(value: unknown, fallback = '未知'): string {
  return lookup(RUNTIME_SOURCE_LABELS, value, fallback)
}
