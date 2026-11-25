import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { useEnvironmentStore } from '@/features/environment/store'
import { usePreparationStore, type PreparationPhase } from '@/features/environment/preparation'
import type { RuntimeSourceDto } from '@/features/runtime/api'
import type { RuntimeState, StatusSnapshot } from '@/contracts/runtimeStatus'

const ipc = vi.hoisted(() => vi.fn())
const events = vi.hoisted(() => new Map<string, (event: unknown) => void>())
vi.mock('@tauri-apps/api/core', () => ({ invoke: ipc }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async (name: string, handler: (event: unknown) => void) => {
  events.set(name, handler)
  return () => events.delete(name)
} }))
vi.mock('@/app/bootstrap', () => ({ primeAppData(app: ReturnType<typeof useAppStore>) {
  void app.refreshEnvironment()
  void app.loadRuntimeSource()
} }))
vi.mock('@/features/updates/useUpdateScheduler', () => ({ useUpdateScheduler() {} }))
vi.mock('@/features/tray/api', () => ({ drainTrayRequests: async () => [], onTrayRequestsAvailable: async () => () => {} }))
vi.mock('@/features/panel/api', () => ({ syncEmbeddedPanel: async () => ({ visible: false, panelUrl: null }) }))
Object.defineProperty(window, 'matchMedia', { writable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })

const Root = defineComponent({ setup() { useAppController(true); return () => h('div') } })
const Page = defineComponent({ setup() { useAppController(false); return () => h('div') } })
function sourceDto(kind: RuntimeSourceDto['kind'] = 'managed'): RuntimeSourceDto {
  const path = kind === 'managed' ? '/I/runtime/bin/ocx' : null
  return { kind, path, version: path ? '2.99.0' : null, resolvedAt: 'first', insideDataRoot: false,
    managedEntry: path, managedPrefix: '/I/runtime', defaultPrefix: '/I/runtime', explicitPath: null, history: [] }
}
function report(ready: boolean) {
  const check = (name: string) => ({ found: ready, path: ready ? '/I/runtime/bin/' + name : null, version: null })
  return { node: check('node'), npm: check('npm'), ocx: check('ocx'), gate: ready ? 'ready' : 'missing_node', shortCircuited: !ready, brewFound: false }
}
function stage(phase: PreparationPhase, revision = 0) {
  return { attemptId: phase === 'idle' ? null : 'offline-1', revision, phase, failedPhase: null, failure: null, retryable: false }
}
function status(runtime: RuntimeState, failure: 'timeout' | 'unreachable' | 'parse' | 'unmapped' | null = null): StatusSnapshot {
  const running = runtime === 'running'
  const sample = {
    matrix: { runtime, connection: 'unconfigured', operation: 'idle' }, source: 'live', status_failure: failure,
    facts: { health: running ? 'healthy' : 'unknown', runtime_label: null, opencodex_home: '/H', data_root: '/D', protection: null,
      reboot_safe: null, service_present: null, shim_present: null, version_drift: null },
    port: running ? 10100 : null, pid: running ? '1402' : null, can_start: ['stopped', 'at_risk'].includes(runtime),
    can_stop: running, can_restart: running,
  } as const
  return sample
}
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(done => { resolve = done })
  return { promise, resolve }
}
const preparationRead = vi.fn(), sourceRead = vi.fn(), discovery = vi.fn(), statusRead = vi.fn()
let source: RuntimeSourceDto, preparation: ReturnType<typeof stage>, wrapper: VueWrapper | undefined
async function sourceEvent() {
  events.get('runtime-source-changed')!({ payload: null })
  await flushPromises()
}
async function mountRoot() { wrapper = mount(Root); await flushPromises() }
beforeEach(() => {
  setActivePinia(createPinia())
  vi.useFakeTimers()
  events.clear()
  source = sourceDto('unresolved')
  preparation = stage('idle')
  preparationRead.mockReset().mockImplementation(() => preparation)
  sourceRead.mockReset().mockImplementation(() => structuredClone(source))
  discovery.mockReset().mockImplementation(() => report(source.kind === 'managed'))
  statusRead.mockReset().mockImplementation(() => status('running'))
  ipc.mockReset().mockImplementation(async (command, args) => {
    switch (command) {
      case 'runtime_preparation_status': return preparationRead()
      case 'runtime_source': return sourceRead()
      case 'discover_environment': return discovery(args)
      case 'get_status_snapshot': return statusRead()
    }
  })
})
afterEach(() => { wrapper?.unmount(); wrapper = undefined; vi.useRealTimers() })

it('source events refresh missing dependencies, ignoring stable identities and read timestamps', async () => {
  await mountRoot()
  const environment = useEnvironmentStore()
  expect(environment.report?.gate).toBe('missing_node')
  expect(statusRead).not.toHaveBeenCalled()
  source = sourceDto()
  await sourceEvent()
  expect(environment.report).toEqual(report(true))
  expect(discovery).toHaveBeenCalledTimes(2)
  expect(discovery).toHaveBeenLastCalledWith({ request: null })
  expect(statusRead).toHaveBeenCalledOnce()
  source = { ...source, resolvedAt: 'later', history: [{ action: 'read', result: 'ok', package: 'ocx', version: '2.99.0', target: '/I', at: 'later' }] }
  await sourceEvent()
  expect(discovery).toHaveBeenCalledTimes(2)
  source = { ...source, path: '/I/runtime/next/ocx', version: '3.0.0' }
  await sourceEvent()
  expect(discovery).toHaveBeenCalledTimes(3)
  expect(statusRead).toHaveBeenCalledTimes(2)
})

