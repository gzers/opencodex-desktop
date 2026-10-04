/**
 * 开发/审计专用的显式测试适配。
 *
 * 只在 `import.meta.env.DEV` 且 URL 带 `?__audit=1` 时安装；正式构建不会包含它，
 * 也不连接真实写入端点。用途：为没有 Tauri 的 Web 页面提供可重复的结构与视觉场景，
 * 避免把真实 IPC 失败静默当成 mock 成功。
 */
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { useRouteStore } from '@/stores/routes'
import { useThemeStore } from '@/app/appearance/theme'
import { useEffectsStore } from '@/app/appearance/effects'
import { useNotificationsStore } from '@/features/notifications/store'
import { usePreferencesStore } from '@/features/preferences/store'
import { useFeedbackStore } from '@/app/feedback/store'
import { useExtensionsStore } from '@/features/extensions/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { nextTick } from 'vue'
import type { NotificationItem, RouteName } from '@/types/ui'
import type { PreferencesDto } from '@/features/preferences/api'
import type { RuntimeState, StatusSnapshot } from '@/contracts/runtimeStatus'
import type { EnvironmentReport } from '@/features/environment/api'

// 概览双形态的显式夹具：`?env=` 给「环境准备 / 就绪」一个可重复输入（只写内存，不连真实端点）。
function environmentReport(gate: EnvironmentReport['gate'], brewFound = true): EnvironmentReport {
  const found = { found: true, path: '/fixtures/bin', version: null }
  const missing = { found: false, path: null, version: null }
  switch (gate) {
    case 'ready':
      return { node: { ...found }, npm: { ...found }, ocx: { ...found }, gate: 'ready', shortCircuited: false, brewFound }
    case 'missing_node':
      return { node: { ...missing }, npm: { ...missing }, ocx: { ...missing }, gate: 'missing_node', shortCircuited: true, brewFound }
    case 'missing_npm':
      return { node: { ...found }, npm: { ...missing }, ocx: { ...missing }, gate: 'missing_npm', shortCircuited: true, brewFound }
    case 'missing_ocx':
      return { node: { ...found }, npm: { ...found }, ocx: { ...missing }, gate: 'missing_ocx', shortCircuited: true, brewFound }
  }
}

// 概览状态形象的可重复观测夹具：只写内存快照，不连接真实端点。
function overviewSnapshot(runtime: RuntimeState, source: StatusSnapshot['source'] = 'fixture'): StatusSnapshot {
  // `at_risk` 由「代理未在运行 + startup at-risk」折叠而来，快照按**未运行**给（无端口/进程）。
  const running = runtime === 'running'
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
    facts: {
      health: running ? 'healthy' : runtime === 'unreachable' ? 'unhealthy' : 'unknown',
      runtime_label: running ? 'codex' : null,
      opencodex_home: '/fixtures/opencodex-home',
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: '1.4.2',
    },
    port: running ? 10100 : null,
    pid: running ? '39421' : null,
    can_start: !running,
    can_stop: running,
    can_restart: running,
    source,
  }
}

type AppStore = ReturnType<typeof useAppStore>

/**
 * Web 审计没有 Tauri 后端，偏好保存必然失败；`&prefs=local` 时安装一个显式的本地偏好适配，
 * 让「保存成功」分支可重复观察。它只改内存状态，不连接真实端点，也不写磁盘。
 */
const AUDIT_PREFERENCES: PreferencesDto = {
  schemaVersion: 1,
  interfaceScale: 100,
  launchMain: true,
  autoPanel: true,
  panelMode: 'embedded',
  keepProxyOnClose: true,
  lifecycleNotifications: true,
  syncConflictAlerts: true,
  launchWithCodex: true,
  autoBackupUpgrade: true,
  autoBackupImport: true,
  autoBackupSync: true,
  backupRetention: '10',
  backupIntegrity: 'sha-256',
  backupIncludeSkills: true,
  exportIncludeSkills: true,
  mcpConflictPolicy: 'ask',
  mcpMask: true,
  backupIncludeMcp: true,
  exportIncludeMcp: true,
  logRetention: '30d-10000',
  notificationRetention: '30',
  startupCleanup: true,
  cleanupBackupSummary: true,
  cliEnabled: false,
  syncConflictPolicy: 'ask',
  coldSync: true,
  backupBeforeOverwrite: true,
  appUpdateChannel: 'stable',
  appUpdateAutoCheck: true,
  appUpdateCheckIntervalSeconds: 86400,
  theme: 'system',
  visualEffects: 'high',
  glowRender: 'mesh',
}

