// 通知功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出）。
// 根 store 通过 getter/委托保留 `app.notifications*` 读取与动作调用；直接写入点改用本 store。
import { defineStore } from 'pinia'
import type { NotificationItem } from '@/types/ui'
import {
  clearNotifications as clearNotificationsCommand,
  clearReadNotifications as clearReadNotificationsCommand,
  clearResolvedNotifications as clearResolvedNotificationsCommand,
  deleteNotification as deleteNotificationCommand,
  listNotifications,
  markAllNotificationsRead as markAllNotificationsReadCommand,
  markNotificationRead as markNotificationReadCommand,
  markNotificationResolved as markNotificationResolvedCommand,
} from './api'
import { aggregateNotifications } from './model'
import { cleanupNotifications as cleanupNotificationsCommand } from '@/platform/workspace'

export const useNotificationsStore = defineStore('notifications', {
  state: () => ({
    // 通知中心面板是否展开（全局通知层消费）。
    panelOpen: false,
    list: [] as NotificationItem[],
    loading: false,
    // 后端写入通知后广播事件，前端据此重拉；加载中收到广播时排队再拉一次，
    // 避免「事件先到、响应是旧的」把新通知丢掉。
    reloadQueued: false,
    error: false,
    stopEventStream: null as (() => void) | null,
  }),
  getters: {
    aggregate: state => aggregateNotifications(state.list),
  },
  actions: {
    /**
     * 后端异步写入通知（状态观测、同步冲突等）后广播 `notifications-changed`；
     * 前端据此重拉同一份列表，避免通知中心停留在启动时的旧快照。
     */
    async startEventStream() {
      if (this.stopEventStream) return this.stopEventStream
      try {
        const { listen } = await import('@tauri-apps/api/event')
        const unlisten = await listen('notifications-changed', () => {
          void this.load()
        })
        this.stopEventStream = () => {
          void unlisten()
          this.stopEventStream = null
        }
        return this.stopEventStream
      } catch {
        return null
      }
    },
    async load() {
      if (this.loading) {
        this.reloadQueued = true
        return
      }
      this.loading = true
      try {
        const result = await listNotifications()
        this.list = result.items
        this.error = false
      } catch {
        this.error = true
      } finally {
        this.loading = false
        if (this.reloadQueued) {
          this.reloadQueued = false
          void this.load()
        }
      }
    },
    async markRead(id: string) {
      try {
        this.list = (await markNotificationReadCommand(id)).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async markAllRead() {
      try {
        this.list = (await markAllNotificationsReadCommand()).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async remove(id: string) {
      try {
        this.list = (await deleteNotificationCommand(id)).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async clearAll() {
      try {
        this.list = (await clearNotificationsCommand()).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async clearRead() {
      try {
        this.list = (await clearReadNotificationsCommand()).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async markResolved(id: string) {
      try {
        this.list = (await markNotificationResolvedCommand(id)).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    async clearResolved() {
      try {
        this.list = (await clearResolvedNotificationsCommand()).items
        this.error = false
      } catch {
        this.error = true
      }
    },
    // 按保留策略清理（平台命令），返回是否成功。
    async cleanupByRetention() {
      try {
        const result = await cleanupNotificationsCommand()
        this.list = result.items
        this.error = false
        return true
      } catch {
        this.error = true
        return false
      }
    },
  },
})
