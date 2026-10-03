import { describe, expect, it } from 'vitest'
import { discoverEnvironment } from '@/features/environment/api'

describe('discoverEnvironment', () => {
  it('does not synthesize discovery state when Tauri is unavailable', async () => {
    await expect(discoverEnvironment()).rejects.toThrow()
  })
})
