import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'
import { useDataRootStore } from '@/features/data-root/store'

describe('overview direct data root actions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    window.location.hash = '#overview'
  })

  it('opens active data root and home through the whitelisted managed path command', async () => {
    const app = useAppStore()
    useAppController()
    invoke.mockResolvedValueOnce({
      managed: [
        { key: 'manager_state', path: '/data/manager-state', label: '管理器设置目录' },
        { key: 'backups', path: '/data/backups', label: '备份目录' },
      ],
      agent: [],
    })
    invoke.mockResolvedValueOnce({ opened: true })
    await expect(app.openOverviewDataRoot()).resolves.toBe(true)
    invoke.mockResolvedValueOnce({ managed: [], agent: [] })
    invoke.mockResolvedValueOnce({ opened: true })
    useDataRootStore().config = { activeDataRoot: '/data', opencodexHome: '/home', opencodexHomeMode: 'external', runtimeActive: true }
    await expect(app.openOverviewOpencodexHome()).resolves.toBe(true)
    expect(invoke).toHaveBeenNthCalledWith(1, 'get_managed_path_targets')
    expect(invoke).toHaveBeenNthCalledWith(2, 'open_managed_path', { path: '/data' })
    expect(invoke).toHaveBeenNthCalledWith(3, 'get_managed_path_targets')
    expect(invoke).toHaveBeenNthCalledWith(4, 'open_managed_path', { path: '/home' })
  })

  it('preserves the current window when loading targets fails', async () => {
    const app = useAppStore()
    const toast = vi.spyOn(app, 'showToast').mockImplementation(() => {})
    invoke.mockRejectedValueOnce(new Error('unavailable'))
    await expect(app.openOverviewDataRoot()).resolves.toBe(false)
    expect(toast).toHaveBeenCalledWith('数据目录不可用；已保留当前窗口。')
    expect(invoke).not.toHaveBeenCalledWith('open_managed_path', expect.anything())
  })

  it('preserves the current window when the backend cannot open a root', async () => {
    const app = useAppStore()
    const toast = vi.spyOn(app, 'showToast').mockImplementation(() => {})
    invoke.mockResolvedValueOnce({ managed: [], agent: [] })
    invoke.mockResolvedValueOnce({ opened: false })
    await expect(app.openOverviewOpencodexHome()).resolves.toBe(false)
    expect(toast).toHaveBeenCalledWith('OPENCODEX_HOME 不可用；已保留当前窗口。')
  })
})
