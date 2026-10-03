// 原型 ↔ 实现配对：概览「环境准备（setup）」形态等**重点状态**（IMP-05 §5.2）。
//
// 与 `prototype-parity-pages.browser.mjs` 同法（进程内静态服务 + 裁剪抓取），
// 覆盖概览双形态中的 setup 形态 × 2 窗口 × 2 主题 × 3 档 = 12 组合。
// 输出 `.adg/work/prototype-parity-20261001/screens-states/<state>/<combo>/{prototype,implementation}.png`。
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import http from 'http'
import { spawn } from 'child_process'
import { createRequire } from 'module'
import { fileURLToPath } from 'url'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const repoRoot = path.resolve(uiDir, '..', '..', '..')
const outRoot = path.join(repoRoot, '.adg', 'work', 'prototype-parity-20261001', 'screens-states')
fs.mkdirSync(outRoot, { recursive: true })

const PROTOTYPE_REL =
  'docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/候选/2026-09-28-Logo本体形变/overview.html'

function resolvePlaywright() {
  const candidates = [process.env.PLAYWRIGHT_CORE]
  const npx = path.join(os.homedir(), '.npm', '_npx')
  if (fs.existsSync(npx)) for (const d of fs.readdirSync(npx)) candidates.push(path.join(npx, d, 'node_modules', 'playwright-core'))
  for (const c of candidates) if (c && fs.existsSync(path.join(c, 'package.json'))) return c
  return null
}
function newestChromiumShell() {
  const base = path.join(os.homedir(), 'Library', 'Caches', 'ms-playwright')
  if (!fs.existsSync(base)) return null
  for (const d of fs.readdirSync(base).filter(x => x.startsWith('chromium_headless_shell-')).sort().reverse()) {
    const p = path.join(base, d, 'chrome-headless-shell-mac-arm64', 'chrome-headless-shell')
    if (fs.existsSync(p)) return p
  }
  return null
}
const pwPath = resolvePlaywright()
const exe = process.env.CHROMIUM || newestChromiumShell()
if (!pwPath || !exe) { console.log('BLOCKED: 环境缺失（playwright-core / chromium headless shell）。'); process.exit(2) }
const require = createRequire(pwPath + '/')
const { chromium } = require('playwright-core')

const APP_PORT = Number(process.env.PARITY_APP_PORT || 5198)
const PROTO_PORT = Number(process.env.PARITY_PROTO_PORT || 5197)
const APP = `http://127.0.0.1:${APP_PORT}`
const PROTO = `http://127.0.0.1:${PROTO_PORT}`
const only = process.env.STATES_CASE || ''

function waitPort(port, timeoutMs = 30000) {
  const start = Date.now()
  return new Promise((resolve, reject) => {
    const tick = () => {
      const sock = net.connect(port, '127.0.0.1')
      sock.once('connect', () => { sock.destroy(); resolve() })
      sock.once('error', () => { sock.destroy(); if (Date.now() - start > timeoutMs) reject(new Error('端口未就绪 ' + port)); else setTimeout(tick, 200) })
    }
    tick()
  })
}