function installLocalPreferences(prefs: ReturnType<typeof usePreferencesStore>) {
  prefs.data = { ...AUDIT_PREFERENCES }
  prefs.load = async () => {
    // 每次返回新的对象引用，让依赖 `app.preferences` 的页面 watcher 能同步到本地适配值。
    prefs.data = { ...AUDIT_PREFERENCES, ...(prefs.data ?? {}) }
    prefs.error = false
  }
  prefs.save = async (next: PreferencesDto) => {
    prefs.data = { ...next }
    return true
  }
  prefs.restore = async () => {
    prefs.data = { ...AUDIT_PREFERENCES }
    return true
  }
}

const LONG_TEXT =
  '这是一条用于审计的超长通知正文：' + '当远端返回的错误摘要很长、又没有换行时，界面必须能够换行收敛而不是把卡片撑宽。'.repeat(4)

/**
 * Markdown 渲染耗时测量夹具（IMP-04 §19.12）：一段远大于常见 SKILL.md 的受限 Markdown，
 * 覆盖标题、段落、列表与内联标记，用于测量详情弹窗正文的渲染成本。仅前端展示，不含真实正文。
 */
const MARKDOWN_DOC = [
  '# 审计文档渲染夹具',
  '',
  '这段正文只用于测量受限 Markdown 的渲染耗时，不代表任何真实 Skill 的内容。',
  '',
  ...Array.from({ length: 48 }, (_, i) =>
    [
      `## 小节 ${i}`,
      '',
      `段落 ${i}：` + '渲染上限、转义与链接契约需要在长文下保持稳定，不能因体量增长而破坏。'.repeat(3),
      '',
      `- 要点 A${i}`,
      `- 要点 B${i}：**加粗**与 \`code\` 混排`,
      `- 要点 C${i}`,
      '',
    ].join('\n'),
  ),
].join('\n')

/**
 * Web 审计的显式场景预设：成功 / 失败 / 慢响应 / 超时 / 取消 / 乱序 / 重复 / 空数据 / 长文本。
 * 全部只改前端展示状态，不写真实资源，也不把 IPC 失败当成 mock 成功。
 */
