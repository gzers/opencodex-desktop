// Actual shared-prototype UI only; never a native-product or production gate.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import fs from 'node:fs/promises'
import path from 'node:path'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:http'
import { execFileSync } from 'node:child_process'

const out = path.dirname(fileURLToPath(import.meta.url))
const repo = path.resolve(out, '../../..')
const prototype = path.join(repo, 'docs/04-项目资料/01-项目支撑资料/01-产品原型/原型')
const require = createRequire('C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright-core/package.json')
const { chromium } = require('playwright-core')
const edge = 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'
const sha = data => createHash('sha256').update(data).digest('hex')
const write = (name, value) => fs.writeFile(path.join(out, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' })
const sourcePaths = ['index.html', 'tree-table.js', 'tree-table.css', 'sync-config.js', 'sync-config.css',
  'maintenance-ui.js', 'maintenance-ui.css', 'model-config.js', 'model-config.css']
const sourceFiles = []
for (const name of sourcePaths) {
  const data = await fs.readFile(path.join(prototype, name))
  sourceFiles.push({ path: name, bytes: data.length, sha256: sha(data) })
}
const baseline = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim()
await write('binding.json', { baseline, sourceFiles, prototypeEntry: path.relative(repo, path.join(prototype, 'index.html')),
  node: process.execPath, nodeVersion: process.version, browser: edge, headless: true,
  nativeProductStarted: false, frontendProductionAcceptance: false, dpiMatrixAdded: false,
  externalRequestsAllowed: false, realFileOrWebDavOperations: false })

const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.woff2': 'font/woff2', '.json': 'application/json' }
const server = createServer(async (request, response) => {
  try {
    assert(request.method === 'GET' || request.method === 'HEAD')
    const relative = decodeURIComponent(new URL(request.url, 'http://127.0.0.1').pathname).replace(/^\/+/, '') || 'index.html'
    const target = path.resolve(prototype, relative)
    assert(target.startsWith(prototype + path.sep))
    const data = await fs.readFile(target)
    response.writeHead(200, { 'Content-Type': (mime[path.extname(target)] || 'application/octet-stream') + '; charset=utf-8' })
    response.end(request.method === 'HEAD' ? undefined : data)
  } catch {
    response.writeHead(404)
    response.end()
  }
})
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
let browserServer, browser, exitPromise
const cases = [], blockedRequests = []
const startedAt = new Date().toISOString()
let failure = null
try {
  browserServer = await chromium.launchServer({ executablePath: edge, headless: true })
  const child = browserServer.process()
  exitPromise = new Promise(resolve => child.once('exit', (code, signal) => resolve({ pid: child.pid, code, signal })))
  browser = await chromium.connect(browserServer.wsEndpoint())
  const origin = `http://127.0.0.1:${server.address().port}`
  for (const width of [1600, 1050]) for (const theme of ['light', 'dark']) for (const effects of ['high', 'mid', 'low']) {
    const context = await browser.newContext({ viewport: { width, height: 1100 } })
    await context.route('**/*', route => {
      if (route.request().url().startsWith(origin + '/')) return route.continue()
      blockedRequests.push(route.request().url())
      return route.abort()
    })
    const page = await context.newPage()
    const errors = []
    page.on('pageerror', error => errors.push(error.message))
    try {
      await page.goto(origin + '/#sync?tab=file')
      await page.waitForSelector('[data-sx="expand"][data-id="channels"]')
      await page.evaluate(({ theme, effects }) => {
        document.documentElement.dataset.theme = theme
        document.documentElement.dataset.effects = effects
      }, { theme, effects })
      for (const tab of ['file', 'webdav']) {
        if (tab === 'webdav') await page.locator('#sxTab-webdav').click()
        const state = () => page.evaluate(() => {
          const value = window.syncPrototype.state()
          value.selected.sort()
          return value
        })
        const before = await state()
        const toggle = page.locator('[data-sx="expand"][data-id="channels"]')
        const collapsedBefore = await toggle.getAttribute('aria-expanded')
        if (collapsedBefore === 'false') await toggle.click()
        assert.deepEqual((await state()).selected, before.selected, 'Expansion changed business selection')
        const check = page.locator('[data-sx-object]:not(:disabled)').first()
        const objectId = await check.getAttribute('data-sx-object')
        const originalSelected = await check.isChecked()
        await check.click()
        assert.equal(await toggle.getAttribute('aria-expanded'), 'true', 'Selection collapsed its group')
        assert.equal(await check.isChecked(), !originalSelected)
        const clickFocusRetained = await check.evaluate(el => document.activeElement === el)
        assert(clickFocusRetained, 'Checkbox render lost keyboard focus')
        const mixed = await page.locator('[data-sx-group="channels"]').evaluate(el => el.indeterminate)
        assert(mixed, 'Partially selected group is not mixed')
        await page.keyboard.press('Space')
        assert.equal(await check.isChecked(), originalSelected, 'Space did not toggle checkbox')
        assert.equal(await toggle.getAttribute('aria-expanded'), 'true')
        assert.deepEqual((await state()).selected, before.selected, 'Checkbox roundtrip changed other selections')
        await toggle.focus()
        await page.keyboard.press('Space')
        assert.equal(await toggle.getAttribute('aria-expanded'), 'false', 'Space did not collapse group')
        assert.deepEqual((await state()).selected, before.selected)
        await page.keyboard.press('Enter')
        assert.equal(await toggle.getAttribute('aria-expanded'), 'true', 'Enter did not expand group')
        assert.deepEqual((await state()).selected, before.selected)
        const geometry = await page.locator('.sx-table .proto-tree-cell').evaluateAll(cells => cells.map(cell => {
          const label = cell.querySelector('.proto-tree-text')
          const box = label.getBoundingClientRect()
          const firstCenter = box.top + parseFloat(getComputedStyle(label).lineHeight) / 2
          return { text: label.textContent, firstCenter, labelHeight: box.height,
            lineHeight: parseFloat(getComputedStyle(label).lineHeight),
            controls: [...cell.children].filter(el => el !== label).map(el => {
              const rect = el.getBoundingClientRect()
              return { kind: el.tagName, className: el.className, height: rect.height,
                centerDelta: rect.top + rect.height / 2 - firstCenter }
            }) }
        }))
        assert(geometry.length > 9)
        assert(geometry.every(row => row.controls.every(control => Math.abs(control.centerDelta) <= 1)),
          'Shared prototype first-line control alignment differs')
        assert(geometry.some(row => row.labelHeight > row.lineHeight * 1.5), 'Multiline row was not exercised')
        const shot = `${width}-${theme}-${effects}-${tab}.png`
        await page.locator('#syncContent').screenshot({ path: path.join(out, shot) })
        cases.push({ width, theme, effects, tab, objectId, clickFocusRetained, partiallySelectedMixed: mixed,
          expansionSelectionSeparated: true, checkboxSpaceAndButtonSpaceEnter: true, geometry, screenshot: shot })
      }
      assert.equal(errors.length, 0, errors.join('; '))
    } finally {
      await context.close()
    }
  }
} catch (error) {
  failure = { message: error.message, stack: error.stack }
} finally {
  await browser?.close()
  await browserServer?.close()
  const childExit = exitPromise ? await exitPromise : null
  await new Promise(resolve => server.close(resolve))
  for (const item of sourceFiles) {
    assert.equal(sha(await fs.readFile(path.join(prototype, item.path))), item.sha256, 'Prototype source changed during read-only QA')
  }
  await write('result.json', { startedAt, finishedAt: new Date().toISOString(), baseline,
    result: failure ? 'failed' : 'passed', cases, failure, childExit, loopbackServerClosed: true, blockedRequests,
    sourceFilesUnchanged: true, scope: 'shared prototype sync file/WebDAV checkboxes and first-line geometry only',
    nativeProductStarted: false, nativeTreeMatrixPassed: false, productionGatePassed: false, releaseAccepted: false })
}
console.log(JSON.stringify({ result: failure ? 'failed' : 'passed', cases: cases.length, failure,
  nativeProductStarted: false, output: out }))
if (failure) process.exitCode = 1
