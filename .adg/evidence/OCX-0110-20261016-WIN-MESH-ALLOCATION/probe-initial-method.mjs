// Background browser diagnostic. This does not run an acceptance EXE or measure RSS.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const ui = join(code, 'apps/desktop/ui')
const runtime = join(code, 'test/runtime/mesh-allocation-20261016')
const baseline = 'de6862d40389d527527b9db8ef52e186a58b625f'
const candidate = '8be87e2d54a895d4fe02e31300100a1ffed90121'
const edge = 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'
const { chromium } = await import(pathToFileURL(
  'C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright-core/index.mjs'))
const { createServer } = await import(pathToFileURL(join(ui, 'node_modules/vite/dist/node/index.js')))
const git = (...args) => execFileSync('git', ['-C', code, ...args])
assert.equal(git('rev-parse', 'HEAD').toString().trim(), candidate)
assert.equal(git('status', '--porcelain').toString(), '')
await mkdir(runtime, { recursive: false })
const graph = [
  'features/runtime/components/RuntimeMotionMark.vue',
  'features/runtime/motion/meshGlow.ts', 'features/runtime/motion/scene.ts',
  'features/runtime/motion/logo.ts', 'features/runtime/motion/states.ts',
  'features/runtime/motion/projection.ts',
]
const sources = {}
for (const mode of ['baseline019', 'candidate0110', 'deferred-diagnostic']) {
  const source = mode === 'baseline019' ? baseline : candidate
  const location = join(runtime, mode)
  for (const rel of graph) {
    let bytes = git('show', `${source}:apps/desktop/ui/src/${rel}`)
    if (mode === 'deferred-diagnostic' && rel.endsWith('RuntimeMotionMark.vue')) {
      let text = bytes.toString('utf8')
      assert(text.includes('onMounted(() => {'))
      text = text.replace('computed, onBeforeUnmount', 'computed, nextTick, onBeforeUnmount')
      text = text.replace('onMounted(() => {', 'onMounted(async () => {')
      text = text.replace(
        "  backdropTarget.value = ambientEl.value.closest<HTMLElement>('.route-section.ov')",
        "  backdropTarget.value = ambientEl.value.closest<HTMLElement>('.route-section.ov')\n" +
        "  await nextTick()\n  if (!ambientEl.value || !markEl.value) return")
      bytes = Buffer.from(text)
    }
    const destination = join(location, rel)
    await mkdir(dirname(destination), { recursive: true })
    await writeFile(destination, bytes, { flag: 'wx' })
    sources[`${mode}/${rel}`] = createHash('sha256').update(bytes).digest('hex')
  }
  const styles = ['tokens.css', 'base.css', 'materials.css', 'effects.css']
  for (const style of styles) {
    const bytes = git('show', `${source}:apps/desktop/ui/src/styles/${style}`)
    await mkdir(join(location, 'styles'), { recursive: true })
    await writeFile(join(location, 'styles', style), bytes, { flag: 'wx' })
    sources[`${mode}/styles/${style}`] = createHash('sha256').update(bytes).digest('hex')
  }
}
await writeFile(join(root, 'source-bindings.json'), JSON.stringify({
  baseline, candidate, sourceDigests: sources,
  rendererDependencies: 'Both use installed candidate Vue/Pinia/Vite; this is a layout diagnostic, not a release comparison',
  backgroundHeadlessBrowser: true, nativeApplicationStarted: false,
  deferredVariantIsTemporaryDiagnosticOnly: true, acceptancePassed: false,
}, null, 2) + '\n', { flag: 'wx' })

