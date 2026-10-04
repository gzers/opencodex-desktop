import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getPreferences, type PreferencesDto } from '@/features/preferences/api'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
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

describe('preferences command contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('loads the frozen preference contract', async () => {
    const payload = dto({ interfaceScale: 150, cliEnabled: true })
    invoke.mockResolvedValue(payload)
    await expect(getPreferences()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('get_preferences')
  })

  it('saves and restores through the shared store', async () => {
    const app = useAppStore()
    invoke.mockResolvedValue(dto())
    await app.loadPreferences()
    expect(app.preferences).toEqual(dto())
    expect(app.preferencesError).toBe(false)

    const changed = dto({ interfaceScale: 200, launchMain: false })
    invoke.mockResolvedValueOnce(changed)
    await app.savePreferences(changed)
    expect(invoke).toHaveBeenLastCalledWith('save_preferences', { preferences: changed })
    expect(app.preferences).toEqual(changed)

    invoke.mockResolvedValueOnce(dto())
    await app.restorePreferences()
    expect(invoke).toHaveBeenLastCalledWith('restore_default_preferences')
    expect(app.preferences).toEqual(dto())
  })

  it('keeps the previous value when loading or saving fails', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(dto())
    await app.loadPreferences()

    invoke.mockRejectedValueOnce(new Error('backend unavailable'))
    await app.loadPreferences()
    expect(app.preferences).toEqual(dto())
    expect(app.preferencesError).toBe(true)

    invoke.mockRejectedValueOnce(new Error('write failed'))
    const saved = await app.savePreferences(dto({ interfaceScale: 75 }))
    expect(saved).toBe(false)
    expect(app.preferences).toEqual(dto())
    expect(app.preferencesError).toBe(true)
  })
})

// 回归：界面缩放此前只写进无人消费的 `--panel-zoom`，且只在打开设置页时生效；
// 现在偏好加载/保存后立即把 `--ui-zoom` 写到根元素，启动即生效、重启可恢复。
describe('interface scale application through the preference lifecycle', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    document.documentElement.style.removeProperty('--ui-zoom')
  })

  it('applies the stored scale as soon as preferences load', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(dto({ interfaceScale: 150 }))
    await app.loadPreferences()
    expect(document.documentElement.style.getPropertyValue('--ui-zoom')).toBe('1.5')
  })

  it('applies the persisted value returned by a save', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(dto())
    await app.loadPreferences()
    invoke.mockResolvedValueOnce(dto({ interfaceScale: 125 }))
    await app.savePreferences(dto({ interfaceScale: 125 }))
    expect(document.documentElement.style.getPropertyValue('--ui-zoom')).toBe('1.25')
  })

  it('keeps the last persisted scale when a save fails', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(dto({ interfaceScale: 150 }))
    await app.loadPreferences()
    invoke.mockRejectedValueOnce(new Error('write failed'))
    const saved = await app.savePreferences(dto({ interfaceScale: 200 }))
    expect(saved).toBe(false)
    expect(document.documentElement.style.getPropertyValue('--ui-zoom')).toBe('1.5')
  })

  it('setInterfaceScale previews immediately and persists the normalized value', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(dto())
    await app.loadPreferences()
    invoke.mockResolvedValueOnce(dto({ interfaceScale: 130 }))
    await app.setInterfaceScale(130.4)
    expect(invoke).toHaveBeenLastCalledWith('save_preferences', {
      preferences: dto({ interfaceScale: 130 }),
    })
    expect(document.documentElement.style.getPropertyValue('--ui-zoom')).toBe('1.3')
  })
})
