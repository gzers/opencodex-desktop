import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import type { NotificationItem } from '@/types/ui'
import { useNotificationsStore } from '@/features/notifications/store'
import { useFeedbackStore } from '@/app/feedback/store'

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
    await Promise.resolve()
    await Promise.resolve()
    expect(listNotifications).toHaveBeenCalledTimes(2)

    second.resolve({ items: [item('run-at-risk')] })
    await Promise.all([inFlight, queued])
    expect(app.notifications.map(n => n.id)).toEqual(['run-at-risk'])
  })

  it('announces the completed channel read once even if an older list was loading', async () => {
    const first = deferred<{ items: NotificationItem[] }>()
    const second = deferred<{ items: NotificationItem[] }>()
    const result: NotificationItem = { ...item('read-operation-one'), kind: 'info', category: 'system',
      action: 'models', resolved: true, title: '现有渠道读取完成', detail: '自动匹配 3 项，更新草稿 2 项。保存配置后才生效。' }
    listNotifications.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
      .mockResolvedValue({ items: [result] })
    const store = useNotificationsStore(), feedback = useFeedbackStore()
    const toast = vi.spyOn(feedback, 'showToast')
    const oldLoad = store.load(), announce = store.announce(), repeated = store.announce()
    first.resolve({ items: [] })
    await Promise.resolve(); await Promise.resolve()
    expect(toast).not.toHaveBeenCalled()
    second.resolve({ items: [result] })
    await Promise.all([oldLoad, announce, repeated])
    expect(toast).toHaveBeenCalledTimes(1)
    expect(toast).toHaveBeenCalledWith('现有渠道读取完成：'+result.detail, { durationMs: 8000, kind: undefined, resultId: undefined })
    await store.announce()
    expect(toast).toHaveBeenCalledTimes(1)
    feedback.clearToast()
  })

  it('does not replay read history on ordinary loads or turn a list failure into read success', async () => {
    const result: NotificationItem = { ...item('read-history'), kind: 'info', resolved: true, action: 'models' }
    listNotifications.mockResolvedValueOnce({ items: [result] }).mockRejectedValueOnce(new Error('list unavailable'))
      .mockResolvedValue({ items: [result] })
    const store = useNotificationsStore(), toast = vi.spyOn(useFeedbackStore(), 'showToast')
    await store.load(); expect(toast).not.toHaveBeenCalled()
    await store.announce(); expect(store.error).toBe(true); expect(toast).not.toHaveBeenCalled()
    await store.announce(); expect(store.error).toBe(false); expect(toast).not.toHaveBeenCalled()
  })
})
