import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createUpgradeBackup } from '@/features/updates/upgrade'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

const backupResult = {
  backupId: 'bk_20260917090000_abcdef01',
  directory: '/tmp/opencodex-data/backups/2026/09/upgrade/bk_20260917090000_abcdef01',
  targetPath: '/tmp/opencodex-data/manager-state/preferences.json',
}

describe('upgrade backup contract and store gates', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('invokes the frozen upgrade command without arguments', async () => {
    invoke.mockResolvedValueOnce(backupResult)
    await expect(createUpgradeBackup()).resolves.toEqual(backupResult)
    expect(invoke).toHaveBeenCalledWith('create_upgrade_backup')
  })

  it('creates a backup and records the sanitized result', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(backupResult)
    const busyBefore = app.upgradeBackupBusy
    const promise = app.createUpgradeBackup()
    expect(app.upgradeBackupBusy).toBe(true)
    await promise
    expect(busyBefore).toBe(false)
    expect(app.upgradeLastBackup).toEqual(backupResult)
    expect(app.upgradeBackupError).toBe('')
    expect(JSON.stringify(app.upgradeLastBackup)).not.toContain('interface_scale')
  })

  it('preserves the previous backup on failure and clears busy state', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(backupResult)
    await app.createUpgradeBackup()
    invoke.mockRejectedValueOnce(new Error('target not found'))
    await app.createUpgradeBackup()
    expect(app.upgradeLastBackup).toEqual(backupResult)
    expect(app.upgradeBackupError).toBe('升级前备份失败；当前配置未修改。')
    expect(app.upgradeBackupBusy).toBe(false)
  })
})
