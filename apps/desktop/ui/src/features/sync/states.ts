// WebDAV 连接状态展示与动作文案，对齐 0.2.18 原型 index.html webdavStates。
// 未配置 / 未连接 / 连接中 / 连接失败：线上版本只写「—」，不画点、不写方向词。
// 原型的「有更新」需要先读取云端快照才能判断，概览不主动联网，见 0.2.18-bugs S18-01 偏差。
import type { ConnectionState } from '@/types/ui'

export type WebdavAction = 'sync' | 'connection'
export interface WebdavStateView {
  tag: string
  conn: string
  remote: { dot: 'ok' | 'info' | 'warn' | 'error' | 'idle'; text: string } | null
  actions: Array<{ label: string; cls: string; act: WebdavAction; disabled?: boolean }>
}

export const webdavStates: Record<ConnectionState, WebdavStateView> = {
  unconfigured: { tag: '未配置', conn: '未配置', remote: null, actions: [{ label: '配置 WebDAV', cls: 'btn primary', act: 'connection' }] },
  disconnected: { tag: '未连接', conn: '未连接', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  connecting: { tag: '连接中', conn: '正在连接', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync', disabled: true }] },
  syncing: { tag: '同步中', conn: '已连接', remote: { dot: 'info', text: '同步中' }, actions: [{ label: '同步中…', cls: 'btn primary', act: 'sync', disabled: true }] },
  synced: { tag: '已同步', conn: '已连接', remote: { dot: 'idle', text: '已一致' }, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  conflict: { tag: '部分完成', conn: '已连接', remote: { dot: 'warn', text: '未完成' }, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }, { label: '查看结果', cls: 'btn ghost', act: 'sync' }] },
  failed: { tag: '连接失败', conn: '连接失败', remote: null, actions: [{ label: '重试连接', cls: 'btn primary', act: 'connection' }] },
}
