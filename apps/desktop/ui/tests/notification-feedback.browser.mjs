// 真实页面点击/截图，文件写入、重启与通知清理均为显式内存适配；不代表真机验收。
import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import net from 'node:net'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { pickEngine } from '../qa/engine.mjs'

const uiDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-notification-feedback-'))
const port = Number(process.env.AUDIT_PORT || 5214)
const effects=process.env.AUDIT_EFFECTS||'high'
fs.mkdirSync(outDir, { recursive: true })
const results = []
function check(name, actual, detail = '') { results.push({ name, ok: Boolean(actual), detail }); assert.ok(actual, name + ': ' + detail) }
async function waitPort() {
  const start = Date.now()
  while (Date.now() - start < 30000) {
    if (await new Promise(resolve => { const socket = net.connect(port, '127.0.0.1'); socket.once('connect', () => { socket.destroy(); resolve(true) }); socket.once('error', () => { socket.destroy(); resolve(false) }) })) return
    await new Promise(resolve => setTimeout(resolve, 100))
  }
  throw new Error('Vite 启动超时')
}
const engine = await pickEngine()
const server = spawn(path.join(uiDir, 'node_modules/.bin/vite'), ['--host', '127.0.0.1', '--port', String(port), '--strictPort'], { cwd: uiDir, stdio: 'ignore' })
let browser
try {
  await waitPort()
  browser = await engine.launch()
  for (const scenario of [
    { width: 1180, height: 760, theme: 'light' },
    { width: 1180, height: 760, theme: 'dark' },
    { width: 900, height: 600, theme: 'light' },
    { width: 900, height: 600, theme: 'dark' },
  ]) {
    const label = engine.name + '-' + scenario.width + 'x' + scenario.height + '-' + scenario.theme
    const page = await browser.newPage({ viewport: { width: scenario.width, height: scenario.height }, deviceScaleFactor: 1 })
    const errors = []
    page.on('pageerror', error => errors.push(error.message))
    await page.goto('http://127.0.0.1:' + port + '/?__audit=1&route=models&env=ready&runtime=running&effects='+effects+'&theme=' + scenario.theme)
    await page.waitForSelector('.channel-table tbody tr')
    await page.evaluate(async () => {
      const { modelApi } = await import('/src/features/models/api.ts')
      const { useModelsStore } = await import('/src/features/models/store.ts')
      const { useAppStore } = await import('/src/stores/app.ts')
      const { useNotificationsStore } = await import('/src/features/notifications/store.ts')
      const models = useModelsStore(), notifications = useNotificationsStore()
      useAppStore().clearUiFault();(await import('/src/app/feedback/store.ts')).useFeedbackStore().clearToast()
      notifications.list = []
      notifications.panelOpen = false
      window.__feedbackCalls = { writes: 0, restarts: [], clears: 0 }
      modelApi.writeFile = async (_expected, policy) => {
        window.__feedbackCalls.writes += 1
        return { changed: true, snapshot: { ...JSON.parse(JSON.stringify(models.snapshot)), policy: JSON.parse(JSON.stringify(policy)), policyDigest: 'mock-file-' + window.__feedbackCalls.writes, workspace: null } }
      }
      modelApi.refreshCache = async restart => {
        window.__feedbackCalls.restarts.push(restart)
        return { success: false, restarted: false, summary: '模拟官方返回：本次未执行重启。' }
      }
      notifications.clearAll = async () => { window.__feedbackCalls.clears += 1; notifications.list = [] }
    })
    async function save() {
      await page.locator('.channel-enable-cell button').first().click()
      await page.getByRole('button', { name: '保存', exact: true }).click()
      const preview = page.getByRole('dialog', { name: '保存预览', exact: true })
      await preview.getByRole('button', { name: '确认保存', exact: true }).click()
      await page.getByRole('button', { name: '立即重启 Codex', exact: true }).waitFor()
      await page.waitForFunction(() => document.activeElement?.textContent?.trim() === '立即重启 Codex')
    }
    await save()
    const saved = await page.evaluate(() => {
      const card = Array.from(document.querySelectorAll('.process-progress')).find(node => node.querySelector('h3')?.textContent === '已写入')
      const rect = card?.getBoundingClientRect()
      return { global: Boolean(card?.closest('.notif-layer')), count: document.querySelectorAll('.process-progress').length, oldCard: Boolean(document.querySelector('main .process-progress, .save-result')), focused: document.activeElement?.textContent?.trim(), inViewport: Boolean(rect && rect.x >= 0 && rect.right <= innerWidth && rect.y >= 0 && rect.bottom <= innerHeight) }
    })
    check(label + ' 确认保存后只有全局结果卡', saved.global && saved.count === 1 && !saved.oldCard, JSON.stringify(saved))
    check(label + ' 默认焦点和几何', saved.focused === '立即重启 Codex' && saved.inViewport)
    await page.screenshot({ path: path.join(outDir, label + '-saved.png') })
    await page.getByRole('button', { name: '稍后', exact: true }).click()
    await page.locator('.notification-btn').click()
    check(label + ' 稍后结束且铃铛不复活', await page.locator('.process-progress').count() === 0)
    await page.locator('.notification-btn').click()
    await save()
    await page.keyboard.press('Escape')
    check(label + ' Esc 结束保存反馈', await page.locator('.process-progress').count() === 0)
    await save()
    await page.getByRole('button', { name: '立即重启 Codex', exact: true }).click()
    const restart = page.getByRole('dialog', { name: '重启 Codex', exact: true })
    await restart.waitFor()
    check(label + ' 复用公共确认弹窗且尚未重启', await restart.evaluate(node => node.classList.contains('ui-dialog')) && await page.evaluate(() => window.__feedbackCalls.restarts.length === 0))
    await page.screenshot({ path: path.join(outDir, label + '-restart.png') })
    await page.keyboard.press('Escape')
    check(label + ' 确认弹窗 Esc 彻底结束反馈且不重启', await page.locator('.process-progress').count() === 0 && await page.evaluate(() => window.__feedbackCalls.restarts.length === 0))
    await save()
    await page.getByRole('button', { name: '立即重启 Codex', exact: true }).click()
    await restart.getByRole('button', { name: '确认重启', exact: true }).click()
    await page.waitForFunction(() => document.querySelector('.toast')?.textContent.includes('模拟官方返回：本次未执行重启。'))
    check(label + ' 如实报告失败并结束保存反馈', await page.locator('.process-progress').count() === 0 && await page.evaluate(() => JSON.stringify(window.__feedbackCalls.restarts) === '[true]'))
    const collapsedBefore = await page.evaluate(async () => {
      const { useOperationTasks } = await import('/src/app/feedback/tasks.ts')
      const { useNotificationsStore } = await import('/src/features/notifications/store.ts')
      const { useFeedbackStore } = await import('/src/app/feedback/store.ts')
      const tasks = useOperationTasks()
      useFeedbackStore().clearToast()
      for (let index = 0; index < 7; index++) tasks.apply({ id: crypto.randomUUID(), kind: 'research', phase: 'running' })
      useNotificationsStore().list = [
        { id: 'pending', kind: 'warning', category: 'system', title: '待处理示例', detail: '仅供通知交互验证', read: false, time: '' },
        { id: 'resolved', kind: 'info', category: 'system', title: '已处理示例', detail: '仅供通知交互验证', read: true, resolved: true, time: '' },
      ]
      return tasks.tasks.filter(task => task.collapsed).map(task => task.id)
    })
    await page.locator('.notification-btn').click()
    const collapsedAfter = await page.evaluate(async () => (await import('/src/app/feedback/tasks.ts')).useOperationTasks().tasks.filter(task => task.collapsed).map(task => task.id))
    check(label + ' 铃铛保持任务收起并在列表逐条展示', JSON.stringify(collapsedBefore) === JSON.stringify(collapsedAfter) && await page.locator('[data-operation-task]').count() === collapsedBefore.length && await page.locator('.process-progress').count() === 3)
    check(label + ' 没有第二展开入口', await page.locator('.task-reopen').count() === 0)
    await page.screenshot({ path: path.join(outDir, label + '-bell.png') })
    await page.locator('.notification-panel').getByRole('button', { name: '清理', exact: true }).click()
    const clear = page.getByRole('dialog', { name: '清理全部通知', exact: true })
    await clear.waitFor()
    check(label + ' 清理公共弹窗说明范围', (await clear.textContent()).includes('将清理全部 2 条通知（其中 1 条未处理）') && await page.locator('.notification-confirm').count() === 0)
    await clear.locator('p').click()
    check(label + ' 弹窗内部点击不意外关闭面板', await page.locator('.notification-panel.show').count() === 1 && await clear.isVisible())
    await page.screenshot({ path: path.join(outDir, label + '-clear.png') })
    await page.keyboard.press('Escape')
    check(label + ' 清理 Esc 不执行清理', await page.evaluate(() => window.__feedbackCalls.clears === 0) && await page.locator('dialog[open]').count() === 0)
    await page.locator('.notification-panel').getByRole('button', { name: '清理', exact: true }).click()
    await clear.getByRole('button', { name: '取消', exact: true }).click()
    check(label + ' 取消不清理且保留面板', await page.evaluate(() => window.__feedbackCalls.clears === 0) && await page.locator('.notification-panel.show').count() === 1)
    await page.locator('.notification-panel').getByRole('button', { name: '清理', exact: true }).click()
    await clear.getByRole('button', { name: '确认清理', exact: true }).click()
    await page.waitForFunction(() => window.__feedbackCalls.clears === 1)
    check(label + ' 确认后才清理且无异常', errors.length === 0, errors.join(' | '))
    await page.close()
  }
} catch (error) {
  if (results.at(-1)?.ok !== false) results.push({ name: 'browser execution', ok: false, detail: error.message })
  throw error
} finally {
  if (browser) await browser.close()
  server.kill('SIGTERM')
  fs.writeFileSync(path.join(outDir, 'report.json'), JSON.stringify({ engine: engine.name, adapter: 'mock file/restart/cleanup; real page clicks', results }, null, 2))
  console.log(JSON.stringify({ engine: engine.name, passed: results.filter(item => item.ok).length, failed: results.filter(item => !item.ok).length, outDir }))
}
