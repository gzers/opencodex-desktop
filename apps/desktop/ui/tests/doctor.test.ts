import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { runDoctor, type DoctorDto } from '@/features/diagnostics/doctor'
import { doctorReportText } from '@/features/diagnostics/presentation'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<DoctorDto> = {}): DoctorDto {
  return {
    mode: 'read_only',
    lines: [
      '<span class="ok">opencodex doctor</span>  · 只读模式',
      '',
      'Provider / OAuth',
      '  <span class="ok">ok</span>  provider key: <span class="mask">••••••••••••</span>',
    ],
    truncated: false,
    ...overrides,
  }
}

describe('doctor command contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('invokes the frozen read-only doctor command', async () => {
    const payload = dto()
    invoke.mockResolvedValue(payload)
    await expect(runDoctor()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('run_doctor')
  })

  it('loads doctor report into the shared store and preserves empty lines', async () => {
    const payload = dto({ lines: ['first', '', 'third'] })
    invoke.mockResolvedValue(payload)
    const app = useAppStore()
    await app.loadDoctorReport()
    expect(app.doctorReport).toEqual(payload)
    expect(app.doctorError).toBe(false)
    expect(app.doctorLoading).toBe(false)
  })

  it('preserves the previous report when IPC fails', async () => {
    const previous = dto({ lines: ['previous'] })
    invoke.mockResolvedValueOnce(previous)
    const app = useAppStore()
    await app.loadDoctorReport()
    invoke.mockRejectedValueOnce(new Error('backend unavailable'))
    await app.loadDoctorReport()
    expect(app.doctorReport).toEqual(previous)
    expect(app.doctorError).toBe(true)
    expect(app.doctorLoading).toBe(false)
  })

  it('does not expose backend report before the user explicitly runs Doctor', () => {
    expect(doctorReportText(dto(), false)).toBe('')
    expect(doctorReportText(null, true)).toBe('')
  })
})
