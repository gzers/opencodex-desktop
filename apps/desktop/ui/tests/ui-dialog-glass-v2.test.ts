// @vitest-environment node
// Actual browser pixels, including the macOS compositor: WebKit's Page.snapshotRect
// omits backdrop filters even though the headed browser paints them (Playwright #28363).
import { afterAll, beforeAll, describe, expect, it } from 'vitest'
import { createServer, type ViteDevServer } from 'vite'
import vue from '@vitejs/plugin-vue'
import { createRequire } from 'node:module'
import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ui = fileURLToPath(new URL('..', import.meta.url))
const prefix = path.join(process.env.OCXD_GLASS_EVIDENCE_DIR ?? tmpdir(), 'ocxd023-dialog-glass-')
const evidence: unknown[] = []
const stage = process.env.OCXD_GLASS_STAGE ?? 'after'
const engines = ['chromium', 'webkit'] as const
const cases = ['light', 'dark'].flatMap(theme => ['high', 'mid', 'low'].flatMap(effects =>
  [1180, 560].map(width => ({ theme, effects, width, name: theme + ' ' + effects + ' ' + (width > 600 ? 'wide' : 'narrow') }))))
let server: ViteDevServer
let base = ''
// Use the declared, locked test tool on both clean CI and developer machines.
function playwright(): any {
  const api = createRequire(import.meta.url)('playwright-core')
  if (engines.every(engine => existsSync(api[engine].executablePath()))) return api
  throw new Error('Real Chromium and WebKit builds are required. Run npm exec -- playwright-core install chromium webkit in apps/desktop/ui.')
}
const swift = prefix + 'owned-window.swift'
const entry = [
  "import {createApp,h,ref} from 'vue'",
  "import UiDialog from '" + ui + "/src/components/ui/UiDialog.vue'",
  ...['tokens', 'base', 'materials', 'effects'].map(name => "import '" + ui + '/src/styles/' + name + ".css'"),
  "const first=ref(true),second=ref(false),long=ref(false),busy=ref(false),guard=ref(true),footer=ref(true),wide=ref(false)",
  "window.fixture={first,second,long,busy,guard,footer,wide}",
  "const body=id=>[h('div',{class:'probe'}),h('p',{class:'front-text'},'前景文字 /actual/data 路径应清楚可读'),h('p',{class:'secondary-text'},'渠道配置文件 · 次要说明同样需要可读'),h('input',{id:id+'-input',value:'explicit path'}),...(long.value?Array.from({length:40},(_,i)=>h('p',{class:'long-row'},'完整路径 '+i+' /actual/long directory/file.json')):[])]",
  "createApp({setup:()=>()=>[h('button',{id:'origin',onClick:()=>first.value=true},'打开第一层'),h(UiDialog,{open:first.value,title:'完整路径 · Native Dialog',id:'dialog-one',wide:wide.value,busy:busy.value,beforeClose:()=>guard.value,onClose:()=>first.value=false},{default:()=>body('one'),actions:()=>footer.value?[h('button',{id:'open-second',onClick:()=>second.value=true},'打开第二层'),h('button',{id:'confirm-one'},'确认')]:null}),h(UiDialog,{open:second.value,title:'第二层完整路径',id:'dialog-two',beforeClose:()=>guard.value,onClose:()=>second.value=false},{default:()=>body('two'),actions:()=>h('button',{id:'confirm-two'},'确认')})]}).mount('#app')",
].join('\n')
const html = '<!doctype html><html data-theme="dark" data-effects="high"><head><meta charset="utf-8"><title>ocxd023-dialog-glass-fixture</title><style>' +
  'html,body{margin:0;height:100%}#pattern{position:fixed;inset:0;background:repeating-linear-gradient(90deg,#000 0 64px,#fff 64px 128px)}' +
  '#colored-grid{position:fixed;left:0;top:0;bottom:0;width:128px;background:repeating-conic-gradient(#14d6bb 0 25%,#c83785 0 50%) 0/64px 64px}' +
  '#behind-text{position:fixed;top:57vh;font:900 32px/1.4 monospace;color:white;mix-blend-mode:difference}' +
  '.probe{height:150px;width:100%}.front-text,.secondary-text,.long-row{font-size:13px}.secondary-text{color:var(--muted)}.ui-dialog-content{min-height:250px}' +
  '</style></head><body><div id="pattern"></div><div id="colored-grid"></div><div id="behind-text">BEHIND TEXT 背景高对比文字 ABCD 012345</div><div id="app"></div><script type="module" src="/__dialog-glass-entry.ts"></script></body></html>'

