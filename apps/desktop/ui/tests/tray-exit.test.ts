import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'

const exitMock = vi.fn()
vi.mock('@tauri-apps/plugin-process', () => ({
  exit: (...args: unknown[]) => exitMock(...args),
}))

describe('tray exit app action', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    exitMock.mockReset()
  })

  it('invokes the Tauri process exit contract with code zero', async () => {
    const app = useAppStore()
    exitMock.mockResolvedValueOnce(undefined)
    await app.exitApp()
    expect(exitMock).toHaveBeenCalledWith(0)
    expect(app.appExitError).toBe('')
  })

  it('shows an explicit failure and preserves current state', async () => {
    const app = useAppStore()
    const showToast = vi.spyOn(app, 'showToast').mockImplementation(() => {})
    exitMock.mockRejectedValueOnce(new Error('plugin unavailable'))
    await app.exitApp()
    expect(exitMock).toHaveBeenCalledWith(0)
    expect(app.appExitError).toBe('退出失败；桌面壳保持当前状态。')
    expect(showToast).toHaveBeenCalledWith('退出失败；桌面壳保持当前状态。')
  })
})
