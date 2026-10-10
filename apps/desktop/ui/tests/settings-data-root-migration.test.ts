import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import AppModal from '@/components/AppModal.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import type { DataRootConfig } from '@/features/data-root/api'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (media: string) => ({ matches: false, media, addEventListener() {}, removeEventListener() {} }),
})

const current: DataRootConfig = {
  activeDataRoot: '/fixtures/source', currentDataRoot: '/fixtures/source',
  opencodexHomeMode: 'external', opencodexHome: '/fixtures/external-home',
  currentOpencodexHome: '/fixtures/external-home', runtimeActive: true,
}
const mounted: Array<ReturnType<typeof mount>> = []
async function setup() {
  const pinia = createPinia()
  setActivePinia(pinia)
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] } })
  const modal = mount(AppModal, { global: { plugins: [pinia] } })
  mounted.push(page, modal)
  await useAppStore().loadDataRootConfig()
  await flushPromises()
  return { page, modal, app: useAppStore() }
}
function switches() { return invoke.mock.calls.filter(call => call[0] === 'switch_data_root') }

describe('data root migration confirmation and restart boundary', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockImplementation(async (command: string) => command === 'get_data_root_config' ? { ...current } : null)
  })
  afterEach(() => { mounted.splice(0).forEach(wrapper => wrapper.unmount()) })

  it('renders an escaped captured target and cancellation never starts migration', async () => {
    const { page, modal, app } = await setup()
    const target = `/fixtures/<img src=x onerror="alert(1)"> & 'target'`
    await page.get('#switch-root-input').setValue(target)
    await page.get('[data-testid="data-root-migrate"]').trigger('click')
    expect(switches()).toHaveLength(0)
    expect(modal.get('.modal-body code').text()).toBe(target)
    expect(modal.find('img').exists()).toBe(false)
    expect(modal.text()).toContain('外部 OPENCODEX_HOME 保持原位')
    expect(modal.text()).toContain('旧目录会保留')
    app.resolveModal('cancel')
    await flushPromises()
    expect(switches()).toHaveLength(0)
    expect(modal.find('[role="dialog"]').exists()).toBe(false)
  })

  it('uses the confirmed target, shows real pending feedback and freezes actions until restart', async () => {
    let finish!: (value: unknown) => void
    invoke.mockImplementation((command: string) => command === 'switch_data_root'
      ? new Promise(resolve => { finish = resolve })
      : Promise.resolve(command === 'get_data_root_config' ? { ...current } : null))
    const { page, app } = await setup()
    await page.get('#switch-root-input').setValue('/fixtures/confirmed')
    await page.get('[data-testid="data-root-migrate"]').trigger('click')
    await page.get('#switch-root-input').setValue('/fixtures/edited-after-open')
    app.resolveModal('confirm')
    await flushPromises()
    expect(switches()).toEqual([['switch_data_root', { request: { targetPath: '/fixtures/confirmed', mode: 'migrate_data' } }]])
    const progress = page.get('[data-testid="data-root-progress"]')
    expect(progress.attributes('aria-busy')).toBe('true')
    expect(progress.attributes('aria-valuenow')).toBeUndefined()
    expect(page.get('[data-testid="data-root-migrate"]').attributes('disabled')).toBeDefined()
    finish({ status: 'restart_required', blocked: null, reconciliationRequired: false,
      config: { ...current, activeDataRoot: '/fixtures/confirmed', runtimeActive: false } })
    await flushPromises()
    expect(page.find('[data-testid="data-root-progress"]').exists()).toBe(false)
    expect(page.text()).toContain('当前服务仍使用上方当前路径')
    expect(app.dataRootConfig?.currentDataRoot).toBe(current.currentDataRoot)
    expect(app.dataRootConfig?.currentOpencodexHome).toBe(current.currentOpencodexHome)
    expect(page.get('[data-testid="data-root-migrate"]').attributes('disabled')).toBeDefined()
    const pathActions = page.findAll('button').filter(button => ['使用外部路径', '回到数据目录内', '校验并初始化', '仅切换引用'].includes(button.text()))
    expect(pathActions).toHaveLength(4)
    pathActions.forEach(button => expect(button.attributes('disabled')).toBeDefined())
  })

  it('presents uncertain binding as pending reconciliation instead of saved', async () => {
    invoke.mockImplementation(async (command: string) => command === 'switch_data_root'
      ? { status: 'restart_required', blocked: null, reconciliationRequired: true,
        config: { ...current, activeDataRoot: '/fixtures/intended', runtimeActive: false } }
      : command === 'get_data_root_config' ? { ...current } : null)
    const { page, app } = await setup()
    await page.get('#switch-root-input').setValue('/fixtures/intended')
    await page.get('[data-testid="data-root-migrate"]').trigger('click')
    app.resolveModal('confirm')
    await flushPromises()
    const pendingRow = page.findAll('tr').find(row => row.text().includes('数据目录 · 重启后生效'))!
    expect(pendingRow.text()).toContain('待核对')
    expect(pendingRow.text()).not.toContain('已保存')
    expect(page.findAll('[role="alert"]').some(node => node.text().includes('上方新位置为预期目标'))).toBe(true)
    expect(page.get('[data-testid="data-root-migrate"]').attributes('disabled')).toBeDefined()
  })

  it('keeps old paths and exposes quarantine warning when copying fails', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'switch_data_root') throw new Error('copy interrupted')
      return command === 'get_data_root_config' ? { ...current } : null
    })
    const { page, app } = await setup()
    await page.get('#switch-root-input').setValue('/fixtures/partial')
    await page.get('[data-testid="data-root-migrate"]').trigger('click')
    app.resolveModal('confirm')
    await flushPromises()
    expect(app.dataRootConfig).toEqual(current)
    expect(page.findAll('[role="alert"]').some(node => node.text().includes('隔离的未完成副本'))).toBe(true)
    expect(page.get('[data-testid="data-root-migrate"]').attributes('disabled')).toBeUndefined()
  })
})