function applyScenario(name: string | null, app: AppStore, notif: ReturnType<typeof useNotificationsStore>) {
  if (!name) return
  const feedback = useFeedbackStore()
  switch (name) {
    case 'empty':
      notif.list = []
      notif.panelOpen = true
      break
    case 'longtext':
      notif.list = [
        { id: 'long-title', kind: 'danger', category: 'run', title: '启动失败：' + '端口不可用与配置冲突的复合原因说明'.repeat(3), detail: LONG_TEXT, time: '2026-09-19T10:00:00Z', read: false },
        { id: 'long-body', kind: 'warning', category: 'sync', title: '同步冲突', detail: LONG_TEXT, time: '2026-09-19T10:01:00Z', read: true, resolved: true, resolvedAt: '2026-09-19T10:02:00Z' },
      ]
      notif.panelOpen = true
      feedback.toast = '错误摘要：' + LONG_TEXT
      break
    case 'manynotif':
      // 长列表渲染测量（IMP-04 §19.12）：200 条同构通知，验证列表渲染耗时可控、无横向溢出。
      notif.list = Array.from({ length: 200 }, (_, i) => ({
        id: `bulk-${i}`,
        kind: (['info', 'warning', 'danger'] as const)[i % 3],
        category: (['run', 'sync', 'update', 'system'] as const)[i % 4],
        title: `批量通知 ${i}（夹具）`,
        detail: '长列表渲染夹具：用于测量通知面板的渲染耗时，不代表真实业务结果。',
        time: '2026-09-19T10:24:00Z',
        read: i % 2 === 0,
      }))
      break
    case 'markdown':
      // Markdown 渲染耗时测量（IMP-04 §19.12）：一条带长 Markdown 描述的 Skill，
      // 供探针点开详情弹窗后计时正文渲染（详情 IPC 读取在 Web 审计下失败，不阻塞弹窗）。
      useExtensionsStore().discovered = {
        skills: [
          {
            id: 'md-skill',
            name: '审计文档渲染夹具',
            source: 'agents',
            updatedAt: null,
            description: MARKDOWN_DOC,
            clients: ['codex', 'claude'],
          },
        ],
        servers: [],
      }
      break
    case 'duplicate':
      // 前端只渲染后端投影；这里验证同键重复不会让展示层崩坏。
      notif.list = [
        { id: 'dup-a', kind: 'info', category: 'sync', title: '同步完成', detail: '第一次到达', time: '2026-09-19T10:00:00Z', read: false, dedupeKey: 'sync:ok' },
        { id: 'dup-b', kind: 'info', category: 'sync', title: '同步完成', detail: '重复到达', time: '2026-09-19T10:00:01Z', read: false, dedupeKey: 'sync:ok' },
      ]
      notif.panelOpen = true
      break
    case 'late':
      // 旧操作的晚到终态：界面只呈现已收到的事实，不臆测新操作成功。
      feedback.recentEvents = []
      useLifecycleStore().processProgressAction = 'restart'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processProgressCompleted = true
      feedback.toast = '重启已完成（旧操作晚到终态，仅记录事实）'
      break
    case 'task-busy':
      useLifecycleStore().processProgressAction = 'start'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processActionBusy = true
      break
    case 'task-success':
      useLifecycleStore().processProgressAction = 'start'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processProgressCompleted = true
      break
    case 'task-failure':
      useLifecycleStore().processProgressAction = 'start'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processProgressError = 'OpenCodex 未能进入可用状态；可以查看日志或重试启动。'
      break
    case 'task-timeout':
      useLifecycleStore().processProgressAction = 'start'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processProgressTimedOut = true
      useLifecycleStore().processProgressError = '等待时间较长，后台仍会继续观察状态；可以先关闭此窗口。'
      break
    case 'task-cancelled':
      useLifecycleStore().processProgressAction = 'stop'
      useLifecycleStore().processProgressOpen = true
      useLifecycleStore().processProgressError = '停止请求失败；已保留当前状态。'
      break
    case 'fault':
      // 运行期异常兜底场景（IMP-04 §14.3）：只看故障条与恢复动作，不模拟真实崩溃。
      app.reportUiFault('界面异常：TypeError: 审计场景触发的示例故障', 'render')
      break
  }
}

function seedNotifications(): NotificationItem[] {
  const base = {
    category: 'run' as const,
    detail: '这是审计夹具正文，用于核对通知中心的结构与视觉，不代表真实业务结果。',
    time: '2026-09-19T10:24:00Z',
    read: false,
  }
  return [
    { ...base, id: 'audit-danger', kind: 'danger', category: 'run', title: '启动失败（夹具）' },
    { ...base, id: 'audit-warning', kind: 'warning', category: 'sync', title: '同步冲突（夹具）', read: true },
    { ...base, id: 'audit-info', kind: 'info', category: 'update', title: '有可用更新（夹具）' },
    { ...base, id: 'audit-resolved', kind: 'info', category: 'system', title: '自动清理完成（夹具）', read: true, resolved: true, resolvedAt: '2026-09-19T10:24:00Z' },
  ]
}

// 审计参数只解析一次；`collectReport` 里的 `scale` 探针需要回读原始入参。
let auditParams = new URLSearchParams()

