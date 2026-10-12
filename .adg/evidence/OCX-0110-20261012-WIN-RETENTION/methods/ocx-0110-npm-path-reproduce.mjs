import { createRequire } from 'node:module'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'

const root = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop\\test\\runtime\\windows0110\\npm-path-repro'
await mkdir(root, { recursive: true })
const npm = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop\\test\\runtime\\windows0110\\data\\home\\.local\\bin\\node_modules\\npm'
const require = createRequire(npm + '\\package.json')
const realpath = require(npm + '\\node_modules\\@npmcli\\arborist\\lib\\realpath.js')
const rows = []
for (const value of [path.toNamespacedPath(root), root]) {
  try {
    const resolved = await realpath(value, new Map(), new Map(), 0)
    rows.push({ path: value, result: 'success', resolved })
  } catch (error) { rows.push({ path: value, result: 'failure', error: String(error) }) }
}
if (rows[0].result !== 'failure' || rows[1].result !== 'success') {
  throw Error('Expected verbatim-path failure and ordinary-path success did not reproduce')
}
const report = { scope: 'same npm Arborist function, same real directory, no network or installation',
  npmVersion: require(npm + '\\package.json').version, nodeVersion: process.version, rows }
await writeFile('D:\\Users\\15119\\WorkSpace\\Code\\tools\\opencodex-desktop\\.adg\\evidence\\OCX-0110-20261012-WIN-RETENTION\\npm-path-red-green.json', JSON.stringify(report, null, 2))
console.log(JSON.stringify(report))
