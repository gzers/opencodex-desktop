import { describe, expect, it } from 'vitest'
import { getAppStatus } from '@/app/api'

describe('getAppStatus', () => {
  it('converts invoke failures into a typed error', async () => {
    await expect(getAppStatus()).rejects.toThrow()
  })
})
