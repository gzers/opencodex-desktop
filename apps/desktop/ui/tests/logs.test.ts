import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readLogs, type LogsDto } from '@/features/diagnostics/logs'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<LogsDto> = {}): LogsDto {
  return {
    kind: 'app',
    lines: ['<span class="time">10:23:40</span> [manager] status refresh: ok'],
    redactedLineCount: 0,
    truncated: false,
    fileMissing: false,
    fallbackFile: null,
    ...overrides,
  }
}

describe('logs command integration', () => {
  beforeEach(() => {
    invoke.mockReset()
  })

  it('invokes the frozen read_logs contract with camelCase DTO', async () => {
    const payload = dto({ kind: 'agent', redactedLineCount: 2, truncated: true })
    invoke.mockResolvedValue(payload)
    await expect(readLogs('agent')).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'agent' })
  })

  it('propagates backend read failures without inventing lines', async () => {
    invoke.mockRejectedValue(new Error('backend unavailable'))
    await expect(readLogs('app')).rejects.toThrow('backend unavailable')
    expect(invoke).toHaveBeenCalledWith('read_logs', { kind: 'app' })
  })
})