export async function installAuditHarness() {
  const params = new URLSearchParams(window.location.search)
  auditParams = params
  if (!params.has('__audit')) return

  const app = useAppStore()
  const routes = useRouteStore()
  const theme = useThemeStore()
  const notif = useNotificationsStore()
  const prefs = usePreferencesStore()

  const requestedTheme = params.get('theme')
  if (requestedTheme === 'light' || requestedTheme === 'dark') theme.apply(requestedTheme)

  // `effects=<high|mid|low>`：给三档外观策略一个可重复的档位输入。
  // 系统「减少动态」由 Playwright `emulateMedia` 驱动，与本参数正交。
  const requestedEffects = params.get('effects')
  if (requestedEffects) useEffectsStore().setSetting(requestedEffects)

  // 显式本地偏好适配：只让「保存成功」分支可重复，不连接真实端点。
  if (params.get('prefs') === 'local') installLocalPreferences(prefs)

  const requestedRoute = params.get('route')
  if (requestedRoute) {
    const valid: RouteName[] = ['overview', 'panel', 'extensions', 'logs', 'settings', 'tray']
    if (valid.includes(requestedRoute as RouteName)) {
      const next = requestedRoute as RouteName
      const routeParams: Record<string, string> = {}
      const tab = params.get('tab')
      const section = params.get('section')
      if (tab) routeParams.tab = tab
      if (section) routeParams.section = section
      routes.go(next, routeParams)
    }
  }

  // `runtime=stub`：给「设置 · 安装配置」的运行来源卡片与卸载弹窗一个可重复的本地适配。
  // 只改内存状态，不连接真实端点，也不执行任何删除。
  if (params.get('runtime') === 'stub') installRuntimeStub(app)

  // `runtime=<state>`：给概览状态形象一个可重复的运行主线（只写内存快照）。
  const requestedRuntime = params.get('runtime')
  const runtimeStates: RuntimeState[] = [
    'loading',
    'not_found',
    'stopped',
    'starting',
    'pending',
    'running',
    'stopping',
    'starting_failed',
    'at_risk',
    'external_takeover',
    'unreachable',
  ]
  if (requestedRuntime && requestedRuntime !== 'stub' && (runtimeStates as string[]).includes(requestedRuntime)) {
    app.setRuntimeState(requestedRuntime as RuntimeState)
    // `stale=1`：模拟「此前真实观测到、但最近一次采集失败」——用于验证「事实过期」接线。
    const stale = params.get('stale') === '1'
    app.setStatusSnapshot(overviewSnapshot(requestedRuntime as RuntimeState, stale ? 'live' : 'fixture'))
    if (stale) useLifecycleStore().statusError = true
  }

  // `env=<ready|missing_node|missing_npm|missing_ocx|checking>`：概览双形态的可重复输入。
  const requestedEnv = params.get('env')
  if (requestedEnv === 'checking') {
    app.setEnvironmentLoading(true)
  } else if (requestedEnv) {
    const gates: EnvironmentReport['gate'][] = ['ready', 'missing_node', 'missing_npm', 'missing_ocx']
    if ((gates as string[]).includes(requestedEnv)) {
      app.setEnvironmentLoading(false)
      app.setEnvironment(environmentReport(requestedEnv as EnvironmentReport['gate'], params.get('brew') !== '0'))
    }
  }

  notif.list = seedNotifications()

  // 路由切换会卸载上一个路由的顶栏（其卸载钩子会收起通知面板），
  // 因此浮层状态必须在路由稳定之后再赋值。
  await nextTick()

  if (params.get('panel') === '1') notif.panelOpen = true
  // 直接赋值，不触发自动关闭计时，保证截图可重复。
  if (params.get('toast') === '1') useFeedbackStore().toast = '偏好已保存（夹具）'
  if (params.get('task') === '1') {
    useLifecycleStore().processProgressAction = 'start'
    useLifecycleStore().processProgressOpen = true
    useLifecycleStore().processActionBusy = true
  }

  applyScenario(params.get('scenario'), app, notif)
  window.document.documentElement.dataset.auditHarness = 'installed'

  if (params.get('report') === '1') {
    // 审计装置与启动装配并发：报告必须在应用真正挂载后再采集，
    // 否则会抓到空 DOM（结构/几何全为空），把「没渲染」误当成「通过」（IMP-04 §17.1）。
    await whenAppMounted()
    await nextTick()
    window.setTimeout(emitReport, 0)
  }

  if (params.get('text') === '1') {
    // 内容对照用：输出当前路由下可见文案的规范化行集合。
    await whenAppMounted()
    await nextTick()
    window.setTimeout(emitText, 0)
  }
}

// 等待应用壳挂载（`#app .app-window`），上限 8s，避免审计报告先于首帧。
async function whenAppMounted(timeoutMs = 8000): Promise<void> {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    if (document.querySelector('#app .app-window')) return
    await new Promise(resolve => window.setTimeout(resolve, 50))
  }
}

function lines(selector: string): string[] {
  const element = document.querySelector(selector)
  if (!element) return []
  return (element as HTMLElement).innerText
    .split('\n')
    .map(line => line.replace(/\s+/g, ' ').trim())
    .filter(line => line.length > 0)
}

