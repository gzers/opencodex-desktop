// WebDAV 连接状态展示与动作文案（原 data/mock 的同步部分）。
import type { ConnectionState } from '@/types/ui'

export const webdavStates: Record<ConnectionState, { tag: string; conn: string; conflict: string; actions: Array<{ label: string; cls: string; act: string }> }> = {
  unconfigured: { tag: '未配置', conn: '未配置', conflict: '未设置', actions: [{ label: '配置 WebDAV', cls: 'btn primary', act: 'open-sync-settings' }] },
  disconnected: { tag: '未连接', conn: '未连接', conflict: '每次询问', actions: [{ label: '立即同步', cls: 'btn primary', act: 'sync' }, { label: '测试连接', cls: 'btn ghost', act: 'test-webdav' }] },
  connecting: { tag: '连接中', conn: '正在连接', conflict: '每次询问', actions: [{ label: '取消', cls: 'btn', act: 'sync' }, { label: '测试连接', cls: 'btn ghost', act: 'test-webdav' }] },
  syncing: { tag: '同步中', conn: '同步中…', conflict: '每次询问', actions: [{ label: '取消同步', cls: 'btn', act: 'sync' }, { label: '测试连接', cls: 'btn ghost', act: 'test-webdav' }] },
  synced: { tag: '已同步', conn: '已同步', conflict: '每次询问', actions: [{ label: '立即同步', cls: 'btn primary', act: 'sync' }, { label: '测试连接', cls: 'btn ghost', act: 'test-webdav' }] },
  conflict: { tag: '冲突待处理', conn: '已同步', conflict: '待处理', actions: [{ label: '处理冲突', cls: 'btn primary', act: 'open-sync-history' }, { label: '立即同步', cls: 'btn ghost', act: 'sync' }] },
  failed: { tag: '连接失败', conn: '连接失败', conflict: '每次询问', actions: [{ label: '重试连接', cls: 'btn primary', act: 'test-webdav' }, { label: '立即同步', cls: 'btn ghost', act: 'sync' }] },
}
