import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { useAppStore } from '@/stores/app'

describe('workspace settings contracts', () => {
  beforeEach(() => {
    invoke.mockReset()
    setActivePinia(createPinia())
  })

  it('loads managed and agent path targets', async () => {
    const targets = {
      managed: [{ key: 'logs', path: '/data/logs', label: '日志目录' }],
      agent: [{ kind: 'skills', client: 'codex', label: 'Codex Skills', path: '/home/.codex/skills' }],
    }
    invoke.mockResolvedValueOnce(targets)
    const app = useAppStore()
    await expect(app.loadManagedPathTargets()).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('get_managed_path_targets')
    expect(app.managedPathTargets).toEqual(targets.managed)
    expect(app.agentPathTargets).toEqual(targets.agent)
  })

  it('opens only backend-confirmed paths and documents', async () => {
    invoke.mockResolvedValueOnce({ opened: true })
    invoke.mockResolvedValueOnce({ opened: false })
    const app = useAppStore()
    await expect(app.openManagedPath('/data/logs')).resolves.toBe(true)
    await expect(app.openLocalDocument('license')).resolves.toBe(false)
    expect(invoke).toHaveBeenNthCalledWith(1, 'open_managed_path', { path: '/data/logs' })
    expect(invoke).toHaveBeenNthCalledWith(2, 'open_local_document', { documentId: 'license' })
  })

  it('runs sync once and projects status without losing local content on failure', async () => {
    const app = useAppStore()
    invoke.mockRejectedValueOnce(new Error('offline'))
    await expect(app.runSyncNow()).resolves.toBe(false)
    expect(app.syncRunning).toBe(false)
    expect(app.syncStatus?.operationState).toBe('failed')
    expect(app.webdavState).toBe('failed')
    expect(invoke).toHaveBeenCalledWith('run_sync_now')
  })

  it('clears local logs and notifications through frozen commands', async () => {
    invoke.mockResolvedValueOnce({ cleanedLogs: 2, summary: 'ok' })
    invoke.mockResolvedValueOnce({ items: [], aggregate: { total: 0, unread: 0, dangerUnread: 0 } })
    const app = useAppStore()
    await expect(app.cleanupLocalLogs()).resolves.toBe(true)
    await expect(app.cleanupNotifications()).resolves.toBe(true)
    expect(app.notifications).toEqual([])
    expect(invoke).toHaveBeenNthCalledWith(1, 'cleanup_local_logs')
    expect(invoke).toHaveBeenNthCalledWith(2, 'cleanup_local_notifications')
  })
})
