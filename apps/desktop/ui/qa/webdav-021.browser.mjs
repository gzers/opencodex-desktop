// Explicit DEV fixtures only. External requests blocked; no native IPC or daily configuration.
import fs from 'node:fs/promises'
import path from 'node:path'
import { pickEngine } from './engine.mjs'
const out = process.env.AUDIT_OUT
if (!out) throw Error('AUDIT_OUT required')
const origin = process.env.AUDIT_BASE ?? 'http://127.0.0.1:53970'
await fs.mkdir(out, { recursive: true })
const engine = await pickEngine(), browser = await engine.launch(), checks = [], errors = [], screenshots = []
function check(name, passed) { checks.push({ name, passed: !!passed }); if (!passed) throw Error(name) }
try {
  for (const spec of [{ width: 1440, height: 900, theme: 'light', effects: 'low', scale: 100 }, { width: 1180, height: 760, theme: 'dark', effects: 'mid', scale: 100 }, { width: 900, height: 650, theme: 'light', effects: 'high', scale: 100 }, { width: 1180, height: 760, theme: 'dark', effects: 'high', scale: 200 }]) {
    const prefix = [spec.width,spec.theme,spec.effects,spec.scale].join('-'), p = await browser.newPage({ viewport: { width: spec.width, height: spec.height } })
    p.on('pageerror', e => errors.push(e.message))
    await p.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort())
    await p.goto(origin+'/?__audit=1&route=models&env=ready&runtime=running&prefs=local&effects='+spec.effects+'&theme='+spec.theme)
    await p.waitForSelector('.channel-table'); await p.evaluate(async()=>{(await import('/src/dev/modelFixtures.ts')).installModelFixtures()})
    await p.evaluate(async scale => {
      const { usePreferencesStore } = await import('/src/features/preferences/store.ts')
      const { useSyncStore } = await import('/src/features/sync/store.ts')
      const { useRouteStore } = await import('/src/stores/routes.ts')
      const { syncEndpointApi } = await import('/src/features/sync/api.ts')
      await usePreferencesStore().load(); await usePreferencesStore().setScale(scale)
      useSyncStore().adoptConfig(await syncEndpointApi.load())
      useRouteStore().go('overview')
    }, spec.scale)
    await p.waitForSelector('[data-testid=overview-remote-version]')
    check(prefix + ' configured is not disconnected', await p.locator('.mods .mod').last().innerText().then(t => t.includes('已配置 · 未检测')))
    const open = async () => { await p.evaluate(async () => (await import('/src/features/sync/store.ts')).useSyncStore().openConnection()); await p.locator('.sync-connection-dialog[open]').waitFor() }
    const shot = async name => { const file = prefix + '-' + name + '.png'; await p.screenshot({ path: path.join(out, file) }); screenshots.push(file) }
    await open()
    const dialog = p.locator('.sync-connection-dialog[open]')
    await p.waitForFunction(() => !document.querySelector('.imported-connections'))
    check(prefix + ' empty candidates hidden', await dialog.getByText('导入的连接', { exact: true }).count() === 0)
    check(prefix + ' one header X', await dialog.locator('.ui-dialog-header .modal-close').count() === 1)
    check(prefix + ' edit footer order', JSON.stringify(await dialog.locator('footer button').allTextContents()) === JSON.stringify(['取消', '保存连接']))
    await dialog.getByRole('button', { name: '测试连接', exact: true }).click()
    await dialog.getByRole('status').filter({ hasText: '连接成功' }).waitFor()
    check(prefix + ' saved test observed', await p.evaluate(async () => (await import('/src/features/sync/store.ts')).useSyncStore().webdavState === 'connected'))
    await shot('connection')
    await p.keyboard.press('Escape'); await dialog.waitFor({ state: 'hidden' })
    check(prefix + ' recent result on overview', await p.locator('.mods .mod').last().innerText().then(t => t.includes('最近检测成功')))
    await p.evaluate(async () => {
      const api = await import('/src/features/sync/api.ts')
      api.importedConnectionApi.list = async () => { throw new Error('fixture candidate read failure') }
    })
    await open(); await dialog.getByRole('alert').filter({ hasText: 'fixture candidate read failure' }).waitFor()
    check(prefix + ' failed list retry visible', await dialog.getByRole('button', { name: '重新读取', exact: true }).isVisible())
    await p.evaluate(async () => {
      const api = await import('/src/features/sync/api.ts')
      api.importedConnectionApi.list = async () => ({ baseline: 'fixture', items: [{ id: 'candidate', endpoint: { id: 'c', url: 'https://other.example.test', remotePath: 'portable', username: 'fixture', conflictPolicy: 'ask' }, hasPassword: true }] })
    })
    await dialog.getByRole('button', { name: '重新读取', exact: true }).click(); await dialog.locator('summary').waitFor()
    check(prefix + ' candidate count', await dialog.locator('summary').innerText() === '导入的连接（1）')
    await dialog.locator('summary').click(); await dialog.getByRole('button', { name: '启用此连接', exact: true }).click()
    const nested = p.locator('dialog[open]').filter({ hasText: '启用导入的WebDAV连接' }); await nested.waitFor()
    check(prefix + ' nested uses shared X', await nested.locator('.modal-close').count() === 1)
    await p.keyboard.press('Escape'); await nested.waitFor({ state: 'hidden' })
    check(prefix + ' Escape keeps parent', await dialog.isVisible())
    await shot('candidate-parent')
    await dialog.getByRole('button', { name: '取消', exact: true }).click(); await dialog.waitFor({ state: 'hidden' })
    await p.evaluate(async () => {
      const { modelTransferApi } = await import('/src/features/sync/modelTransfer.ts')
      modelTransferApi.checkSync = async () => ({ source: 'undetermined', reason: 'same_instant_different_content', hasRemote: true, localSavedAt: '2026-11-04T04:00:00Z', remoteSavedAt: '2026-11-04T12:00:00+08:00' })
    })
    await p.locator('.mods .mod').last().getByRole('button', { name: '同步', exact: true }).click()
    const preview = p.locator('dialog[open].transfer-review'); await preview.waitFor()
    await preview.getByText('无法判定', { exact: true }).waitFor()
    check(prefix + ' cannot confirm ambiguous time', await preview.getByRole('button', { name: '确认更新', exact: true }).isDisabled())
    check(prefix + ' no direction fabricated', await preview.innerText().then(t => t.includes('本机保存时间') && t.includes('云端保存时间') && !t.includes('云端配置 → 本机')))
    const bounds = await preview.boundingBox()
    check(prefix + ' preview fits viewport', bounds && bounds.x >= -1 && bounds.y >= -1 && bounds.x + bounds.width <= spec.width + 1 && bounds.y + bounds.height <= spec.height + 1)
    await shot('undetermined')
    await p.keyboard.press('Escape'); await preview.waitFor({ state: 'hidden' })
    check(prefix + ' keeps overview route', await p.evaluate(async () => (await import('/src/stores/routes.ts')).useRouteStore().current === 'overview'))
    await p.close()
  }
} finally {
  await browser.close()
  await fs.writeFile(path.join(out, 'report.json'), JSON.stringify({ engine: engine.name, source: 'current 0.2.21 development software with explicit DEV fixtures', scope: 'WebDAV observation/candidates and ambiguous sync preview; no real IPC/network/config', checks, screenshots, errors }, null, 2))
}
if (errors.length || checks.some(c => !c.passed)) throw Error(JSON.stringify({ errors, failed: checks.filter(c => !c.passed) }))
console.log(JSON.stringify({ engine: engine.name, passed: checks.length, screenshots: screenshots.length }))
