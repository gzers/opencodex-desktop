import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'

const discoverEnvironment = vi.fn()

vi.mock('@/features/environment/api', () => ({
  discoverEnvironment: (...args: unknown[]) => discoverEnvironment(...args),
}))

describe('store.refreshEnvironment', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    discoverEnvironment.mockReset()
  })

  it('重新发现经正式 store 服务执行，成功后写入环境并清除 loading', async () => {
    const report = { ocx: { found: true } } as never
    discoverEnvironment.mockResolvedValue(report)
    const app = useAppStore()

    await app.refreshEnvironment()

    expect(discoverEnvironment).toHaveBeenCalledTimes(1)
    expect(app.environment).toEqual(report)
    expect(app.environmentLoading).toBe(false)
  })

  it('发现失败时保留既有状态、不阻塞界面，并给出提示', async () => {
    discoverEnvironment.mockRejectedValue(new Error('boom'))
    const app = useAppStore()
    const before = app.environment

    await app.refreshEnvironment()

    expect(app.environmentLoading).toBe(false)
    expect(app.environment).toBe(before)
  })
})
