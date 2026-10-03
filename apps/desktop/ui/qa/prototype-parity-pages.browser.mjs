// 原型 ↔ 实现配对截图与几何对照：全部软件页面（IMP-05 §5.2）。
//
// 覆盖 6 个路由 × 2 窗口 × 2 主题 × 3 档 = 72 组合；每组合截取应用内容区域
// （原型 `.window` / 实现 `.app-window`），并记录 `.main` / `.topbar` / 当前路由容器几何。
// 输出到 `.adg/work/prototype-parity-20261001/screens-pages/<route>/<combo>/{prototype,implementation}.png`
// 与 `geometry.json`；像素 diff 由 `gen-parity.py <root>`（PIL）生成。
//
// 运行：node qa/prototype-parity-pages.browser.mjs
//   可选 CHROMIUM=<chrome-headless-shell 路径>、PAGES_CASE=<子串>（匹配 route 或 combo）
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import { spawn } from 'child_process'
import { createRequire } from 'module'
import { fileURLToPath } from 'url'
import http from 'http'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const repoRoot = path.resolve(uiDir, '..', '..', '..')
const outRoot = path.join(repoRoot, '.adg', 'work', 'prototype-parity-20261001', 'screens-pages')
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
if (!pwPath || !exe) {
  console.log('BLOCKED: 环境缺失（playwright-core / chromium headless shell）。')
  process.exit(2)
}
const require = createRequire(pwPath + '/')
const { chromium } = require('playwright-core')

const APP_PORT = Number(process.env.PARITY_APP_PORT || 5198)
const PROTO_PORT = Number(process.env.PARITY_PROTO_PORT || 5197)
const APP = `http://127.0.0.1:${APP_PORT}`
const PROTO = `http://127.0.0.1:${PROTO_PORT}`
const only = process.env.PAGES_CASE || ''
const want = (route, name) => !only || route.includes(only) || name.includes(only)

function waitPort(port, timeoutMs = 30000) {
  const start = Date.now()
  return new Promise((resolve, reject) => {
    const tick = () => {
      const sock = net.connect(port, '127.0.0.1')
      sock.once('connect', () => { sock.destroy(); resolve() })
      sock.once('error', () => {
        sock.destroy()
        if (Date.now() - start > timeoutMs) reject(new Error('端口未就绪 ' + port))
        else setTimeout(tick, 200)
      })
    }
    tick()
  })
}

// 与实现路由同名；`runtime`/`env` 给实现审计夹具一个可重复的运行态。
const ROUTES = ['overview', 'panel', 'extensions', 'logs', 'settings', 'tray']

const combos = []
for (const win of [{ w: 1180, h: 760 }, { w: 900, h: 600 }]) {
  for (const theme of ['light', 'dark']) {
    for (const effects of ['high', 'mid', 'low']) {
      combos.push({ win, theme, effects })
    }
  }
}

const summary = []

// 稳健抓取：先回到页顶，再按元素包围盒裁剪整页截图，避免 locator.screenshot 的
// scrollIntoView 在 `fixed`/超高元素（如原型 tray 路由）上失败。
async function captureEl(page, selector, outPath) {
  await page.evaluate(() => window.scrollTo(0, 0))
  const box = await page.evaluate(sel => {
    const el = document.querySelector(sel)
    if (!el) return null
    const b = el.getBoundingClientRect()
    if (b.width < 1 || b.height < 1) return null
    return { x: Math.round(b.x), y: Math.round(b.y), width: Math.round(b.width), height: Math.round(b.height) }
  }, selector)
  if (!box) throw new Error(`元素不可见或缺失：${selector}`)
  await page.screenshot({ path: outPath, clip: box })
}

