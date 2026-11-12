import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { clientRecoveryApi, type ClientRecoveryPreview, type ClientRecoveryFinalizeOutcome } from './client-recovery'
import { runtimeErrorText } from './errors'
import { useLifecycleStore } from '@/app/lifecycle/store'

export const useClientRecoveryStore = defineStore('client-recovery', () => {
  const open = ref(false), reading = ref(false), executing = ref(false), refreshing = ref(false)
  const checked = ref(false), error = ref(''), refreshError = ref('')
  const preview = shallowRef<ClientRecoveryPreview | null>(null)
  const outcome = shallowRef<ClientRecoveryFinalizeOutcome | null>(null)
  const ready = ref(false)
  const continuing = ref(false)
  const canFinalize = computed(() => {
    const p = preview.value
    return p?.status === 'pending' && p.phase === 'restore_prepared' && p.canFinalize && !p.canContinue && !p.canRollback
      && p.originalEvidence === 'verified' && p.planEvidence === 'verified'
      && (p.completionEvidence === 'not_recorded' || p.completionEvidence === 'unregistered')
      && !!p.attemptId && Number.isSafeInteger(p.revision) && p.revision! > 0
      && typeof p.confirmationDigest === 'string' && /^[a-f0-9]{64}$/.test(p.confirmationDigest)
  })
  const canContinue = computed(() => {
    const p = preview.value
    return p?.status === 'pending' && p.phase === 'restore_prepared' && p.canContinue && !p.canFinalize && !p.canRollback
      && p.originalEvidence === 'verified' && p.planEvidence === 'verified' && p.completionEvidence === 'not_recorded'
      && !!p.attemptId && Number.isSafeInteger(p.revision) && p.revision! > 0
      && typeof p.confirmationDigest === 'string' && /^[a-f0-9]{64}$/.test(p.confirmationDigest)
  })
  let generation = 0
  let completionScope: { dataRoot: string; home: string } | null = null
  async function inspect() {
    if (executing.value || refreshing.value) return
    const seq = ++generation
    reading.value = true; checked.value = false; preview.value = null; outcome.value = null
    error.value = ''; refreshError.value = ''; ready.value = false; completionScope = null
    continuing.value = false
    try {
      const p = await clientRecoveryApi.preview()
      if (seq !== generation || !open.value) return
      preview.value = p; checked.value = true
    } catch (cause) { if (seq === generation && open.value) error.value = runtimeErrorText(cause) }
    finally { if (seq === generation) reading.value = false }
  }
  function start() {
    open.value = true
    // Keep an unrefreshed success receipt for status-only retry. A fully
    // completed view reopens by inspecting the CURRENT session, not stale success.
    if (!executing.value && !refreshing.value && (!outcome.value || ready.value)) void inspect()
  }
  function close() {
    open.value = false; generation++; reading.value = false
    // Detach observation only. Route remount never resubmits an active mutation.
  }
  async function refresh() {
    if (!outcome.value || refreshing.value || !completionScope) return false
    refreshing.value = true; refreshError.value = ''; ready.value = false
    try {
      const snapshot = await clientRecoveryApi.status()
      ready.value = useLifecycleStore().acceptFinalizedClientRecovery(completionScope, snapshot)
      if (!ready.value) refreshError.value = '收尾已完成，当前运行状态仍待核对。请刷新运行状态。'
      return ready.value
    } catch { refreshError.value = '收尾已完成，运行状态读取失败。请刷新运行状态。'; return false }
    finally { refreshing.value = false }
  }
  async function apply() {
    if ((!canFinalize.value && !canContinue.value) || reading.value || executing.value || refreshing.value) return false
    continuing.value = canContinue.value
    const p = preview.value!
    const request = { attemptId: p.attemptId!, revision: p.revision!, confirmationDigest: p.confirmationDigest! }
    executing.value = true; error.value = ''; refreshError.value = ''
    try {
      // UI scope reconciliation grants no mutation authority; backend rechecks.
      const before = await clientRecoveryApi.status()
      if (before.source !== 'live' || !before.facts.data_root || !before.facts.opencodex_home) {
        throw new Error('当前配置位置尚未核对，请重新检查客户端恢复。')
      }
      completionScope = { dataRoot: before.facts.data_root, home: before.facts.opencodex_home }
      const result = await (continuing.value ? clientRecoveryApi.continue(request) : clientRecoveryApi.finalize(request))
      if (result.status !== 'restored' || result.attemptId !== request.attemptId || result.revision !== request.revision + 1) {
        throw new Error('恢复回执与本次确认不一致，请重新核对原会话。')
      }
      outcome.value = result; preview.value = null; checked.value = false
      await refresh()
      return true
    } catch (cause) {
      error.value = runtimeErrorText(cause); preview.value = null; checked.value = false
      completionScope = null; return false
    } finally { executing.value = false }
  }
  return { open, reading, executing, refreshing, checked, error, refreshError, preview, outcome, ready, canFinalize, canContinue, continuing, start, close, inspect, apply, refresh }
})
