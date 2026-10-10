// 应用级浏览器功能/视觉探针（§17.5 层级③）。
//
// 用 Vite 开发服务器 + 无头 Chromium 打开带 `?__audit=1` 的审计装置，覆盖所有路由、
// 关键状态（主题/缩放/浮层/任务卡/通知场景），断言结构与不变量并截图。
// 审计装置仅在 DEV 提供（`src/dev/auditHarness.ts`），因此本探针跑开发服务器，不代表正式制品。
//
// 运行：node apps/desktop/ui/qa/audit.browser.mjs
//   - 可选 PLAYWRIGHT_CORE=<playwright-core 目录>；CHROMIUM=<chrome-headless-shell 路径>
//   - 环境缺失以退出码 2 报 BLOCKED，不把「没跑」当通过。
import { createRequire } from 'module'
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import { spawn } from 'child_process'
import { fileURLToPath } from 'url'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
// Evidence belongs outside the code branch; callers may select governance storage.
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-browser-audit-'))
fs.mkdirSync(outDir, { recursive: true })

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
  const dirs = fs.readdirSync(base).filter(d => d.startsWith('chromium_headless_shell-')).sort()
  for (const d of dirs.reverse()) {
    const p = path.join(base, d, 'chrome-headless-shell-mac-arm64', 'chrome-headless-shell')
    if (fs.existsSync(p)) return p
  }
  return null
}
const pwPath = resolvePlaywright()
const exe = process.env.CHROMIUM || newestChromiumShell()
if (!pwPath || !exe) {
  console.log('BLOCKED: 环境缺失，未执行浏览器检查。')
  console.log('  playwright-core: ' + (pwPath || '未找到（可设 PLAYWRIGHT_CORE）'))
  console.log('  chromium headless shell: ' + (exe || '未找到（可设 CHROMIUM）'))
  process.exit(2)
}
const require = createRequire(pwPath + '/')
const { chromium } = require('playwright-core')

const PORT = Number(process.env.AUDIT_PORT || 5199)
const BASE = `http://127.0.0.1:${PORT}`

function waitPort(port, timeoutMs = 30000) {
  const start = Date.now()
  return new Promise((resolve, reject) => {
    const tick = () => {
      const sock = net.connect(port, '127.0.0.1')
      sock.once('connect', () => { sock.destroy(); resolve() })
      sock.once('error', () => {
        sock.destroy()
        if (Date.now() - start > timeoutMs) reject(new Error('vite 未在超时内就绪'))
        else setTimeout(tick, 200)
      })
    }
    tick()
  })
}

const results = []
const check = (name, ok, detail = '') => results.push({ name, ok: !!ok, detail })
const warn = []
// 性能观测（§19.12）：只记录数值，不参与通过/失败判定。
const notes = []
// 可选：AUDIT_CASE=<子串> 只跑匹配的用例（调试用，正式验收不带此变量）。
const onlyCase = process.env.AUDIT_CASE || ''
const want = name => !onlyCase || name.includes(onlyCase)

async function main() {
  const viteBin = path.join(uiDir, 'node_modules', '.bin', 'vite')
  const server = spawn(viteBin, ['--host', '127.0.0.1', '--port', String(PORT), '--strictPort'], {
    cwd: uiDir,
    stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env },
  })
  let serverLog = ''
  server.stdout.on('data', d => { serverLog += d })
  server.stderr.on('data', d => { serverLog += d })
  const stopServer = () => { try { server.kill('SIGTERM') } catch { /* ignore */ } }
  process.on('exit', stopServer)

  let browser
  try {
    await waitPort(PORT)
    browser = await chromium.launch({ executablePath: exe })

    const pages = [
      { name: '概览', route: 'overview', sel: '[data-testid="overview-motion"]' },
      { name: '面板', route: 'panel', sel: '.panel-shell' },
      { name: '扩展·Skills', route: 'extensions', tab: 'skills', sel: '.ext-tabs' },
      { name: '扩展·MCP', route: 'extensions', tab: 'mcp', sel: '.ext-tabs' },
      { name: '日志·诊断', route: 'logs', tab: 'doctor', sel: '.diag-tabs' },
      { name: '日志·历史', route: 'logs', tab: 'logs', sel: '.diag-tabs' },
      { name: '日志·通知', route: 'logs', tab: 'notifications', sel: '.diag-tabs' },
      { name: '托盘', route: 'tray', sel: '.tray-stage' },
    ]
    const settingsSections = ['general', 'installation', 'backup', 'extensions', 'migration', 'sync', 'cleanup', 'cli', 'upgrade', 'about']
    for (const s of settingsSections) pages.push({ name: `设置·${s}`, route: 'settings', section: s, sel: '.settings-tabs' })

    for (const p of pages) if (want(p.name)) await auditPage(browser, p)

    // 状态/浮层/场景探针（默认概览页承载全局壳层）。
    const states = [
      ['通知中心', { panel: '1' }, r => r.notificationCenter.storeFlagOpen === true && r.notificationCenter.items >= 1],
      ['Toast', { toast: '1' }, r => (r.surfaces.toastText || '').length > 0],
      ['任务卡', { task: '1' }, r => (r.surfaces.taskTitle || '').length > 0],
      ['通知·空', { panel: '1', scenario: 'empty' }, r => (r.surfaces.panelEmpty || '').length > 0],
      ['通知·长文本', { panel: '1', scenario: 'longtext' }, r => r.overflow.docHorizontalOverflow <= 1],
      ['任务·超时', { task: '1', scenario: 'task-timeout' }, r => /超时|等待时间/.test(r.surfaces.taskMessage || '') || (r.surfaces.taskSteps || []).some(s => /timeout/.test(s))],
      ['任务·失败', { task: '1', scenario: 'task-failure' }, r => (r.surfaces.taskMessage || '').length > 0],
    ]
    for (const [n, params, fn] of states) if (want(n)) await auditState(browser, n, params, fn)

    // 主题与缩放：多路由扫一遍，检查不横向溢出且缩放因子生效。
    for (const theme of ['light', 'dark']) {
      for (const route of ['overview', 'settings', 'extensions', 'logs', 'tray', 'panel']) {
        if (want(`${theme}·${route}`)) await auditThemeScale(browser, `${theme}·${route}`, { theme, route })
      }
    }
    // 概览定稿 A：宽窄窗口 × 深浅主题 + 弹窗可达。
    await auditOverviewMotion(browser, want)
    // 缩放：成功分支（本地偏好适配）应用并回写；失败分支回退到已保存值。
    const scales = [
      ['缩放·50 成功', '50', { prefs: 'local', expectZoom: 0.5, expectNormalized: '50' }],
      ['缩放·100 成功', '100', { prefs: 'local', expectZoom: 1, expectNormalized: '100' }],
      ['缩放·200 成功', '200', { prefs: 'local', expectZoom: 2, expectNormalized: '200' }],
      ['缩放·越界 999→200', '999', { prefs: 'local', expectZoom: 2, expectNormalized: '200' }],
      ['缩放·空值→100', '', { prefs: 'local', expectZoom: 1, expectNormalized: '100' }],
      ['缩放·保存失败回退 50→100', '50', { expectZoom: 1, expectNormalized: '100' }],
    ]
    for (const [n, s, o] of scales) if (want(n)) await auditScale(browser, n, s, o)

    // 三档特效：软件画质独立定义，系统减少动态不改变高档。
    await auditEffects(browser, want)

    // 高性能预算（IMP-05 §7）：中/低档不得持续 rAF；页面隐藏必须暂停装饰动画。
    await auditOverviewActions(browser, want)
    await auditAnimation(browser, want)
    // 启动与资源预算（IMP-05 §7）：启动采样 + 重复挂载后不增长。
    await auditLifecycle(browser, want)

    // 真实交互（§17.2）：点击驱动状态迁移，而非只断言结构存在。
    await auditInteractions(browser, want)

    // 性能测量（IMP-04 §19.12）：长列表渲染与路由切换耗时（③ 层；数值记录在 warnings）。
    await auditCpuMemory(browser, want)
    await auditInteractionDistribution(browser, want)
    await auditPerf(browser, want)
  } finally {
    if (browser) await browser.close().catch(() => {})
    stopServer()
  }

  const failed = results.filter(r => !r.ok)
  const report = {
    generatedAt: new Date().toISOString(),
    base: BASE,
    total: results.length,
    passed: results.length - failed.length,
    failed: failed.length,
    warnings: warn,
    performance: notes,
    results,
  }
  fs.writeFileSync(path.join(outDir, 'audit-report.json'), JSON.stringify(report, null, 2))
  console.log(`\n浏览器探针：${report.passed}/${report.total} 通过，失败 ${report.failed}`)
  for (const f of failed) console.log(`  FAIL ${f.name}${f.detail ? ' — ' + f.detail : ''}`)
  console.log(`新页面错误 ${warn.length ? '' : '0'}`)
  for (const n of notes) console.log(`  性能 ${n}`)
  console.log(`报告：${path.join(outDir, 'audit-report.json')}`)
  if (failed.length) process.exitCode = 1
}

