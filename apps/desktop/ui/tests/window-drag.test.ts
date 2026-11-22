import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

// 窗口拖拽靠 macOS（titleBarStyle: Overlay）下的 .titlebar + data-tauri-drag-region：
// Tauri 注入脚本只在 mousedown 命中该元素时才调 startDragging，所以属性与可命中缺一不可。
// 用户报告「窗口不能拖拽」（2026-09-24），这里做代码级回归。
const app = readFileSync(resolve(__dirname, '../src/App.vue'), 'utf8')
const css = readFileSync(resolve(__dirname, '../src/styles/base.css'), 'utf8')

describe('窗口拖拽区', () => {
  it('标题栏带 data-tauri-drag-region', () => {
    const m = app.match(/<header[^>]*class="titlebar"[^>]*>/)
    expect(m, 'App.vue 里应存在 .titlebar 头部').not.toBeNull()
    expect(m![0]).toContain('data-tauri-drag-region')
    expect(m![0]).toContain('v-if="!isWindows"')
  })

  it('标题栏可命中（不能 pointer-events:none）', () => {
    const block = css.match(/\.titlebar \{([^}]*)\}/)
    expect(block, 'base.css 里应有 .titlebar 规则').not.toBeNull()
    expect(block![1]).not.toMatch(/pointer-events:\s*none/)
  })

  it('拖拽区高度只随 macOS 打开，其它平台为 0（走原生标题栏）', () => {
    expect(css).toMatch(/--window-controls-height:\s*0px/)
    expect(css).toMatch(/\.app-window\.is-macos[\s\S]{0,120}--window-controls-height:\s*calc\(28px/)
  })

  it('capability 授权 core:window:allow-start-dragging', () => {
    // Tauri v2 的 core:window:default 只给只读 getter + internal-toggle-maximize，
    // 不含 start-dragging；少了这条授权，drag region 命中后 invoke 会被 ACL 拒绝、窗口纹丝不动。
    const cap = JSON.parse(
      readFileSync(resolve(__dirname, '../../tauri/capabilities/default.json'), 'utf8'),
    ) as { windows: string[]; permissions: string[] }
    expect(cap.windows).toContain('main')
    expect(cap.permissions).toContain('core:window:allow-start-dragging')
  })
})
