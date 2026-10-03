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

export async function getAppAbout(): Promise<AboutAppDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<AboutAppDto>('app_about')
}

export async function getOfficialProjectFacts(): Promise<OfficialProjectDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OfficialProjectDto>('official_project_facts')
}
