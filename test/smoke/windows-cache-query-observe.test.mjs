import assert from 'node:assert/strict'
import test from 'node:test'
import { assessCacheQueryGeneration } from './windows-cache-query-observe.mjs'

function fixture() {
  return {
    expectedRoot: 'C:\\Owned\\native-data',
    metadata: { roots: { currentDataRoot: '\\\\?\\C:\\Owned\\native-data', opencodexHomeMode: 'inside',
      currentOpencodexHome: '\\\\?\\C:\\Owned\\native-data\\opencodex-home' }, autoCheck: true,
      keepProxyOnClose: false, managerVersion: '0.1.10', managerChannel: 'stable',
      managerError: null, initialRoute: 'overview' },
    raw: { armedPerf: 1000, events: [
      { phase: 'begin', command: 'update_schedule_plan', callId: 1, atMs: 45010 },
      { phase: 'end', command: 'update_schedule_plan', callId: 1, status: 'fulfilled', atMs: 45011 },
    ] },
    summary: { queryCalls: 0, pendingObservedCalls: 0 },
    actions: Array.from({ length: 10 }, (_, index) => ({ index: index + 1,
      route: index % 2 ? 'overview' : 'settings', expectedRoute: index % 2 ? 'overview' : 'settings' })),
  }
}

test('requires ten actual transitions plus a fulfilled real startup planner, not a quick restart', () => {
  const result = assessCacheQueryGeneration(fixture())
  assert.equal(result.routeTransitions, 10)
  assert.equal(result.realStartupPlannerCompleted, true)
  assert.equal(result.nativeCounterAndNormalExitStillRequired, true)
  assert.equal(result.fullPerformanceGatePassed, false)
  for (const change of [
    value => { value.metadata.autoCheck = false },
    value => { value.metadata.keepProxyOnClose = true },
    value => { value.metadata.roots.currentDataRoot = 'C:\\Daily' },
    value => { value.metadata.roots.currentOpencodexHome = 'C:\\Daily' },
    value => { value.raw.armedPerf = 45000 },
    value => { value.raw.events[0].atMs = 44999 },
    value => { value.raw.events[1].status = 'rejected' },
    value => { value.summary.queryCalls = 1 },
    value => { value.summary.pendingObservedCalls = 1 },
    value => { value.actions.pop() },
    value => { value.actions[1].route = 'settings' },
    value => { value.actions[1].index = 3 },
  ]) {
    const value = fixture()
    change(value)
    assert.throws(() => assessCacheQueryGeneration(value))
  }
})
