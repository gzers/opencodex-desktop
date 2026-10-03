import type { NotificationItem } from '@/types/ui'

type NotificationAction = NonNullable<NotificationItem['action']>

/**
 * 通知动作 → 详情弹窗主按钮文案；空字符串表示该通知没有可执行动作。
 *
 * 取值必须与后端 `NotificationAction` 的序列化结果一致（**snake_case**，见
 * `apps/desktop/tauri/src/types/notifications.rs` 的 DTO 契约测试）。此前前端写成 kebab-case
 * （`settings-installation`），导致多词动作查不到按钮，通知详情只剩「关闭」。
 */
export const notificationConfirmLabels: Record<NotificationAction, string> = {
  restore: '前往处理',
  upgrade: '前往处理',
  app_update: '前往处理',
  sync: '前往处理',
  logs: '查看日志',
  settings_installation: '前往设置',
  settings_cleanup: '前往设置',
}

export function notificationConfirmLabel(action: NotificationItem['action']): string {
  return action ? notificationConfirmLabels[action] ?? '' : ''
}