beforeAll(async () => {
  writeFileSync(swift, 'import Foundation\nimport CoreGraphics\nlet ws=CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []\nlet own=ws.filter{($0[kCGWindowName as String] as? String ?? "").contains("ocxd023-dialog-glass-fixture") && ($0[kCGWindowLayer as String] as? Int ?? -1)==0}\nprint(String(data:try JSONSerialization.data(withJSONObject:own),encoding:.utf8)!)\n')
  server = await createServer({ root: ui, configFile: false, plugins: [vue(), {
    name: 'owned-dialog-glass-fixture',
    configureServer(s) { s.middlewares.use((req, res, next) => {
      if (req.url?.startsWith('/__dialog-glass?') || req.url === '/__dialog-glass') { res.setHeader('Content-Type', 'text/html'); res.end(html) } else next()
    }) },
    resolveId(id) { if (id === '/__dialog-glass-entry.ts') return '\0dialog-glass-entry' },
    load(id) { if (id === '\0dialog-glass-entry') return entry },
  }], resolve: { alias: { '@': path.join(ui, 'src'), vue: path.join(ui, 'node_modules/vue/dist/vue.runtime.esm-bundler.js') } },
  optimizeDeps: { exclude: ['vue'] }, server: { host: '127.0.0.1', port: 0, hmr: false, watch: null } })
  await server.listen(); base = server.resolvedUrls!.local[0]!
}, 60000)
afterAll(async () => {
  writeFileSync(prefix + stage + '-results.json', JSON.stringify(evidence, null, 2))
  await server?.close()
})

// Only capture the titled task-owned browser window, never the user's desktop.
async function capture(page: any, name: string): Promise<{ buffer: Buffer; xScale: number; top: number; physical: boolean }> {
  const file = prefix + stage + '-' + name + '.png'
  if (process.platform === 'darwin') {
    const windows = JSON.parse(execFileSync('swift', [swift], { encoding: 'utf8' }))
    expect(windows.length, 'one task-owned window').toBe(1)
    const window = windows[0], bounds = window.kCGWindowBounds
    execFileSync('/usr/sbin/screencapture', ['-x', '-o', '-l', String(window.kCGWindowNumber), file])
    const buffer = readFileSync(file)
    const dimensions = await page.evaluate(async (b64: string) => {
      const img = new Image(); img.src = 'data:image/png;base64,' + b64; await img.decode(); return { width: img.width, height: img.height }
    }, buffer.toString('base64'))
    return { buffer, xScale: dimensions.width / bounds.Width, top: bounds.Height - page.viewportSize().height, physical: true }
  }
  return { buffer: await page.screenshot({ path: file }), xScale: 1, top: 0, physical: false }
}
async function pixels(page: any, shot: Awaited<ReturnType<typeof capture>>, selector: string) {
  const box = await page.locator(selector + ' .probe').boundingBox()
  const outer = await page.locator(selector).boundingBox()
  const colors = await page.locator(selector).evaluate((d: HTMLElement) => ({
    text: getComputedStyle(d.querySelector('.front-text')!).color,
    secondary: getComputedStyle(d.querySelector('.secondary-text')!).color,
    root: getComputedStyle(d).getPropertyValue('backdrop-filter'), prefixed: getComputedStyle(d).getPropertyValue('-webkit-backdrop-filter'),
    background: getComputedStyle(d).backgroundColor, zoom: getComputedStyle(d).zoom,
    backdrop: getComputedStyle(d, '::backdrop').backgroundColor,
  }))
  return page.evaluate(async ({ b64, box, outer, scale, top, colors }: any) => {
    const image = new Image(); image.src = 'data:image/png;base64,' + b64; await image.decode()
    const canvas = document.createElement('canvas'); canvas.width = image.width; canvas.height = image.height
    const ctx = canvas.getContext('2d')!; ctx.drawImage(image, 0, 0)
    const y = Math.round((top + box.y + 25) * scale), x = Math.ceil((box.x + 12) * scale)
    const n = Math.floor((box.width - 24) * scale), data = ctx.getImageData(x, y, n, 1).data
    const luminance = (rgb: number[]) => rgb.map(v => v / 255).map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4).reduce((s, v, i) => s + v * [.2126, .7152, .0722][i]!, 0)
    const lum: number[] = [], gray: number[] = []
    for (let i = 0; i < data.length; i += 4) { const rgb = [data[i]!, data[i + 1]!, data[i + 2]!]; lum.push(luminance(rgb)); gray.push(rgb.reduce((a, b) => a + b, 0) / 3) }
    const min = Math.min(...gray), max = Math.max(...gray), range = max - min
    const maxStep = Math.max(...gray.slice(1).map((v, i) => Math.abs(v - gray[i]!)))
    const outside = ctx.getImageData(Math.ceil(140 * scale), Math.round((top + Math.max(2, outer.y - 20)) * scale), Math.floor((innerWidth - 152) * scale), 1).data
    const outsideGray: number[] = []
    for (let i = 0; i < outside.length; i += 4) outsideGray.push((outside[i]! + outside[i + 1]! + outside[i + 2]!) / 3)
    const outsideRange = Math.max(...outsideGray) - Math.min(...outsideGray)
    const outsideStep = Math.max(...outsideGray.slice(1).map((v, i) => Math.abs(v - outsideGray[i]!)))
    const ratios = (color: string) => { ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = color; ctx.fillRect(0, 0, 1, 1); const rgb = Array.from(ctx.getImageData(0, 0, 1, 1).data).slice(0, 3); const fg = luminance(rgb); return Math.min(...lum.map(bg => (Math.max(bg, fg) + .05) / (Math.min(bg, fg) + .05))) }
    return { range, maxStep, outsideNormalizedStep: outsideStep / outsideRange, normalizedStep: range > 1 ? maxStep / range : 0, textContrast: ratios(colors.text), secondaryContrast: ratios(colors.secondary), colors, physicalScale: scale, physicalTop: top }
  }, { b64: shot.buffer.toString('base64'), box, outer, scale: shot.xScale, top: shot.top, colors })
}

