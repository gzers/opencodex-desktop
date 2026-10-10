import { events, jobs } from '../../../tauri/config/event-registry.json'

// Share the backend catalog; UI feedback stays memory-only. Neither message text
// nor UI faults become persisted notifications through this adapter.
const owner = jobs.find(job => job.id === 'ui_feedback')
const allowed = new Set(events.filter(event =>
  owner?.availability === 'active' && owner.triggers.includes('status_change') &&
  event.job === owner.id && event.availability === 'active' &&
  event.fact === 'signal' && event.channels.includes('local') &&
  event.policy === null && event.resolves.length === 0,
).map(event => event.id))

export function allowsFeedbackSignal(event: string): boolean {
  return allowed.has(event)
}
