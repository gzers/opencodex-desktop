import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppController } from '@/composables/useAppController'

// IMP-04 §19.12「后台空闲 IPC 频率」：在没有用户操作、没有状态迁移的空闲窗口内，
// 计数前端发起的 IPC 调用，验证「后台空闲不出现无效轮询刷屏」——空闲 IPC 只应是
// 托盘事件与低频兜底，且不随时间增长；状态/通知/来源均为事件驱动（listen，非轮询）。
const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: () => Promise.resolve(() => {}) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: () => Promise.resolve(null) }))
vi.mock('@/features/panel/api', () => ({
  syncEmbeddedPanel: () => Promise.resolve({ visible: false, panelUrl: null }),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

const Harness = defineComponent({
  setup() {
    useAppController(true)
    return () => h('div')
  },
})

describe('idle IPC frequency (IMP-04 §19.12)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    // 托盘排空返回空队列；其余命令返回 undefined（各功能方法内部自行收敛失败）。
    invoke.mockImplementation((command: string) =>
      command === 'drain_tray_requests' ? Promise.resolve([]) : Promise.resolve(undefined),
    )
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('uses low-frequency fallback without ramping while idle', async () => {
    const wrapper = mount(Harness)
    // 让启动装配（primeAppData 的一次性拉取）先跑完，再进入空闲观测窗口。
    await vi.advanceTimersByTimeAsync(2000)
    invoke.mockClear()

    await vi.advanceTimersByTimeAsync(60_000)
    const commands = invoke.mock.calls.map(call => call[0] as string)
    const drains = commands.filter(name => name === 'drain_tray_requests').length

    // 空闲 60s：仅 2 次低频兜底排空，且没有其它空闲 IPC（无状态/日志/通知轮询）。
    expect(drains).toBe(2)
    expect(commands.length).toBe(drains)

    // 不随时间增长：后半段与前半段速率一致（每 30s 各 1 次），排除「越跑越多」的泄漏式轮询。
    invoke.mockClear()
    await vi.advanceTimersByTimeAsync(30_000)
    const firstHalf = invoke.mock.calls.length
    await vi.advanceTimersByTimeAsync(30_000)
    const secondHalf = invoke.mock.calls.length - firstHalf
    expect(firstHalf).toBe(1)
    expect(secondHalf).toBe(1)

    wrapper.unmount()
  })
})
