import { createPinia, setActivePinia } from 'pinia'
import { expect, it, vi } from 'vitest'

it('does not subscribe to system reduced-motion and keeps the selected tier', async () => {
  vi.resetModules()
  setActivePinia(createPinia())
  const media = vi.fn((_query: string) => ({ matches: true, addEventListener: vi.fn() }))
  vi.stubGlobal('matchMedia', media)
  const { installEffectsRuntime, useEffectsStore } = await import('@/app/appearance/effects')
  installEffectsRuntime()
  useEffectsStore().setSetting('high')
  expect(media.mock.calls.some(([query]) => query.includes('prefers-reduced-motion'))).toBe(false)
  expect(document.documentElement.dataset.effects).toBe('high')
  vi.unstubAllGlobals()
})
