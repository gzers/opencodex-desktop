import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import {
  DEFAULT_GLOW_RENDER,
  applyGlowRenderAttribute,
  clampGlowRender,
  resolveGlowRender,
  useGlowRenderStore,
} from '@/app/appearance/glowRender'

const glowAttr = () => document.documentElement.getAttribute('data-glow-render')

describe('background glow renderer policy', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    document.documentElement.removeAttribute('data-glow-render')
  })

  it('defaults to the WEBGL mesh renderer', () => {
    expect(DEFAULT_GLOW_RENDER).toBe('mesh')
    expect(clampGlowRender(undefined)).toBe('mesh')
    expect(clampGlowRender('canvas2d')).toBe('mesh')
  })

  it('accepts the two frozen values only', () => {
    expect(clampGlowRender('mesh')).toBe('mesh')
    expect(clampGlowRender('css')).toBe('css')
  })

  it('falls back to the pure CSS aurora when WEBGL is unavailable', () => {
    expect(resolveGlowRender('mesh', true)).toBe('mesh')
    expect(resolveGlowRender('mesh', false)).toBe('css')
    // CSS 是显式选择：设备支持与否都不改变。
    expect(resolveGlowRender('css', true)).toBe('css')
    expect(resolveGlowRender('css', false)).toBe('css')
  })

  it('writes the resolved renderer to the root element', () => {
    const store = useGlowRenderStore()
    store.setWebglSupported(true)
    store.setSetting('mesh')
    expect(glowAttr()).toBe('mesh')

    store.setSetting('css')
    expect(glowAttr()).toBe('css')
  })

  it('reports the fallback state instead of pretending WEBGL is active', () => {
    const store = useGlowRenderStore()
    store.setSetting('mesh')
    store.setWebglSupported(false)
    expect(store.mode).toBe('css')
    expect(store.fellBack).toBe(true)
    expect(glowAttr()).toBe('css')

    store.setWebglSupported(true)
    expect(store.mode).toBe('mesh')
    expect(store.fellBack).toBe(false)
    expect(glowAttr()).toBe('mesh')
  })

  it('only writes through the single application point', () => {
    applyGlowRenderAttribute('css')
    expect(glowAttr()).toBe('css')
    applyGlowRenderAttribute('mesh', null)
    expect(glowAttr()).toBe('css')
  })
})
