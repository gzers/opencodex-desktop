import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import TrayRoute from '@/routes/TrayRoute.vue'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))

describe('tray route copy', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
    setActivePinia(createPinia())
  })

  // TR-1：标题按原型 index.html:2978 改为「托盘原型 · 状态菜单」。
  it('uses the prototype tray title', () => {
    const wrapper = mount(TrayRoute)
    expect(wrapper.get('.card-head h2').text()).toBe('托盘原型 · 状态菜单')
    expect(wrapper.text()).toContain('展示 macOS 与 Windows 的样式差异；菜单动作全部由管理器自有域触发，不直接读写 OpenCodex 配置。')
    wrapper.unmount()
  })
})
