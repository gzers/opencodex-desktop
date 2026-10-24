// 真实目录命名不变量（WP-64 / §29.1，2026-10-24）。
//
// 依据官方 2.84.0 实现：Codex 目录的 display_name 走 routedDisplayName，等于 slug
// （providerId/模型ID）；modelDisplayNames 明文禁止斜杠，不得用于渠道模型主名称。
// 本脚本只读本机真实 .codex/opencodex-catalog.json 与管理器 config.json，核验：
//   1. 所有渠道行 display_name === slug；
//   2. slug 前缀是该渠道的真实 providerId；
//   3. 不存在把友好名称写进渠道行的条目。
// 不写入任何文件；文件不存在时以 BLOCKED 退出，不当作通过。
import fs from 'fs'
import os from 'os'
import path from 'path'

function fail(message) {
  console.log('FAIL ' + message)
  process.exitCode = 1
}
const catalogPath = process.argv[2] || path.join(os.homedir(), '.codex', 'opencodex-catalog.json')
const configPath = process.argv[3] || path.join(os.homedir(), 'Library', 'Application Support', 'com.gzers.opencodex.desktop', 'opencodex-home', 'config.json')
if (!fs.existsSync(catalogPath) || !fs.existsSync(configPath)) {
  console.log('BLOCKED: 缺少真实目录或配置，未执行检查')
  console.log('  catalog: ' + catalogPath)
  console.log('  config:  ' + configPath)
  process.exit(2)
}
const catalog = JSON.parse(fs.readFileSync(catalogPath, 'utf8'))
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'))
const models = Array.isArray(catalog.models) ? catalog.models : []
const providerIds = new Set(Object.keys(config.providers || {}))
const routed = models.filter(model => typeof model.slug === 'string' && model.slug.includes('/'))

let checked = 0
for (const model of routed) {
  const slug = String(model.slug)
  const providerId = slug.slice(0, slug.indexOf('/'))
  const modelId = slug.slice(slug.indexOf('/') + 1)
  checked += 1
  if (model.display_name !== slug) fail(slug + ' display_name=' + JSON.stringify(model.display_name) + ' 应等于 slug')
  if (!providerIds.has(providerId)) fail(slug + ' 前缀不是真实 providerId')
  if (!modelId) fail(slug + ' 缺少原始模型ID')
}
if (!checked) fail('目录没有渠道行，无法核验（不代表通过）')
console.log('CHECKED ' + checked + ' routed rows; providers=' + providerIds.size)
if (!process.exitCode) console.log('PASS 目录渠道行统一为 providerId/原始模型ID')