async function main() {
  const viteBin = path.join(uiDir, 'node_modules', '.bin', 'vite')
  const app = spawn(viteBin, ['--host', '127.0.0.1', '--port', String(APP_PORT), '--strictPort'], {
    cwd: uiDir, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env },
  })
  // 进程内 Node 静态服务：并发安全、不会像单线程 `http.server` 那样被 iframe 连接占死。
  const MIME = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.png': 'image/png', '.json': 'application/json', '.svg': 'image/svg+xml' }
  const proto = http.createServer((req, res) => {
    try {
      const rel = decodeURIComponent(req.url.split('?')[0]).replace(/^\/+/, '')
      const file = path.resolve(repoRoot, rel)
      if (!file.startsWith(repoRoot) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
        res.writeHead(404); res.end('not found'); return
      }
      res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] || 'application/octet-stream' })
      fs.createReadStream(file).pipe(res)
    } catch { try { res.writeHead(500); res.end('err') } catch {} }
  })
  proto.listen(PROTO_PORT, '127.0.0.1')
  const stop = () => { try { app.kill('SIGTERM') } catch {}; try { proto.close() } catch {} }
  process.on('exit', stop)

  let browser
  try {
    await waitPort(APP_PORT)
    await waitPort(PROTO_PORT)
    browser = await chromium.launch({ executablePath: exe })

    for (const route of ROUTES) {
      for (const combo of combos) {
        const name = `${combo.win.w}x${combo.win.h}-${combo.theme}-${combo.effects}`
        if (!want(route, name)) continue
        let p1 = null
        let p2 = null
        try {
        const dir = path.join(outRoot, route, name)
        fs.mkdirSync(dir, { recursive: true })
        const errors = []

        // 原型（软件窗口区域）
        p1 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h + 60 }, deviceScaleFactor: 2 })
        p1.on('pageerror', e => errors.push('prototype: ' + e.message))
        // 原型是离线单体页；中止任何外部请求，避免内嵌资源把加载挂住。
        await p1.route('**', r => {
          const u = r.request().url()
          if (u.startsWith(PROTO) || u.startsWith('data:') || u.startsWith('blob:') || u === 'about:blank') r.continue()
          else r.abort()
        })
        await p1.goto(`${PROTO}/${encodeURI(PROTOTYPE_REL)}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
        await p1.waitForSelector('.window', { timeout: 20000 })
        await p1.evaluate(({ w, h, theme, effects, route }) => {
          const root = document.documentElement
          root.style.setProperty('--win-w', w + 'px')
          root.style.setProperty('--win-h', h + 'px')
          root.setAttribute('data-theme', theme)
          root.setAttribute('data-effects', effects)
        }, { w: combo.win.w, h: combo.win.h, theme: combo.theme, effects: combo.effects, route })
        await p1.waitForTimeout(950)
        await p1.evaluate(({ route }) => {
          window.setState?.('running')
          window.setRoute?.(route)
        }, { route })
        await p1.waitForTimeout(450)
        // 原型 tray 路由把应用壳换成独立的托盘预览舞台（`html[data-route="tray"] .desktop .stage{display:none}`），
        // 故该路由抓 `.tray-stage`，其余抓 `.window`。
        const protoRootSel = route === 'tray' ? '.tray-stage' : '.window'
        await captureEl(p1, protoRootSel, path.join(dir, 'prototype.png'))
        const protoGeom = await p1.evaluate((rootSel) => {
          const root = document.querySelector(rootSel).getBoundingClientRect()
          const r = sel => {
            const el = document.querySelector(sel)
            if (!el) return null
            const b = el.getBoundingClientRect()
            return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) }
          }
          return {
            route: document.documentElement.getAttribute('data-route'),
            main: r('.main'),
            topbar: r('.topbar'),
            section: r('.route-section.active'),
          }
        }, protoRootSel)
        // 实现（应用内容区域）
        p2 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h }, deviceScaleFactor: 2 })
        p2.on('pageerror', e => errors.push('implementation: ' + e.message))
        const q = new URLSearchParams({ __audit: '1', route, runtime: 'running', env: 'ready', theme: combo.theme, effects: combo.effects })
        await p2.route('**', r => {
          const u = r.request().url()
          if (u.startsWith(APP) || u.startsWith('data:') || u.startsWith('blob:')) r.continue()
          else r.abort()
        })
        await p2.goto(`${APP}/?${q}`, { waitUntil: 'domcontentloaded', timeout: 20000 })
        await p2.waitForSelector('.app-window', { timeout: 20000 })
        await p2.waitForTimeout(700)
        await captureEl(p2, '.app-window', path.join(dir, 'implementation.png'))
        const implGeom = await p2.evaluate(() => {
          const root = document.querySelector('.app-window').getBoundingClientRect()
          const r = sel => {
            const el = document.querySelector(sel)
            if (!el) return null
            const b = el.getBoundingClientRect()
            return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) }
          }
          return {
            route: document.documentElement.dataset.route ?? null,
            main: r('.main'),
            topbar: r('.topbar'),
            section: r('.route-section.active'),
          }
        })
        const geometry = { route, name, window: combo.win, theme: combo.theme, effects: combo.effects, prototype: protoGeom, implementation: implGeom, errors }
        fs.writeFileSync(path.join(dir, 'geometry.json'), JSON.stringify(geometry, null, 2))
        const d = k => {
          const a = protoGeom[k]; const b = implGeom[k]
          if (!a || !b) return { key: k, missing: true }
          return { key: k, dx: b.x - a.x, dy: b.y - a.y, dw: b.w - a.w, dh: b.h - a.h }
        }
        summary.push({ route, name, errors, deltas: ['main', 'topbar', 'section'].map(d) })
        console.log(`captured ${route}/${name}`)
        } catch (err) {
          // 单个组合失败不终止整轮；如实记录，继续其余组合。
          summary.push({ route, name, errors: ['FAILED: ' + (err && err.message ? err.message : String(err))], deltas: [] })
          console.log(`FAILED ${route}/${name}: ${err && err.message ? err.message : err}`)
        } finally {
          if (p1) await p1.close().catch(() => {})
          if (p2) await p2.close().catch(() => {})
        }
      }
    }
  } finally {
    if (browser) await browser.close().catch(() => {})
    stop()
  }

  fs.writeFileSync(path.join(outRoot, 'geometry-summary.json'), JSON.stringify(summary, null, 2))
  const bad = summary.filter(s => s.deltas.some(x => !x.missing && (Math.abs(x.dx) > 2 || Math.abs(x.dy) > 2 || Math.abs(x.dw) > 2 || Math.abs(x.dh) > 2)))
  console.log(`\n配对 ${summary.length} 组；几何超差(>2px) ${bad.length} 组`)
  for (const s of bad.slice(0, 40)) console.log(`  DELTA ${s.route}/${s.name} ${JSON.stringify(s.deltas)}`)
  const errs = summary.flatMap(s => s.errors)
  console.log(`页面错误 ${errs.length}`)
  if (errs.length) console.log(errs.slice(0, 20).join('\n'))
}

void main()