function labels(selector: string): string[] {
  return Array.from(document.querySelectorAll(selector)).map(node => (node.textContent ?? '').replace(/\s+/g, ' ').trim())
}

function emitText() {
  const report = {
    route: document.documentElement.dataset.route ?? null,
    settingsTabs: labels('.settings-tabs button'),
    activeSettings: lines('.settings-panel.active'),
    settingsControls: settingsControls(),
    extTabs: labels('.ext-tabs button'),
    activeExt: lines('.ext-view.active'),
    diagTabs: labels('.diag-tabs button'),
    notifications: lines('.history-list'),
  }
  const node = document.createElement('pre')
  node.id = 'audit-text'
  node.textContent = JSON.stringify(report, null, 2)
  document.body.appendChild(node)
  document.documentElement.dataset.auditText = 'ready'
}

/** 逐行的字段级对照：标题 + 控件类型（含 select 选项 / range 取值范围）。 */
function settingsControls() {
  return Array.from(document.querySelectorAll('.settings-panel.active .setting-row')).map(row => {
    const title = (row.querySelector('.setting-title')?.textContent ?? '').replace(/\s+/g, ' ').trim()
    const controls = row.querySelector('.controls')
    if (!controls) return { title, kind: 'none' as const }
    const options = Array.from(controls.querySelectorAll('.select-option'))
      .map(node => (node.textContent ?? '').trim())
      .filter(Boolean)
    if (options.length) return { title, kind: 'select' as const, options }
    if (controls.querySelector('.toggle,[role="switch"]')) return { title, kind: 'switch' as const }
    const range = controls.querySelector<HTMLInputElement>('input[type="range"]')
    if (range) {
      return {
        title,
        kind: 'range' as const,
        min: range.min, max: range.max, step: range.step,
      }
    }
    const input = controls.querySelector<HTMLInputElement>('input')
    if (input) return { title, kind: 'input' as const, type: input.type }
    const text = (controls.textContent ?? '').replace(/\s+/g, ' ').trim()
    return { title, kind: 'buttons' as const, labels: text ? text.split(' ').filter(Boolean) : [] }
  })
}

async function emitReport() {
  // `scale` 探针要等保存分支落地，因此报告本身是异步生成的。
  const scaleProbe = await runScaleProbe()
  const node = document.createElement('pre')
  node.id = 'audit-report'
  node.textContent = JSON.stringify({ ...collectReport(), scaleProbe }, null, 2)
  document.body.appendChild(node)
  document.documentElement.dataset.auditReport = 'ready'
}

function style(selector: string, props: string[]): Record<string, string> | null {
  const element = document.querySelector(selector)
  if (!element) return null
  const computed = window.getComputedStyle(element)
  const result: Record<string, string> = {}
  props.forEach(prop => { result[prop] = computed.getPropertyValue(prop) })
  return result
}

function leftOf(selector: string): number | null {
  const element = document.querySelector(selector)
  if (!element) return null
  return Math.round(element.getBoundingClientRect().left)
}

function customProp(selector: string, prop: string): string {
  const element = document.querySelector(selector)
  if (!element) return ''
  return window.getComputedStyle(element).getPropertyValue(prop).trim()
}

