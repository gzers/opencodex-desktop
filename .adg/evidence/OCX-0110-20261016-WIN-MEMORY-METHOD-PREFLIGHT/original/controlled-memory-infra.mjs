// Supported CDP memory-infra method check; one owned browser and synthetic page.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { createRequire } from 'node:module'
import { execFileSync, spawn } from 'node:child_process'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const out = path.join(root, 'controlled')
const pwsh = 'C:\\Users\\15119\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\native\\powershell\\pwsh.exe'
const edge = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
const requireTools = createRequire('C:\\Users\\15119\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\node\\node_modules\\playwright-core\\package.json')
const { chromium } = requireTools('playwright-core')
assert.equal(process.platform, 'win32')
assert.match(process.version, /^v24\.19\.0$/)
assert(!process.execPath.includes('24.21'))
const sha = data => createHash('sha256').update(data).digest('hex')
const write = (name, value) => fs.writeFile(path.join(out, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' })
function nativeCount() {
  return Number(execFileSync(pwsh, ['-NoProfile', '-Command',
    "@(Get-CimInstance Win32_Process -Filter \"Name='opencodex-desktop.exe' OR Name='ocxd.exe'\").Count"],
  { encoding: 'utf8', windowsHide: true, timeout: 15000 }).trim())
}
assert.equal(nativeCount(), 0)
await fs.mkdir(out)
await write('binding.json', {
  methodSha256: sha(await fs.readFile(fileURLToPath(import.meta.url))),
  observerSha256: sha(await fs.readFile(path.join(root, 'root-observer.ps1'))),
  officialProtocolSha256: sha(await fs.readFile(path.join(root, 'browser_protocol.original.json'))),
  officialProtocolUrl: 'https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/master/json/browser_protocol.json',
  node: process.execPath, nodeVersion: process.version, browserExecutable: edge,
  controlledFixtureOnly: true, memoryContentsCaptured: false, desktopProductStarted: false,
  nativeIdlePairRepeated: false, deterministicDumpForcesGc: false, originalRssFailureCleared: false,
  priorFailedComponentSnapshotRetried: false, priorRejectedPixelPrototypesUsed: false,
  methodMaximumSeconds: 75, traceEventCapBytes: 32 * 1024 * 1024,
})
let server, browser, session, observer
let rootExit, observerExit
let tracing = false
let traced = false
let overCap = false
let traceBytes = 0
const events = []
const requests = []
const start = Date.now()
function budget() { assert(Date.now() - start < 75000, 'Controlled method budget expired') }
try {
  server = await chromium.launchServer({ executablePath: edge, headless: true })
  const child = server.process()
  assert(child?.pid > 0)
  rootExit = new Promise(resolve => child.once('exit', (code, signal) => resolve({ pid: child.pid, code, signal })))
  let diagnostic = ''
  observer = spawn(pwsh, ['-NoProfile', '-File', path.join(root, 'root-observer.ps1'),
    '-BrowserProcessId', String(child.pid), '-BrowserExecutable', edge, '-OutputDirectory', out],
  { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] })
  observer.stderr.on('data', data => { diagnostic += data.toString() })
  observerExit = new Promise((resolve, reject) => {
    observer.once('error', reject)
    observer.once('exit', code => resolve({ code, diagnostic }))
  })
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Retained root observer not ready')), 15000)
    observer.stdout.on('data', data => { if (data.toString().includes('ready')) { clearTimeout(timer); resolve() } })
    observer.once('exit', () => { clearTimeout(timer); reject(new Error(diagnostic || 'Observer stopped before ready')) })
  })
  browser = await chromium.connect(server.wsEndpoint())
  session = await browser.newBrowserCDPSession()
  const version = await session.send('Browser.getVersion')
  const categories = await session.send('Tracing.getCategories')
  assert(categories.categories.includes('disabled-by-default-memory-infra'))
  await write('actual-protocol.original.json', { version, memoryInfraCategoryPresent: true })
  session.on('Tracing.dataCollected', ({ value }) => {
    const size = Buffer.byteLength(JSON.stringify(value))
    traceBytes += size
    if (traceBytes > 32 * 1024 * 1024) overCap = true
    else events.push(...value)
  })
  const complete = new Promise(resolve => session.once('Tracing.tracingComplete', resolve))
  await session.send('Tracing.start', {
    transferMode: 'ReportEvents',
    traceConfig: {
      recordMode: 'recordUntilFull', traceBufferSizeInKb: 16384,
      includedCategories: ['disabled-by-default-memory-infra'],
      excludedCategories: ['*'], enableSampling: false, enableSystrace: false,
      memoryDumpConfig: { triggers: [] },
    },
  })
  tracing = true
  const context = await browser.newContext({ viewport: { width: 512, height: 512 } })
  const page = await context.newPage()
  await page.goto('about:blank')
  budget()
  async function dump(stage) {
    budget()
    const result = await session.send('Tracing.requestMemoryDump', { deterministic: false, levelOfDetail: 'detailed' })
    requests.push({ stage, atUtc: new Date().toISOString(), ...result })
    await write('dump-' + stage + '.original.json', requests.at(-1))
    assert(result.success, 'Memory dump refused for ' + stage)
  }
  await dump('empty')
  const scene = await page.evaluate(() => {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 512
    document.body.style.margin = '0'
    document.body.append(canvas)
    const gl = canvas.getContext('webgl', { depth: false, stencil: false, antialias: false })
    if (!gl) throw new Error('Controlled WebGL unavailable')
    const texture = gl.createTexture()
    if (!texture) throw new Error('Controlled texture unavailable')
    gl.bindTexture(gl.TEXTURE_2D, texture)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1024, 1024, 0, gl.RGBA, gl.UNSIGNED_BYTE, null)
    gl.clearColor(0.2, 0.4, 0.8, 1)
    gl.clear(gl.COLOR_BUFFER_BIT)
    gl.finish()
    const error = gl.getError()
    const debug = gl.getExtension('WEBGL_debug_renderer_info')
    window.ownMemoryControl = { gl, texture, canvas }
    return { textureRequestedRgba8Bytes: 1024 * 1024 * 4, canvasPixels: 512 * 512,
      error, contextLost: gl.isContextLost(), renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null }
  })
  assert.equal(scene.error, 0)
  assert.equal(scene.contextLost, false)
  await write('controlled-scene.original.json', scene)
  await dump('texture')
  const processes = await session.send('SystemInfo.getProcessInfo')
  assert(processes.processInfo.some(row => row.type === 'browser' && row.id === child.pid))
  await write('owned-process-info.original.json', processes)
  await session.send('Tracing.end')
  tracing = false
  const completion = await Promise.race([complete, new Promise((_, reject) => {
    const timer = setTimeout(() => reject(new Error('Trace completion expired')), 15000)
    timer.unref()
  })])
  traced = true
  await write('trace-completion.original.json', completion)
  await write('trace-events.original.json', { traceEvents: events })
  assert(!overCap && !completion.dataLossOccurred)
  const dumpEvents = events.filter(row => row.ph === 'v' && row.args?.dumps)
  const gpuPids = processes.processInfo.filter(row => row.type === 'GPU').map(row => row.id)
  const summary = {
    traceEvents: events.length, dumpEvents: dumpEvents.length, traceBytes,
    gpuPids, dumps: dumpEvents.map(row => ({
      pid: row.pid, id: row.id, name: row.name, topKeys: Object.keys(row.args.dumps),
      allocatorCount: Object.keys(row.args.dumps.allocators ?? {}).length,
      allocatorProviderPrefixes: [...new Set(Object.keys(row.args.dumps.allocators ?? {}).map(key => key.split('/')[0]))].sort(),
    })),
    requests, controlledScene: scene, hostProductStarted: false, formalRssBudgetPassed: false,
    actualProductAllocationCauseAttributed: false,
  }
  await write('method-result.json', summary)
  await page.evaluate(() => {
    const { gl, texture, canvas } = window.ownMemoryControl
    gl.deleteTexture(texture); gl.getExtension('WEBGL_lose_context')?.loseContext(); canvas.remove()
  })
  await context.close()
} catch (error) {
  await write('failure.json', { atUtc: new Date().toISOString(), name: error.name, message: error.message,
    traceStarted: tracing || traced, traceCompleted: traced, desktopProductStarted: false, automaticRetry: false })
  throw error
} finally {
  if (tracing && session) { try { await session.send('Tracing.end') } catch {} }
  if (server) {
    await server.close()
    const exit = await rootExit
    await write('node-root-exit.original.json', exit)
    assert.equal(exit.code, 0)
    assert.equal(exit.signal, null)
  }
  if (observerExit) {
    const exit = await observerExit
    await write('observer-exit.original.json', exit)
    assert.equal(exit.code, 0, exit.diagnostic)
  }
  assert.equal(nativeCount(), 0)
}
console.log(JSON.stringify({ output: out, elapsedMilliseconds: Date.now() - start, traceEvents: events.length }))
