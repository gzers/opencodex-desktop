import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function mountGeneral() {
  const pinia = createPinia()
  setActivePinia(pinia)
  const routes = useRouteStore()
  routes.go('settings', { section: 'general' })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
  return { wrapper }
}

function shimRow(wrapper: ReturnType<typeof mount>) {
  const row = wrapper.findAll('.setting-row').find(node => node.text().includes('随 Codex 启动 OpenCodex'))
  expect(row, '找不到设置行：随 Codex 启动 OpenCodex').toBeTruthy()
  return row!
}

const notInstalled = (reason = '') => ({ state: 'not_installed', installed: false, summary: 'not installed', reason })
const installed = () => ({ state: 'installed', installed: true, summary: 'Codex autostart shim: wrapper shim present.', reason: '' })

// 2026-09-27：该行从「只读事实」改为经官方 shim 真实读写的开关（写入通道跟随官方，AC-11）。
describe('随 Codex 启动 OpenCodex 经官方 shim 读写', () => {
  beforeEach(() => {
    invoke.mockReset()
  })

  it('按官方状态渲染可交互开关，而不是只读结论', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(installed())
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    const row = shimRow(wrapper)
    expect(row.find('.setting-readonly').exists()).toBe(false)
    const toggle = row.find('button.toggle')
    expect(toggle.exists()).toBe(true)
    expect(toggle.attributes('aria-checked')).toBe('true')
    await wrapper.unmount()
  })

  it('点击开关经官方 CLI 写入并回读真实状态', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(installed())
      if (command === 'set_codex_shim') return Promise.resolve(notInstalled())
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    await shimRow(wrapper).find('button.toggle').trigger('click')
    await flushPromises()
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('set_codex_shim', { enabled: false })
    expect(shimRow(wrapper).find('button.toggle').attributes('aria-checked')).toBe('false')
    await wrapper.unmount()
  })

  // 真机实测：官方 shim 安装会先做探针，要 ~20 秒；这段必须给出进行态，
  // 否则用户只会看到「点了没反应」。2026-09-27 起进行态落在该行内部底部的小框里。
  it('写入进行中在该行内部底部显示进行态小框（开关保持可见但禁用）', async () => {
    // 用可变对象持有 resolve，避免 TS 把闭包里的赋值当成 never。
    const pending: { resolve?: (value: unknown) => void } = {}
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(notInstalled())
      if (command === 'set_codex_shim') {
        return new Promise(resolve => {
          pending.resolve = resolve
        })
      }
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    const row = shimRow(wrapper)
    expect(row.find('[data-testid="codex-shim-panel"]').exists()).toBe(false)

    await row.find('button.toggle').trigger('click')
    await flushPromises()
    await nextTick()

    // 面板是 setting-row 的直接子元素（独占该行底部一行）。
    const panel = row.find('[data-testid="codex-shim-panel"]')
    expect(panel.exists()).toBe(true)
    expect(panel.element.parentElement?.classList.contains('setting-row')).toBe(true)
    expect(panel.find('[data-testid="codex-shim-panel-title"]').text()).toBe('正在安装官方 shim')
    // 开启是长耗时动作：给不确定进度条，且**不编百分比**（无 aria-valuenow）。
    const bar = panel.find('[role="progressbar"]')
    expect(bar.exists()).toBe(true)
    expect(bar.attributes('aria-valuenow')).toBeUndefined()
    // 开关保持可见但禁用，不换成只读文案。
    const toggle = row.find('button.toggle')
    expect(toggle.exists()).toBe(true)
    expect(toggle.attributes('disabled')).toBeDefined()

    pending.resolve?.(installed())
    await flushPromises()
    await nextTick()
    await flushPromises()
    await nextTick()
    // 完成后结果说明写清「指令做了什么」。
    expect(panel.find('[data-testid="codex-shim-result"]').text()).toContain('已安装官方 shim')
    expect(panel.find('[data-testid="codex-shim-result"]').text()).toContain('codex.opencodex-real')
    expect(row.find('button.toggle').attributes('aria-checked')).toBe('true')
    expect(row.find('button.toggle').attributes('disabled')).toBeUndefined()
    await wrapper.unmount()
  })

  // 用户纠偏：关闭是短、单一动作，不该复用开启那根长耗时进度条。
  it('关闭时不给进度条，并在完成后说明「还原」了什么', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(installed())
      if (command === 'set_codex_shim') return Promise.resolve(notInstalled())
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    await shimRow(wrapper).find('button.toggle').trigger('click')
    await flushPromises()
    await nextTick()

    const panel = shimRow(wrapper).find('[data-testid="codex-shim-panel"]')
    expect(panel.classes()).toContain('is-close')
    expect(panel.find('[role="progressbar"]').exists()).toBe(false)
    const result = panel.find('[data-testid="codex-shim-result"]').text()
    expect(result).toContain('已关闭官方 shim')
    expect(result).toContain('还原')
    await wrapper.unmount()
  })

  it('可展开下拉查看实际执行的具体指令', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(notInstalled())
      if (command === 'set_codex_shim') {
        return new Promise(() => {}) // 一直挂起，停在进行态
      }
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    await shimRow(wrapper).find('button.toggle').trigger('click')
    await flushPromises()
    await nextTick()

    const row = shimRow(wrapper)
    expect(row.find('[data-testid="codex-shim-details"]').exists()).toBe(false)
    await row.find('.shim-details-toggle').trigger('click')
    const details = row.find('[data-testid="codex-shim-details"]')
    expect(details.exists()).toBe(true)
    const cmds = details.findAll('.shim-cmds code').map(node => node.text())
    expect(cmds).toContain('ocx codex-shim install')
    expect(cmds).toContain('ocx codex-shim status')
    await wrapper.unmount()
  })

  it('官方 CLI 未就绪时按只读事实呈现，并在行内说明原因', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') {
        return Promise.resolve({ state: 'unreachable', installed: false, summary: '', reason: 'unresolved' })
      }
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()

    const row = shimRow(wrapper)
    expect(row.find('button.toggle').exists()).toBe(false)
    expect(row.find('.setting-readonly').text()).toBe('官方 CLI 未就绪')
    expect(row.find('[data-testid="codex-shim-notice"]').text()).toContain('未解析到 ocx 可执行文件')
    await wrapper.unmount()
  })

  // 真机缺陷回归：官方 CLI 未真正生效时，开关不能停在开启态；
  // 原因要写进行内说明（toast 1.9 秒就消失，承载不了「去开哪个权限」）。
  it.each([
    [
      '找不到 codex（官方退出 0 但什么都没做）',
      'managed runtime operation failed: codex_shim_rejected; \u26a0\ufe0f  Could not find a codex executable on PATH.',
      '未找到 codex 可执行文件',
    ],
    [
      'macOS 拒绝改动 Codex 所在的应用包（EPERM）',
      "managed runtime operation failed: codex_shim_rejected; EPERM: operation not permitted, rename '/Applications/ChatGPT.app/Contents/Resources/codex'",
      'App 管理',
    ],
  ])('官方 CLI 未生效（%s）时保持关闭并给出行内说明', async (_label, message, expected) => {
    invoke.mockImplementation((command: string) => {
      if (command === 'codex_shim_status') return Promise.resolve(notInstalled())
      if (command === 'set_codex_shim') {
        return Promise.reject({ code: 14, message })
      }
      return Promise.resolve(null)
    })

    const { wrapper } = mountGeneral()
    await flushPromises()
    await nextTick()
    const app = useAppStore()
    const toast = vi.spyOn(app, 'showToast').mockImplementation(() => {})

    await shimRow(wrapper).find('button.toggle').trigger('click')
    await flushPromises()
    await nextTick()

    expect(invoke).toHaveBeenCalledWith('set_codex_shim', { enabled: true })
    const row = shimRow(wrapper)
    expect(row.find('button.toggle').attributes('aria-checked')).toBe('false')
    const notice = row.find('[data-testid="codex-shim-notice"]')
    expect(notice.exists()).toBe(true)
    expect(notice.text()).toContain(expected)
    expect(toast).toHaveBeenCalled()
    await wrapper.unmount()
  })
})
