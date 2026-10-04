import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { createMeshGlow } from '@/features/runtime/motion/meshGlow'
let pending: Map<number, FrameRequestCallback>, nextId: number
let gl: Record<string, ReturnType<typeof vi.fn>>
beforeEach(() => {
  pending = new Map(); nextId = 0
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { pending.set(++nextId, cb); return nextId })
  vi.stubGlobal('cancelAnimationFrame', (id: number) => pending.delete(id))
  gl = Object.fromEntries(['createShader', 'shaderSource', 'compileShader', 'getShaderParameter', 'deleteShader', 'createProgram', 'attachShader', 'linkProgram', 'getProgramParameter', 'deleteProgram', 'useProgram', 'createBuffer', 'bindBuffer', 'bufferData', 'getAttribLocation', 'enableVertexAttribArray', 'vertexAttribPointer', 'getUniformLocation', 'uniform3fv', 'uniform1f', 'uniform2f', 'viewport', 'drawArrays', 'deleteBuffer', 'getExtension'].map(name => [name, vi.fn()]))
  gl.createShader.mockReturnValue({}); gl.createProgram.mockReturnValue({}); gl.createBuffer.mockReturnValue({})
  gl.getShaderParameter.mockReturnValue(true); gl.getProgramParameter.mockReturnValue(true)
  gl.getUniformLocation.mockImplementation((_, name) => name)
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => gl as unknown as WebGLRenderingContext)
})
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals() })
function frame(time: number) { const cbs = [...pending.values()]; pending.clear(); cbs.forEach(cb => cb(time)) }
function time() { return gl.uniform1f.mock.calls.filter(call => call[0] === 'u_time').at(-1)?.[1] }

describe('WebGL rendering lifetime', () => {
  it('retains full canvas resolution while avoiding repeated uniforms and background frames', () => {
    const host = document.createElement('div')
    vi.spyOn(host, 'getBoundingClientRect').mockReturnValue({ width: 889, height: 660 } as DOMRect)
    vi.stubGlobal('devicePixelRatio', 2)
    const mesh = createMeshGlow(host, { getDark: () => false })!
    expect(host.querySelector('canvas')!.width).toBe(Math.round(889 * 1.6))
    mesh.setColors(['rgb(1,2,3)', 'rgb(4,5,6)', 'rgb(7,8,9)'])
    const uploads = gl.uniform3fv.mock.calls.length
    mesh.setColors(['rgb(1,2,3)', 'rgb(4,5,6)', 'rgb(7,8,9)'])
    frame(1000); frame(1016); frame(1032)
    expect(time()).toBeCloseTo(0.032)
    expect(gl.uniform3fv).toHaveBeenCalledTimes(uploads)
    expect(gl.uniform2f).toHaveBeenCalledTimes(1)
    mesh.setActive(false); expect(pending.size).toBe(0)
    const draws = gl.drawArrays.mock.calls.length
    frame(301_032); expect(gl.drawArrays).toHaveBeenCalledTimes(draws)
    mesh.setActive(true); frame(301_048)
    expect(time()).toBeCloseTo(0.032)
    frame(301_064); expect(time()).toBeCloseTo(0.048)
    mesh.setReduced(true); frame(301_080); expect(pending.size).toBe(0)
    mesh.destroy(); mesh.setActive(true)
    expect(pending.size).toBe(0)
    expect(gl.deleteBuffer).toHaveBeenCalledTimes(1)
    expect(gl.deleteProgram).toHaveBeenCalledTimes(1)
  })

  it('releases resources if program linking fails', () => {
    gl.getProgramParameter.mockReturnValue(false)
    const host = document.createElement('div')
    expect(createMeshGlow(host, { getDark: () => false })).toBeNull()
    expect(host.children.length).toBe(0)
    expect(gl.deleteShader).toHaveBeenCalledTimes(2)
    expect(gl.deleteProgram).toHaveBeenCalledTimes(1)
    expect(pending.size).toBe(0)
  })
})
