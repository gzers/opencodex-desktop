// 纯导航层（IMP-04 §13.2 / §13.7）：
// 路由名与元数据、设置分区、诊断 Tab、扩展 Tab，以及 hash 的**纯**解析/序列化与默认值归一。
// 约定：本模块不做任何副作用——不导入页面组件、不触碰 Pinia/store、不读写 window。
// 页面注册与 hash 监听属于应用层（`stores/routes.ts` 的 app-router 角色）。
import type { RouteName, SettingsSection } from '@/types/ui'

export interface RouteMeta {
  title: string
  subtitle: string
}

export const ROUTE_NAMES: readonly RouteName[] = [
  'overview',
  'panel',
  'extensions',
  'logs',
  'settings',
  'tray',
]

export const routeMetadata: Record<RouteName, RouteMeta> = {
  overview: { title: '概览', subtitle: '运行状态与常用操作；环境明细收进运行详情。' },
  panel: { title: '面板', subtitle: '窄图标栏常驻承载 OpenCodex 官方 GUI；视觉与状态由官方页面自己维护。' },
  extensions: { title: '扩展管理', subtitle: '集中发现与同步本机 Skills 和 MCP 服务器；目标客户端按图标开关。' },
  logs: { title: '诊断中心', subtitle: '环境诊断、日志历史与通知历史分 Tab 管理；日志支持清理与自动保留策略。' },
  settings: { title: '设置', subtitle: '集中管理偏好、安装配置、备份、迁移、同步、升级与官方归属。' },
  tray: { title: '托盘', subtitle: '系统托盘状态菜单的跨平台样式预览；控制动作走管理器自有域。' },
}

export const settingsSections: readonly SettingsSection[] = [
  'general',
  'installation',
  'backup',
  'extensions',
  'migration',
  'sync',
  'cleanup',
  'cli',
  'upgrade',
  'about',
]

/// 诊断中心的分区 Tab（2026-09-24）：环境诊断第一个。
export type DiagnosticsTab = 'doctor' | 'logs' | 'notifications'
export const diagnosticsTabs: readonly DiagnosticsTab[] = ['doctor', 'logs', 'notifications']

export type ExtensionTab = 'skills' | 'mcp'
export const extensionTabs: readonly ExtensionTab[] = ['skills', 'mcp']

export function isRouteName(value: unknown): value is RouteName {
  return typeof value === 'string' && (ROUTE_NAMES as readonly string[]).includes(value)
}

export function isSettingsSection(value: unknown): value is SettingsSection {
  return typeof value === 'string' && (settingsSections as readonly string[]).includes(value)
}

// 未知或缺失的取值一律回落到「环境诊断」——诊断中心的默认落地页（2026-09-28 用户要求，
// 与原型一致：进入诊断中心先看到只读的环境诊断，而不是日志历史）。
export function normalizeDiagnosticsTab(value: string | null | undefined): DiagnosticsTab {
  return (diagnosticsTabs as readonly string[]).includes(value ?? '')
    ? (value as DiagnosticsTab)
    : 'doctor'
}

export function normalizeExtensionTab(value: string | null | undefined): ExtensionTab {
  return value === 'mcp' ? 'mcp' : 'skills'
}

export interface ParsedNavigation {
  route: RouteName
  section: SettingsSection
  extensionTab: ExtensionTab
  diagnosticsTab: DiagnosticsTab
}

/** 解析 `#route?query`（允许带或不带前导 `#`）；未知取值回落到既定默认值。 */
export function parseHash(hash: string): ParsedNavigation {
  const raw = hash.replace(/^#/, '')
  const [name, query = ''] = raw.split('?')
  const params = new URLSearchParams(query)
  return {
    route: isRouteName(name) ? name : 'overview',
    section: isSettingsSection(params.get('section')) ? (params.get('section') as SettingsSection) : 'general',
    extensionTab: normalizeExtensionTab(params.get('tab')),
    diagnosticsTab: normalizeDiagnosticsTab(params.get('tab')),
  }
}

/** 序列化 `#route?query`；参数顺序保持传入顺序。 */
export function serializeHash(route: RouteName, params: Record<string, string> = {}): string {
  const query = new URLSearchParams(params).toString()
  return `#${route}${query ? `?${query}` : ''}`
}
