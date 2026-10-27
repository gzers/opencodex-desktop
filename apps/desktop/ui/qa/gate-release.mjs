// 0.2.7 发布候选门禁快照：一次跑完可自动化的检查并汇总。
//
// 每项独立执行、独立记录；任一失败不中断后续，最后统一给出结论。
// 需要外部条件的检查（真实官方保存读回、真机触控、双平台原生）不在此脚本内，
// 以免把“没跑”冒充为“通过”。
import { execFileSync } from 'child_process'
import fs from 'fs'
import os from 'os'
import path from 'path'
import { fileURLToPath } from 'url'

const here = path.dirname(fileURLToPath(import.meta.url))
const uiDir = path.resolve(here, '..')
const outDir = process.env.GATE_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-gate-'))
fs.mkdirSync(outDir, { recursive: true })

const steps = [
  { id: 'typecheck', cmd: ['npx', ['vue-tsc', '--noEmit']], needsEnv: false },
  { id: 'unit', cmd: ['npx', ['vitest', 'run']], needsEnv: false },
  { id: 'build', cmd: ['npx', ['vite', 'build']], needsEnv: false },
  // Confirm Save and reopen through the actual Rust file writer, using only an
  // isolated temporary configuration. No user file or desktop restart is touched.
  { id: 'audit:file-save', cmd: ['node', ['qa/file-save-browser.mjs']], needsEnv: false },
  { id: 'audit:notification-feedback', cmd: ['node', ['tests/notification-feedback.browser.mjs']], needsEnv: false },
  { id: 'audit:notification-parity', cmd: ['node', ['qa/notification-parity.mjs']], needsEnv: false },
  { id: 'audit:models', cmd: ['node', ['qa/model-visual.mjs']], needsEnv: false },
  { id: 'audit:preview', cmd: ['node', ['qa/preview-visual.mjs']], needsEnv: false },
  { id: 'audit:geometry', cmd: ['node', ['qa/geometry-visual.mjs']], needsEnv: false },
  // macOS 原生引擎族（WKWebView 用 WebKit）：Chromium 通过不等于原生窗口通过。
  { id: 'audit:geometry:webkit', cmd: ['node', ['qa/geometry-webkit.mjs']], needsEnv: false },
  // 其余四套实屏检查的 WebKit 复跑：同一批断言换个引擎，专抓只在 WebKit 出现的问题。
  { id: 'audit:webkit', cmd: ['npm', ['run', 'audit:webkit']], needsEnv: false },
  { id: 'audit:template', cmd: ['node', ['qa/template-visual.mjs']], needsEnv: false },
  { id: 'audit:export', cmd: ['node', ['qa/export-visual.mjs']], needsEnv: false },
  { id: 'audit:update', cmd: ['node', ['qa/update-visual.mjs']], needsEnv: false },
  // 原型并排对照：软件 UI 必须与原型一致（用户明确要求），四项各自独立。
  { id: 'audit:parity:list', cmd: ['node', ['qa/parity-list.mjs']], needsEnv: false },
  { id: 'audit:parity', cmd: ['node', ['qa/parity-preview.mjs']], needsEnv: false },
  { id: 'audit:parity:export', cmd: ['node', ['qa/parity-export.mjs']], needsEnv: false },
  { id: 'audit:parity:template', cmd: ['node', ['qa/parity-template.mjs']], needsEnv: false },
  { id: 'audit:parity:agent', cmd: ['node', ['qa/parity-agent.mjs']], needsEnv: false },
  { id: 'audit:catalog-naming', cmd: ['node', ['qa/catalog-naming-real.mjs']], needsEnv: false },
  {
    id: 'audit:catalog-crossvalidate',
    cmd: ['node', ['qa/catalog-naming-crossvalidate.mjs']],
    needsEnv: true,
    skipReason: '需要 OCX_OFFICIAL_ENTRY 指向官方 ocx.mjs',
  },
]

function summarize(output) {
  const lines = String(output).trim().split('\n')
  const tests = lines.find(line => /Tests\s+\d+/.test(line))
  const shots = lines.find(line => /failed=\d+\/\d+/.test(line))
  const checked = lines.find(line => /^CHECKED/.test(line))
  const pass = lines.find(line => /^PASS/.test(line))
  return (tests || shots || checked || pass || lines.at(-1) || '').trim().slice(0, 160)
}

const results = []
for (const step of steps) {
  const envMissing = step.needsEnv && !process.env.OCX_OFFICIAL_ENTRY
  if (envMissing) {
    results.push({ id: step.id, status: 'skipped', detail: step.skipReason })
    console.log('SKIP ' + step.id + ' · ' + step.skipReason)
    continue
  }
  const started = Date.now()
  try {
    const output = execFileSync(step.cmd[0], step.cmd[1], {
      cwd: uiDir,
      encoding: 'utf8',
      timeout: 15 * 60 * 1000,
      stdio: ['ignore', 'pipe', 'pipe'],
      // 默认 1MB 缓冲：前端测试输出（含 Vue 警告）会超过它，抛 ENOBUFS 让整项
      // 看起来像失败。这里放宽到 64MB；真正的失败仍由退出码判定。
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, AUDIT_OUT_DIR: path.join(outDir, step.id.replace(/[:]/g, '-')) },
    })
    const ms = Date.now() - started
    results.push({ id: step.id, status: 'passed', ms, detail: summarize(output) })
    console.log('PASS ' + step.id + ' · ' + summarize(output) + ' · ' + ms + 'ms')
  } catch (error) {
    const ms = Date.now() - started
    const output = String(error.stdout || '') + String(error.stderr || '')
    // 环境缺失（退出码 2 + BLOCKED）记为跳过，不冒充通过也不误判为缺陷。
    if (error.status === 2 && /^BLOCKED/m.test(output.trim())) {
      results.push({ id: step.id, status: 'skipped', detail: output.trim().split('\n')[0].slice(0, 160) })
      console.log('SKIP ' + step.id + ' · ' + output.trim().split('\n')[0].slice(0, 160))
      continue
    }
    results.push({ id: step.id, status: 'failed', ms, detail: summarize(output) || String(error.message).slice(0, 160) })
    console.log('FAIL ' + step.id + ' · ' + (summarize(output) || error.message))
  }
}

const failed = results.filter(r => r.status === 'failed')
const skipped = results.filter(r => r.status === 'skipped')
const report = {
  generatedAt: new Date().toISOString(),
  total: results.length,
  passed: results.filter(r => r.status === 'passed').length,
  failed: failed.length,
  skipped: skipped.length,
  results,
}
fs.writeFileSync(path.join(outDir, 'gate-report.json'), JSON.stringify(report, null, 2))
console.log('')
console.log('门禁快照：' + report.passed + ' 通过 / ' + report.failed + ' 失败 / ' + report.skipped + ' 跳过')
console.log('报告：' + path.join(outDir, 'gate-report.json'))
if (skipped.length) console.log('跳过项不计入通过：' + skipped.map(s => s.id).join('、'))
process.exit(failed.length ? 1 : 0)

