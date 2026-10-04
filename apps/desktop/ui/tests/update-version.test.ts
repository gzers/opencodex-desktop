import { describe, expect, it } from 'vitest'
import { compareVersions, hasNewerVersion } from '@/features/updates/version'

describe('official version comparison (U-03)', () => {
  it('ranks newer core versions above older ones', () => {
    expect(compareVersions('2.66.0', '2.50.0')).toBeGreaterThan(0)
    expect(compareVersions('2.50.0', '2.66.0')).toBeLessThan(0)
    expect(compareVersions('v2.50.0', '2.50.0')).toBe(0)
    expect(compareVersions('2.50.1', '2.50.0')).toBeGreaterThan(0)
    expect(compareVersions('2.5', '2.50.0')).toBeLessThan(0)
  })

  it('treats a prerelease as older than the same stable version', () => {
    expect(compareVersions('2.50.0-beta.1', '2.50.0')).toBeLessThan(0)
    expect(compareVersions('2.50.0', '2.50.0-beta.1')).toBeGreaterThan(0)
  })

  it('returns null when a side is missing or unparseable instead of guessing', () => {
    expect(hasNewerVersion(null, '2.66.0')).toBeNull()
    expect(hasNewerVersion('2.50.0', null)).toBeNull()
    expect(hasNewerVersion('latest', '2.66.0')).toBeNull()
  })

  it('reports availability only when the remote is strictly newer', () => {
    expect(hasNewerVersion('2.50.0', '2.66.0')).toBe(true)
    expect(hasNewerVersion('2.66.0', '2.66.0')).toBe(false)
    expect(hasNewerVersion('2.66.0', '2.50.0')).toBe(false)
  })
})
