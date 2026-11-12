// Explicit browser audit only. No IPC, storage, files or real service actions.
import { clientRecoveryApi, type ClientRecoveryPreview } from '@/features/runtime/client-recovery'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
type Scene = 'eligible' | 'continue' | 'continue-slow' | 'continue-error' | 'pending' | 'unknown' | 'active' | 'none' | 'unsupported' | 'error' | 'slow' | 'refresh-failed'
export function installClientRecoveryFixture(initial: string) {
  let scene = initial as Scene, finalized = false
  const calls: string[] = []
  const data = '/fixtures/client-recovery/desktop', home = '/fixtures/client-recovery/home'
  const snapshot = (): StatusSnapshot => ({ source: 'live', matrix: { runtime: 'stopped', connection: 'unconfigured', operation: 'idle' },
    facts: { client_recovery: finalized ? 'not_required' : 'required', data_root: data, opencodex_home: home, health: 'unknown',
      runtime_label: 'bundled', protection: null, reboot_safe: true, service_present: null, shim_present: null, version_drift: null },
    pid: null, port: null, can_start: finalized, can_stop: false, can_restart: false })
  useLifecycleStore().setStatusSnapshot(snapshot())
  clientRecoveryApi.preview = async () => {
    calls.push('preview')
    await new Promise(resolve => setTimeout(resolve, 100))
    if (scene === 'error') throw new Error('模拟读取失败，请重新检查。')
    const value: ClientRecoveryPreview = { status: 'pending', phase: 'restore_prepared', revision: 6,
      attemptId: 'f394bd75-355b-4873-83cc-6818efeb547a', originalEvidence: 'verified', planEvidence: 'verified',
      completionEvidence: 'unregistered', phaseCount: 3, canContinue: false, canRollback: false, canFinalize: true, confirmationDigest: 'a'.repeat(64) }
    if (scene === 'pending') { value.canFinalize = false; value.confirmationDigest = null; value.completionEvidence = 'not_recorded' }
    if (scene.startsWith('continue')) {value.canContinue=true;value.canFinalize=false;value.completionEvidence='not_recorded'}
    if (scene === 'unknown' || scene === 'unsupported') { value.status = scene; value.phase = null; value.attemptId = null; value.revision = null; value.canFinalize = false; value.confirmationDigest = null }
    if (scene === 'active') { value.status = 'active'; value.phase = 'ready'; value.canFinalize = false; value.confirmationDigest = null }
    if (scene === 'none') { value.status = 'not_required'; value.phase = null; value.canFinalize = false; value.confirmationDigest = null }
    return value
  }
  clientRecoveryApi.finalize = async request => {
    calls.push('finalize')
    await new Promise(resolve => setTimeout(resolve, scene === 'slow' ? 1200 : 200))
    finalized = true
    return { attemptId: request.attemptId, revision: request.revision + 1, status: 'restored' }
  }
  clientRecoveryApi.continue = async request => {
    calls.push('continue')
    await new Promise(resolve=>setTimeout(resolve,scene==='continue-slow'?1200:200))
    if(scene==='continue-error')throw new Error('恢复现场已变化，请重新检查。')
    finalized=true
    return {attemptId:request.attemptId,revision:request.revision+1,status:'restored'}
  }
  clientRecoveryApi.status = async () => {
    calls.push('status')
    if (scene === 'refresh-failed' && finalized) throw new Error('mock status unavailable')
    return snapshot()
  }
  const api = { calls, setScene(value: Scene) { scene = value },
    state: () => ({ scene, finalized, blocked: useLifecycleStore().clientRecoveryRequired }) }
  // Tests use a fresh browser page per scene. Progress store owns active work.
  Object.assign(window, { __clientRecoveryFixture: api })
}
