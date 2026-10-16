// Compare retained pixels against an explicit committed full-buffer reference.
// This isolated headless rendering check does not start the desktop product.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import fs from 'node:fs/promises'
import path from 'node:path'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import { execFileSync } from 'node:child_process'
import { createServer } from 'node:http'

const ui = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repo = path.resolve(ui, '../../..')
const args = new Map()
for (let i = 2; i < process.argv.length; i += 2) args.set(process.argv[i], process.argv[i + 1])
const baseline = args.get('--baseline')
const out = path.resolve(args.get('--out') || '')
assert.match(baseline || '', /^[a-f0-9]{40}$/)
assert(out.startsWith(path.join(repo, 'test', 'runtime') + path.sep))
assert(process.env.PLAYWRIGHT_CORE && process.env.CHROMIUM, 'Explicit browser and Playwright paths required')
await fs.mkdir(out)
const sourcePath = 'apps/desktop/ui/src/features/runtime/motion/meshGlow.ts'
const original = execFileSync('git', ['show', `${baseline}:${sourcePath}`], { cwd: repo, encoding: 'utf8' })
const candidate = await fs.readFile(path.join(repo, sourcePath), 'utf8')
assert.notEqual(original, candidate, 'The reference must precede the proposed optimization')
const requireUi = createRequire(path.join(ui, 'package.json'))
const ts = requireUi('typescript')
const moduleUrl = source => 'data:text/javascript;base64,' + Buffer.from(ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
}).outputText).toString('base64')
const requireBrowser = createRequire(path.join(process.env.PLAYWRIGHT_CORE, 'package.json'))
const { chromium } = requireBrowser('playwright-core')
const geometries = [
  { name: 'observed-839', width: 935, height: 839, cropTop: 80, dpr: 2 },
  { name: 'standard-840', width: 935, height: 840, cropTop: 80, dpr: 2 },
  { name: 'compact-1x', width: 703, height: 660, cropTop: 80, dpr: 1 },
  { name: 'fractional-125', width: 703.25, height: 839.5, cropTop: 80, dpr: 1.25 },
]
const results = []
const server = createServer((_request, response) => {
  response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
  response.end('<!doctype html><html><body style="margin:0"></body></html>')
})
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
const origin = `http://127.0.0.1:${server.address().port}`
let browser
try {
  browser = await chromium.launch({ executablePath: process.env.CHROMIUM, headless: true })
  for (const geometry of geometries) {
    const context = await browser.newContext({
      viewport: { width: Math.ceil(geometry.width), height: Math.ceil(geometry.height) },
      deviceScaleFactor: geometry.dpr,
    })
    const page = await context.newPage()
    try {
      await page.goto(origin)
      await page.evaluate(async ({ originalUrl, candidateUrl, g }) => {
        const fullModule = await import(originalUrl)
        const clippedModule = await import(candidateUrl)
        let serial = 0
        const pending = new Map()
        window.requestAnimationFrame = callback => { pending.set(++serial, callback); return serial }
        window.cancelAnimationFrame = id => pending.delete(id)
        const flush = now => {
          const callbacks = [...pending.values()]
          pending.clear()
          callbacks.forEach(callback => callback(now))
        }
        const style = document.createElement('style')
        style.textContent = `.clip{position:absolute;top:0;left:0;overflow:hidden;width:${g.width}px;height:${g.height - g.cropTop}px}
          .host{position:absolute;top:-${g.cropTop}px;left:0;width:100%;height:${g.height}px}
          .motion-mesh{position:absolute;inset:0;display:block;width:100%;height:100%;opacity:.6}`
        document.head.append(style)
        let dark = false
        const make = (id, create, clipped) => {
          const clip = document.createElement('div')
          clip.id = id
          clip.className = 'clip'
          const host = document.createElement('div')
          host.className = 'host'
          clip.append(host)
          document.body.append(clip)
          const handle = create(host, {
            getDark: () => dark,
            getCenterFraction: () => 0.26,
            ...(clipped ? { clipTopAt: clip } : {}),
          })
          if (!handle) throw Error('Real WebGL context unavailable')
          const canvas = host.querySelector('canvas')
          const gl = canvas.getContext('webgl')
          const draw = gl.drawArrays.bind(gl)
          const item = { handle, clip, host, canvas, gl, pixels: null }
          gl.drawArrays = (...args) => {
            draw(...args)
            gl.finish()
            item.pixels = new Uint8Array(canvas.width * canvas.height * 4)
            gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, item.pixels)
            if (gl.getError() !== gl.NO_ERROR) throw Error('WebGL readback error')
          }
          return item
        }
        const reference = make('reference', fullModule.createMeshGlow, false)
        const clipped = make('candidate', clippedModule.createMeshGlow, true)
        const cases = []
        for (const nextDark of [false, true]) {
          dark = nextDark
          const colors = dark
            ? ['rgb(230,91,71)', 'rgb(175,123,232)', 'rgb(86,185,214)']
            : ['rgb(177,167,255)', 'rgb(122,157,255)', 'rgb(57,65,255)']
          for (const item of [reference, clipped]) {
            item.handle.setColors(colors)
            item.handle.setReduced(false)
          }
          for (const now of [1000, 1016, 1066]) {
            flush(now)
            const left = reference.pixels.subarray(0, clipped.pixels.length)
            const right = clipped.pixels
            let differentBytes = 0, maxDifference = 0, nonzeroBytes = 0
            for (let i = 0; i < right.length; i++) {
              const difference = Math.abs(left[i] - right[i])
              if (difference) differentBytes++
              if (left[i]) nonzeroBytes++
              maxDifference = Math.max(maxDifference, difference)
            }
            const hash = async bytes => [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))]
              .map(value => value.toString(16).padStart(2, '0')).join('')
            cases.push({ dark, frameTime: now, fullWidth: reference.canvas.width,
              fullHeight: reference.canvas.height, retainedWidth: clipped.canvas.width,
              retainedHeight: clipped.canvas.height, differentBytes, maxDifference, nonzeroBytes,
              rgbaSha256: [await hash(left), await hash(right)] })
          }
        }
        for (const item of [reference, clipped]) item.handle.setReduced(true)
        flush(2000)
        const rectangle = element => {
          const r = element.getBoundingClientRect()
          return { x: r.x, y: r.y, width: r.width, height: r.height, bottom: r.bottom }
        }
        const debug = clipped.gl.getExtension('WEBGL_debug_renderer_info')
        window.meshComparison = {
          cases, reference, clipped, flush,
          geometry: { reference: rectangle(reference.canvas), candidate: rectangle(clipped.canvas),
            clip: rectangle(clipped.clip), attributes: clipped.gl.getContextAttributes(),
            renderer: debug ? clipped.gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null },
          show(id) {
            for (const item of [reference, clipped]) {
              item.clip.style.background = '#f7f9fc'
              item.clip.style.visibility = item.clip.id === id ? 'visible' : 'hidden'
              item.handle.setActive(false)
              item.handle.setActive(true)
            }
            flush(2016)
          },
        }
      }, { originalUrl: moduleUrl(original), candidateUrl: moduleUrl(candidate), g: geometry })
      const record = await page.evaluate(() => ({
        cases: window.meshComparison.cases, geometry: window.meshComparison.geometry,
      }))
      for (const id of ['reference', 'candidate']) {
        await page.evaluate(id => window.meshComparison.show(id), id)
        await page.screenshot({
          path: path.join(out, `${geometry.name}-${id}.png`),
          clip: { x: 0, y: 0, width: geometry.width, height: geometry.height - geometry.cropTop },
        })
      }
      results.push({ input: geometry, ...record })
      await fs.writeFile(path.join(out, 'progress.json'), JSON.stringify({
        completedGeometries: results.length, phase: 'pixel-comparison',
      }, null, 2))
      console.log(`${geometry.name}: ${record.cases.length} retained-pixel comparisons recorded`)
    } finally { await context.close() }
  }
  const report = {
    baseline, originalSourceSha256: createHash('sha256').update(original).digest('hex'),
    candidateSourceSha256: createHash('sha256').update(candidate).digest('hex'),
    nodeVersion: process.version, browserVersion: browser.version(), results,
    hostProductStarted: false, nativePerformanceGatePassed: false,
    foregroundInputInjected: false,
  }
  await fs.writeFile(path.join(out, 'pixels.original.json'), JSON.stringify(report, null, 2))
  for (const result of results) for (const test of result.cases) {
    assert(test.nonzeroBytes > 0)
    assert.equal(test.differentBytes, 0)
    assert.equal(test.maxDifference, 0)
    assert.equal(test.rgbaSha256[0], test.rgbaSha256[1])
    assert.equal(test.fullWidth, test.retainedWidth)
    assert(test.retainedHeight < test.fullHeight)
  }
  console.log(`${results.length * 6} real WebGL cases retained identical RGBA; compositor PNGs require separate verification.`)
} finally {
  await browser?.close()
  await new Promise(resolve => server.close(resolve))
}