function queryFor(p) {
  const q = new URLSearchParams()
  q.set('__audit', '1')
  q.set('report', '1')
  if (p.route) q.set('route', p.route)
  if (p.tab) q.set('tab', p.tab)
  if (p.section) q.set('section', p.section)
  if (p.theme) q.set('theme', p.theme)
  // 空字符串也是有效输入（越界/空值回写用例），因此用 !== undefined 判定。
  if (p.scale !== undefined) q.set('scale', p.scale)
  if (p.panel) q.set('panel', p.panel)
  if (p.toast) q.set('toast', p.toast)
  if (p.task) q.set('task', p.task)
  if (p.scenario) q.set('scenario', p.scenario)
  if (p.prefs) q.set('prefs', p.prefs)
  if (p.runtime) q.set('runtime', p.runtime)
  if (p.stale) q.set('stale', p.stale)
  if (p.effects) q.set('effects', p.effects)
  if (p.env) q.set('env', p.env)
  return q.toString()
}

async function openPage(browser, p) {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 })
  const errors = []
  page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
  page.on('pageerror', e => errors.push('pageerror: ' + e.message))
  await page.goto('about:blank')
  await page.goto(`${BASE}/?${queryFor(p)}`)
  // 报告已由审计装置在应用挂载后采集；这里再等壳出现，保证断言面对真实 DOM。
  await page.waitForSelector('#app .app-window', { timeout: 15000 }).catch(() => {})
  await page.waitForFunction(() => document.documentElement.dataset.auditReport === 'ready', null, { timeout: 15000 })
    .catch(() => {})
  return { page, errors }
}

async function readReport(page) {
  const raw = await page.evaluate(() => document.getElementById('audit-report')?.textContent ?? 'null')
  try { return JSON.parse(raw) } catch { return null }
}

async function shot(page, name) {
  const safe = name.replace(/[^\w\u4e00-\u9fa5·-]/g, '_')
  await page.screenshot({ path: path.join(outDir, safe + '.png'), fullPage: false }).catch(() => {})
}

async function auditPage(browser, p) {
  const { page, errors } = await openPage(browser, p)
  try {
    const report = await readReport(page)
    check(`${p.name} 审计报告就绪`, report !== null, report === null ? '#audit-report 未生成' : '')
    const hasSel = await page.$(p.sel)
    check(`${p.name} 关键结构存在（${p.sel}）`, hasSel !== null)
    // IMP-06：顶栏全局「刷新状态」入口。托盘页走 `TrayRoute`、面板页（IMP-07）不渲染 AppTopbar，均不参与。
    if (p.name !== '托盘' && p.name !== '面板') {
      const refreshBtn = await page.$('.topbar-refresh')
      check(`${p.name} 顶栏刷新入口存在`, refreshBtn !== null)
      if (refreshBtn) {
        check(`${p.name} 顶栏刷新入口为「刷新状态」`, (await refreshBtn.getAttribute('title')) === '刷新状态')
      }
    }
    if (p.name === '面板') {
      check('面板页不保留软件顶栏（IMP-07）', (await page.$('.topbar')) === null)
    }
    if (report) {
      check(`${p.name} 无横向溢出`, report.overflow.docHorizontalOverflow <= 1, `docOverflow=${report.overflow.docHorizontalOverflow}`)
      check(`${p.name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
    }
    await shot(page, `page-${p.name}`)
  } finally {
    await page.close()
  }
}

async function auditState(browser, name, params, assertFn) {
  const { page, errors } = await openPage(browser, params)
  try {
    const report = await readReport(page)
    check(`${name} 审计报告就绪`, report !== null)
    if (report) {
      check(`${name} 状态断言`, assertFn(report))
      check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
    }
    await shot(page, `state-${name}`)
  } finally {
    await page.close()
  }
}

async function auditThemeScale(browser, name, p) {
  const { page, errors } = await openPage(browser, p)
  try {
    const report = await readReport(page)
    if (report) {
      check(`${name} 主题生效`, report.theme === p.theme, `theme=${report.theme}`)
      check(`${name} 无横向溢出`, report.overflow.docHorizontalOverflow <= 1, `docOverflow=${report.overflow.docHorizontalOverflow}`)
      check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
    } else {
      check(`${name} 审计报告就绪`, false)
    }
    await shot(page, `theme-${name}`)
  } finally {
    await page.close()
  }
}

/**
 * 三档特效验证（UI规范 §26.2 / IMP-05 §3）：
 * - `data-effects` 由唯一外观策略写入；高/中档保留玻璃，低档统一实底、无 backdrop-filter、无光场；
 * - 系统「减少动态」把高档有效表现降为中档，但不改用户保存值（属性即有效表现）。
 */
async function auditEffects(browser, want) {
  const tiers = [
    { effects: 'high', expect: 'high', glass: true, ambient: 'block' },
    { effects: 'mid', expect: 'mid', glass: true, ambient: 'block' },
    { effects: 'low', expect: 'low', glass: false, ambient: 'none' },
  ]
  for (const t of tiers) {
    const name = `三档·${t.effects}`
    if (!want(name)) continue
    const { page, errors } = await openPage(browser, { route: 'overview', panel: '1', effects: t.effects })
    try {
      const report = await readReport(page)
      check(`${name} 审计报告就绪`, report !== null)
      if (report) {
        check(`${name} 属性=${t.expect}`, report.effects === t.expect, `data-effects=${report.effects}`)
        const blur = String(report.effectsSurface.panelBlur || '')
        check(`${name} 通知面玻璃=${t.glass ? '有模糊' : '无模糊'}`, t.glass ? blur.includes('blur') : blur === 'none', `panelBlur=${blur}`)
        check(`${name} 环境光=${t.ambient}`, report.effectsSurface.effectsSurfaceAmbient === t.ambient, `ambient=${report.effectsSurface.effectsSurfaceAmbient}`)
        const cardBlur = String(report.effectsSurface.cardBlur || '')
        check(`${name} 卡片面玻璃=${t.glass ? '有模糊' : '无模糊'}`, t.glass ? cardBlur.includes('blur') : cardBlur === 'none', `cardBlur=${cardBlur}`)
        check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
      }
      await shot(page, `effects-${t.effects}`)
    } finally {
      await page.close()
    }
  }

  // 逐页低档一致退实底：设置 / 扩展 / 诊断 / 面板壳 的主要表面都不得再带 backdrop-filter。
  const solidPages = [
    { name: '设置·通用', p: { route: 'settings', section: 'general', effects: 'low' }, sel: '.card' },
    { name: '扩展·Skills', p: { route: 'extensions', tab: 'skills', effects: 'low' }, sel: '.card' },
    { name: '诊断·环境', p: { route: 'logs', tab: 'doctor', effects: 'low' }, sel: '.card' },
    { name: '面板壳', p: { route: 'panel', effects: 'low' }, sel: '.panel-shell' },
  ]
  for (const item of solidPages) {
    const name = `三档·低档实底·${item.name}`
    if (!want(name)) continue
    const { page, errors } = await openPage(browser, item.p)
    try {
      await page.waitForSelector(item.sel, { timeout: 15000 }).catch(() => {})
      await page.waitForTimeout(200)
      const probe = await page.evaluate(sel => {
        const el = document.querySelector(sel)
        if (!el) return { found: false }
        const cs = getComputedStyle(el)
        return {
          found: true,
          blur: cs.backdropFilter || cs.webkitBackdropFilter || 'none',
          attr: document.documentElement.getAttribute('data-effects'),
        }
      }, item.sel)
      check(`${name} 属性=low`, probe.attr === 'low', `data-effects=${probe.attr}`)
      check(`${name} ${item.sel} 无 backdrop-filter`, probe.found && (probe.blur === 'none' || probe.blur === ''), `found=${probe.found} blur=${probe.blur}`)
      check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
      await shot(page, `effects-low-${item.name}`)
    } finally {
      await page.close()
    }
  }

  // 软件画质独立于系统减少动态。
  const reducedName = '三档·减少动态保持高档'
  if (want(reducedName)) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 }, deviceScaleFactor: 2, reducedMotion: 'reduce' })
    const errors = []
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
    page.on('pageerror', e => errors.push('pageerror: ' + e.message))
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', effects: 'high' })}`)
      await page.waitForSelector('#app .app-window', { timeout: 15000 }).catch(() => {})
      await page.waitForFunction(() => document.documentElement.dataset.auditReport === 'ready', null, { timeout: 15000 }).catch(() => {})
      await page.waitForTimeout(150)
      const report = await readReport(page)
      check(`${reducedName} 审计报告就绪`, report !== null)
      if (report) {
        check(`${reducedName} 高档在减少动态下保持 high`, report.effects === 'high', `data-effects=${report.effects}`)
        check(`${reducedName} 无未捕获错误`, errors.length === 0, errors.join(' | '))
      }
      await shot(page, 'effects-reduced')
    } finally {
      await page.close()
    }
  }
}

/**
 * 启动与资源预算（IMP-05 §7，③ 层开发服务器）：
 * - 冷启动到首帧（`.app-window` 出现）与到业务就绪（审计报告就绪）各 5 次，取中位数；
 * - 切页 / 开关通知面板各 20 次后，listener / observer / interval / timeout 计数不增长。
 * 数值记录在 warnings，阈值只用于抓「明显泄漏」。
 */
