import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ThemeSetting } from '@/types/ui'

function systemTheme(): ThemeSetting {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export const useThemeStore = defineStore('theme', () => {
  const setting = ref<ThemeSetting>(readStored())
  const resolved = ref<ThemeSetting>(setting.value === 'system' ? systemTheme() : setting.value)

  function readStored(): ThemeSetting {
    try {
      const stored = window.localStorage.getItem('ocx-theme') as ThemeSetting | null
      return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system'
    } catch {
      return 'system'
    }
  }

  function apply(next: ThemeSetting) {
    setting.value = next
    resolved.value = next === 'system' ? systemTheme() : next
    try { window.localStorage.setItem('ocx-theme', setting.value) } catch {}
    document.documentElement.setAttribute('data-theme', resolved.value)
  }

  function syncSystem() {
    if (setting.value === 'system') apply('system')
  }

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', syncSystem)
  apply(setting.value)

  return { setting, resolved, apply }
})
