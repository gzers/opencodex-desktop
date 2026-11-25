import { mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import type { EnvironmentReport } from '@/features/environment/api'
import EnvironmentGate from '@/features/environment/components/EnvironmentGate.vue'
import { usePreparationStore, type PreparationPhase } from '@/features/environment/preparation'
import { buildEnvironmentPresentation } from '@/features/environment/presentation'
import { useEnvironmentStore } from '@/features/environment/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { useRouteStore } from '@/stores/routes'

function report(gate: EnvironmentReport['gate']): EnvironmentReport {
  const check = (found: boolean) => ({ found, path: found ? '/I/bin' : null, version: null })
  return {
    gate, node: check(gate !== 'missing_node'), npm: check(gate === 'ready' || gate === 'missing_ocx'),
    ocx: check(gate === 'ready'), shortCircuited: gate === 'missing_node' || gate === 'missing_npm', brewFound: false,
  }
}
function preparing(phase: PreparationPhase, revision: number) {
  usePreparationStore().snapshot = { attemptId: 'install', phase, revision, failedPhase: null, failure: null, retryable: false }
}
let view: VueWrapper | undefined
beforeEach(() => { setActivePinia(createPinia()) })
afterEach(() => { view?.unmount(); view = undefined })

it.each([
  ['missing_node', 'Node.js', ['未发现', '待检查', '待检查']],
  ['missing_npm', 'npm', ['已发现', '未发现', '待检查']],
  ['missing_ocx', 'OpenCodex', ['已发现', '已发现', '未发现']],
] as const)('renders the built-in %s gate and its real actions', async (gate, missing, values) => {
  const environment = useEnvironmentStore()
  environment.set(report(gate))
  const recheck = vi.fn()
  view = mount(EnvironmentGate, { props: { onRecheck: recheck } })
  expect(view.get('h2').text()).toBe('未发现内置 ' + missing)
  expect(view.findAll('.overview-check-value').map(check => check.text())).toEqual(values)
  expect(view.text()).not.toMatch(/Homebrew|brew install|npm install|官方下载安装|导入离线包/)
  const buttons = view.findAll('button')
  expect(buttons.map(button => button.text())).toEqual(['重新检查', '查看安装配置', '稍后处理'])
  await buttons[0].trigger('click')
  expect(recheck).toHaveBeenCalledOnce()
  await buttons[1].trigger('click')
  expect(useRouteStore().current).toBe('settings')
  expect(useRouteStore().settingsSection).toBe('installation')
  expect(environment.ready).toBe(false)
  await buttons[2].trigger('click')
  expect(view.get('h2').text()).toBe('运行环境待处理')
  expect(view.get('button').text()).toBe('查看环境准备')
  await view.get('button').trigger('click')
  expect(view.get('h2').text()).toBe('未发现内置 ' + missing)
})

it('a discovery error, a running check, and a ready report display distinct observations', async () => {
  const environment = useEnvironmentStore()
  environment.set(report('ready'))
  environment.error = 'probe failed'
  view = mount(EnvironmentGate)
  expect(environment.ready).toBe(false)
  expect(view.get('h2').text()).toBe('集成运行程序检查未就绪')
  expect(view.findAll('.overview-check-value').map(check => check.text())).toEqual(['未声明', '未声明', '未声明'])
  environment.loading = true
  await view.vm.$nextTick()
  expect(view.findAll('.overview-check-value').map(check => check.text())).toEqual(['检查中', '待检查', '待检查'])
  expect(view.get('button').attributes('disabled')).toBeDefined()
  environment.set(report('ready'))
  expect(environment.ready).toBe(true)
  expect(environment.presentation.checks.every(check => check.tone === 'ok')).toBe(true)
})

it('preparation masks an old ready report until discovery for the supplied attempt settles', async () => {
  const environment = useEnvironmentStore()
  environment.set(report('ready'))
  preparing('extracting', 2)
  view = mount(EnvironmentGate)
  expect(environment.ready).toBe(false)
  expect(view.get('h2').text()).toBe('展开运行包与依赖')
  expect(view.text()).toContain('无需联网')
  expect(view.text()).not.toContain('稍后处理')
  preparing('supplied', 3)
  await view.vm.$nextTick()
  expect(environment.ready).toBe(false)
  expect(view.text()).toContain('等待内置依赖检查结果')
  environment.checkedPreparationAttempt = 'install'
  environment.set(report('missing_ocx'))
  await view.vm.$nextTick()
  expect(view.get('h2').text()).toBe('未发现内置 OpenCodex')
  expect(environment.ready).toBe(false)
})

it('explicit missing reports remain visible even when the current installation source resolves', () => {
  useRuntimeStore().source = {
    kind: 'managed', path: '/I/bin/ocx', version: '2.99.0', resolvedAt: null, insideDataRoot: false,
    managedEntry: '/I/bin/ocx', managedPrefix: '/I', defaultPrefix: '/I', explicitPath: null, history: [],
  }
  const environment = useEnvironmentStore()
  const missing = report('missing_ocx')
  environment.set(missing)
  view = mount(EnvironmentGate)
  expect(view.get('h2').text()).toBe('未发现内置 OpenCodex')
  const withBrew = buildEnvironmentPresentation({ ...missing, brewFound: true })
  expect(withBrew).toEqual(environment.presentation)
})
