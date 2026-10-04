// 概览状态形象的渲染引擎（命令式、无依赖）。
//
// 移植自已确认候选 `2026-09-28-Logo本体形变/morph.js` 的 v08 内核：任意切态都从
// **当前值与当前速度**做临界阻尼追踪，不归零、不排队；背景光团与光束用连续相位，
// 切态不重置角度。它只消费 `projection.ts` 给出的形象状态，不采集状态、不推定成功。
import { CENTER, LOGO_BODY, LOGO_HOLES, NODE_RADIUS, ORIGINAL_HOLES_TRANSFORM } from './logo'
import type { MotionStateId } from './projection'
import {
  BRAND_RGB,
  EFFECTS,
  MOTION_STATES,
  STRUCTURE_AMPLITUDE,
  colorText,
  computeGeometry,
  motionState,
  paletteRgb,
  rgb,
} from './states'

const NS = 'http://www.w3.org/2000/svg'

type Vec = number[]

function el<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string> = {}): SVGElementTagNameMap[K] {
  const node = document.createElementNS(NS, tag)
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value)
  return node
}

export interface MotionSceneHandle {
  setState(id: MotionStateId, immediate?: boolean): void
  setActive(active: boolean): void
  setReduced(reduced: boolean): void
  destroy(): void
}

export interface MotionSceneOptions {
  ambient: HTMLElement
  markHost: HTMLElement
  /** 每帧抛出的状态投影配色（与 CSS 光团同源）；WEBGL 网格着色器据此取色。 */
  onGlow?: (colors: readonly string[]) => void
}

interface Mark {
  svg: SVGSVGElement
  core: SVGGElement
  originalCore: SVGGElement
  gradient: SVGLinearGradientElement
  stops: SVGStopElement[]
  links: SVGPathElement[]
  nodes: SVGCircleElement[]
}

let serial = 0

function createMark(host: HTMLElement): Mark {
  const uid = `motion-mark-${serial++}`
  const svg = el('svg', { viewBox: '0 0 24 24', 'aria-hidden': 'true' })
  svg.style.transformOrigin = `${(CENTER.x / 24) * 100}% ${(CENTER.y / 24) * 100}%`
  const defs = el('defs')
  const gradient = el('linearGradient', { id: `${uid}-g`, gradientUnits: 'userSpaceOnUse', x1: '12', x2: '12', y1: '3', y2: '21' })
  const stops = BRAND_RGB.map((color, index) => el('stop', { offset: String(index / 2), 'stop-color': colorText(color) }))
  stops.forEach((stop) => gradient.append(stop))
  const mask = el('mask', { id: `${uid}-m`, maskUnits: 'userSpaceOnUse', x: '-4', y: '-4', width: '32', height: '32' })
  mask.append(el('rect', { x: '-4', y: '-4', width: '32', height: '32', fill: 'white' }))
  const core = el('g', { 'data-layer': 'core' })
  const originalCore = el('g', { 'data-layer': 'original', transform: ORIGINAL_HOLES_TRANSFORM, display: 'none' })
  const links = Array.from({ length: 3 }, () =>
    el('path', { fill: 'none', stroke: 'black', 'stroke-width': '0.78', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }),
  )
  const nodes = Array.from({ length: 3 }, () => el('circle', { r: String(NODE_RADIUS), fill: 'black' }))
  core.append(...links, ...nodes)
  LOGO_HOLES.forEach((d) => originalCore.append(el('path', { d, fill: 'black' })))
  mask.append(core, originalCore)
  const body = el('path', { d: LOGO_BODY, fill: `url(#${uid}-g)`, mask: `url(#${uid}-m)` })
  defs.append(gradient, mask)
  svg.append(defs, body)
  host.append(svg)
  return { svg, core, originalCore, gradient, stops, links, nodes }
}