const probeEntry = '/__mesh_allocation.ts'
const server = await createServer({
  root: ui, configFile: join(ui, 'vite.config.ts'),
  server: { host: '127.0.0.1', port: 0, fs: { allow: [code] } },
  plugins: [{
    name: 'bounded-mesh-allocation-diagnostic',
    resolveId(id) { if (id === probeEntry) return '\0' + probeEntry },
    load(id) {
      if (id !== '\0' + probeEntry) return
      return `
        import {createApp, h, nextTick} from 'vue';
        import {createPinia, setActivePinia} from 'pinia';
        import {useEffectsStore} from '/src/app/appearance/effects.ts';
        import {useGlowRenderStore} from '/src/app/appearance/glowRender.ts';
        import {useThemeStore} from '/src/app/appearance/theme.ts';
        const p=new URLSearchParams(location.search),mode=p.get('mode');
        const base='/@fs/${runtime}/'+mode+'/';
        await Promise.all(['tokens','base','materials','effects'].map(name=>import(/* @vite-ignore */base+'styles/'+name+'.css')));
        const {default:Mark}=await import(/* @vite-ignore */base+'features/runtime/components/RuntimeMotionMark.vue');
        const pinia=createPinia();setActivePinia(pinia);
        useEffectsStore().setSetting(p.get('effects')||'high');
        useGlowRenderStore().setWebglSupported(true);useGlowRenderStore().setSetting('mesh');
        useThemeStore().apply(p.get('theme')||'light');
        document.documentElement.dataset.route='overview';
        document.documentElement.dataset.overviewSize=p.get('size')||'standard';
        const app=createApp({render:()=>h('div',{class:'app-window'},[
          h('aside',{class:'sidebar'},'diagnostic'),
          h('main',{class:'main'},h('section',{class:'route-section active ov','data-mode':'setup'},[
            h('header',{class:'topbar'},'diagnostic'),
            h('section',{class:'motion-overview'},h('div',{class:'motion-fixed'},[
              h('div',{class:'motion-stage'},h(Mark,{state:'not_ready',active:true})),
              h('div',null,'diagnostic')
            ]))
          ]))
        ])});
        app.use(pinia);app.mount('#app');await nextTick();window.__probeMounted=true;
        window.__probeUnmount=()=>app.unmount();
      `
    },
    configureServer(vite) {
      vite.middlewares.use(async (req, res, next) => {
        if (!req.url?.startsWith('/__mesh_allocation.html')) return next()
        const html = await vite.transformIndexHtml(req.url,
          '<!doctype html><html><head><meta charset="utf-8"></head><body><div id="app"></div>' +
          `<script type="module" src="${probeEntry}"></script></body></html>`)
        res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.end(html)
      })
    },
  }],
})
let browser
const results = []
try {
  await server.listen()
  const address = server.httpServer.address()
  browser = await chromium.launch({ executablePath: edge, headless: true })
  for (const effects of ['high', 'mid']) {
    for (const mode of ['baseline019', 'candidate0110', 'deferred-diagnostic']) {
      const context = await browser.newContext({
        viewport: { width: 1180, height: 760 }, deviceScaleFactor: 2,
      })
      await context.addInitScript(() => {
        window.__allocations = []
        window.__frameDraws = 0
        for (const dimension of ['width', 'height']) {
          const original = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, dimension)
          Object.defineProperty(HTMLCanvasElement.prototype, dimension, {
            ...original,
            set(value) {
              const old = this[dimension]
              original.set.call(this, value)
              if (this.classList.contains('motion-mesh')) window.__allocations.push({
                event: dimension, old, value, width: this.width, height: this.height,
                connected: this.isConnected, parent: this.parentElement?.className,
                teleported: this.closest('.motion-backdrop')?.parentElement?.className,
                at: performance.now(),
              })
            },
          })
        }
        const draw = WebGLRenderingContext.prototype.drawArrays
        WebGLRenderingContext.prototype.drawArrays = function (...args) {
          const result = draw.apply(this, args)
          if (this.canvas.classList.contains('motion-mesh')) window.__frameDraws++
          return result
        }
      })
      const page = await context.newPage()
      const errors = []
      page.on('pageerror', error => errors.push(String(error)))
      await page.goto(`http://127.0.0.1:${address.port}/__mesh_allocation.html?mode=${mode}&effects=${effects}`)
      await page.waitForFunction(() => window.__probeMounted &&
        document.querySelector('canvas.motion-mesh')?.width > 1, { timeout: 15000 })
      await page.waitForTimeout(1200)
      const observation = await page.evaluate(() => {
        const canvas = document.querySelector('canvas.motion-mesh')
        const bounds = canvas.getBoundingClientRect()
        const parent = canvas.parentElement.getBoundingClientRect()
        const gl = canvas.getContext('webgl')
        return {
          allocations: window.__allocations, draws: window.__frameDraws,
          canvas: { width: canvas.width, height: canvas.height,
            cssWidth: bounds.width, cssHeight: bounds.height, top: bounds.top,
            attributes: gl.getContextAttributes(), parentWidth: parent.width, parentHeight: parent.height },
          dpr: devicePixelRatio, meshCount: document.querySelectorAll('canvas.motion-mesh').length,
          dataEffects: document.documentElement.dataset.effects, finalParent:
            canvas.closest('.motion-backdrop')?.parentElement?.className ?? 'original-mark',
        }
      })
      assert.equal(errors.length, 0, errors.join('\n'))
      assert.equal(observation.meshCount, 1)
      await page.evaluate(() => window.__probeUnmount())
      const remainingCanvases = await page.locator('canvas.motion-mesh').count()
      assert.equal(remainingCanvases, 0)
      const result = { mode, effects, ...observation, errors, remainingCanvases }
      results.push(result)
      await writeFile(join(root, `${mode}-${effects}.original.json`),
        JSON.stringify(result, null, 2) + '\n', { flag: 'wx' })
      await context.close()
    }
  }
  await writeFile(join(root, 'results.json'), JSON.stringify({
    measuredAt: new Date().toISOString(), browserVersion: browser.version(),
    edge, results, nativeApplicationStarted: false, rssMeasured: false,
    fullPerformanceGatePassed: false,
  }, null, 2) + '\n', { flag: 'wx' })
  console.log(JSON.stringify(results.map(r => ({ mode: r.mode, effects: r.effects,
    changes: r.allocations.map(e => `${e.width}x${e.height}@${e.teleported}`),
    final: [r.canvas.width, r.canvas.height], draws: r.draws }))))
} finally {
  if (browser) await browser.close()
  await server.close()
}
