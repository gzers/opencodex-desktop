import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppController } from '@/composables/useAppController'

const readLogs = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: () => Promise.resolve(undefined) }))
vi.mock('@/features/diagnostics/logs', () => ({
  readLogs: (...args: unknown[]) => readLogs(...args),
}))

const Harness = defineComponent({
  setup() {
    return useAppController()
  },
  render: () => h('div'),
})

type Vm = {
  loadLogs: () => void
  setLogCategory: (kind: 'app' | 'audit') => void
  logsState: { lines: string[] } | null
}

// IMP-04 §13.3 A03：请求有序，晚到的旧响应不得覆盖新界面。
describe('log load ordering', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    readLogs.mockReset()
  })

  it('切换分类后，晚到的旧分类响应不覆盖新结果', async () => {
    const pending: Record<string, (value: unknown) => void> = {}
    readLogs.mockImplementation(
      (kind: string) => new Promise(resolve => { pending[kind] = resolve }),
    )

    const wrapper = mount(Harness)
    const vm = wrapper.vm as unknown as Vm

    vm.loadLogs() // app，挂起
    await Promise.resolve()
    vm.setLogCategory('audit') // 新请求：audit
    await Promise.resolve()

    pending.audit({ lines: ['audit-line'] })
    for (let i = 0; i < 4; i += 1) await Promise.resolve()
    expect(vm.logsState?.lines).toEqual(['audit-line'])

    pending.app({ lines: ['app-line'] }) // 旧响应晚到
    for (let i = 0; i < 4; i += 1) await Promise.resolve()
    expect(vm.logsState?.lines).toEqual(['audit-line'])
  })
})