async function auditLifecycle(browser, want) {
  if (want('性能·启动采样')) {
    const firstFrames = []
    const readyFrames = []
    for (let i = 0; i < 5; i += 1) {
      const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
      await page.addInitScript(() => { window.__t0 = performance.now() })
      try {
        const t0 = Date.now()
        await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready' })}`)
        await page.waitForSelector('#app .app-window', { timeout: 20000 })
        firstFrames.push(Date.now() - t0)
        await page.waitForFunction(() => document.documentElement.dataset.auditReport === 'ready', null, { timeout: 20000 }).catch(() => {})
        readyFrames.push(Date.now() - t0)
      } finally {
        await page.close()
      }
    }
    const median = list => { const s = [...list].sort((a, b) => a - b); return s[Math.floor(s.length / 2)] }
    const mFirst = median(firstFrames)
    const mReady = median(readyFrames)
    check('性能·启动采样 有 5 个样本', firstFrames.length === 5, `n=${firstFrames.length}`)
    check('性能·启动采样 无挂起样本（< 20s）', firstFrames.every(v => v < 20000), firstFrames.join(','))
    notes.push(`性能·启动 首帧中位数=${mFirst}ms 业务就绪中位数=${mReady}ms（5 次，③ 层开发服务器）`)
  }

  if (want('性能·资源不增长')) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
    await page.addInitScript(() => {
      // 同时统计 Tauri IPC 调用次数（切档不得新增状态 IPC 或轮询）。
      window.__res = window.__res || { add: 0, remove: 0, interval: 0, liveObserver: 0, ipc: 0 }
      const internals = window.__TAURI_INTERNALS__
      if (internals && typeof internals.invoke === 'function') {
        const invoke = internals.invoke.bind(internals)
        internals.invoke = (...args) => { window.__res.ipc += 1; return invoke(...args) }
      }
      // 只统计**长寿目标**（window/document/MediaQueryList）与**活的** observer / 定时器：
      // Vue 3 卸载时不逐个 removeEventListener 元素处理器（随节点一起 GC），
      // 统计它们会把「正常行为」误报为泄漏，因此不纳入。
      const proto = EventTarget.prototype
      const add = proto.addEventListener
      const rm = proto.removeEventListener
      const longLived = self =>
        self === window || self === document || (typeof MediaQueryList !== 'undefined' && self instanceof MediaQueryList)
      proto.addEventListener = function (...args) { if (longLived(this)) window.__res.add += 1; return add.apply(this, args) }
      proto.removeEventListener = function (...args) { if (longLived(this)) window.__res.remove += 1; return rm.apply(this, args) }
      const si = window.setInterval
      window.setInterval = (...args) => { window.__res.interval += 1; return si.apply(this, args) }
      for (const key of ['ResizeObserver', 'IntersectionObserver', 'MutationObserver']) {
        if (window[key]) {
          const O = window[key]
          window[key] = class extends O {
            constructor(...a) { super(...a); window.__res.liveObserver += 1 }
            disconnect(...a) { window.__res.liveObserver -= 1; return super.disconnect(...a) }
          }
        }
      }
    })
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready' })}`)
      await page.waitForSelector('#app .app-window', { timeout: 20000 })
      await page.waitForTimeout(600)
      const before = await page.evaluate(() => ({ ...window.__res }))
      // 切档 20 次：不得新增状态 IPC 或轮询（§7）。
      const ipcBefore = await page.evaluate(() => window.__res.ipc)
      for (let i = 0; i < 20; i += 1) {
        await page.evaluate(tier => document.documentElement.setAttribute('data-effects', tier), ['high', 'mid', 'low'][i % 3])
        await page.waitForTimeout(40)
      }
      const ipcDelta = (await page.evaluate(() => window.__res.ipc)) - ipcBefore
      check('性能·资源不增长 切档不新增 IPC', ipcDelta === 0, `Δipc=${ipcDelta}`)

      // 切页 20 次（概览 ↔ 设置 ↔ 扩展 ↔ 诊断）。
      const routes = ['overview', 'settings', 'extensions', 'logs']
      for (let i = 0; i < 20; i += 1) {
        const r = routes[i % routes.length]
        await page.evaluate(target => { location.hash = '#' + target }, r)
        await page.waitForTimeout(60)
      }
      // 开关通知面板 20 次（用同一按钮开/关；Escape 不负责关闭该浮层，避免把「探针未关闭」误判为泄漏）。
      for (let i = 0; i < 20; i += 1) {
        await page.locator('.notification-btn').first().click()
        await page.waitForTimeout(40)
        await page.locator('.notification-btn').first().click()
        await page.waitForTimeout(40)
      }
      await page.waitForTimeout(400)
      const after = await page.evaluate(() => ({ ...window.__res }))
      const delta = { add: after.add - before.add, remove: after.remove - before.remove, interval: after.interval - before.interval, observer: after.liveObserver - before.liveObserver }
      // §7：长寿目标上的 listener、定时器与活 observer 都不得随重复操作增长。
      check('性能·资源不增长 interval 不增长', delta.interval <= 0, `Δinterval=${delta.interval}`)
      check('性能·资源不增长 活 observer 不增长', delta.observer <= 0, `Δobserver=${delta.observer}`)
      check('性能·资源不增长 长寿 listener 不增长', delta.add - delta.remove <= 2, `Δadd=${delta.add} Δremove=${delta.remove}`)
      notes.push(`性能·资源 40 次切换后 长寿 listener Δadd=${delta.add} Δremove=${delta.remove} Δinterval=${delta.interval} Δ活observer=${delta.observer}`)
    } finally {
      await page.close()
    }
  }
}

/**
 * 概览动作按钮材质（IMP-07）：对齐原型 `BTN_MATERIAL_DEFAULT='glass'` 的玻璃观感——
 * 半透明分层填充 + 非透明边框 + 外阴影；0.1.10 统一取消按钮内高光。
 * 刻意不做实时 `backdrop-filter`（保证三档动画的 p95 帧间隔预算），故断言 blur=none。
 */
async function auditOverviewActions(browser, want) {
  const name = '概览动作按钮材质'
  if (!want(name)) return
  const { page, errors } = await openPage(browser, { route: 'overview', runtime: 'running', env: 'ready' })
  try {
    await page.waitForSelector('.motion-actions > .btn', { timeout: 15000 }).catch(() => {})
    const mat = await page.evaluate(() => {
      const buttons = [...document.querySelectorAll('.motion-actions > .btn')]
      if (!buttons.length) return null
      const primary = document.querySelector('.motion-actions > .btn.primary')
      const read = el => {
        const s = getComputedStyle(el)
        return {
          cls: el.className,
          shadow: s.boxShadow,
          border: s.borderTopColor,
          blur: s.backdropFilter || s.webkitBackdropFilter || 'none',
          background: s.backgroundImage,
        }
      }
      return { all: buttons.map(read), primary: primary ? read(primary) : null, count: buttons.length }
    })
    check(`${name} 存在`, !!mat && mat.count > 0, mat ? JSON.stringify(mat.all.map(x => x.cls)) : '')
    // 所有按钮统一无内高光，仍保留填充、边框和主操作层级。
    check(
      `${name} 所有按钮统一取消内高光`,
      !!mat && mat.all.every(x => !/inset/.test(x.shadow)),
      mat ? JSON.stringify(mat.all.map(x => [x.cls, x.shadow])) : '',
    )
    check(`${name} 边框非透明`, !!mat && mat.all.every(x => x.border !== 'rgba(0, 0, 0, 0)'), mat ? JSON.stringify(mat.all.map(x => [x.cls, x.border])) : '')
    check(`${name} 不做实时模糊（性能预算）`, !!mat && mat.all.every(x => x.blur === 'none'), mat ? JSON.stringify(mat.all.map(x => [x.cls, x.blur])) : '')
    check(`${name} 主按钮为强调玻璃`, !!mat && !!mat.primary && /linear-gradient/.test(mat.primary.background), mat && mat.primary ? mat.primary.background : '')
    check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
    await shot(page, 'overview-actions')
  } finally {
    await page.close()
  }
}

/**
 * 三档动画预算（IMP-05 §7）：
 * - 高档可见时连续动画（统计 rAF 回调）；
 * - 中/低档不产生持续动画 rAF（静态目标，状态改变才重绘）；
 * - 页面隐藏（visibilitychange=hidden）后必须暂停装饰动画。
 */
