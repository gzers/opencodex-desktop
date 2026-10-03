import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import LogsRoute from '@/routes/LogsRoute.vue'

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

function logsDto(kind: 'app' | 'audit', lines: string[]) {
  return { kind, lines, redactedLineCount: 0, truncated: false, fileMissing: false, fallbackFile: null }
}

// 诊断中心默认落地「环境诊断」；日志面板是 v-if 渲染，测日志相关用例需显式落在日志 Tab。
// 传 null 表示「没有显式入口」，用来验证默认落地页。
function mountRoute(
  diagnosticsTab: 'doctor' | 'logs' | null = 'logs',
  config?: { errorHandler?: (error: unknown, instance: unknown, info: string) => void },
) {
  window.location.hash = diagnosticsTab === null ? '' : `#logs?tab=${diagnosticsTab}`
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(LogsRoute, { global: { plugins: [pinia], config } })
}

describe('log categories', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockImplementation(async (command: string, payload?: { kind?: string }) => {
      if (command === 'read_logs') {
        return payload?.kind === 'audit'
          ? logsDto('audit', ['2026-09-23 调用 opx status', '2026-09-23 调用 opx start'])
          : logsDto('app', ['2026-09-23 应用启动'])
      }
      return null
    })
  })

  it('reads the application log first and exposes both categories', async () => {
    invoke.mockImplementation(async (command: string, payload?: { kind?: string }) => {
      if (command === 'read_logs') {
        return payload?.kind === 'audit'
          ? logsDto('audit', ['调用 opx status'])
          : logsDto('app', ['应用启动'])
      }
      return null
    })
    const wrapper = mountRoute()
    await nextTick()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'app' }))
    const tabs = wrapper.findAll('[aria-label="日志分类"] button')
    expect(tabs.map(tab => tab.text())).toEqual(['应用日志', '调用日志'])
    expect(wrapper.find('.log').text()).toContain('应用启动')
    await wrapper.unmount()
  })

  it('switches to the call log (audit) when the second category is chosen', async () => {
    const wrapper = mountRoute()
    await nextTick()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'app' }))
    await wrapper.findAll('[aria-label="日志分类"] button')[1].trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'audit' }))
    await vi.waitFor(() => expect(wrapper.find('.log').text()).toContain('调用 opx start'))
    expect(wrapper.text()).toContain('调用日志')
    await wrapper.unmount()
  })

  it('renders log content as text, never as HTML', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'read_logs') return logsDto('app', ['<b>不应作为标签执行</b>', '<img src=x onerror=alert(1)>'])
      return null
    })
    const wrapper = mountRoute()
    await nextTick()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'app' }))
    const log = wrapper.find('.log')
    expect(log.text()).toContain('<b>不应作为标签执行</b>')
    expect(log.find('b').exists()).toBe(false)
    expect(log.find('img').exists()).toBe(false)
    await wrapper.unmount()
  })

  it('offers a panel request log entry that navigates to the official panel route', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'read_logs') return logsDto('app', ['应用启动'])
      return null
    })
    const wrapper = mountRoute()
    await nextTick()
    const entry = wrapper.findAll('button').find(button => button.text() === '面板请求日志')
    expect(entry).toBeTruthy()
    await entry!.trigger('click')
    const { useRouteStore } = await import('@/stores/routes')
    expect(useRouteStore().current).toBe('panel')
    await wrapper.unmount()
  })

  // 2026-09-24 用户要求：Doctor 做成 Tab、标题四个字、放第一个。
  it('puts 环境诊断 first among the diagnostics tabs and shares the content card', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'read_logs') return logsDto('app', ['应用启动'])
      return null
    })
    // 不带显式 tab：验证默认落地页是「环境诊断」。
    const wrapper = mountRoute(null)
    await nextTick()
    const { useRouteStore } = await import('@/stores/routes')
    const routes = useRouteStore()
    const tabs = wrapper.findAll('div.diag-tabs')[0].findAll('button')
    expect(tabs.map(tab => tab.text())).toEqual(['环境诊断', '日志历史', '通知历史'])
    expect(tabs[0].text()).toHaveLength(4)
    // 2026-09-28 用户要求：默认落地页是环境诊断（与原型一致）。
    expect(routes.diagnosticsTab).toBe('doctor')
    expect(wrapper.text()).toContain('运行环境诊断')
    expect(wrapper.find('.log').exists()).toBe(false)
    expect(wrapper.find('.doctor-hint').exists()).toBe(true)
    // 默认面板里就有「界面诊断」自检入口（与原型同款）。
    expect(wrapper.find('#uiSelfCheckBtn').text()).toBe('界面诊断')
    // 三个 Tab 共处同一张内容卡：不存在第二张 doctor 卡。
    expect(wrapper.findAll('article.card').length).toBe(1)
    await tabs[1].trigger('click')
    expect(routes.diagnosticsTab).toBe('logs')
    expect(wrapper.text()).not.toContain('运行环境诊断')
    expect(wrapper.find('.log').exists()).toBe(true)
    await wrapper.unmount()
  })

  it('falls back to 环境诊断 for unknown diagnostics tab values', async () => {
    const { diagnosticsTabs } = await import('@/navigation')
    expect(diagnosticsTabs).toEqual(['doctor', 'logs', 'notifications'])
    // 初始落地页由 hash 决定：清掉 hash 才代表「没有显式入口」。
    window.location.hash = ''
    setActivePinia(createPinia())
    const { useRouteStore } = await import('@/stores/routes')
    const routes = useRouteStore()
    expect(routes.diagnosticsTab).toBe('doctor')
    routes.go('logs', { tab: 'nonsense' })
    expect(routes.diagnosticsTab).toBe('doctor')
    routes.go('logs', { tab: 'logs' })
    expect(routes.diagnosticsTab).toBe('logs')
    routes.go('logs', { tab: 'doctor' })
    expect(routes.diagnosticsTab).toBe('doctor')
  })

  // IMP-04 §14.3：运行期兜底条在正式制品里没有触发入口；诊断中心提供「界面诊断」自检，
  // 故意抛出的受控异常必须走真实错误边界（Vue errorHandler）→ 置位兜底条，而不是直接改状态。
  it('界面诊断走真实错误边界：受控异常经 errorHandler 置位兜底条，可关闭恢复', async () => {
    const { describeFault } = await import('@/app/errors')
    const { useAppStore } = await import('@/stores/app')
    const infos: string[] = []
    const wrapper = mountRoute('doctor', {
      errorHandler: (error, _instance, info) => {
        infos.push(info)
        // 与 main.ts 同一责任位置：错误边界 → reportUiFault。
        useAppStore().reportUiFault(describeFault('界面异常', error), info)
      },
    })
    await nextTick()
    const app = useAppStore()
    const button = wrapper.find('#uiSelfCheckBtn')
    expect(button.exists()).toBe(true)
    expect(button.text()).toBe('界面诊断')
    // Doctor 按钮也已是中文名，与原型一致。
    expect(wrapper.findAll('button').some(item => item.text() === '环境诊断')).toBe(true)

    expect(app.uiFault).toBe('')
    await button.trigger('click')
    await nextTick()
    expect(infos).toHaveLength(1)
    expect(app.uiFault).toContain('界面异常自检')
    expect(app.recentEvents[0]?.message).toContain('界面异常')

    app.clearUiFault()
    expect(app.uiFault).toBe('')
    await wrapper.unmount()
  })
})
