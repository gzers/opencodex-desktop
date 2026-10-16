// Bounded component pixels and context-creation order; no native product/RSS run.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import fs from 'node:fs/promises'
import path from 'node:path'
import { createRequire } from 'node:module'
import { execFileSync } from 'node:child_process'
import { createServer } from 'node:http'

const repo = 'C:\\Users\\15119\\.codex\\worktrees\\windows-mesh-initial-size\\opencodex-desktop'
const root = path.join(repo, 'test/runtime/mesh-initial-size-20261016T1857')
const out = path.join(root, 'pixel-check')
const source = 'b4d487e4e04cd4c80fd5570663e1ed21dd483b19'
const baseline = '823a8a0615153068594daef530c2e80f1d95fc29'
const relative = 'apps/desktop/ui/src/features/runtime/motion/meshGlow.ts'
assert.equal(execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim(), source)
assert.equal(execFileSync('git', ['status', '--porcelain', '--untracked-files=no'], { cwd: repo, encoding: 'utf8' }), '')
await fs.mkdir(out)
const sha = data => createHash('sha256').update(data).digest('hex')
const write = (name, value) => fs.writeFile(path.join(out, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' })
const before = execFileSync('git', ['show', `${baseline}:${relative}`], { cwd: repo, encoding: 'utf8' })
const after = execFileSync('git', ['show', `${source}:${relative}`], { cwd: repo, encoding: 'utf8' })
const requireUi = createRequire(path.join(repo, 'apps/desktop/ui/package.json'))
const ts = requireUi('typescript')
const requireTools = createRequire('C:\\Users\\15119\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\node\\node_modules\\playwright-core\\package.json')
const { chromium } = requireTools('playwright-core')
const moduleUrl = text => 'data:text/javascript;base64,' + Buffer.from(ts.transpileModule(text, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
}).outputText).toString('base64')
const executable = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
const scenes = [
  { name: 'standard-ready', width: 935, height: 840, dpr: 1, center: (98 + 70 + 80) / 840 },
  { name: 'standard-setup', width: 889.4, height: 840.2, dpr: 1.25, center: (87 + 45 + 80) / 840.2 },
  { name: 'compact-ready', width: 720, height: 720, dpr: 2, center: (69 + 50 + 80) / 720 },
  { name: 'compact-setup', width: 610.4, height: 560.2, dpr: 1.5, center: (83 + 37 + 80) / 560.2 },
]
await write('binding.json', {
  source, baseline, beforeSha256: sha(before), afterSha256: sha(after), node: process.execPath,
  nodeVersion: process.version, browserExecutable: executable, headlessOnly: true,
  nativeProductStarted: false, nativePerformanceRepeated: false, officialQueryNodeExecuted: false,
  scenes, timesSeconds: [0, 1.5, 4.5], themes: ['light', 'dark'],
  injectedTimeForPixelComparisonOnly: true, frameTimingMeasured: false, rssMeasured: false,
  originalRssFailureCleared: false, dpiAcceptanceAdded: false, automaticRetry: false,
})
const http = createServer((_request, response) => {
  response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
  response.end('<!doctype html><html><head></head><body style="margin:0"></body></html>')
})
await new Promise(resolve => http.listen(0, '127.0.0.1', resolve))
let server
let rootExit
const started = Date.now()
const rows = []
try {
  server = await chromium.launchServer({ executablePath: executable, headless: true })
  const process = server.process()
  assert(process?.pid > 0)
  rootExit = new Promise(resolve => process.once('exit', (code, signal) => resolve({ code, signal, pid: process.pid })))
  const browser = await chromium.connect(server.wsEndpoint())
  await write('browser.json', { pid: process.pid, version: browser.version(), ownChildExitListenerBound: true })
  for (const scene of scenes) {
    for (const dark of [false, true]) {
      const pair = {}
      for (const [variant, text] of [['before', before], ['after', after]]) {
        assert(Date.now() - started < 180000, 'Bounded pixel comparison exceeded its observation budget')
        const context = await browser.newContext({
          viewport: { width: Math.ceil(scene.width), height: Math.ceil(scene.height - 80) },
          deviceScaleFactor: scene.dpr,
        })
        try {
          const page = await context.newPage()
          await page.goto(`http://127.0.0.1:${http.address().port}`)
          const init = await page.evaluate(async ({ url, scene, dark }) => {
            window.requestAnimationFrame = () => 1
            window.cancelAnimationFrame = () => {}
            window.ResizeObserver = class { observe() {} disconnect() {} }
            const module = await import(url)
            document.body.style.background = dark ? '#151922' : '#f5f6fa'
            const style = document.createElement('style')
            style.textContent = `.clip{position:absolute;inset:0;overflow:hidden}.host{position:absolute;top:-80px;left:0;width:${scene.width}px;height:${scene.height}px}.motion-mesh{position:absolute;inset:0;display:block;width:100%;height:100%;opacity:${dark ? 0.52 : 0.6}}`
            document.head.append(style)
            const clip = document.createElement('div'); clip.className = 'clip'
            const host = document.createElement('div'); host.className = 'host'
            clip.append(host); document.body.append(clip)
            const getContext = HTMLCanvasElement.prototype.getContext
            let creationSize
            HTMLCanvasElement.prototype.getContext = function (...args) {
              if (args[0] === 'webgl') creationSize = [this.width, this.height]
              return getContext.apply(this, args)
            }
            const handle = module.createMeshGlow(host, { getDark: () => dark, getCenterFraction: () => scene.center })
            HTMLCanvasElement.prototype.getContext = getContext
            if (!handle) throw new Error('Real WebGL unavailable')
            handle.setActive(false)
            handle.setColors(['rgb(177,167,255)', 'rgb(122,157,255)', 'rgb(57,65,255)'])
            const canvas = host.querySelector('canvas')
            const gl = canvas.getContext('webgl')
            const debug = gl.getExtension('WEBGL_debug_renderer_info')
            const program = gl.getParameter(gl.CURRENT_PROGRAM)
            window.pixelCheck = { handle, canvas, gl, time: gl.getUniformLocation(program, 'u_time') }
            return { creationSize, finalSize: [canvas.width, canvas.height], attributes: gl.getContextAttributes(),
              renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null }
          }, { url: moduleUrl(text), scene, dark })
          const frames = []
          for (const seconds of [0, 1.5, 4.5]) {
            const raw = await page.evaluate(async seconds => {
              const { canvas, gl, time } = window.pixelCheck
              gl.uniform1f(time, seconds); gl.drawArrays(gl.TRIANGLES, 0, 3)
              const bytes = new Uint8Array(canvas.width * canvas.height * 4)
              gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, bytes)
              const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))
              return { seconds, rawSha256: Array.from(digest, b => b.toString(16).padStart(2, '0')).join(''),
                bytes: bytes.length, glError: gl.getError(), contextLost: gl.isContextLost() }
            }, seconds)
            assert.equal(raw.glError, 0); assert.equal(raw.contextLost, false)
            const png = await page.screenshot({ animations: 'disabled' })
            const name = `${scene.name}-${dark ? 'dark' : 'light'}-${variant}-${seconds}.png`
            await fs.writeFile(path.join(out, name), png, { flag: 'wx' })
            frames.push({ ...raw, png: name, pngBytes: png.length, pngSha256: sha(png) })
          }
          pair[variant] = { init, frames }
          await page.evaluate(() => window.pixelCheck.handle.destroy())
        } finally { await context.close() }
      }
      assert.deepEqual(pair.before.init.finalSize, pair.after.init.finalSize)
      assert.deepEqual(pair.before.init.attributes, pair.after.init.attributes)
      assert.deepEqual(pair.before.init.creationSize, [300, 150])
      assert.deepEqual(pair.after.init.creationSize, pair.after.init.finalSize)
      for (let i = 0; i < 3; i++) {
        assert.equal(pair.before.frames[i].rawSha256, pair.after.frames[i].rawSha256, 'Raw WebGL pixels differ')
        assert.equal(pair.before.frames[i].pngSha256, pair.after.frames[i].pngSha256, 'Composited screenshots differ')
      }
      rows.push({ scene: scene.name, dark, ...pair, allPixelsIdentical: true })
      await write(`scene-${scene.name}-${dark ? 'dark' : 'light'}.json`, rows.at(-1))
    }
  }
  await write('result.json', { status: 'identical-component-pixels-and-composition', scenes: 8, comparisons: 24,
    elapsedMilliseconds: Date.now() - started, rows, nativeProductStarted: false,
    performanceMeasured: false, nativeGpuAllocationAttributed: false, originalRssFailureCleared: false })
} catch (error) {
  await write('failure.json', { message: error.message, completedScenes: rows.length,
    automaticRetry: false, nativeProductStarted: false, performanceMeasured: false })
  throw error
} finally {
  if (server) {
    await server.close()
    let timer
    try {
      const exited = await Promise.race([rootExit, new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error('Own headless root exit not observed')), 15000)
      })])
      await write('browser-exit.json', { ...exited, forceTerminated: false,
        observationSource: 'exit event on the exact Node ChildProcess; no OS kernel FILETIME claimed',
        descendantCleanupFullyProven: false })
      assert.equal(exited.code, 0); assert.equal(exited.signal, null)
    } finally { clearTimeout(timer) }
  }
  await new Promise(resolve => http.close(resolve))
}
console.log(JSON.stringify({ comparisons: 24, rawPixelsIdentical: true, compositedPngBytesIdentical: true,
  headlessOnly: true, nativePerformanceMeasured: false }))
