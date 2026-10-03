import type { NotificationItem } from '@/types/ui'

export interface NotificationAggregateDto {
  total: number
  unread: number
  unresolved: number
  dangerUnread: number
}

export interface NotificationsDto {
  items: NotificationItem[]
  aggregate: NotificationAggregateDto
}

export async function listNotifications(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('list_notifications')
}

export async function markNotificationRead(id: string): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('mark_notification_read', { id })
}

export async function markAllNotificationsRead(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('mark_all_notifications_read')
}

export async function deleteNotification(id: string): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('delete_notification', { id })
}

export async function markNotificationResolved(id: string): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('mark_notification_resolved', { id })
}

export async function clearNotifications(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('clear_notifications')
}

export async function clearReadNotifications(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('clear_read_notifications')
}

export async function clearResolvedNotifications(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('clear_resolved_notifications')
}
