import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'
import type { PreferencesDto } from '@/features/preferences/api'
import type { SyncConfigDto, SyncConflictPolicy } from '@/features/sync/api'

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
    interfaceScale: 100,
    launchMain: true,
    autoPanel: true,
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

function endpointDto(conflictPolicy: SyncConflictPolicy): SyncConfigDto {
  return {
    endpoint: {
      endpointId: 'default',
      url: 'https://dav.example.test',
      remotePath: 'desktop-sync/current',
      username: 'user',
      credentialRefId: 'cred_12345678-1234-1234-1234-123456789abc',
      tlsPolicy: 'verify_required',
      payloadProtection: 'integrity_only',
      conflictPolicy,
    },
  }
}

async function flush(wrapper: VueWrapper) {
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
}

async function mountSyncSection(preferences: PreferencesDto, config: SyncConfigDto = { endpoint: null }) {
  invoke.mockImplementation((command: string) => {
    if (command === 'get_preferences') return Promise.resolve(preferences)
    if (command === 'get_sync_config') return Promise.resolve(config)
    return Promise.resolve(undefined)
  })
  const pinia = createPinia()
  setActivePinia(pinia)
  const wrapper = mount(App, { global: { plugins: [pinia] } })
  useRouteStore().go('settings', { section: 'sync' })
  await flush(wrapper)
  return wrapper
}

function policyField(wrapper: VueWrapper) {
  const el = wrapper.find('[aria-label="冲突策略"]')
  return {
    exists: el.exists(),
    readonly: el.attributes('readonly'),
    value: el.exists() ? (el.element as HTMLInputElement).value : undefined,
    hasSelect: wrapper.find('[aria-label="冲突策略"] .select-value').exists(),
  }
}

// 回归：冲突策略四档静默语义从未在同步引擎落码（modules/sync/{engine,runner,mod}.rs 都不读它），
// 界面此前把它渲染成可选下拉＝把无效果的承诺摆到用户面前（§26.1）。现固定为只读「每次询问」，
// 用户已确认按「每次询问」处理并同步修改原型。
describe('sync conflict policy is fixed to 每次询问', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    window.location.hash = ''
  })

  it('renders a read-only 每次询问 for a new endpoint even if the preference says keep-remote', async () => {
    const wrapper = await mountSyncSection(dto({ syncConflictPolicy: 'keep-remote' }))
    const field = policyField(wrapper)
    expect(field.exists).toBe(true)
    expect(field.readonly).toBeDefined()
    expect(field.value).toBe('每次询问')
    expect(field.hasSelect).toBe(false)
  })

  it('renders a read-only 每次询问 even when a stored endpoint says keep-remote', async () => {
    const wrapper = await mountSyncSection(dto(), endpointDto('keep-remote'))
    const field = policyField(wrapper)
    expect(field.readonly).toBeDefined()
    expect(field.value).toBe('每次询问')
    expect(field.hasSelect).toBe(false)
  })
})
