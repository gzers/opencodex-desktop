import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

// 概览页批注结论（UI规范 §21）：软件底层不透明，透明的是「内容页」这一层。
// 生产只上「中透」一档；页面透上来的是软件自己的底 + 环境光，与桌面/壁纸无关。
const base = readFileSync(resolve(__dirname, '../src/styles/base.css'), 'utf8')
const tokens = readFileSync(resolve(__dirname, '../src/styles/tokens.css'), 'utf8')
const tauriConf = readFileSync(resolve(__dirname, '../../tauri/tauri.conf.json'), 'utf8')

function rule(selector: string, css = base): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  // 必须从行首开始，否则 `.main` 会先命中 `.app-shell > .main { grid-row: 2 }`。
  const match = css.match(new RegExp(`(?:^|\\n)${escaped}\\s*\\{([^}]*)\\}`))
  expect(match, `base.css 应有 ${selector} 规则`).not.toBeNull()
  return match![1]
}

/** 取亮色段（`:root` 到 `html[data-theme="dark"]` 之前）与暗色段。 */
function themeBlocks(css: string): { light: string; dark: string } {
  const split = css.indexOf('html[data-theme="dark"]')
  expect(split, 'tokens.css 应有暗色段').toBeGreaterThan(0)
  return { light: css.slice(0, split), dark: css.slice(split) }
}

describe('内容页表面材质（半透明磨砂）', () => {
  it('内容页用半透明填充，不再是实底 --panel', () => {
    const main = rule('.main')
    expect(main).toContain('background: var(--page-fill)')
    expect(main).not.toMatch(/background:\s*var\(--panel\)/)
  })

  // 真机实测：WKWebView 里 .main 上的 backdrop-filter 会去采样窗口背后的**桌面**
  // （页面颜色跟着窗口位置变，而外壳不变），那就违反了「参照物在软件内部」。
  it('内容页不得用 backdrop-filter（会把桌面采样进来）', () => {
    expect(rule('.main')).not.toContain('backdrop-filter')
    const { light } = themeBlocks(tokens)
    expect(light).not.toContain('--page-blur')
  })

  it('外壳保持不透明：底面仍是 --glass-rail，且不带 backdrop-filter', () => {
    for (const selector of ['.app-window', '.side', '.titlebar']) {
      const block = rule(selector)
      // 允许显式 `none`（表示「这一层不模糊」），但不允许真的挂上页面那档模糊。
      expect(block, `${selector} 不应带页面那层模糊`).not.toMatch(/backdrop-filter:\s*var\(--page-blur\)/)
      expect(block, `${selector} 不应重新变成半透明页面材质`).not.toContain('var(--page-fill)')
    }
    expect(rule('.app-window')).toContain('background: var(--glass-rail)')
  })

  it('「中透」= alpha .40，亮/暗两套都有定义', () => {
    const { light, dark } = themeBlocks(tokens)
    expect(light).toMatch(/--page-fill:\s*rgba\(255,\s*255,\s*255,\s*\.40\)/)
    expect(dark).toMatch(/--page-fill:\s*rgba\(38,\s*38,\s*38,\s*\.40\)/)
  })

  it('不做系统级窗口透明：配置里不得出现 transparent / macOSPrivateApi', () => {
    expect(tauriConf).not.toMatch(/"transparent"\s*:/)
    expect(tauriConf).not.toMatch(/macOSPrivateApi/)
  })

  it('页面透出的「柔和感」来自窗口那层环境光自身的模糊', () => {
    const glow = base.slice(base.indexOf('.app-window::before'))
    expect(glow.slice(0, glow.indexOf('}'))).toMatch(/filter:\s*blur\(/)
  })
})