async function auditAnimation(browser, want) {
  const tiers = [
    { tier: 'high', continuous: true },
    { tier: 'mid', continuous: false },
    { tier: 'low', continuous: false },
  ]
  for (const item of tiers) {
    const name = `性能·档位动画-${item.tier}`
    if (!want(name)) continue
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
    await page.addInitScript(() => {
      window.__rafCount = 0
      window.__rafTimes = []
      const original = window.requestAnimationFrame.bind(window)
      window.requestAnimationFrame = cb => {
        window.__rafCount += 1
        window.__rafTimes.push(performance.now())
        return original(cb)
      }
    })
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready', effects: item.tier })}`)
      await page.waitForSelector('[data-testid="overview-motion"]', { timeout: 15000 }).catch(() => {})
      await page.waitForTimeout(400)
      await page.evaluate(() => { window.__rafCount = 0; window.__rafTimes = [] })
      // §7：高档稳定可见采样 30s；中/低档只需证明无持续动画，取 1.2s 样本即可。
      await page.waitForTimeout(item.continuous ? 30000 : 1200)
      const frames = await page.evaluate(() => window.__rafCount)
      if (item.continuous) check(`${name} 高档持续动画`, frames >= 30, `frames=${frames}`)
      else check(`${name} 无持续动画 rAF`, frames <= 3, `frames=${frames}`)
      // 帧间隔分布：以屏幕刷新间隔 B（无头环境取 16.7ms）为预算，p95 ≤ 2B、无 > 6B 卡顿。
      const timing = await page.evaluate(() => {
        const t = window.__rafTimes || []
        const gaps = []
        for (let i = 1; i < t.length; i += 1) gaps.push(t[i] - t[i - 1])
        gaps.sort((a, b) => a - b)
        const at = q => (gaps.length ? gaps[Math.min(gaps.length - 1, Math.floor(q * gaps.length))] : 0)
        return { count: gaps.length, p50: at(0.5), p95: at(0.95), max: gaps.length ? gaps[gaps.length - 1] : 0 }
      })
      if (item.continuous) {
        check(`${name} 帧间隔 p95 ≤ 2B(33.4ms)`, timing.p95 <= 33.4, `p95=${timing.p95.toFixed(1)}ms`)
        check(`${name} 无 > 6B(100ms) 卡顿`, timing.max <= 100, `max=${timing.max.toFixed(1)}ms`)
      }
      notes.push(`性能·档位动画 ${item.tier} rAF=${frames} p50=${timing.p50.toFixed(1)}ms p95=${timing.p95.toFixed(1)}ms max=${timing.max.toFixed(1)}ms（${item.continuous ? '30s' : '1.2s'}，③ 层）`)

      // 隐藏后暂停（仅对高档有实际动画的情形断言）。
      await page.evaluate(() => {
        Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'hidden' })
        document.dispatchEvent(new Event('visibilitychange'))
      })
      await page.waitForTimeout(150)
      await page.evaluate(() => { window.__rafCount = 0; window.__rafTimes = [] })
      await page.waitForTimeout(900)
      const hiddenFrames = await page.evaluate(() => window.__rafCount)
      check(`${name} 隐藏后暂停`, hiddenFrames <= 3, `frames=${hiddenFrames}`)
    } finally {
      await page.close()
    }
  }
}

/**
 * 概览双形态（UI规范 §25.1）验证：1180×760 走标准就绪（300px，168/66/66，Logo 140），
 * 900×600 走紧凑就绪（222px，128/50/44，Logo 100）；不再有常驻环境摘要行；就绪态保留底部三卡与最近事件。
 * 另核对环境准备态（门禁卡 + 三卡/事件收起）与运行详情弹窗可达。
 */
