// 全量路由审计器：抓原型与软件的结构指纹，输出结构化差异供人工判定。
//
// 设计原则（针对上一轮教训）：
// - 不做「存在性断言通过就算一致」；这里只**采集**事实，判定交给清单。
// - 每个路由两侧都抓同一组维度：标题 / 区块标题顺序 / 按钮文案顺序 / 表头 / 说明文字 / 关键尺寸。
// - 输出 JSON 落盘，作为审计清单的原始证据（可回溯、可复跑）。
//
// 用法：
//   node qa/audit-routes.mjs                 # 全部路由
//   node qa/audit-routes.mjs models sync     # 指定路由
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import { spawn } from 'child_process'
import { fileURLToPath } from 'url'
import { createRequire } from 'module'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-route-audit-'))
fs.mkdirSync(outDir, { recursive: true })
const PROTOTYPE = process.env.PROTOTYPE_FILE || path.resolve(uiDir, '..', '..', '..', '..', 'governance-main', 'docs', '04-项目资料', '01-项目支撑资料', '01-产品原型', '原型', 'index.html')
const PORT = Number(process.env.AUDIT_PORT || 5198)
const BASE = 'http://127.0.0.1:' + PORT

function playwrightCandidates() {
  const out = []
  const npx = path.join(os.homedir(), '.npm', '_npx')
  if (fs.existsSync(npx)) for (const d of fs.readdirSync(npx)) out.push(path.join(npx, d, 'node_modules', 'playwright-core'))
  return out.filter(c => fs.existsSync(path.join(c, 'package.json')))
}
function chromiumShell() {
  const base = path.join(os.homedir(), 'Library', 'Caches', 'ms-playwright')
  if (!fs.existsSync(base)) return null
  for (const d of fs.readdirSync(base).filter(x => x.startsWith('chromium_headless_shell-')).sort().reverse()) {
    const p = path.join(base, d, 'chrome-headless-shell-mac-arm64', 'chrome-headless-shell')
    if (fs.existsSync(p)) return p
  }
  return null
}
function waitPort(port, t = 40000) {
  return new Promise((res, rej) => {
    const s = Date.now()
    const tick = () => {
      const k = net.connect(port, '127.0.0.1')
      k.once('connect', () => { k.destroy(); res() })
      k.once('error', () => { k.destroy(); if (Date.now() - s > t) rej(new Error('vite 未就绪')); else setTimeout(tick, 200) })
    }
    tick()
  })
}

// 路由清单：原型 anchor / 软件 hash / 侧栏文案。含组件打样页。
const ROUTES = [
  { id: 'overview', nav: '概览', proto: 'overview', impl: 'overview' },
  { id: 'panel', nav: '面板', proto: 'panel', impl: 'panel' },
  { id: 'models', nav: '模型', proto: 'models', impl: 'models' },
  { id: 'extensions', nav: '拓展', proto: 'extensions', impl: 'extensions' },
  { id: 'sync', nav: '同步', proto: 'sync', impl: 'sync' },
  { id: 'logs', nav: '诊断', proto: 'logs', impl: 'logs' },
  { id: 'settings', nav: '设置', proto: 'settings', impl: 'settings' },
  { id: 'tray', nav: '托盘', proto: 'tray', impl: 'tray' },
  { id: 'conversations', nav: '对话配置', proto: 'conversations', impl: null },
  { id: 'components', nav: '组件与样式', proto: 'components', impl: null },
  { id: 'model-picker', nav: '模型选择器', proto: 'model-picker', impl: null },
]

