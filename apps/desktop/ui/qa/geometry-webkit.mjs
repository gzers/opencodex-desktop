// macOS 原生引擎族（WebKit）实屏复核：WKWebView 用的就是 WebKit。
//
// 目的：Chromium 通过不等于 macOS 原生窗口通过。本脚本用 Playwright 的 WebKit
// 跑与 geometry-visual WP58 相同的量化断言，并把「拖动排序」跑成真实指针事件序列。
// 环境缺失（未装 webkit 构建）时以退出码 2 报 BLOCKED，不把「没跑」当通过。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const PORT = 5197, BASE = 'http://127.0.0.1:' + PORT

function playwrightCandidates() {
  const dirs = []
  const npx = path.join(os.homedir(), '.npm', '_npx')
  if (fs.existsSync(npx)) for (const d of fs.readdirSync(npx)) dirs.push(path.join(npx, d, 'node_modules', 'playwright-core'))
  return dirs.filter(c => fs.existsSync(path.join(c, 'package.json')))
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

// 不同 playwright 版本自带的 webkit 修订号不同；逐个试，取第一个能启动的。
let webkit = null
for (const c of playwrightCandidates()) {
  try {
    const mod = createRequire(c + '/')('playwright-core')
    const b = await mod.webkit.launch()
    await b.close()
    webkit = mod.webkit
    break
  } catch { /* 继续试下一个 */ }
}
if (!webkit) { console.log('BLOCKED: 本机没有可用的 Playwright WebKit 构建'); process.exit(2) }

const results = []
const check = (n, ok, d = '') => results.push({ n, ok: !!ok, d })

const server = spawn(path.resolve(uiDir, 'node_modules', '.bin', 'vite'),
  ['--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
  { cwd: uiDir, stdio: ['ignore', 'pipe', 'pipe'] })
process.on('exit', () => { try { server.kill('SIGTERM') } catch {} })

let browser
try {
  await waitPort(PORT)
  browser = await webkit.launch()
  const page = await browser.newPage({ viewport: { width: 1180, height: 1000 }, deviceScaleFactor: 2 })
  const errors = []
  page.on('pageerror', e => errors.push('pageerror: ' + e.message))
  page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
  const events = []
  await page.exposeFunction('__rec', e => events.push(e))

  await page.goto(BASE + '/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
  await page.waitForSelector('.channel-table', { timeout: 25000 })
  await page.waitForTimeout(1200)
  const engine = await page.evaluate(() => navigator.userAgent)
  check('WK 引擎为 WebKit', /AppleWebKit/.test(engine) && !/Chrome\//.test(engine), engine.slice(0, 48))

  await page.locator('.channel-table tbody tr').first().locator('button', { hasText: '编辑' }).first().click()
  await page.waitForSelector('.channel-model-table', { timeout: 20000 })
  await page.waitForTimeout(900)
  const nav = page.locator('a', { hasText: '模型列表' }).first()
  if (await nav.count()) { await nav.click(); await page.waitForTimeout(600) }

  // —— 与 Chromium 同一组 WP58 量化断言 ——
  const m = await page.evaluate(() => {
    const rows = [...document.querySelectorAll('.channel-model-table tbody tr[data-model]')].slice(0, 3)
    const bar = document.querySelector('.model-toolbar')
    const items = bar ? [...bar.children].map(el => { const r = el.getBoundingClientRect(); return { h: Math.round(r.height), cy: Math.round(r.top + r.height / 2) } }) : []
    const box = bar?.querySelector('.ui-inline-check input[type=checkbox]')
    const search = bar?.querySelector('input[type=search]')
    return {
      rows: rows.map(tr => {
        const rr = tr.getBoundingClientRect(), mid = rr.top + rr.height / 2
        const d = el => { if (!el) return null; const r = el.getBoundingClientRect(); return Math.round(r.top + r.height / 2 - mid) }
        const s = tr.querySelector('.model-row-summary')
        return {
          rowH: Math.round(rr.height),
          cells: [...tr.children].map(td => getComputedStyle(td).verticalAlign),
          pick: d(tr.querySelector('.model-row-pick')), enable: d(tr.querySelector('.model-enabled-cell')), actions: d(tr.querySelector('.model-row-actions')),
          summaryLines: s ? s.children.length : 0, summaryDisplay: s ? getComputedStyle(s).display : '',
        }
      }),
      bar: bar ? {
        heights: [...new Set(items.filter(i => i.h >= 28).map(i => i.h))],
        centers: [...new Set(items.filter(i => i.h >= 28).map(i => i.cy))],
        checkboxW: box ? Math.round(box.getBoundingClientRect().width) : null,
        searchW: search ? Math.round(search.getBoundingClientRect().width) : null,
        barH: Math.round(bar.getBoundingClientRect().height),
      } : null,
      docOverflow: Math.max(0, document.documentElement.scrollWidth - document.documentElement.clientWidth),
    }
  })
  check('WK 模型行已渲染', m.rows.length >= 3, 'rows=' + m.rows.length)
  for (const [i, row] of m.rows.entries()) {
    check('WK 行' + (i + 1) + ' 单元格垂直居中', row.cells.every(v => v === 'middle'), 'cells=' + row.cells.join(','))
    check('WK 行' + (i + 1) + ' 勾选列与整行同轴', Math.abs(row.pick ?? 99) <= 2, 'delta=' + row.pick)
    check('WK 行' + (i + 1) + ' 启停列与整行同轴', Math.abs(row.enable ?? 99) <= 2, 'delta=' + row.enable)
    check('WK 行' + (i + 1) + ' 操作组与整行同轴', Math.abs(row.actions ?? 99) <= 2, 'delta=' + row.actions)
    check('WK 行' + (i + 1) + ' 摘要分两行显示', row.summaryDisplay === 'grid' && row.summaryLines === 2, 'display=' + row.summaryDisplay + ' lines=' + row.summaryLines)
  }
  check('WK 模型行高度未被摘要撑破', m.rows.every(r => r.rowH <= 112), 'rowH=' + m.rows.map(r => r.rowH).join('/'))
  check('WK 过滤区控件同高', m.bar && m.bar.heights.length === 1, 'h=' + (m.bar?.heights ?? []).join('/'))
  check('WK 过滤区控件同轴', m.bar && m.bar.centers.length === 1, 'centers=' + (m.bar?.centers ?? []).join('/'))
  check('WK 仅启用勾选框未被撑宽', m.bar && m.bar.checkboxW !== null && m.bar.checkboxW <= 20, 'w=' + m.bar?.checkboxW)
  check('WK 搜索框吃剩余宽度', m.bar && m.bar.searchW !== null && m.bar.searchW >= 200, 'w=' + m.bar?.searchW)
  check('WK 过滤区不高出单行', m.bar && m.bar.barH <= 38, 'barH=' + m.bar?.barH)
  check('WK 无横向溢出', m.docOverflow <= 1, 'overflow=' + m.docOverflow)

  // —— 拖动排序：真实指针序列。HTML5 拖放（dragstart/drop）在 WKWebView 里会静默失败，
  // 实现已改为指针拖拽；这里断言的是「按下→移动→松手」这条真实路径。
  await page.exposeFunction('__flag', v => events.push('flag:' + v))
  await page.evaluate(() => {
    window.__selAtDrop = ''
    window.addEventListener('pointerup', () => {
      window.__selAtDrop = String(window.getSelection() || '').replace(/\s+/g, ' ').trim().slice(0, 60)
      window.__flag('pointerup')
    }, true)
  })
  const order = () => page.evaluate(() => [...document.querySelectorAll('.channel-model-table tbody tr[data-model] .model-raw-id')].slice(0, 4).map(e => e.textContent))
  const rows = page.locator('.channel-model-table tbody tr[data-model]')
  await rows.nth(0).scrollIntoViewIfNeeded()
  await page.waitForTimeout(400)
  const before = await order()
  const sb = await rows.nth(0).locator('.model-raw-id').boundingBox()
  const db = await rows.nth(2).locator('.model-raw-id').boundingBox()
  check('WK 拖动源与落点在视口内', !!sb && !!db)
  if (sb && db) {
    const x = sb.x + 8, y0 = sb.y + sb.height / 2, y1 = db.y + db.height / 2
    await page.mouse.move(x, y0); await page.waitForTimeout(60)
    await page.mouse.down(); await page.waitForTimeout(120)
    for (let i = 1; i <= 20; i++) { await page.mouse.move(x, y0 + (y1 - y0) * i / 20); await page.waitForTimeout(40) }
    await page.mouse.up(); await page.waitForTimeout(500)
    const after = await order()
    const count = t => events.filter(e => e === t).length
    const selection = await page.evaluate(() => window.__selAtDrop)
    check('WK 拖拽用指针事件完成', count('flag:pointerup') === 1, 'pointerup=' + count('flag:pointerup'))
    check('WK 拖拽过程不框选文字', selection === '', 'selection=' + JSON.stringify(selection))
    check('WK 拖拽结束后解除禁选', await page.evaluate(() => !document.documentElement.classList.contains('ocx-pointer-drag')))
    check('WK 拖动后顺序按落点改变', JSON.stringify(before) !== JSON.stringify(after), before[0] + ' → ' + after.join('|'))
    check('WK 被拖型号落到目标位置', after[2] === before[0], 'after[2]=' + after[2] + ' expect=' + before[0])
    // Esc 取消：开始拖拽后按 Esc，顺序必须保持不变、且不留半透明行状态。
    const stable = await order()
    const sb2 = await rows.nth(0).locator('.model-raw-id').boundingBox()
    const db2 = await rows.nth(2).locator('.model-raw-id').boundingBox()
    if (sb2 && db2) {
      await page.mouse.move(sb2.x + 8, sb2.y + sb2.height / 2)
      await page.mouse.down()
      for (let i = 1; i <= 10; i++) { await page.mouse.move(sb2.x + 8, sb2.y + sb2.height / 2 + (db2.y - sb2.y) * i / 10); await page.waitForTimeout(30) }
      await page.keyboard.press('Escape')
      await page.mouse.up(); await page.waitForTimeout(300)
      const afterEsc = await order()
      check('WK Esc 取消拖拽不改顺序', JSON.stringify(afterEsc) === JSON.stringify(stable), stable[0] + ' vs ' + afterEsc[0])
      check('WK Esc 后无残留拖拽状态', await page.evaluate(() => !document.documentElement.classList.contains('ocx-pointer-drag') && !document.querySelector('.channel-model-table tbody tr.dragging')))
    }
  }
  check('WK 无控制台错误', errors.length === 0, errors.slice(0, 2).join(' | '))
  await page.screenshot({ path: path.join(process.env.AUDIT_OUT_DIR || os.tmpdir(), 'webkit-models.png') })

  // —— 拖到视口边缘要能自动滚动，否则长列表里拖不过一屏 ——
  {
    await page.setViewportSize({ width: 1180, height: 520 })
    await page.waitForTimeout(500)
    const rowsScroll = page.locator('.channel-model-table tbody tr[data-model]')
    await rowsScroll.nth(1).scrollIntoViewIfNeeded()
    await page.waitForTimeout(400)
    const src = await rowsScroll.nth(1).locator('.model-raw-id').boundingBox()
    const mainBox = await page.locator('main.main').boundingBox()
    if (src && mainBox) {
      const y0 = src.y + src.height / 2
      const yEnd = mainBox.y + mainBox.height - 14
      await page.mouse.move(src.x + 8, y0); await page.waitForTimeout(80)
      await page.mouse.down(); await page.waitForTimeout(120)
      for (let i = 1; i <= 12; i++) { await page.mouse.move(src.x + 8, y0 + (yEnd - y0) * i / 12); await page.waitForTimeout(45) }
      const beforeScroll = await page.evaluate(() => document.querySelector('main.main').scrollTop)
      await page.waitForTimeout(1200)
      const afterScroll = await page.evaluate(() => document.querySelector('main.main').scrollTop)
      await page.mouse.up(); await page.waitForTimeout(300)
      check('WK 拖到边缘自动滚动', afterScroll > beforeScroll + 5, beforeScroll + ' → ' + afterScroll)
      check('WK 自动滚动后收尾干净', await page.evaluate(() => !document.documentElement.classList.contains('ocx-pointer-drag')))
    }
  }

  // —— 渠道列表行排序：同一套指针拖拽（原先也用 HTML5 拖放，同样会掉 drop） ——
  const back = page.locator('button', { hasText: '返回渠道列表' }).first()
  if (await back.count()) { await back.click(); await page.waitForTimeout(800) }
  const channelRows = page.locator('.channel-table tbody tr')
  const channelCount = await channelRows.count()
  check('WK 渠道列表已返回', channelCount >= 3, 'rows=' + channelCount)
  if (channelCount >= 3) {
    const names = () => page.evaluate(() => [...document.querySelectorAll('.channel-table tbody tr')].slice(0, 4).map(tr => tr.querySelector('td:nth-child(2) strong')?.textContent))
    const beforeChannels = await names()
    const sb3 = await channelRows.nth(0).locator('td:nth-child(2) strong').boundingBox()
    const db3 = await channelRows.nth(2).locator('td:nth-child(2) strong').boundingBox()
    if (sb3 && db3) {
      const x3 = sb3.x + 10, y3 = sb3.y + sb3.height / 2, y4 = db3.y + db3.height / 2
      await page.mouse.move(x3, y3); await page.waitForTimeout(60)
      await page.mouse.down(); await page.waitForTimeout(100)
      for (let i = 1; i <= 16; i++) { await page.mouse.move(x3, y3 + (y4 - y3) * i / 16); await page.waitForTimeout(38) }
      const duringSelection = await page.evaluate(() => String(window.getSelection() || '').trim().slice(0, 40))
      await page.mouse.up(); await page.waitForTimeout(500)
      const afterChannels = await names()
      check('WK 渠道拖拽不框选文字', duringSelection === '', 'selection=' + JSON.stringify(duringSelection))
      check('WK 渠道拖拽按落点换序', JSON.stringify(beforeChannels) !== JSON.stringify(afterChannels), beforeChannels[0] + ' → ' + afterChannels.join('|'))
    }
  }
} finally {
  if (browser) await browser.close().catch(() => {})
  try { server.kill('SIGTERM') } catch {}
}

const failed = results.filter(r => !r.ok)
for (const r of results) console.log((r.ok ? 'PASS ' : 'FAIL ') + r.n + (r.d ? ' | ' + r.d : ''))
console.log('failed=' + failed.length + '/' + results.length)
process.exit(failed.length ? 1 : 0)
