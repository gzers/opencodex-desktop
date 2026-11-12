import { beforeEach, expect, it, vi } from 'vitest'
import { clientRecoveryApi, type ClientRecoveryPreview } from '@/features/runtime/client-recovery'
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
beforeEach(() => { invoke.mockReset() })
it('observes the current bound session without supplying paths, credentials or a recovery decision', async () => {
  const preview: ClientRecoveryPreview = {
    status: 'pending', phase: 'restore_prepared', revision: 6, attemptId: 'original-attempt',
    originalEvidence: 'verified', planEvidence: 'verified', completionEvidence: 'unregistered',
    phaseCount: 3, canContinue: false, canRollback: false,
  }
  invoke.mockResolvedValue(preview)
  expect(await clientRecoveryApi.preview()).toEqual(preview)
  expect(invoke.mock.calls).toEqual([['client_recovery_preview']])
})
it('does not turn an IPC failure into absent recovery or dispatch a retry/mutation', async () => {
  const error = new Error('observation unavailable')
  invoke.mockRejectedValue(error)
  await expect(clientRecoveryApi.preview()).rejects.toBe(error)
  expect(invoke).toHaveBeenCalledTimes(1)
})
