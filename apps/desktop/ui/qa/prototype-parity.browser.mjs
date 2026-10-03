// 原型 ↔ 实现配对截图与几何对照（IMP-05 §5.1 / §5.2）。
//
// - 同逻辑窗口、同主题、同档位，分别截取**应用内容区域**（原型 `.window` / 实现 `.app-window`）；
//   原型外部评审工具条（proto-controls）不计入。
// - 输出到 `.adg/work/prototype-parity-20261001/screens/<combo>/{prototype,implementation}.png`
//   与 `geometry.json`；像素 diff 由 `gen-parity.py`（PIL）生成 `diff.png` / `diff.json`。
// - 状态固定为「运行中 + 环境就绪」，避免原型评审壳的随机切态影响配对。
//
// 运行：node qa/prototype-parity.browser.mjs
//   可选 CHROMIUM=<chrome-headless-shell 路径>、PARITY_CASE=<子串>
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import { spawn } from 'child_process'
import { createRequire } from 'module'
import { fileURLToPath } from 'url'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const repoRoot = path.resolve(uiDir, '..', '..', '..')
const outRoot = path.join(repoRoot, '.adg', 'work', 'prototype-parity-20261001', 'screens')
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
const only = process.env.PARITY_CASE || ''
const want = n => !only || n.includes(only)

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

const combos = []
for (const win of [{ w: 1180, h: 760 }, { w: 900, h: 600 }]) {
  for (const theme of ['light', 'dark']) {
    for (const effects of ['high', 'mid', 'low']) {
      combos.push({ win, theme, effects })
    }
  }
}

const summary = []

