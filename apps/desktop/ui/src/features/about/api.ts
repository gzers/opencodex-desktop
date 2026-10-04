export interface AboutAppDto {
  name: string
  version: string
  identifier: string
  platform: string
  framework: string
  license: string
}

export interface OfficialProjectDto {
  displayName: string
  version: string | null
  rawVersion: string
  truncated: boolean
}

/** 官方远端最新版本只读查询结果（U-03）；失败抛错，不改写本地状态。 */
export interface OfficialRemoteLatestDto {
  tag: string
  version: string
  integrity: string | null
}

export async function getAppAbout(): Promise<AboutAppDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<AboutAppDto>('app_about')
}

export async function getOfficialProjectFacts(): Promise<OfficialProjectDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OfficialProjectDto>('official_project_facts')
}

export async function getOfficialRemoteLatest(): Promise<OfficialRemoteLatestDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OfficialRemoteLatestDto>('official_remote_latest')
}
