import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRuntimeStore } from '@/features/runtime/store'
import { useRouteStore } from '@/stores/routes'
import type { InstallPhase, RuntimeInstallOutcomeDto, RuntimeSourceDto } from '@/features/runtime/api'
import { useEffectsStore } from '@/app/appearance/effects'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
const source: RuntimeSourceDto = {
  kind: 'managed', path: '/fixture/runtime/bin/ocx', version: '2.50.0', resolvedAt: null,
  insideDataRoot: true, managedEntry: '/fixture/runtime/bin/ocx',
  managedPrefix: '/fixture/runtime/opencodex', defaultPrefix: '/fixture/runtime/opencodex',
  explicitPath: null, history: [],
}
const outcome: RuntimeInstallOutcomeDto = {
  package: '@bitkyc08/opencodex', version: '2.51.0', target: source.managedPrefix!,
  entry: source.managedEntry!, tarballSha256: 'a'.repeat(64), source: 'registry',
  scriptsEnabled: false, npmPath: '/fixture/npm', installedAt: '2026-10-14T00:00:00Z',
  proxyUsed: false, needsRestart: true, protectionReconciliationPending: true,
}
const mounted: Array<ReturnType<typeof mount>> = []
beforeEach(() => {
  invoke.mockReset()
  invoke.mockImplementation(async command => {
    if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
    if (command === 'runtime_source') return source
    return null
  })
})
afterEach(() => mounted.splice(0).forEach(wrapper => wrapper.unmount()))

async function wizard() {
  const pinia = createPinia()
  setActivePinia(pinia)
  const runtime = useRuntimeStore()
  runtime.source = source
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] } })
  mounted.push(page)
  await flushPromises()
  await page.get('[data-testid="runtime-install"]').trigger('click')
  const modal = page.get('[data-testid="runtime-install-modal"]')
  const next = () => modal.findAll('button').find(button => button.text() === '下一步')!
  await next().trigger('click')
  await next().trigger('click')
  await modal.findAll('input[type="checkbox"]').at(-1)!.setValue(true)
  let reject!: (reason: unknown) => void
  invoke.mockImplementation(command => command === 'install_runtime'
    ? new Promise((_, fail) => { reject = fail }) : Promise.resolve(command === 'runtime_source' ? source : null))
  await next().trigger('click')
  await flushPromises()
  return { page, modal, runtime, reject }
}

describe('installation wizard factual progress', () => {
  it.each(['preparing', 'downloading', 'installing', 'extracting', 'verifying', 'activating', 'done'] as InstallPhase[])
    ('never presents a stage constant as completed work in %s', async phase => {
      const { modal, runtime } = await wizard()
      runtime.observeInstallProgress({ phase, percent: 88, line: 'stage marker' })
      await flushPromises()
      const bar = modal.get('[role="progressbar"]')
      expect(bar.attributes('aria-valuenow')).toBeUndefined()
      expect(bar.attributes('aria-valuetext')).toContain('等待实际完成')
      expect(modal.get('[data-testid="wiz-status"]').text()).not.toMatch(/\d+%/)
      expect(modal.get('[data-testid="wiz-status"]').text()).not.toContain('安装完成')
    })

  it('stops the busy indicator and keeps failure without a false 100 percent', async () => {
    const { modal, reject } = await wizard()
    reject({ message: 'fixture installation rejected' })
    await flushPromises()
    expect(modal.get('[data-testid="wiz-error"]').text()).toContain('fixture installation rejected')
    expect(modal.find('[role="progressbar"]').exists()).toBe(false)
    expect(modal.get('[data-testid="wiz-status"]').text()).not.toMatch(/\d+%/)
    expect(modal.get('[data-testid="wiz-status"]').text()).toContain('安装失败')
  })

  it('shows completion only after a successful outcome and keeps restart/protection facts separate', async () => {
    const { modal, runtime } = await wizard()
    runtime.setInstallOutcome(outcome)
    runtime.markInstallDone()
    await flushPromises()
    expect(modal.get('[role="progressbar"]').attributes('aria-valuenow')).toBeUndefined()
    runtime.finishInstall()
    await flushPromises()
    expect(modal.get('[role="progressbar"]').attributes('aria-valuenow')).toBe('100')
    expect(modal.get('[data-testid="wiz-status"]').text()).toContain('安装完成')
    expect(modal.get('[data-testid="wiz-outcome"]').text()).toContain('重启后生效')
    expect(modal.get('[data-testid="wiz-outcome"]').text()).toContain('备份仍保留')
    expect(modal.get('[data-testid="wiz-status"]').text()).not.toMatch(/\d+%/)
  })

  it('keeps the stage readable while quality or window visibility pauses animation', async () => {
    const { modal, runtime } = await wizard()
    const effects = useEffectsStore()
    runtime.observeInstallProgress({ phase: 'installing', percent: 45, line: 'actual npm output' })
    for (const setting of ['high', 'mid', 'low'] as const) {
      effects.setSetting(setting)
      await flushPromises()
      expect(modal.get('.wiz-progress-bar').classes().includes('is-animated')).toBe(setting === 'high')
      expect(modal.get('[role="progressbar"]').attributes('aria-valuenow')).toBeUndefined()
      expect(modal.get('[data-testid="wiz-status"]').text()).toContain('正在安装进私有前缀')
    }
    effects.setSetting('high')
    effects.setVisible(false)
    await flushPromises()
    expect(modal.get('.wiz-progress-bar').classes()).not.toContain('is-animated')
    effects.setVisible(true)
    effects.setForeground(false)
    await flushPromises()
    expect(modal.get('.wiz-progress-bar').classes()).not.toContain('is-animated')
    effects.setForeground(true)
    await flushPromises()
    expect(modal.get('.wiz-progress-bar').classes()).toContain('is-animated')
  })
})
