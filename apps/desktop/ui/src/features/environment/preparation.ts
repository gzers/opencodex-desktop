import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

const activePhases = ['checking', 'preparing', 'downloading', 'verifying', 'extracting', 'assembling', 'activating'] as const
const phases = [...activePhases, 'idle', 'supplied', 'failed', 'removed', 'review'] as const
export type PreparationPhase = typeof phases[number]

interface PreparationSnapshot {
  attemptId: string | null
  revision: number
  phase: PreparationPhase
  failedPhase: PreparationPhase | null
  failure: 'integrity' | 'busy' | 'recovery_required' | 'unavailable' | 'missing_archive' | null
  retryable: boolean
}

export const usePreparationStore = defineStore('runtimePreparation', () => {
  const snapshot = ref<PreparationSnapshot | null>(null)
  const error = ref('')
  const retryError = ref('')
  const retrying = ref(false)
  const active = computed(() => activePhases.some(phase => phase === snapshot.value?.phase))
  const needsAttention = computed(() => ['failed', 'removed', 'review'].some(phase => phase === snapshot.value?.phase))
  const suppliedAttempt = computed(() => snapshot.value?.phase === 'supplied' ? snapshot.value.attemptId : null)
  const canRetry = computed(() => !error.value && snapshot.value?.phase === 'failed' &&
    snapshot.value.retryable === true && !!snapshot.value.attemptId)
  const needsDownloadConsent = computed(() => canRetry.value && snapshot.value?.failure === 'missing_archive')

  let observing = false
  let generation = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let pending: Promise<void> | null = null

  function accept(next: PreparationSnapshot) {
    if (!next || !phases.includes(next.phase) || !Number.isSafeInteger(next.revision) || next.revision < 0 ||
      (next.phase !== 'idle' && (typeof next.attemptId !== 'string' || !next.attemptId))) {
      throw new Error('Invalid runtime preparation snapshot')
    }
    if (!snapshot.value || next.revision > snapshot.value.revision) snapshot.value = next
    error.value = ''
  }

  function schedule() {
    clearTimeout(timer)
    if (observing && active.value) timer = setTimeout(observe, 600)
  }

  function observe(): Promise<void> {
    if (!observing || retrying.value) return Promise.resolve()
    if (pending) return pending
    clearTimeout(timer)
    const own = generation
    pending = Promise.resolve().then(() => invoke<PreparationSnapshot>('runtime_preparation_status')).then(next => {
      if (own === generation) accept(next)
    }).catch(() => {
      if (own === generation) error.value = '准备状态暂不可读取，请重新检查。'
    }).finally(() => {
      if (own !== generation) return
      pending = null
      schedule()
    })
    return pending
  }

  function start() {
    if (observing) return
    observing = true
    void observe()
  }

  function stop() {
    observing = false
    generation++
    clearTimeout(timer)
    pending = null
    snapshot.value = null
    error.value = ''
    retryError.value = ''
    retrying.value = false
  }

  async function retry(confirmedAttemptId?: string) {
    const attemptId = snapshot.value?.attemptId
    if (!observing || !canRetry.value || !attemptId || retrying.value) return
    if (needsDownloadConsent.value ? confirmedAttemptId !== attemptId : confirmedAttemptId !== undefined) return
    retrying.value = true
    retryError.value = ''
    clearTimeout(timer)
    const own = ++generation
    pending = null
    try {
      const request = confirmedAttemptId === undefined ? { attemptId } : { attemptId, downloadConfirmed: true }
      const next = await invoke<PreparationSnapshot>('runtime_preparation_retry', { request })
      if (own === generation) accept(next)
    } catch {
      if (own === generation) retryError.value = '本次准备未开始，请核对当前准备状态。'
    } finally {
      if (own === generation) {
        retrying.value = false
        if (retryError.value) void observe()
        else schedule()
      }
    }
  }

  // Startup discovery can share the root's first read without creating a second observer.
  function initialObservation() {
    return snapshot.value === null ? pending : null
  }

  return { snapshot, error, retryError, retrying, active, needsAttention, suppliedAttempt, canRetry, needsDownloadConsent, observe, retry, start, stop, initialObservation }
})