// 概览「重点状态」矩阵（IMP-05 §5.2）：双形态 setup 跑全 12 组合；
// 运行主线各态在代表性组合（1180×760 light/dark high）取证即可。
const RUNTIME_STATES = ['loading', 'not_found', 'stopped', 'starting', 'pending', 'running', 'starting_failed', 'at_risk', 'external_takeover', 'unreachable']
const REP_COMBOS = ['1180x760-light-high', '1180x760-dark-high']
// 设置 10 个分区（原型 `showSettingsSection` / 实现 `?route=settings&section=`），两者分区集合一致。
const SETTINGS_SECTIONS = ['general', 'installation', 'backup', 'extensions', 'migration', 'sync', 'cleanup', 'cli', 'upgrade', 'about']
// 诊断中心 3 个 Tab（原型 `activateDiagTab` / 实现 `?route=logs&tab=`）。
const DIAG_TABS = ['doctor', 'logs', 'notifications']
const STATES = [
  { name: 'overview-setup', combos: 'all', proto: { env: 'missing_node_brew', state: 'stopped' }, impl: { env: 'missing_node', brew: '1', runtime: 'stopped' } },
  ...RUNTIME_STATES.map(s => ({
    name: `overview-runtime-${s}`, combos: REP_COMBOS,
    proto: { env: 'ready', state: s }, impl: { env: 'ready', runtime: s },
  })),
  ...SETTINGS_SECTIONS.map(sec => ({
    name: `settings-${sec}`, combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'settings', section: sec },
    impl: { env: 'ready', runtime: 'running', route: 'settings', section: sec },
  })),
  ...DIAG_TABS.map(tab => ({
    name: `diagnostics-${tab}`, combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'logs', tab },
    impl: { env: 'ready', runtime: 'running', route: 'logs', tab },
  })),
  ...['skills', 'mcp'].map(tab => ({
    name: `extensions-${tab}`, combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'extensions', js: `document.querySelector('.ext-tabs button[data-ext-tab="${tab}"]')?.click()` },
    impl: { env: 'ready', runtime: 'running', route: 'extensions', tab },
  })),
  // 反馈层（浮层/宿主）：通知面板打开、Toast。
  {
    name: 'feedback-notifications', combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'overview', js: "window.renderNotifications&&window.renderNotifications();document.querySelector('.notification-panel')?.classList.add('show')" },
    impl: { env: 'ready', runtime: 'running', route: 'overview', panel: '1' },
  },
  {
    name: 'feedback-toast', combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'overview', js: "window.toast&&window.toast('偏好已保存（夹具）')" },
    impl: { env: 'ready', runtime: 'running', route: 'overview', toast: '1' },
  },
  // 运行详情弹窗（§5.2「运行详情：默认及紧凑布局」）：就绪/未运行两态 × 标准/紧凑窗口。
  ...['running', 'stopped'].map(st => ({
    name: `runtime-detail-${st}`, combos: ['1180x760-light-high', '900x600-light-high'],
    proto: { env: 'ready', state: st, route: 'overview', js: "document.querySelector('.motion-detail-icon')?.click()" },
    impl: { env: 'ready', runtime: st, route: 'overview', js: "document.querySelector('.motion-mainline')?.click()" },
  })),
  // 安装 / 卸载向导（§5.2「设置与向导」）：安装弹窗、卸载确认弹窗。
  {
    name: 'wizard-install', combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'settings', section: 'installation', js: "window.openRuntimeInstall&&window.openRuntimeInstall('net')" },
    impl: { env: 'ready', route: 'settings', section: 'installation', runtime: 'stub', js: "document.querySelector('[data-testid=\"runtime-install\"]')?.click()" },
  },
  {
    name: 'wizard-uninstall', combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'settings', section: 'installation', js: "window.openRuntimeUninstall&&window.openRuntimeUninstall()" },
    impl: { env: 'ready', route: 'settings', section: 'installation', runtime: 'stub', js: "document.querySelector('[data-testid=\"runtime-uninstall\"]')?.click()" },
  },
  // 托盘动作（§5.2「面板与托盘」）：托盘按运行事实逐态表现（实现 TrayRoute 直接消费 runtimeState）。
  ...['running', 'stopped', 'starting_failed', 'at_risk'].map(st => ({
    name: `tray-state-${st}`, combos: REP_COMBOS,
    proto: { env: 'ready', state: st, route: 'tray' },
    impl: { env: 'ready', runtime: st, route: 'tray' },
  })),
  // 扩展条目详情（长 Markdown，§5.2「扩展：长 Markdown、只读详情」）：两侧同为 `.skill-info[data-ext-detail]` 触发。
  {
    name: 'extensions-detail-markdown', combos: REP_COMBOS,
    proto: { env: 'ready', state: 'running', route: 'extensions', js: "document.querySelector('.skill-info[data-ext-detail]')?.click()" },
    impl: { env: 'ready', runtime: 'running', route: 'extensions', scenario: 'markdown', js: "document.querySelector('.skill-info[data-ext-detail]')?.click()" },
  },
]

const combos = []
for (const win of [{ w: 1180, h: 760 }, { w: 900, h: 600 }])
  for (const theme of ['light', 'dark'])
    for (const effects of ['high', 'mid', 'low']) combos.push({ win, theme, effects })

async function captureEl(page, selector, outPath) {
  await page.evaluate(() => window.scrollTo(0, 0))
  const box = await page.evaluate(sel => {
    const el = document.querySelector(sel); if (!el) return null
    const b = el.getBoundingClientRect(); if (b.width < 1 || b.height < 1) return null
    return { x: Math.round(b.x), y: Math.round(b.y), width: Math.round(b.width), height: Math.round(b.height) }
  }, selector)
  if (!box) throw new Error(`元素不可见或缺失：${selector}`)
  await page.screenshot({ path: outPath, clip: box })
}

