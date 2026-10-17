// One passive generation of the frozen 100-route/10-restart scenario.
// UI input and process/configuration lifetime belong to the native controller.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { discoverMainTarget } from './windows-cdp.mjs'
import { attachSchedulerObserver, ObservationCdp, requireObservationWindow } from './windows-scheduler-observer.mjs'
import { writeFreshObservationJson } from './windows-observation-files.mjs'

const normalize = path => path.replace(/^\\\\\?\\/, '').replaceAll('/', '\\').replace(/\\$/, '').toLowerCase()
export function assessCacheQueryGeneration({ raw, summary, metadata, actions, expectedRoot }) {
  assert.equal(normalize(metadata.roots.currentDataRoot), normalize(expectedRoot), 'Actual native data root differs')
  assert.equal(metadata.roots.opencodexHomeMode, 'inside', 'Actual HOME mode differs')
  assert.equal(normalize(metadata.roots.currentOpencodexHome), normalize(expectedRoot + '\\opencodex-home'), 'Actual HOME escaped fixture')
  assert.equal(metadata.autoCheck, true, 'Actual automatic checks are disabled')
  assert.equal(metadata.keepProxyOnClose, false, 'Actual normal-close preference differs')
  assert.equal(metadata.managerVersion, '0.1.10')
  assert.equal(metadata.managerChannel, 'stable')
  assert.equal(metadata.managerError, null)
  assert(raw.armedPerf < 45_000, 'Observer missed the original startup grace interval')
  assert.equal(summary.queryCalls, 0, 'Actual query observed in the valid-cache scenario')
  assert.equal(summary.pendingObservedCalls, 0, 'Native observed call is still pending')
  const plans = raw.events.filter(event => event.phase === 'begin' && event.command === 'update_schedule_plan')
  const completed = new Map(raw.events.filter(event => event.phase === 'end').map(event => [event.callId, event]))
  assert(plans.some(event => event.atMs >= 45_000 && completed.get(event.callId)?.status === 'fulfilled'),
    'Original elapsed startup evaluation did not actually complete')
  assert.equal(actions.length, 10)
  let previous = metadata.initialRoute
  for (let index = 0; index < actions.length; index++) {
    const action = actions[index]
    assert.equal(action.index, index + 1)
    assert(['overview', 'settings'].includes(action.route))
    assert.notEqual(action.route, previous, 'A repeated route is not a transition')
    assert.equal(action.route, action.expectedRoute)
    previous = action.route
  }
  return { routeTransitions: 10, realStartupPlannerCompleted: true, observedQueryCalls: 0,
    nativeCounterAndNormalExitStillRequired: true, fullPerformanceGatePassed: false }
}