/** 在页面内采集结构指纹。两侧用同一段代码，保证可比。 */
function fingerprint() {
  const visible = el => !!(el && el.getClientRects().length)
  const text = el => (el?.innerText || '').replace(/\s+/g, ' ').trim()
  const uniq = list => [...new Set(list.filter(Boolean))]

  const headings = [...document.querySelectorAll('h1,h2,h3')].filter(visible).map(h => ({ level: +h.tagName[1], text: text(h) })).filter(h => h.text)

  // 主内容区按钮（排除侧栏与隐藏元素），按 DOM 顺序。
  // 只取**静态文案**做结构比对：aria-label 往往含夹具数据（如「NewApi 启用状态」），
  // 两边夹具不同会产生大量假差异。数据驱动控件单独记在 ariaDriven，不参与文案比对。
  const nav = document.querySelector('nav, .sidebar, .rail')
  // 原型自带「原型工具栏 / 注释便签」（0.2.0 · … / 原型 NN ＋ / NN 页面导览 ↗ 等），
  // 它们是原型工具而非产品界面；不排除会在每个路由上制造同一条假差异。
  // 原型右侧工具板（.prototype-side）承载版本选择、评审条件、注释列表与演示场景，
  // 全部是原型自身工具，不属于产品界面。
  const toolbox = document.querySelector('.prototype-side, #prototypeTools, .proto-toolbar, .prototype-toolbar, #annotationLayer')
  const isPrototypeChrome = el =>
    !!el.closest('.proto-annotation, .annotation-marker, .prototype-toolbar, .proto-toolbar, #prototypeTools, .prototype-side, .annotation-source') ||
    /^原型\s*\d+/.test((el.innerText || '').trim()) ||
    /^\d+\s.*(页面导览|实现边界|行为约定)·/.test((el.innerText || '').trim())
  const all = [...document.querySelectorAll('button,[role=button]')]
    .filter(b => visible(b) && !nav?.contains(b) && !isPrototypeChrome(b) && !toolbox?.contains(b))
    .map(b => ({ text: text(b), label: b.getAttribute('aria-label') || '', disabled: b.disabled || b.getAttribute('aria-disabled') === 'true' }))
  // 归一化：数字徽标（通知计数 5 / 2）、下拉箭头 ▾、以及「已安装 N」这类计数文案
  // 会随夹具变化，不是结构差异。
  const norm = t => t
    .replace(/\s*▾\s*$/, '')
    .replace(/\s+/g, ' ')
    .trim()
  const isNoise = t => /^\d+$/.test(t) || /^(已安装|Claude|Codex|Gemini|OpenCode)\s*\d+$/.test(t)
  const buttons = all.filter(b => b.text && !isNoise(norm(b.text))).map(b => ({ ...b, text: norm(b.text) }))
  const ariaDriven = all.filter(b => !b.text && b.label).map(b => b.label)

  const tabs = [...document.querySelectorAll('[role=tab], .tab, .ui-tab')].filter(visible).map(t => text(t)).filter(Boolean)

  const tables = [...document.querySelectorAll('table')].filter(visible).map(t => ({
    headers: [...t.querySelectorAll('thead th')].map(h => text(h)),
    rowCount: t.querySelectorAll('tbody tr').length,
    firstRowCells: [...(t.querySelector('tbody tr')?.children || [])].map(c => text(c).slice(0, 40)),
  }))

  // 说明性段落（原型大量用 muted 段落承载规格）
  // 必须限定在当前激活路由内：原型右侧工具板 .prototype-side 也有大量 p / .mp-muted，
  // 不限定会把工具板说明算成产品正文（2026-10-25 独立复核发现，直接造成 4 条假差异）。
  const scope = document.querySelector('.route-section.active') || document.querySelector('main') || document.body
  const notes = uniq([...scope.querySelectorAll('p,.model-muted,.mp-muted')].filter(visible).map(p => text(p)).filter(t => t && t.length > 8).map(t => t.slice(0, 120)))

  // 关键尺寸：主内容首屏标题与第一个卡片
  const main = document.querySelector('main, .mp-page, .sx-page, .cv-page') || document.body
  const block = main.querySelector('section, article, .card')
  const rect = block ? block.getBoundingClientRect() : null

  return {
    headings,
    tabs,
    buttons: buttons.map(b => b.text),
    disabledButtons: buttons.filter(b => b.disabled).map(b => b.text),
    ariaDrivenCount: ariaDriven.length,
    tables,
    notes,
    sectionCount: main.querySelectorAll('section').length,
    firstBlock: rect ? { w: Math.round(rect.width), h: Math.round(rect.height) } : null,
    bodyTextLen: (document.body.innerText || '').length,
  }
}

const which = process.argv.slice(2).filter(a => !a.startsWith('-'))
const targets = which.length ? ROUTES.filter(r => which.includes(r.id)) : ROUTES
if (!targets.length) { console.log('无匹配路由：' + which.join(',')); process.exit(1) }
if (!fs.existsSync(PROTOTYPE)) { console.log('BLOCKED: 找不到原型'); process.exit(2) }

const pw = playwrightCandidates()
const exe = chromiumShell()
if (!pw.length || !exe) { console.log('BLOCKED: 缺少 playwright-core 或 chromium'); process.exit(2) }
const { chromium } = createRequire(pw[0] + '/')('playwright-core')

const server = spawn(path.resolve(uiDir, 'node_modules', '.bin', 'vite'),
  ['--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
  { cwd: uiDir, stdio: ['ignore', 'pipe', 'pipe'] })
process.on('exit', () => { try { server.kill('SIGTERM') } catch {} })

const report = { generatedAt: new Date().toISOString(), prototype: PROTOTYPE, routes: [] }
let browser
try {
  await waitPort(PORT)
  browser = await chromium.launch({ executablePath: exe })

  for (const route of targets) {
    const entry = { id: route.id, nav: route.nav, protoImpl: route.impl ? 'both' : 'prototype-only' }

    // 原型
    const p = await browser.newPage({ viewport: { width: 1180, height: 900 }, deviceScaleFactor: 2 })
    const pErr = []
    p.on('pageerror', e => pErr.push(String(e.message).slice(0, 120)))
    await p.goto('file://' + encodeURI(PROTOTYPE) + '#' + route.proto)
    await p.waitForTimeout(1500)
    entry.prototype = await p.evaluate(fingerprint)
    entry.prototypeErrors = pErr
    await p.screenshot({ path: path.join(outDir, route.id + '-prototype.png'), fullPage: false })
    await p.close()

    // 软件
    if (route.impl) {
      const i = await browser.newPage({ viewport: { width: 1180, height: 900 }, deviceScaleFactor: 2 })
      const iErr = []
      i.on('pageerror', e => iErr.push(String(e.message).slice(0, 120)))
      await i.goto(BASE + '/?__audit=1&report=1&route=' + route.impl + '&env=ready&runtime=running&effects=low&theme=light')
      await i.waitForSelector('#app .app-window', { timeout: 20000 }).catch(() => {})
      await i.waitForTimeout(1500)
      entry.implementation = await i.evaluate(fingerprint)
      entry.implementationErrors = iErr
      await i.screenshot({ path: path.join(outDir, route.id + '-implementation.png'), fullPage: false })
      await i.close()
    }

    report.routes.push(entry)
    console.log('collected ' + route.id)
  }
} finally {
  if (browser) await browser.close().catch(() => {})
  try { server.kill('SIGTERM') } catch {}
}

const out = path.join(outDir, 'route-audit.json')
fs.writeFileSync(out, JSON.stringify(report, null, 2) + '\n')
console.log('')
console.log('routes=' + report.routes.length + ' 证据目录=' + outDir)
console.log('JSON=' + out)

