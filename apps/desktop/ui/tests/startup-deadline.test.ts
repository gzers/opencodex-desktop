import { describe, expect, it } from 'vitest'
import { waitForPreferences, waitForSettledSnapshot } from '@/startup'

// IMP-04 §13.3 A04：等待必须有可执行的上限，单次 refresh/偏好读取挂起也不能越过 deadline。
describe('startup deadlines', () => {
  it('refresh 永不返回时仍在上限内退出，返回最后一次读到的快照', async () => {
    const snapshot = { matrix: { runtime: 'loading' } } as never
    const started = Date.now()
    const result = await waitForSettledSnapshot({
      read: () => snapshot,
      refresh: () => new Promise(() => {}),
      delay: async () => {},
      timeoutMs: 40,
    })
    expect(result).toBe(snapshot)
    expect(Date.now() - started).toBeLessThan(1000)
  })

  it('偏好读取挂起时按“未读到”继续，不阻塞首帧', async () => {
    const started = Date.now()
    const ok = await waitForPreferences(() => new Promise(() => {}), 40)
    expect(ok).toBe(false)
    expect(Date.now() - started).toBeLessThan(1000)
  })

  it('偏好读取正常完成返回 true', async () => {
    expect(await waitForPreferences(() => Promise.resolve(), 40)).toBe(true)
  })
})
