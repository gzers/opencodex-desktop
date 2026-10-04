export type TrayAction = 'start' | 'stop' | 'restart' | 'open_main' | 'open_panel' | 'open_logs' | 'open_data_dir' | 'run_doctor' | 'open_settings'

export interface TrayStateDto {
  runtimeLabel: string
  address: string | null
  canStart: boolean
  canStop: boolean
  canRestart: boolean
}

export async function trayState(): Promise<TrayStateDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<TrayStateDto>('tray_state')
}

export async function drainTrayRequests(): Promise<TrayAction[]> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<TrayAction[]>('drain_tray_requests')
}

export async function onTrayRequestsAvailable(handler: () => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event')
  return listen('tray-requests-available', handler)
}