async function run(planFile) {
  const plan = JSON.parse((await readFile(planFile, 'utf8')).replace(/^\uFEFF/, ''))
  const closeAt = requireObservationWindow(plan.authorization)
  assert.equal(plan.source, '8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b')
  const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
  const output = resolve(plan.outputDirectory)
  const owned = relative(join(repo, 'test/runtime'), output)
  assert(owned && !owned.startsWith('..') && !isAbsolute(owned))
  const nativeBytes = await readFile(plan.nativeBinding.path)
  assert.equal(createHash('sha256').update(nativeBytes).digest('hex'), plan.nativeBinding.sha256)
  const native = JSON.parse(nativeBytes.toString('utf8').replace(/^\uFEFF/, ''))
  assert(native.schemaVersion === 1 && native.source === plan.source && native.retainedProductHandleVerified === true &&
    native.loopbackListenerVerified === true && native.listenerIsOwnedWebViewDescendant === true &&
    native.nativeProductCount === 1 && native.product.elevated === false && native.product.elevationType === 3 &&
    native.debugPort === Number(new URL(plan.endpoint).port), 'Exact ordinary product/listener binding required')
  assert.equal(native.exeSha256, '870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a')
  assert.equal(createHash('sha256').update(await readFile(native.product.executable)).digest('hex'), native.exeSha256)
  assert(Date.parse(native.product.creationTimeUtc) >= Date.parse(plan.authorization.authorizedAtUtc))
  assert(Date.parse(native.observedAtUtc) >= Date.parse(plan.authorization.authorizedAtUtc) &&
    Date.parse(native.observedAtUtc) <= Date.now())
  await mkdir(output)
  await writeFile(join(output, 'plan.original.json'), JSON.stringify(plan, null, 2) + '\n', { flag: 'wx' })
  let client, observer, record, metadata
  const actions = []
  let previous
  let finished = false
  try {
    const target = await discoverMainTarget(plan.endpoint)
    assert.equal(new URL(target.webSocketDebuggerUrl).port, new URL(plan.endpoint).port)
    client = await ObservationCdp.open(target.webSocketDebuggerUrl)
    const readyDeadline = Date.now() + 10_000
    let ready = false
    while (Date.now() < readyDeadline) {
      ready = await client.evaluate("!!window.__TAURI_INTERNALS__?.postMessage && !!document.querySelector('#app')?.children.length")
      if (ready) break
      await new Promise(resolveDelay => setTimeout(resolveDelay, 50))
    }
    assert(ready, 'Original bridge and mounted UI did not become ready')
    observer = await attachSchedulerObserver(client)
    assert(observer.installation.armedPerf < 45_000, 'Observer installed after the original startup grace')
    const local = await client.evaluate(`(async()=>{
      const invoke=window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
      const roots=await invoke('get_data_root_config'),prefs=await invoke('get_preferences'),
        manager=await invoke('get_update_status'),panel=await invoke('official_remote_cache');
      return {roots,autoCheck:prefs.appUpdateAutoCheck,keepProxyOnClose:prefs.keepProxyOnClose,
        managerVersion:manager.currentVersion,managerChannel:manager.channel,
        managerError:manager.error,managerLastChecked:manager.lastCheckedAt,panelVersion:panel?.version,
        initialRoute:document.documentElement.dataset.route};
    })()`)
    metadata = local
    assert.equal(normalize(local.roots.currentDataRoot), normalize(plan.expectedRoot))
    assert.equal(local.roots.opencodexHomeMode, 'inside')
    assert.equal(normalize(local.roots.currentOpencodexHome), normalize(plan.expectedRoot + '\\opencodex-home'))
    assert.equal(local.autoCheck, true)
    assert.equal(local.keepProxyOnClose, false)
    assert.equal(Date.parse(local.managerLastChecked), Date.parse(plan.expectedManagerCheckedAt))
    assert.equal(local.panelVersion, plan.expectedPanelVersion)
    assert(['overview', 'settings'].includes(local.initialRoute))
    previous = local.initialRoute
    await writeFreshObservationJson(join(output, 'observer-ready.json'), {
      ...observer.installation, metadata, ordinaryProductPid: native.product.processId,
    })
    while (Date.now() < closeAt) {
      // The controller performs exactly one real UIA navigation, then asks for
      // a read-only location confirmation before performing the next input.
      const index = actions.length + 1
      if (index <= 10) {
        let request
        try { request = JSON.parse(await readFile(join(output, `route-${index}-request.json`), 'utf8')) }
        catch (error) { if (error.code !== 'ENOENT') throw error }
        if (request) {
          assert.equal(request.index, index)
          const location = await client.evaluate(`({route:document.documentElement.dataset.route,hash:location.hash,
            atMs:performance.now(),atUtc:new Date().toISOString(),visible:document.visibilityState,focused:document.hasFocus()})`)
          if (location.route === request.expectedRoute && location.route !== previous) {
            const action = { index, expectedRoute: request.expectedRoute, ...location }
            actions.push(action); previous = location.route
            await writeFreshObservationJson(join(output, `route-${index}-confirmed.json`), action)
          }
        }
      }
      record = await observer.snapshot({ includeFrameStatistics: false })
      await appendFile(join(output, 'observations.original.jsonl'), JSON.stringify(record.delta) + '\n')
      assert.equal(record.summary.queryCalls, 0, 'Actual query observed; preserve this attempt')
      if (actions.length === 10) {
        const starts = record.raw.events.filter(event => event.phase === 'begin' && event.command === 'update_schedule_plan')
        if (starts.some(event => event.atMs >= 45_000 &&
          record.raw.events.some(end => end.phase === 'end' && end.callId === event.callId && end.status === 'fulfilled'))) {
          const assessment = assessCacheQueryGeneration({ ...record, metadata, actions, expectedRoot: plan.expectedRoot })
          await writeFreshObservationJson(join(output, 'generation-result.json'), {
            ...assessment, metadata, actions, summary: record.summary,
          })
          finished = true
          break
        }
      }
      try { await readFile(plan.stopFile); throw Error('Native controller stopped this incomplete generation') }
      catch (error) { if (error.code !== 'ENOENT') throw error }
      await new Promise(resolveDelay => setTimeout(resolveDelay, actions.length < 10 ? 50 : 1000))
    }
    assert(finished, 'No complete generation before the reserved cleanup time')
  } catch (error) {
    await writeFreshObservationJson(join(output, 'failure.original.json'), {
      error: String(error), actions, metadata, pauses: client?.pauses ?? 0,
      complete: false, fullPerformanceGatePassed: false,
    })
    throw error
  } finally {
    const cleanup = observer ? await observer.cleanup() : { observerInstalled: false }
    await writeFreshObservationJson(join(output, 'observer-cleanup.json'), cleanup)
    client?.socket.close()
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--observe' && process.argv[3]) await run(process.argv[3])
  else console.log(JSON.stringify({ status: 'prepared-only', productStarted: false, uiInputInjected: false }))
}