export function createMotionScene({ ambient, markHost, onGlow }: MotionSceneOptions): MotionSceneHandle {
  const mark = createMark(markHost)
  const clouds = Array.from(ambient.querySelectorAll<HTMLElement>('.cloud'))
  const beam = ambient.querySelector<HTMLElement>('.energy')
  const floor = ambient.querySelector<HTMLElement>('.light-floor')

  const defaultIndex = MOTION_STATES.findIndex((state) => state.id === 'confirming')
  let selected = defaultIndex >= 0 ? defaultIndex : 0
  let clock = 0
  let lastTime = 0
  let frame = 0
  let disposed = false
  let active = true
  let reduced = false
  const strength = STRUCTURE_AMPLITUDE

  let current: Vec = [...MOTION_STATES[selected].shape]
  let staleShape: Vec = [0, 1, 0, 0]
  let currentColors: Vec[] = paletteRgb(MOTION_STATES[selected].id)
  let appearance: Vec = [1, 1]
  let floatPose: Vec = [0, 0]
  let outerAngle = 0
  let innerAngle = 0
  let outerVelocity = 0
  let innerVelocity = 0
  let motionPhase = 0
  let phaseSpeed = (Math.PI * 2) / 6
  let beamAngle = 0
  let beamVelocity = 0
  let transitionStart = 0

  const tracks = new Map<string, Vec>()
  const palettes = new Map(MOTION_STATES.map(state => [state.id, paletteRgb(state.id)]))
  const shapeTarget: Vec = [0, 0, 0, 0]
  const ambientBuffer: Vec = Array(14).fill(0)
  const normalAppearance: Vec = [1, 1]
  const staleAppearance: Vec = [0.15, 0.62]
  let lastGlow = ''
  let lastGeometry = ''
  let ambientState: Vec = [...ambientTarget()]

  function reducedMotion(): boolean {
    return reduced
  }

  function liveShape(id: MotionStateId, shape: readonly number[]): Vec {
    const next = shapeTarget
    const source = id === 'stale' ? staleShape : shape
    for (let i = 0; i < 4; i++) next[i] = source[i]
    if (id === 'stale') return next
    if (reducedMotion() || id === 'running') return next
    if (id === 'starting') {
      next[0] += 0.1 * (1 + Math.sin(motionPhase))
      next[1] = 0.62 + 0.16 * Math.sin(motionPhase)
      next[2] = 0.16 * Math.sin(motionPhase)
      next[3] = 0.16 * Math.cos(motionPhase)
    } else if (id === 'problem') {
      next[2] = 0.22 * Math.sin(motionPhase)
      next[3] = 0.22 * Math.cos(motionPhase)
    } else if (id === 'not_ready' || id === 'confirming') {
      next[0] += 0.1 * Math.sin(motionPhase)
    }
    return next
  }

  function floatTarget(id: MotionStateId, age: number): Vec {
    const phase = motionPhase
    // 悬浮幅度按「肉眼可辨」标定（2026-09-29 用户反馈：原幅度太小，不仔细观察看不出在动）。
    // 单位是 24 单位视图坐标（1 单位 ≈ 5.83px @140px），下同；以下纵向振幅 ≈ 10–13px 峰峰值。
    switch (id) {
      case 'starting':
        return [0.3 * Math.sin(phase), -0.3 - 1.05 * Math.pow((1 - Math.cos(phase)) / 2, 2)]
      case 'running':
        return [0.3 * Math.sin(phase), -0.55 + 1.0 * Math.sin(phase)]
      case 'stopping':
        return [0, 0.65 + 0.35 * Math.sin(phase)]
      case 'failed':
        return [0.26 * Math.sin(phase), 0.6 + 0.4 * Math.sin(phase) + 0.35 * Math.exp(-Math.max(0, age) / 0.6)]
      case 'not_ready':
        return [0.6 * Math.sin(phase), -0.1 + 1.1 * Math.cos(phase)]
      case 'stopped':
        return [0, 0.65 + 0.5 * Math.sin(phase)]
      case 'problem':
        return [0.6 * Math.sin(phase), -0.25 + 0.6 * Math.sin(phase * 2)]
      case 'confirming':
        return [1.0 * Math.sin(phase), -0.15 + 0.6 * Math.cos(phase)]
      default:
        return [0.3 * Math.sin(phase), 0.45 * Math.cos(phase)]
    }
  }

  function rotateStep(dt: number): void {
    const id = MOTION_STATES[selected].id
    const effect = EFFECTS[id]
    const age = (clock - transitionStart) / 1000
    const phase = motionPhase
    let inside = effect.inner + effect.rock * ((Math.PI * 2) / effect.period) * Math.cos(phase)
    if (id === 'starting') inside *= 0.78 + 0.22 * Math.sin(phase)
    if (id === 'problem') inside *= 0.65 + 0.35 * Math.sin(phase)
    if (id === 'stopping') inside += 50 * Math.exp(-Math.max(0, age) / 1.3)
    const error = (((outerAngle + 180) % 360) + 360) % 360 - 180
    const outside = id === 'stale' ? 0 : effect.outer || -error * 2.5
    const smooth = 1 - Math.exp(-dt * 5)
    outerVelocity += (outside - outerVelocity) * smooth
    innerVelocity += (inside - innerVelocity) * smooth
    outerAngle += outerVelocity * dt
    innerAngle += innerVelocity * dt
  }

  // 连续相位 + 保留位置/速度的临界阻尼追踪。切态只换目标，不重置画面或速度。
  function track(key: string, values: Vec, target: Vec, dt: number, omega = 9): Vec {
    let velocity = tracks.get(key)
    if (!velocity) {
      velocity = values.map(() => 0)
      tracks.set(key, velocity)
    }
    const decay = Math.exp(-omega * dt)
    for (let index = 0; index < values.length; index++) {
      const offset = values[index] - target[index]
      const j = velocity[index] + omega * offset
      const next = target[index] + (offset + j * dt) * decay
      velocity[index] = (velocity[index] - omega * j * dt) * decay
      if (Math.abs(next - target[index]) < 1e-7 && Math.abs(velocity[index]) < 1e-6) {
        velocity[index] = 0
        values[index] = target[index]
      } else values[index] = next
    }
    return values
  }

  function ambientTarget(): Vec {
    const effect = EFFECTS[MOTION_STATES[selected].id]
    const phase = motionPhase
    const target = ambientBuffer
    for (let index = 0; index < 3; index++) {
      const offset = index * 4
      target[offset] = Math.sin(phase * (1 + index * 0.17) + index * 2) * (20 + effect.energy * 32)
      target[offset + 1] = Math.cos(phase * (0.83 + index * 0.13) + index * 1.4) * (12 + effect.energy * 19)
      target[offset + 2] = 1 + Math.sin(phase + index) * 0.18
      target[offset + 3] = 0.32 + Math.sin(phase + index) * 0.08
    }
    target[12] = 0.88 + effect.energy * 0.2 + Math.sin(phase) * 0.1
    target[13] = (0.1 + effect.energy * 0.2) * (1 + Math.sin(phase) * 0.18)
    return target
  }

  function draw(): void {
    const time = clock / 1000
    const colors = currentColors
    const glowText = colors.map((color) => colorText(color))
    const glowKey = glowText.join('|')
    if (glowKey !== lastGlow) {
      lastGlow = glowKey
      ;['a', 'b', 'c'].forEach((key, index) => ambient.style.setProperty(`--glow-${key}`, glowText[index]))
      onGlow?.(glowText)
      mark.stops.forEach((stop, index) => stop.setAttribute('stop-color', glowText[index]))
    }
    if (beam) {
      beam.style.transform = `rotate(${beamAngle}deg) scale(${ambientState[12]})`
      beam.style.opacity = String(ambientState[13])
    }
    if (floor) {
      const height = reducedMotion() ? 0 : floatPose[1]
      floor.style.transform = `scaleX(${1 - height * 0.13})`
      floor.style.opacity = String(0.2 + height * 0.06)
    }
    clouds.forEach((cloud, index) => {
      const offset = index * 4
      const x = ambientState[offset], y = ambientState[offset + 1], scale = ambientState[offset + 2], opacity = ambientState[offset + 3]
      cloud.style.transform = `translate(${x}px,${y}px) scale(${scale})`
      cloud.style.opacity = String(opacity)
    })

    const geometryKey = current.join('|')
    if (geometryKey !== lastGeometry) {
      lastGeometry = geometryKey
      const geometry = computeGeometry(current as unknown as readonly [number, number, number, number], strength)
      geometry.nodes.forEach((node, index) => {
        mark.nodes[index].setAttribute('cx', String(node.x))
        mark.nodes[index].setAttribute('cy', String(node.y))
    })
    geometry.links.forEach((d, index) => {
      mark.links[index].setAttribute('d', d)
      mark.links[index].setAttribute('opacity', String(geometry.connection))
    })
    }
    const shift = Math.sin(time * 0.65) * 3
    mark.gradient.setAttribute('x1', String(8 + shift))
    mark.gradient.setAttribute('x2', String(16 - shift))
    mark.stops[1].setAttribute('offset', String(0.5 + Math.sin(time * 0.85) * 0.14))

    const moving = !reducedMotion()
    mark.svg.style.transform = `translate(${moving ? (floatPose[0] / 24) * 100 : 0}%,${moving ? (floatPose[1] / 24) * 100 : 0}%) rotate(${moving ? outerAngle : 0}deg)`
    mark.core.setAttribute('transform', `rotate(${moving ? innerAngle : 0} ${CENTER.x} ${CENTER.y})`)
    mark.svg.style.filter = `saturate(${appearance[0]})`
    mark.svg.style.opacity = String(appearance[1])
  }

  function tick(now: number): void {
    frame = 0
    if (disposed || !active) return
    const dt = lastTime ? Math.min(now - lastTime, 50) / 1000 : 0
    clock += dt * 1000
    lastTime = now
    const state = MOTION_STATES[selected]
    const age = (clock - transitionStart) / 1000
    if (!reducedMotion()) {
      phaseSpeed += ((Math.PI * 2) / EFFECTS[state.id].period - phaseSpeed) * (1 - Math.exp(-dt * 5))
      motionPhase += phaseSpeed * dt
      rotateStep(dt)
      let target = liveShape(state.id, state.shape)
      if (state.id === 'stopping' && age < 1) target[2] += 0.12 * Math.sin(Math.PI * age) ** 2
      current = track('shape', current, target, dt)
      floatPose = track('float', floatPose, floatTarget(state.id, age), dt, 8)
      for (let index = 0; index < currentColors.length; index++) track(`color${index}`, currentColors[index], palettes.get(state.id)![index], dt, 8)
      appearance = track('appearance', appearance, state.id === 'stale' ? staleAppearance : normalAppearance, dt, 8)
      ambientState = track('ambient', ambientState, ambientTarget(), dt, 7)
      const targetVelocity = state.id === 'confirming' ? 65 * phaseSpeed * Math.cos(motionPhase) : innerVelocity * 0.55
      beamVelocity += (targetVelocity - beamVelocity) * (1 - Math.exp(-dt * 5))
      beamAngle += beamVelocity * dt
    }
    draw()
    // 续帧只排程、**不**清空 lastTime：否则每帧 dt 恒为 0，动画永不推进（同时 rAF 空转耗 CPU）。
    if (!reducedMotion()) schedule()
  }

  function schedule(): void {
    if (!frame && !disposed && active && typeof requestAnimationFrame === 'function') {
      frame = requestAnimationFrame(tick)
    }
  }

  // 重新起跑（首帧 / 暂停恢复 / 切态）：清空 lastTime 让首帧 dt=0，避免时间跳变。
  function wake(): void {
    lastTime = 0
    schedule()
  }

  function setState(id: MotionStateId, immediate = false): void {
    const index = MOTION_STATES.findIndex((state) => state.id === id)
    const next = index >= 0 ? index : selected
    if (MOTION_STATES[next].id === 'stale' && MOTION_STATES[selected].id !== 'stale') staleShape = [...current]
    selected = next
    transitionStart = clock
    if (immediate || reducedMotion()) {
      current = MOTION_STATES[next].id === 'stale' ? [...staleShape] : [...MOTION_STATES[next].shape]
      currentColors = paletteRgb(MOTION_STATES[next].id)
      appearance = MOTION_STATES[next].id === 'stale' ? [0.15, 0.62] : [1, 1]
      ambientState = [...ambientTarget()]
      tracks.clear()
      draw()
    }
    // 暂停时只更新目标；继续后再平滑趋近。
    wake()
  }

  draw()
  wake()

  return {
    setState,
    setActive(next: boolean) {
      active = next
      if (!active && frame) {
        cancelAnimationFrame(frame)
        frame = 0
        lastTime = 0
      } else if (active) {
        wake()
      }
    },
    setReduced(next: boolean) {
      if (next === reduced) return
      reduced = next
      // 减少动态：直接呈现当前状态的静态目标（不逐帧更新）。
      if (reduced) {
        const state = MOTION_STATES[selected]
        current = state.id === 'stale' ? [...staleShape] : [...state.shape]
        currentColors = paletteRgb(state.id)
        appearance = state.id === 'stale' ? [0.15, 0.62] : [1, 1]
        tracks.clear()
        draw()
      } else {
        wake()
      }
    },
    destroy() {
      disposed = true
      active = false
      if (frame) cancelAnimationFrame(frame)
      frame = 0
      mark.svg.remove()
    },
  }
}

// 供测试引用的稳定常量。
export const MOTION_IDS = MOTION_STATES.map((state) => state.id)
export { motionState, rgb }
