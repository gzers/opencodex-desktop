import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'
import type { NotificationItem } from '@/types/ui'
import { useNotificationsStore } from '@/features/notifications/store'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: query === '(prefers-color-scheme: dark)',
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function item(overrides: Partial<NotificationItem>): NotificationItem {
  return {
    id: 'n',
    kind: 'warning',
    category: 'run',
    title: '标题',
    detail: '正文',
    time: '2026-09-19T10:00:00Z',
    read: false,
    ...overrides,
  }
}

function mountApp() {
  return mount(App, { global: { plugins: [createPinia()] } })
}

describe('notification center', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
  })

  it('shows every live notification by default, not only unread ones', async () => {
    const wrapper = mountApp()
    useNotificationsStore().list = [
      item({ id: 'unread', read: false }),
      item({ id: 'read', read: true }),
      item({ id: 'resolved', read: true, resolved: true }),
    ]
    useNotificationsStore().panelOpen = true
    await wrapper.vm.$nextTick()

    const rendered = wrapper.findAll('.notification-panel .notification-item')
    expect(rendered).toHaveLength(3)
    expect(wrapper.find('.notification-panel').classes()).toContain('show')
  })

  it('distinguishes read from resolved with an inline status tag', async () => {
    const wrapper = mountApp()
    useNotificationsStore().list = [
      item({ id: 'unresolved-read', read: true }),
      item({ id: 'done', read: true, resolved: true }),
    ]
    useNotificationsStore().panelOpen = true
    await wrapper.vm.$nextTick()

    const tags = wrapper.findAll('.notification-panel .n-tag').map(node => node.text())
    expect(tags).toContain('未解决')
    expect(tags).toContain('已解决')
    expect(wrapper.findAll('.notification-panel .notification-item.resolved')).toHaveLength(1)
  })

  it('renders neutral per-category counts instead of an unread-only badge', async () => {
    const wrapper = mountApp()
    useNotificationsStore().list = [
      item({ id: 'run-read', category: 'run', read: true }),
      item({ id: 'run-unread', category: 'run' }),
      item({ id: 'sync', category: 'sync' }),
    ]
    useNotificationsStore().panelOpen = true
    await wrapper.vm.$nextTick()

    const counts = wrapper.findAll('.notification-panel .notification-cats .n-count').map(node => node.text())
    expect(counts).toEqual(['3', '2', '1', '0', '0'])
  })

  it('collapses the panel to the bell without touching notification state', async () => {
    const wrapper = mountApp()
    useNotificationsStore().list = [item({ id: 'keep', read: false })]
    useNotificationsStore().panelOpen = true
    await wrapper.vm.$nextTick()

    await wrapper.find('.notification-panel .notify-min').trigger('click')
    expect(useNotificationsStore().panelOpen).toBe(false)
    // 收起只隐藏展示，不改动 read / resolved 状态。
    expect(useNotificationsStore().list[0]?.read).toBe(false)
    expect(useNotificationsStore().list[0]?.resolved).toBeFalsy()
  })
})

describe('diagnostics notification history', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
  })

  it('reuses the shared item component and shows counts', async () => {
    const wrapper = mountApp()
    const routes = useRouteStore()
    useNotificationsStore().list = [
      item({ id: 'a', read: false }),
      item({ id: 'b', read: true }),
      item({ id: 'c', read: true, resolved: true }),
    ]
    routes.go('logs', { tab: 'notifications' })
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.history-list .notification-item')).toHaveLength(3)
    expect(wrapper.find('.history-meta').text()).toContain('未读 1')
    expect(wrapper.find('.history-meta').text()).toContain('未解决 2')
  })

  it('defaults cleanup to resolved-only and warns when unresolved items exist', async () => {
    const wrapper = mountApp()
    const routes = useRouteStore()
    const app = useAppStore()
    useNotificationsStore().list = [item({ id: 'pending' }), item({ id: 'done', resolved: true, read: true })]
    routes.go('logs', { tab: 'notifications' })
    await wrapper.vm.$nextTick()

    const button = wrapper.findAll('.toolbar .btn').find(node => node.text().includes('清理已解决'))
    expect(button).toBeTruthy()
    await button!.trigger('click')
    // 存在未处理通知时改文案，不直接清空。
    expect(app.modal?.title).toContain('还有 1 条未处理通知')
  })
})
