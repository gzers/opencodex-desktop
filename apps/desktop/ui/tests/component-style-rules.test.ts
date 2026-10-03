import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

// IMP-04 §7.10：公共组件不得新造颜色常量、任意字号/字重/层级；
// 数值应来自 tokens / 既有配方类。该检查同时覆盖 Vue SFC 的 scoped/普通样式块。
const root = path.resolve(__dirname, '../src/components')

function vueFilesUnder(dir: string): string[] {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) return vueFilesUnder(full)
    return entry.name.endsWith('.vue') ? [full] : []
  })
}

function styleBlocks(source: string): string[] {
  return [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map(match => match[1])
}

// 只检查公共组件目录（ui/layout/patterns），业务组件不在此约束内。
const publicDirs = ['ui', 'layout', 'patterns'].map(name => path.join(root, name))
const files = publicDirs.filter(dir => fs.existsSync(dir)).flatMap(vueFilesUnder)

describe('public component style rules', () => {
  it('存在待检查的公共组件', () => {
    expect(files.length).toBeGreaterThan(0)
  })

  it('公共组件样式不含硬编码颜色、字号、字重与层级常量', () => {
    const violations: string[] = []
    for (const file of files) {
      const rel = path.relative(root, file)
      for (const block of styleBlocks(fs.readFileSync(file, 'utf8'))) {
        const checks: Array<[string, RegExp]> = [
          ['hex-color', /#[0-9a-fA-F]{3,8}\b/],
          ['color-fn', /\b(?:rgba?|hsla?)\s*\(/],
          ['font-size', /font-size\s*:\s*[^;]*\d+px/],
          ['font-weight', /font-weight\s*:\s*\d{3}/],
          ['z-index', /z-index\s*:\s*\d+/],
        ]
        for (const [name, re] of checks) {
          if (re.test(block)) violations.push(`${rel}: ${name}`)
        }
      }
    }
    expect(violations).toEqual([])
  })
})
