// Native caption/Mica state is supplied by DWM. It does not change user quality.
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useThemeStore } from './theme'

export interface NativeAppearance {
  platform: 'Windows'
  material: 'mica' | 'solid'
  theme: 'light' | 'dark'
  reason: string
}

export function applyNativeAppearance(value: NativeAppearance | null): void {
  if (!value || value.platform !== 'Windows') return
  document.documentElement.dataset.nativeMaterial = value.material
  const theme = useThemeStore()
  // The OS theme is authoritative only while the user's theme choice is system.
  if (theme.setting === 'system') {
    theme.resolved = value.theme
    document.documentElement.dataset.theme = value.theme
  }
}

export async function installNativeAppearance(): Promise<void> {
  if (!/^Win/.test(navigator.platform) || !('__TAURI_INTERNALS__' in window)) return
  let revision = 0
  try {
    await listen<NativeAppearance>('native-window-appearance', event => {
      revision++
      applyNativeAppearance(event.payload)
    })
    const before = revision
    const initial = await invoke<NativeAppearance | null>('window_appearance')
    if (before === revision) applyNativeAppearance(initial)
  } catch {
    // Native fallback retains readable product surfaces and system caption.
    document.documentElement.dataset.nativeMaterial = 'solid'
  }
}
