import assert from 'node:assert/strict'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { connect } from './cdp-session.mjs'
import { assertModelSandbox, modelConfigPath } from '../windows-model-ui.mjs'

const [endpoint, sandbox, output, gateFile] = process.argv.slice(2)
assert(endpoint && sandbox && output && gateFile, 'usage: panel-retry-navigation.mjs ENDPOINT SANDBOX OUTPUT OWNED_GATE')
assert(resolve(gateFile).includes('post021-msvc-followup'), 'explicit owned fixture gate required')
assert.equal((await readFile(gateFile, 'utf8')).trim(), 'delay', 'begin with delayed official navigation')
await mkdir(output, { recursive: true })
const session = await connect(endpoint), report = { result: 'fail', scope: 'Controlled HTTP response-header delay; actual manager retry, real IPC, no service restart' }
const expression = (fn, ...args) => '(' + fn.toString() + ')(' + args.map(value => JSON.stringify(value)).join(',') + ')'
async function until(fn, name, timeoutMs = 10000) {
  const start = performance.now()
  while (performance.now() - start < timeoutMs) {
    const result = await session.evaluate(expression(fn))
    if (result) return result
    await new Promise(resolve => setTimeout(resolve, 80))
  }
  throw Error('bounded native wait: ' + name)
}
try {
  const facts = { about: await session.invoke('app_about'), dataRoot: await session.invoke('get_data_root_config') }
  assertModelSandbox(facts, sandbox)
  assert(await session.invoke('app_foreground'), 'native foreground required')
  const config = modelConfigPath(facts, sandbox), before = await readFile(config)
  report.serviceBefore = await session.invoke('get_status_snapshot')
  assert.equal(report.serviceBefore.matrix.runtime, 'running')
  await session.evaluate(expression(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    if (location.hash !== '#overview') throw Error('fresh overview entry required')
    if (p._s.get('notifications').panelOpen || p._s.get('app').toast) throw Error('overlay must be closed before load')
    p._s.get('routes').go('panel')
  }))
  report.timeout = await until(() => {
    const panel = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('panel')
    if (panel.loadTrace.outcome !== 'timeout') return null
    return { frame: document.querySelector('.panel-frame')?.innerText, trace: JSON.parse(JSON.stringify(panel.loadTrace)) }
  }, 'original 8s load deadline', 12000)
  assert(report.timeout.frame.includes('官方面板加载失败') && report.timeout.frame.includes('重试加载'))
  const timeoutPhase = report.timeout.trace.phases.find(phase => phase.phase === 'timeout')
  assert(timeoutPhase && timeoutPhase.elapsedMs >= 7900 && timeoutPhase.elapsedMs < 10000, 'original deadline changed or heavily delayed')
  await writeFile(join(output, 'timeout-observed.json'), JSON.stringify(report.timeout, null, 2))
  await writeFile(gateFile, 'ready' + String.fromCharCode(10))
  report.retryStartedAt = new Date().toISOString()
  const start = performance.now()
  await session.evaluate(expression(() => {
    const button = [...document.querySelectorAll('.panel-frame button')].find(node => node.textContent.trim() === '重试加载' && !node.disabled)
    if (!button) throw Error('actual retry button missing')
    button.click()
  }))
  report.restored = await until(() => {
    const panel = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('panel')
    if (!panel.loaded || panel.loadTrace.outcome !== 'visible-ready') return null
    return { frame: document.querySelector('.panel-frame')?.innerText, trace: JSON.parse(JSON.stringify(panel.loadTrace)) }
  }, 'retry creates ready view within original deadline', 8000)
  report.retryElapsedMs = performance.now() - start
  assert(report.restored.trace.id > report.timeout.trace.id, 'a late original navigation is not a user retry')
  assert(report.retryElapsedMs < 8000, 'retry exceeded original deadline')
  report.serviceAfter = await session.invoke('get_status_snapshot')
  assert.equal(report.serviceAfter.pid, report.serviceBefore.pid, 'retry changed service process')
  assert.equal(report.serviceAfter.port, report.serviceBefore.port)
  assert((await readFile(config)).equals(before), 'retry changed saved config bytes')
  assert(await session.invoke('app_foreground'), 'foreground lost')
  report.result = 'pass'
} catch (error) {
  report.error = { message: error.message, stack: error.stack }; process.exitCode = 1
} finally {
  report.visible = await session.evaluate('document.body.innerText').catch(() => null)
  await writeFile(join(output, 'panel-retry-navigation.json'), JSON.stringify(report, null, 2) + String.fromCharCode(10))
  session.close()
}
