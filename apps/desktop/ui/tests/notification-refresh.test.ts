import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import type { NotificationItem } from '@/types/ui'

const listNotifications = vi.fn()

vi.mock('@/features/notifications/api', () => ({
  listNotifications: (...args: unknown[]) => listNotifications(...args),
  markNotificationRead: vi.fn(),
  markAllNotificationsRead: vi.fn(),
  deleteNotification: vi.fn(),
  clearNotifications: vi.fn(),
  clearReadNotifications: vi.fn(),
  clearResolvedNotifications: vi.fn(),
  markNotificationResolved: vi.fn(),
}))

function item(id: string): NotificationItem {
  return {
    id,
    kind: 'warning',
    category: 'run',
    title: id,
    detail: '',
    time: '10:00',
    read: false,
    resolved: false,
  } as NotificationItem
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(r => { resolve = r })
  return { promise, resolve }
}

describe('notification list refresh after backend writes', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    listNotifications.mockReset()
  })

  it('queues one extra reload when a change arrives mid-flight', async () => {
    const first = deferred<{ items: NotificationItem[] }>()
    const second = deferred<{ items: NotificationItem[] }>()
    listNotifications
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise)

    const app = useAppStore()
    const inFlight = app.loadNotifications()
    // 后端在第一次拉取返回之前写了新通知并广播 → 必须排队再拉一次，不能丢。
    const queued = app.loadNotifications()
    expect(listNotifications).toHaveBeenCalledTimes(1)

    first.resolve({ items: [] })
    await inFlight
    await queued
    expect(listNotifications).toHaveBeenCalledTimes(2)

    second.resolve({ items: [item('run-at-risk')] })
    await Promise.resolve()
    await Promise.resolve()
    expect(app.notifications.map(n => n.id)).toEqual(['run-at-risk'])
  })
})
