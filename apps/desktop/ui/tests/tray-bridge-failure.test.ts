import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'

// 回归（F-07）：托盘命令经前端轮询派发。IPC 故障此前被 catch(() => {}) 静默吞掉，
// 表现为「菜单能展开但点了没反应」。这里要求故障可见（提示 + 事件），恢复后登记恢复。
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

describe('tray command channel failures are not silent (F-07)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    vi.useFakeTimers()
  })
  afterEach(() => vi.useRealTimers())

  it('surfaces the failure once and reports recovery', async () => {
    let failing = true
    invoke.mockImplementation((command: string) => {
      if (command === 'drain_tray_requests') {
        return failing ? Promise.reject(new Error('ipc down')) : Promise.resolve([])
      }
      return Promise.resolve(undefined)
    })
    const wrapper = mount(Harness)
    const app = useAppStore()
    await vi.advanceTimersByTimeAsync(1100)
    expect(app.toast).toBe('托盘命令通道暂不可用；可改用应用内按钮。')

    // 恢复：后续排空成功，通道恢复登记一次，不再重复告警。
    failing = false
    await vi.advanceTimersByTimeAsync(1100)
    expect(app.recentEvents.some(event => event.message.includes('托盘命令通道已恢复'))).toBe(true)
    wrapper.unmount()
  })
})

