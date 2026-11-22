import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

// 用户报告（2026-09-25）：概览摘要行里真实数据根路径把「数据目录」挤成「数据目」/「录」。
// 口径（UI规范 §16 / §19）：按钮类永不换行；短 label 不换行；长值在自己那一行里截断。
const css = readFileSync(resolve(__dirname, '../src/styles/base.css'), 'utf8')

function rule(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escaped}\\s*\\{([^}]*)\\}`))
  expect(match, `base.css 应有 ${selector} 规则`).not.toBeNull()
  return match![1]
}

describe('短文本不断行', () => {
  it('概览摘要事实（含 label）不换行', () => {
    expect(rule('.ovc-fact')).toMatch(/white-space:\s*nowrap/)
  })

  it('概览长路径在自己那一行里截断，不挤走 label', () => {
    const mono = rule('.ovc-fact.mono')
    expect(mono).toMatch(/min-width:\s*0/)
    expect(mono).toMatch(/max-width:\s*100%/)
    const value = rule('.ovc-fact.mono b')
    expect(value).toMatch(/overflow:\s*hidden/)
    expect(value).toMatch(/text-overflow:\s*ellipsis/)
    expect(value).toMatch(/min-width:\s*0/)
  })

  it('按钮与短控件保持永不换行（§16 底线）', () => {
    expect(css).toMatch(/\.btn[^{]*\{[^}]*white-space:\s*nowrap/)
    expect(css).toMatch(/\.tag[^{]*\{[^}]*white-space:\s*nowrap/)
    expect(css).toMatch(/\.state-pill[^{]*\{[^}]*white-space:\s*nowrap/)
  })
})
