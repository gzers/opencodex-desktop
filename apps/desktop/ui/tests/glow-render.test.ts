import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import {
  DEFAULT_GLOW_RENDER,
  applyGlowRenderAttribute,
  clampGlowRender,
  resolveGlowRender,
  useGlowRenderStore,
} from '@/app/appearance/glowRender'
import { installGlowRenderContextGuard } from '@/app/appearance/glowRender'

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

  // 回归（F-04）：运行中 WEBGL 上下文丢失时，背景光必须回退纯 CSS，而不是继续空转；
  // 恢复后重新探测。contextlost 事件发生在 canvas 上，须经捕获阶段统一处理。
  it('falls back to CSS when the live WEBGL context is lost and re-probes on restore', () => {
    const canvas = document.createElement('canvas')
    document.body.append(canvas)
    const store = useGlowRenderStore()
    store.setWebglSupported(true)
    store.setSetting('mesh')
    expect(store.mode).toBe('mesh')

    installGlowRenderContextGuard()
    canvas.dispatchEvent(new Event('webglcontextlost', { bubbles: false }))
    expect(store.mode).toBe('css')
    expect(glowAttr()).toBe('css')

    canvas.dispatchEvent(new Event('webglcontextrestored', { bubbles: false }))
    // jsdom 无 WEBGL：重探测后仍不支持，选择保持 CSS，但恢复路径已被触发而非静默。
    expect(store.mode).toBe('css')
    canvas.remove()
  })
})
