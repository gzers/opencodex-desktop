import {describe,expect,it} from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
// @ts-expect-error QA scanner is intentionally usable by node without a TS build.
import {statusToneViolations} from '../qa/status-contract.mjs'

const src = path.resolve(__dirname, '../src')
function files(dir: string): string[] {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap(entry =>
    entry.isDirectory() ? files(path.join(dir, entry.name)) : entry.name.endsWith('.vue') ? [path.join(dir, entry.name)] : [],
  )
}

describe('状态展示门禁', () => {
  it('拒绝在 SFC 新写 data-tone 语义别名', () => {
    const bad = '<template><span class="tag" data-tone="warning">x</span></template>'
    expect(statusToneViolations(bad).map((v: {kind: string}) => v.kind)).toEqual(['data-tone-alias'])
  })

  it('拒绝直接渲染后端英文枚举', () => {
    const bad = '<template><span>{{ statusSnapshot.matrix.runtime }}</span></template>'
    expect(statusToneViolations(bad).map((v: {kind: string}) => v.kind)).toEqual(['raw-enum'])
  })

  it('放行检查判定类（overview/motion check-value 不是状态标签）', () => {
    const ok = '<template><span class="overview-check-value" :data-tone="check.tone">x</span></template>'
    expect(statusToneViolations(ok)).toEqual([])
  })

  it('实际源码零违规：状态统一后不允许回退', () => {
    const issues = ['routes', 'features', 'components', 'layout'].flatMap(dir => {
      const root = path.join(src, dir)
      return fs.existsSync(root) ? files(root) : []
    }).flatMap(file => statusToneViolations(fs.readFileSync(file, 'utf8'), path.relative(src, file)))
    expect(issues).toEqual([])
  })
})

