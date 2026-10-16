// One bounded A/B/A headless resource diagnostic, outside all native acceptance spans.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import fs from 'node:fs/promises'
import path from 'node:path'
import { createRequire } from 'node:module'
import { execFileSync, spawn } from 'node:child_process'
import { createServer } from 'node:http'

const repo = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop'
const out = path.resolve(process.argv[2] || '')
assert(out.startsWith(path.join(repo, 'test', 'runtime') + path.sep))
const source = '823a8a0615153068594daef530c2e80f1d95fc29'
assert.equal(execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim(), source)
assert.equal(execFileSync('git', ['status', '--porcelain', '--untracked-files=no'], { cwd: repo, encoding: 'utf8' }), '')
await fs.mkdir(out)
const sha = value => createHash('sha256').update(value).digest('hex')
const write = (name, value) => fs.writeFile(path.join(out, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' })
const reference = execFileSync('git', ['show', `${source}:apps/desktop/ui/src/features/runtime/motion/meshGlow.ts`], { cwd: repo, encoding: 'utf8' })
const prototypePath = path.join(repo, 'test/runtime/mesh-alpha-20261016T1354/rejected-premultiplied-source.original.ts')
const prototype = await fs.readFile(prototypePath, 'utf8')
assert.equal(sha(reference), '6991a6898ae798bf4c97c0aa5f8d67dbb920768032050057e2d0efd6d05e0f1d')
assert.equal(sha(prototype), '67e88b7afcf7588c1261fbf9fcccda76e93c3b2eb8ebb0e487f25c5034c667ea')
const requireUi = createRequire(path.join(repo, 'apps/desktop/ui/package.json'))
const ts = requireUi('typescript')
const requireTools = createRequire('C:\\Users\\15119\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\node\\node_modules\\playwright-core\\package.json')
const { chromium } = requireTools('playwright-core')
const moduleUrl = text => 'data:text/javascript;base64,' + Buffer.from(ts.transpileModule(text, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
}).outputText).toString('base64')
const browserExecutable = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
const pwsh = 'C:\\Users\\15119\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\native\\powershell\\pwsh.exe'
const observer = path.join(repo, 'test/runtime/gpu-alpha-resource-observer-20261016T1636.ps1')
await write('binding.json', { source, referenceSha256: sha(reference), prototypeSha256: sha(prototype),
  priorPixelFailureRetained: true, prototypeAdopted: false, node: process.execPath, nodeVersion: process.version,
  order: ['reference-a', 'premultiplied-b', 'reference-a2'], maximumExperimentSeconds: 120,
  componentOnly: true, nativeProductStarted: false, nativeIdlePairRepeated: false,
  pinnedOfficialNodeExecuted: false, productionCodeChanged: false })
const http = createServer((_request, response) => {
  response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
  response.end('<!doctype html><html><head></head><body style="margin:0"></body></html>')
})
await new Promise(resolve => http.listen(0, '127.0.0.1', resolve))
const origin = `http://127.0.0.1:${http.address().port}`
const started = Date.now()
let currentServer
const rows = []
try {
  for (const name of ['reference-a', 'premultiplied-b', 'reference-a2']) {
    assert(Date.now() - started < 75000, 'Keep at least 45 seconds for one bounded stage')
    const stage = path.join(out, name)
    await fs.mkdir(stage)
    const server = await chromium.launchServer({ executablePath: browserExecutable, headless: true })
    currentServer = server
    const root = server.process()
    assert(root?.pid > 0)
    const browser = await chromium.connect(server.wsEndpoint())
    const context = await browser.newContext({ viewport: { width: 935, height: 759 }, deviceScaleFactor: 2 })
    const page = await context.newPage()
    await page.goto(origin)
    await page.evaluate(async ({ url }) => {
      const module = await import(url)
      const style = document.createElement('style')
      style.textContent = '.clip{position:absolute;inset:0;overflow:hidden}.host{position:absolute;top:-80px;left:0;width:935px;height:839px}.motion-mesh{position:absolute;inset:0;display:block;width:100%;height:100%;opacity:.6}'
      document.head.append(style)
      const clip = document.createElement('div'); clip.className = 'clip'
      const host = document.createElement('div'); host.className = 'host'
      clip.append(host); document.body.append(clip)
      const handle = module.createMeshGlow(host, { getDark: () => false, getCenterFraction: () => 0.26 })
      if (!handle) throw Error('Real WebGL is unavailable')
      const canvas = host.querySelector('canvas'), gl = canvas.getContext('webgl')
      const draw = gl.drawArrays.bind(gl)
      let frames = 0
      gl.drawArrays = (...args) => { frames++; return draw(...args) }
      const debug = gl.getExtension('WEBGL_debug_renderer_info')
      window.resourceProbe = { handle, capture: () => ({ frames, canvas: [canvas.width, canvas.height],
        attributes: gl.getContextAttributes(), renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null,
        contextLost: gl.isContextLost(), glError: gl.getError() }) }
    }, { url: moduleUrl(name === 'premultiplied-b' ? prototype : reference) })
    await page.waitForTimeout(4000)
    const before = await page.evaluate(() => window.resourceProbe.capture())
    assert(before.frames > 0 && !before.contextLost && before.glError === 0)
    await fs.writeFile(path.join(stage, 'scene-before.original.json'), JSON.stringify(before, null, 2) + '\n', { flag: 'wx' })
    let diagnosticError = ''
    const child = spawn(pwsh, ['-NoProfile', '-File', observer, '-BrowserProcessId', String(root.pid),
      '-BrowserExecutable', browserExecutable, '-OutputDirectory', stage], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] })
    child.stderr.on('data', data => { diagnosticError += data.toString() })
    const observerExit = new Promise((resolve, reject) => {
      child.once('error', reject); child.once('exit', code => resolve(code))
    })
    const readyAt = Date.now()
    while (true) {
      try { if ((await fs.stat(path.join(stage, 'snapshot-ready'))).size > 0) break } catch (error) { if (error.code !== 'ENOENT') throw error }
      assert(child.exitCode === null, `Snapshot observer stopped: ${diagnosticError.slice(0, 600)}`)
      assert(Date.now() - readyAt < 25000, 'Headless snapshot ready timeout')
      await new Promise(resolve => setTimeout(resolve, 100))
    }
    const after = await page.evaluate(() => window.resourceProbe.capture())
    assert(after.frames > before.frames && !after.contextLost && after.glError === 0)
    await fs.writeFile(path.join(stage, 'scene-after.original.json'), JSON.stringify(after, null, 2) + '\n', { flag: 'wx' })
    await context.close()
    await server.close()
    currentServer = undefined
    assert.equal(await observerExit, 0, diagnosticError.slice(0, 600))
    const snapshot = JSON.parse(await fs.readFile(path.join(stage, 'gpu-snapshot.original.json'), 'utf8'))
    const exit = JSON.parse(await fs.readFile(path.join(stage, 'normal-exit.original.json'), 'utf8'))
    assert.equal(exit.exitCode, 0)
    assert.equal(exit.nativeCount, 0)
    const record = { name, pid: root.pid, browserVersion: browser.version(), renderer: after.renderer,
      canvas: after.canvas, frames: [before.frames, after.frames], contextAttributes: after.attributes,
      gpuCounterReadStatus: snapshot.gpuCounterReadStatus,
      gpuInstances: snapshot.gpuCounterInstances?.length ?? null,
      summedOwnedSharedUsageBytes: snapshot.gpuCounterInstances?.reduce((sum, row) => sum + BigInt(row.SharedUsage ?? 0), 0n).toString() ?? null,
      summedOwnedWorkingSetBytes: snapshot.members.reduce((sum, member) => sum + member.workingSetBytes, 0),
      normalExitCode: exit.exitCode, kernelTimes: exit.kernelTimes }
    rows.push(record)
    console.log(JSON.stringify(record))
  }
  assert(Date.now() - started < 120000)
  await write('result.json', { status: 'completed-controlled-component-resources', elapsedMilliseconds: Date.now() - started,
    rows, isolatedBrowserOnly: true, productStarted: false, nativePerformanceGatePassed: false,
    missingGpuRowsAreNotZero: true, sharedResidentPagesDeduplicated: false, singleGlBuffersAttributed: false,
    priorPixelFailureRetained: true, prototypeAdopted: false, historicalFailuresCleared: false })
} catch (error) {
  await write('failure.json', { error: error.message, completedStages: rows, productStarted: false, automaticRetry: false })
  throw error
} finally {
  await currentServer?.close()
  await new Promise(resolve => http.close(resolve))
}
