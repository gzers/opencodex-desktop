import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { useAppStore } from '@/stores/app'
import { usePreparationStore } from './preparation'
import { useEnvironmentStore } from './store'

export function useEnvironmentObservation(app: ReturnType<typeof useAppStore>) {
  const preparation = usePreparationStore()
  const environment = useEnvironmentStore()
  const lifecycle = useLifecycleStore()
  let disposed = false
  onMounted(preparation.start)
  onBeforeUnmount(() => {
    disposed = true
    preparation.stop()
    environment.checkedPreparationAttempt = null
  })

  const sourceIdentity = computed(() => {
    const source = app.runtimeSource
    return source ? JSON.stringify([source.kind, source.path, source.version,
      source.managedEntry, source.managedPrefix, source.explicitPath]) : null
  })
  let handledReady = false
  watch([sourceIdentity, () => preparation.suppliedAttempt], ([source, attempt], [previousSource, previousAttempt]) => {
    if ((previousSource !== null && source !== previousSource) || (attempt !== null && attempt !== previousAttempt)) {
      handledReady = false
      lifecycle.invalidateStatusObservation()
    }
  })
  // The worker records supplied and then announces a source change. Read the
  // same source even when polling sees completion before that event arrives.
  const sourceReadFor = ref<string | null>(null)
  watch(() => preparation.suppliedAttempt, async (attempt, _, onCleanup) => {
    sourceReadFor.value = null
    if (!attempt) return
    let current = true
    onCleanup(() => { current = false })
    // An earlier card read may still describe the installation before supply.
    if (app.runtimeSourceLoading) {
      await app.loadRuntimeSource()
      if (!current) return
    }
    await app.loadRuntimeSource()
    if (current) sourceReadFor.value = attempt
  }, { immediate: true })

  const canCheckDependencies = computed(() => !preparation.active && !preparation.retrying &&
    (!!preparation.snapshot || !!preparation.error) && preparation.suppliedAttempt === sourceReadFor.value)
  let checkedSource = sourceIdentity.value
  watch([sourceIdentity, canCheckDependencies, sourceReadFor, () => environment.loading,
    () => environment.ready, () => environment.report?.gate, () => environment.checkingPreparation], async () => {
    if (environment.report?.gate !== 'ready' || environment.checkingPreparation) handledReady = false
    if (!canCheckDependencies.value || environment.loading) return

    // Default discoveries share a backend flight. If the installation changed,
    // its new probe must take precedence over a ready report from the old one.
    const identity = sourceIdentity.value
    const sourceChanged = identity !== checkedSource &&
      (checkedSource !== null || app.runtimeSource?.kind !== 'unresolved')
    const attempt = preparation.suppliedAttempt
    checkedSource = identity
    if (sourceChanged || (attempt !== null && attempt !== environment.checkedPreparationAttempt)) {
      handledReady = false
      const ok = await environment.refresh()
      if (!disposed && !ok) app.showToast(environment.error)
      return
    }

    // The startup route and Overview use the same first observation, including an unknown result.
    if (!environment.ready || handledReady) return
    handledReady = true
    if (!lifecycle.statusLoading && !lifecycle.statusObserved && !lifecycle.statusSnapshot) {
      void app.loadStatusSnapshot()
    }
  }, { immediate: true })
}
