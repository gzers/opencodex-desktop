import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getPreferences, savePreferences, type PreferencesDto } from '@/features/preferences/api'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    interfaceScale: 100, launchMain: true, autoPanel: true, panelMode: 'embedded',
    keepProxyOnClose: true, lifecycleNotifications: true, syncConflictAlerts: true,
    launchWithCodex: true, autoBackupUpgrade: true, autoBackupImport: true,
    autoBackupSync: true, backupRetention: '10', backupIntegrity: 'sha-256',
    backupIncludeSkills: true, exportIncludeSkills: true,
    mcpConflictPolicy: 'ask', mcpMask: true, backupIncludeMcp: true,
    exportIncludeMcp: true, logRetention: '30d-10000', notificationRetention: '30',
    startupCleanup: true, cleanupBackupSummary: true, cliEnabled: false,
    syncConflictPolicy: 'ask', coldSync: true,
    backupBeforeOverwrite: true, appUpdateChannel: 'stable', appUpdateAutoCheck: true, appUpdateCheckIntervalSeconds: 86400, theme: 'system', visualEffects: 'high',
    glowRender: 'mesh', ...overrides,
  }
}

describe('CLI/IPC preferences and frozen contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('defaults to disabled and persists explicit enablement', async () => {
    expect(dto().cliEnabled).toBe(false)
    const app = useAppStore()
    invoke.mockResolvedValue(dto({ cliEnabled: true }))
    await app.loadPreferences()
    expect(invoke).toHaveBeenCalledWith('get_preferences')

    invoke.mockResolvedValueOnce(dto({ cliEnabled: true }))
    const next = dto({ cliEnabled: true })
    await app.savePreferences(next)
    expect(invoke).toHaveBeenLastCalledWith('save_preferences', { preferences: next })
    expect(app.preferences?.cliEnabled).toBe(true)
  })

  it('keeps command loaders bound to real preference commands', async () => {
    const payload = dto({ cliEnabled: true })
    invoke.mockResolvedValue(payload)
    await expect(getPreferences()).resolves.toEqual(payload)
    await expect(savePreferences(payload)).resolves.toEqual(payload)
    expect(invoke).toHaveBeenLastCalledWith('save_preferences', { preferences: payload })
  })
})
