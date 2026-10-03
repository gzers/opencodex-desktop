import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'
import { projectMotion, type MotionStateId } from '@/features/runtime/motion/projection'
import { MOTION_STATES, PALETTES, STRUCTURE_AMPLITUDE, computeGeometry } from '@/features/runtime/motion/states'
import { environmentChecksOrdered } from '@/features/environment/presentation'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import { createMotionScene } from '@/features/runtime/motion/scene'
import type { RuntimeState } from '@/contracts/runtimeStatus'
import type { EnvironmentReport } from '@/features/environment/api'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: query === 'prefers-color-scheme: dark',
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

describe('motion projection consumes the existing runtime line', () => {
  const cases: Array<[RuntimeState, MotionStateId]> = [
    ['loading', 'confirming'],
    ['not_found', 'not_ready'],
    ['stopped', 'stopped'],
    ['starting', 'starting'],
    ['pending', 'starting'],
    ['stopping', 'stopping'],
    ['starting_failed', 'failed'],
    ['running', 'running'],
    // at_risk = 代理未在运行 + startup at-risk：主线与形象都按「未运行」，风险只走说明。
    ['at_risk', 'stopped'],
    ['external_takeover', 'problem'],
    ['unreachable', 'problem'],
  ]

  it.each(cases)('maps %s to %s', (runtimeState, expected) => {
    expect(projectMotion({ runtimeState }).state).toBe(expected)
  })

  it('turns an ongoing problem into the柔性 problem structure while staying running', () => {
    const result = projectMotion({ runtimeState: 'running', problem: true })
    expect(result.state).toBe('problem')
    expect(result.mainline).toBe('运行中')
  })

  it('keeps the mainline labels aligned with the confirmed prototype STATE_MODEL', () => {
    // 待就绪（pending）进程已起：主线归「运行中」，不是「未运行」。
    const pending = projectMotion({ runtimeState: 'pending' })
    expect(pending.mainline).toBe('运行中')
    // UI规范 §25.1：进程已起时的说明为「进程已起 · 待就绪」。
    expect(pending.caption).toBe('进程已起 · 待就绪')
    expect(projectMotion({ runtimeState: 'starting' }).mainline).toBe('未运行')
    expect(projectMotion({ runtimeState: 'starting_failed' }).mainline).toBe('未运行')
    expect(projectMotion({ runtimeState: 'stopped' }).mainline).toBe('未运行')
    expect(projectMotion({ runtimeState: 'not_found' }).mainline).toBe('待接入')
    expect(projectMotion({ runtimeState: 'loading' }).mainline).toBe('正在确认')
    // at_risk 的进程事实是「未运行」，概览形象与「未运行」逐字一致（2026-10-03 用户确认：
    // 原型的「未运行」呈现才是对的），不再另加说明句。
    const atRisk = projectMotion({ runtimeState: 'at_risk' })
    expect(atRisk.mainline).toBe('未运行')
    expect(atRisk.caption).toBe('')
    expect(atRisk.state).toBe('stopped')
  })

  it('only marks settled lines as stale, never in-flight operations', () => {
    expect(projectMotion({ runtimeState: 'running', fresh: false }).state).toBe('stale')
    expect(projectMotion({ runtimeState: 'stopped', fresh: false }).stale).toBe(true)
    // 进行中的启动/确认不被旧事实顶替。
    expect(projectMotion({ runtimeState: 'starting', fresh: false }).state).toBe('starting')
    expect(projectMotion({ runtimeState: 'pending', fresh: false }).state).toBe('starting')
    expect(projectMotion({ runtimeState: 'stopping', fresh: false }).state).toBe('stopping')
    expect(projectMotion({ runtimeState: 'loading', fresh: false }).state).toBe('confirming')
  })

  it('defaults to fresh so stale is never faked', () => {
    expect(projectMotion({ runtimeState: 'running' }).stale).toBe(false)
  })

  it('keeps 未运行 and 运行中 clearly apart so start/stop are distinguishable', () => {
    // 回归（2026-09-29 用户报告）：曾把 at_risk 归「运行中」，导致启停两态都显示「运行中」、
    // 状态形象也不变。at_risk 的进程事实是「未运行」，必须与 running 明显不同。
    const stoppedRisk = projectMotion({ runtimeState: 'at_risk' })
    const running = projectMotion({ runtimeState: 'running' })
    expect(stoppedRisk.mainline).toBe('未运行')
    expect(running.mainline).toBe('运行中')
    expect(stoppedRisk.state).not.toBe(running.state)
  })

  it('marks an old observation as stale when the latest collect failed', () => {
    // 采集失败但仍有旧的已落定观测：主线归「正在确认」并标记过期。
    const stale = projectMotion({ runtimeState: 'running', fresh: false })
    expect(stale.state).toBe('stale')
    expect(stale.mainline).toBe('正在确认')
    expect(stale.stale).toBe(true)
  })
})

