// 只挂载更新中心，用于 WP-63 结果页的实屏对照。
// 先装只读 IPC 桩，再动态导入应用模块，避免完整应用启动期的其它依赖。
const status = {
  channel: 'stable',
  currentVersion: '0.1.9',
  availableVersion: '0.1.10',
  lastCheckedAt: '2026-10-24T00:00:00Z',
  signatureVerified: true,
  error: null,
  notes: '本版修复渠道保存链路与导出结构。',
  releaseUrl: null,
  publishedAt: '2026-10-24',
  pendingRestart: null,
}
const details = (args: { target?: string; channel?: string; candidateVersion?: string }) => ({
  target: args?.target ?? 'manager',
  channel: args?.channel ?? 'stable',
  candidateVersion: args?.candidateVersion ?? '0.1.10',
  notesState: 'ready',
  notes: '本版修复渠道保存链路与导出结构。',
  notesSource: 'updater',
  releaseUrl: null,
  fetchedAt: 1,
  publishedAt: '2026-10-24',
  cached: false,
})
const handlers: Record<string, (args?: never) => unknown> = {
  get_update_status: () => status,
  set_update_channel: () => status,
  check_for_update: () => ({ status: 'available', update: status }),
  update_release_details: (args) => details(args as never),
  official_project_info: () => ({ displayName: 'OpenCodex', version: '2.50.0', rawVersion: '2.50.0', truncated: false }),
  official_remote_latest: () => ({ tag: 'latest', version: '2.51.0', integrity: null }),
  list_notifications: () => ({ items: [], aggregate: { total: 0, unread: 0, unresolved: 0, dangerUnread: 0 } }),
  get_preferences: () => ({}),
}
;(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
  invoke: async (command: string, args?: never) => {
    const handler = handlers[command]
    if (handler) return handler(args)
    return null
  },
  transformCallback: (callback: unknown) => callback,
  convertFileSrc: (source: string) => source,
  metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
}

async function boot() {
  const [{ createApp, h }, { createPinia }, { default: UpdateCenter }, updateModule, updates, appStore] = await Promise.all([
    import('vue'),
    import('pinia'),
    import('/src/features/updates/UpdateCenter.vue'),
    import('/src/features/updates/update'),
    import('/src/features/updates/store'),
    import('/src/stores/app'),
  ])
  const pinia = createPinia()
  const app = createApp({
    render: () => h(UpdateCenter, { open: true, target: null }),
  })
  app.use(pinia)
  const updatesStore = updates.useUpdatesStore(pinia)
  updatesStore.officialProject = { displayName: 'OpenCodex', version: '2.50.0', rawVersion: '2.50.0', truncated: false }
  updatesStore.officialRemote = { tag: 'latest', version: '2.51.0', integrity: null }
  appStore.useAppStore(pinia).aboutApp = { version: '0.1.9' } as never
  updateModule.managerUpdateStatus.value = status as never
  app.mount('#harness')
}
void boot()

