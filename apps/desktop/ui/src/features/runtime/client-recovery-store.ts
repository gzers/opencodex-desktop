import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { clientRecoveryApi, type ClientRecoveryPreview, type ClientRecoveryOutcome, type ClientRecoveryAction } from './client-recovery'
import { runtimeErrorText } from './errors'
import { useLifecycleStore } from '@/app/lifecycle/store'
const hash = (value: unknown): value is string => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value)

export const useClientRecoveryStore = defineStore('client-recovery', () => {
  const open = ref(false), reading = ref(false), executing = ref(false), refreshing = ref(false)
  const checked = ref(false), error = ref(''), refreshError = ref('')
  const preview = shallowRef<ClientRecoveryPreview | null>(null)
  const outcome = shallowRef<ClientRecoveryOutcome | null>(null)
  const ready = ref(false), reconfirmed = ref(false)
  const action = ref<ClientRecoveryAction>('finalize')
  const choice = ref<ClientRecoveryAction | null>(null)
  const continuing = computed(() => action.value === 'continue')
  const bound = computed(() => {
    const p = preview.value
    return p?.status === 'pending' && p.originalEvidence === 'verified' && p.planEvidence === 'verified'
      && !!p.attemptId && Number.isSafeInteger(p.revision) && p.revision! > 0
  })
  const canFinalize = computed(() => {
    const p = preview.value
    return bound.value && (p?.phase === 'restore_prepared'||p?.phase==='recovery_prepared'||p?.phase==='root_retirement_prepared') && p.canFinalize && !p.canContinue && !p.canRollback && !p.canRearm && !p.canPreparePlan && !p.canFinishUnchanged && !p.canCancelPreparation
      && (p.completionEvidence === 'not_recorded' || p.completionEvidence === 'unregistered') && hash(p.confirmationDigest) && (p.phase==='recovery_prepared'?[1,2].includes(p.phaseCount):p.phaseCount===3)
      && (p.phase !== 'root_retirement_prepared' || p.completionEvidence === 'unregistered')
  })
  const canContinue = computed(() => {
    const p = preview.value
    return bound.value && (p?.phase === 'restore_prepared'||p?.phase==='recovery_prepared') && p.canContinue && !p.canFinalize && !p.canRearm && !p.canPreparePlan && !p.canFinishUnchanged && !p.canCancelPreparation
      && p.completionEvidence === 'not_recorded' && hash(p.confirmationDigest) && (p.phase==='recovery_prepared'?[1,2].includes(p.phaseCount):p.phaseCount===3)
      && (!p.canRollback || hash(p.rollbackDigest) && p.rollbackDigest !== p.confirmationDigest)
  })
  const canRollback = computed(() => {
    const p = preview.value
    return bound.value && (p?.phase === 'restore_prepared'||p?.phase==='recovery_prepared') && p.canRollback && !p.canFinalize && !p.canRearm && !p.canPreparePlan && !p.canFinishUnchanged && !p.canCancelPreparation
      && p.completionEvidence === 'not_recorded' && hash(p.rollbackDigest) && (p.phase==='recovery_prepared'?[1,2].includes(p.phaseCount):p.phaseCount===3)
      && (!p.canContinue || hash(p.confirmationDigest) && p.rollbackDigest !== p.confirmationDigest)
  })
  const canRearm = computed(() => {
    const p = preview.value
    return bound.value && p?.phase === 'restore_rolled_back' && p.canRearm && !p.canPreparePlan && !p.canContinue && !p.canFinalize && !p.canRollback && !p.canFinishUnchanged && !p.canCancelPreparation
      && p.completionEvidence === 'verified' && [1,2,3].includes(p.phaseCount) && hash(p.rearmDigest)
  })
  const canFinishUnchanged = computed(() => {
    const p=preview.value
    return p?.status==='pending' && ['captured','ready','stop_prepared','stopped','failed'].includes(p.phase ?? '')
      && p.canFinishUnchanged && !p.canCancelPreparation && !p.canContinue && !p.canRollback && !p.canFinalize && !p.canRearm && !p.canPreparePlan
      && p.originalEvidence==='verified' && p.planEvidence==='not_recorded'
      && (p.completionEvidence==='not_recorded'||p.completionEvidence==='unregistered')
      && !!p.attemptId && Number.isSafeInteger(p.revision) && p.revision!>0 && hash(p.unchangedDigest)
  })
  const canCancelPreparation=computed(()=>{
    const p=preview.value
    return p?.status==='pending'&&(p.phase==='preparing'||p.phase==='capture_reserved')&&p.canCancelPreparation
      && (p.preparationKind==null||p.preparationKind==='created_root'&&p.phase==='preparing'||p.preparationKind==='created_root_capture'&&p.phase==='capture_reserved')
      && !p.canContinue&&!p.canRollback&&!p.canFinalize&&!p.canRearm && !p.canPreparePlan&&!p.canFinishUnchanged
      && p.originalEvidence==='not_recorded'&&p.planEvidence==='not_recorded'&&p.completionEvidence==='not_recorded'&&p.phaseCount===0
      && !!p.attemptId&&Number.isSafeInteger(p.revision)&&p.revision!>0&&hash(p.preparationDigest)
  })
  const canPreparePlan=computed(()=>{
    const p=preview.value
    return p?.status==='pending'&&(p.phase==='stopped'||p.phase==='stop_prepared'||p.phase==='ready')&&p.canPreparePlan&&p.originalEvidence==='verified'
      && p.planEvidence==='not_recorded'&&p.completionEvidence==='not_recorded'&&p.phaseCount===(p.phase==='ready'?1:p.phase==='stop_prepared'?2:3)
      && !p.canContinue&&!p.canRollback&&!p.canFinalize&&!p.canRearm&&!p.canFinishUnchanged&&!p.canCancelPreparation
      && !!p.attemptId&&Number.isSafeInteger(p.revision)&&p.revision!>0&&hash(p.planDigest)
  })
  const can = (value: ClientRecoveryAction) => ({ finalize: canFinalize.value, continue: canContinue.value, rollback: canRollback.value, rearm: canRearm.value, finishUnchanged: canFinishUnchanged.value, cancelPreparation: canCancelPreparation.value, preparePlan: canPreparePlan.value })[value]
  let generation = 0
  let completionScope: { dataRoot: string; home: string } | null = null
  function verifiedClosedRoot(p: ClientRecoveryPreview): boolean {
    return p.status==='restored' && (p.phase==='root_cancellation_absent'||p.phase==='root_absent')
      && !!p.attemptId && Number.isSafeInteger(p.revision) && p.revision!>0
      && p.planEvidence==='verified' && p.completionEvidence==='verified'
      && (p.phase==='root_cancellation_absent'?p.originalEvidence==='not_recorded':p.originalEvidence==='verified')
      && !p.canContinue&&!p.canRollback&&!p.canFinalize&&!p.canRearm&&!p.canPreparePlan&&!p.canFinishUnchanged&&!p.canCancelPreparation
  }
  async function inspect() {
    if (executing.value || refreshing.value) return
    const seq = ++generation
    reading.value = true; checked.value = false; preview.value = null; outcome.value = null; choice.value = null
    error.value = ''; refreshError.value = ''; ready.value = false; reconfirmed.value = false; completionScope = null
    action.value = 'finalize'
    try {
      const p = await clientRecoveryApi.preview()
      if (seq !== generation || !open.value) return
      preview.value = p; checked.value = true
      if (verifiedClosedRoot(p) && useLifecycleStore().clientRecoveryRequired) {
        // Explicit read-only reinspection can reconcile a stale UI latch after
        // a fully indexed root close. Preview alone never clears it: bracket a
        // live same-scope status with another complete, unchanged preview.
        const current=useLifecycleStore().statusSnapshot?.facts
        if (!current?.data_root||!current.opencodex_home) return
        const scope={dataRoot:current.data_root,home:current.opencodex_home}
        try {
          const snapshot=await clientRecoveryApi.status()
          const confirmed=await clientRecoveryApi.preview()
          if(seq!==generation||!open.value)return
          if (!verifiedClosedRoot(confirmed)||confirmed.phase!==p.phase||confirmed.attemptId!==p.attemptId||confirmed.revision!==p.revision) {
            preview.value=null;checked.value=false;error.value='原恢复记录已变化，请重新检查。';return
          }
          ready.value=useLifecycleStore().acceptFinalizedClientRecovery(scope,snapshot)
          if(!ready.value) refreshError.value='恢复记录已核对，当前运行状态仍待核对；请重新检查。'
        } catch { if(seq===generation&&open.value)refreshError.value='恢复记录已核对，运行状态读取失败；请重新检查。' }
      }
    } catch (cause) { if (seq === generation && open.value) error.value = runtimeErrorText(cause) }
    finally { if (seq === generation) reading.value = false }
  }
  function start() {
    open.value = true
    // A successful mutation with failed observation must only retry observation.
    if (!executing.value && !refreshing.value && (!outcome.value || ready.value || reconfirmed.value)) void inspect()
  }
  function close() { open.value = false; generation++; reading.value = false; choice.value = null }
  function choose(value: ClientRecoveryAction) {
    if (can(value) && !reading.value && !executing.value && !refreshing.value) choice.value = value
  }
  function cancelChoice() { if (!executing.value) choice.value = null }
  async function refresh() {
    if (!outcome.value || refreshing.value || !completionScope) return false
    refreshing.value = true; refreshError.value = ''; ready.value = false; reconfirmed.value = false
    try {
      if (outcome.value.status === 'restored' || outcome.value.status === 'unchanged' || outcome.value.status === 'cancelled_before_capture' || outcome.value.status === 'cancelled_capture' || outcome.value.status === 'root_cancelled') {
        const snapshot = await clientRecoveryApi.status()
        ready.value = useLifecycleStore().acceptFinalizedClientRecovery(completionScope, snapshot)
        if (!ready.value) refreshError.value = '收尾已完成，当前运行状态仍待核对。请刷新运行状态。'
        return ready.value
      }
      // Rollback/rearm NEVER clear the lifecycle latch. Reconcile the current
      // bound scope, matching session/revision and expected durable phase.
      const snapshot = await clientRecoveryApi.status()
      const p = await clientRecoveryApi.preview()
      const current = useLifecycleStore().statusSnapshot
      const phase = outcome.value.status === 'rolled_back' ? 'restore_rolled_back' : outcome.value.status==='recovery_prepared'?'recovery_prepared':'restore_prepared'
      const matches = snapshot.source === 'live' && snapshot.matrix.runtime === 'stopped' && snapshot.matrix.operation === 'idle'
        && snapshot.facts.client_recovery === 'required' && snapshot.pid == null && snapshot.port == null
        && !snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart
        && snapshot.facts.data_root === completionScope.dataRoot && snapshot.facts.opencodex_home === completionScope.home
        && current?.facts.data_root === completionScope.dataRoot && current.facts.opencodex_home === completionScope.home
        && p.status === 'pending' && p.phase === phase && p.attemptId === outcome.value.attemptId && p.revision === outcome.value.revision
      if (matches) { preview.value = p; checked.value = true; reconfirmed.value = true }
      else { preview.value = null; checked.value = false; refreshError.value = '操作已完成，当前恢复状态仍待核对。请刷新恢复状态。' }
      return reconfirmed.value
    } catch {
      refreshError.value = (outcome.value.status === 'restored' || outcome.value.status === 'unchanged' || outcome.value.status === 'cancelled_before_capture' || outcome.value.status === 'cancelled_capture' || outcome.value.status === 'root_cancelled') ? '收尾已完成，运行状态读取失败。请刷新运行状态。' : '操作已完成，恢复状态读取失败。请刷新恢复状态。'
      return false
    } finally { refreshing.value = false }
  }
  async function apply(selected?: ClientRecoveryAction) {
    const next = selected ?? (canFinalize.value ? 'finalize' : 'continue')
    if (!can(next) || reading.value || executing.value || refreshing.value) return false
    // Actions that change direction require an explicit review state in this
    // dialog. Direct store calls cannot bypass that confirmation.
    if ((next === 'rollback' || next === 'rearm' || next === 'finishUnchanged' || next === 'cancelPreparation' || next === 'preparePlan') && choice.value !== next) return false
    const p = preview.value!
    const digest = next === 'preparePlan' ? p.planDigest! : next === 'cancelPreparation' ? p.preparationDigest! : next === 'finishUnchanged' ? p.unchangedDigest! : next === 'rollback' ? p.rollbackDigest! : next === 'rearm' ? p.rearmDigest! : p.confirmationDigest!
    const request = { attemptId: p.attemptId!, revision: p.revision!, confirmationDigest: digest }
    action.value = next; executing.value = true; error.value = ''; refreshError.value = ''
    try {
      const before = await clientRecoveryApi.status()
      if (before.source !== 'live' || !before.facts.data_root || !before.facts.opencodex_home) throw new Error('当前配置位置尚未核对，请重新检查客户端恢复。')
      completionScope = { dataRoot: before.facts.data_root, home: before.facts.opencodex_home }
      const result = await clientRecoveryApi[next](request)
      const expected = next === 'cancelPreparation' ? ((p.preparationKind==='created_root'||p.preparationKind==='created_root_capture')?'root_cancelled':p.phase==='capture_reserved'?'cancelled_capture':'cancelled_before_capture') : next === 'finishUnchanged' ? 'unchanged' : next === 'rollback' ? 'rolled_back' : next==='preparePlan' ? (p.phase==='stopped'?'restore_prepared':'recovery_prepared') : next==='rearm' ? ([1,2].includes(p.phaseCount)?'recovery_prepared':'restore_prepared') : 'restored'
      if (result.status !== expected || result.attemptId !== request.attemptId || result.revision !== request.revision + (next==='cancelPreparation'&&(p.preparationKind==='created_root'||p.preparationKind==='created_root_capture')?2:1)) throw new Error('恢复回执与本次确认不一致，请重新核对原会话。')
      outcome.value = result; preview.value = null; checked.value = false; choice.value = null
      // The mutation receipt is complete. The following read is observation,
      // so show its outcome while keeping the lifecycle gate until a verified
      // current snapshot arrives. A busy/failed read cannot replay the action.
      executing.value = false
      await refresh()
      return true
    } catch (cause) {
      error.value = runtimeErrorText(cause); preview.value = null; checked.value = false; choice.value = null; outcome.value = null
      completionScope = null; return false
    } finally { executing.value = false }
  }
  return { open, reading, executing, refreshing, checked, error, refreshError, preview, outcome, ready, reconfirmed,
    action, choice, canFinalize, canContinue, canRollback, canRearm, canFinishUnchanged, canCancelPreparation, canPreparePlan, continuing, start, close, choose, cancelChoice, inspect, apply, refresh }
})
