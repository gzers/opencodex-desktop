import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { useAppStore } from '@/stores/app'

describe('tray navigation bridge contracts', () => {
  beforeEach(() => {
    invoke.mockReset()
    setActivePinia(createPinia())
    window.location.hash = '#overview'
  })

  it('serializes the frozen tray action domain without lifecycle wrapper payloads', async () => {
    invoke.mockResolvedValueOnce([])
    const { drainTrayRequests } = await import('@/features/tray/api')
    await drainTrayRequests()
    expect(invoke).toHaveBeenCalledWith('drain_tray_requests')
  })

  it('opens backend-confirmed manager state directory', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce({
      managed: [{ key: 'manager_state', path: '/data/manager-state', label: '管理器设置目录' }],
      agent: [],
    })
    invoke.mockResolvedValueOnce({ opened: true })
    await expect(app.loadManagedPathTargets()).resolves.toBe(true)
    const target = app.managedPathTargets?.find(item => item.key === 'manager_state')
    await expect(app.openManagedPath(target?.path)).resolves.toBe(true)
    expect(invoke).toHaveBeenNthCalledWith(1, 'get_managed_path_targets')
    expect(invoke).toHaveBeenNthCalledWith(2, 'open_managed_path', { path: '/data/manager-state' })
  })

  it('preserves a failure message when backend path projection is unavailable', async () => {
    const app = useAppStore()
    const toast = vi.spyOn(app, 'showToast').mockImplementation(() => {})
    invoke.mockRejectedValueOnce(new Error('unavailable'))
    await expect(app.loadManagedPathTargets()).resolves.toBe(false)
    expect(app.managedPathTargets).toBeNull()
    expect(toast).not.toHaveBeenCalled()
  })
})
