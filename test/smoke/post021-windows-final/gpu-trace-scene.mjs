import assert from 'node:assert/strict'
import { readFile, writeFile, realpath, access } from 'node:fs/promises'
import { join, relative, isAbsolute } from 'node:path'
import { fileURLToPath } from 'node:url'
import { connect } from './cdp-session.mjs'
import { assertModelSandbox } from '../windows-model-ui.mjs'
import { readCollectorStart, assertCaptureOverlap } from './gpu-trace-handshake.mjs'

const [endpoint, sandboxRoot, outputRoot] = process.argv.slice(2)
assert(endpoint && sandboxRoot && outputRoot, 'Usage: node gpu-trace-scene.mjs <CDP endpoint> <sandbox> <fresh owned output directory>')
const tools = await realpath(fileURLToPath(new URL('../../../../.toolchain/', import.meta.url)))
const output = await realpath(outputRoot), sandbox = await realpath(sandboxRoot)
const child = (root, path) => { const part = relative(root, path); return part && part !== '..' && !part.startsWith('..\\') && !part.startsWith('../') && !isAbsolute(part) }
assert(child(join(tools, 'post021-msvc-followup'), output), 'Output must be below owned followup root')
assert(child(join(tools, 'post021-windows-final-msvc-retry-41944607861'), sandbox), 'Verified MSVC isolated sandbox required')
for (const name of ['scene-ready.json', 'scene-result.json']) {
  await assert.rejects(access(join(output, name)), { code: 'ENOENT' }, 'Never overwrite a previous scene')
}
const expected = JSON.parse((await readFile(join(output, 'target.json'), 'utf8')).replace(/^\uFEFF/, ''))
assert.equal(expected.sha256, 'fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68')
const session = await connect(endpoint)
const evaluate = (fn, ...args) => session.evaluate('(' + fn.toString() + ')(' + args.map(v => JSON.stringify(v)).join(',') + ')')
const report = { result: 'waiting-for-administrator', acceptance: false, scene: 'Original 30s high/dark static overview; unchanged browser scheduling' }
const save = (name, value) => writeFile(join(output, name), JSON.stringify(value, null, 2) + '\n')
try {
  assertModelSandbox({ about: await session.invoke('app_about'), dataRoot: await session.invoke('get_data_root_config') }, sandbox)
  await evaluate(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia, preferences = p._s.get('preferences')
    if (!await preferences.save({ ...preferences.data, theme: 'dark', visualEffects: 'high', appUpdateAutoCheck: false })) throw Error('Isolated preference save failed')
    p._s.get('theme').apply('dark'); p._s.get('sync').connectionOpen = false; p._s.get('notifications').panelOpen = false
    p._s.get('feedback').clearToast(); p._s.get('operation-tasks').tasks = []; p._s.get('model-save-feedback').dismiss()
    p._s.get('routes').go('overview'); await new Promise(r => setTimeout(r, 1200))
  })
  // Execute the original QA function without changing its 30s/B/long-frame gates.
  const source = await readFile(new URL('../../../apps/desktop/ui/qa/windows-materials.mjs', import.meta.url), 'utf8')
  const start = source.indexOf('async function performanceSample(scroll) {'), end = source.indexOf('\nconst results = ', start)
  assert(start >= 0 && end > start, 'Original QA sampler not found')
  const sample = new Function('evaluate', source.slice(start, end) + ';return performanceSample')(evaluate)
  const deadline = Date.now() + 3600000
  await save('scene-ready.json', { at: new Date().toISOString(), pid: expected.pid, ready: true, waitLimitSeconds: 3600 })
  let started
  while (Date.now() < deadline) {
    started = await readCollectorStart(join(output, 'capture/collector-started.json'))
    if (started?.recorderStarted) break
    await new Promise(r => setTimeout(r, 500))
  }
  assert(started?.recorderStarted, 'No successful administrator recording observed in 3600s; no sample performed')
  for (const key of ['pid', 'exe', 'createdAtUtcTicks', 'sessionId', 'ownerDomain', 'ownerUser']) assert.equal(started.targetAtStart[key], expected[key], 'Collector target mismatch: ' + key)
  assert.equal(started.targetExeSha256, expected.sha256)
  assertCaptureOverlap(started)
  while (Date.now() < Date.parse(started.startedAtUtc) + 10000 && !await session.invoke('app_foreground')) await new Promise(r => setTimeout(r, 200))
  assert(await session.invoke('app_foreground'), 'Native test window not foreground')
  assertCaptureOverlap(started)
  Object.assign(report, { result: 'sampling', collectorInstance: started.instanceName, collectorStartedAtUtc: started.startedAtUtc, startedAtUtc: new Date().toISOString() })
  await save('scene-started.json', report)
  report.sample = await sample(false); report.endedAtUtc = new Date().toISOString()
  assert(report.sample.foreground, 'Lost native foreground')
  assert(report.sample.p95 <= 2 * report.sample.B, 'Original p95 <= 2B budget failed')
  assert.equal(report.sample.over6B, 0, 'Original over6B budget failed')
  report.result = 'passed-original-frame-budget'
} catch (error) {
  report.result = 'failed'; report.error = { message: error.message, stack: error.stack }; process.exitCode = 1
} finally {
  report.finishedAtUtc = new Date().toISOString(); await save('scene-result.json', report)
  session.close(); console.log(JSON.stringify({ result: report.result, error: report.error?.message }))
}
