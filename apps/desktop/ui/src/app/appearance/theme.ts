import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ThemeSetting } from '@/types/ui'

// 主题事实源在偏好（`appearance.theme`）；这里只保留首屏缓存与即时投影。
// 启动早于后端偏好返回，先用 localStorage 的缓存渲染，随后由偏好回读覆盖；
// 偏好一旦有值就不再被旧缓存反向覆盖（H-15）。官方面板桥接仍用同一协议 key。
const THEME_CACHE_KEY = 'ocx-theme'

function prefersReduced(): MediaQueryList | null {
  try {
    return typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : null
  } catch {
    return null
  }
}

function systemTheme(): ThemeSetting {
  return prefersReduced()?.matches ? 'dark' : 'light'
}

function normalizeChoice(value: unknown): ThemeSetting {
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system'
}

export function readThemeCache(): ThemeSetting {
  try {
    return normalizeChoice(window.localStorage.getItem(THEME_CACHE_KEY))
  } catch {
    return 'system'
  }
}

/// 本机是否曾经写过主题缓存（用于一次性历史导入判定，不臆造默认值）。
export function hasStoredThemeCache(): boolean {
  try {
    return window.localStorage.getItem(THEME_CACHE_KEY) !== null
  } catch {
    return false
  }
}

export function writeThemeCache(value: ThemeSetting) {
  try { window.localStorage.setItem(THEME_CACHE_KEY, value) } catch {}
}

export const useThemeStore = defineStore('theme', () => {
  const setting = ref<ThemeSetting>(readThemeCache())
  const resolved = ref<ThemeSetting>(setting.value === 'system' ? systemTheme() : setting.value)

  function apply(next: ThemeSetting, options: { persistCache?: boolean } = {}) {
    const choice = normalizeChoice(next)
    const changed = setting.value !== choice
    setting.value = choice
    resolved.value = choice === 'system' ? systemTheme() : choice
    if (options.persistCache !== false) writeThemeCache(choice)
    document.documentElement.setAttribute('data-theme', resolved.value)
    return changed
  }

  function syncSystem() {
    if (setting.value === 'system') apply('system', { persistCache: false })
  }

  prefersReduced()?.addEventListener?.('change', syncSystem)
  apply(setting.value, { persistCache: false })

  return { setting, resolved, apply, syncSystem }
})
