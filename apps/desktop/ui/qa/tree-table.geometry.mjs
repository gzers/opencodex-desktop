// Render the production Vue component. Browser geometry proves the regression;
// this is not Windows WebView, physical DPI, or complete native acceptance.
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'
import { mkdir, writeFile } from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import path from 'node:path'

const ui = fileURLToPath(new URL('../', import.meta.url))
const { createServer } = await import(pathToFileURL(path.join(ui, 'node_modules/vite/dist/node/index.js')).href)
assert(process.env.PLAYWRIGHT_CORE && process.env.CHROMIUM && process.env.TREE_GEOMETRY_OUT,
  'Provide PLAYWRIGHT_CORE, CHROMIUM and TREE_GEOMETRY_OUT')
const require = createRequire(path.join(process.env.PLAYWRIGHT_CORE, 'package.json'))
const { chromium } = require('playwright-core')
const output = path.resolve(process.env.TREE_GEOMETRY_OUT)
await mkdir(output, { recursive: true })
const server = await createServer({ root: ui, server: { host: '127.0.0.1', port: 0 } })
const results = []
let browser
try {
  await server.listen()
  const address = server.httpServer.address()
  browser = await chromium.launch({ executablePath: process.env.CHROMIUM, headless: true })
  for (const theme of ['light', 'dark']) {
    for (const scale of [1, 1.25, 1.5, 2]) {
      const page = await browser.newPage({ viewport: { width: 1700, height: 1200 } })
      await page.goto(`http://127.0.0.1:${address.port}/qa/tree-table.geometry.html`)
      await page.waitForSelector('.tree-name')
      await page.evaluate(({ theme, scale }) => {
        document.documentElement.dataset.theme = theme
        document.documentElement.style.zoom = String(scale)
      }, { theme, scale })
      const rows = await page.evaluate(scale => [...document.querySelectorAll('tbody tr')].map(row => {
        const label = row.querySelector('.tree-label')
        const top = label.getBoundingClientRect().top
        const firstLine = top + parseFloat(getComputedStyle(label).lineHeight) * scale / 2
        const controls = [...row.querySelectorAll('.tree-toggle,.tree-icon,.tree-check,td:last-child button')]
          .map(control => {
            const box = control.getBoundingClientRect()
            return { className: control.className, center: box.top + box.height / 2, height: box.height,
              offsetFromFirstLine: box.top + box.height / 2 - firstLine }
          })
        const text = row.querySelector('.tree-cell:not(.tree-cell-action)') ?? row.querySelector('td')
        const purposeLine = text.getBoundingClientRect().top
          + (parseFloat(getComputedStyle(text).paddingTop) + 11) * scale
        return { label: label.textContent, firstLine, controls, purposeLine,
          labelHeight: label.getBoundingClientRect().height, lineHeight: parseFloat(getComputedStyle(label).lineHeight) }
      }), scale)
      for (const row of rows) {
        assert(row.controls.every(control => Math.abs(control.offsetFromFirstLine) <= 1),
          `${theme}/${scale}/${row.label}: first-line alignment failed ${JSON.stringify(row.controls)}`)
        assert(Math.abs(row.purposeLine - row.firstLine) <= 1, 'Purpose column first line differs')
        assert(row.controls.filter(control => control.className.includes('btn'))
          .every(control => control.height >= 34 * scale), 'Action hit height was reduced')
      }
      assert(rows.some(row => row.labelHeight > row.lineHeight * scale * 1.5), 'Multiline name was not exercised')
      const selections = await page.locator('.tree-check').evaluateAll(inputs =>
        inputs.map(input => ({ disabled: input.disabled, mixed: input.indeterminate, selected: input.checked })))
      assert(selections.some(input => input.mixed) && selections.some(input => input.disabled)
        && selections.some(input => input.selected), 'Selection states missing')
      const screenshot = `${theme}-${scale}.png`
      await page.screenshot({ path: path.join(output, screenshot) })
      results.push({ theme, scale, rows, selections, screenshot, result: 'passed' })
      await page.close()
    }
  }
  await writeFile(path.join(output, 'geometry.json'), JSON.stringify({
    result: 'passed', scope: 'production shared component in Chromium; CSS zoom, not physical DPI',
    cases: results,
  }, null, 2))
  console.log(`PASS: ${results.length} shared tree geometry cases`)
} catch (error) {
  await writeFile(path.join(output, 'geometry.json'), JSON.stringify({
    result: 'failed', error: String(error), scope: 'browser geometry only', cases: results,
  }, null, 2))
  throw error
} finally { await browser?.close(); await server.close() }
