import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import { useEffectsStore } from '@/app/appearance/effects'
import { useGlowRenderStore } from '@/app/appearance/glowRender'

const layouts = [
  { name: 'standard ready', logoTop: 98, logoSize: 140 },
  { name: 'standard setup', logoTop: 87, logoSize: 90 },
  { name: 'compact ready', logoTop: 69, logoSize: 100 },
  { name: 'compact setup', logoTop: 83, logoSize: 74 },
]
let layout: typeof layouts[number]
let gl: Record<string, ReturnType<typeof vi.fn>>
let pending: Map<number, FrameRequestCallback>
let rectReads: ReturnType<typeof vi.fn<() => DOMRect>>
let observers: { targets: Set<Element>; callback: ResizeObserverCallback; disconnect: ReturnType<typeof vi.fn> }[]

beforeEach(() => {
  layout = layouts[0]
  pending = new Map()
  let serial = 0
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => { pending.set(++serial, callback); return serial })
  vi.stubGlobal('cancelAnimationFrame', (id: number) => pending.delete(id))
  observers = []
  vi.stubGlobal('ResizeObserver', class {
    targets = new Set<Element>()
    disconnect = vi.fn(() => this.targets.clear())
    constructor(public callback: ResizeObserverCallback) { observers.push(this) }
    observe(target: Element) { this.targets.add(target) }
  })
  gl = Object.fromEntries(['createShader', 'shaderSource', 'compileShader', 'getShaderParameter', 'deleteShader', 'createProgram', 'attachShader', 'linkProgram', 'getProgramParameter', 'deleteProgram', 'useProgram', 'createBuffer', 'bindBuffer', 'bufferData', 'getAttribLocation', 'enableVertexAttribArray', 'vertexAttribPointer', 'getUniformLocation', 'uniform3fv', 'uniform1f', 'uniform2f', 'viewport', 'drawArrays', 'deleteBuffer', 'getExtension'].map(name => [name, vi.fn()]))
  gl.createShader.mockReturnValue({}); gl.createProgram.mockReturnValue({}); gl.createBuffer.mockReturnValue({})
  gl.getShaderParameter.mockReturnValue(true); gl.getProgramParameter.mockReturnValue(true)
  gl.getUniformLocation.mockImplementation((_, name) => name)
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => gl as unknown as WebGLRenderingContext)
  rectReads = vi.fn(function (this: Element) {
    if (this.matches('.motion-mesh-host')) return { top: -80, width: 889, height: 840 } as DOMRect
    if (this.matches('.motion-hero svg')) return { top: layout.logoTop, width: layout.logoSize, height: layout.logoSize } as DOMRect
    return { top: 0, width: 889, height: 660 } as DOMRect
  })
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(rectReads)
  setActivePinia(createPinia())
  useEffectsStore().setSetting('mid')
  useGlowRenderStore().setWebglSupported(true)
  useGlowRenderStore().setSetting('mesh')
})

afterEach(() => {
  vi.restoreAllMocks(); vi.unstubAllGlobals()
  document.documentElement.removeAttribute('data-effects')
  document.documentElement.removeAttribute('data-glow-render')
})

function flushFrame(time: number) {
  const callbacks = [...pending.values()]
  pending.clear()
  callbacks.forEach(callback => callback(time))
}
function center() { return gl.uniform1f.mock.calls.filter(call => call[0] === 'u_cy').at(-1)?.[1] }
function expectedCenter() { return (layout.logoTop + layout.logoSize / 2 + 80) / 840 }
async function mountOverview() {
  const shell = document.createElement('main')
  shell.className = 'main'
  shell.innerHTML = '<section class="route-section ov"><div class="motion-stage"></div></section>'
  document.body.append(shell)
  const wrapper = mount(RuntimeMotionMark, { attachTo: shell.querySelector('.motion-stage')!, props: { state: 'running', active: true } })
  await nextTick()
  return { shell, wrapper }
}

it.each(layouts)('uses the actual Logo center after Teleport in $name', async nextLayout => {
  layout = nextLayout
  const { shell, wrapper } = await mountOverview()
  try {
    const backdrop = shell.querySelector('.motion-backdrop')!
    expect(backdrop.parentElement).toBe(shell.querySelector('.route-section'))
    expect(backdrop.querySelector('.motion-hero')).toBeNull()
    observers.forEach(observer => observer.callback([], {} as ResizeObserver))
    flushFrame(1000)
    expect(center()).toBeCloseTo(expectedCenter(), 8)
    expect(pending.size).toBe(0)
  } finally { wrapper.unmount(); shell.remove() }
})

it('repositions a static glow when only the Logo layout changes, and releases its observer on route disposal', async () => {
  const { shell, wrapper } = await mountOverview()
  try {
    flushFrame(1000)
    const anchor = shell.querySelector('.motion-hero svg')!
    expect(observers.some(observer => observer.targets.has(anchor))).toBe(true)
    for (const nextLayout of layouts.slice(1)) {
      layout = nextLayout
      observers.filter(observer => observer.targets.has(anchor)).forEach(observer => observer.callback([], {} as ResizeObserver))
      flushFrame(1016)
      expect(center()).toBeCloseTo(expectedCenter(), 8)
      expect(pending.size).toBe(0)
    }
    expect(gl.drawArrays).toHaveBeenCalledTimes(4)
  } finally {
    wrapper.unmount(); shell.remove()
    expect(observers.every(observer => observer.disconnect.mock.calls.length === 1)).toBe(true)
    expect(pending.size).toBe(0)
  }
})

it('does not read geometry in continuous high-tier frames or draw after backgrounding', async () => {
  useEffectsStore().setSetting('high')
  const { shell, wrapper } = await mountOverview()
  try {
    const reads = rectReads.mock.calls.length
    flushFrame(1000); flushFrame(1016); flushFrame(1032)
    expect(gl.drawArrays).toHaveBeenCalledTimes(3)
    expect(rectReads).toHaveBeenCalledTimes(reads)
    useEffectsStore().setForeground(false)
    await nextTick()
    const draws = gl.drawArrays.mock.calls.length
    observers.forEach(observer => observer.callback([], {} as ResizeObserver))
    flushFrame(2000)
    expect(gl.drawArrays).toHaveBeenCalledTimes(draws)
    expect(pending.size).toBe(0)
  } finally { wrapper.unmount(); shell.remove() }
})