for (const engine of engines) describe(engine + ' native dialog glass', () => {
  let browser: any, page: any
  beforeAll(async () => {
    const api = playwright()
    browser = await api[engine].launch({ headless: process.platform !== 'darwin' })
    page = await browser.newPage({ viewport: { width: 1180, height: 900 }, deviceScaleFactor: 1 })
  }, 60000)
  afterAll(async () => { await browser?.close() })
  for (const scenario of cases) it(scenario.name + ' blurs actual background and preserves readable contrast', async () => {
    await page.setViewportSize({ width: scenario.width, height: 900 })
    await page.goto(base + '__dialog-glass?case=' + encodeURIComponent(scenario.name))
    await page.waitForSelector('#dialog-one[open]')
    await page.evaluate(({ theme, effects }: any) => { document.documentElement.dataset.theme = theme; document.documentElement.dataset.effects = effects }, scenario)
    await page.waitForTimeout(120)
    const name = engine + '-' + scenario.name.replaceAll(' ', '-')
    const shot = await capture(page, name)
    const measured = await pixels(page, shot, '#dialog-one')
    evidence.push({ engine, browserVersion: await browser.version(), ...scenario, capture: shot.physical ? 'owned-window-compositor' : 'browser-screenshot', measured })
    const modal = await page.locator('#dialog-one').evaluate((d: HTMLDialogElement) => ({ native: d.matches(':modal'), x: d.getBoundingClientRect().x, right: d.getBoundingClientRect().right, width: innerWidth, effects: document.documentElement.dataset.effects }))
    expect(modal.native).toBe(true); expect(modal.x).toBeGreaterThanOrEqual(0); expect(modal.right).toBeLessThanOrEqual(modal.width + 1); expect(modal.effects).toBe(scenario.effects)
    if (scenario.effects === 'low') {
      expect(measured.range, 'low effects use the solid recipe').toBeLessThan(2)
      expect(measured.colors.root || measured.colors.prefixed).toBe('none')
    } else {
      expect(measured.range, 'glass retains measurable low-frequency background variation').toBeGreaterThan(8)
      expect(measured.normalizedStep, 'pixel edge is blurred rather than a sharp alpha-tinted step').toBeLessThan(.18)
    }
    expect(measured.outsideNormalizedStep, 'background outside the dialog remains sharp').toBeGreaterThan(.35)
    expect(measured.textContrast, '13px foreground contrast').toBeGreaterThanOrEqual(4.5)
    expect(measured.secondaryContrast, '13px secondary text contrast').toBeGreaterThanOrEqual(4.5)
  }, 30000)

  it('keeps the existing wide dialog size and material', async () => {
    await page.setViewportSize({ width: 1180, height: 900 }); await page.goto(base + '__dialog-glass')
    await page.waitForSelector('#dialog-one[open]')
    await page.evaluate(() => { (window as any).fixture.wide.value = true })
    await expect.poll(() => page.locator('#dialog-one').evaluate((d: HTMLElement) => d.getBoundingClientRect().width)).toBe(850)
    const shot = await capture(page, engine + '-wide-dialog')
    const measured = await pixels(page, shot, '#dialog-one')
    expect(measured.normalizedStep).toBeLessThan(.18)
    expect(measured.textContrast).toBeGreaterThanOrEqual(4.5); expect(measured.secondaryContrast).toBeGreaterThanOrEqual(4.5)
    evidence.push({ engine, wideDialog: measured })
  }, 30000)

  it('retains stack, scroll, footer, X, focus, zoom, busy and close guards', async () => {
    await page.setViewportSize({ width: 560, height: 820 }); await page.goto(base + '__dialog-glass')
    await page.waitForSelector('#dialog-one[open]'); await page.evaluate(() => {
      const f = (window as any).fixture; f.long.value = true
      document.documentElement.style.setProperty('--ui-zoom', '1.25')
    })
    const close = page.locator('#dialog-one header button[aria-label="关闭"]'), footer = page.locator('#dialog-one footer')
    const before = await close.boundingBox(), footerBefore = await footer.boundingBox()
    await page.locator('#dialog-one .ui-dialog-content').evaluate((e: HTMLElement) => { e.scrollTop = e.scrollHeight })
    expect(await page.locator('#dialog-one .ui-dialog-content').evaluate((e: HTMLElement) => e.scrollTop)).toBeGreaterThan(500)
    const after = await close.boundingBox(), footerAfter = await footer.boundingBox()
    expect(Math.abs(after.y - before.y)).toBeLessThan(1); expect(Math.abs(after.x - before.x)).toBeLessThan(1)
    expect(Math.abs(footerAfter.y - footerBefore.y)).toBeLessThan(1); expect(footerAfter.y + footerAfter.height).toBeLessThanOrEqual(821)
    await page.locator('#dialog-one .ui-dialog-content').evaluate((e: HTMLElement) => { e.style.background = 'repeating-linear-gradient(90deg, #000 0 64px, #fff 64px 128px)' })
    await page.locator('#open-second').click(); await page.waitForSelector('#dialog-two[open]')
    expect(await page.locator('#two-input').evaluate((e: HTMLElement) => document.activeElement === e)).toBe(true)
    const stacked = await capture(page, engine + '-stack-zoom-scroll')
    const stackedPixels = await pixels(page, stacked, '#dialog-two')
    expect(stackedPixels.range).toBeGreaterThan(8)
    expect(stackedPixels.normalizedStep).toBeLessThan(.18)
    await page.locator('#dialog-two').evaluate((e: HTMLElement) => { e.style.setProperty('-webkit-backdrop-filter', 'none'); e.style.setProperty('backdrop-filter', 'none') })
    const unfiltered = await capture(page, engine + '-stack-no-filter-control')
    const controlPixels = await pixels(page, unfiltered, '#dialog-two')
    expect(stackedPixels.normalizedStep).toBeLessThan(controlPixels.normalizedStep * .4)
    await page.locator('#dialog-two').evaluate((e: HTMLElement) => { e.style.removeProperty('-webkit-backdrop-filter'); e.style.removeProperty('backdrop-filter') })
    evidence.push({ engine, stackedPixels, controlPixels })
    await page.evaluate(() => { (window as any).fixture.guard.value = false })
    await page.keyboard.press('Escape'); expect(await page.locator('#dialog-two').evaluate((d: HTMLDialogElement) => d.open)).toBe(true)
    await page.evaluate(() => { (window as any).fixture.guard.value = true })
    await page.keyboard.press('Escape'); await page.waitForSelector('#dialog-two:not([open])', { state: 'attached' })
    expect(await page.locator('#dialog-one').evaluate((d: HTMLDialogElement) => d.open)).toBe(true)
    expect(await page.locator('#open-second').evaluate((e: HTMLElement) => document.activeElement === e)).toBe(true)
    await page.evaluate(() => { (window as any).fixture.busy.value = true })
    await page.keyboard.press('Escape'); expect(await page.locator('#dialog-one').evaluate((d: HTMLDialogElement) => d.open)).toBe(true)
    await page.evaluate(() => { const f = (window as any).fixture; f.busy.value = false; f.footer.value = false })
    await expect.poll(() => page.locator('#dialog-one footer').count()).toBe(0)
    expect(await close.isVisible()).toBe(true)
    await page.keyboard.press('Escape'); await page.waitForSelector('#dialog-one:not([open])', { state: 'attached' })
    expect(await page.evaluate(() => document.documentElement.style.getPropertyValue('--ui-zoom'))).toBe('1.25')
    const fallback = await page.evaluate(() => {
      const supports = [...document.styleSheets].flatMap(sheet => [...sheet.cssRules]).filter(rule => rule instanceof CSSSupportsRule) as CSSSupportsRule[]
      return supports.some(rule => rule.conditionText.startsWith('not ((backdrop-filter:') && [...rule.cssRules].some(child => child instanceof CSSStyleRule && child.style.getPropertyValue('--surface-overlay-dialog-bg').trim() === 'var(--panel)' && child.style.getPropertyValue('--surface-overlay-dialog-muted').trim() === 'var(--muted)'))
    })
    expect(fallback, 'unsupported-backdrop branch uses the existing solid recipe').toBe(true)
    evidence.push({ engine, nativeBehavior: 'stack/scroll/footer/X/focus/zoom/guard/busy/view-footer passed' })
  }, 30000)
})
