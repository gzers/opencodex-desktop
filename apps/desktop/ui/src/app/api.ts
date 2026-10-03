export type AppMode = 'shell'

export interface AppStatus {
  mode: AppMode
  ready: boolean
}

export async function getAppStatus(): Promise<AppStatus> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<AppStatus>('app_status')
}
