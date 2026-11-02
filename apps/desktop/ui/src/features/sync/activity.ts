// S18-01：概览「文件同步 / WebDAV」卡片读取的最近导入、导出与线上版本记录。
// 只存时间与项数，保存在本机 localStorage；读写失败时按「尚未记录」处理。
import { ref } from 'vue'

export interface SyncActivityEntry { time: string; count: number }
export interface SyncActivity { import?: SyncActivityEntry; export?: SyncActivityEntry; sync?: { remoteTime: string } }

const storageKey = 'ocx-sync-activity'

function read(): SyncActivity {
  try {
    const raw = globalThis.localStorage?.getItem(storageKey)
    const value = raw ? JSON.parse(raw) as unknown : null
    return value && typeof value === 'object' ? value as SyncActivity : {}
  } catch {
    return {}
  }
}

export const syncActivity = ref<SyncActivity>(read())

function write(next: SyncActivity) {
  syncActivity.value = next
  try { globalThis.localStorage?.setItem(storageKey, JSON.stringify(next)) } catch { /* 仅影响概览展示 */ }
}

export function recordSyncActivity(kind: 'import' | 'export', count: number, time = new Date().toISOString()) {
  write({ ...syncActivity.value, [kind]: { time, count } })
}

export function recordRemoteVersion(remoteTime: string) {
  write({ ...syncActivity.value, sync: { remoteTime } })
}

/** 概览按原型只显示 HH:MM；无效时间返回空串。 */
export function clockText(value?: string) {
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '' : date.toLocaleTimeString('sv-SE').slice(0, 5)
}