function collectReport() {
  const root = window.getComputedStyle(document.documentElement)
  return {
    theme: document.documentElement.getAttribute('data-theme'),
    effects: document.documentElement.getAttribute('data-effects'),
    effectsSurface: {
      mainBg: style('.main', ['background-color'])?.['background-color'] ?? '',
      toastBlur: style('.notif-host .toast', ['backdrop-filter'])?.['backdrop-filter'] ?? '',
      panelBlur: style('.notification-panel', ['backdrop-filter'])?.['backdrop-filter'] ?? '',
      cardBlur: style('.card', ['backdrop-filter'])?.['backdrop-filter'] ?? '',
      buttonBlur: style('.btn', ['backdrop-filter'])?.['backdrop-filter'] ?? '',
      cardBg: style('.card', ['background-color'])?.['background-color'] ?? '',
      effectsSurfaceAmbient: customProp('.app-window', '--surface-ambient-display'),
    },
    window: style('.app-window', ['background-color']),
    sidebar: style('.side', ['background-color', 'backdrop-filter', 'padding-top']),
    titlebar: style('.titlebar', ['background-color', 'backdrop-filter']),
    main: style('.main', ['background-color', 'border-top-width', 'border-left-width', 'border-top-left-radius']),
    scrollbar: style('.main', ['scrollbar-width', 'scrollbar-color']),
    tokens: {
      glassFill: root.getPropertyValue('--glass-fill').trim(),
      glassBlurNotif: root.getPropertyValue('--glass-blur-notif').trim(),
      notifCtlH: root.getPropertyValue('--notif-ctl-h').trim(),
      notifMark: root.getPropertyValue('--notif-mark').trim(),
    },
    iconColumns: {
      brand: leftOf('.brand .brand-mark'),
      navIcon: leftOf('.nav svg'),
    },
    notificationCenter: {
      storeFlagOpen: useNotificationsStore().panelOpen,
      root: style('.notification-panel', ['display', 'width', 'max-height', 'background-color', 'backdrop-filter']),
      items: document.querySelectorAll('.notification-panel .notification-item').length,
      resolvedItems: document.querySelectorAll('.notification-panel .notification-item.resolved').length,
      tags: Array.from(document.querySelectorAll('.notification-panel .n-tag')).map(node => node.textContent?.trim()),
      counts: Array.from(document.querySelectorAll('.notification-panel .n-count')).map(node => node.textContent?.trim()),
      activeTab: style('.notification-cats button.active', ['background-color', 'border-bottom-width', 'border-bottom-color', 'border-top-width']),
      trashCenterDelta: trashCenterDelta(),
    },
    toast: style('.notif-host .toast', ['background-color', 'display']),
    taskCard: style('.process-progress', ['background-color', 'display']),
    surfaces: {
      toastText: (document.querySelector('.notif-host .toast')?.textContent ?? '').replace(/\s+/g, ' ').trim(),
      taskTitle: (document.querySelector('.process-progress h3')?.textContent ?? '').trim(),
      taskMessage: (document.querySelector('.process-progress p')?.textContent ?? '').trim(),
      taskSteps: Array.from(document.querySelectorAll('.process-progress-step')).map(node => `${node.className.replace('process-progress-step ', '')}:${(node.textContent ?? '').trim()}`),
      panelEmpty: (document.querySelector('.notification-panel .notification-empty')?.textContent ?? '').trim(),
    },
    layerLayout: layerLayout(),
    overflow: overflowReport(),
    historyItems: document.querySelectorAll('.history-list .notification-item').length,
    rail: { sideW: customProp('.desktop-stage', '--side-w') },
  }
}

/**
 * 界面缩放入参探针：把 `&scale=<原始输入>` 写进数字读数并派发 change，
 * 回读归一化后的输入框值与生效缩放因子 `--ui-zoom`，用于对照「越界/空值是否回写」。
 */
async function runScaleProbe() {
  const raw = auditParams.get('scale')
  if (raw === null) return null
  const input = document.querySelector<HTMLInputElement>('.range .range-value')
  if (!input) return null
  input.value = raw
  input.dispatchEvent(new Event('change', { bubbles: true }))
  // 归一化回写发生在保存分支之后（成功与失败都要回写到生效值），等它落到 DOM。
  await nextTick()
  await new Promise(resolve => window.setTimeout(resolve, 250))
  return {
    raw,
    normalizedValue: input.value,
    sliderValue: document.querySelector<HTMLInputElement>('.range input[type="range"]')?.value ?? null,
    uiZoom: getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom').trim(),
  }
}

/** 窗口与内容区是否出现横向溢出（长文本/边界挤压）。 */
function overflowReport() {
  const main = document.querySelector('.main')
  const docEl = document.documentElement
  return {
    viewport: { width: window.innerWidth, height: window.innerHeight },
    docHorizontalOverflow: docEl.scrollWidth - docEl.clientWidth,
    docVerticalOverflow: docEl.scrollHeight - docEl.clientHeight,
    mainHorizontalOverflow: main ? main.scrollWidth - main.clientWidth : null,
    mainScrollable: main ? main.scrollHeight > main.clientHeight : null,
  }
}

function rectOf(selector: string) {
  const element = document.querySelector(selector)
  if (!element) return null
  const rect = element.getBoundingClientRect()
  return {
    top: Math.round(rect.top), left: Math.round(rect.left),
    width: Math.round(rect.width), height: Math.round(rect.height),
    bottom: Math.round(rect.bottom), right: Math.round(rect.right),
  }
}