async function auditOverviewMotion(browser, want) {
  const viewports = [
    { w: 1180, h: 760, label: '默认窗口', fixed: 300, stage: 168, mark: 660, hero: 140 },
    { w: 900, h: 600, label: '最小窗口', fixed: 222, stage: 128, mark: 640, hero: 100 },
  ]
  for (const vp of viewports) {
    for (const theme of ['light', 'dark']) {
      const name = `概览形象·${vp.label}-${vp.w}x${vp.h}-${theme}`
      if (!want(name)) continue
      const page = await browser.newPage({ viewport: { width: vp.w, height: vp.h }, deviceScaleFactor: 2 })
      const errors = []
      page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
      page.on('pageerror', e => errors.push('pageerror: ' + e.message))
      try {
        await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready', theme })}`)
        await page.waitForSelector('[data-testid="overview-motion"]', { timeout: 15000 }).catch(() => {})
        await page.waitForTimeout(250)
        const geom = await page.evaluate(() => {
          const px = sel => {
            const el = document.querySelector(sel)
            return el instanceof HTMLElement || el instanceof SVGElement ? Math.round(el.getBoundingClientRect().height) : -1
          }
          const w = sel => {
            const el = document.querySelector(sel)
            return el ? Math.round(el.getBoundingClientRect().width) : -1
          }
          const ambient = document.querySelector('.motion-ambient')
          const energy = document.querySelector('.motion-ambient .energy')
          const ambientStyle = ambient ? getComputedStyle(ambient) : null
          const energyStyle = energy ? getComputedStyle(energy) : null
          const bottom = sel => {
            const el = document.querySelector(sel)
            return el ? Math.round(el.getBoundingClientRect().bottom) : -1
          }
          // 背景光二选一：WEBGL 网格着色器层 / 纯 CSS 极光；几何断言针对**当前生效的那一层**。
          const renderMode = document.documentElement.getAttribute('data-glow-render') === 'mesh' ? 'mesh' : 'css'
          return {
            mode: document.querySelector('.ov')?.getAttribute('data-mode') ?? '',
            fixedH: px('.motion-fixed'),
            stageH: px('.motion-stage'),
            markH: px('.motion-mark'),
            heroW: w('.motion-hero svg'),
            fixedW: w('.motion-fixed'),
            mainW: w('.main'),
            mainEdges: document.querySelector('.main')?.getBoundingClientRect().toJSON(),
            backdropEdges: document.querySelector('.motion-backdrop')?.getBoundingClientRect().toJSON(),
            renderMode,
            glowW: renderMode === 'mesh' ? w('.motion-mesh-host') : w('.motion-ambient'),
            meshCanvas: document.querySelectorAll('.motion-mesh-host canvas.motion-mesh').length,
            glowBlur: ambientStyle ? (ambientStyle.filter || 'none') : 'none',
            glowLayerBlur: [...document.querySelectorAll('.motion-ambient .cloud, .motion-ambient .energy, .motion-ambient .light-floor')]
              .filter(el => /blur\(/.test(getComputedStyle(el).filter || '')).length,
            glowMask: ambientStyle ? (ambientStyle.maskImage || ambientStyle.webkitMaskImage || 'none') : 'none',
            energyMask: energyStyle ? (energyStyle.maskImage || energyStyle.webkitMaskImage || 'none') : 'none',
            markBottom: bottom('.motion-mark'),
            modsBottom: bottom('.mods'),
            envRow: document.querySelectorAll('.motion-env-row').length,
            mods: document.querySelectorAll('.mods .mod').length,
            events: document.querySelectorAll('[data-testid="overview-recent-events"]').length,
            gate: document.querySelectorAll('[data-testid="environment-gate"]').length,
            docOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
          }
        })
        check(`${name} 就绪形态`, geom.mode === 'ready', `mode=${geom.mode}`)
        check(`${name} 固定舞台 ${vp.fixed}px`, geom.fixedH === vp.fixed, `fixedH=${geom.fixedH}`)
        check(`${name} 动画舞台 ${vp.stage}px`, geom.stageH === vp.stage, `stageH=${geom.stageH}`)
        check(`${name} 光场绘制层 ${vp.mark}px`, geom.markH === vp.mark, `markH=${geom.markH}`)
        check(`${name} Logo 展示 ${vp.hero}px`, geom.heroW === vp.hero, `heroW=${geom.heroW}`)
        // 光场范围回归（IMP-10）：容器必须铺满正文列，且柔化做在容器一层上。
        check(`${name} 光场铺满主内容全边界`, geom.glowW === geom.mainW && geom.backdropEdges?.left === geom.mainEdges?.left && geom.backdropEdges?.right === geom.mainEdges?.right && geom.backdropEdges?.top === geom.mainEdges?.top, `glowW=${geom.glowW} fixedW=${geom.fixedW}`)
        // IMP-11：光场不靠 blur 柔化（低频渐变本身够平滑，模糊只吃帧预算且成像差 ≤0.5/255）。
        check(`${name} 光场不做逐层模糊（保帧预算）`, !/blur\(/.test(geom.glowBlur) && geom.glowLayerBlur === 0, `ambient=${geom.glowBlur} layers=${geom.glowLayerBlur}`)
        // 纵向渐隐：CSS 极光用 mask-template；WEBGL 网格着色器在片元里做长缓坡渐隐。
        check(
          `${name} 光场纵向渐隐`,
          geom.renderMode === 'mesh' ? geom.meshCanvas === 1 : /gradient/.test(geom.glowMask),
          `mode=${geom.renderMode} canvas=${geom.meshCanvas} mask=${geom.glowMask}`,
        )
        check(`${name} 能量束径向遮罩（无硬边圆环）`, /radial-gradient/.test(geom.energyMask), `mask=${geom.energyMask}`)
        // 光场要越过状态区底部、盖到下方卡片后面，玻璃卡才「透光」（IMP-11）。
        check(`${name} 光场盖过下方三卡`, geom.modsBottom > 0 && geom.markBottom >= geom.modsBottom, `markBottom=${geom.markBottom} modsBottom=${geom.modsBottom}`)
        check(`${name} 无常驻环境摘要行`, geom.envRow === 0, `envRow=${geom.envRow}`)
        check(`${name} 无准备态门禁卡`, geom.gate === 0, `gate=${geom.gate}`)
        check(`${name} 底部原三卡`, geom.mods === 3, `mods=${geom.mods}`)
        check(`${name} 最近事件卡`, geom.events === 1, `events=${geom.events}`)
        check(`${name} 无横向溢出`, geom.docOverflow <= 1, `docOverflow=${geom.docOverflow}`)
        check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
        await shot(page, `overview-motion-${vp.w}x${vp.h}-${theme}`)
      } finally {
        await page.close()
      }
    }
  }

  // 环境准备态：三卡与最近事件收起，展示门禁卡；顶部缩短到 174px。
  if (want('概览形象·环境准备')) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 }, deviceScaleFactor: 2 })
    const errors = []
    page.on('pageerror', e => errors.push('pageerror: ' + e.message))
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'stopped', env: 'missing_ocx' })}`)
      await page.waitForSelector('[data-testid="environment-gate"]', { timeout: 15000 }).catch(() => {})
      await page.waitForTimeout(200)
      const state = await page.evaluate(() => ({
        mode: document.querySelector('.ov')?.getAttribute('data-mode') ?? '',
        fixedH: Math.round(document.querySelector('.motion-fixed')?.getBoundingClientRect().height ?? -1),
        mods: document.querySelectorAll('.mods .mod').length,
        events: document.querySelectorAll('[data-testid="overview-recent-events"]').length,
        checks: document.querySelectorAll('.overview-checks li').length,
        install: document.querySelectorAll('[data-testid="env-install-actions"] button').length,
      }))
      check('概览形象·环境准备 准备形态', state.mode === 'setup', `mode=${state.mode}`)
      check('概览形象·环境准备 顶部缩短 174px', state.fixedH === 174, `fixedH=${state.fixedH}`)
      check('概览形象·环境准备 三卡收起', state.mods === 0, `mods=${state.mods}`)
      check('概览形象·环境准备 最近事件收起', state.events === 0, `events=${state.events}`)
      check('概览形象·环境准备 检查三项', state.checks === 3, `checks=${state.checks}`)
      // 门禁卡动作：安装 OpenCodex / 导入离线包 / 重新检查。
      check('概览形象·环境准备 安装与离线导入出口', state.install === 3, `install=${state.install}`)
      check('概览形象·环境准备 无未捕获错误', errors.length === 0, errors.join(' | '))
      await shot(page, 'overview-motion-环境准备')
    } finally {
      await page.close()
    }
  }

  // 运行详情弹窗可达（就绪态）：上部两列进程与环境、整宽来源目录，长值有复制出口。
  if (want('概览形象·弹窗')) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 }, deviceScaleFactor: 2 })
    const errors = []
    page.on('pageerror', e => errors.push('pageerror: ' + e.message))
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready' })}`)
      await page.waitForSelector('[data-testid="overview-motion"]', { timeout: 15000 }).catch(() => {})
      await page.locator('.motion-mainline').click()
      await page.waitForTimeout(200)
      const detail = await page.locator('.motion-modal').innerText().catch(() => '')
      check('概览形象·弹窗 运行详情可达', /端口/.test(detail) && /数据目录/.test(detail), detail.slice(0, 60))
      const cols = await page.evaluate(() => ({
        env: document.querySelectorAll('.motion-runtime-environment').length,
        checks: document.querySelectorAll('.motion-runtime-modal .motion-check-list li').length,
        copies: document.querySelectorAll('.motion-runtime-paths .motion-value-copy').length,
      }))
      check('概览形象·弹窗 上部两列（环境列）', cols.env === 1, `env=${cols.env}`)
      check('概览形象·弹窗 环境三项', cols.checks === 3, `checks=${cols.checks}`)
      check('概览形象·弹窗 长值复制出口', cols.copies === 2, `copies=${cols.copies}`)
      await shot(page, 'overview-motion-运行详情弹窗')
      await page.locator('.motion-modal-actions .btn.primary').click()
      await page.waitForTimeout(150)
      check('概览形象·弹窗 运行详情可关闭', (await page.locator('.motion-modal').count()) === 0)
      check('概览形象·弹窗 无未捕获错误', errors.length === 0, errors.join(' | '))
    } finally {
      await page.close()
    }
  }

  // 主线标签逐态核验（对齐已确认原型 STATE_MODEL.main）。
  if (want('概览形象·主线')) {
    const cases = [
      ['pending', '运行中'],
      ['running', '运行中'],
      // at_risk = 代理未在运行 + startup at-risk：主线与托盘同一口径，说「未运行」。
      ['at_risk', '未运行'],
      ['external_takeover', '运行中'],
      ['unreachable', '运行中'],
      ['stopped', '未运行'],
      ['starting', '未运行'],
      ['starting_failed', '未运行'],
      ['not_found', '待接入'],
      ['loading', '正在确认'],
    ]
    for (const [rt, label] of cases) {
      const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
      try {
        await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: rt, env: 'ready' })}`)
        await page.waitForSelector('.motion-mainline', { timeout: 15000 }).catch(() => {})
        await page.waitForTimeout(150)
        const text = (await page.locator('.motion-mainline').innerText()).trim().replace(/\s+/g, '')
        check(`概览形象·主线 ${rt} → ${label}`, text === label, text)
      } finally {
        await page.close()
      }
    }
  }

  // 概览动作逐态核验（对齐已确认原型 `actionsByState`；2026-10-03 用户确认）：
  // at_risk 与「未运行」逐字一致（只「启动 OpenCodex」）；starting_failed 的启动读作「重试启动」。
  if (want('概览形象·动作')) {
    const cases = [
      { rt: 'stopped', buttons: ['启动 OpenCodex'] },
      { rt: 'starting_failed', buttons: ['重试启动', '查看日志'] },
      { rt: 'at_risk', buttons: ['启动 OpenCodex'] },
      { rt: 'external_takeover', buttons: ['查看建议'] },
      { rt: 'unreachable', buttons: ['查看日志'] },
      { rt: 'running', buttons: ['打开面板', '停止', '重启', '查看日志'] },
    ]
    for (const { rt, buttons } of cases) {
      const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
      try {
        await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: rt, env: 'ready' })}`)
        await page.waitForSelector('.motion-actions', { timeout: 15000 }).catch(() => {})
        await page.waitForTimeout(150)
        const visible = await page.evaluate(() => {
          const vis = (el) => el && getComputedStyle(el).display !== 'none'
          return Array.from(document.querySelectorAll('.motion-actions > .btn'))
            .filter(vis)
            .map((el) => el.textContent.trim())
        })
        check(`概览形象·动作 ${rt} → ${buttons.join('/')}`, JSON.stringify(visible) === JSON.stringify(buttons), visible.join('/'))
      } finally {
        await page.close()
      }
    }
  }

  // 事实新鲜度接线：最近一次采集失败 + 此前真实观测 → 过期的「正在确认」。
  if (want('概览形象·新鲜度')) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 }, deviceScaleFactor: 2 })
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready', stale: '1' })}`)
      await page.waitForSelector('.motion-mainline', { timeout: 15000 }).catch(() => {})
      await page.waitForTimeout(250)
      const mainline = (await page.locator('.motion-mainline').innerText()).trim().replace(/\s+/g, '')
      const caption = (await page.locator('.motion-caption').innerText()).trim()
      check('概览形象·新鲜度 主线归「正在确认」', mainline === '正在确认', mainline)
      check('概览形象·新鲜度 说明标注过期', /过期/.test(caption), caption)
      await shot(page, 'overview-motion-过期')
    } finally {
      await page.close()
    }
  }

  // 环境检查按顺序（UI规范 §25.3）：检查中只亮当前项、其后「待检查」；缺失态不出现含糊的「未检查」。
  if (want('概览形象·环境顺序')) {
    const first = await browser.newPage({ viewport: { width: 1180, height: 760 } })
    try {
      await first.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'stopped', env: 'checking' })}`)
      await first.waitForSelector('[data-testid="environment-gate"]', { timeout: 15000 }).catch(() => {})
      await first.waitForTimeout(200)
      const checking = await first.locator('.overview-check-value').evaluateAll(
        els => els.map(el => (el.textContent ?? '').trim()),
      )
      check('概览形象·环境顺序 检查中只亮首项', checking[0] === '检查中' && checking[1] === '待检查' && checking[2] === '待检查', JSON.stringify(checking))
      await shot(first, 'overview-motion-环境检查中')
    } finally {
      await first.close()
    }
    const second = await browser.newPage({ viewport: { width: 1180, height: 760 } })
    try {
      await second.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'stopped', env: 'missing_npm' })}`)
      await second.waitForSelector('[data-testid="environment-gate"]', { timeout: 15000 }).catch(() => {})
      await second.waitForTimeout(200)
      const values = await second.locator('.overview-check-value').evaluateAll(
        els => els.map(el => (el.textContent ?? '').trim()),
      )
      check('概览形象·环境顺序 三项且无「未检查」', values.length === 3 && values.every(v => v !== '未检查'), JSON.stringify(values))
      check('概览形象·环境顺序 通过/未通过/待检查', values[0] === '已发现' && values[1] === '未发现' && values[2] === '待检查', JSON.stringify(values))
      // 门禁卡与运行详情同源：准备态主入口不打开详情，环境明细在门禁卡中直接呈现。
      const gateText = await second.locator('[data-testid="environment-gate"]').innerText()
      check('概览形象·环境顺序 门禁卡含三项结论', ['Node.js', 'npm', 'OpenCodex'].every(n => gateText.includes(n)), gateText.slice(0, 40))
      await shot(second, 'overview-motion-环境顺序')
    } finally {
      await second.close()
    }
  }
}

/**
 * 性能测量（§19.12）：数值记入 warnings 以便报告可检索；断言用宽松上限，
 * 避免开发服务器抖动把门禁变成噪声（真机制品的路由切换已另行实测）。
 */
