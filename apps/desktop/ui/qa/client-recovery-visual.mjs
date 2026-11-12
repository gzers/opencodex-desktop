// Full diagnostic route + shared dialog, explicit memory-only audit fixtures.
import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'
import net from 'node:net'
import { spawn } from 'node:child_process'
import { fileURLToPath, pathToFileURL } from 'node:url'
import assert from 'node:assert/strict'
import { pickEngine } from './engine.mjs'
const ui = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const out = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocxd-client-recovery-visual-'))
fs.mkdirSync(out, { recursive: true })
const probe = net.createServer(); await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve))
const port = probe.address().port; await new Promise(resolve => probe.close(resolve))
const base = 'http://127.0.0.1:' + port
const server = spawn(path.join(ui, 'node_modules/.bin/vite'), ['--host', '127.0.0.1', '--port', String(port), '--strictPort'], { cwd: ui, stdio: ['ignore', 'pipe', 'pipe'] })
const logs = fs.createWriteStream(path.join(out, 'vite.log'))
server.stdout.pipe(logs); server.stderr.pipe(logs)
const report = { accepted: false, scope: 'mock browser UI/prototype only; no native finalization or business operations', engine: '', checks: [] }
const record = (name, detail) => report.checks.push({ name, pass: true, detail })
let browser
async function screenshot(page, name) {
  await page.screenshot({ path: path.join(out, name + '.png'), fullPage: false })
}
async function geometry(page) {
  const value = await page.locator('dialog[open]').evaluate(d => { const r = d.getBoundingClientRect(); return { top: r.top, bottom: r.bottom, left: r.left, right: r.right, height: innerHeight, width: innerWidth, overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth } })
  assert(value.top >= -1 && value.bottom <= value.height + 1 && value.left >= -1 && value.right <= value.width + 1 && value.overflow <= 1, JSON.stringify(value))
  return value
}
try {
  const deadline = Date.now() + 20000
  while (true) { try { if ((await fetch(base)).ok) break } catch {} if (Date.now() > deadline) throw Error('Vite did not start'); await new Promise(resolve => setTimeout(resolve, 100)) }
  const engine = await pickEngine(); report.engine = engine.name; browser = await engine.launch()
  for (const scene of (process.env.AUDIT_RECOVERY_SCENES?.split(',') ?? ['eligible', 'pending', 'unknown', 'active', 'none', 'unsupported', 'error', 'slow', 'refresh-failed'])) {
    const sizes = ['eligible','continue','rollback','rearm','early','prepare'].includes(scene) ? [[1180, 760, 'light'], [900, 600, 'dark']] : [[1180, 760, 'light']]
    for (const [width, height, theme] of sizes) {
      const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 })
      const errors = []; page.on('pageerror', e => errors.push(e.message))
      await page.goto(base + '/?__audit=1&route=logs&tab=doctor&env=ready&effects=low&theme=' + theme + '&client-recovery=' + scene)
      await page.waitForFunction(() => !!window.__clientRecoveryFixture)
      await page.getByRole('button', { name: '检查客户端恢复', exact: true }).click()
      await page.locator('dialog[open]').waitFor()
      await page.waitForFunction(() => !document.querySelector('dialog[open]')?.textContent.includes('正在核对原会话和恢复记录'))
      const name = scene + '-' + width + '-' + theme
      if(scene.startsWith('prepare')){
        await page.getByText('启动准备已中断，尚未开始捕获原件。',{exact:true}).waitFor()
        assert.equal(await page.getByRole('button',{name:'确认继续恢复',exact:true}).count(),0)
        await page.getByRole('button',{name:'取消启动准备',exact:true}).click()
        await page.getByText('保持当前配置，不恢复或删除文件，也不会自动启动。',{exact:true}).waitFor()
        const size=await geometry(page);await screenshot(page,name+'-review')
        if(scene==='prepare'){
          await page.getByRole('button',{name:'返回',exact:true}).click()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='cancelPreparation').length),0)
          await page.getByRole('button',{name:'取消启动准备',exact:true}).click()
        }
        await page.getByRole('button',{name:'确认取消启动准备',exact:true}).click()
        if(scene==='prepare-slow'){
          await page.getByText('正在核对并取消启动准备…',{exact:true}).waitFor()
          await page.getByRole('button',{name:'关闭',exact:true}).last().click()
          await page.getByRole('button',{name:'查看恢复进度',exact:true}).click()
        }
        if(scene==='prepare-error'){
          await page.getByText('启动准备或停止状态尚未核实；原现场保留。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
        }else{
          await page.getByText('启动准备已取消，当前配置保持不变。',{exact:true}).waitFor()
          if(scene==='prepare-refresh-failed'){
            await page.getByText('收尾已完成，运行状态读取失败。请刷新运行状态。',{exact:true}).waitFor()
            assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
            await page.evaluate(()=>window.__clientRecoveryFixture.setScene('prepare'))
            await page.getByRole('button',{name:'刷新运行状态',exact:true}).click()
          }
          await page.getByText('服务保持停止，现在可以启动。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),false)
        }
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='cancelPreparation').length),1)
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>['finalize','continue','rollback','rearm'].includes(n)).length),0)
        await screenshot(page,name+'-result');record(name,{geometry:size,explicitReview:true,noReplayCommand:true,oneConfirmation:true})
      }else if(scene.startsWith('early')){
        await page.getByText('启动前原件保持完整，原会话尚未结束。',{exact:true}).waitFor()
        assert.equal(await page.getByRole('button',{name:'确认继续恢复',exact:true}).count(),0)
        await page.getByRole('button',{name:'核对并结束会话',exact:true}).click()
        await page.getByText('客户端配置保持不变，不会自动启动服务。',{exact:true}).waitFor()
        const size=await geometry(page);await screenshot(page,name+'-review')
        if(scene==='early'){
          await page.getByRole('button',{name:'返回',exact:true}).click()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='finishUnchanged').length),0)
          await page.getByRole('button',{name:'核对并结束会话',exact:true}).click()
        }
        await page.getByRole('button',{name:'确认结束会话',exact:true}).click()
        if(scene==='early-slow'){
          await page.getByText('正在核对原件并结束原会话…',{exact:true}).waitFor()
          await page.getByRole('button',{name:'关闭',exact:true}).last().click()
          await page.getByRole('button',{name:'查看恢复进度',exact:true}).click()
        }
        if(scene==='early-error'){
          await page.getByText('原件或停止状态尚未核实；原件与现场保留。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
        }else{
          await page.getByText('原件保持完整，原会话已结束。',{exact:true}).waitFor()
          if(scene==='early-refresh-failed'){
            await page.getByText('收尾已完成，运行状态读取失败。请刷新运行状态。',{exact:true}).waitFor()
            assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
            await page.evaluate(()=>window.__clientRecoveryFixture.setScene('early'))
            await page.getByRole('button',{name:'刷新运行状态',exact:true}).click()
          }
          await page.getByText('服务保持停止，现在可以启动。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),false)
        }
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='finishUnchanged').length),1)
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>['finalize','continue','rollback','rearm'].includes(n)).length),0)
        await screenshot(page,name+'-result');record(name,{geometry:size,explicitReview:true,noReplayCommand:true,oneConfirmation:true})
      }else if(scene.startsWith('rollback')||scene.startsWith('rearm')){
        const action=scene.startsWith('rollback')?'rollback':'rearm'
        const choose=action==='rollback'?'撤销本次恢复':'重新准备恢复',confirm=action==='rollback'?'确认撤销恢复':'确认重新准备'
        await page.getByRole('button',{name:choose,exact:true}).click()
        await page.getByRole('button',{name:confirm,exact:true}).waitFor()
        let size=await geometry(page);await screenshot(page,name+'-review')
        if(scene===action){
          await page.getByRole('button',{name:'返回',exact:true}).click()
          assert.equal(await page.evaluate(action=>window.__clientRecoveryFixture.calls.filter(n=>n===action).length,action),0)
          await page.getByRole('button',{name:choose,exact:true}).click()
        }
        await page.getByRole('button',{name:confirm,exact:true}).click()
        if(scene.endsWith('-slow')){
          await page.getByText(action==='rollback'?'正在撤销本次恢复…':'正在重新准备原件恢复…',{exact:true}).waitFor()
          await page.getByRole('button',{name:'关闭',exact:true}).last().click()
          await page.getByRole('button',{name:'查看恢复进度',exact:true}).click()
        }
        if(scene.endsWith('-error')){
          await page.getByText(action==='rollback'?'撤销恢复未确认完成；原件与当前文件保留。':'恢复准备未确认完成；原件与当前文件保留。',{exact:true}).waitFor()
          assert.equal(await page.getByRole('button',{name:confirm,exact:true}).count(),0)
        }else{
          if(scene==='rollback-refresh-failed'){
            await page.getByText('操作已完成，恢复状态读取失败。请刷新恢复状态。',{exact:true}).waitFor()
            await screenshot(page,name+'-refresh-failure')
            await page.evaluate(()=>window.__clientRecoveryFixture.setScene('rollback'))
            await page.getByRole('button',{name:'刷新恢复状态',exact:true}).click()
          }
          await page.getByRole('button',{name:action==='rollback'?'重新准备恢复':'确认继续恢复',exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='continue').length),0)
        }
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
        assert.equal(await page.evaluate(action=>window.__clientRecoveryFixture.calls.filter(n=>n===action).length,action),1)
        await screenshot(page,name+'-result');record(name,{geometry:size,explicitReview:true,oneMutation:true,gateRetained:true,noAutomaticContinue:true})
        if(scene==='rollback'&&width===1180){
          await page.getByRole('button',{name:'重新准备恢复',exact:true}).click()
          await page.getByRole('button',{name:'确认重新准备',exact:true}).click()
          await page.getByRole('button',{name:'确认继续恢复',exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='continue').length),0)
          await page.getByRole('button',{name:'确认继续恢复',exact:true}).click()
          await page.getByText('服务保持停止，现在可以启动。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),false)
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='rollback').length),1)
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='rearm').length),1)
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='continue').length),1)
          record('app-full-rollback-rearm-continue',{threeSeparateConfirmations:true,onlyFinalRestoreUnblocks:true})
        }
      }else if (scene.startsWith('continue')) {
        assert.equal(await page.getByRole('button',{name:'确认完成收尾',exact:true}).count(),0)
        await page.getByRole('button',{name:'确认继续恢复',exact:true}).waitFor()
        await page.getByText('诊断详情',{exact:true}).click()
        await page.getByText('尚未登记',{exact:true}).waitFor()
        const size=await geometry(page);await screenshot(page,name)
        if(scene==='continue'){
          await page.keyboard.press('Escape');await page.locator('dialog[open]').waitFor({state:'hidden'})
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='continue').length),0)
          await page.getByRole('button',{name:'检查客户端恢复',exact:true}).click()
          await page.getByRole('button',{name:'确认继续恢复',exact:true}).waitFor()
        }
        await page.getByRole('button',{name:'确认继续恢复',exact:true}).click()
        if(scene==='continue-slow'){
          await page.getByText('正在继续恢复客户端配置…',{exact:true}).waitFor()
          await page.getByRole('button',{name:'关闭',exact:true}).last().click()
          await page.getByRole('button',{name:'查看恢复进度',exact:true}).click()
        }
        if(scene==='continue-error'){
          await page.getByText('恢复现场已变化，请重新检查。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
        }else{
          await page.getByText('服务保持停止，现在可以启动。',{exact:true}).waitFor()
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),false)
        }
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='continue').length),1)
        assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.calls.filter(n=>n==='finalize').length),0)
        await screenshot(page,name+'-result');record(name,{geometry:size,oneExplicitContinuation:true,noFinalizeSubstitution:true})
      } else if (scene === 'eligible') {
        await page.keyboard.press('Escape'); await page.locator('dialog[open]').waitFor({ state: 'hidden' })
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.calls.filter(n => n === 'finalize').length), 0)
        await page.getByRole('button', { name: '检查客户端恢复', exact: true }).click()
        await page.getByRole('button', { name: '确认完成收尾', exact: true }).waitFor()
        await page.getByText('诊断详情', { exact: true }).click(); const size = await geometry(page)
        const focusTrace = []
        for (let i = 0; i < 9; i++) {
          const beforeFocus = await page.locator('dialog[open]').evaluate(d => ({ inside: d.contains(document.activeElement), tag: document.activeElement?.tagName, label: document.activeElement?.getAttribute('aria-label'), text: document.activeElement?.textContent?.slice(0, 100) }))
          await page.keyboard.press('Tab')
          const focus = await page.locator('dialog[open]').evaluate(d => ({ inside: d.contains(document.activeElement), tag: document.activeElement?.tagName, label: document.activeElement?.getAttribute('aria-label'), text: document.activeElement?.textContent?.slice(0, 100) }))
          focusTrace.push({ before: beforeFocus, after: focus })
          if (!focus.inside) { fs.writeFileSync(path.join(out, name + '-focus-failure.json'), JSON.stringify(focusTrace, null, 2)); await screenshot(page, name + '-focus-failure') }
          assert(focus.inside, JSON.stringify(focusTrace))
        }
        assert.equal(await page.getByRole('button', { name: '确认完成收尾', exact: true }).count(), 1)
        await screenshot(page, name)
        await page.getByRole('button', { name: '确认完成收尾', exact: true }).click()
        await page.getByText('服务保持停止，现在可以启动。', { exact: true }).waitFor()
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.calls.filter(n => n === 'finalize').length), 1)
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.state().blocked), false)
        await screenshot(page, name + '-success'); record(name, { geometry: size, matchedOutcomeAndScopedStatusUnblocks: true, keyboardCancelNoMutationAndFocusContained: true })
      } else if (scene === 'slow') {
        await page.getByRole('button', { name: '确认完成收尾', exact: true }).click()
        await page.getByText('正在完成恢复收尾…', { exact: true }).waitFor()
        await page.getByRole('button', { name: '关闭', exact: true }).last().click()
        await page.getByRole('button', { name: '查看恢复进度', exact: true }).click()
        await page.getByText('服务保持停止，现在可以启动。', { exact: true }).waitFor()
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.calls.filter(n => n === 'finalize').length), 1)
        record(name, { closedReopenedOneConfirmation: true })
      } else if (scene === 'refresh-failed') {
        await page.getByRole('button', { name: '确认完成收尾', exact: true }).click()
        await page.getByText('收尾已完成，运行状态读取失败。请刷新运行状态。', { exact: true }).waitFor()
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.state().blocked), true)
        await screenshot(page, name)
        await page.evaluate(() => window.__clientRecoveryFixture.setScene('eligible'))
        await page.getByRole('button', { name: '刷新运行状态', exact: true }).click()
        await page.getByText('服务保持停止，现在可以启动。', { exact: true }).waitFor()
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.calls.filter(n => n === 'finalize').length), 1)
        record(name, { statusOnlyRetry: true })
      } else {
        if(scene==='rolled-back'){
          await page.getByText('启动前原件仍保留，尚未恢复；服务保持停止。',{exact:true}).waitFor()
          assert.equal(await page.getByRole('button',{name:'确认继续恢复',exact:true}).count(),0)
          assert.equal(await page.evaluate(()=>window.__clientRecoveryFixture.state().blocked),true)
        }
        assert.equal(await page.getByRole('button', { name: '确认完成收尾', exact: true }).count(), 0)
        assert.equal(await page.evaluate(() => window.__clientRecoveryFixture.calls.filter(n => n === 'finalize').length), 0)
        const size = await geometry(page); await screenshot(page, name); record(name, { noExecutablePartialRecovery: true, geometry: size })
      }
      assert.deepEqual(errors, []); await page.close()
    }
  }
  const prototype = process.env.AUDIT_PROTOTYPE_INDEX
  assert(prototype && fs.existsSync(prototype), 'explicit existing total prototype index required')
  for (const scene of (process.env.AUDIT_RECOVERY_SCENES?.split(',') ?? ['eligible', 'pending', 'unknown', 'slow', 'refresh-failed'])) {
    const page = await browser.newPage({ viewport: { width: 1240, height: 900 } })
    const errors = []; page.on('pageerror', e => errors.push(e.message))
    await page.goto(pathToFileURL(prototype).href + '#logs?tab=doctor')
    await page.waitForFunction(() => !!window.clientRecovery024)
    await page.evaluate(scene => { window.clientRecovery024.setScene(scene); window.clientRecovery024.open() }, scene)
    await page.waitForFunction(() => !window.clientRecovery024.snapshot().reading)
    await page.waitForFunction(() => getComputedStyle(document.querySelector('#modal')).maxWidth === '560px')
    assert.equal(await page.locator('#modalTitle').textContent(), '客户端配置恢复')
    const size = await page.locator('#modal').evaluate(d => { const r = d.getBoundingClientRect(); return { width: r.width, top: r.top, bottom: r.bottom, height: innerHeight } })
    assert(size.width <= 560 && size.top >= 0 && size.bottom <= size.height, JSON.stringify(size))
    if(scene.startsWith('prepare')){
      assert.equal(await page.locator('#modalConfirm').textContent(),'取消启动准备')
      await page.locator('#modalConfirm').click()
      assert.equal(await page.locator('#modalConfirm').textContent(),'确认取消启动准备')
      await page.getByText('保持当前配置，不恢复或删除文件，也不会自动启动。',{exact:true}).waitFor()
      await screenshot(page,'prototype-'+scene+'-review')
      if(scene==='prepare'){
        await page.locator('#modalCancel').click();assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),0)
        await page.locator('#modalConfirm').click()
      }
      await page.locator('#modalConfirm').click()
      if(scene==='prepare-slow'){
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().executing)
        await page.locator('#modalCancel').click();await page.evaluate(()=>window.clientRecovery024.open())
      }
      if(scene==='prepare-error')await page.getByText('启动准备或停止状态尚未核实；原现场保留。',{exact:true}).waitFor()
      else{
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().outcome)
        await page.getByText('启动准备已取消，当前配置保持不变。',{exact:true}).waitFor()
        if(scene==='prepare-refresh-failed'){assert.equal(await page.locator('#modalConfirm').textContent(),'刷新运行状态');await page.locator('#modalConfirm').click()}
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().ready),true)
      }
      assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),1)
    }else     if(scene.startsWith('early')){
      assert.equal(await page.locator('#modalConfirm').textContent(),'核对并结束会话')
      await page.locator('#modalConfirm').click()
      assert.equal(await page.locator('#modalConfirm').textContent(),'确认结束会话')
      await page.getByText('客户端配置保持不变，不会自动启动服务。',{exact:true}).waitFor()
      await screenshot(page,'prototype-'+scene+'-review')
      if(scene==='early'){
        await page.locator('#modalCancel').click();assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),0)
        await page.locator('#modalConfirm').click()
      }
      await page.locator('#modalConfirm').click()
      if(scene==='early-slow'){
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().executing)
        await page.locator('#modalCancel').click();await page.evaluate(()=>window.clientRecovery024.open())
      }
      if(scene==='early-error')await page.getByText('原件或停止状态尚未核实；原件与现场保留。',{exact:true}).waitFor()
      else{
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().outcome)
        await page.getByText('原件保持完整，原会话已结束。',{exact:true}).waitFor()
        if(scene==='early-refresh-failed'){assert.equal(await page.locator('#modalConfirm').textContent(),'刷新运行状态');await page.locator('#modalConfirm').click()}
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().ready),true)
      }
      assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),1)
    }else if(scene.startsWith('rollback')||scene.startsWith('rearm')){
      const action=scene.startsWith('rollback')?'rollback':'rearm'
      const confirm=action==='rollback'?'确认撤销恢复':'确认重新准备'
      if(action==='rollback')await page.locator('[data-recovery-rollback]').click();else await page.locator('#modalConfirm').click()
      assert.equal(await page.locator('#modalConfirm').textContent(),confirm)
      await screenshot(page,'prototype-'+scene+'-review')
      if(scene===action){
        await page.locator('#modalCancel').click()
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),0)
        if(action==='rollback')await page.locator('[data-recovery-rollback]').click();else await page.locator('#modalConfirm').click()
      }
      await page.locator('#modalConfirm').click()
      if(scene.endsWith('-slow')){
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().executing)
        await page.locator('#modalCancel').click()
        await page.evaluate(()=>window.clientRecovery024.open())
      }
      if(scene.endsWith('-error'))await page.getByText(action==='rollback'?'撤销恢复未确认完成；原件与当前文件保留。':'恢复准备未确认完成；原件与当前文件保留。',{exact:true}).waitFor()
      else{
        await page.waitForFunction(()=>!window.clientRecovery024.snapshot().executing&&window.clientRecovery024.snapshot().outcome)
        if(scene==='rollback-refresh-failed'){
          assert.equal(await page.locator('#modalConfirm').textContent(),'刷新恢复状态');await page.locator('#modalConfirm').click()
        }
        assert.equal(await page.locator('#modalConfirm').textContent(),action==='rollback'?'重新准备恢复':'确认继续恢复')
      }
      assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),1)
      assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().finalized),false)
      if(scene==='rollback'){
        await page.locator('#modalConfirm').click();assert.equal(await page.locator('#modalConfirm').textContent(),'确认重新准备')
        await page.locator('#modalConfirm').click()
        await page.waitForFunction(()=>!window.clientRecovery024.snapshot().executing&&window.clientRecovery024.snapshot().phase==='restore_prepared')
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),2)
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().finalized),false)
        await page.locator('#modalConfirm').click()
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().finalized)
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),3)
        record('prototype-full-rollback-rearm-continue',{threeSeparateConfirmations:true,memoryOnly:true})
      }
    }else if (scene.startsWith('continue')) {
      assert.equal(await page.locator('#modalConfirm').textContent(),'确认继续恢复')
      await page.locator('#clientRecovery024Body summary').click()
      await page.locator('#clientRecovery024Body').getByText('尚未登记',{exact:true}).waitFor()
      await screenshot(page,'prototype-'+scene);await page.locator('#modalConfirm').click()
      if(scene==='continue-error'){
        await page.getByText('恢复现场已变化，请重新检查。',{exact:true}).waitFor()
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().outcome),false)
      }else{
        await page.waitForFunction(()=>window.clientRecovery024.snapshot().outcome)
        assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().ready),true)
      }
      assert.equal(await page.evaluate(()=>window.clientRecovery024.snapshot().confirmations),1)
    } else if (scene === 'eligible' || scene === 'slow' || scene === 'refresh-failed') {
      assert.equal(await page.locator('#modalConfirm').textContent(), '确认完成收尾')
      await screenshot(page, 'prototype-' + scene)
      await page.locator('#modalConfirm').click()
      await page.waitForFunction(() => window.clientRecovery024.snapshot().outcome)
      if (scene === 'refresh-failed') { assert.equal(await page.locator('#modalConfirm').textContent(), '刷新运行状态'); await page.locator('#modalConfirm').click() }
      assert.equal(await page.evaluate(() => window.clientRecovery024.snapshot().confirmations), 1)
      assert.equal(await page.evaluate(() => window.clientRecovery024.snapshot().ready), true)
    } else {
      if(scene==='rolled-back'){
        await page.locator('#clientRecovery024Body').getByText('启动前原件仍保留，尚未恢复；服务保持停止。',{exact:true}).waitFor()
        assert.notEqual(await page.locator('#modalConfirm').textContent(),'确认继续恢复')
      }
      assert.notEqual(await page.locator('#modalConfirm').textContent(), '确认完成收尾')
      await screenshot(page, 'prototype-' + scene)
    }
    assert.deepEqual(errors, []); record('prototype-' + scene, { sameUserFlow: true, memoryOnly: true }); await page.close()
  }
  report.result = 'passed'
} catch (error) { report.result = 'failed'; report.error = String(error.stack || error); process.exitCode = 1 }
finally {
  if (browser) await browser.close()
  server.kill('SIGTERM'); await new Promise(resolve => server.once('exit', resolve)); logs.end()
  report.cleanup = { browserClosed: true, ownedViteProcessExited: true }
  fs.writeFileSync(path.join(out, 'summary.json'), JSON.stringify(report, null, 2))
  console.log(JSON.stringify(report, null, 2))
}