type Rect = NonNullable<ReturnType<typeof rectOf>>

function overlaps(a: Rect, b: Rect): boolean {
  return a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/** 通知层内三个表面必须顺次避让、且在视口内。 */
function layerLayout() {
  const panel = rectOf('.notification-panel.show')
  const toast = rectOf('.notif-host .toast')
  const task = rectOf('.process-progress')
  const surfaces = [panel, toast, task].filter((item): item is Rect => item !== null)
  let overlap = false
  for (let i = 0; i < surfaces.length; i += 1) {
    for (let j = i + 1; j < surfaces.length; j += 1) {
      if (overlaps(surfaces[i], surfaces[j])) overlap = true
    }
  }
  return {
    panel, toast, task,
    overlap,
    withinViewport: surfaces.every(item =>
      item.top >= 0 && item.left >= 0 &&
      item.bottom <= window.innerHeight && item.right <= window.innerWidth),
  }
}

/** 垃圾桶图标中心相对按钮中心的偏移，应为 (0,0)。 */
function trashCenterDelta(): { dx: number; dy: number } | null {
  const button = document.querySelector('.notification-panel .n-del')
  const svg = button?.querySelector('svg')
  if (!button || !svg) return null
  const a = button.getBoundingClientRect()
  const b = svg.getBoundingClientRect()
  return {
    dx: Math.round((b.left + b.width / 2) - (a.left + a.width / 2)),
    dy: Math.round((b.top + b.height / 2) - (a.top + a.height / 2)),
  }
}

/**
 * 卸载弹窗审计夹具（`runtime=stub`）。
 *
 * 场景固定为「自动发现到的 npm 全局安装」：入口常显「卸载」、方案里给出将执行的固定命令、
 * 未检测到可复用备份（因此默认勾选自动备份）；确认后返回一组真实形状的结果供渲染断言。
 */
function installRuntimeStub(app: AppStore) {
  const runtime = useRuntimeStore()
  runtime.source = {
    kind: 'discovered',
    path: '/home/.local/bin/ocx',
    version: '2.64.0',
    resolvedAt: '2026-09-28T00:00:00Z',
    insideDataRoot: false,
    managedEntry: '/data/runtime/bin/ocx',
    managedPrefix: '/data/runtime/opencodex',
    defaultPrefix: '/data/runtime/opencodex',
    explicitPath: null,
    history: [],
  }
  runtime.sourceLoading = false
  runtime.sourceError = ''
  app.loadRuntimeSource = async () => true
  app.loadUninstallPlan = async () => {
    runtime.uninstallPlan = {
      sourceKind: 'discovered',
      sourcePath: '/home/.local/bin/ocx',
      managed: false,
      backupId: null,
      removable: true,
      reason: null,
      removeObjects: ['npm 全局包 @bitkyc08/opencodex', '/home/.local/bin/ocx'],
      runtimeObjects: ['service · 官方 launchd/systemd/WinSW 注册'],
      dataObjects: ['/home/.opencodex/config.json'],
      residueCandidates: ['/home/.opencodex/routing-history.sqlite'],
      officialCommand: "'/home/.local/bin/ocx' uninstall",
      externalCommand: 'npm uninstall -g @bitkyc08/opencodex',
    }
    runtime.uninstallPlanLoading = false
    runtime.uninstallPlanError = ''
    return true
  }
  app.uninstallRuntime = async () => {
    runtime.uninstallResult = {
      scope: 'full',
      sourceKind: 'unresolved',
      sourcePath: null,
      backupId: 'bk_20260928000000_deadbeef',
      backupDirectory: '/data/backups/2026/09/runtime-uninstall/bk_x',
      steps: [
        { name: '移除包体与入口', status: 'ok', detail: 'npm uninstall -g @bitkyc08/opencodex' },
        { name: '执行官方 ocx uninstall', status: 'ok', detail: '官方已清理 service / shim / config' },
      ],
      residue: [
        { path: '/home/.local/bin/ocx', status: 'cleared' },
        { path: '/home/Library/LaunchAgents/com.opencodex.proxy.plist', status: 'cleared' },
      ],
      officialOutput: ['✅ service removed', '✅ native Codex restored'],
      needsRestart: false,
      message: '已按「完整卸载」卸载',
    }
    return true
  }
}