/**
 * 交互耗时分布（IMP-05 §7）：路由切换与通知面板渲染各 ≥20 次，记录 median/p95。
 */
/**
 * CPU / 内存 / 帧成本（IMP-05 §7）：三档同条件比较。
 * 用 CDP `Performance.getMetrics` 取脚本/任务耗时与 JS 堆；中低档应消除连续动画的 CPU 成本。
 */
async function auditCpuMemory(browser, want) {
  const name = '性能·三档CPU内存'
  if (!want(name)) return
  const results = []
  for (const tier of ['high', 'mid', 'low']) {
    const page = await browser.newPage({ viewport: { width: 1180, height: 760 } })
    try {
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready', effects: tier })}`)
      await page.waitForSelector('[data-testid="overview-motion"]', { timeout: 20000 }).catch(() => {})
      const client = await page.context().newCDPSession(page)
      await client.send('Performance.enable')
      const read = async () => {
        const { metrics } = await client.send('Performance.getMetrics')
        const map = {}
        for (const m of metrics) map[m.name] = m.value
        return map
      }
      await page.waitForTimeout(500)
      const a = await read()
      await page.waitForTimeout(3000)
      const b = await read()
      results.push({
        tier,
        taskMs: Math.round((b.TaskDuration - a.TaskDuration) * 1000),
        scriptMs: Math.round((b.ScriptDuration - a.ScriptDuration) * 1000),
        heapMB: Math.round(b.JSHeapUsedSize / 1048576),
        layoutMs: Math.round((b.LayoutDuration - a.LayoutDuration) * 1000),
      })
    } finally {
      await page.close()
    }
  }
  for (const r of results) {
    notes.push(`性能·三档CPU内存 ${r.tier} 3s任务耗时=${r.taskMs}ms 脚本=${r.scriptMs}ms 布局=${r.layoutMs}ms 堆=${r.heapMB}MB（③ 层）`)
  }
  const high = results.find(r => r.tier === 'high')
  const idle = results.filter(r => r.tier !== 'high')
  check(`${name} 三档均有采样`, results.length === 3, `n=${results.length}`)
  if (high) {
    check(`${name} 中/低档脚本成本低于高档`, idle.every(r => r.scriptMs < high.scriptMs), `high=${high.scriptMs}ms others=${idle.map(r => r.scriptMs).join('/')}ms`)
    check(`${name} 中/低档任务耗时低于高档`, idle.every(r => r.taskMs < high.taskMs), `high=${high.taskMs}ms others=${idle.map(r => r.taskMs).join('/')}ms`)
    check(`${name} 堆占用不随重复加载单调增长（单页 < 200MB）`, results.every(r => r.heapMB < 200), results.map(r => `${r.tier}:${r.heapMB}MB`).join(' '))
  }
}

async function auditInteractionDistribution(browser, want) {
  const name = '性能·交互分布'
  if (!want(name)) return
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 })
  try {
    await page.goto(`${BASE}/?${queryFor({ route: 'overview', runtime: 'running', env: 'ready', scenario: 'manynotif' })}`)
    await page.waitForFunction(() => document.documentElement.dataset.auditReport === 'ready', null, { timeout: 20000 }).catch(() => {})
    const pct = (list, q) => { const s = [...list].sort((a, b) => a - b); return s[Math.min(s.length - 1, Math.floor(q * s.length))] }
    const routes = []
    for (let i = 0; i < 20; i += 1) {
      const target = i % 2 === 0 ? 'settings' : 'overview'
      const t0 = Date.now()
      await page.evaluate(r => { location.hash = '#' + r }, target)
      await page.waitForTimeout(0)
      await page.waitForFunction(r => document.documentElement.dataset.route === r, target, { timeout: 5000 }).catch(() => {})
      routes.push(Date.now() - t0)
    }
    const panels = []
    for (let i = 0; i < 20; i += 1) {
      const t0 = Date.now()
      await page.locator('.notification-btn').first().click()
      await page.waitForFunction(() => document.querySelectorAll('.notification-list .notification-item').length >= 200, null, { timeout: 8000 }).catch(() => {})
      panels.push(Date.now() - t0)
      await page.locator('.notification-btn').first().click()
      await page.waitForTimeout(30)
    }
    const rMed = pct(routes, 0.5); const rP95 = pct(routes, 0.95)
    const pMed = pct(panels, 0.5); const pP95 = pct(panels, 0.95)
    check(`${name} 路由 20 样本`, routes.length === 20, `n=${routes.length}`)
    check(`${name} 通知面板 20 样本`, panels.length === 20, `n=${panels.length}`)
    check(`${name} 路由 p95 有界(<1500ms)`, rP95 < 1500, `p95=${rP95}ms`)
    check(`${name} 通知面板 p95 有界(<3000ms)`, pP95 < 3000, `p95=${pP95}ms`)
    notes.push(`性能·交互分布 路由 median=${rMed}ms p95=${rP95}ms；通知面板(200 条) median=${pMed}ms p95=${pP95}ms（各 20 次，③ 层）`)
  } finally {
    await page.close()
  }
}

async function auditPerf(browser, want) {
  if (want('性能·通知长列表渲染')) {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 })
    try {
      // 只测「长列表渲染」本身：先让带 200 条夹具的应用就绪，再点开通知面板计时到列表渲染完成。
      await page.goto(`${BASE}/?${queryFor({ route: 'overview', scenario: 'manynotif' })}`)
      await page.waitForFunction(() => document.documentElement.dataset.auditReport === 'ready', null, { timeout: 20000 }).catch(() => {})
      const t0 = Date.now()
      await page.locator('.notification-btn').first().click()
      await page.waitForFunction(
        () => document.querySelectorAll('.notification-list .notification-item').length >= 200,
        null,
        { timeout: 8000 },
      ).catch(() => {})
      const renderMs = Date.now() - t0
      const rendered = await page.evaluate(() => document.querySelectorAll('.notification-list .notification-item').length)
      check('性能·通知长列表渲染 渲染 200 条', rendered >= 200, `rendered=${rendered}`)
      check('性能·通知长列表渲染 渲染耗时上限', renderMs < 3000, `renderMs=${renderMs}`)
      notes.push(`性能·通知长列表渲染 rendered=${rendered} renderMs=${renderMs}（③ 层，开发服务器）`)
      await shot(page, 'perf-通知长列表')
    } finally {
      await page.close()
    }
  }

  if (want('性能·路由切换')) {
    const { page, errors } = await openPage(browser, { route: 'overview' })
    try {
      const t0 = Date.now()
      await page.locator('.side .nav button', { hasText: '诊断' }).first().click()
      await page.waitForSelector('.diag-tabs', { timeout: 5000 }).catch(() => {})
      const switchMs = Date.now() - t0
      check('性能·路由切换 到达目标页', (await page.$('.diag-tabs')) !== null)
      check('性能·路由切换 耗时上限', switchMs < 1500, `switchMs=${switchMs}`)
      check('性能·路由切换 无未捕获错误', errors.length === 0, errors.join(' | '))
      notes.push(`性能·路由切换 switchMs=${switchMs}（③ 层，开发服务器）`)
    } finally {
      await page.close()
    }
  }

  if (want('性能·Markdown 渲染')) {
    // Markdown 渲染耗时（§19.12）：点开带长 Markdown 描述的 Skill 详情，计时到正文渲染完成。
    const { page, errors } = await openPage(browser, { route: 'extensions', tab: 'skills', scenario: 'markdown' })
    try {
      await page.waitForSelector('.skill-info[data-ext-detail]', { timeout: 8000 }).catch(() => {})
      const t0 = Date.now()
      await page.locator('.skill-info[data-ext-detail]').first().click()
      await page.waitForSelector('.ext-detail .ext-detail-md', { timeout: 8000 }).catch(() => {})
      await page.waitForFunction(
        () => {
          // MarkdownContent 的根元素同时带自身类与传入的 `.ext-detail-md`（Vue 类合并）。
          const el = document.querySelector('.ext-detail .ext-detail-md')
          return Boolean(el) && el.querySelectorAll('h1, h2, p, li').length > 50
        },
        null,
        { timeout: 8000 },
      ).catch(() => {})
      const renderMs = Date.now() - t0
      const nodes = await page.evaluate(
        () => {
          const el = document.querySelector('.ext-detail .ext-detail-md')
          return el ? el.querySelectorAll('h1, h2, p, li').length : 0
        },
      )
      check('性能·Markdown 渲染 正文已渲染', nodes > 50, `nodes=${nodes}`)
      check('性能·Markdown 渲染 耗时上限', renderMs < 3000, `renderMs=${renderMs}`)
      check('性能·Markdown 渲染 无未捕获错误', errors.length === 0, errors.join(' | '))
      notes.push(`性能·Markdown 渲染 nodes=${nodes} renderMs=${renderMs}（③ 层，开发服务器）`)
      await shot(page, 'perf-markdown')
    } finally {
      await page.close()
    }
  }
}

