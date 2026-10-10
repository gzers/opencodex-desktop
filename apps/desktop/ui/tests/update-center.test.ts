import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import UpdateCenter from '@/features/updates/UpdateCenter.vue'
import { managerUpdateStatus, managerInstallProgress, type UpdateProgress } from '@/features/updates/update'
import { useAppStore } from '@/stores/app'
import { useUpdatesStore } from '@/features/updates/store'
import { useRuntimeStore } from '@/features/runtime/store'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), install: vi.fn(), restart: vi.fn(), check: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mocks.invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@/features/updates/update', async original => ({
  ...await original<typeof import('@/features/updates/update')>(),
  installUpdate: (...args: unknown[]) => mocks.install(...args),
  restartAfterUpdate: () => mocks.restart(),
  checkForUpdate: () => mocks.check(),
}))
Object.defineProperty(window, 'matchMedia', { writable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
let wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
function render(props = {}) {
  const wrapper = mount(UpdateCenter, { props: { open: true, ...props }, global: { plugins: [pinia], stubs: { teleport: true } }, attachTo: document.body })
  wrappers.push(wrapper)
  return wrapper
}
function button(wrapper: VueWrapper, label: string) {
  const found = wrapper.findAll('button').find(item => item.text() === label)
  expect(found, label).toBeDefined()
  return found!
}
function progress(overrides: Partial<UpdateProgress> = {}): UpdateProgress {
  return { operationId: 'op-1', target: 'manager', channel: 'stable', candidateVersion: '0.1.10', generation: 1, sequence: 1, stage: 'downloading', downloadedBytes: 50, totalBytes: null, ...overrides }
}
beforeEach(() => {
  vi.clearAllMocks()
  mocks.install.mockResolvedValue(undefined)
  mocks.restart.mockResolvedValue(undefined)
  mocks.check.mockResolvedValue(undefined)
  pinia = createPinia(); setActivePinia(pinia)
  managerUpdateStatus.value = { channel: 'stable', currentVersion: '0.1.9', availableVersion: '0.1.10', lastCheckedAt: '2026-10-10', signatureVerified: false, error: null }
  managerInstallProgress.value = null
  mocks.invoke.mockImplementation(async command => command === 'get_update_status' ? managerUpdateStatus.value : null)
  const updates = useUpdatesStore()
  updates.officialProject = { displayName: 'OpenCodex', version: '2.50.0', rawVersion: '2.50.0', truncated: false }
  updates.officialRemote = { tag: 'latest', version: '2.51.0', integrity: null }
})
afterEach(() => { wrappers.forEach(w => w.unmount()); wrappers = []; document.body.innerHTML = ''; vi.restoreAllMocks() })

describe('shared production update center', () => {
  it('opens without a network check, backup or installation and presents both objects', async () => {
    const wrapper = render(); await flushPromises()
    expect(wrapper.find('[data-testid="manager-update-card"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="official-update-card"]').exists()).toBe(true)
    expect(wrapper.get('input[type="checkbox"]').element).toHaveProperty('checked', true)
    expect(mocks.check).not.toHaveBeenCalled()
    expect(mocks.install).not.toHaveBeenCalled()
    expect(mocks.invoke.mock.calls.some(call => call[0] === 'create_upgrade_backup')).toBe(false)
    await button(wrapper, '重新检查更新').trigger('click'); await flushPromises()
    expect(mocks.check).toHaveBeenCalledOnce()
  })
  it('renders remote notes as text and only opens validated HTTPS links', async () => {
    managerUpdateStatus.value = { ...managerUpdateStatus.value!, notes: '<img src=x onerror=alert(1)>', releaseUrl: 'https://example.com/releases/0.1.10', publishedAt: '2026-10-10' }
    const open = vi.spyOn(useAppStore(), 'openExternalLink').mockResolvedValue(true)
    const wrapper = render({ target: 'manager' }); await flushPromises()
    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.get('.update-notes').text()).toContain('<img src=x onerror=alert(1)>')
    await button(wrapper, '查看网页详情 ↗').trigger('click')
    expect(open).toHaveBeenCalledWith('https://example.com/releases/0.1.10')
    for (const releaseUrl of ['javascript:alert(1)', 'http://example.com', 'https://user:secret@example.com', String.raw`https://example.com\evil`, 'file:///tmp/release', 'https://']) {
      managerUpdateStatus.value = { ...managerUpdateStatus.value!, releaseUrl }; await wrapper.vm.$nextTick()
      expect(wrapper.text()).not.toContain('查看网页详情 ↗')
    }
  })
  it('locks the confirmed manager candidate and backup choice, and observes byte progress after remount', async () => {
    let finish!: () => void
    mocks.install.mockImplementationOnce(() => new Promise<void>(resolve => { finish = resolve }))
    const wrapper = render(); await flushPromises()
    await wrapper.get('input[type="checkbox"]').setValue(false)
    await wrapper.get('[data-testid="install-manager-update"]').trigger('click')
    expect(mocks.install).toHaveBeenCalledWith({ candidateVersion: '0.1.10', channel: 'stable', backup: false })
    managerInstallProgress.value = progress()
    await wrapper.setProps({ open: false })
    wrapper.unmount(); wrappers = []
    const reopened = render(); await flushPromises()
    expect(reopened.get('progress[aria-label="管理器更新进度"]').attributes('value')).toBeUndefined()
    expect(reopened.get('[data-testid="install-manager-update"]').attributes('disabled')).toBeDefined()
    managerInstallProgress.value = progress({ totalBytes: 200 }); await reopened.vm.$nextTick()
    expect(reopened.get('progress[aria-label="管理器更新进度"]').attributes('value')).toBe('25')
    expect(reopened.text()).toContain('25%')
    managerInstallProgress.value = progress({ stage: 'pending_restart' }); finish(); await flushPromises()
    expect(reopened.text()).toContain('更新已准备就绪；请确认重启并应用。')
    expect(reopened.text()).not.toContain('已安装')
    expect(mocks.restart).not.toHaveBeenCalled()
    await button(reopened, '重启并应用更新').trigger('click'); await flushPromises()
    expect(mocks.restart).toHaveBeenCalledOnce()
  })
  it('permits retry after an actual manager failure without inventing completion', async () => {
    mocks.install.mockImplementationOnce(async () => {
      managerInstallProgress.value = progress()
      throw new Error('download interrupted')
    })
    const wrapper = render(); await flushPromises()
    await wrapper.get('[data-testid="install-manager-update"]').trigger('click'); await flushPromises()
    expect(wrapper.text()).toContain('download interrupted')
    expect(wrapper.find('progress[aria-label="管理器更新进度"]').exists()).toBe(false)
    expect(button(wrapper, '重试安装').attributes('disabled')).toBeUndefined()
    await button(wrapper, '重试安装').trigger('click'); await flushPromises()
    expect(mocks.install).toHaveBeenCalledTimes(2)
    expect(mocks.restart).not.toHaveBeenCalled()
  })
  it('requires a successful fresh panel backup and retains the confirmed version through that await', async () => {
    const updates = useUpdatesStore()
    const backup = vi.spyOn(updates, 'createUpgradeBackup').mockResolvedValueOnce(null)
    const apply = vi.spyOn(useAppStore(), 'applyOfficialUpdate').mockResolvedValue(true)
    const wrapper = render({ target: 'official' }); await flushPromises()
    await wrapper.get('[data-testid="install-official-update"]').trigger('click'); await flushPromises()
    expect(apply).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('备份失败')
    let finish!: (value: { backupId: string; directory: string; targetPath: string }) => void
    backup.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    useRuntimeStore().install = { phase: 'installing', percent: 65, line: null }
    useRuntimeStore().installLines = ['previous install output']
    await wrapper.get('[data-testid="install-official-update"]').trigger('click')
    expect(wrapper.text()).toContain('正在备份配置')
    expect(wrapper.text()).not.toContain('previous install output')
    updates.officialRemote!.version = '2.52.0'
    finish({ backupId: 'fresh', directory: '/backup', targetPath: '/config' }); await flushPromises()
    expect(apply).toHaveBeenCalledWith('2.51.0')
    expect(backup).toHaveBeenCalledTimes(2)
  })
  it('uses indeterminate panel progress despite the runtime phase percentage', async () => {
    useUpdatesStore().officialUpdateBusy = true
    useRuntimeStore().install = { phase: 'installing', percent: 65, line: null }
    const wrapper = render({ target: 'official' }); await flushPromises()
    expect(wrapper.get('progress[aria-label="面板更新进度"]').attributes('value')).toBeUndefined()
    expect(wrapper.text()).not.toContain('65%')
  })
  it('traps keyboard focus and restores the opener when hidden', async () => {
    const opener = document.createElement('button'); document.body.append(opener); opener.focus()
    const wrapper = render(); await flushPromises()
    expect(document.activeElement).toBe(wrapper.get('[role="dialog"]').element)
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }))
    expect(document.activeElement).toBe(wrapper.findAll('button')[0].element)
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(wrapper.emitted('update:open')).toEqual([[false]])
    await wrapper.setProps({ open: false })
    expect(document.activeElement).toBe(opener)
  })
  it('settings reuses the component and does not back up merely by entering', async () => {
    useRouteStore().go('settings', { section: 'upgrade' })
    const backup = vi.spyOn(useUpdatesStore(), 'createUpgradeBackup')
    const wrapper = mount(SettingsRoute, { global: { plugins: [pinia], stubs: { teleport: true } } }); wrappers.push(wrapper)
    await flushPromises()
    expect(backup).not.toHaveBeenCalled()
    await button(wrapper, '查看管理器更新').trigger('click'); await flushPromises()
    expect(wrapper.findComponent(UpdateCenter).props('open')).toBe(true)
    expect(wrapper.get('[role="dialog"]').text()).toContain('桌面管理器更新')
  })
})
