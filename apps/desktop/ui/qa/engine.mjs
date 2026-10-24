/**
 * 实屏检查的引擎选择器。
 *
 * 为什么需要：本项目的桌面端在 macOS 用 WKWebView、Windows 用 WebView2。
 * 只跑 Chromium 会漏掉只在 WebKit 出现的问题——0.2.7 的「拖拽时框选文字 + drop 不触发」
 * 就是这样漏过去的（Chromium 100% 通过，WebKit 约 1/3 轮次失败）。
 *
 * 用法：
 *   AUDIT_ENGINE=webkit node qa/xxx-visual.mjs   // 用 WebKit 跑同一批断言
 *   node qa/xxx-visual.mjs                        // 默认 Chromium
 * 缺少对应引擎构建时抛 Blocked（退出码 2），由调用方记为「跳过」而不是失败。
 */
import fs from 'fs'
import os from 'os'
import path from 'path'
import { createRequire } from 'module'

const WANTED = (process.env.AUDIT_ENGINE || 'chromium').toLowerCase()

function playwrightCandidates() {
  const dirs = []
  const npx = path.join(os.homedir(), '.npm', '_npx')
  if (fs.existsSync(npx)) {
    for (const d of fs.readdirSync(npx)) dirs.push(path.join(npx, d, 'node_modules', 'playwright-core'))
  }
  return dirs.filter(c => fs.existsSync(path.join(c, 'package.json')))
}

function chromiumShell() {
  const base = path.join(os.homedir(), 'Library', 'Caches', 'ms-playwright')
  if (!fs.existsSync(base)) return null
  for (const d of fs.readdirSync(base).filter(x => x.startsWith('chromium_headless_shell-')).sort().reverse()) {
    const p = path.join(base, d, 'chrome-headless-shell-mac-arm64', 'chrome-headless-shell')
    if (fs.existsSync(p)) return p
  }
  return null
}

export class Blocked extends Error {}

/**
 * 返回 { name, module, launch } —— launch() 已带好该引擎的启动参数。
 */
export async function pickEngine() {
  if (!['chromium', 'webkit'].includes(WANTED)) throw new Blocked('未知 AUDIT_ENGINE: ' + WANTED)
  for (const c of playwrightCandidates()) {
    try {
      const mod = createRequire(c + '/')('playwright-core')
      if (!mod[WANTED]) continue
      if (WANTED === 'chromium') {
        const exe = chromiumShell()
        if (!exe) throw new Blocked('缺少 chromium headless shell')
        const browser = await mod.chromium.launch({ executablePath: exe })
        await browser.close()
        return { name: 'chromium', module: mod, launch: () => mod.chromium.launch({ executablePath: exe }) }
      }
      const browser = await mod.webkit.launch()
      await browser.close()
      return { name: 'webkit', module: mod, launch: () => mod.webkit.launch() }
    } catch (error) {
      if (error instanceof Blocked) throw error
      // 该 playwright 版本自带的引擎不可用，继续试下一个。
    }
  }
  throw new Blocked('本机没有可用的 ' + WANTED + ' 引擎')
}

export function engineLabel(name) {
  return name === 'webkit' ? 'WK' : 'CR'
}

