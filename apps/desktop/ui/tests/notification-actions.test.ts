import { describe, expect, it } from 'vitest'
import { notificationConfirmLabel, notificationConfirmLabels } from '@/features/notifications/actions'

/**
 * 后端 `NotificationAction` 冻结为 snake_case（`types/notifications.rs` 的 DTO 契约测试
 * 断言 `payload["action"] === "app_update"`）。前端曾按 kebab-case 取名，
 * 导致 `settings-installation` 这类多词动作查不到按钮，通知详情只剩「关闭」。
 */
describe('notification action label contract', () => {
  it('covers every frozen backend action value', () => {
    const frozen = [
      'restore',
      'upgrade',
      'app_update',
      'sync',
      'logs',
      'settings_installation',
      'settings_cleanup',
    ] as const
    expect(Object.keys(notificationConfirmLabels).sort()).toEqual([...frozen].sort())
    for (const action of frozen) {
      expect(notificationConfirmLabel(action), `${action} 必须有按钮文案`).not.toBe('')
    }
  })

  it('gives the install guidance notification a way to act', () => {
    expect(notificationConfirmLabel('settings_installation')).toBe('前往设置')
    expect(notificationConfirmLabel('settings_cleanup')).toBe('前往设置')
    expect(notificationConfirmLabel('logs')).toBe('查看日志')
  })

  it('returns no label for notifications without an action', () => {
    expect(notificationConfirmLabel(undefined)).toBe('')
  })
})
