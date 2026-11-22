import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  detectWebglSupport,
  installGlowRenderContextGuard,
  useGlowRenderStore,
} from '@/app/appearance/glowRender'
import { createMeshGlow } from '@/features/runtime/motion/meshGlow'

beforeEach(() => {
  setActivePinia(createPinia())
  vi.stubGlobal('requestAnimationFrame', () => 1)
  vi.stubGlobal('cancelAnimationFrame', () => {})
})
afterEach(() => {
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
  document.body.replaceChildren()
  document.documentElement.removeAttribute('data-glow-render')
})

describe('WebGL context ownership', () => {
  it('releases the capability probe immediately instead of retaining a second GPU context', () => {
    const loseContext = vi.fn()
    const getExtension = vi.fn(() => ({ loseContext }))
    let probe: HTMLCanvasElement | undefined
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (this: HTMLCanvasElement) {
      probe = this
      return { getExtension } as unknown as WebGLRenderingContext
    })
    expect(detectWebglSupport()).toBe(true)
    expect([probe!.width, probe!.height]).toEqual([1, 1])
    expect(getExtension).toHaveBeenCalledWith('WEBGL_lose_context')
    expect(loseContext).toHaveBeenCalledTimes(1)
    expect(probe!.isConnected).toBe(false)
  })

  it('keeps mesh capability when an owned canvas is deliberately disposed', () => {
    const gl = Object.fromEntries([
      'createShader', 'shaderSource', 'compileShader', 'getShaderParameter', 'deleteShader',
      'createProgram', 'attachShader', 'linkProgram', 'getProgramParameter', 'deleteProgram',
      'useProgram', 'createBuffer', 'bindBuffer', 'bufferData', 'getAttribLocation',
      'enableVertexAttribArray', 'vertexAttribPointer', 'getUniformLocation',
      'uniform3fv', 'uniform1f', 'uniform2f', 'viewport', 'deleteBuffer', 'getExtension',
    ].map(name => [name, vi.fn()]))
    gl.createShader.mockReturnValue({})
    gl.createProgram.mockReturnValue({})
    gl.createBuffer.mockReturnValue({})
    gl.getShaderParameter.mockReturnValue(true)
    gl.getProgramParameter.mockReturnValue(true)
    let canvas: HTMLCanvasElement
    const loseContext = vi.fn(() => {
      canvas.dispatchEvent(new Event('webglcontextlost', { bubbles: false }))
    })
    gl.getExtension.mockReturnValue({ loseContext })
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (this: HTMLCanvasElement) {
      canvas = this
      return gl as unknown as WebGLRenderingContext
    })
    const host = document.createElement('div')
    document.body.append(host)
    vi.spyOn(host, 'getBoundingClientRect').mockReturnValue({ width: 889, height: 840 } as DOMRect)
    const store = useGlowRenderStore()
    store.setWebglSupported(true)
    installGlowRenderContextGuard()
    const mesh = createMeshGlow(host, { getDark: () => false })!
    mesh.destroy()
    expect(loseContext).toHaveBeenCalledTimes(1)
    expect(host.children).toHaveLength(0)
    expect(store.mode).toBe('mesh')
    expect(store.fellBack).toBe(false)

    // A fault on a different, still-live context must retain the existing fallback.
    const liveCanvas = document.createElement('canvas')
    document.body.append(liveCanvas)
    liveCanvas.dispatchEvent(new Event('webglcontextlost', { bubbles: false }))
    expect(store.mode).toBe('css')
    expect(store.fellBack).toBe(true)
  })
})