describe('environment checks follow the confirmed order semantics', () => {
  const checks = [
    { name: 'Node.js', value: '未发现', state: 'bad' as const, label: '未通过' as const },
    { name: 'npm', value: '未检查', state: 'muted' as const, label: '未检查' as const },
    { name: 'OpenCodex', value: '未检查', state: 'muted' as const, label: '未检查' as const },
  ]

  it('shows only the current check as 检查中 and the rest as 待检查 while checking', () => {
    const ordered = environmentChecksOrdered(checks, true)
    expect(ordered.map(item => item.value)).toEqual(['检查中', '待检查', '待检查'])
    expect(ordered.map(item => item.tone)).toEqual(['busy', 'idle', 'idle'])
    expect(ordered.map(item => item.name)).toEqual(['Node.js', 'npm', 'OpenCodex'])
  })

  it('maps 未检查 to 待检查 outside the checking state', () => {
    const ordered = environmentChecksOrdered(checks, false)
    expect(ordered.map(item => item.value)).toEqual(['未发现', '待检查', '待检查'])
    expect(ordered.map(item => item.tone)).toEqual(['bad', 'idle', 'idle'])
  })
})

describe('motion model constants', () => {
  it('fixes the structure amplitude at the confirmed maximum 1.25', () => {
    expect(STRUCTURE_AMPLITUDE).toBe(1.25)
  })

  it('defines exactly nine states with complete palettes', () => {
    expect(MOTION_STATES).toHaveLength(9)
    expect(new Set(MOTION_STATES.map(state => state.id)).size).toBe(9)
    for (const state of MOTION_STATES) {
      expect(PALETTES[state.id].colors).toHaveLength(3)
      expect(state.effect.period).toBeGreaterThan(0)
    }
  })
})

describe('motion geometry', () => {
  it('produces three finite nodes and three links within valid connection range', () => {
    for (const state of MOTION_STATES) {
      const geometry = computeGeometry(state.shape, STRUCTURE_AMPLITUDE)
      expect(geometry.nodes).toHaveLength(3)
      expect(geometry.links).toHaveLength(3)
      expect(geometry.connection).toBeGreaterThanOrEqual(0)
      expect(geometry.connection).toBeLessThanOrEqual(1)
      for (const node of geometry.nodes) {
        expect(Number.isFinite(node.x)).toBe(true)
        expect(Number.isFinite(node.y)).toBe(true)
      }
      for (const link of geometry.links) {
        expect(link).not.toContain('NaN')
      }
    }
  })
})

describe('RuntimeMotionMark renders the brand structure', () => {
  it('draws the cloud outline with masked inner nodes and links, decoration only', () => {
    setActivePinia(createPinia())
    const wrapper = mount(RuntimeMotionMark, { props: { state: 'running', active: false } })
    const root = wrapper.find('.motion-mark')
    expect(root.attributes('aria-hidden')).toBe('true')
    expect(wrapper.findAll('.motion-hero svg')).toHaveLength(1)
    // 云形轮廓 1 条；内部聚合层 3 段连接 + 3 圆节点；原始镂空 5 条（对照用，默认隐藏）。
    expect(wrapper.findAll('.motion-hero > svg > path')).toHaveLength(1)
    expect(wrapper.findAll('.motion-hero [data-layer="core"] path')).toHaveLength(3)
    expect(wrapper.findAll('.motion-hero [data-layer="core"] circle')).toHaveLength(3)
    expect(wrapper.findAll('.motion-hero [data-layer="original"] path')).toHaveLength(5)
    // 背景三光团 + 能量光束 + 近地投影。
    expect(wrapper.findAll('.motion-ambient .cloud')).toHaveLength(3)
    expect(wrapper.findAll('.motion-ambient .energy')).toHaveLength(1)
  })
})

