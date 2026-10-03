import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { buildEnvironmentPresentation } from '@/features/environment/presentation'
import EnvironmentGate from '@/features/environment/components/EnvironmentGate.vue'

import type { EnvironmentReport } from '@/features/environment/api'
import { useEnvironmentStore } from '@/features/environment/store'

function report(overrides: Partial<EnvironmentReport> = {}): EnvironmentReport {
  const found = { found: true, path: '/fixtures/node', version: null }
  const notChecked = { found: false, path: null, version: null }
  return {
    node: { ...found },
    npm: { ...notChecked },
    ocx: { ...notChecked },
    gate: 'missing_node',
    shortCircuited: true,
    brewFound: true,
    ...overrides,
  }
}

describe('environment presentation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('maps all five backend gates without inventing values', () => {
    const cases: Array<[EnvironmentReport['gate'], string, string, boolean, string]> = [
      ['ready', '运行环境检查通过', 'ok', true, 'ready'],
      ['missing_node', '缺少 Node.js，OpenCodex 发现已暂停', '', false, 'missing_node_brew'],
      ['missing_npm', 'Node.js 已发现，但 npm 不可用', '', false, 'missing_npm'],
      ['missing_ocx', '运行环境已就绪，未发现 OpenCodex', '', false, 'missing_ocx'],
    ]

    cases.forEach(([gate, title, statusClass, hidden, expectedState]) => {
      const presentation = buildEnvironmentPresentation(report({ gate }))
      expect(presentation.title).toBe(title)
      expect(presentation.statusClass).toBe(statusClass)
      expect(presentation.hidden).toBe(hidden)
      expect(presentation.state).toBe(expectedState)
    })
  })

  it('uses official missing node guidance and label based on brew discovery', () => {
    const withBrew = buildEnvironmentPresentation(report({ brewFound: true }))
    const withoutBrew = buildEnvironmentPresentation(report({ brewFound: false }))

    expect(withBrew.commands).toEqual([
      { label: '安装 Node.js', command: 'brew install node' },
      { label: '安装后校验', command: 'node -v && npm -v' },
    ])
    expect(withBrew.link).toBeUndefined()
    expect(withoutBrew.commands).toEqual([{ label: '安装后校验', command: 'node -v && npm -v' }])
    expect(withoutBrew.link?.text).toBe('Node.js LTS 官方安装渠道')
  })

  it('marks unchecked dependencies and blocked installation area', () => {
    const app = useAppStore()
    app.setEnvironment(report({ gate: 'missing_npm' }))
    const presentation = buildEnvironmentPresentation(app.environment)
    expect(presentation.checks.map(({ name, value, state }) => [name, value, state])).toEqual([
      ['Node.js', '已发现', 'running'],
      ['npm', '未发现', 'bad'],
      ['OpenCodex', '未检查', 'muted'],
    ])
    expect(presentation.blocked).toBe(true)
    expect(presentation.installEmptyMessage).toBe(
      '已按前置条件暂停 OpenCodex 发现；请先处理概览中的环境门禁。',
    )
  })

  it('treats a resolved runtime source as OpenCodex available instead of missing', () => {
    // 真机回归：托管安装落在数据根内、不在发现候选里；只按自动发现判断会误报
    // 「未发现 OpenCodex」并给出安装动作（UI规范 §18.5）。
    for (const kind of ['managed', 'explicit', 'discovered'] as const) {
      const presentation = buildEnvironmentPresentation(report({ gate: 'missing_ocx' }), false, kind)
      expect(presentation.state).toBe('ready_via_source')
      expect(presentation.hidden).toBe(false)
      expect(presentation.blocked).toBe(false)
      expect(presentation.statusClass).toBe('ok')
      expect(presentation.title).not.toContain('未发现 OpenCodex')
      expect(presentation.checks[2]).toEqual({
        name: 'OpenCodex',
        value: '运行来源',
        state: 'running',
        label: '通过',
      })
      // 不再给出「安装 / 导入离线包」这类动作，也不再阻断设置页安装区。
      expect(presentation.commands).toEqual([])
      expect(presentation.installEmptyMessage).toBeUndefined()
    }
  })

  it('keeps the missing ocx gate when no runtime source is resolved', () => {
    for (const kind of [null, 'unresolved'] as const) {
      const presentation = buildEnvironmentPresentation(report({ gate: 'missing_ocx' }), false, kind)
      expect(presentation.state).toBe('missing_ocx')
      expect(presentation.blocked).toBe(true)
    }
  })

  it('does not let a resolved source hide a real node/npm prerequisite gap', () => {
    for (const gate of ['missing_node', 'missing_npm'] as const) {
      const presentation = buildEnvironmentPresentation(report({ gate }), false, 'managed')
      expect(presentation.blocked).toBe(true)
      expect(presentation.state).not.toBe('ready_via_source')
    }
  })

  it('uses checking text only while a request is pending', () => {
    const app = useAppStore()
    expect(app.environmentLoading).toBe(false)
    app.setEnvironmentLoading(true)
    const presentation = buildEnvironmentPresentation(null, true)
    expect(presentation.title).toBe('正在检查运行环境')
    expect(presentation.hidden).toBe(false)
    expect(presentation.statusClass).toBe('busy')
    expect(presentation.blocked).toBe(false)
    expect(presentation.installEmptyMessage).toBeUndefined()
  })
})

describe('environment gate copy action', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  function mountGate() {
    useEnvironmentStore().report = report({ gate: 'missing_node' })
    // 使用 setActivePinia 设定的同一 store 实例，避免组件挂到另一份 pinia。
    return mount(EnvironmentGate)
  }

  it('copies the command without claiming to run it or to be a simulation', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } })
    const wrapper = mountGate()
    const app = useAppStore()

    const button = wrapper.find('.cli-copy-btn')
    expect(button.exists()).toBe(true)
    await button.trigger('click')

    expect(writeText).toHaveBeenCalled()
    expect(app.toast).toContain('不会代你执行')
    expect(app.toast).not.toContain('原型')
    expect(app.toast).not.toContain('模拟')
  })

  it('does not claim success when the clipboard is unavailable', async () => {
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: undefined })
    const wrapper = mountGate()
    const app = useAppStore()

    await wrapper.find('.cli-copy-btn').trigger('click')
    expect(app.toast).toContain('剪贴板不可用')
    expect(app.toast).not.toContain('已复制')
  })
})
