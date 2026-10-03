import { describe, expect, it } from 'vitest'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
import { windowAddress } from '@/lib/windowAddress'

function snapshot(runtime: StatusSnapshot['matrix']['runtime'], port: number | null): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
    facts: {
      health: 'healthy',
      runtime_label: null,
      opencodex_home: null,
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port,
    pid: '39421',
    can_start: false,
    can_stop: true,
    can_restart: true,
    source: 'live',
  }
}

describe('window address', () => {
  it('shows only while official runtime is running with a port', () => {
    expect(windowAddress(snapshot('running', 10100))).toBe('本地 · 127.0.0.1:10100')
    expect(windowAddress(snapshot('stopped', 10100))).toBe('')
    expect(windowAddress(snapshot('running', null))).toBe('')
    expect(windowAddress(null)).toBe('')
  })
})
