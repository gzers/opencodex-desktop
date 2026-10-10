import { mount, flushPromises } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import BackupManagement from '../src/features/backup/BackupManagement.vue'
const mocks = vi.hoisted(() => ({ listBackups: vi.fn(), getBackupPolicy: vi.fn(), createBackup: vi.fn(), saveBackupPolicy: vi.fn(), pinBackup: vi.fn(), previewCleanup: vi.fn(), executeCleanup: vi.fn(), restoreBackup: vi.fn(), openBackupFile: vi.fn(), openModal: vi.fn(), loadPreferences: vi.fn(), showToast: vi.fn(), getUpdateStatus: vi.fn() }))
vi.mock('../src/features/backup/api', () => mocks)
vi.mock('../src/stores/app', () => ({ useAppStore: () => mocks }))
vi.mock('../src/features/updates/update', () => ({ getUpdateStatus: mocks.getUpdateStatus }))
const policy = { schema_version: 1, mode: 'manual', keep_recent: 10, keep_days: 30 }
const preview = { token: 'owned-preview', createdAt: '2026-10-10', candidateIds: ['old'], candidateBytes: 500, retainedCount: 10, keepRecent: 10, keepDays: 30 }
async function setup() { const w = mount(BackupManagement, { global: { stubs: { BackupFiles: true } } }); await flushPromises(); return w }
function button(w: Awaited<ReturnType<typeof setup>>, label: string) { const b = w.findAll('button').find(b => b.text() === label); if (!b) throw Error(label); return b }
describe('backup transactions', () => {
  beforeEach(() => {
    Object.values(mocks).forEach(m => m.mockReset())
    mocks.listBackups.mockResolvedValue([{ id: 'bk_actual', action: 'manual-preferences', createdAt: '2026-10-10T00:00:00Z', bytes: 500, pinned: false, protected: false, preferencesCandidate: true, integrityVerified: false }])
    mocks.getBackupPolicy.mockResolvedValue(policy)
    mocks.previewCleanup.mockResolvedValue(preview)
    mocks.executeCleanup.mockResolvedValue(['old'])
    mocks.createBackup.mockResolvedValue({ cleanupError: null })
  })
  it('mount only reads metadata and never creates or deletes', async () => {
    const w = await setup()
    expect(w.text()).toContain('恢复前重新校验')
    expect(mocks.createBackup).not.toHaveBeenCalled()
    expect(mocks.previewCleanup).not.toHaveBeenCalled()
    expect(mocks.executeCleanup).not.toHaveBeenCalled()
  })
  it('saving automatic policy does not clean', async () => {
    const w = await setup()
    await w.get('select').setValue('automatic')
    await button(w, '保存策略').trigger('click'); await flushPromises()
    expect(mocks.saveBackupPolicy).toHaveBeenCalledWith({ ...policy, mode: 'automatic' })
    expect(mocks.executeCleanup).not.toHaveBeenCalled()
    expect(mocks.createBackup).not.toHaveBeenCalled()
  })
  it('cleanup requires confirmation and sends the exact preview', async () => {
    const w = await setup()
    await button(w, '预览清理').trigger('click'); await flushPromises()
    expect(mocks.executeCleanup).not.toHaveBeenCalled()
    mocks.openModal.mock.calls[0][0].onConfirm(); await flushPromises()
    expect(mocks.executeCleanup).toHaveBeenCalledWith(preview)
  })
  it('preview failure neither deletes nor leaks native error', async () => {
    mocks.previewCleanup.mockRejectedValue(new Error('secret/raw/path'))
    const w = await setup()
    await button(w, '预览清理').trigger('click'); await flushPromises()
    expect(mocks.executeCleanup).not.toHaveBeenCalled()
    expect(w.text()).toContain('未删除任何备份')
    expect(w.text()).not.toContain('secret/raw/path')
  })
  it('restores only after confirmation and refreshes stores after commit', async () => {
    const w = await setup()
    await button(w, '恢复').trigger('click')
    expect(mocks.restoreBackup).not.toHaveBeenCalled()
    mocks.openModal.mock.calls[0][0].onConfirm(); await flushPromises()
    expect(mocks.restoreBackup).toHaveBeenCalledWith('bk_actual')
    expect(mocks.restoreBackup.mock.invocationCallOrder[0]).toBeLessThan(mocks.loadPreferences.mock.invocationCallOrder[0])
    expect(mocks.loadPreferences.mock.invocationCallOrder[0]).toBeLessThan(mocks.getUpdateStatus.mock.invocationCallOrder[0])
  })
  it('restore failure does not reload preferences', async () => {
    mocks.restoreBackup.mockRejectedValue(new Error('bad hash'))
    const w = await setup(); await button(w, '恢复').trigger('click')
    mocks.openModal.mock.calls[0][0].onConfirm(); await flushPromises()
    expect(mocks.loadPreferences).not.toHaveBeenCalled()
    expect(mocks.getUpdateStatus).not.toHaveBeenCalled()
  })
  it('reports committed restoration even when UI reload fails', async () => {
    mocks.loadPreferences.mockRejectedValue(new Error('reload'))
    const w = await setup(); await button(w, '恢复').trigger('click')
    mocks.openModal.mock.calls[0][0].onConfirm(); await flushPromises()
    expect(w.text()).toContain('偏好已恢复；界面刷新未完成')
  })
  it('distinguishes a committed restore from a native projection failure', async () => {
    mocks.restoreBackup.mockResolvedValue({ refreshRequired: true })
    const w = await setup(); await button(w, '恢复').trigger('click')
    mocks.openModal.mock.calls[0][0].onConfirm(); await flushPromises()
    expect(w.text()).toContain('偏好已恢复；运行状态刷新未完成')
    expect(mocks.loadPreferences).not.toHaveBeenCalled()
    expect(mocks.getUpdateStatus).not.toHaveBeenCalled()
  })
  it('backup remains successful when automatic cleanup fails', async () => {
    mocks.createBackup.mockResolvedValue({ cleanupError: 'protected' })
    const w = await setup(); await button(w, '生成备份').trigger('click'); await flushPromises()
    expect(w.text()).toContain('备份已生成；自动清理未完成')
  })
})
