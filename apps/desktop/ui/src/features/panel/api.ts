import type { PanelAction } from '@/types/ui'

export interface PanelBounds {
  x: number
  y: number
  width: number
  height: number
}

export interface PanelRequest {
  action: PanelAction
  bounds?: PanelBounds
  reload?: boolean
  scale?: number
  // 只驱动管理器注入的浮层主题；不改写官方页面自身样式。
  theme?: 'light' | 'dark'
  // 按官方面板自己的 `ocx-theme` 存储键同步明暗（light / dark / system）。
  themeSetting?: 'light' | 'dark' | 'system'
  effects?: 'high' | 'mid' | 'low'
  toast?: string
}

export interface PanelResult {
  visible: boolean
  panelUrl: string | null
}

export async function syncEmbeddedPanel(request: PanelRequest): Promise<PanelResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<PanelResult>('sync_embedded_panel', { request })
}
