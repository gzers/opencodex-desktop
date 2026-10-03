import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import {
  aggregateNotifications,
  clearReadNotifications,
  clearResolvedNotifications,
  deleteNotification as projectDelete,
  markNotificationRead as projectMarkRead,
  markNotificationResolved as projectMarkResolved,
  unreadNotifications,
  unresolvedNotifications,
} from '@/features/notifications/model'
import type { NotificationItem } from '@/types/ui'
import { useNotificationsStore } from '@/features/notifications/store'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<NotificationItem> = {}): NotificationItem {
  return {
    id: 'external-takeover',
    kind: 'warning',
    category: 'system',
    title: '外部 provider 已接管 Codex 配置',
    detail: '桌面壳只解释影响，不会覆盖外部配置。',
    time: '2026-09-15T10:24:00Z',
    read: false,
    action: 'restore',
    target: 'external_takeover',
    ...overrides,
  }
}

function backendItems(items: NotificationItem[]) {
  return {
    items,
    aggregate: {
      total: items.length,
      unread: items.filter(item => !item.read && !item.resolved).length,
      unresolved: items.filter(item => !item.resolved).length,
      dangerUnread: items.filter(item => !item.read && !item.resolved && item.kind === 'danger').length,
    },
  }
}

describe('notification command contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('loads backend notifications and preserves frozen camelCase DTO', async () => {
    const payload = backendItems([
      dto(),
      dto({ id: 'app-update-available', kind: 'info', action: 'app_update', target: undefined }),
    ])
    invoke.mockResolvedValue(payload)
    const app = useAppStore()
    await app.loadNotifications()
    expect(invoke).toHaveBeenCalledWith('list_notifications')
    expect(app.notifications).toEqual(payload.items)
    expect(app.notificationsError).toBe(false)
    expect(app.notificationsLoading).toBe(false)
    expect(app.notificationAggregate()).toEqual(payload.aggregate)
  })

  it('marks one and all notifications read through backend commands', async () => {
    const app = useAppStore()
    useNotificationsStore().list = [dto()]
    invoke.mockImplementation(async (command: string) => {
      if (command === 'mark_notification_read') return backendItems([dto({ read: true })])
      if (command === 'mark_all_notifications_read') return backendItems([dto({ read: true })])
      throw new Error(`unexpected command: ${command}`)
    })
    await app.markNotificationRead('external-takeover')
    expect(invoke).toHaveBeenNthCalledWith(1, 'mark_notification_read', { id: 'external-takeover' })
    expect(app.notifications[0]?.read).toBe(true)
    await app.markAllNotificationsRead()
    expect(invoke).toHaveBeenNthCalledWith(2, 'mark_all_notifications_read')
    expect(app.notificationAggregate().unread).toBe(0)
  })

  it('deletes and clears through one shared backend list', async () => {
    const app = useAppStore()
    useNotificationsStore().list = [dto(), dto({ id: 'sync-connected', action: 'sync' })]
    invoke.mockImplementation(async (command: string) => {
      if (command === 'delete_notification') return backendItems([dto({ id: 'sync-connected', action: 'sync' })])
      if (command === 'clear_notifications') return backendItems([])
      throw new Error(`unexpected command: ${command}`)
    })
    await app.deleteNotification('external-takeover')
    expect(invoke).toHaveBeenNthCalledWith(1, 'delete_notification', { id: 'external-takeover' })
    expect(app.notifications).toHaveLength(1)
    await app.clearNotifications()
    expect(invoke).toHaveBeenNthCalledWith(2, 'clear_notifications')
    expect(app.notifications).toHaveLength(0)
  })

  it('marks resolved and clears resolved through dedicated backend commands', async () => {
    const app = useAppStore()
    useNotificationsStore().list = [dto(), dto({ id: 'sync-conflict', kind: 'warning' })]
    invoke.mockImplementation(async (command: string) => {
      if (command === 'mark_notification_resolved') {
        return backendItems([dto({ resolved: true, resolvedAt: '2026-09-19T00:00:00Z' }), dto({ id: 'sync-conflict', kind: 'warning' })])
      }
      if (command === 'clear_resolved_notifications') {
        return backendItems([dto({ id: 'sync-conflict', kind: 'warning' })])
      }
      throw new Error(`unexpected command: ${command}`)
    })
    await app.markNotificationResolved('external-takeover')
    expect(invoke).toHaveBeenNthCalledWith(1, 'mark_notification_resolved', { id: 'external-takeover' })
    expect(app.notifications[0]?.resolved).toBe(true)
    expect(app.notificationAggregate().unresolved).toBe(1)
    await app.clearResolvedNotifications()
    expect(invoke).toHaveBeenNthCalledWith(2, 'clear_resolved_notifications')
    expect(app.notifications).toHaveLength(1)
  })

  it('keeps prior state on IPC failure without inventing notifications', async () => {
    const previous = [dto()]
    const app = useAppStore()
    useNotificationsStore().list = previous
    invoke.mockRejectedValue(new Error('backend unavailable'))
    await app.loadNotifications()
    await app.markNotificationRead('external-takeover')
    await app.clearReadNotifications()
    expect(app.notifications).toEqual(previous)
    expect(app.notificationsError).toBe(true)
  })
})

describe('notification projection guards', () => {
  it('aggregates FZ-43 levels without changing the backend list', () => {
    const items = [
      dto(),
      dto({ id: 'risk', kind: 'danger' }),
      dto({ id: 'read', read: true }),
    ]
    expect(aggregateNotifications(items)).toEqual({ total: 3, unread: 2, unresolved: 3, dangerUnread: 1 })
    const next = projectMarkRead(items, 'risk')
    expect(aggregateNotifications(next).unread).toBe(1)
    expect(unreadNotifications(next)).toHaveLength(1)
    expect(projectDelete(items, 'risk')).toHaveLength(2)
    expect(clearReadNotifications(items)).toHaveLength(2)
  })

  it('keeps read and resolved as independent projection dimensions', () => {
    const items = [dto({ read: true }), dto({ id: 'risk', kind: 'danger' })]
    const resolved = projectMarkResolved(items, 'risk', '2026-09-19T00:00:00Z')
    // 已读条目仍是未解决；标记已解决的条目退出未读与未解决计数。
    expect(aggregateNotifications(resolved)).toEqual({ total: 2, unread: 0, unresolved: 1, dangerUnread: 0 })
    expect(unresolvedNotifications(resolved).map(item => item.id)).toEqual(['external-takeover'])
    expect(clearResolvedNotifications(resolved).map(item => item.id)).toEqual(['external-takeover'])
  })
})
