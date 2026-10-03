import type { NotificationItem } from '@/types/ui'

export type NotificationLevel = NotificationItem['kind']
export type NotificationCategory = NotificationItem['category']

export interface NotificationAggregate {
  total: number
  unread: number
  unresolved: number
  dangerUnread: number
}

/**
 * 前端只消费同一份存活通知数组；不复制数据，不引入额外数据源。
 * 后端已剔除软删除条目，这里按「已读、已解决」两个独立维度聚合。
 */
export function aggregateNotifications(notifications: NotificationItem[]): NotificationAggregate {
  return {
    total: notifications.length,
    unread: notifications.filter(isUnread).length,
    unresolved: notifications.filter(item => !item.resolved).length,
    dangerUnread: notifications.filter(item => isUnread(item) && item.kind === 'danger').length,
  }
}

function isUnread(item: NotificationItem): boolean {
  return !item.read && !item.resolved
}

export function unreadNotifications(notifications: NotificationItem[]): NotificationItem[] {
  return notifications.filter(isUnread)
}

export function unresolvedNotifications(notifications: NotificationItem[]): NotificationItem[] {
  return notifications.filter(item => !item.resolved)
}

export function filterByCategory(notifications: NotificationItem[], category: NotificationCategory | 'all'): NotificationItem[] {
  if (category === 'all') return notifications
  return notifications.filter(item => item.category === category)
}

export function markNotificationRead(notifications: NotificationItem[], id: string): NotificationItem[] {
  return notifications.map(item => (item.id === id ? { ...item, read: true } : item))
}

export function markNotificationResolved(notifications: NotificationItem[], id: string, resolvedAt: string): NotificationItem[] {
  return notifications.map(item => (
    item.id === id ? { ...item, resolved: true, resolvedAt } : item
  ))
}

export function deleteNotification(notifications: NotificationItem[], id: string): NotificationItem[] {
  return notifications.filter(item => item.id !== id)
}

export function clearReadNotifications(notifications: NotificationItem[]): NotificationItem[] {
  return notifications.filter(item => !item.read)
}

export function clearResolvedNotifications(notifications: NotificationItem[]): NotificationItem[] {
  return notifications.filter(item => !item.resolved)
}

/**
 * 通知时间展示：后端下发 RFC 3339（UTC），界面按**本地 `HH:mm`** 展示。
 * 直接渲染原始字符串会把 `2026-09-25T07:38:38.394958+00:00` 这类值铺进列表与详情。
 * 解析失败时原样返回，不臆造时间。
 */
export function formatNotificationTime(value: string): string {
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return value
  return parsed.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
  })
}
