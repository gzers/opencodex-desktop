import { describe, expect, it } from 'vitest'
import { formatNotificationTime } from '@/features/notifications/model'

describe('notification time display', () => {
  it('formats the backend RFC 3339 timestamp as a local HH:mm clock', () => {
    const value = '2026-09-25T07:38:38.394958+00:00'
    const rendered = formatNotificationTime(value)
    expect(rendered).toMatch(/^\d{2}:\d{2}$/)
    // 展示必须与后端本地时刻一致，而不是把 ISO 原样铺出来。
    const local = new Date(value)
    const expected = `${String(local.getHours()).padStart(2, '0')}:${String(local.getMinutes()).padStart(2, '0')}`
    expect(rendered).toBe(expected)
    expect(rendered).not.toContain('T')
    expect(rendered).not.toContain('+')
  })

  it('keeps already-short values and unparseable input intact', () => {
    expect(formatNotificationTime('10:24')).toBe('10:24')
    expect(formatNotificationTime('not-a-time')).toBe('not-a-time')
  })
})
