// Actual shared-prototype UI only; never a native-product or production gate.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import fs from 'node:fs/promises'
import path from 'node:path'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:http'
import { execFileSync } from 'node:child_process'

const methodRoot = path.dirname(fileURLToPath(import.meta.url))
const mode = process.argv[2] || 'initial'
assert(['initial', 'remaining-before', 'remaining-before-visible', 'remaining-after', 'remaining-extras'].includes(mode))
const out = mode === 'initial' ? methodRoot : path.join(methodRoot, mode)
if (mode !== 'initial') await fs.mkdir(out)
const repo = path.resolve(methodRoot, '../../..')
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
const layoutFailures = []
async function captureTree(page, selector, name, scene) {
  const geometry = await page.locator(selector + ' .proto-tree-cell').evaluateAll(cells =>
    cells.filter(cell => cell.getClientRects().length > 0).map(cell => {
    const label = cell.querySelector('.proto-tree-text')
    const labelBox = label.getBoundingClientRect()
    const lineHeight = parseFloat(getComputedStyle(label).lineHeight)
    const firstCenter = labelBox.top + lineHeight / 2
    const row = cell.closest('.proto-tree-row')
    const controls = [...cell.children].filter(el => el !== label)
    if (row?.tagName === 'TR') controls.push(...row.querySelectorAll('td:last-child button'))
    return { text: label.textContent, firstCenter, lineHeight, labelHeight: labelBox.height,
      controls: controls.filter(el => el.getClientRects().length > 0).map(el => {
        const box = el.getBoundingClientRect()
        return { kind: el.tagName, className: el.getAttribute('class'), height: box.height,
          centerDelta: box.top + box.height / 2 - firstCenter }
      }) }
  }))
  assert(geometry.length, name + ' has no rendered shared rows')
  const shot = `${scene.size}-${scene.theme}-${scene.effects}-${name}.png`
  await page.locator(selector).screenshot({ path: path.join(out, shot) })
  const frame = await page.locator('.window').boundingBox()
  const mismatches = geometry.flatMap(row => row.controls.filter(control => Math.abs(control.centerDelta) > 1)
    .map(control => ({ text: row.text, ...control })))
  if (mismatches.length) layoutFailures.push({ ...scene, name, mismatches })
  cases.push({ ...scene, name, frame, geometry, screenshot: shot, aligned: mismatches.length === 0 })
}
async function remaining(page, scene) {
  await page.goto(`http://127.0.0.1:${server.address().port}/#settings?section=backup`)
  await page.waitForSelector('#maintenanceBackupFiles')
  await page.evaluate(({ size, theme, effects }) => {
    setWinSize(size, size === 900 ? 600 : 760)
    document.documentElement.dataset.theme = theme
    document.documentElement.dataset.effects = effects
  }, scene)
  for (const id of ['month', 'action', 'record'])
    await page.locator(`[data-maint-action="backup-tree-toggle"][data-target="${id}"]`).click()
  assert.equal(await page.locator('#maintenanceFileRow-preferences').count(), 1)
  const record = page.locator('[data-maint-action="backup-tree-toggle"][data-target="record"]')
  await record.focus()
  await page.keyboard.press('Space')
  assert.equal(await page.locator('#maintenanceFileRow-preferences').count(), 0)
  assert(await record.evaluate(el => el === document.activeElement))
  await page.keyboard.press('Enter')
  assert.equal(await page.locator('#maintenanceFileRow-preferences').count(), 1)
  await captureTree(page, '#maintenanceBackupFiles', 'backup', scene)
  cases.at(-1).spaceEnterAndRerenderFocus = true

  await page.goto(`http://127.0.0.1:${server.address().port}/#sync?tab=file`)
  await page.waitForSelector('[data-sx="import"]')
  await page.evaluate(({ size, theme, effects }) => {
    setWinSize(size, size === 900 ? 600 : 760)
    document.documentElement.dataset.theme = theme
    document.documentElement.dataset.effects = effects
    window.syncPrototype.setImportSample('keyless')
  }, scene)
  // At 900px this is a genuinely narrower application window. The initial
  // run varied browser viewports while leaving the prototype frame at1180px.
  if (scene.size === 900) {
    for (const tab of ['file', 'webdav']) {
      if (tab === 'webdav') await page.locator('#sxTab-webdav').click()
      await page.locator('[data-sx="expand"][data-id="channels"]').click()
      await captureTree(page, '#syncContent', 'scope-' + tab, scene)
    }
    await page.locator('#sxTab-file').click()
  }
  await page.locator('[data-sx="import"]').click()
  await page.locator('[data-sx="choose-import-file"]').click()
  await page.locator('#modalConfirm').click()
  await page.waitForSelector('.sx-compare-tree')
  const group = page.locator('[data-sx="compare-expand"]').first()
  const initial = await page.evaluate(() => window.syncPrototype.state().comparison.selected.slice().sort())
  if (await group.getAttribute('aria-expanded') === 'false') await group.click()
  const pick = page.locator('[data-sx-import-pick]:not(:disabled)').first()
  await pick.click()
  assert(await pick.evaluate(el => el === document.activeElement))
  await page.keyboard.press('Space')
  assert.deepEqual(await page.evaluate(() => window.syncPrototype.state().comparison.selected.slice().sort()), initial)
  assert.equal(await group.getAttribute('aria-expanded'), 'true')
  await captureTree(page, '#modal .sx-compare-tree', 'import-compare', scene)
  cases.at(-1).checkboxSpaceFocusAndExpansionIsolation = true
  await page.locator('#modalCancel').click()

  await page.goto(`http://127.0.0.1:${server.address().port}/#models`)
  await page.waitForFunction(() => !!window.modelPrototype)
  await page.evaluate(({ size, theme, effects }) => {
    setWinSize(size, size === 900 ? 600 : 760)
    document.documentElement.dataset.theme = theme
    document.documentElement.dataset.effects = effects
    window.modelPrototype.scene('incoming')
  }, scene)
  await page.locator('[data-mp="save"]:not(:disabled)').first().click()
  await page.waitForSelector('#modal .mp-changes')
  const summary = page.locator('#modal .mp-changes details summary').first()
  await summary.focus()
  await page.keyboard.press('Space')
  assert.equal(await summary.evaluate(el => el.parentElement.open), false)
  await page.keyboard.press('Enter')
  assert.equal(await summary.evaluate(el => el.parentElement.open), true)
  assert(await summary.evaluate(el => el === document.activeElement))
  await captureTree(page, '#modal .mp-changes', 'model-policy-compare', scene)
  cases.at(-1).nativeDetailsSpaceEnterAndFocus = true
  await page.locator('#modalCancel').click()
}
async function extras(page, scene) {
  const configure = () => page.evaluate(({ size, theme, effects }) => {
    setWinSize(size, size === 900 ? 600 : 760)
    document.documentElement.dataset.theme = theme
    document.documentElement.dataset.effects = effects
  }, scene)
  await page.goto(`http://127.0.0.1:${server.address().port}/#sync?tab=file`)
  await page.waitForSelector('[data-sx="import"]')
  await configure()
  await page.evaluate(() => window.syncPrototype.setImportSample('keyless'))
  await page.locator('[data-sx="import"]').click()
  await page.locator('[data-sx="choose-import-file"]').click()
  await page.locator('#modalConfirm').click()
  const selected = await page.evaluate(() => window.syncPrototype.state().comparison.selected.slice().sort())
  await page.locator('[data-sx="compare-expand"][data-id="templates"]').click()
  const dependency = page.locator('[data-sx-import-pick="template:gpt"]')
  assert(await dependency.isDisabled() && await dependency.isChecked())
  assert.equal(await dependency.evaluate(el => { el.focus(); return document.activeElement === el }), false)
  assert.deepEqual(await page.evaluate(() => window.syncPrototype.state().comparison.selected.slice().sort()), selected)
  await captureTree(page, '#modal .sx-compare-tree', 'disabled-dependency', scene)
  cases.at(-1).dependentTemplateDisabledSelectedAndUnfocusable = true
  await page.locator('#modalCancel').click()
  await page.goto(`http://127.0.0.1:${server.address().port}/#models`)
  await page.waitForFunction(() => !!window.modelPrototype)
  await configure()
  const savedVersions = await page.evaluate(() => JSON.stringify(window.modelPrototype.state().templates))
  await page.locator('[data-mp="tab"][data-tab="templates"]').click()
  await page.locator('[data-mp="open-template-editor"][data-id="gpt"]').click()
  await page.locator('#mpTemplateDescription').fill('树表比较检查：仅修改本页候选说明，取消后不新增版本。')
  await page.locator('[data-mp="save-template"]:not(:disabled)').click()
  await page.waitForSelector('#modal .mp-template-change-group')
  await captureTree(page, '#modal .mp-changes', 'template-save-compare', scene)
  await page.locator('#modalCancel').click()
  assert.equal(await page.evaluate(() => JSON.stringify(window.modelPrototype.state().templates)), savedVersions)
  cases.at(-1).cancelPreservedSavedVersions = true
}
try {
  browserServer = await chromium.launchServer({ executablePath: edge, headless: true })
  const child = browserServer.process()
  exitPromise = new Promise(resolve => child.once('exit', (code, signal) => resolve({ pid: child.pid, code, signal })))
  browser = await chromium.connect(browserServer.wsEndpoint())
  const origin = `http://127.0.0.1:${server.address().port}`
  for (const width of [1600, 1050]) for (const theme of ['light', 'dark']) for (const effects of ['high', 'mid', 'low']) {
    if (mode === 'remaining-extras' && !((width === 1600 && theme === 'light' && effects === 'high')
      || (width === 1050 && theme === 'dark' && effects === 'low'))) continue
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
      if (mode === 'remaining-extras') {
        await extras(page, { size: width === 1600 ? 1180 : 900, theme, effects })
        assert.equal(errors.length, 0, errors.join('; '))
        continue
      }
      if (mode !== 'initial') {
        await remaining(page, { size: width === 1600 ? 1180 : 900, theme, effects })
        assert.equal(errors.length, 0, errors.join('; '))
        continue
      }
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
    result: failure || layoutFailures.length ? 'failed' : 'passed', cases, failure, layoutFailures,
    childExit, loopbackServerClosed: true, blockedRequests,
    sourceFilesUnchanged: true, scope: mode === 'initial' ? 'shared prototype sync file/WebDAV checkboxes and first-line geometry only'
      : 'actual1180/900px prototype backup, import and model-policy shared trees; no application or business implementation',
    nativeProductStarted: false, nativeTreeMatrixPassed: false, productionGatePassed: false, releaseAccepted: false })
}
console.log(JSON.stringify({ result: failure || layoutFailures.length ? 'failed' : 'passed',
  cases: cases.length, failure, layoutFailures: layoutFailures.map(({ name, size, theme, effects, mismatches }) =>
    ({ name, size, theme, effects, controls: mismatches.length, maximumDelta: Math.max(...mismatches.map(v => Math.abs(v.centerDelta))) })),
  nativeProductStarted: false, output: out }))
if (failure || layoutFailures.length) process.exitCode = 1
