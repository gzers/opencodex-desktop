import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

import { createRestoreBackup, getRestoreRiskSummary } from '@/features/updates/upgrade'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'

const summary = {
  guidance: {
    runtimeState: 'at_risk',
    health: 'degraded',
    startupStatus: 'at-risk',
    protection: 'none',
    rebootSafe: false,
    servicePresent: false,
    shimPresent: false,
    versionDrift: '2.50.0',
    dataRoot: '/tmp/opencodex-data',
    opencodexHome: '/tmp/opencodex-home',
  },
  preferencesConfigured: true,
  extensionConfigured: true,
  syncEndpointConfigured: false,
  opencodexHome: '/tmp/opencodex-home',
}

const backup = {
  backupId: 'bk_restore_01',
  directory: '/tmp/opencodex-data/backups/upgrade/bk_restore_01',
  targetPath: '/tmp/opencodex-data/manager-state/preferences.json',
}

describe('official restore guidance', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    window.location.hash = '#overview'
  })

  it('serializes the frozen risk summary and backup contracts', async () => {
    invoke.mockResolvedValueOnce(summary)
    invoke.mockResolvedValueOnce(backup)
    await expect(getRestoreRiskSummary()).resolves.toEqual(summary)
    await expect(createRestoreBackup()).resolves.toEqual(backup)
    expect(invoke).toHaveBeenNthCalledWith(1, 'restore_risk_summary')
    expect(invoke).toHaveBeenNthCalledWith(2, 'create_restore_backup')
    expect(JSON.stringify(summary)).not.toContain('apiKey')
  })

  it('shows risks, creates the gate backup, then shows official-only guidance', async () => {
    const app = useAppStore()
    useAppController()
    invoke.mockResolvedValueOnce(summary)
    invoke.mockResolvedValueOnce(backup)
    await openRestore()
    expect(invoke).toHaveBeenNthCalledWith(1, 'restore_risk_summary')
    expect(app.modal?.title).toBe('恢复前确认')
    expect(app.modal?.body).toContain('启动状态：存在风险')
    expect(app.modal?.body).toContain('管理器偏好设置：已配置')
    expect(app.modal?.onConfirm).toBeTypeOf('function')
    app.modal?.onConfirm?.()
    await vi.waitFor(() => {
      expect(invoke).toHaveBeenNthCalledWith(2, 'create_restore_backup')
      expect(app.modal?.title).toBe('官方恢复引导')
    })
    expect(app.modal?.body).toContain('<code>ocx restore</code>')
    expect(app.modal?.body).toContain('本应用不会代为执行恢复')
    expect(app.restoreError).toBe('')
  })

  it('blocks the guidance when the summary is unavailable', async () => {
    const app = useAppStore()
    useAppController()
    invoke.mockRejectedValueOnce(new Error('unavailable'))
    await openRestore()
    expect(app.modal?.title).toBe('影响摘要不可用')
    expect(invoke).not.toHaveBeenCalledWith('create_restore_backup')
  })

  it('blocks guidance and preserves configuration when the gate backup fails', async () => {
    const app = useAppStore()
    useAppController()
    invoke.mockResolvedValueOnce(summary)
    invoke.mockRejectedValueOnce(new Error('backup failed'))
    await openRestore()
    app.modal?.onConfirm?.()
    await vi.waitFor(() => {
      expect(app.restoreError).toBe('restore 前备份失败；已保留当前配置。')
      expect(app.restoreBusy).toBe(false)
    })
    expect(app.modal?.title).toBe('恢复前确认')
    expect(app.modal?.body).not.toContain('<code>ocx restore</code>')
    expect(invoke).toHaveBeenCalledTimes(2)
  })

  async function openRestore() {
    const controller = useAppController()
    await controller.openRestoreConfirm('at_risk')
  }
})
