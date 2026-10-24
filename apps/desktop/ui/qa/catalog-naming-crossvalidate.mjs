// 写入输出与官方校验的交叉验证（WP-64 / §29.1，2026-10-24）。
//
// 管理器保存时会清除渠道的 modelDisplayNames（不使用友好名称）。本脚本验证这一步
// 产出的配置能通过官方 2.84.0 的 'ocx config validate'：先把真实配置复制到临时目录、
// 只删除 modelDisplayNames，再交给官方 CLI 校验。真实配置不被修改。
//
// 用法：OCX_OFFICIAL_ENTRY=<ocx.mjs> node qa/catalog-naming-crossvalidate.mjs
// 缺少官方入口或真实配置时以 BLOCKED 退出，不当作通过。
import { execFileSync } from 'child_process'
import fs from 'fs'
import os from 'os'
import path from 'path'

const entry = process.env.OCX_OFFICIAL_ENTRY || ''
const node = process.env.OCX_OFFICIAL_NODE || process.execPath
const source = process.argv[2] || path.join(os.homedir(), 'Library', 'Application Support', 'com.gzers.opencodex.desktop', 'opencodex-home', 'config.json')
if (!entry || !fs.existsSync(entry) || !fs.existsSync(source)) {
  console.log('BLOCKED: 缺少官方 ocx 入口或真实配置')
  console.log('  OCX_OFFICIAL_ENTRY=' + (entry || '(未设置)'))
  console.log('  config=' + source)
  process.exit(2)
}

const config = JSON.parse(fs.readFileSync(source, 'utf8'))
let cleared = 0
for (const provider of Object.values(config.providers || {})) {
  if (provider && Object.prototype.hasOwnProperty.call(provider, 'modelDisplayNames')) {
    delete provider.modelDisplayNames
    cleared += 1
  }
}
const work = fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-crossvalidate-'))
const home = path.join(work, 'home')
fs.mkdirSync(home, { recursive: true })
fs.writeFileSync(path.join(home, 'config.json'), JSON.stringify(config))

let output = ''
let code = 0
try {
  output = execFileSync(node, [entry, 'config', 'validate'], {
    env: { ...process.env, OPENCODEX_HOME: home },
    encoding: 'utf8',
    timeout: 120000,
  })
} catch (error) {
  code = error.status ?? 1
  output = String(error.stdout || '') + String(error.stderr || '')
} finally {
  fs.rmSync(work, { recursive: true, force: true })
}

console.log('cleared modelDisplayNames from ' + cleared + ' providers')
console.log('official validate exit=' + code)
console.log(output.trim().split('\n').slice(-3).join('\n'))
if (code !== 0) { console.log('FAIL 清除友好名后的配置未通过官方校验'); process.exitCode = 1 }
else console.log('PASS 清除友好名后的配置通过官方 config validate')