describe('motion scene advances the animation over frames (regression)', () => {
  it('rotates the inner core across successive frames instead of freezing at 0', () => {
    // 回归：`tick` 曾每帧调用 `wake()`，而 `wake()` 会清空 `lastTime` ⇒ 每帧 `dt=0`，
    // 动画永不推进（rAF 仍空转耗 CPU）。本用例用受控帧时钟断言逐帧推进。
    const pending: Array<(t: number) => void> = []
    const origRaf = window.requestAnimationFrame
    const origCaf = window.cancelAnimationFrame
    ;(window as unknown as { requestAnimationFrame: (cb: (t: number) => void) => number }).requestAnimationFrame = (cb) => {
      pending.push(cb)
      return pending.length
    }
    ;(window as unknown as { cancelAnimationFrame: (id: number) => void }).cancelAnimationFrame = () => {}
    try {
      setActivePinia(createPinia())
      const wrapper = mount(RuntimeMotionMark, { props: { state: 'running', active: true } })
      const angle = () => {
        const t = wrapper.find('.motion-hero [data-layer="core"]').attributes('transform') ?? ''
        const m = /rotate\(([-0-9.]+)/.exec(t)
        return m ? Number(m[1]) : 0
      }
      const seen = [angle()]
      let now = 1000
      for (let i = 0; i < 12; i += 1) {
        const cb = pending.shift()
        if (!cb) break
        now += 16
        cb(now)
        seen.push(angle())
      }
      expect(seen.length).toBeGreaterThan(3)
      // 首帧允许 dt=0 停在 0；此后必须严格推进（不冻结）。
      expect(seen[seen.length - 1]).toBeGreaterThan(0)
      for (let i = 2; i < seen.length; i += 1) expect(seen[i]).toBeGreaterThan(seen[i - 1])
    } finally {
      ;(window as unknown as { requestAnimationFrame: typeof origRaf }).requestAnimationFrame = origRaf
      ;(window as unknown as { cancelAnimationFrame: typeof origCaf }).cancelAnimationFrame = origCaf
    }
  })

  it('eases a state change over multiple frames instead of jumping', () => {
    const pending: Array<(t: number) => void> = []
    const origRaf = window.requestAnimationFrame
    const origCaf = window.cancelAnimationFrame
    ;(window as unknown as { requestAnimationFrame: (cb: (t: number) => void) => number }).requestAnimationFrame = (cb) => {
      pending.push(cb)
      return pending.length
    }
    ;(window as unknown as { cancelAnimationFrame: (id: number) => void }).cancelAnimationFrame = () => {}
    const ambient = document.createElement('div')
    const host = document.createElement('div')
    document.body.append(ambient, host)
    try {
      const scene = createMotionScene({ ambient, markHost: host })
      scene.setReduced(false)
      scene.setState('running', true)
      scene.setActive(true)
      // 取首节点 cy（= 12 - radius，随结构幅度变化）；首节点 cx 恒为 12（cos(-90°)=0）。
      const nodeX = () => Number(host.querySelector('[data-layer="core"] circle')?.getAttribute('cy') ?? '0')
      let now = 1000
      const step = () => {
        const cb = pending.shift()
        if (!cb) return false
        now += 16
        cb(now)
        return true
      }
      for (let i = 0; i < 20; i += 1) step() // 先跑到 running 稳定
      expect(nodeX()).toBeGreaterThan(0)
      scene.setState('stopped') // 非 immediate：应逐帧缓动
      const xs: number[] = []
      for (let i = 0; i < 40; i += 1) {
        if (!step()) break
        xs.push(nodeX())
      }
      const distinct = new Set(xs.map((x) => x.toFixed(3)))
      expect(distinct.size).toBeGreaterThan(3) // 多帧中间态 ⇒ 平滑，而非一帧跳变
      expect(xs.length).toBeGreaterThan(10)
      scene.destroy()
    } finally {
      ;(window as unknown as { requestAnimationFrame: typeof origRaf }).requestAnimationFrame = origRaf
      ;(window as unknown as { cancelAnimationFrame: typeof origCaf }).cancelAnimationFrame = origCaf
      ambient.remove()
      host.remove()
    }
  })

  it('carries a perceptible float amplitude while running (regression)', () => {
    // 回归（2026-09-29 用户报告）：原悬浮幅度太小（running 纵向仅 0.32/24 ≈ 1.9px），
    // 不仔细观察看不出在动。断言 svg 位移百分比峰峰值达到肉眼可辨的量级。
    const pending: Array<(t: number) => void> = []
    const origRaf = window.requestAnimationFrame
    const origCaf = window.cancelAnimationFrame
    ;(window as unknown as { requestAnimationFrame: (cb: (t: number) => void) => number }).requestAnimationFrame = (cb) => {
      pending.push(cb)
      return pending.length
    }
    ;(window as unknown as { cancelAnimationFrame: (id: number) => void }).cancelAnimationFrame = () => {}
    const ambient = document.createElement('div')
    const host = document.createElement('div')
    document.body.append(ambient, host)
    try {
      const scene = createMotionScene({ ambient, markHost: host })
      scene.setReduced(false)
      scene.setState('running', true)
      scene.setActive(true)
      const shiftY = () => {
        const t = host.querySelector('svg')?.style.transform ?? ''
        const m = /translate\([-0-9.]+%,\s*([-0-9.]+)%\)/.exec(t)
        return m ? Math.abs(Number(m[1])) : 0
      }
      let now = 1000
      let min = Number.POSITIVE_INFINITY
      let max = 0
      for (let i = 0; i < 600; i += 1) {
        const cb = pending.shift()
        if (!cb) break
        now += 16
        cb(now)
        if (i < 120) continue // 先让阻尼追踪收敛，再取峰谷
        const v = shiftY()
        if (v < min) min = v
        if (v > max) max = v
      }
      // running：1.0 单位 / 24 单位 ≈ 4.17% 振幅 ⇒ 峰峰值 ≈ 8.3%（≈11.7px @140px）。
      expect(max - min).toBeGreaterThan(6)
      scene.destroy()
    } finally {
      ;(window as unknown as { requestAnimationFrame: typeof origRaf }).requestAnimationFrame = origRaf
      ;(window as unknown as { cancelAnimationFrame: typeof origCaf }).cancelAnimationFrame = origCaf
      ambient.remove()
      host.remove()
    }
  })
})

describe('overview integrates the motion area', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
  })

  function readyEnvironment(): EnvironmentReport {
    const found = { found: true, path: '/fixtures/bin', version: null }
    return { node: { ...found }, npm: { ...found }, ocx: { ...found }, gate: 'ready', shortCircuited: false, brewFound: true }
  }

  it('环境就绪时：居中状态区、无环境摘要行、保留三卡与最近事件', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    invoke.mockImplementation((command: string) =>
      command === 'discover_environment' ? Promise.resolve(readyEnvironment()) : Promise.resolve(undefined),
    )
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    useRouteStore().go('overview')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.motion-mark').exists()).toBe(true)
    // UI规范 §25.1：环境摘要行不再常驻；环境明细并入运行详情。
    expect(wrapper.find('.motion-env-row').exists()).toBe(false)
    expect(wrapper.find('[data-testid="overview-env-summary"]').exists()).toBe(false)
    // 底部原三卡保留。
    expect(wrapper.findAll('.mods .mod')).toHaveLength(3)
    // 最近事件摘要卡（就绪态）。
    expect(wrapper.find('[data-testid="overview-recent-events"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="environment-gate"]').exists()).toBe(false)
    expect(wrapper.find('.ov').attributes('data-mode')).toBe('ready')
  })

  it('环境未就绪时：收起三卡与最近事件，展示准备卡', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    invoke.mockImplementation((command: string) =>
      command === 'discover_environment'
        ? Promise.resolve({ node: { found: true, path: '/n', version: null }, npm: { found: true, path: '/n', version: null }, ocx: { found: false, path: null, version: null }, gate: 'missing_ocx', shortCircuited: true, brewFound: true })
        : Promise.resolve(undefined),
    )
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    useRouteStore().go('overview')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ov').attributes('data-mode')).toBe('setup')
    expect(wrapper.find('[data-testid="environment-gate"]').exists()).toBe(true)
    expect(wrapper.findAll('.mods .mod')).toHaveLength(0)
    expect(wrapper.find('[data-testid="overview-recent-events"]').exists()).toBe(false)
    expect(wrapper.findAll('.overview-checks li')).toHaveLength(3)
  })
})