async function main() {
  const viteBin = path.join(uiDir, 'node_modules', '.bin', 'vite')
  const app = spawn(viteBin, ['--host', '127.0.0.1', '--port', String(APP_PORT), '--strictPort'], {
    cwd: uiDir, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env },
  })
  const proto = spawn('python3', ['-m', 'http.server', String(PROTO_PORT), '--bind', '127.0.0.1'], {
    cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'],
  })
  const stop = () => { for (const p of [app, proto]) { try { p.kill('SIGTERM') } catch {} } }
  process.on('exit', stop)

  let browser
  try {
    await waitPort(APP_PORT)
    await waitPort(PROTO_PORT)
    browser = await chromium.launch({ executablePath: exe })

    for (const combo of combos) {
      const name = `${combo.win.w}x${combo.win.h}-${combo.theme}-${combo.effects}`
      if (!want(name)) continue
      const dir = path.join(outRoot, name)
      fs.mkdirSync(dir, { recursive: true })
      const errors = []

      // 原型（软件窗口区域）
      const p1 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h + 60 }, deviceScaleFactor: 2 })
      p1.on('pageerror', e => errors.push('prototype: ' + e.message))
      await p1.goto(`${PROTO}/${encodeURI(PROTOTYPE_REL)}`)
      await p1.waitForSelector('.window', { timeout: 20000 })
      // 原型首帧有基于 hash 的锚点复位与 650ms 的启动落定（loading→stopped）；
      // 先把窗口/主题/档位定好，等落定后再切到「运行中」，避免被启动时序覆盖。
      await p1.evaluate(({ w, h, theme, effects }) => {
        const root = document.documentElement
        root.style.setProperty('--win-w', w + 'px')
        root.style.setProperty('--win-h', h + 'px')
        root.setAttribute('data-theme', theme)
        root.setAttribute('data-effects', effects)
      }, { w: combo.win.w, h: combo.win.h, theme: combo.theme, effects: combo.effects })
      await p1.waitForTimeout(950)
      await p1.evaluate(() => { window.setState?.('running') })
      await p1.waitForTimeout(300)
      await p1.locator('.window').screenshot({ path: path.join(dir, 'prototype.png') })
      const protoGeom = await p1.evaluate(() => {
        const root = document.querySelector('.window').getBoundingClientRect()
        const r = sel => {
          const el = document.querySelector(sel)
          if (!el) return null
          const b = el.getBoundingClientRect()
          // 坐标一律相对**被截取的应用区域**（原型 `.window`），消除外部评审工具条的偏移。
          return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) }
        }
        return {
          size: document.documentElement.getAttribute('data-overview-size'),
          mode: document.getElementById('route-overview')?.getAttribute('data-mode') ?? '',
          status: r('#card-overview-status'),
          stage: r('#card-overview-status .ovb-summary'),
          actions: r('#card-overview-status .ovb-actions'),
          mods: r('#route-overview #mods'),
          main: r('.main'),
          topbar: r('.topbar'),
          pageTitle: r('.page-title'),
          topbarActions: r('.topbar-actions'),
          actionLabels: Array.from(document.querySelectorAll('#card-overview-status .ovb-actions > *'))
            .filter(el => !el.hidden && el.offsetParent !== null)
            .map(el => (el.textContent ?? '').trim().replace(/\s+/g, ' ')),
          mainline: (document.querySelector('.motion-mainline')?.textContent ?? '').trim(),
          caption: (document.querySelector('.motion-caption')?.textContent ?? '').trim(),
        }
      })
      await p1.close()

      // 实现（应用内容区域）
      const p2 = await browser.newPage({ viewport: { width: combo.win.w, height: combo.win.h }, deviceScaleFactor: 2 })
      p2.on('pageerror', e => errors.push('implementation: ' + e.message))
      const q = new URLSearchParams({ __audit: '1', report: '1', route: 'overview', runtime: 'running', env: 'ready', theme: combo.theme, effects: combo.effects })
      await p2.goto(`${APP}/?${q}`)
      await p2.waitForSelector('.app-window', { timeout: 20000 })
      await p2.waitForTimeout(700)
      await p2.locator('.app-window').screenshot({ path: path.join(dir, 'implementation.png') })
      const implGeom = await p2.evaluate(() => {
        const root = document.querySelector('.app-window').getBoundingClientRect()
        const r = sel => {
          const el = document.querySelector(sel)
          if (!el) return null
          const b = el.getBoundingClientRect()
          return { x: Math.round(b.x - root.x), y: Math.round(b.y - root.y), w: Math.round(b.width), h: Math.round(b.height) }
        }
        return {
          size: document.documentElement.getAttribute('data-overview-size'),
          mode: document.querySelector('.ov')?.getAttribute('data-mode') ?? '',
          status: r('[data-testid="overview-motion"]'),
          stage: r('.motion-fixed'),
          actions: r('.motion-actions'),
          mods: r('.mods'),
          main: r('.main'),
          topbar: r('.topbar'),
          actionLabels: Array.from(document.querySelectorAll('.motion-actions > *'))
            .filter(el => !el.hidden && el.offsetParent !== null)
            .map(el => (el.textContent ?? '').trim().replace(/\s+/g, ' ')),
          mainline: (document.querySelector('.motion-mainline')?.textContent ?? '').trim(),
          caption: (document.querySelector('.motion-caption')?.textContent ?? '').trim(),
        }
      })
      await p2.close()

      const geometry = { name, window: combo.win, theme: combo.theme, effects: combo.effects, prototype: protoGeom, implementation: implGeom, errors }
      fs.writeFileSync(path.join(dir, 'geometry.json'), JSON.stringify(geometry, null, 2))
      const d = k => {
        const a = protoGeom[k]; const b = implGeom[k]
        if (!a || !b) return { key: k, missing: true }
        return { key: k, dx: b.x - a.x, dy: b.y - a.y, dw: b.w - a.w, dh: b.h - a.h }
      }
      summary.push({
        name, errors,
        mode: [protoGeom.mode, implGeom.mode],
        size: [protoGeom.size, implGeom.size],
        deltas: ['status', 'stage', 'actions'].map(d),
      })
      console.log(`captured ${name}`)
    }
  } finally {
    if (browser) await browser.close().catch(() => {})
    stop()
  }

  fs.writeFileSync(path.join(outRoot, 'geometry-summary.json'), JSON.stringify(summary, null, 2))
  const bad = summary.filter(s => s.deltas.some(x => !x.missing && (Math.abs(x.dx) > 2 || Math.abs(x.dy) > 2 || Math.abs(x.dw) > 2 || Math.abs(x.dh) > 2)))
  console.log(`\n配对 ${summary.length} 组；几何超差(>2px) ${bad.length} 组`)
  for (const s of bad) console.log(`  DELTA ${s.name} ${JSON.stringify(s.deltas)}`)
  const errs = summary.flatMap(s => s.errors)
  console.log(`页面错误 ${errs.length}`)
  if (errs.length) console.log(errs.join('\n'))
}

void main()
