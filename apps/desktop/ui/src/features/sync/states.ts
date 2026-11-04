// WebDAV 连接状态展示与动作文案，对齐 0.2.18 原型 index.html webdavStates。
// 未配置 / 未连接 / 连接中 / 连接失败：线上版本只写「—」，不画点、不写方向词。
// 原型的「有更新」需要先读取云端快照才能判断，概览不主动联网，见 0.2.19-bugs S18-01 偏差。
import type { WebdavState } from './observation'

export type WebdavAction = 'sync' | 'connection' | 'result'
export interface WebdavStateView {
  tag: string
  conn: string
  remote: { dot: 'ok' | 'info' | 'warn' | 'error' | 'idle'; text: string } | null
  actions: Array<{ label: string; cls: string; act: WebdavAction; disabled?: boolean }>
}

export const webdavStates: Record<WebdavState, WebdavStateView> = {
  unconfigured: { tag: '未配置', conn: '未配置', remote: null, actions: [{ label: '配置 WebDAV', cls: 'btn primary', act: 'connection' }] },
  unknown: { tag: '状态未知', conn: '连接配置读取失败', remote: null, actions: [{ label: '连接设置', cls: 'btn primary', act: 'connection' }] },
  configured: { tag: '已配置', conn: '已配置 · 未检测', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  disconnected: { tag: '未检测', conn: '已配置 · 未检测', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  connected: { tag: '检测成功', conn: '最近检测成功', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  connecting: { tag: '连接中', conn: '正在连接', remote: null, actions: [{ label: '同步', cls: 'btn primary', act: 'sync', disabled: true }] },
  syncing: { tag: '同步中', conn: '已连接', remote: { dot: 'info', text: '同步中' }, actions: [{ label: '同步中…', cls: 'btn primary', act: 'sync', disabled: true }] },
  synced: { tag: '已同步', conn: '最近同步成功', remote: { dot: 'idle', text: '上次已一致' }, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }] },
  conflict: { tag: '部分完成', conn: '已连接', remote: { dot: 'warn', text: '未完成' }, actions: [{ label: '同步', cls: 'btn primary', act: 'sync' }, { label: '查看结果', cls: 'btn ghost', act: 'result' }] },
  failed: { tag: '检测失败', conn: '最近检测失败', remote: null, actions: [{ label: '连接设置', cls: 'btn primary', act: 'connection' }] },
}
