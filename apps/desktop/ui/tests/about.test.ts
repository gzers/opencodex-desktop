import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { getAppAbout, getOfficialProjectFacts, type AboutAppDto, type OfficialProjectDto } from '@/features/about/api'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

const appDto: AboutAppDto = {
  name: 'OpenCodeX-Desktop',
  version: '0.1.0',
  identifier: 'com.gzers.opencodex.desktop',
  platform: 'macOS',
  framework: 'Tauri v2',
  license: 'MIT License',
}

const officialDto: OfficialProjectDto = {
  displayName: 'OpenCodex',
  version: '2.50.0',
  rawVersion: 'OpenCodex 2.50.0',
  truncated: false,
}

describe('about contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('invokes frozen about commands', async () => {
    invoke.mockResolvedValueOnce(appDto).mockResolvedValueOnce(officialDto)
    await expect(getAppAbout()).resolves.toEqual(appDto)
    await expect(getOfficialProjectFacts()).resolves.toEqual(officialDto)
    expect(invoke).toHaveBeenNthCalledWith(1, 'app_about')
    expect(invoke).toHaveBeenNthCalledWith(2, 'official_project_facts')
  })

  it('loads about metadata into the shared store', async () => {
    invoke.mockResolvedValueOnce(appDto).mockResolvedValueOnce(officialDto)
    const app = useAppStore()
    await app.loadAboutApp()
    await app.loadOfficialProject()
    expect(app.aboutApp).toEqual(appDto)
    expect(app.officialProject).toEqual(officialDto)
    expect(app.officialProjectError).toBe(false)
    expect(app.officialProjectLoading).toBe(false)
  })

  it('preserves the previous official facts when IPC fails', async () => {
    invoke.mockResolvedValueOnce(appDto).mockResolvedValueOnce(officialDto)
    const app = useAppStore()
    await app.loadAboutApp()
    await app.loadOfficialProject()
    invoke.mockRejectedValueOnce(new Error('backend unavailable'))
    await app.loadOfficialProject()
    expect(app.officialProject).toEqual(officialDto)
    expect(app.officialProjectError).toBe(true)
    expect(app.officialProjectLoading).toBe(false)
  })
})
