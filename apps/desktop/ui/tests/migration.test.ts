import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { exportMigration, importMigration } from '@/features/migration/api'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

const exportResult = {
  path: '/tmp/opencodex-data/exports/opencodex-config.ocxdconf',
  backupId: null,
  documentSha256: 'a'.repeat(64),
  formatVersion: 2,
  sections: ['extension_config.mcp', 'extension_config.skills', 'preferences'],
  excluded: ['系统钥匙串中的口令密文（WebDAV 口令、旧版加密口令）'],
}
const importResult = {
  backupId: 'bk_import_01',
  documentSha256: 'a'.repeat(64),
  formatVersion: 2,
  appliedSections: ['extension_config', 'preferences'],
  skippedSections: [{ section: 'asset_files', reason: '不在容器内' }],
  excluded: exportResult.excluded,
}

describe('migration contract and store gates', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  // 回归：新容器不再索要额外口令，导出请求不带任何参数。
  it('exports without any passphrase and imports without one by default', async () => {
    invoke.mockResolvedValueOnce(exportResult)
    invoke.mockResolvedValueOnce(importResult)
    await expect(exportMigration()).resolves.toEqual(exportResult)
    await expect(importMigration()).resolves.toEqual(importResult)
    expect(invoke).toHaveBeenNthCalledWith(1, 'export_migration')
    expect(invoke).toHaveBeenNthCalledWith(2, 'import_migration', { request: {} })
  })

  // 旧版加密容器才带口令；请求体只在这条路径上出现 passphrase。
  it('passes a passphrase only when the caller supplies one', async () => {
    invoke.mockResolvedValueOnce(importResult)
    await importMigration({ passphrase: 'legacy secret' })
    expect(invoke).toHaveBeenLastCalledWith('import_migration', {
      request: { passphrase: 'legacy secret' },
    })
  })

  it('exports and records the sanitized result without a passphrase value', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(exportResult)
    const result = await app.exportMigration()
    expect(result).toBe(true)
    expect(app.migrationLastExport).toEqual(exportResult)
    expect(app.migrationError).toBe('')
    expect(JSON.stringify(app.migrationLastExport)).not.toContain('password')
  })

  it('imports, reloads preferences, and preserves current state on failure', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(importResult)
    invoke.mockResolvedValueOnce({ interfaceScale: 175 })
    const success = await app.importMigration()
    expect(success).toBe(true)
    expect(app.migrationLastImport).toEqual(importResult)
    expect(app.preferences).toEqual({ interfaceScale: 175 })
    expect(invoke).toHaveBeenLastCalledWith('get_preferences')

    invoke.mockRejectedValueOnce(new Error('authentication failed'))
    const failed = await app.importMigration('wrong passphrase')
    expect(failed).toBe(false)
    expect(app.migrationError).toBe('配置导入失败；当前配置未修改。')
    expect(app.migrationLastImport).toEqual(importResult)
    expect(app.preferences).toEqual({ interfaceScale: 175 })
  })

  // 错误码 13 = 旧版加密容器需要原口令：前端单独提示，不混同普通失败。
  it('surfaces the legacy passphrase requirement distinctly', async () => {
    const app = useAppStore()
    invoke.mockRejectedValueOnce({ code: 13, message: 'legacy container requires the original passphrase' })
    const failed = await app.importMigration()
    expect(failed).toBe(false)
    expect(app.migrationError).toContain('原口令')
  })
})
