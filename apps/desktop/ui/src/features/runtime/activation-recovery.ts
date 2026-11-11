import { ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { runtimeErrorText } from './errors'

export type ActivationDecision = 'complete' | 'rollback'
export interface ActivationPreview {
  attemptId: string; from: string | null; to: string
  programRoot: string; configPath: string; confirmationDigest: string
}
export interface ActivationOutcome { attemptId: string; decision: ActivationDecision; selectedVersion: string | null }
export interface ActivationRequest { attemptId: string; confirmationDigest: string; decision: ActivationDecision }
export const activationRecoveryApi = {
  preview: () => invoke<ActivationPreview | null>('runtime_activation_preview'),
  recover: (request: ActivationRequest) => invoke<ActivationOutcome>('runtime_activation_recover', { request }),
}

// Operation state survives settings route unmount; closing is only observation.
export const useActivationRecoveryStore = defineStore('runtime-activation-recovery', () => {
  const open = ref(false), reading = ref(false), executing = ref(false), checked = ref(false)
  const preview = shallowRef<ActivationPreview | null>(null), outcome = shallowRef<ActivationOutcome | null>(null)
  const decision = ref<ActivationDecision>('complete'), error = ref('')
  let generation = 0
  async function inspect() {
    if (executing.value) return
    const seq = ++generation
    reading.value = true; error.value = ''; preview.value = null; outcome.value = null; checked.value = false
    try {
      const result = await activationRecoveryApi.preview()
      if (seq !== generation || !open.value) return
      preview.value = result; decision.value = 'complete'; checked.value = true
    } catch (cause) { if (seq === generation) error.value = runtimeErrorText(cause) }
    finally { if (seq === generation) reading.value = false }
  }
  function start() {
    open.value = true
    if (!executing.value) void inspect()
  }
  function close() {
    open.value = false; generation++; reading.value = false
    // Do not cancel or discard an in-flight mutation or its result.
  }
  async function apply(): Promise<boolean> {
    const item = preview.value, choice = decision.value
    if (!item || executing.value || reading.value) return false
    const request = { attemptId: item.attemptId, confirmationDigest: item.confirmationDigest, decision: choice }
    executing.value = true; error.value = ''
    try {
      const result = await activationRecoveryApi.recover(request)
      const expected = choice === 'complete' ? item.to : item.from
      if (result.attemptId !== item.attemptId || result.decision !== choice || result.selectedVersion !== expected) {
        throw new Error('恢复回执与本次确认不一致。请重新核对原操作。')
      }
      outcome.value = result; preview.value = null; checked.value = false
      return true
    } catch (cause) {
      error.value = runtimeErrorText(cause)
      // A failed/lost reply grants no retry; explicitly observe the original state.
      preview.value = null; checked.value = false
      return false
    } finally { executing.value = false }
  }
  return { open, reading, executing, checked, preview, outcome, decision, error, start, close, inspect, apply }
})
