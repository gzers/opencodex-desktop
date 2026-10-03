import { describe, expect, it } from 'vitest'
import {
  applyStartupRoute,
  panelReady,
  runtimeSettled,
  startupRouteHash,
  waitForSettledSnapshot,
} from '@/startup'
import type { HealthState, RuntimeState, StatusSnapshot } from '@/contracts/runtimeStatus'

function snapshot(
  runtime: RuntimeState,
  options: { port?: number | null; health?: HealthState } = {},
): StatusSnapshot {
  const port = options.port === undefined ? (runtime === 'running' ? 7317 : null) : options.port
  return {
    matrix: { runtime, connection: 'unconfigured', operation: 'idle' },
    facts: { health: options.health ?? (runtime === 'running' ? 'healthy' : 'unknown') },
    port,
    source: 'live',
  } as unknown as StatusSnapshot
}

describe('first-open landing page', () => {
  it('lands on the panel when the panel is already running', () => {
    expect(startupRouteHash(snapshot('running'), '')).toBe('#panel')
    expect(startupRouteHash(snapshot('running'), '#')).toBe('#panel')
  })

  it('stays on the overview while the panel is not running', () => {
    for (const runtime of ['not_found', 'stopped', 'starting_failed', 'at_risk', 'unreachable'] as RuntimeState[]) {
      expect(startupRouteHash(snapshot(runtime), '')).toBe('#overview')
    }
  })

  it('treats a running panel without a usable port as not ready', () => {
    expect(panelReady(snapshot('running', { port: null }))).toBe(false)
    expect(startupRouteHash(snapshot('running', { port: null }), '')).toBe('#overview')
  })

  it('treats an unhealthy panel as not ready', () => {
    expect(panelReady(snapshot('running', { health: 'unhealthy' }))).toBe(false)
    expect(startupRouteHash(snapshot('running', { health: 'unhealthy' }), '')).toBe('#overview')
  })

  it('falls back to the overview when no snapshot is available', () => {
    expect(startupRouteHash(null, '')).toBe('#overview')
  })

  it('never overrides an explicit entry (deep link / tray action)', () => {
    expect(startupRouteHash(snapshot('running'), '#extensions?tab=mcp')).toBeNull()
    expect(startupRouteHash(snapshot('stopped'), '#logs')).toBeNull()
    expect(startupRouteHash(snapshot('running'), '#tray')).toBeNull()
  })

  it('writes the landing page before mount so the first frame is already correct', () => {
    const running = { hash: '' }
    applyStartupRoute(snapshot('running'), running)
    expect(running.hash).toBe('#panel')

    const stopped = { hash: '' }
    applyStartupRoute(snapshot('stopped'), stopped)
    expect(stopped.hash).toBe('#overview')

    const explicit = { hash: '#settings?section=backup' }
    applyStartupRoute(snapshot('running'), explicit)
    expect(explicit.hash).toBe('#settings?section=backup')
  })
})

describe('runtime settle detection', () => {
  it('only accepts a settled runtime', () => {
    expect(runtimeSettled('running')).toBe(true)
    expect(runtimeSettled('not_found')).toBe(true)
    for (const transient of ['loading', 'starting', 'stopping'] as RuntimeState[]) {
      expect(runtimeSettled(transient)).toBe(false)
    }
    expect(runtimeSettled(null)).toBe(false)
    expect(runtimeSettled(undefined)).toBe(false)
  })
})

describe('waitForSettledSnapshot', () => {
  it('keeps polling until the runtime is settled', async () => {
    const seen: (StatusSnapshot | null)[] = [snapshot('loading'), snapshot('starting'), snapshot('running')]
    let index = -1
    const result = await waitForSettledSnapshot({
      read: () => seen[Math.max(index, 0)],
      refresh: async () => {
        index += 1
      },
      delay: async () => {},
      timeoutMs: 500,
    })
    expect(result?.matrix.runtime).toBe('running')
    expect(index).toBe(2)
  })

  // `get_status_snapshot` 每轮都先做一次真实采集（去抖期内读到的是上一次采集结果），
  // 因此读到 not_found 就是「确实没发现面板」的判定，而不是「还没采集」。
  it('accepts not_found as a settled verdict when the panel is absent', async () => {
    const result = await waitForSettledSnapshot({
      read: () => snapshot('not_found'),
      refresh: async () => {},
      delay: async () => {},
      timeoutMs: 0,
    })
    expect(result?.matrix.runtime).toBe('not_found')
  })

  it('gives up at the timeout and returns whatever was read last', async () => {
    let ticks = 0
    const result = await waitForSettledSnapshot({
      read: () => snapshot('loading'),
      refresh: async () => {
        ticks += 1
      },
      delay: async () => {},
      timeoutMs: 0,
    })
    expect(result?.matrix.runtime).toBe('loading')
    expect(ticks).toBe(1)
  })

  it('survives a failing refresh and still resolves', async () => {
    const result = await waitForSettledSnapshot({
      read: () => snapshot('stopped'),
      refresh: async () => {
        throw new Error('backend unavailable')
      },
      delay: async () => {},
      timeoutMs: 100,
    })
    expect(result?.matrix.runtime).toBe('stopped')
  })
})