it('a changed source waits out an old backend discovery flight and reports the new probe error', async () => {
  const first = deferred<ReturnType<typeof report>>()
  discovery.mockReturnValueOnce(first.promise).mockRejectedValueOnce(new Error('probe failed'))
  await mountRoot()
  source = sourceDto()
  await sourceEvent()
  expect(discovery).toHaveBeenCalledOnce()
  first.resolve(report(true))
  await flushPromises()
  expect(discovery).toHaveBeenCalledTimes(2)
  expect(useEnvironmentStore().report).toBeNull()
  expect(useEnvironmentStore().ready).toBe(false)
  expect(useAppStore().toast).toBe(useEnvironmentStore().error)
  expect(statusRead).not.toHaveBeenCalled()
})

it.each(['before', 'after'])('off Overview, the source event %s the supplied poll produces one post-supply probe', async order => {
  const old = deferred<ReturnType<typeof report>>()
  const fresh = deferred<ReturnType<typeof report>>()
  discovery.mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise)
  preparation = stage('extracting', 2)
  useRouteStore().go('settings', { section: 'installation' })
  useLifecycleStore().setStatusSnapshot(status('not_found'))
  await mountRoot()
  source = sourceDto()
  if (order === 'before') await sourceEvent()
  preparation = stage('supplied', 6)
  useRouteStore().go('logs')
  await vi.advanceTimersByTimeAsync(600)
  if (order === 'after') await sourceEvent()
  expect(discovery).toHaveBeenCalledOnce()
  expect(statusRead).not.toHaveBeenCalled()
  old.resolve(report(true))
  await flushPromises()
  expect(discovery).toHaveBeenCalledTimes(2)
  expect(useEnvironmentStore().ready).toBe(false)
  expect(statusRead).not.toHaveBeenCalled()
  fresh.resolve(report(true))
  await flushPromises()
  expect(useEnvironmentStore().ready).toBe(true)
  expect(statusRead).toHaveBeenCalledOnce()
  await sourceEvent()
  await vi.advanceTimersByTimeAsync(2400)
  expect(preparationRead).toHaveBeenCalledTimes(2)
  expect(discovery).toHaveBeenCalledTimes(2)
  expect(statusRead).toHaveBeenCalledOnce()
  expect(ipc.mock.calls.some(([name]) => name === 'process_action' || name === 'runtime_preparation_retry')).toBe(false)
})

it('a source read started before supply cannot consume the completion broadcast with an obsolete result', async () => {
  const prior = deferred<RuntimeSourceDto>()
  sourceRead.mockReturnValueOnce(prior.promise)
  preparation = stage('activating', 5)
  await mountRoot()
  source = sourceDto()
  preparation = stage('supplied', 6)
  await vi.advanceTimersByTimeAsync(600)
  await sourceEvent()
  prior.resolve(sourceDto('unresolved'))
  await flushPromises()
  expect(useAppStore().runtimeSource).toEqual(source)
  expect(sourceRead).toHaveBeenCalledTimes(2)
  expect(discovery).toHaveBeenCalledTimes(2)
  expect(statusRead).toHaveBeenCalledOnce()
})

it.each(['failed', 'review', 'removed'] as const)('a %s terminal never treats pre-supply readiness as current', async phase => {
  preparation = stage('checking', 1)
  discovery.mockResolvedValue(report(true))
  await mountRoot()
  expect(useEnvironmentStore().ready).toBe(false)
  preparation = stage(phase, 3)
  await vi.advanceTimersByTimeAsync(2400)
  expect(useEnvironmentStore().ready).toBe(false)
  expect(discovery).toHaveBeenCalledOnce()
  expect(statusRead).not.toHaveBeenCalled()
  expect(preparationRead).toHaveBeenCalledTimes(2)
})

it.each([
  ['entry not yet found', status('not_found')],
  ['official timeout', status('unreachable', 'timeout')],
  ['unmapped observation', status('unreachable', 'unmapped')],
  ['failed IPC', null],
] as const)('readiness retains the completed pre-mount result for %s', async (_, result) => {
  const lifecycle = useLifecycleStore()
  discovery.mockResolvedValue(report(true))
  if (result) statusRead.mockResolvedValue(result)
  else statusRead.mockRejectedValue(new Error('status IPC unavailable'))
  await useAppStore().loadStatusSnapshot()
  await mountRoot()
  expect(useEnvironmentStore().ready).toBe(true)
  expect(statusRead).toHaveBeenCalledOnce()
  expect(lifecycle.statusSnapshot).toEqual(result?.status_failure ? null : result)
  expect(lifecycle.statusError).toBe(!result || !!result.status_failure)
  expect(lifecycle.processActionBlocked('start')).toBe(true)
})