const summary = []
// 可续跑：已存在的组合默认跳过（除非 STATES_FORCE=1），摘要按 (state,name) 合并写回。
const summaryPath = path.join(outRoot, 'summary.json')
const prevByKey = new Map()
try {
  for (const item of JSON.parse(fs.readFileSync(summaryPath, 'utf8'))) prevByKey.set(`${item.state}/${item.name}`, item)
} catch { /* 首次运行无摘要 */ }
const FORCE = process.env.STATES_FORCE === '1'

async function main() {
  const app = spawn(path.join(uiDir, 'node_modules', '.bin', 'vite'), ['--host', '127.0.0.1', '--port', String(APP_PORT), '--strictPort'], { cwd: uiDir, stdio: 'ignore' })
  const MIME = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.png': 'image/png', '.json': 'application/json', '.svg': 'image/svg+xml' }
  const protoSrv = http.createServer((req, res) => {
    try {
      const rel = decodeURIComponent(req.url.split('?')[0]).replace(/^\/+/, '')
      const file = path.resolve(repoRoot, rel)
      if (!file.startsWith(repoRoot) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end('nf'); return }
      res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] || 'application/octet-stream' })
      fs.createReadStream(file).pipe(res)
    } catch { try { res.writeHead(500); res.end('err') } catch {} }
  })
  protoSrv.listen(PROTO_PORT, '127.0.0.1')
  const stop = () => { try { app.kill('SIGTERM') } catch {}; try { protoSrv.close() } catch {} }
  process.on('exit', stop)

  let browser
  try {
    await waitPort(APP_PORT); await waitPort(PROTO_PORT)
    browser = await chromium.launch({ executablePath: exe })
    for (const state of STATES) {
      for (const combo of combos) {
        const name = `${combo.win.w}x${combo.win.h}-${combo.theme}-${combo.effects}`
        if (state.combos !== 'all' && !state.combos.includes(name)) continue
        if (only && !(state.name.includes(only) || name.includes(only))) continue
        const dir = path.join(outRoot, state.name, name); fs.mkdirSync(dir, { recursive: true })
        const key = `${state.name}/${name}`
        const havePair = fs.existsSync(path.join(dir, 'prototype.png')) && fs.existsSync(path.join(dir, 'implementation.png'))
        if (havePair && !FORCE) {
          if (prevByKey.has(key)) summary.push(prevByKey.get(key))
          else {
            // 摘要缺失（被单点冒烟覆盖）时，从磁盘 geometry.json 重建，避免丢条目。
            try {
              const g = JSON.parse(fs.readFileSync(path.join(dir, 'geometry.json'), 'utf8'))
              summary.push({
                state: g.state, name: g.name,
                route: [g.prototype?.route, g.implementation?.route],
                mode: [g.prototype?.mode, g.implementation?.mode],
                mainline: [g.prototype?.mainline, g.implementation?.mainline],
                dialog: [g.prototype?.dialog, g.implementation?.dialog],
                overlay: [g.prototype?.overlay, g.implementation?.overlay],
              })
            } catch { /* 无 geometry.json 则跳过 */ }
          }
          continue
        }
        let p1 = null, p2 = null
        try {
          p1 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h + 60 }, deviceScaleFactor: 2 })
          await p1.route('**', r => { const u = r.request().url(); (u.startsWith(PROTO) || u.startsWith('data:') || u.startsWith('blob:')) ? r.continue() : r.abort() })
          await p1.goto(`${PROTO}/${encodeURI(PROTOTYPE_REL)}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
          await p1.waitForSelector('.window', { timeout: 20000 })
          await p1.evaluate(({ w, h, theme, effects }) => {
            const r = document.documentElement
            r.style.setProperty('--win-w', w + 'px'); r.style.setProperty('--win-h', h + 'px')
            r.setAttribute('data-theme', theme); r.setAttribute('data-effects', effects)
          }, { w: combo.win.w, h: combo.win.h, theme: combo.theme, effects: combo.effects })
          await p1.waitForTimeout(950)
          await p1.evaluate(({ env, st, route, section, tab, js }) => {
            window.setEnv?.(env); window.setState?.(st); window.setRoute?.(route || 'overview')
            if (section) window.showSettingsSection?.(section)
            if (tab) window.activateDiagTab?.(tab)
            if (js) { try { (0, eval)(js) } catch (e) { console.warn('protoJs', e) } }
          }, { env: state.proto.env, st: state.proto.state, route: state.proto.route, section: state.proto.section, tab: state.proto.tab, js: state.proto.js })
          await p1.waitForTimeout(450)
          // 原型 tray 路由把应用壳换成独立托盘预览舞台（`html[data-route="tray"] .desktop .stage{display:none}`）。
          const protoRootSel = state.proto.route === 'tray' ? '.tray-stage' : '.window'
          await captureEl(p1, protoRootSel, path.join(dir, 'prototype.png'))
          const protoGeom = await p1.evaluate((rootSel) => {
            const root = document.querySelector(rootSel).getBoundingClientRect()
            const r = sel => { const el = document.querySelector(sel); if (!el) return null; const b = el.getBoundingClientRect(); return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) } }
            return {
              route: document.documentElement.getAttribute('data-route'),
              mode: document.getElementById('route-overview')?.getAttribute('data-mode') ?? null,
              mainline: (document.querySelector('.motion-mainline')?.textContent ?? '').trim(),
              caption: (document.querySelector('.motion-caption')?.textContent ?? '').trim(),
              dialog: !!document.querySelector('#modal.motion-runtime-modal'),
              overlay: (() => { const m = document.querySelector('.modal-mask'); return !!m && getComputedStyle(m).display !== 'none' })(),
              main: r('.main'), topbar: r('.topbar'), section: r('.route-section.active'),
            }
          }, protoRootSel)

          p2 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h }, deviceScaleFactor: 2 })
          await p2.route('**', r => { const u = r.request().url(); (u.startsWith(APP) || u.startsWith('data:') || u.startsWith('blob:')) ? r.continue() : r.abort() })
          const q = new URLSearchParams({ __audit: '1', route: 'overview', theme: combo.theme, effects: combo.effects, ...state.impl })
          await p2.goto(`${APP}/?${q}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
          await p2.waitForSelector('.app-window', { timeout: 20000 })
          await p2.waitForTimeout(700)
          if (state.impl.js) {
            await p2.evaluate(js => { try { (0, eval)(js) } catch (e) { console.warn('implJs', e) } }, state.impl.js)
            await p2.waitForTimeout(400)
          }
          await captureEl(p2, '.app-window', path.join(dir, 'implementation.png'))
          const implGeom = await p2.evaluate(() => {
            const root = document.querySelector('.app-window').getBoundingClientRect()
            const r = sel => { const el = document.querySelector(sel); if (!el) return null; const b = el.getBoundingClientRect(); return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) } }
            return {
              route: document.documentElement.dataset.route ?? null,
              mode: document.querySelector('.ov')?.getAttribute('data-mode') ?? null,
              mainline: (document.querySelector('.motion-mainline')?.textContent ?? '').trim(),
              caption: (document.querySelector('.motion-caption')?.textContent ?? '').trim(),
              dialog: !!document.querySelector('.motion-runtime-modal'),
              overlay: !!document.querySelector('.modal-mask'),
              main: r('.main'), topbar: r('.topbar'), section: r('.route-section.active'),
            }
          })
          fs.writeFileSync(path.join(dir, 'geometry.json'), JSON.stringify({ state: state.name, name, window: combo.win, theme: combo.theme, effects: combo.effects, prototype: protoGeom, implementation: implGeom }, null, 2))
          summary.push({ state: state.name, name, route: [protoGeom.route, implGeom.route], mode: [protoGeom.mode, implGeom.mode], mainline: [protoGeom.mainline, implGeom.mainline], dialog: [protoGeom.dialog, implGeom.dialog], overlay: [protoGeom.overlay, implGeom.overlay] })
          console.log(`captured ${state.name}/${name} route=${protoGeom.route}/${implGeom.route} overlay=${protoGeom.overlay}/${implGeom.overlay} mainline=${JSON.stringify(protoGeom.mainline)}/${JSON.stringify(implGeom.mainline)}`)
        } catch (err) {
          summary.push({ state: state.name, name, errors: ['FAILED: ' + (err && err.message ? err.message : String(err))] })
          console.log(`FAILED ${state.name}/${name}: ${err && err.message ? err.message : err}`)
        } finally { if (p1) await p1.close().catch(() => {}); if (p2) await p2.close().catch(() => {}) }
      }
    }
  } finally { if (browser) await browser.close().catch(() => {}); stop() }
  fs.writeFileSync(path.join(outRoot, 'summary.json'), JSON.stringify(summary, null, 2))
  console.log(`\n状态配对 ${summary.length} 组；失败 ${summary.filter(s => s.errors).length}`)
}

void main()
