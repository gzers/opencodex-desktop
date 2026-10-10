import { onMounted, onUnmounted, watch } from 'vue'
import { useEffectsStore } from '@/app/appearance/effects'
import { usePreferencesStore } from '@/features/preferences/store'
import { useDataRootStore } from '@/features/data-root/store'
import { useRouteStore } from '@/stores/routes'
import { getOfficialRemoteCache } from '@/features/about/api'
import { useUpdatesStore } from './store'
import { checkForUpdate, getUpdateStatus } from './update'
import { createUpdateScheduler, getUpdateSchedulePlan } from './scheduler'

/** Application-owned coordinator. Page mounts never own update timers or queries. */
export function useUpdateScheduler() {
  const effects = useEffectsStore()
  const preferences = usePreferencesStore()
  const roots = useDataRootStore()
  const routes = useRouteStore()
  const updates = useUpdatesStore()
  let disposed = false
  const scheduler = createUpdateScheduler({
    plan: getUpdateSchedulePlan,
    visible: () => effects.visible && effects.foreground,
    busy: () => updates.appUpdateBusy || updates.officialUpdateBusy || roots.switching,
    check: async (target, trigger) => {
      if (target === 'panel') await updates.loadRemoteLatest(trigger)
      else await checkForUpdate(trigger)
    },
    // Native command persists the outcome/backoff. No repetitive foreground toast.
    onError: () => {},
  })
  const wake = () => scheduler.wake()
  const foregroundWake = () => scheduler.wake('foreground')
  const onlineWake = () => scheduler.wake('online')
  // Entering the overview is a foreground trigger; deadline is reserved for
  // the persisted scheduler wake so the event contract keeps the source visible.
  watch(() => routes.current, route => { if (route === 'overview') foregroundWake() })
  watch(() => [effects.visible, effects.foreground, roots.switching,
    updates.appUpdateBusy, updates.officialUpdateBusy], foregroundWake)
  watch(() => [preferences.data?.appUpdateChannel, preferences.data?.appUpdateAutoCheck], async () => {
    // Preferences transaction already switched native ownership; synchronize UI epoch first.
    try { await getUpdateStatus() } catch { /* retain last known result */ }
    if (!disposed) wake()
  })
  onMounted(() => {
    void getUpdateStatus().catch(() => {})
    void getOfficialRemoteCache().then(value => {
      if (!disposed && !updates.officialRemote && value) updates.officialRemote = value
    }).catch(() => {})
    window.addEventListener('online', onlineWake)
    scheduler.start()
  })
  onUnmounted(() => {
    disposed = true
    scheduler.stop()
    window.removeEventListener('online', onlineWake)
  })
}