it('readiness without a prior observation settles once at the five-second deadline', async () => {
  discovery.mockResolvedValue(report(true))
  statusRead.mockReturnValue(deferred<StatusSnapshot>().promise)
  await mountRoot()
  const lifecycle = useLifecycleStore()
  expect(useEnvironmentStore().ready).toBe(true)
  await vi.advanceTimersByTimeAsync(4999)
  expect(lifecycle.statusLoading).toBe(true)
  await vi.advanceTimersByTimeAsync(1)
  expect(lifecycle.statusLoading).toBe(false)
  expect(lifecycle.statusError).toBe(true)
  expect(lifecycle.statusSnapshot).toBeNull()
  expect(lifecycle.processActionBlocked('start')).toBe(true)
  expect(statusRead).toHaveBeenCalledOnce()
})

it('a replaced observation cannot clear loading or restore action authority', async () => {
  const old = deferred<StatusSnapshot>()
  const replacement = deferred<StatusSnapshot>()
  discovery.mockResolvedValue(report(true))
  statusRead.mockReturnValueOnce(old.promise).mockReturnValueOnce(replacement.promise)
  source = sourceDto()
  await mountRoot()
  source = { ...source, version: '2.99.1' }
  await sourceEvent()
  expect(statusRead).toHaveBeenCalledTimes(2)
  const lifecycle = useLifecycleStore()
  old.resolve(status('stopped'))
  events.get('status-snapshot-changed')!({ payload: status('stopped') })
  await flushPromises()
  expect(lifecycle.statusLoading).toBe(true)
  expect(lifecycle.statusSnapshot).toBeNull()
  expect(lifecycle.processActionBlocked('start')).toBe(true)
  replacement.resolve(status('stopped'))
  await flushPromises()
  expect(lifecycle.statusLoading).toBe(false)
  expect(lifecycle.processActionBlocked('start')).toBe(false)
})

it.each(['running', 'stopped', 'at_risk'] as const)('preserves confirmed %s across readiness', async runtime => {
  const prior = status(runtime)
  useLifecycleStore().setStatusSnapshot(prior)
  discovery.mockResolvedValue(report(true))
  await mountRoot()
  expect(useEnvironmentStore().ready).toBe(true)
  expect(statusRead).not.toHaveBeenCalled()
  expect(useLifecycleStore().statusSnapshot).toEqual(prior)
})

it.each([
  ['running', status('running')], ['ManagedHomeIdle', status('stopped')],
  ['not found', status('not_found')], ['unknown', status('unreachable', 'unmapped')],
] as const)('shares the pre-mount Status command that returns %s', async (_, result) => {
  const pending = deferred<StatusSnapshot>()
  discovery.mockResolvedValue(report(true))
  statusRead.mockReturnValueOnce(pending.promise)
  const initial = useAppStore().loadStatusSnapshot()
  await mountRoot()
  expect(useEnvironmentStore().ready).toBe(true)
  expect(statusRead).toHaveBeenCalledOnce()
  pending.resolve(result)
  await initial
  await flushPromises()
  expect(statusRead).toHaveBeenCalledOnce()
})

it.each([
  ['unknown event followed by confirmed command', status('unreachable', 'timeout'), status('running')],
  ['confirmed event followed by unknown command', status('running'), status('unreachable', 'timeout')],
] as const)('a polling event cannot settle the pre-mount command: %s', async (_, event, result) => {
  const pending = deferred<StatusSnapshot>()
  discovery.mockResolvedValue(report(true))
  statusRead.mockReturnValueOnce(pending.promise)
  const initial = useAppStore().loadStatusSnapshot()
  await mountRoot()
  events.get('status-snapshot-changed')!({ payload: event })
  await flushPromises()
  expect(useLifecycleStore().statusLoading).toBe(true)
  const callsWhilePending = statusRead.mock.calls.length
  pending.resolve(result)
  await initial
  await flushPromises()
  expect(callsWhilePending).toBe(1)
  expect(statusRead).toHaveBeenCalledOnce()
})

it('a route controller has no observer, and teardown discards a late supply without scheduling anything', async () => {
  wrapper = mount(Page)
  await flushPromises()
  expect(preparationRead).not.toHaveBeenCalled()
  expect(discovery).not.toHaveBeenCalled()
  wrapper.unmount()
  const pending = deferred<ReturnType<typeof stage>>()
  preparationRead.mockReturnValueOnce(pending.promise)
  await mountRoot()
  wrapper!.unmount()
  wrapper = undefined
  pending.resolve(stage('supplied', 5))
  await flushPromises()
  await vi.advanceTimersByTimeAsync(2400)
  expect(usePreparationStore().snapshot).toBeNull()
  expect(preparationRead).toHaveBeenCalledOnce()
  expect(discovery).toHaveBeenCalledOnce()
  expect(sourceRead).toHaveBeenCalledOnce()
  expect(statusRead).not.toHaveBeenCalled()
  expect(events.has('runtime-source-changed')).toBe(false)
})
