// Explicit browser audit only. No IPC, storage, files or real service actions.
import { clientRecoveryApi, type ClientRecoveryPreview } from '@/features/runtime/client-recovery'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
type Scene = 'prepare' | 'prepare-slow' | 'prepare-error' | 'prepare-refresh-failed' | 'early' | 'early-slow' | 'early-error' | 'early-refresh-failed' | 'eligible' | 'continue' | 'continue-slow' | 'continue-error' | 'rollback' | 'rollback-slow' | 'rollback-error' | 'rollback-refresh-failed' | 'rearm' | 'rearm-slow' | 'rearm-error' | 'rolled-back' | 'pending' | 'unknown' | 'active' | 'none' | 'unsupported' | 'error' | 'slow' | 'refresh-failed'
export function installClientRecoveryFixture(initial: string) {
  let scene = initial as Scene, finalized = false, revision = initial.startsWith('rearm') ? 7 : 6
  let phase: 'restore_prepared'|'restore_rolled_back' = initial.startsWith('rearm') ? 'restore_rolled_back' : 'restore_prepared'
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
    if (scene === 'error' || scene==='rollback-refresh-failed'&&phase==='restore_rolled_back') throw new Error('模拟读取失败，请重新检查。')
    const value: ClientRecoveryPreview = { status: 'pending', phase, revision,
      attemptId: 'f394bd75-355b-4873-83cc-6818efeb547a', originalEvidence: 'verified', planEvidence: 'verified',
      completionEvidence: 'unregistered', phaseCount: 3, canContinue: false, canRollback: false, canFinalize: true, canRearm:false,canFinishUnchanged:false,canCancelPreparation:false,preparationDigest:null,unchangedDigest:null,rollbackDigest:null,rearmDigest:null, confirmationDigest: 'a'.repeat(64) }
    if (scene === 'pending') { value.canFinalize = false; value.confirmationDigest = null; value.completionEvidence = 'not_recorded' }
    if(scene.startsWith('early'))Object.assign(value,{phase:'captured',revision:2,phaseCount:0,planEvidence:'not_recorded',completionEvidence:'not_recorded',canFinalize:false,confirmationDigest:null,canFinishUnchanged:true,unchangedDigest:'e'.repeat(64)})
    if(scene.startsWith('prepare'))Object.assign(value,{phase:'preparing',revision:1,phaseCount:0,originalEvidence:'not_recorded',planEvidence:'not_recorded',completionEvidence:'not_recorded',canFinalize:false,confirmationDigest:null,canCancelPreparation:true,preparationDigest:'f'.repeat(64)})
    if(scene==='rolled-back'){value.phase='restore_rolled_back';value.canFinalize=false;value.confirmationDigest=null;value.completionEvidence='verified'}
    if (scene.startsWith('continue')||scene.startsWith('rollback')||scene.startsWith('rearm')) {value.canContinue=true;value.canRollback=true;value.rollbackDigest='b'.repeat(64);value.canFinalize=false;value.completionEvidence='not_recorded'}
    if(phase==='restore_rolled_back'){value.canContinue=false;value.canRollback=false;value.canFinalize=false;value.canRearm=true;value.confirmationDigest=null;value.rollbackDigest=null;value.rearmDigest='c'.repeat(64);value.completionEvidence='verified'}
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
  clientRecoveryApi.rollback=async request=>{
    calls.push('rollback');await new Promise(resolve=>setTimeout(resolve,scene==='rollback-slow'?1200:200))
    if(scene==='rollback-error')throw new Error('撤销恢复未确认完成；原件与当前文件保留。')
    phase='restore_rolled_back';revision=request.revision+1
    return {attemptId:request.attemptId,revision,status:'rolled_back'}
  }
  clientRecoveryApi.rearm=async request=>{
    calls.push('rearm');await new Promise(resolve=>setTimeout(resolve,scene==='rearm-slow'?1200:200))
    if(scene==='rearm-error')throw new Error('恢复准备未确认完成；原件与当前文件保留。')
    phase='restore_prepared';revision=request.revision+1
    return {attemptId:request.attemptId,revision,status:'restore_prepared'}
  }
  clientRecoveryApi.status = async () => {
    calls.push('status')
    if ((scene === 'refresh-failed'||scene==='early-refresh-failed'||scene==='prepare-refresh-failed') && finalized) throw new Error('mock status unavailable')
    return snapshot()
  }
  clientRecoveryApi.finishUnchanged=async request=>{
    calls.push('finishUnchanged');await new Promise(resolve=>setTimeout(resolve,scene==='early-slow'?1200:200))
    if(scene==='early-error')throw new Error('原件或停止状态尚未核实；原件与现场保留。')
    finalized=true;revision=request.revision+1
    return {attemptId:request.attemptId,revision,status:'unchanged'}
  }
  clientRecoveryApi.cancelPreparation=async request=>{
    calls.push('cancelPreparation');await new Promise(resolve=>setTimeout(resolve,scene==='prepare-slow'?1200:200))
    if(scene==='prepare-error')throw new Error('启动准备或停止状态尚未核实；原现场保留。')
    finalized=true;revision=request.revision+1
    return {attemptId:request.attemptId,revision,status:'cancelled_before_capture'}
  }
  const api = { calls, setScene(value: Scene) { scene = value },
    state: () => ({ scene, finalized,phase,revision, blocked: useLifecycleStore().clientRecoveryRequired }) }
  // Tests use a fresh browser page per scene. Progress store owns active work.
  Object.assign(window, { __clientRecoveryFixture: api })
}
