// 模型页实屏对照（0.2.7）：用开发审计装置渲染真实组件，逐档截图并断言关键结构。
import fs from 'fs'
import path from 'path'
import os from 'os'
import net from 'net'
import { spawn } from 'child_process'
import { fileURLToPath } from 'url'
import { createRequire } from 'module'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-model-shots-'))
fs.mkdirSync(outDir, { recursive: true })
const PORT = 5199
const BASE = 'http://127.0.0.1:' + PORT

function resolvePlaywright() {
  const npx = path.join(os.homedir(), '.npm', '_npx')
  if (fs.existsSync(npx)) for (const d of fs.readdirSync(npx)) {
    const c = path.join(npx, d, 'node_modules', 'playwright-core')
    if (fs.existsSync(path.join(c, 'package.json'))) return c
  }
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
const pwPath = resolvePlaywright(), exe = newestChromiumShell()
if (!pwPath || !exe) { console.log('BLOCKED: 缺少 playwright-core 或 chromium'); process.exit(2) }
const { chromium } = createRequire(pwPath + '/')( 'playwright-core')

function waitPort(port, timeoutMs = 40000) {
  return new Promise((resolve, reject) => {
    const start = Date.now()
    const tick = () => {
      const sock = net.connect(port, '127.0.0.1')
      sock.once('connect', () => { sock.destroy(); resolve() })
      sock.once('error', () => {
        sock.destroy()
        if (Date.now() - start > timeoutMs) reject(new Error('vite 未就绪'))
        else setTimeout(tick, 200)
      })
    }
    tick()
  })
}

const cases = [
  { name: 'models-1180x760-light', width: 1180, height: 760, theme: 'light' },
  { name: 'models-1180x760-dark', width: 1180, height: 760, theme: 'dark' },
  { name: 'models-1440x900-light', width: 1440, height: 900, theme: 'light' },
  { name: 'models-900x600-light', width: 900, height: 600, theme: 'light' },
  // 界面缩放：--ui-zoom 生效后仍不得横向溢出，渠道表与操作组保持可用。
  { name: 'models-1440x900-scale150', width: 1440, height: 900, theme: 'light', scale: 150 },
  { name: 'models-1440x900-scale200', width: 1440, height: 900, theme: 'dark', scale: 200 },
]
const results = []
const check = (name, ok, detail = '') => results.push({ name, ok: !!ok, detail })

async function main() {
  const viteBin = path.resolve(uiDir, 'node_modules', '.bin', 'vite')
  const server = spawn(viteBin, ['--host', '127.0.0.1', '--port', String(PORT), '--strictPort'], {
    cwd: uiDir, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env },
  })
  const stop = () => { try { server.kill('SIGTERM') } catch { /* ignore */ } }
  process.on('exit', stop)
  let browser
  try {
    await waitPort(PORT)
    browser = await chromium.launch({ executablePath: exe })
    for (const c of cases) {
      const page = await browser.newPage({ viewport: { width: c.width, height: c.height }, deviceScaleFactor: 2 })
      const errors = []
      page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
      page.on('pageerror', e => errors.push('pageerror: ' + e.message))
      if (c.scale) {
        // 模块夹具装在 route=models；缩放不是模型页控件，这里直接给页面根设 --ui-zoom，
        // 用来验证缩放后的布局（等同于偏好生效时的呈现），并断言该值真的落到 CSS 变量。
        await page.goto(BASE + '/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=' + c.theme)
        await page.waitForSelector('.channel-table', { timeout: 20000 }).catch(() => {})
        await page.evaluate(value => document.documentElement.style.setProperty('--ui-zoom', String(value)), c.scale / 100)
        await page.waitForTimeout(600)
      } else {
        await page.goto(BASE + '/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=' + c.theme)
      }
      await page.waitForSelector('#app .app-window', { timeout: 20000 }).catch(() => {})
      await page.waitForTimeout(1200)
      // 结构：渠道表 + 工具栏 + 只有一个保存入口，没有独立预览按钮。
      const structure = await page.evaluate(() => {
        const text = document.body.innerText
        const tables = document.querySelectorAll('.channel-table').length
        const previewButtons = [...document.querySelectorAll('button')].filter(b => b.innerText.trim() === '预览选择器').length
        const saveButtons = [...document.querySelectorAll('button')].filter(b => b.innerText.trim() === '保存').length
        const zoom = getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom').trim()
        return {
          zoom,
          tables, previewButtons, saveButtons,
          hasUpstream: text.includes('读取现有渠道'),
          hasSearch: !!document.querySelector('input[aria-label="搜索渠道"]'),
          overflow: Math.max(0, document.documentElement.scrollWidth - document.documentElement.clientWidth),
        }
      })
      check(c.name + ' 渠道表存在', structure.tables >= 1, 'tables=' + structure.tables)
      check(c.name + ' 无独立预览按钮', structure.previewButtons === 0, 'preview=' + structure.previewButtons)
      check(c.name + ' 保留读取现有渠道', structure.hasUpstream)
      check(c.name + ' 无横向溢出', structure.overflow <= 1, 'overflow=' + structure.overflow)
      if (c.scale) check(c.name + ' 缩放已生效', structure.zoom === String(c.scale / 100), 'ui-zoom=' + structure.zoom + ' expect=' + c.scale / 100)
      check(c.name + ' 无未捕获错误', errors.length === 0, errors.slice(0, 2).join(' | '))
      await page.screenshot({ path: path.join(outDir, c.name + '.png'), fullPage: false })
      await page.close()
    }
  } finally {
    if (browser) await browser.close().catch(() => {})
    stop()
  }
  const failed = results.filter(r => !r.ok)
  fs.writeFileSync(path.join(outDir, 'model-shots-report.json'), JSON.stringify({ total: results.length, failed: failed.length, results }, null, 2))
  for (const r of results) console.log((r.ok ? 'PASS ' : 'FAIL ') + r.name + (r.detail ? ' | ' + r.detail : ''))
  console.log('shots: ' + outDir + '  failed=' + failed.length + '/' + results.length)
  process.exit(failed.length ? 1 : 0)
}
main().catch(e => { console.error(e); process.exit(3) })

