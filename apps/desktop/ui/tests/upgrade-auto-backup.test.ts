import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'
import type { PreferencesDto } from '@/features/preferences/api'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: () => Promise.resolve(null),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    schemaVersion: 1,
    themeNeedsImport: false,
    networkProxyMode: 'none',
    networkProxyScheme: 'http',
    networkProxyHost: '',
    networkNoProxy: '',
    interfaceScale: 100,
    launchMain: true,
    autoPanel: false,
    panelMode: 'embedded',
    keepProxyOnClose: true,
    lifecycleNotifications: true,
    syncConflictAlerts: true,
    launchWithCodex: true,
    autoBackupUpgrade: true,
    autoBackupImport: true,
    autoBackupSync: true,
    backupRetention: '10',
    backupIntegrity: 'sha-256',
    backupIncludeSkills: true,
    exportIncludeSkills: true,
    mcpConflictPolicy: 'ask',
    mcpMask: true,
    backupIncludeMcp: true,
    exportIncludeMcp: true,
    logRetention: '30d-10000',
    notificationRetention: '30',
    startupCleanup: true,
    cleanupBackupSummary: true,
    cliEnabled: false,
    syncConflictPolicy: 'ask',
    coldSync: true,
    backupBeforeOverwrite: true,
    appUpdateChannel: 'stable', appUpdateAutoCheck: true, appUpdateCheckIntervalSeconds: 86400, theme: 'system',
    visualEffects: 'high',
    glowRender: 'mesh',
    ...overrides,
  }
}

async function flush(wrapper: VueWrapper) {
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
}

async function mountUpgrade(preferences: PreferencesDto) {
  invoke.mockImplementation((command: string) => {
    if (command === 'get_preferences') return Promise.resolve(preferences)
    if (command === 'create_upgrade_backup') {
      return Promise.resolve({ backupId: 'bk_test', directory: '/tmp/bk', targetPath: '/tmp/preferences.json' })
    }
    return Promise.resolve(undefined)
  })
  const pinia = createPinia()
  setActivePinia(pinia)
  const wrapper = mount(App, { global: { plugins: [pinia] } })
  useRouteStore().go('settings', { section: 'upgrade' })
  await flush(wrapper)
  return wrapper
}

describe('upgrade auto backup preference', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    window.location.hash = ''
  })

  // 回归：「升级前自动备份」此前没有任何消费方，进入升级引导不会生成备份。
  it('creates a backup before the upgrade guide when enabled', async () => {
    await mountUpgrade(dto({ autoBackupUpgrade: true }))
    expect(invoke).toHaveBeenCalledWith('create_upgrade_backup')
  })

  it('does not create a backup when disabled', async () => {
    await mountUpgrade(dto({ autoBackupUpgrade: false }))
    expect(invoke).not.toHaveBeenCalledWith('create_upgrade_backup')
  })
})