async function auditScale(browser, name, scale, opts = {}) {
  const { page } = await openPage(browser, { route: 'settings', section: 'general', scale, prefs: opts.prefs })
  try {
    const report = await readReport(page)
    if (report) {
      const sp = report.scaleProbe
      check(`${name} 缩放探针就绪`, sp !== null && sp !== undefined)
      if (sp) {
        check(`${name} 生效缩放 --ui-zoom=${opts.expectZoom}`, Number.parseFloat(sp.uiZoom) === opts.expectZoom, `uiZoom=${sp.uiZoom} raw=${JSON.stringify(sp.raw)} slider=${sp.sliderValue}`)
        check(`${name} 输入回写为 ${opts.expectNormalized}`, sp.normalizedValue === opts.expectNormalized, `normalized=${sp.normalizedValue} raw=${JSON.stringify(sp.raw)}`)
      }
    } else {
      check(`${name} 审计报告就绪`, false)
    }
    await shot(page, `scale-${name}`)
  } finally {
    await page.close()
  }
}

// §17.2「不是只断言 click handler 调用过」：点击后回读真实界面状态。
async function auditInteractions(browser, want) {
  const cases = []
  const add = (name, params, fn) => cases.push([name, params, fn])

  add('交互·设置开关', { route: 'settings', section: 'general', prefs: 'local' }, async page => {
    const toggle = page.locator('.setting-row .toggle[role="switch"]').first()
    const before = await toggle.getAttribute('aria-checked')
    await toggle.click()
    await page.waitForTimeout(200)
    const after = await toggle.getAttribute('aria-checked')
    check('交互·设置开关 状态翻转', before !== after, `${before}->${after}`)
    const toast = await page.locator('.notif-host .toast').innerText().catch(() => '')
    check('交互·设置开关 给保存反馈', /已(开启|关闭)/.test(toast), toast)
  })

  add('交互·设置选择器', { route: 'settings', section: 'general', prefs: 'local' }, async page => {
    const trigger = page.locator('.select-trigger[aria-label="面板打开方式"]')
    check('交互·选择器 初始收起', (await trigger.getAttribute('aria-expanded')) === 'false')
    await trigger.click()
    await page.waitForTimeout(150)
    check('交互·选择器 展开', (await trigger.getAttribute('aria-expanded')) === 'true')
    await page.locator('.select-menu[aria-label="面板打开方式"] .select-option', { hasText: '浏览器兜底' }).click()
    await page.waitForTimeout(200)
    const value = await page.locator('.select-trigger[aria-label="面板打开方式"] .select-value').innerText()
    check('交互·选择器 选择生效', value.trim() === '浏览器兜底', value)
    check('交互·选择器 选择后收起', (await trigger.getAttribute('aria-expanded')) === 'false')
    await trigger.click()
    const selected = await page.locator('.select-menu[aria-label="面板打开方式"] .select-option[aria-selected="true"]').innerText()
    check('交互·选择器 选中态回读', selected.trim() === '浏览器兜底', selected)
  })

  // 画质卡片（IMP-14）：与原型同构——独立卡片，含「界面特效」与「背景光渲染」两项。
  add('交互·画质卡片', { route: 'settings', section: 'general', prefs: 'local' }, async page => {
    const quality = page.locator('[data-testid="card-quality"]')
    check('画质卡片存在', (await quality.count()) === 1, `count=${await quality.count()}`)
    check('画质卡片含界面特效', (await quality.locator('[data-testid="setting-visual-effects"]').count()) === 1, '')
    const glowRow = quality.locator('[data-testid="setting-glow-render"]')
    check('画质卡片含背景光渲染', (await glowRow.count()) === 1, '')
    const glowValue = (await glowRow.locator('.select-value').innerText()).trim()
    check('背景光渲染默认 WEBGL', glowValue.startsWith('WEBGL'), glowValue)
    // 切到 CSS：根属性即时生效，供样式层二选一。
    await glowRow.locator('.select-trigger').click()
    await page.waitForTimeout(150)
    await page.locator('.select-menu[aria-label="背景光渲染"] .select-option', { hasText: 'CSS' }).click()
    await page.waitForTimeout(250)
    const mode = await page.evaluate(() => document.documentElement.getAttribute('data-glow-render'))
    check('背景光渲染切 CSS 生效', mode === 'css', `mode=${mode}`)
    await glowRow.locator('.select-trigger').click()
    await page.waitForTimeout(150)
    await page.locator('.select-menu[aria-label="背景光渲染"] .select-option', { hasText: 'WEBGL' }).click()
    await page.waitForTimeout(250)
    const back = await page.evaluate(() => document.documentElement.getAttribute('data-glow-render'))
    check('背景光渲染切回 WEBGL 生效', back === 'mesh', `mode=${back}`)
  })

  add('交互·设置分区切换', { route: 'settings', section: 'general' }, async page => {
    await page.locator('.settings-tabs button', { hasText: '备份' }).click()
    await page.waitForTimeout(200)
    const active = await page.locator('.settings-tabs button.active').innerText()
    check('交互·设置分区切换 生效', active.trim() === '数据与备份', active)
    const hash = await page.evaluate(() => location.hash)
    check('交互·设置分区切换 同步 hash', /section=backup/.test(hash), hash)
  })

  add('交互·卸载弹窗（Revision 11）', { route: 'settings', section: 'installation', runtime: 'stub' }, async page => {
    const entry = page.locator('[data-testid="runtime-uninstall"]')
    check('交互·卸载弹窗 非托管来源也常显入口', (await entry.count()) === 1)
    const entryText = (await entry.innerText()).trim()
    check('交互·卸载弹窗 入口文案无省略号', entryText === '卸载', entryText)
    await entry.click()
    await page.waitForTimeout(300)
    const modal = page.locator('[data-testid="runtime-uninstall-modal"] .modal')
    check('交互·卸载弹窗 打开', (await modal.count()) === 1)
    check(
      '交互·卸载弹窗 范围两级',
      (await page.locator('[data-testid="uninstall-scope-full"]').count()) === 1 &&
        (await page.locator('[data-testid="uninstall-scope-body"]').count()) === 1,
    )
    const backup = await page.locator('[data-testid="uninstall-backup"]').innerText()
    check('交互·卸载弹窗 备份只提醒、不阻断', /未检测到可恢复备份/.test(backup), backup)
    // 折叠的 <details> 里 innerText 不含正文，用 textContent 取全集（与原型一致，默认折叠）。
    const planText = await page.locator('[data-testid="uninstall-plan-objects"]').evaluate(el => el.textContent || '')
    check('交互·卸载弹窗 给出将执行的固定命令', /npm uninstall -g @bitkyc08\/opencodex/.test(planText), planText)
    check('交互·卸载弹窗 未确认前主行动禁用', await page.locator('[data-testid="uninstall-confirm"]').isDisabled())
    const hint = await page.locator('[data-testid="uninstall-ack-hint"]').innerText()
    check('交互·卸载弹窗 禁用原因可读', /勾选/.test(hint), hint)
    const geometry = await page.evaluate(() => {
      const modalEl = document.querySelector('[data-testid="runtime-uninstall-modal"] .modal')
      const ack = document.querySelector('[data-testid="uninstall-ack"]')
      if (!(modalEl instanceof HTMLElement) || !(ack instanceof HTMLElement)) return null
      const m = modalEl.getBoundingClientRect()
      const a = ack.getBoundingClientRect()
      return { overflow: modalEl.scrollHeight - modalEl.clientHeight, top: a.top - m.top, bottom: a.bottom - m.top, height: m.height }
    })
    check('交互·卸载弹窗 无滚动条', !!geometry && geometry.overflow <= 0, JSON.stringify(geometry))
    check('交互·卸载弹窗 确认项常驻可见', !!geometry && geometry.top >= 0 && geometry.bottom <= geometry.height, JSON.stringify(geometry))
    await shot(page, '交互-卸载弹窗-确认')

    await page.locator('[data-testid="uninstall-ack"]').check()
    await page.waitForTimeout(150)
    check('交互·卸载弹窗 勾选后主行动可用', !(await page.locator('[data-testid="uninstall-confirm"]').isDisabled()))
    await page.locator('[data-testid="uninstall-confirm"]').click()
    await page.waitForTimeout(350)
    const title = await page.locator('[data-testid="uninstall-progress-title"]').innerText()
    check('交互·卸载弹窗 走到完成态', /卸载完成/.test(title), title)
    const steps = await page.locator('[data-testid="uninstall-steps"]').innerText()
    check('交互·卸载弹窗 完成态为真实步骤', /执行官方 ocx uninstall/.test(steps), steps)
    const residue = await page.locator('[data-testid="uninstall-residue"]').innerText()
    check('交互·卸载弹窗 残留核验逐项给出结论', /已清除/.test(residue), residue)
    const consoleBox = await page.evaluate(() => {
      const el = document.querySelector('[data-testid="uninstall-console"]')
      return el instanceof HTMLElement ? Math.round(el.getBoundingClientRect().height) : -1
    })
    check('交互·卸载弹窗 命令行明细高度固定 132px', consoleBox === 132, String(consoleBox))
    await shot(page, '交互-卸载弹窗-完成')
  })

  add('交互·主题切换', { route: 'overview' }, async page => {
    await page.locator('.theme-seg button[aria-label="深色"]').click()
    await page.waitForTimeout(200)
    const dark = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    check('交互·主题切换 深色', dark === 'dark', String(dark))
    await page.locator('.theme-seg button[aria-label="浅色"]').click()
    await page.waitForTimeout(200)
    const light = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    check('交互·主题切换 浅色', light === 'light', String(light))
  })

  add('交互·通知分类与动作', { route: 'overview', panel: '1' }, async page => {
    const tabs = page.locator('.notification-cats button')
    check('交互·通知中心 分类数', (await tabs.count()) === 5, String(await tabs.count()))
    const runTab = page.locator('.notification-cats button', { hasText: '运行' })
    await runTab.click()
    await page.waitForTimeout(150)
    check('交互·通知分类 选中', (await runTab.getAttribute('aria-selected')) === 'true')
    const count = Number((await runTab.locator('.n-count').innerText()).trim())
    const visible = await page.locator('.notification-list .notification-item').count()
    check('交互·通知分类 过滤数量一致', visible === count, `visible=${visible} count=${count}`)
  })

  // 无后端时，通知写动作（全部已读 / 清理已解决）必须显式失败、不得假成功：状态保持、无成功假象。
  // 成功路径由 ① 层覆盖（tests/notification-actions.test.ts、notification-center.test.ts）。
  add('交互·通知写动作无后端不假成功', { route: 'overview', panel: '1' }, async page => {
    const dotBefore = (await page.locator('.notification-dot').innerText()).trim()
    await page.locator('.notification-head-actions button', { hasText: '全部已读' }).click()
    await page.waitForTimeout(200)
    const dotAfter = (await page.locator('.notification-dot').innerText()).trim()
    check('交互·全部已读 无后端保持未读（不假成功）', dotAfter === dotBefore, `${dotBefore}->${dotAfter}`)
    const resolvedBefore = await page.locator('.notification-list .notification-item.resolved').count()
    await page.locator('.notification-head-actions button', { hasText: '清理已解决' }).click()
    await page.waitForTimeout(200)
    const resolvedAfter = await page.locator('.notification-list .notification-item.resolved').count()
    check('交互·清理已解决 无后端保持现状（不假成功）', resolvedAfter === resolvedBefore, `${resolvedBefore}->${resolvedAfter}`)
  })

  add('交互·诊断分区切换', { route: 'logs', tab: 'doctor' }, async page => {
    await page.locator('.diag-tabs button', { hasText: '日志历史' }).click()
    await page.waitForTimeout(200)
    const active = await page.locator('.diag-tabs button.active').first().innerText()
    check('交互·诊断分区切换 生效', active.trim() === '日志历史', active)
    const view = await page.locator('.diag-view.active').first().innerText()
    check('交互·诊断分区切换 内容切换', /应用日志|调用日志/.test(view), view.slice(0, 40))
  })

  add('交互·托盘平台切换', { route: 'tray' }, async page => {
    await page.locator('.tray-os button', { hasText: 'Windows 通知区' }).click()
    await page.waitForTimeout(200)
    check('交互·托盘切换 Windows', (await page.locator('.tray-statusbar.windows').count()) === 1)
    await page.locator('.tray-os button', { hasText: 'macOS 菜单栏' }).click()
    await page.waitForTimeout(150)
    check('交互·托盘切换 macOS', (await page.locator('.tray-menubar.mac').count()) === 1)
  })

  // 就绪态主线才是详情入口：环境准备态（检查中/前置缺失）主线按 §25.3 不充当无效详情入口。
  add('交互·概览主线详情入口', { route: 'overview', env: 'ready' }, async page => {
    check('交互·概览主线详情入口 存在', (await page.locator('.motion-mainline').count()) === 1)
    await page.locator('.motion-mainline').click()
    await page.waitForTimeout(200)
    check('交互·概览主线详情入口 打开弹窗', (await page.locator('.motion-modal').count()) === 1)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(150)
    const closed = (await page.locator('.motion-modal').count()) === 0
    check('交互·概览主线详情入口 关闭', closed)
  })

  add('交互·扩展分页签', { route: 'extensions', tab: 'skills' }, async page => {
    await page.locator('.ext-tabs button', { hasText: 'MCP' }).click()
    await page.waitForTimeout(200)
    const active = await page.locator('.ext-tabs button.active').innerText()
    check('交互·扩展分页签 生效', active.trim() === 'MCP', active)
  })

  add('交互·正常无故障提示', { route: 'overview' }, async page => {
    check('交互·正常无故障提示', (await page.locator('.ui-fault-modal').count()) === 0)
  })

  // 2026-09-28 用户要求：故障提示带恢复动作，改为居中弹窗（原来是窗口顶部一条横条）。
  add('交互·故障提示为弹窗与恢复', { route: 'overview', scenario: 'fault' }, async page => {
    const modal = page.locator('.ui-fault-modal')
    check('交互·故障弹窗出现', (await modal.count()) === 1)
    check('交互·故障弹窗 role=alertdialog', (await modal.getAttribute('role')) === 'alertdialog')
    check('交互·故障弹窗 modal 语义', (await modal.getAttribute('aria-modal')) === 'true')
    check('交互·故障弹窗有遮罩', (await page.locator('.ui-fault-mask').count()) === 1)
    const text = await modal.locator('.ui-fault-text').innerText()
    check('交互·故障弹窗含原因', /界面异常/.test(text), text)
    const geom = await page.evaluate(() => {
      const r = document.querySelector('.ui-fault').getBoundingClientRect()
      return { cx: r.left + r.width / 2, cy: r.top + r.height / 2, vw: window.innerWidth, vh: window.innerHeight }
    })
    check('交互·故障弹窗居中', Math.abs(geom.cx - geom.vw / 2) <= 2 && Math.abs(geom.cy - geom.vh / 2) <= 2, JSON.stringify(geom))
    await modal.locator('button', { hasText: '关闭提示' }).click()
    await page.waitForTimeout(150)
    check('交互·故障弹窗可关闭（恢复）', (await page.locator('.ui-fault-modal').count()) === 0)
  })

  // IMP-04 §14.3：运行期兜底条此前没有制品内触发入口。诊断中心的「界面诊断」自检通过
  // 抛出一个受控异常，走真实错误边界（Vue errorHandler）置位兜底条，链路端到端可验证。
  add('交互·界面诊断自检（制品内触发故障弹窗）', { route: 'logs', tab: 'doctor' }, async page => {
    const fault = page.locator('.ui-fault-modal')
    check('自检·初始无故障弹窗', (await fault.count()) === 0)
    check('自检·入口文案是「界面诊断」', (await page.locator('#uiSelfCheckBtn').innerText()).trim() === '界面诊断')
    check('自检·Doctor 按钮用中文名', (await page.locator('.toolbar button', { hasText: '环境诊断' }).count()) >= 1)
    await page.locator('#uiSelfCheckBtn').click()
    await page.waitForTimeout(200)
    check('自检·触发后出现故障弹窗', (await fault.count()) === 1)
    check('自检·故障弹窗 role=alertdialog', (await fault.getAttribute('role')) === 'alertdialog')
    const text = await fault.locator('.ui-fault-text').innerText()
    check('自检·故障弹窗含自检原因', /界面异常自检/.test(text), text)
    // 生产构建下 Vue 的第三参数是错误文档 URL，绝不能进用户可见文案（UI规范 §10 / IMP-04 §19.17）。
    check('自检·故障弹窗文案不含英文链接', !/https?:\/\//i.test(text), text)
    const actions = await fault.locator('.ui-fault-actions button').allInnerTexts()
    check('自检·故障弹窗提供两个恢复动作', actions.map(t => t.trim()).join(',') === '关闭提示,重新加载界面', actions.join(','))
    await fault.locator('button', { hasText: '关闭提示' }).click()
    await page.waitForTimeout(150)
    check('自检·「关闭提示」恢复', (await page.locator('.ui-fault-modal').count()) === 0)
    await page.locator('#uiSelfCheckBtn').click()
    await page.waitForTimeout(200)
    check('自检·可重复触发', (await page.locator('.ui-fault-modal').count()) === 1)
    // 遮罩点击也关闭（与其它弹窗一致）
    await page.locator('.ui-fault-mask').click({ position: { x: 10, y: 10 } })
    await page.waitForTimeout(200)
    check('自检·点遮罩恢复', (await page.locator('.ui-fault-modal').count()) === 0)
    await page.locator('#uiSelfCheckBtn').click()
    await page.waitForTimeout(200)
    // 「重新加载界面」整页重载：重载后应回到干净状态，不再显示故障弹窗。
    await page.locator('.ui-fault button', { hasText: '重新加载界面' }).click()
    await page.waitForLoadState('load').catch(() => {})
    await page.waitForSelector('#app .app-window', { timeout: 15000 }).catch(() => {})
    await page.waitForTimeout(400)
    check('自检·「重新加载界面」后无故障弹窗', (await page.locator('.ui-fault-modal').count()) === 0)
  })

  for (const [name, params, fn] of cases) {
    if (!want(name)) continue
    const { page, errors } = await openPage(browser, params)
    try {
      await fn(page)
      check(`${name} 无未捕获错误`, errors.length === 0, errors.join(' | '))
    } catch (err) {
      check(`${name} 执行`, false, err && err.message ? err.message : String(err))
    } finally {
      await shot(page, `interact-${name}`)
      await page.close()
    }
  }
}

await main().catch(err => {
  console.log('ERROR: ' + (err && err.stack ? err.stack : err))
  process.exitCode = 1
})
