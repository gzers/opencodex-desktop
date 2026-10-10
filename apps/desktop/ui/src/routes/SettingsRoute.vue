<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import AppTopbar from '@/components/AppTopbar.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import SettingRow from '@/components/patterns/SettingRow.vue'
import { useAppStore } from '@/stores/app'
import { useRuntimeStore } from '@/features/runtime/store'
import { buildEnvironmentPresentation } from '@/features/environment/presentation'
import { useRouteStore } from '@/stores/routes'
import { settingsSections } from '@/navigation'
import { checkNetworkProxy, type PreferencesDto } from '@/features/preferences/api'
import { getCodexShimStatus, setCodexShim, type CodexShimDto } from '@/features/codex-shim/api'
import { getUpdateStatus, managerUpdateStatus } from '@/features/updates/update'
import { hasNewerVersion } from '@/features/updates/version'
import type { SyncConflictPolicy } from '@/features/sync/api'
import { applyInterfaceScale, clampScale, DEFAULT_INTERFACE_SCALE } from '@/app/appearance/scale'
import { useGlowRenderStore } from '@/app/appearance/glowRender'
import type { InstallSourceKind, ProxyScheme } from '@/features/runtime/api'
import {
  installPhaseLabel,
  offlineSteps,
  phaseShowsCommandLine,
  proxyMaskLabel,
  sourceKindLabel,
  sourceLandingLabel,
} from '@/features/runtime/presentation'

const routes = useRouteStore()
const app = useAppStore()
const dataRootInput = ref('')
const switchInput = ref('')
const externalHomeInput = ref('')
const defaultPreferences: PreferencesDto = {
  schemaVersion: 1,
  themeNeedsImport: false,
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
  networkProxyMode: 'none',
  networkProxyScheme: 'http',
  networkProxyHost: '',
  networkNoProxy: '',
  visualEffects: 'high',
  glowRender: 'mesh',
}
const preferences = ref<PreferencesDto>({ ...defaultPreferences })
const effectsLabel = computed(() =>
  preferences.value.visualEffects === 'low' ? '低' : preferences.value.visualEffects === 'mid' ? '中' : '高（默认）',
)
const glowRender = useGlowRenderStore()
const glowRenderLabel = computed(() =>
  preferences.value.glowRender === 'css' ? 'CSS（纯极光）' : 'WEBGL（网格渐变＋颗粒）',
)
const savingPreference = ref(false)
const codexShim = ref<CodexShimDto | null>(null)
const codexShimLoading = ref(false)
const codexShimBusy = ref(false)
// 行内持久说明：失败 / 未就绪原因写在这里，不依赖 1.9 秒就消失的 toast。
const codexShimError = ref('')

// 写入进行态（行内底部独占一行的小框）。
// 官方 install 先做探针、真机实测 ~20 秒且**不提供分步进度**，所以只给不确定进度条 + 真实耗时，
// 不编百分比；关闭是短、单一动作，不给进度条，只给转圈 + 文案。进行态由**真实命令耗时**驱动，
// 不引入人工假延迟。
interface ShimCommand { cmd: string; note: string }
interface ShimPlan { close: boolean; title: string; hint: string; commands: ShimCommand[]; result: string; hold: number }
const CODEX_SHIM_PLANS: Record<'install' | 'uninstall', ShimPlan> = {
  install: {
    close: false,
    title: '正在安装官方 shim',
    hint: '官方会先做一次探针校验，通常 10–30 秒；期间请勿退出应用。',
    commands: [
      { cmd: 'ocx codex-shim status', note: '读当前状态' },
      { cmd: 'ocx codex-shim install', note: '写入官方 wrapper（含探针校验）' },
      { cmd: 'ocx codex-shim status', note: '读回确认' },
    ],
    result: '已安装官方 shim：把 Codex 启动器替换为官方 wrapper，原可执行文件备份为 codex.opencodex-real；之后启动 Codex 会自动运行 ocx ensure。',
    hold: 1400,
  },
  uninstall: {
    close: true,
    title: '正在关闭官方 shim',
    hint: '关闭只把 Codex 启动器还原成原始可执行文件；通常几秒内完成，比开启快得多。',
    commands: [
      { cmd: 'ocx codex-shim uninstall', note: '移除官方 wrapper，还原 codex 原始可执行文件' },
      { cmd: 'ocx codex-shim status', note: '读回确认已还原' },
    ],
    result: '已关闭官方 shim：移除 wrapper、把 Codex 启动器还原成原始可执行文件，备份 codex.opencodex-real 已清理；Codex 启动不再自动运行 ocx ensure。',
    hold: 2000,
  },
}
const codexShimPlanKey = ref<'install' | 'uninstall'>('install')
const codexShimPhase = ref<'idle' | 'run' | 'done'>('idle')
const codexShimElapsed = ref(0)
const codexShimResult = ref('')
const codexShimResultOk = ref(true)
const codexShimShowDetails = ref(false)
let codexShimTick: number | undefined
let codexShimHold: number | undefined
const codexShimPlan = computed(() => CODEX_SHIM_PLANS[codexShimPlanKey.value])
const codexShimClosing = computed(() => codexShimPlan.value.close)
const codexShimPanelTitle = computed(() => {
  if (codexShimPhase.value === 'run') return codexShimPlan.value.title
  if (!codexShimResultOk.value) return '官方 shim 未生效'
  return codexShimPlanKey.value === 'install' ? '官方 shim 安装完成' : '已还原 Codex 启动器'
})
const codexShimPanelHint = computed(() => (codexShimPhase.value === 'run' ? codexShimPlan.value.hint : ''))
const codexShimElapsedText = computed(() => `已用 ${codexShimElapsed.value}s`)
const codexShimInstalled = computed(() => codexShim.value?.installed === true)
const codexShimUnreachable = computed(() => codexShim.value?.state === 'unreachable')
const codexShimNotice = computed(() => {
  if (codexShimError.value) return codexShimError.value
  if (!codexShimUnreachable.value) return ''
  switch (codexShim.value?.reason) {
    case 'unresolved':
      return '未解析到 ocx 可执行文件；请先在「安装配置」里指定运行来源。'
    case 'timeout':
      return '读取官方状态超时；可稍后重试，或改用终端执行 ocx codex-shim status。'
    case 'locked':
      return '状态被占用；请稍后重试。'
    default:
      return '官方 CLI 读取异常；可在终端执行 ocx codex-shim status 复核。'
  }
})
const openSelect = ref('')
const updateStatus = managerUpdateStatus
const updateChecking = ref(false)
const installRequested = ref(false)
const updateInstallDisabled = computed(() =>
  !updateStatus.value?.availableVersion
  || updateStatus.value.signatureVerified === true
  || installRequested.value
  || app.appUpdateBusy,
)
const updateError = ref('')
const SYNC_CONFLICT_POLICY_ASK: SyncConflictPolicy = 'ask'
const syncEndpointInput = ref({
  baseUrl: '',
  remotePath: '',
  username: '',
  password: '',
  // 冲突策略只有「每次询问」一种真实行为（引擎不读四档），界面固定只读；见下方 watch 注释。
  conflictPolicy: SYNC_CONFLICT_POLICY_ASK,
})
watch(() => app.syncConfig?.endpoint, endpoint => {
  if (!endpoint) {
    // 未配置端点：空表单。冲突策略固定「每次询问」，不再由「同步冲突策略」偏好驱动——
    // 该偏好四档从未在同步引擎落码，让它左右表单等于把无效果的承诺摆到界面上（§26.1）。
    syncEndpointInput.value.conflictPolicy = SYNC_CONFLICT_POLICY_ASK
    return
  }
  syncEndpointInput.value = {
    baseUrl: endpoint.url,
    remotePath: endpoint.remotePath,
    username: endpoint.username,
    password: '',
    // 只读固定值：无论历史端点里存过哪一档，界面与保存都回到真实行为「每次询问」。
    conflictPolicy: SYNC_CONFLICT_POLICY_ASK,
  }
}, { immediate: true })

async function saveSyncEndpoint() {
  await app.saveSyncEndpoint(syncEndpointInput.value)
}

// 设置页每次进入都会重新挂载，而偏好通常在启动阶段就已加载完成；
// 若只监听「变化」，挂载后再也不会触发，界面会停留在默认值（100）而与实际/持久化值不符。
// 因此立即同步一次已有的偏好，保证「显示值 = 实际比例 = 持久化值」。
watch(() => app.preferences, value => {
  if (!value) return
  preferences.value = { ...preferences.value, ...value }
  maybeAutoUpgradeBackup()
}, { immediate: true })
// 「升级前自动备份」：进入官方升级引导且本次会话尚无备份时先自动生成一份，
// 让「升级前必须备份」在不额外点按钮的情况下也成立。
function maybeAutoUpgradeBackup() {
  if (routes.settingsSection !== 'upgrade') return
  if (!app.preferences?.autoBackupUpgrade) return
  if (app.upgradeLastBackup || app.upgradeBackupBusy) return
  void app.createUpgradeBackup()
}
// 界面缩放的 DOM 镜像跟随生效偏好（含加载、保存成功与还原默认值）。
watch(() => routes.settingsSection, section => {
  if (section === 'general') void loadCodexShim()
  if (section === 'installation') void app.loadDataRootConfig()
  if (section === 'extensions' || section === 'about') void app.loadManagedPathTargets()
  // 扩展管理分区需要统一配置里的源目录与分发方式；顺带刷新一次发现结果，
  // 避免用上一次进入时的陈旧投影。
  if (section === 'extensions') {
    void app.loadExtensionConfig()
    void app.loadExtensions()
  }
  if (section === 'upgrade') {
    if (!app.officialProject && !app.officialProjectLoading) {
      void app.loadOfficialProject()
    }
    maybeAutoUpgradeBackup()
  }
}, { immediate: true })

async function persist(next: PreferencesDto, successMessage: string): Promise<boolean> {
  if (savingPreference.value) return false
  savingPreference.value = true
  const saved = await app.savePreferences(next)
  savingPreference.value = false
  if (saved) {
    preferences.value = { ...app.preferences ?? next }
    app.showToast(successMessage)
  } else {
    app.showToast('偏好保存失败；已保留当前显示值。')
  }
  return saved
}

const updateResultText = computed(() => {
  if (updateError.value) return updateError.value
  if (updateStatus.value?.error) return updateStatus.value.error
  if (updateStatus.value?.availableVersion) return `可用 ${updateStatus.value.availableVersion}；签名校验通过后才会安装。`
  if (updateStatus.value?.lastCheckedAt) return '已是最新版本；未发现可安装更新。'
  return '尚未检查；当前版本保持不变。'
})

if (typeof window !== 'undefined') {
  void getUpdateStatus().catch(() => {})
}

async function installUpdate() {
  if (installRequested.value || app.appUpdateBusy) return
  installRequested.value = true
  app.openModal({
    title: '安装应用更新',
    body: '<p>更新包将重新下载并在本地完成签名校验，通过后才会安装并重启桌面壳。</p><p>托管中的 OpenCodex 代理不会被手动停止。</p>',
    confirmLabel: '安装并重启',
    onConfirm: async () => {
      try {
        await app.installAppUpdate()
      } catch {
        // 更新 store 展示失败原因；异常路径也必须允许再次安装。
      } finally {
        installRequested.value = false
      }
    },
    onCancel: () => {
      installRequested.value = false
    },
  })
}

async function runUpdateCheck() {
  if (updateChecking.value) return
  updateChecking.value = true
  updateError.value = ''
  try {
    const { checkForUpdate } = await import('@/features/updates/update')
    await checkForUpdate()
  } catch {
    updateError.value = '更新服务不可用；已保留当前版本。'
  } finally {
    updateChecking.value = false
  }
}

// 更新通道（U-07）：偏好是唯一事实源，不再另起内存通道事务；保存后由偏好回读驱动状态。
async function chooseUpdateChannel(value: PreferencesDto['appUpdateChannel'], label: string) {
  openSelect.value = ''
  await chooseOption('appUpdateChannel', value, label)
}

// 「自动检查」开关：关闭后通道仍有效，手动检查仍可执行。
async function toggleUpdateAutoCheck() {
  const next = !preferences.value.appUpdateAutoCheck
  await persist(
    { ...preferences.value, appUpdateAutoCheck: next },
    next ? '已开启自动检查更新。' : '已关闭自动检查更新；仍可手动检查。',
  )
}

async function saveDataRoot() {
  const saved = await app.saveDataRoot(dataRootInput.value)
  if (saved) app.showToast('数据目录已初始化；未执行依赖管理。')
  else app.showToast(app.dataRootError || '数据目录初始化失败。')
}
async function applyDataRootSwitch(migrateData: boolean) {
  const saved = await app.switchDataRoot(switchInput.value, migrateData)
  if (saved) {
    app.showToast(migrateData ? '已切换数据目录并迁移数据。' : '已切换数据目录引用。')
    switchInput.value = ''
  } else app.showToast(app.dataRootError || '数据目录切换失败。')
}
async function applyOpencodexHome(mode: 'inside' | 'external') {
  const saved = await app.saveOpencodexHome(
    mode,
    mode === 'external' ? externalHomeInput.value : undefined,
  )
  if (saved) {
    app.showToast('OPENCODEX_HOME 已保存。')
    externalHomeInput.value = ''
  } else app.showToast(app.dataRootError || 'OPENCODEX_HOME 保存失败。')
}
const upgradeGuideBody = `<p>升级前请确认已生成备份，然后在终端执行官方命令：</p><pre class="modal-code"><code>ocx update</code></pre><p>桌面管理器只提供展示型引导，不会执行 <code>ocx update</code>，不会接管 npm 包管理器，也不重写官方更新事务。</p>`

function openUpgradeGuide() {
  app.openModal({
    title: '官方升级引导',
    body: upgradeGuideBody,
    confirmLabel: '复制命令',
    onConfirm: () => {
      void navigator.clipboard?.writeText('ocx update')
      app.showToast('已复制官方升级命令。')
    },
  })
}

function openUpgradeAdvice() {
  const backup = app.upgradeLastBackup
  const dataRoot = app.dataRootConfig?.activeDataRoot ?? '—'
  const backupLine = backup
    ? `<p>最近一次升级备份：<code>${backup.backupId}</code></p><p>备份目录：<code>${backup.directory}</code></p><p>备份来源：<code>${backup.targetPath}</code></p>`
    : '<p>本会话尚未生成升级备份。</p>'
  const body = `<p>数据根：<code>${dataRoot}</code></p><p>常规备份目录模式：<code>${dataRoot}/backups/&lt;YYYY&gt;/&lt;MM&gt;/upgrade/</code></p>${backupLine}<p>如需恢复，请先退出升级操作，从对应备份目录复制 <code>preferences.json</code> 后重新进入官方升级；本界面只展示建议，不会执行恢复。</p>`
  app.openModal({
    title: '升级失败建议',
    body,
    confirmLabel: '复制备份目录',
    onConfirm: () => {
      if (!backup) return
      void navigator.clipboard?.writeText(backup.directory)
      app.showToast('已复制备份目录。')
    },
  })
}

// 同 EnvironmentGate：运行来源已解析时不再把「未发现 npm 全局安装」当成待处理的阻断项。
const environment = computed(() =>
  buildEnvironmentPresentation(app.environment, app.environmentLoading, app.runtimeSource?.kind ?? null, app.aboutApp?.platform),
)
const officialProject = computed(() => app.officialProject)
const officialVersion = computed(() => {
  if (app.officialProjectLoading && !app.officialProject) return '检查中…'
  if (app.officialProjectError && !app.officialProject) return '未发现'
  const version = app.officialProject?.version ?? app.environment?.ocx.version
  return version ? `v${version}` : '未发现'
})
const officialInstallLabel = computed(() => {
  const found = app.environment?.ocx.found
  return found ? 'npm 全局 · @bitkyc08/opencodex' : '未发现 npm 全局安装'
})
const officialProjectState = computed(() => {
  if (app.officialProjectLoading && !app.officialProject) return '检查中'
  if (app.officialProjectError) return app.officialProject ? '上次结果' : '未发现'
  return app.officialProject?.truncated ? '已截断' : '外部项目'
})
// U-03：远端最新版本只读展示；查询失败如实标注，不显示假结果。
const officialRemoteText = computed(() => {
  if (app.officialRemoteLoading) return '查询中…'
  if (app.officialRemoteError) return '远端查询不可用'
  const version = app.officialRemote?.version
  return version ? `v${version}` : '尚未查询'
})
const officialUpdateAvailable = computed(() => {
  if (app.officialRemoteError) return null
  return hasNewerVersion(app.officialProject?.version ?? null, app.officialRemote?.version ?? null)
})
const officialUpdateText = computed(() => {
  if (app.officialRemoteError) return '远端不可用，无法比较版本。'
  if (app.officialRemoteLoading) return '正在查询远端最新版本…'
  if (officialUpdateAvailable.value === true) return '有可用更新；可在下方确认后由管理器代跑。'
  if (officialUpdateAvailable.value === false) return '已是最新版本。'
  return '尚未查询远端版本。'
})
async function runOfficialCheck() {
  await app.loadOfficialProject()
  await app.loadOfficialRemoteLatest()
}
function confirmOfficialUpdate() {
  const version = app.officialRemote?.version ?? '当前最新'
  app.openModal({
    title: '应用官方更新',
    body: `<p class="modal-lead">将联网把官方包 <code>@bitkyc08/opencodex@${version}</code> 安装到当前登记的托管前缀。</p><p>走受控 <code>install_runtime</code>：写 <code>.runtime-manifest.json</code>、不落全局 npm 前缀；如代理正在运行，完成后按提示重启生效。</p>`,
    confirmLabel: '安装并应用',
    onConfirm: () => {
      void app.applyOfficialUpdate()
    },
  })
}
const appVersion = computed(() => app.aboutApp ? `v${app.aboutApp.version}` : 'v0.1.0')
const installRows = computed(() => {
  const report = app.environment
  if (!report) return []
  const text = (check: typeof report.node, fallback: string) => (check.found ? `${check.path ?? fallback}${check.version ? ` · ${check.version}` : ''}` : fallback)
  return [
    ['Node.js', text(report.node, '未发现'), report.node.found],
    ['npm', text(report.npm, '未发现'), report.npm.found],
    ['可执行文件', text(report.ocx, '未发现'), report.ocx.found],
    ['安装形态', report.ocx.found ? 'npm 全局 · @bitkyc08/opencodex' : '未发现 npm 全局安装', report.ocx.found],
    ['版本', report.ocx.found ? (report.ocx.version ?? '—') : '—', report.ocx.found],
    ['Codex 运行时', report.ocx.found ? 'ChatGPT Codex 运行时 0.154.0-alpha.6.2' : '—', report.ocx.found],
  ] as const
})

// 引导安装事件在挂载时注册、卸载时成对移除（IMP-04 §13.3 A03）：
// 原来在模块作用域匿名注册且从不移除，反复进入/离开设置页会持续累积监听。
function onGuideInstallation() {
  routes.go('settings', { section: 'installation' })
}
onMounted(() => {
  window.addEventListener('opencodex:guide-installation', onGuideInstallation)
})

function refreshEnvironment() {
  void app.refreshEnvironment()
}

const sectionLabels: Record<string, string> = {
  general: '通用', installation: '安装配置', backup: '数据与备份', extensions: '扩展管理', migration: '配置迁移', sync: 'WebDAV 同步', cleanup: '日志与通知', cli: 'CLI 控制面', upgrade: '版本升级', about: '关于',
}
const scaleTicks = [
  { value: 50, pos: '0%' },
  { value: 100, pos: '33.333%' },
  { value: 150, pos: '66.666%' },
  { value: 200, pos: '100%' },
] as const
const section = computed(() => routes.settingsSection)
const OFFICIAL_HOME = 'https://opencodex.dev'
const OFFICIAL_REPO = 'https://github.com/lidge-jun/OpenCodex'
const OFFICIAL_ISSUES = 'https://github.com/lidge-jun/OpenCodex/issues'
const OFFICIAL_LICENSE = 'https://github.com/lidge-jun/OpenCodex/blob/main/LICENSE'
const APP_REPO = 'https://github.com/gzers/opencodex-desktop'
const APP_ISSUES = 'https://github.com/gzers/opencodex-desktop/issues'
const managedPartitions = computed(() => (app.managedPathTargets ?? []).map(item => ({ key: item.key, path: item.path, label: item.label })))
const skillsSourcePath = computed(() => app.agentPathTargets?.find(item => item.kind === 'skills-source')?.path ?? '—')
// 源目录是配置值：是否自定义取自统一配置；分发方式同理。
const skillsSourceIsCustom = computed(() => app.extensionConfig?.sourceDirCustom === true)
const skillsSyncMethod = computed(() => app.extensionConfig?.syncMethod ?? 'symlink')
const skillsPaths = computed(() => (app.agentPathTargets ?? []).filter(item => item.kind === 'skills').map(item => [item.label, item.path] as const))
const mcpPaths = computed(() => (app.agentPathTargets ?? []).filter(item => item.kind === 'mcp').map(item => [item.label, item.path] as const))

// 界面缩放：滑块拖动时立即预览，落盘去抖并以最后一次选择为准；刻度与数字读数离散落盘。
// 成功以磁盘回读值为准；失败回退到持久化值，保证「显示值 = 实际比例 = 持久化值」。
let scalePersistTimer: number | null = null
let scalePersistSeq = 0
let pendingScale = DEFAULT_INTERFACE_SCALE

function cancelPendingScale() {
  if (scalePersistTimer !== null) {
    window.clearTimeout(scalePersistTimer)
    scalePersistTimer = null
  }
}

async function persistScale(next: number): Promise<void> {
  const seq = ++scalePersistSeq
  const saved = await app.savePreferences({ ...preferences.value, interfaceScale: next })
  // 已有更新的选择在途/在手时，旧请求的结果不覆盖新值。
  if (seq !== scalePersistSeq) return
  if (saved) {
    preferences.value = { ...app.preferences ?? preferences.value }
    app.showToast(`界面缩放已保存为 ${next}%。`)
  } else {
    const persisted = app.preferences?.interfaceScale ?? DEFAULT_INTERFACE_SCALE
    preferences.value = { ...preferences.value, interfaceScale: persisted }
    applyInterfaceScale(persisted)
    app.showToast('界面缩放保存失败；已恢复上次保存值。')
  }
}

// 拖动预览：即时改变读数与实际比例，落盘去抖。
function previewScale(value: unknown) {
  const next = clampScale(value)
  pendingScale = next
  preferences.value = { ...preferences.value, interfaceScale: next }
  applyInterfaceScale(next)
  cancelPendingScale()
  scalePersistTimer = window.setTimeout(() => {
    scalePersistTimer = null
    void persistScale(next)
  }, 240)
}

// 离散取值（刻度、数字读数、面板 hub）：立即应用并落盘，不做去抖。
async function setScale(value: unknown): Promise<void> {
  const next = clampScale(value)
  cancelPendingScale()
  pendingScale = next
  preferences.value = { ...preferences.value, interfaceScale: next }
  applyInterfaceScale(next)
  if (next === app.preferences?.interfaceScale) return
  await persistScale(next)
}

// 数字读数越界、清空或保存失败后，都以最终生效值回写输入框，避免读数与滑块/实际取值不一致。
// 冻结原型 applyInterfaceScale 同样会把归一化结果回写到读数 value。
async function commitScale(value: unknown, event: Event) {
  await setScale(value)
  const input = event.target as HTMLInputElement | null
  if (input) input.value = String(preferences.value.interfaceScale)
}
function toggle(key: keyof PreferencesDto, label: string) {
  const next = !preferences.value[key]
  void persist({ ...preferences.value, [key]: next } as PreferencesDto, `${label}已${next ? '开启' : '关闭'}。`)
}

// 「随 Codex 启动 OpenCodex」不在管理器偏好里落盘：开关的真实状态与写入通道
// 都跟随官方 shim（`ocx codex-shim`），管理器只做投影与转发（AC-11 不做旁路写入）。
async function loadCodexShim() {
  codexShimLoading.value = true
  try {
    codexShim.value = await getCodexShimStatus()
    codexShimError.value = ''
  } catch {
    codexShim.value = { state: 'unreachable', installed: false, summary: '', reason: 'failed' }
  } finally {
    codexShimLoading.value = false
  }
}

async function toggleCodexShim() {
  if (codexShimBusy.value || codexShimLoading.value || !codexShim.value || codexShim.value.state === 'unreachable') return
  const next = !codexShim.value.installed
  codexShimError.value = ''
  // 官方 shim 安装会先做一次探针，真机实测要 ~20 秒；这段必须给出进行态，
  // 否则用户只会看到「点了没反应」。进行态由真实耗时驱动（不造假延迟）。
  codexShimPlanKey.value = next ? 'install' : 'uninstall'
  codexShimPhase.value = 'run'
  codexShimResult.value = ''
  codexShimResultOk.value = true
  codexShimShowDetails.value = false
  codexShimBusy.value = true
  startCodexShimTimer()
  try {
    codexShim.value = await setCodexShim(next)
    codexShimResult.value = codexShimPlan.value.result
    codexShimResultOk.value = true
    codexShimPhase.value = 'done'
    app.showToast(`随 Codex 启动 OpenCodex 已${next ? '开启' : '关闭'}。`)
    holdCodexShimPanel(codexShimPlan.value.hold)
  } catch (error) {
    // 官方 CLI 的失败原因有两种真实形态，后端按目标状态校验后原样带回来，
    // 这里映射成可执行的中文说明（官方原文是英文，不直接展示）。
    // 说明写进行内，toast 只做提示：1.9 秒的 toast 承载不了「去开哪个权限」。
    const detail = String((error as { message?: string } | null)?.message ?? '')
    const notice = codexShimFailureNotice(detail)
    app.showToast('官方 shim 未生效；原因见该行说明。')
    // 先把真实状态读回来（会清掉旧说明），再落本次失败说明，避免被回读覆盖。
    await loadCodexShim()
    codexShimError.value = notice
    // 失败也留在框里说清「做了什么」，别只留 toast。
    codexShimResult.value = notice
    codexShimResultOk.value = false
    codexShimPhase.value = 'done'
    holdCodexShimPanel(2000)
  } finally {
    codexShimBusy.value = false
  }
}

/// 进行态计时：真实耗时，启动即开始，收尾即停。
function startCodexShimTimer() {
  stopCodexShimTimer()
  codexShimElapsed.value = 0
  const startedAt = Date.now()
  codexShimTick = window.setInterval(() => {
    codexShimElapsed.value = Math.round((Date.now() - startedAt) / 1000)
  }, 500)
}
function stopCodexShimTimer() {
  if (codexShimTick !== undefined) {
    window.clearInterval(codexShimTick)
    codexShimTick = undefined
  }
}
/// 完成后让结果说明停留一会儿再收起，避免一闪而过。
function holdCodexShimPanel(hold: number) {
  stopCodexShimTimer()
  if (codexShimHold !== undefined) window.clearTimeout(codexShimHold)
  codexShimHold = window.setTimeout(() => {
    codexShimPhase.value = 'idle'
    codexShimResult.value = ''
    codexShimShowDetails.value = false
  }, hold)
}

/// 写入失败的行内说明（比 toast 更持久，用户要能对着它去处理）。
function codexShimFailureNotice(detail: string): string {
  if (/EPERM|operation not permitted/.test(detail)) {
    return '官方 shim 未生效：macOS 拒绝本应用改动 Codex 所在的应用包。请在「系统设置 → 隐私与安全性 → App 管理」里授权「OpenCodeX Desktop」（授权后需重启应用），或改用终端执行 ocx codex-shim install。'
  }
  if (detail.includes('Could not find a codex executable')) {
    return '官方 shim 未生效：未找到 codex 可执行文件；请先确认 codex 已安装并可从终端运行。'
  }
  return '官方 shim 写入失败；已保留原状态。可在终端执行 ocx codex-shim status 复核。'
}

function chooseOption(key: keyof PreferencesDto, value: PreferencesDto[keyof PreferencesDto], label: string) {
  if (preferences.value[key] === value) {
    openSelect.value = ''
    return
  }
  openSelect.value = ''
  void persist({ ...preferences.value, [key]: value } as PreferencesDto, `已选择「${label}」。`)
}

// 三档特效（UI规范 §26.2）：即时生效 + 落盘回读；失败回退持久化值（含 DOM 档位属性）。
async function chooseVisualEffects(value: 'high' | 'mid' | 'low', label: string) {
  openSelect.value = ''
  if (preferences.value.visualEffects === value) return
  preferences.value = { ...preferences.value, visualEffects: value }
  const saved = await app.setVisualEffects(value)
  if (saved) {
    preferences.value = { ...(app.preferences ?? preferences.value) }
    app.showToast(`特效档位已选择「${label}」。`)
  } else {
    preferences.value = { ...preferences.value, visualEffects: app.preferences?.visualEffects ?? 'high' }
    app.showToast('特效档位保存失败；已恢复上次保存值。')
  }
}
// 背景光渲染方式（画质卡片）：即时生效 + 落盘回读；设备不支持 WEBGL 时由统一策略回退 CSS。
async function chooseGlowRender(value: 'mesh' | 'css', label: string) {
  openSelect.value = ''
  if (preferences.value.glowRender === value) return
  preferences.value = { ...preferences.value, glowRender: value }
  const saved = await app.setGlowRender(value)
  if (saved) {
    preferences.value = { ...(app.preferences ?? preferences.value) }
    app.showToast(`背景光渲染已选择「${label}」。`)
  } else {
    preferences.value = { ...preferences.value, glowRender: app.preferences?.glowRender ?? 'mesh' }
    app.showToast('背景光渲染保存失败；已恢复上次保存值。')
  }
}
// 网络代理（U-05）：模式与手动地址落盘；不含凭据，不使用钥匙串。
async function chooseProxyMode(value: PreferencesDto['networkProxyMode'], label: string) {
  openSelect.value = ''
  await persist({ ...preferences.value, networkProxyMode: value }, `代理模式已选择「${label}」。`)
}
async function chooseProxyScheme(value: PreferencesDto['networkProxyScheme'], label: string) {
  openSelect.value = ''
  await persist({ ...preferences.value, networkProxyScheme: value }, `代理协议已选择「${label}」。`)
}
async function saveProxyHost() {
  await persist({ ...preferences.value }, '网络代理地址已保存。')
}
function goToNetwork() {
  routes.go('settings', { section: 'general' })
}

// 网络连通性检查（U-05）：带超时、不改写任何状态；结果如实展示。
const proxyProbeBusy = ref(false)
const proxyProbeResult = ref('')
const proxyProbeOk = ref(false)
async function runProxyProbe() {
  if (proxyProbeBusy.value) return
  proxyProbeBusy.value = true
  proxyProbeResult.value = ''
  try {
    const result = await checkNetworkProxy()
    proxyProbeOk.value = result.ok
    proxyProbeResult.value = result.detail
  } catch {
    proxyProbeOk.value = false
    proxyProbeResult.value = '检查失败；未改写任何设置。'
  } finally {
    proxyProbeBusy.value = false
  }
}

function toggleSelect(key: string) {
  openSelect.value = openSelect.value === key ? '' : key
}
function closeSelects(event: MouseEvent) {
  if (!(event.target as HTMLElement).closest('.select, .select-menu')) openSelect.value = ''
}
if (typeof window !== 'undefined') window.addEventListener('click', closeSelects)
onBeforeUnmount(() => {
  window.removeEventListener('opencodex:guide-installation', onGuideInstallation)
  window.removeEventListener('click', closeSelects)
  stopCodexShimTimer()
  if (codexShimHold !== undefined) window.clearTimeout(codexShimHold)
  // 离开设置页时把拖动中的待落盘值立即提交，避免「看起来已改、实际未持久化」。
  if (scalePersistTimer !== null) {
    cancelPendingScale()
    void persistScale(pendingScale)
  }
})
const cliSupported = computed(() => app.aboutApp?.platform !== 'Windows')

function toggleCli() {
  if (!cliSupported.value) return
  void persist(
    { ...preferences.value, cliEnabled: !preferences.value.cliEnabled },
    preferences.value.cliEnabled ? 'CLI 控制面已关闭；socket 将移除。' : 'CLI 控制面已启用；本机 IPC 已启动。',
  )
}

const agentPrompt = [
  '管理器 CLI：ocxd（先 ocxd --help；仅限管理器自有域）',
  '官方 CLI：ocx（提供方 / 路由 / 模型映射）',
  '规则：变更先 --confirm；新容器导出导入无需口令，仅旧加密容器导入用 --password-stdin；机读输出用 --json。',
].join('\n')

async function copyAgentPrompt() {
  try {
    await navigator.clipboard.writeText(agentPrompt)
    app.showToast('Agent 提示已复制。')
  } catch {
    app.showToast('复制失败；请手动复制页面提示。')
  }
}

// 新容器不含额外口令：导出只需落到数据根 exports 分区。
function exportMigration() {
  void app.exportMigration().then(success => {
    if (success) {
      const sections = app.migrationLastExport?.sections ?? []
      app.showToast(sections.length ? `配置已导出（${sections.join('、')}）。` : '配置已导出到数据目录 exports 分区。')
    } else {
      app.showToast(app.migrationError || '配置导出失败。')
    }
  })
}

// 导入先按新格式直接执行；只有旧版加密容器才回退到「输入原口令」。
function importMigration() {
  app.openModal({
    title: '导入配置',
    body: '将校验容器、备份当前配置并应用桌面壳设置。容器内的凭据只保留引用，不包含日志与缓存。',
    confirmLabel: '导入配置',
    onConfirm: () => {
      void runImport()
    },
  })
}

async function runImport(passphrase = '') {
  const success = await app.importMigration(passphrase)
  if (success) {
    const applied = app.migrationLastImport?.appliedSections ?? []
    app.showToast(applied.length ? `配置已导入（${applied.join('、')}）。` : '配置已导入。')
    return
  }
  if ((app.migrationError || '').includes('原口令')) {
    openLegacyPassphraseModal()
    return
  }
  app.showToast(app.migrationError || '配置导入失败。')
}

function openLegacyPassphraseModal() {
  const field = { key: 'passphrase', label: '旧容器口令', type: 'password' as const }
  app.openModal({
    title: '这是旧版加密容器',
    body: '<p>旧版导出文件仍由用户口令保护；请输入导出时使用的原口令。口令只在本次导入使用，不会保存。</p>',
    fields: [field],
    confirmLabel: '用原口令导入',
    onConfirm: (values?: Record<string, string>) => {
      void runImport(values?.passphrase ?? '')
    },
  })
}


async function copyPath(path: string) {
  try {
    await navigator.clipboard.writeText(path)
    app.showToast('路径已复制。')
  } catch {
    app.showToast('复制失败；请手动复制路径。')
  }
}

function openPath(path: string | undefined, label: string) {
  void app.openManagedPath(path).then(success => {
    if (!success) app.showToast(`${label}不可用；已保留当前状态。`)
  })
}

function openManagedTarget(key: 'manager_state' | 'backups' | 'logs' | 'exports' | 'cache' | 'sync_state') {
  // 先加载路径投影再打开；未加载时此前会静默无反应。
  void app.openManagedPathByKey(key)
}

// FZ-23：源目录是配置值——默认为共享目录，可改为自定义目录。
// 目录选择走系统原生对话框（macOS / Windows 一致），不自造浏览器；
// 合法性校验仍由后端完成，失败保留原值并给出原因，不静默换目录。
async function chooseSkillsSourceDir() {
  let picked: string | null = null
  try {
    const result = await open({
      directory: true,
      multiple: false,
      title: '选择 Skills 源目录',
      defaultPath: skillsSourcePath.value === '—' ? undefined : skillsSourcePath.value,
    })
    picked = typeof result === 'string' ? result : null
  } catch {
    app.showToast('无法打开系统目录选择器；已保留当前源目录。')
    return
  }
  if (!picked) {
    app.showToast('未选择目录；已保留当前源目录。')
    return
  }
  const saved = await app.setExtensionSourceDir(picked)
  if (saved) app.showToast('源目录已切换；未移动任何 Skill。')
  else app.showToast(app.extensionConfigError ? '源目录不可用；已保留原值。' : '源目录切换失败。')
}

async function restoreDefaultSkillsSourceDir() {
  const saved = await app.setExtensionSourceDir(null)
  if (saved) app.showToast('已恢复默认源目录。')
  else app.showToast('恢复默认源目录失败；已保留原值。')
}

// FZ-24：分发方式是用户可选项，写入路径按它决定建软链接还是写独立副本。
async function chooseSkillsSyncMethod(value: 'symlink' | 'copy', label: string) {
  openSelect.value = ''
  if (app.extensionConfig?.syncMethod === value) return
  const saved = await app.setExtensionSyncMethod(value)
  if (saved) app.showToast(`已选择「${label}」；已同步的目标会按新方式重新落地。`)
  else app.showToast('切换同步方式失败；已保留原值。')
}

// 「检查并修正」：按当前源目录 + 分发方式把已落地的客户端目标重新对齐一次。
async function resyncSkills() {
  const ok = await app.resyncExtensionSkills()
  if (ok) app.showToast('已按当前源目录与同步方式重新对齐 Skills 落点。')
  else app.showToast('重新对齐失败；已保留现有落点。')
}

function runSyncNow() {
  void app.runSyncNow().then(success => {
    if (!success) app.showToast(app.syncStatus?.message || '同步未执行；已保留本地内容。')
  })
}

function cleanupLogs() {
  void app.cleanupLocalLogs().then(success => {
    if (success) app.showToast('本地日志视图已清理。')
    else app.showToast('日志清理失败；原始文件已保留。')
  })
}

function cleanupNotifications() {
  void app.cleanupNotifications().then(success => {
    if (success) app.showToast('通知已清理。')
    else app.showToast('通知清理失败；当前通知已保留。')
  })
}

function openExternal(url: string) {
  void app.openExternalLink(url).then(success => {
    if (!success) app.showToast('外链打开失败；已保留当前窗口。')
  })
}

function openLocalDocument(documentId: 'license' | 'third-party') {
  void app.openLocalDocument(documentId).then(success => {
    if (!success) app.showToast('文档不可用；已保留当前窗口。')
  })
}

// ---- 运行来源 / 托管安装 / 卸载（IMP Track B · B5）----
const installOpen = ref(false)
const installStep = ref(1)
const installPrefix = ref('')
const installSource = ref<InstallSourceKind>('registry')
const installVersion = ref('latest')
const installUseSpecificVersion = ref(false)
const installOfflinePath = ref('')
const installConfirmed = ref(false)
const installProxy = ref({ scheme: 'http' as ProxyScheme, host: '', username: '', secret: '' })
const installAllowScripts = ref(false)

const uninstallOpen = ref(false)
const uninstallScope = ref<'body' | 'full'>('full')
const uninstallAck = ref(false)
const uninstallAutoBackup = ref(true)
const uninstallCleanData = ref(true)
const uninstallRunning = ref(false)
const uninstallDone = ref(false)

const runtimeSourceLabel = computed(() =>
  app.runtimeSource ? sourceKindLabel(app.runtimeSource.kind) : '未解析',
)
const installPrefixError = computed(() => {
  const value = installPrefix.value.trim()
  if (!value) return ''
  if (!value.startsWith('/')) return '安装位置必须是绝对路径'
  const protectedRoots = ['/', '/bin', '/sbin', '/usr', '/etc', '/System', '/Library', '/private', '/Applications']
  if (protectedRoots.some(root => value === root || (root !== '/' && value.startsWith(`${root}/`)))) {
    return '安装位置落在系统保护目录内，已拒绝'
  }
  return ''
})
const installLandingText = computed(() => {
  const root = app.dataRootConfig?.activeDataRoot ?? ''
  const value = installPrefix.value.trim()
  if (!root || !value) return ''
  return value.startsWith(root) ? '落点：数据根内' : '落点：数据根外'
})
const proxyHint = computed(() => {
  const host = installProxy.value.host.trim()
  if (!host) return '留空直连；代理只作用于本次安装，不写系统代理与全局 npm 配置。'
  return `本次安装将通过 ${proxyMaskLabel(installProxy.value.scheme, host)}（SOCKS5 使用 socks5h://）。`
})
const installCanAdvance = computed(() => {
  if (installStep.value === 1) return installPrefix.value.trim() !== '' && !installPrefixError.value
  if (installStep.value === 2) {
    if (installSource.value === 'offline') return app.offlinePreview?.ok === true
    return true
  }
  if (installStep.value === 3) return installConfirmed.value
  return true
})
const installProgress = computed(() => app.runtimeInstall)
const offlineStepList = computed(() => offlineSteps(app.runtimeInstall?.phase ?? 'preparing'))
const showInstallCommandLine = computed(() =>
  installSource.value === 'registry' && phaseShowsCommandLine(app.runtimeInstall?.phase ?? 'preparing'),
)
const uninstallCanConfirm = computed(() => {
  if (uninstallRunning.value || app.runtimeUninstallBusy) return false
  if (app.runtimeUninstallPlan?.removable === false) return false
  return uninstallAck.value
})

/** 进行中显示计划步骤；完成后换成后端返回的真实步骤与状态。 */
const uninstallStepRows = computed(() => {
  const result = app.runtimeUninstallResult
  if (result && uninstallDone.value) {
    return result.steps.map(step => ({
      name: step.name,
      detail: step.detail ?? '',
      cls: step.status === 'ok' ? 'ok' : step.status === 'failed' ? 'failed' : 'skip',
    }))
  }
  const planned = uninstallScope.value === 'full'
    ? ['停止代理', '生成备份', '移除包体与入口', '执行官方 ocx uninstall', '清空 OPENCODEX_HOME 非官方自有残留', '残留核验']
    : ['停止代理', '生成备份', '移除包体与入口', '残留核验']
  return planned.map(name => ({ name, detail: '', cls: 'on' }))
})

const residueOk = computed(() =>
  (app.runtimeUninstallResult?.residue ?? []).every(item => item.status === 'cleared'),
)

// 卸载头必须反映**真实结果**：有失败步骤时不得报「100%」成功（§26.1 禁止假成功）。
const uninstallFailedSteps = computed(() =>
  (app.runtimeUninstallResult?.steps ?? []).filter(step => step.status === 'failed').length,
)
const uninstallPartialFailure = computed(() => uninstallDone.value && uninstallFailedSteps.value > 0)

function residueLabel(status: string) {
  if (status === 'cleared') return '已清除'
  if (status === 'present') return '残留'
  return '未知（未确认清除）'
}

function defaultInstallPrefix() {
  // 重装落在**登记过的**前缀上（自定义前缀安装后不要悄悄搬回默认位置）。
  return app.runtimeSource?.managedPrefix ?? ''
}

/** 「恢复默认」只认数据根默认落点，不受自定义前缀影响。 */
function managedDefaultPrefix() {
  return app.runtimeSource?.defaultPrefix ?? ''
}

function openInstallModal(source: InstallSourceKind, version?: string) {
  installOpen.value = true
  installStep.value = 1
  installPrefix.value = defaultInstallPrefix()
  installSource.value = source
  installVersion.value = version ?? 'latest'
  installUseSpecificVersion.value = Boolean(version && version !== 'latest')
  installOfflinePath.value = ''
  installConfirmed.value = false
  installProofReset()
  app.clearOfflinePreview()
}

function installProofReset() {
  installProxy.value = { scheme: 'http', host: '', username: '', secret: '' }
  installAllowScripts.value = false
}

function closeInstallModal() {
  if (app.runtimeInstalling) return
  installOpen.value = false
}

async function chooseRuntimeSource() {
  const picked = await open({ multiple: false, title: '选择 ocx 可执行文件' })
  if (typeof picked !== 'string') return
  await app.setRuntimeSourcePath(picked)
}

async function chooseInstallPrefix() {
  const picked = await open({ directory: true, multiple: false, title: '选择 OpenCodex 托管安装位置' })
  if (typeof picked === 'string') installPrefix.value = picked
}

function resetInstallPrefix() {
  installPrefix.value = managedDefaultPrefix()
}

async function chooseOfflinePackage() {
  const picked = await open({
    multiple: false,
    title: '选择 OpenCodex 离线包',
    filters: [{ name: '离线包', extensions: ['tgz', 'gz'] }],
  })
  if (typeof picked !== 'string') return
  installOfflinePath.value = picked
  await app.previewOfflinePackage(picked)
}

function advanceInstall() {
  if (!installCanAdvance.value) return
  if (installStep.value === 3) {
    void runInstall()
    return
  }
  installStep.value += 1
}

async function runInstall() {
  installStep.value = 4
  const useProxy = installSource.value === 'registry' && installProxy.value.host.trim() !== ''
  await app.installRuntime({
    prefix: installPrefix.value.trim() || null,
    source: installSource.value,
    version: installSource.value === 'registry' ? (installUseSpecificVersion.value ? installVersion.value.trim() || 'latest' : 'latest') : null,
    offlinePath: installSource.value === 'offline' ? installOfflinePath.value || null : null,
    proxyScheme: useProxy ? installProxy.value.scheme : null,
    proxyHost: useProxy ? installProxy.value.host.trim() : null,
    proxyUsername: useProxy && installProxy.value.username.trim() ? installProxy.value.username.trim() : null,
    proxySecret: useProxy && installProxy.value.username.trim() ? installProxy.value.secret : null,
    allowScripts: installAllowScripts.value,
  })
}

async function cancelInstall() {
  if (app.runtimeInstalling) {
    const cancelled = await app.cancelRuntimeInstall()
    app.showToast(cancelled ? '已请求取消安装。' : '当前没有可取消的安装。')
    return
  }
  closeInstallModal()
}

function finishInstall() {
  const version = app.runtimeInstallOutcome?.version
  installOpen.value = false
  if (version) app.showToast(`OpenCodex ${version} 已就绪。`)
}

function openUninstall() {
  uninstallOpen.value = true
  uninstallScope.value = 'full'
  uninstallAck.value = false
  uninstallCleanData.value = true
  uninstallRunning.value = false
  uninstallDone.value = false
  const runtime = useRuntimeStore()
  runtime.uninstallResult = null
  runtime.uninstallError = ''
  runtime.uninstallPlan = null
  // 备份只提醒、不阻断：没检测到可复用备份时默认勾选「卸载前自动生成备份」。
  uninstallAutoBackup.value = true
  void app.loadUninstallPlan().then(() => {
    if (app.runtimeUninstallPlan?.backupId) uninstallAutoBackup.value = false
  })
}

function closeUninstall() {
  if (uninstallRunning.value || app.runtimeUninstallBusy) return
  uninstallOpen.value = false
}

async function confirmUninstall() {
  if (!uninstallCanConfirm.value) return
  uninstallRunning.value = true
  const ok = await app.uninstallRuntime({
    scope: uninstallScope.value,
    autoBackup: uninstallAutoBackup.value,
    cleanData: uninstallScope.value === 'full' && uninstallCleanData.value,
    confirmation: 'confirmed',
  })
  uninstallRunning.value = false
  if (ok) uninstallDone.value = true
}

function formatBytes(size: number) {
  if (!size) return '0 B'
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

// 概览门禁的「安装 / 导入离线包」出口：置位后打开对应弹窗。
watch(() => app.installModalRequest, value => {
  if (!value) return
  const source = app.consumeInstallModalRequest()
  if (source) openInstallModal(source)
})
watch(() => routes.settingsSection, section => {
  if (section === 'installation') {
    if (!app.runtimeSource) void app.loadRuntimeSource()
    void app.loadOfficialUninstallObservation()
  }
})

function restoreGeneralDefaults() {
  void app.restorePreferences().then(restored => {
    if (restored) {
      preferences.value = { ...defaultPreferences, ...(app.preferences ?? {}) }
      app.showToast('已还原 30 项默认偏好。')
    } else app.showToast('偏好还原失败；已保留当前显示值。')
  })
}
</script>

<template>
  <section class="route-section active">
    <AppTopbar />
    <div class="settings-tabs" role="tablist">
      <button v-for="item in settingsSections" :key="item" :class="{ active: section === item }" @click="routes.go('settings', { section: item })">{{ sectionLabels[item] }}</button>
    </div>

    <section v-if="section === 'general'" class="settings-panel active">
      <UiCard>
        <UiCardHeader><h2>桌面壳偏好</h2><p>管理器自有行为、面板承载与通知。</p></UiCardHeader>
        <div class="setting-list">
          <SettingRow title="界面缩放" description="外壳与官方面板使用同一缩放因子；支持 50%–200% 自定义，默认 100%。">
            <template #actions>
              <div class="controls"><div class="range"><div class="range-slider"><input type="range" min="50" max="200" step="1" :value="preferences.interfaceScale" aria-label="界面缩放" @input="previewScale(Number(($event.target as HTMLInputElement).value))"><div class="range-ticks" role="group" aria-label="界面缩放刻度"><button v-for="tick in scaleTicks" :key="tick.value" type="button" class="range-tick" :class="{ active: preferences.interfaceScale === tick.value }" :style="{ '--pos': tick.pos }" :aria-pressed="preferences.interfaceScale === tick.value ? 'true' : 'false'" @click="setScale(tick.value)">{{ tick.value }}</button></div></div><input class="range-value" type="number" inputmode="numeric" min="50" max="200" step="1" :value="preferences.interfaceScale" aria-label="界面缩放数值" @change="commitScale(($event.target as HTMLInputElement).value, $event)"></div></div>
            </template>
          </SettingRow>
          <div class="setting-row"><div><div class="setting-title">启动时打开主界面</div><div class="setting-desc">默认打开；关闭后启动只保留托盘，不自动显示主窗口，也不自动启动代理。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.launchMain" @click="toggle('launchMain', '启动时打开主界面')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">面板打开方式</div><div class="setting-desc">默认内嵌到 App 窗口；加载异常时提供浏览器兜底入口。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'panelMode' }"><button class="select-trigger" :aria-expanded="openSelect === 'panelMode' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="面板打开方式" @click="toggleSelect('panelMode')"><span class="select-value">{{ preferences.panelMode === 'embedded' ? '内嵌优先' : '浏览器兜底' }}</span></button><UiSelectMenu :open="openSelect === 'panelMode'" @close="openSelect = ''" role="listbox" aria-label="面板打开方式"><button class="select-option" :class="{ selected: preferences.panelMode === 'embedded' }" type="button" role="option" :aria-selected="preferences.panelMode === 'embedded' ? 'true' : 'false'" @click="chooseOption('panelMode', 'embedded', '内嵌优先')">内嵌优先</button><button class="select-option" :class="{ selected: preferences.panelMode === 'browser' }" type="button" role="option" :aria-selected="preferences.panelMode === 'browser' ? 'true' : 'false'" @click="chooseOption('panelMode', 'browser', '浏览器兜底')">浏览器兜底</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">关闭窗口后保持代理运行</div><div class="setting-desc">关闭窗口只隐藏桌面壳；从托盘退出会完全退出并可能中断由桌面壳托管的代理。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.keepProxyOnClose" @click="toggle('keepProxyOnClose', '关闭窗口后保持代理运行')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">启停结果通知</div><div class="setting-desc">通知启动、停止、重启和失败结果；不展示敏感配置。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.lifecycleNotifications" @click="toggle('lifecycleNotifications', '启停结果通知')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">同步冲突提醒</div><div class="setting-desc">检测到本地与远端冲突时暂停覆盖，并要求显式确认。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.syncConflictAlerts" @click="toggle('syncConflictAlerts', '同步冲突提醒')"></button></div></div>
        </div>
      </UiCard>
      <UiCard data-testid="card-quality">
        <UiCardHeader><h2>画质</h2><p>背景光与视觉效果的渲染方式；按设备能力自动降级。</p></UiCardHeader>
        <div class="setting-list">
          <div class="setting-row" data-testid="setting-visual-effects"><div><div class="setting-title">界面特效</div><div class="setting-desc">高档为通知玻璃与动态光场；中档玻璃静态；低档统一实底、无光场。画质由所选档位决定，不跟随系统减少动态。即时生效并按偏好恢复。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'visualEffects' }"><button class="select-trigger" :aria-expanded="openSelect === 'visualEffects' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="界面特效" @click="toggleSelect('visualEffects')"><span class="select-value">{{ effectsLabel }}</span></button><UiSelectMenu :open="openSelect === 'visualEffects'" @close="openSelect = ''" role="listbox" aria-label="界面特效"><button class="select-option" :class="{ selected: preferences.visualEffects === 'high' }" type="button" role="option" :aria-selected="preferences.visualEffects === 'high' ? 'true' : 'false'" @click="chooseVisualEffects('high', '高（默认）')">高（默认）</button><button class="select-option" :class="{ selected: preferences.visualEffects === 'mid' }" type="button" role="option" :aria-selected="preferences.visualEffects === 'mid' ? 'true' : 'false'" @click="chooseVisualEffects('mid', '中')">中</button><button class="select-option" :class="{ selected: preferences.visualEffects === 'low' }" type="button" role="option" :aria-selected="preferences.visualEffects === 'low' ? 'true' : 'false'" @click="chooseVisualEffects('low', '低')">低</button></UiSelectMenu></div></div></div>
          <div class="setting-row" data-testid="setting-glow-render"><div><div class="setting-title">背景光渲染</div><div class="setting-desc">WEBGL 用网格渐变＋颗粒着色器，过渡最平滑、色带最少；CSS 为纯极光兜底，不依赖 GPU。<span v-if="glowRender.fellBack" class="setting-note" data-testid="glow-render-fallback">当前环境不支持 WEBGL，已自动回退纯 CSS 极光。</span></div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'glowRender' }"><button class="select-trigger" :aria-expanded="openSelect === 'glowRender' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="背景光渲染" @click="toggleSelect('glowRender')"><span class="select-value">{{ glowRenderLabel }}</span></button><UiSelectMenu :open="openSelect === 'glowRender'" @close="openSelect = ''" role="listbox" aria-label="背景光渲染"><button class="select-option" :class="{ selected: preferences.glowRender === 'mesh' }" type="button" role="option" :aria-selected="preferences.glowRender === 'mesh' ? 'true' : 'false'" @click="chooseGlowRender('mesh', 'WEBGL（网格渐变＋颗粒）')">WEBGL（网格渐变＋颗粒）</button><button class="select-option" :class="{ selected: preferences.glowRender === 'css' }" type="button" role="option" :aria-selected="preferences.glowRender === 'css' ? 'true' : 'false'" @click="chooseGlowRender('css', 'CSS（纯极光）')">CSS（纯极光）</button></UiSelectMenu></div></div></div>
        </div>
      </UiCard>
<UiCard data-testid="setting-network">
        <UiCardHeader><h2>网络</h2><p>应用自更新、官方版本查询与托管安装的出站请求代理；不含凭据，不写入系统代理设置。</p></UiCardHeader>
        <div class="setting-list">
          <div class="setting-row" data-testid="setting-proxy-mode"><div><div class="setting-title">代理模式</div><div class="setting-desc">无代理直连；自动沿用系统代理设置；手动指定 HTTP 或 SOCKS5 代理。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'networkProxyMode' }"><button class="select-trigger" :aria-expanded="openSelect === 'networkProxyMode' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="代理模式" @click="toggleSelect('networkProxyMode')"><span class="select-value">{{ preferences.networkProxyMode === 'system' ? '自动（系统代理）' : preferences.networkProxyMode === 'manual' ? '手动配置' : '无代理' }}</span></button><UiSelectMenu :open="openSelect === 'networkProxyMode'" @close="openSelect = ''" role="listbox" aria-label="代理模式"><button class="select-option" :class="{ selected: preferences.networkProxyMode === 'none' }" type="button" role="option" :aria-selected="preferences.networkProxyMode === 'none' ? 'true' : 'false'" @click="chooseProxyMode('none', '无代理')">无代理</button><button class="select-option" :class="{ selected: preferences.networkProxyMode === 'system' }" type="button" role="option" :aria-selected="preferences.networkProxyMode === 'system' ? 'true' : 'false'" @click="chooseProxyMode('system', '自动（系统代理）')">自动（系统代理）</button><button class="select-option" :class="{ selected: preferences.networkProxyMode === 'manual' }" type="button" role="option" :aria-selected="preferences.networkProxyMode === 'manual' ? 'true' : 'false'" @click="chooseProxyMode('manual', '手动配置')">手动配置</button></UiSelectMenu></div></div></div>
          <div v-if="preferences.networkProxyMode === 'manual'" class="setting-row" data-testid="setting-proxy-scheme"><div><div class="setting-title">代理协议</div><div class="setting-desc">SOCKS5 使用 socks5h（DNS 也走代理，避免本地泄漏）。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'networkProxyScheme' }"><button class="select-trigger" :aria-expanded="openSelect === 'networkProxyScheme' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="代理协议" @click="toggleSelect('networkProxyScheme')"><span class="select-value">{{ preferences.networkProxyScheme === 'socks5h' ? 'SOCKS5' : 'HTTP' }}</span></button><UiSelectMenu :open="openSelect === 'networkProxyScheme'" @close="openSelect = ''" role="listbox" aria-label="代理协议"><button class="select-option" :class="{ selected: preferences.networkProxyScheme === 'http' }" type="button" role="option" :aria-selected="preferences.networkProxyScheme === 'http' ? 'true' : 'false'" @click="chooseProxyScheme('http', 'HTTP')">HTTP</button><button class="select-option" :class="{ selected: preferences.networkProxyScheme === 'socks5h' }" type="button" role="option" :aria-selected="preferences.networkProxyScheme === 'socks5h' ? 'true' : 'false'" @click="chooseProxyScheme('socks5h', 'SOCKS5')">SOCKS5</button></UiSelectMenu></div></div></div>
          <div v-if="preferences.networkProxyMode === 'manual'" class="setting-row" data-testid="setting-proxy-host"><div><div class="setting-title">主机名与端口</div><div class="setting-desc">例如 127.0.0.1:7890；不接受含空白、@ 或凭据的地址。</div></div><div class="controls"><input class="text-input" type="text" :value="preferences.networkProxyHost" aria-label="代理主机名与端口" placeholder="127.0.0.1:7890" @change="preferences.networkProxyHost = ($event.target as HTMLInputElement).value; saveProxyHost()"></div></div>
          <div v-if="preferences.networkProxyMode === 'manual'" class="setting-row" data-testid="setting-proxy-no-proxy"><div><div class="setting-title">不使用代理的地址</div><div class="setting-desc">逗号分隔的例外，例如 localhost,127.0.0.1,192.168.*。</div></div><div class="controls"><input class="text-input" type="text" :value="preferences.networkNoProxy" aria-label="不使用代理的地址" placeholder="localhost,127.0.0.1" @change="preferences.networkNoProxy = ($event.target as HTMLInputElement).value; saveProxyHost()"></div></div>
          <div class="setting-row"><div><div class="setting-title">应用范围</div><div class="setting-desc">应用自更新、官方版本查询与托管安装共用此代理；TLS 证书校验不可关闭。WebDAV 同步使用其端点自带代理。</div></div><div class="controls"><span class="setting-readonly">本版不含代理鉴权</span></div></div>
          <div class="setting-row" data-testid="setting-network-probe"><div><div class="setting-title">检查连接</div><div class="setting-desc">{{ proxyProbeResult || '用当前配置对固定地址发一次带超时请求；只回报结果，不改写任何设置。' }}</div></div><div class="controls"><span v-if="proxyProbeResult" class="tag" :class="{ danger: !proxyProbeOk, ok: proxyProbeOk }">{{ proxyProbeOk ? '可用' : '不可用' }}</span><button class="btn ghost" type="button" :disabled="proxyProbeBusy" @click="runProxyProbe()">{{ proxyProbeBusy ? '检查中' : '检查连接' }}</button></div></div>
        </div>
      </UiCard>

<article class="card"><div class="card-head"><div><h2>官方共享配置</h2><p>与官方 OpenCodex 使用同一配置源；模型、路由与提供方仍由官方面板或官方 CLI 管理。</p></div></div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">随 Codex 启动 OpenCodex <span class="tag">官方共享</span></div><div class="setting-desc">与官方面板同一配置；开关的读写都经官方 shim <code>ocx codex-shim</code>，管理器不旁路改写官方配置。</div><div v-if="codexShimNotice" class="setting-note" data-testid="codex-shim-notice">{{ codexShimNotice }}</div></div><div class="controls"><span v-if="codexShimUnreachable" class="setting-readonly">官方 CLI 未就绪</span><button v-else class="toggle" role="switch" :aria-checked="codexShimInstalled" :disabled="codexShimLoading || codexShimBusy" aria-label="随 Codex 启动 OpenCodex" @click="toggleCodexShim()"></button></div><div v-if="codexShimPhase !== 'idle'" class="shim-panel" :class="{ 'is-close': codexShimClosing, 'is-done': codexShimPhase === 'done', 'is-error': !codexShimResultOk }" data-testid="codex-shim-panel"><div class="shim-panel-head"><span class="shim-panel-spinner" aria-hidden="true"></span><b data-testid="codex-shim-panel-title">{{ codexShimPanelTitle }}</b><span class="shim-panel-meta">{{ codexShimElapsedText }}</span></div><div v-if="!codexShimClosing" class="shim-panel-bar" role="progressbar" aria-label="官方 shim 写入进度"><div class="shim-panel-fill"></div></div><div v-if="codexShimPanelHint" class="shim-panel-hint">{{ codexShimPanelHint }}</div><div v-if="codexShimResult" class="shim-panel-result" :class="{ error: !codexShimResultOk }" data-testid="codex-shim-result">{{ codexShimResult }}</div><button class="shim-details-toggle" type="button" :aria-expanded="codexShimShowDetails" @click="codexShimShowDetails = !codexShimShowDetails"><span class="chev" :class="{ open: codexShimShowDetails }" aria-hidden="true">▸</span> 查看执行的具体指令</button><div v-if="codexShimShowDetails" class="shim-details" data-testid="codex-shim-details"><ol class="shim-cmds"><li v-for="(c, i) in codexShimPlan.commands" :key="i" :data-cmd-state="codexShimPhase === 'done' && codexShimResultOk ? 'done' : 'todo'"><code>{{ c.cmd }}</code><span>{{ c.note }}</span></li></ol><p class="shim-details-note">写入严格经官方 CLI 转发，管理器不直接改写官方配置或启动器二进制。</p></div></div></div></div></article>
      <div class="section-actions"><button class="btn ghost" @click="restoreGeneralDefaults">还原通用默认</button></div>
    </section>

    <section v-else-if="section === 'installation'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>运行环境与安装发现</h2><p>只发现 / 展示 / 校验现有环境与安装。</p></div><button class="btn ghost" :disabled="app.environmentLoading" @click="refreshEnvironment()">重新发现</button></div>
        <div v-if="environment.blocked" class="empty" data-testid="install-blocked">已按前置条件暂停 OpenCodex 发现；请先处理概览中的环境门禁。</div>
        <div v-else-if="!installRows.length" class="empty">正在探测本机 OpenCodex 安装…</div>
        <div v-else class="install-grid"><div v-for="row in installRows" :key="row[0]" class="install-row"><label>{{ row[0] }}</label><div :class="{ mono: row[0] === 'Node.js' || row[0] === 'npm' || row[0] === '可执行文件' }">{{ row[1] }}</div></div></div>
      </article>
      <article class="card" data-testid="runtime-source-card">
        <div class="card-head"><div><h2>OpenCodex 运行来源</h2><p>应用不读取 PATH；来源只来自这里的记录与候选目录，并与发现、启停、版本检查共用同一份结果。</p></div><button class="btn ghost" :disabled="app.runtimeSourceLoading" @click="app.loadRuntimeSource()">刷新</button></div>
        <div v-if="app.runtimeSourceError" class="empty">{{ app.runtimeSourceError }}</div>
        <div v-else-if="!app.runtimeSource" class="empty">正在读取运行来源…</div>
        <template v-else>
          <div class="runtime-facts">
            <div class="runtime-fact"><label>来源类型</label><div>{{ runtimeSourceLabel }} <span class="tag">{{ sourceLandingLabel(app.runtimeSource.insideDataRoot) }}</span></div></div>
            <div class="runtime-fact"><label>路径</label><div class="runtime-path">{{ app.runtimeSource.path ?? '未解析' }}</div></div>
            <div class="runtime-fact"><label>版本</label><div>{{ app.runtimeSource.version ?? '未知' }}</div></div>
          </div>
          <div class="runtime-actions">
            <button class="btn primary" type="button" data-testid="runtime-install" @click="openInstallModal('registry')">安装 OpenCodex</button>
            <button class="btn ghost" type="button" data-testid="runtime-import-offline" @click="openInstallModal('offline')">导入离线包</button>
            <button class="btn ghost" type="button" @click="chooseRuntimeSource">更换运行来源…</button>
            <button class="btn ghost" type="button" :disabled="!app.runtimeSource.explicitPath" @click="app.restoreDiscoveredRuntime()">恢复自动发现</button>
            <button v-if="app.runtimeSource.kind === 'managed'" class="btn ghost" type="button" @click="routes.go('settings', { section: 'upgrade' })">更新（官方 ocx update）</button>
            <button v-if="app.runtimeSource.kind !== 'unresolved'" class="btn danger" type="button" data-testid="runtime-uninstall" @click="openUninstall()">卸载</button>
          </div>
          <p class="env-bound">应用不读取 PATH；来源只来自这里的记录与候选目录。未解析时请从上面的两个出口入手。</p>
        </template>
      </article>
      <article class="card"><div class="card-head"><div><h2>数据目录与 OPENCODEX_HOME</h2><p>自动发现当前路径，可在同一处统一修改。</p></div></div>
        <div v-if="app.dataRootConfigError" class="empty">数据目录配置读取失败；请重新进入安装配置。</div>
        <div v-else-if="!app.dataRootConfig" class="empty">正在读取数据目录配置…</div>
        <table v-else class="data-root-table"><thead><tr><th>路径</th><th>用途</th><th>操作</th></tr></thead><tbody>
          <tr><td><button class="cli-copy-btn" @click="copyPath(app.dataRootConfig.activeDataRoot)"><code>{{ app.dataRootConfig.activeDataRoot }}</code></button></td><td>数据目录{{ app.dataRootConfig.runtimeActive ? '' : ' · 需重启' }}</td><td><button class="btn ghost" @click="openPath(app.dataRootConfig.activeDataRoot, '数据目录')">打开</button></td></tr>
          <tr><td><button class="cli-copy-btn" @click="copyPath(app.dataRootConfig.opencodexHome)"><code>{{ app.dataRootConfig.opencodexHome }}</code></button></td><td>OPENCODEX_HOME · {{ app.dataRootConfig.opencodexHomeMode === 'external' ? '外部路径' : '数据目录内' }}</td><td><button class="btn ghost" @click="openPath(app.dataRootConfig.opencodexHome, 'OPENCODEX_HOME')">打开</button></td></tr>
        </tbody></table>
        <div class="field-grid">
          <div class="field"><label for="external-home-input">外部 OPENCODEX_HOME</label><input id="external-home-input" v-model="externalHomeInput" class="input" placeholder="/absolute/path/to/OpenCodexHome" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        </div>
        <div class="controls">
          <button class="btn" :disabled="app.dataRootHomeSaving" @click="applyOpencodexHome('external')">{{ app.dataRootHomeSaving ? '保存中…' : '使用外部路径' }}</button>
          <button class="btn ghost" :disabled="app.dataRootHomeSaving" @click="applyOpencodexHome('inside')">回到数据目录内</button>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>初始化或切换数据目录</h2><p>使用显式绝对路径；先校验结构，成功后才初始化冻结分区。</p></div></div>
        <div class="field-grid">
          <div class="field"><label for="data-root-input">数据目录路径</label><input id="data-root-input" v-model="dataRootInput" class="input" placeholder="/absolute/path/to/OpenCodexData" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        </div>
        <div class="field-grid">
          <div class="field"><label for="switch-root-input">切换目标路径</label><input id="switch-root-input" v-model="switchInput" class="input" placeholder="/absolute/path/to/ExistingOpenCodexData" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        </div>
        <div class="controls">
          <button class="btn" :disabled="app.dataRootSaving" @click="saveDataRoot">{{ app.dataRootSaving ? '校验中…' : '校验并初始化' }}</button>
          <button class="btn ghost" :disabled="app.dataRootSwitching" @click="applyDataRootSwitch(false)">{{ app.dataRootSwitching ? '切换中…' : '仅切换引用' }}</button>
          <button class="btn ghost" :disabled="app.dataRootSwitching" @click="applyDataRootSwitch(true)">{{ app.dataRootSwitching ? '迁移中…' : '迁移数据' }}</button>
          <span v-if="app.dataRootLastValidation === 'valid'" class="tag">结构有效</span>
          <span v-else-if="app.dataRootLastValidation === 'future_version'" class="tag">版本过新</span>
          <span v-else-if="app.dataRootLastValidation === 'corrupted'" class="tag">结构损坏</span>
          <span v-if="app.dataRootError" class="tag">失败：未写入</span>
          <span v-else-if="app.dataRootLastResult" class="tag">{{ app.dataRootLastResult.created ? '已创建' : '已引用' }}</span>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>数据目录分区</h2><p>分区名已按 IMP 冻结；点击路径可复制，右侧可直接打开。</p></div></div>
        <table class="data-root-table"><thead><tr><th>路径</th><th>用途</th><th>操作</th></tr></thead><tbody>
          <tr v-for="row in managedPartitions" :key="row.key"><td><button class="cli-copy-btn" @click="copyPath(row.path)"><code>{{ row.path }}</code></button></td><td>{{ row.label }}</td><td><button class="btn ghost" @click="openPath(row.path, row.label)">打开</button></td></tr>
        </tbody></table>
      </article>
    </section>

    <section v-else-if="section === 'backup'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>数据与备份</h2><p>备份边界基于安装配置里的数据目录。</p></div><button class="btn ghost" @click="routes.go('settings', { section: 'installation' })">打开安装配置</button></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">升级前自动备份</div><div class="setting-desc">进入官方升级引导前先生成可定位备份。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.autoBackupUpgrade" @click="toggle('autoBackupUpgrade', '升级前自动备份')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">导入前自动备份</div><div class="setting-desc">导入校验通过后先备份当前配置，再进入最终确认。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.autoBackupImport" @click="toggle('autoBackupImport', '导入前自动备份')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">同步覆盖前自动备份</div><div class="setting-desc">任何覆盖动作都保留被覆盖侧的可恢复历史。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.autoBackupSync" @click="toggle('autoBackupSync', '同步覆盖前自动备份')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">备份保留策略</div><div class="setting-desc">保留最近 10 份；超出的备份必须由用户显式清理。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'backupRetention' }"><button class="select-trigger" :aria-expanded="openSelect === 'backupRetention' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="备份保留策略" @click="toggleSelect('backupRetention')"><span class="select-value">{{ preferences.backupRetention === '5' ? '最近 5 份' : preferences.backupRetention === '20' ? '最近 20 份' : '最近 10 份' }}</span></button><UiSelectMenu :open="openSelect === 'backupRetention'" @close="openSelect = ''" role="listbox" aria-label="备份保留策略"><button v-for="option in [['5','最近 5 份'],['10','最近 10 份'],['20','最近 20 份']]" :key="option[0]" class="select-option" :class="{ selected: preferences.backupRetention === option[0] }" type="button" role="option" :aria-selected="preferences.backupRetention === option[0] ? 'true' : 'false'" @click="chooseOption('backupRetention', option[0] as PreferencesDto['backupRetention'], option[1])">{{ option[1] }}</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">备份完整性校验</div><div class="setting-desc">生成备份后写入 SHA-256 校验摘要，恢复前先校验；算法在备份契约中固定。</div></div><div class="controls"><span class="setting-readonly">SHA-256</span></div></div>
        </div>
      </article>
    </section>

    <section v-else-if="section === 'extensions'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>Skills 管理</h2><p>控制源目录、同步与资产参与范围。</p></div><button class="btn ghost" @click="routes.go('extensions', { tab: 'skills' })">打开 Skills 列表</button></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">Skills 源目录</div><div class="setting-desc">默认目录；也可以自定义目录，管理器只读该目录。导入与恢复的 Skill 由管理器自行保存。</div></div><div class="controls"><button class="btn ghost" @click="openPath(skillsSourcePath, 'Skills 源目录')">打开</button><button class="btn" @click="chooseSkillsSourceDir()">选择自定义目录</button><button v-if="skillsSourceIsCustom" class="btn ghost" @click="restoreDefaultSkillsSourceDir()">恢复默认</button></div><button class="cli-copy-btn skills-source" title="复制 Skills 源目录" @click="copyPath(skillsSourcePath)"><code>{{ skillsSourcePath }}</code></button></div>
          <div class="setting-row"><div><div class="setting-title">同步方式</div><div class="setting-desc">软链接优先：省空间、与源实时联动；创建失败或安全校验不通过时自动回退为复制。文件复制更稳定，适合迁移和备份快照。改动源目录或同步方式后，已落地的目标不会自动重做——用「检查并修正」对齐。</div></div><div class="controls"><button class="btn ghost" @click="resyncSkills()" title="按当前源目录与同步方式，重新对齐已落地的客户端目标">检查并修正</button><div class="select" :class="{ open: openSelect === 'skillsSyncMode' }"><button class="select-trigger" :aria-expanded="openSelect === 'skillsSyncMode' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="Skills 同步方式" @click="toggleSelect('skillsSyncMode')"><span class="select-value">{{ skillsSyncMethod === 'copy' ? '文件复制' : '软链接优先' }}</span></button><UiSelectMenu :open="openSelect === 'skillsSyncMode'" @close="openSelect = ''" role="listbox" aria-label="Skills 同步方式"><button class="select-option" :class="{ selected: skillsSyncMethod === 'symlink' }" type="button" role="option" :aria-selected="skillsSyncMethod === 'symlink' ? 'true' : 'false'" @click="chooseSkillsSyncMethod('symlink', '软链接优先')">软链接优先</button><button class="select-option" :class="{ selected: skillsSyncMethod === 'copy' }" type="button" role="option" :aria-selected="skillsSyncMethod === 'copy' ? 'true' : 'false'" @click="chooseSkillsSyncMethod('copy', '文件复制')">文件复制</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">备份包含 Skills 目录</div><div class="setting-desc">当前备份是写入前的单文件备份，不含 Skills 目录快照；纳入扩展快照需先扩备份契约。</div></div><div class="controls"><span class="setting-readonly">不含</span></div></div>
          <div class="setting-row"><div><div class="setting-title">导出包含 Skills 目录</div><div class="setting-desc">配置迁移导出时纳入 Skills；导入前先生成可恢复快照。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.exportIncludeSkills" @click="toggle('exportIncludeSkills', '导出包含 Skills 目录')"></button></div></div>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>MCP 管理</h2><p>统一管理 MCP 写入策略与目标。</p></div><button class="btn ghost" @click="routes.go('extensions', { tab: 'mcp' })">打开 MCP 列表</button></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">同名冲突策略</div><div class="setting-desc">目标客户端已有同名 MCP 时一律暂停写入、展示差异并要求确认；不提供静默保留策略。</div></div><div class="controls"><span class="setting-readonly">每次询问</span></div></div>
          <div class="setting-row"><div><div class="setting-title">MCP 配置落点</div><div class="setting-desc">按客户端适配器写入各自 MCP 配置节点。</div></div><div class="controls"><button class="btn ghost" @click="routes.go('extensions', { tab: 'mcp' })">查看落点</button></div></div>
          <div class="setting-row"><div><div class="setting-title">敏感值脱敏</div><div class="setting-desc">URL 令牌、请求头、环境变量与 API Key 默认掩码展示。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.mcpMask" @click="toggle('mcpMask', 'MCP 敏感值脱敏')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">备份包含 MCP 配置</div><div class="setting-desc">当前备份是写入前的单文件备份，不含受管 MCP 定义快照；凭据只以钥匙串引用保存。</div></div><div class="controls"><span class="setting-readonly">不含</span></div></div>
          <div class="setting-row"><div><div class="setting-title">导出包含 MCP 配置</div><div class="setting-desc">迁移导出纳入 MCP 定义；导入前先备份目标客户端配置。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.exportIncludeMcp" @click="toggle('exportIncludeMcp', '导出包含 MCP 配置')"></button></div></div>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>Skills 路径</h2><p>集中查看并打开各客户端 Skills 目录。</p></div></div><div class="agent-path-list"><div v-for="row in skillsPaths" :key="row[1]" class="agent-path-row"><div class="agent-path-app"><span>{{ row[0] }}</span></div><button class="cli-copy-btn agent-path-copy" @click="copyPath(row[1])"><code>{{ row[1] }}</code></button><div class="agent-path-actions"><button class="btn ghost" @click="openPath(row[1], `${row[0]} Skills`)">打开目录</button></div></div></div></article>
      <article class="card"><div class="card-head"><div><h2>MCP 路径</h2><p>集中查看并打开各客户端 MCP 配置。</p></div></div><div class="agent-path-list"><div v-for="row in mcpPaths" :key="row[1]" class="agent-path-row"><div class="agent-path-app"><span>{{ row[0] }}</span></div><button class="cli-copy-btn agent-path-copy" @click="copyPath(row[1])"><code>{{ row[1] }}</code></button><div class="agent-path-actions"><button class="btn ghost" @click="openPath(row[1], `${row[0]} MCP 配置`)">打开配置</button></div></div></div></article>
    </section>

    <section v-else-if="section === 'migration'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>配置迁移</h2><p>导出 / 导入管理器自有配置；文件为未加密配置，真实凭据仍在系统钥匙串，只携带引用。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">导出范围</div><div class="setting-desc">偏好、扩展配置（Skills 清单、MCP 清单、客户端开关、分发方式）与 WebDAV 端点配置；受上方「导出包含」开关控制。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'exportScope' }"><button class="select-trigger" :aria-expanded="openSelect === 'exportScope' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="导出范围" @click="toggleSelect('exportScope')"><span class="select-value">管理器配置</span></button><UiSelectMenu :open="openSelect === 'exportScope'" @close="openSelect = ''" role="listbox" aria-label="导出范围"><button class="select-option selected" type="button" role="option" aria-selected="true" @click="app.showToast('范围固定为管理器自有配置：偏好、扩展配置与 WebDAV 端点配置。')">管理器配置</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">文件保护方式</div><div class="setting-desc">未加密配置 + SHA-256 完整性校验；只用于检测损坏，不是端到端加密，请自行保管导出文件。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'exportEncryption' }"><button class="select-trigger" :aria-expanded="openSelect === 'exportEncryption' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="文件保护方式" @click="toggleSelect('exportEncryption')"><span class="select-value">完整性校验</span></button><UiSelectMenu :open="openSelect === 'exportEncryption'" @close="openSelect = ''" role="listbox" aria-label="文件保护方式"><button class="select-option selected" type="button" role="option" aria-selected="true" @click="app.showToast('当前为明文配置 + SHA-256 完整性校验，不含额外口令。')">完整性校验</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">导入流程</div><div class="setting-desc">校验格式 → 备份当前配置 → 用户确认 → 应用；旧版加密容器会单独索要原口令。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'importFlow' }"><button class="select-trigger" :aria-expanded="openSelect === 'importFlow' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="导入流程" @click="toggleSelect('importFlow')"><span class="select-value">严格校验</span></button><UiSelectMenu :open="openSelect === 'importFlow'" @close="openSelect = ''" role="listbox" aria-label="导入流程"><button class="select-option selected" type="button" role="option" aria-selected="true" @click="app.showToast('当前流程固定为严格校验、备份确认与失败保留。')">严格校验</button></UiSelectMenu></div></div></div>
        </div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">容器不含以下内容</div><div class="setting-desc">{{ (app.migrationLastExport?.excluded ?? app.migrationLastImport?.excluded ?? []).join('；') || '钥匙串口令密文、MCP 环境变量值、Skills 文件内容、官方运行配置。' }}</div></div></div>
          <div v-if="app.migrationLastExport || app.migrationLastImport" class="setting-row"><div><div class="setting-title">最近迁移结果</div><div class="setting-desc">只显示格式、区段、备份编号与文档摘要，不展示文件内容。</div></div><div class="controls"><code v-if="app.migrationLastExport">导出 v{{ app.migrationLastExport.formatVersion }} · {{ app.migrationLastExport.sections.join('、') }} · {{ app.migrationLastExport.backupId ?? '无备份' }}</code><code v-if="app.migrationLastImport">导入 v{{ app.migrationLastImport.formatVersion }} · {{ app.migrationLastImport.appliedSections.join('、') }}</code></div></div>
          <div v-if="app.migrationLastImport?.skippedSections?.length" class="setting-row"><div><div class="setting-title">未应用的区段</div><div class="setting-desc">{{ app.migrationLastImport.skippedSections.map(item => `${item.section}：${item.reason}`).join('；') }}</div></div></div>
        </div>
        <div class="controls"><button class="btn" :disabled="app.migrationExporting || app.migrationImporting" @click="exportMigration()">{{ app.migrationExporting ? '导出中…' : '导出配置' }}</button><button class="btn" :disabled="app.migrationExporting || app.migrationImporting" @click="importMigration()">{{ app.migrationImporting ? '导入中…' : '导入配置' }}</button><button class="btn ghost" @click="openManagedTarget('exports')">打开导出目录</button></div>
      </article>
    </section>

    <section v-else-if="section === 'sync'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>WebDAV 同步</h2><p>端点、服务端认证凭据与冲突策略集中在设置里维护；同步载荷为未加密配置 + 完整性校验。</p></div></div>
        <div class="field-grid">
          <div class="field"><label>服务地址</label><input v-model="syncEndpointInput.baseUrl" class="input" placeholder="https://dav.example.com/opencodex/" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label>远端目录</label><input v-model="syncEndpointInput.remotePath" class="input" placeholder="/desktop-sync/current" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label>账号</label><input v-model="syncEndpointInput.username" class="input" placeholder="账号" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label>应用密码</label><input v-model="syncEndpointInput.password" class="input" type="password" autocomplete="new-password" autocapitalize="off" autocorrect="off" spellcheck="false" placeholder="输入应用密码"></div>
          <!-- 只读展示：引擎只实现「每次询问」，四档静默策略从未落码，不能让用户以为可改（§26.1）。
               仍按技术型输入保留关自动大写三属性，避免日后误改成可编辑时被 WKWebView 首字母大写。 -->
          <div class="field"><label>冲突策略</label><input class="input" readonly aria-readonly="true" aria-label="冲突策略" value="每次询问" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        </div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">冷同步</div><div class="setting-desc">运行中不直接写关键文件；覆盖前需要用户确认。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.coldSync" @click="toggle('coldSync', '冷同步')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">载荷保护方式</div><div class="setting-desc">远端保存未加密配置，用 SHA-256 校验完整性；WebDAV 认证口令保存在系统钥匙串，请自行选择可信远端并确保 HTTPS。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'payloadProtection' }"><button class="select-trigger" :aria-expanded="openSelect === 'payloadProtection' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="载荷保护方式" @click="toggleSelect('payloadProtection')"><span class="select-value">完整性校验</span></button><UiSelectMenu :open="openSelect === 'payloadProtection'" @close="openSelect = ''" role="listbox" aria-label="载荷保护方式"><button class="select-option selected" type="button" role="option" aria-selected="true" @click="app.showToast('当前为明文载荷 + SHA-256 完整性校验，不含额外加密口令。')">完整性校验</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">覆盖前备份</div><div class="setting-desc">保留被覆盖侧快照，冲突时可恢复。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.backupBeforeOverwrite" @click="toggle('backupBeforeOverwrite', '覆盖前备份')"></button></div></div>
        </div>
        <div v-if="app.syncConfig?.endpoint" class="setting-row"><div><div class="setting-title">已配置端点</div><div class="setting-desc">{{ app.syncConfig.endpoint.url }} · {{ app.syncConfig.endpoint.remotePath }} · 凭据引用 {{ app.syncConfig.endpoint.credentialRefId.slice(0, 12) }}</div></div><div class="controls"><button class="btn danger" :disabled="app.syncSaving" @click="app.deleteSyncEndpoint(true)">删除端点</button></div></div>
        <div v-if="app.syncStatus" class="setting-row"><div><div class="setting-title">最近连接结果</div><div class="setting-desc">{{ app.syncStatus.message || '连接状态已更新。' }}</div></div><div class="controls"><span class="tag">{{ app.syncStatus.connectionState === 'synced' ? '成功' : app.syncStatus.connectionState === 'failed' ? '失败' : app.syncStatus.connectionState }}</span></div></div>
        <div class="controls"><button class="btn" :disabled="app.syncSaving || app.syncTesting" @click="saveSyncEndpoint()">{{ app.syncSaving ? '保存中…' : '保存端点' }}</button><button class="btn ghost" :disabled="app.syncTesting || app.syncSaving" @click="app.testSyncConnection()">{{ app.syncTesting ? '测试中…' : '测试连接' }}</button><button class="btn ghost" :disabled="app.syncRunning" @click="runSyncNow()">立即同步</button></div>
      </article>
    </section>

    <section v-else-if="section === 'cleanup'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>日志与通知清理</h2><p>自动清理只作用于管理器本地记录；不会清理官方 OpenCodex 服务日志。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">日志自动清理</div><div class="setting-desc">本地日志超过上限或保留期后自动截断。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'logRetention' }"><button class="select-trigger" :aria-expanded="openSelect === 'logRetention' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="日志自动清理" @click="toggleSelect('logRetention')"><span class="select-value">{{ preferences.logRetention === '7d-5000' ? '7 天 / 5,000 行' : preferences.logRetention === '90d-30000' ? '90 天 / 30,000 行' : '30 天 / 10,000 行' }}</span></button><UiSelectMenu :open="openSelect === 'logRetention'" @close="openSelect = ''" role="listbox" aria-label="日志自动清理"><button v-for="option in [['7d-5000','7 天 / 5,000 行'],['30d-10000','30 天 / 10,000 行'],['90d-30000','90 天 / 30,000 行']]" :key="option[0]" class="select-option" :class="{ selected: preferences.logRetention === option[0] }" type="button" role="option" :aria-selected="preferences.logRetention === option[0] ? 'true' : 'false'" @click="chooseOption('logRetention', option[0] as PreferencesDto['logRetention'], option[1])">{{ option[1] }}</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">通知自动清理</div><div class="setting-desc">已读通知超过保留期后自动清理。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'notificationRetention' }"><button class="select-trigger" :aria-expanded="openSelect === 'notificationRetention' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="通知自动清理" @click="toggleSelect('notificationRetention')"><span class="select-value">{{ preferences.notificationRetention === '7' ? '7 天' : preferences.notificationRetention === '90' ? '90 天' : '30 天' }}</span></button><UiSelectMenu :open="openSelect === 'notificationRetention'" @close="openSelect = ''" role="listbox" aria-label="通知自动清理"><button v-for="option in [['7','7 天'],['30','30 天'],['90','90 天']]" :key="option[0]" class="select-option" :class="{ selected: preferences.notificationRetention === option[0] }" type="button" role="option" :aria-selected="preferences.notificationRetention === option[0] ? 'true' : 'false'" @click="chooseOption('notificationRetention', option[0] as PreferencesDto['notificationRetention'], option[1])">{{ option[1] }}</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">启动时清理</div><div class="setting-desc">应用启动时按策略执行一次轻量清理，不阻塞主窗口。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.startupCleanup" @click="toggle('startupCleanup', '启动时清理')"></button></div></div>
          <div class="setting-row"><div><div class="setting-title">清理前备份摘要</div><div class="setting-desc">danger 日志和重要通知会先生成脱敏摘要。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.cleanupBackupSummary" @click="toggle('cleanupBackupSummary', '清理前备份摘要')"></button></div></div>
        </div>
        <div class="controls"><button class="btn ghost" @click="routes.go('logs')">打开诊断中心</button><button class="btn" @click="cleanupLogs()">立即清理日志</button><button class="btn" :disabled="!app.notifications.length" @click="cleanupNotifications()">立即清理通知</button></div>
      </article>
    </section>

    <section v-else-if="section === 'cli'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>CLI 控制面</h2><p>让终端、脚本或 AI 代理控制本应用的自有域；不提供改写 OpenCodex 配置的通道。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">启用 CLI 控制面</div><div class="setting-desc">{{ cliSupported ? '默认关闭；开启后注册本机命令入口。CLI 仅作为运行中实例的客户端，经本地 IPC 委托执行。' : 'Windows 暂不支持 CLI 控制面；本机 IPC 未实现。' }}</div></div><div class="controls"><button class="toggle" role="switch" :disabled="!cliSupported" :aria-checked="cliSupported && preferences.cliEnabled" @click="toggleCli()"></button></div></div>
          <template v-if="cliSupported && preferences.cliEnabled">
            <div class="setting-row"><div><div class="setting-title">本机 IPC 端点</div><div class="setting-desc"><code>~/Library/Caches/OpenCodex Desktop/ipc/opencodex.ipc</code>；目录 0700，socket 0600，仅限当前用户 UID。</div></div><div class="controls"><span class="cli-badge ok">在线</span></div></div>
            <div class="setting-row"><div><div class="setting-title">PATH 注册</div><div class="setting-desc">默认关闭；注册为指向应用内 <code>ocxd</code> 的 symlink，不会覆盖他人目标。</div></div><div class="controls"><span class="cli-badge">未注册</span></div></div>
            <div class="setting-row"><div><div class="setting-title">调用日志</div><div class="setting-desc">记录命令、参数键、请求 ID 与结果；保留 90 天 / 5,000 条，单文件 5 MB × 5 份。在「诊断中心 → 日志历史 → 调用日志」中查看。</div></div><div class="controls"><span class="cli-badge">已启用</span></div></div>
            <div class="setting-row"><div><div class="setting-title">Agent 提示</div><div class="setting-desc">管理器用 <code>ocxd</code>；OpenCodex 自身配置仍用官方 <code>ocx</code>。命令能力先 <code>ocxd --help</code>。</div></div><div class="controls"><button class="btn ghost" @click="copyAgentPrompt()">复制 Agent 提示</button></div></div>
          </template>
        </div>
        <template v-if="cliSupported && preferences.cliEnabled">
          <div class="cli-rules"><h3>开放能力</h3><p>发现与只读状态、启停、数据目录、备份、加密导出 / 导入、WebDAV 同步、自更新检查。</p><h3>明确不开放</h3><p>提供方、路由、模型映射等 OpenCodex 配置仍走官方 CLI（<code>ocx</code>）；管理器 CLI 不做第二个写入者。常用入口见下方「OpenCodex 配置 · 官方 CLI」。</p></div>
          <div class="cli-preview"><h4>先看帮助</h4><pre><code>ocxd --help
ocxd status --json</code></pre><p><code>--help</code> 列出当前版本实际支持的命令与参数；脚本和 Agent 用它自发现能力，不硬编码命令集。</p></div>
          <div class="cli-preview"><h4>管理器自有域命令</h4><pre><code>ocxd status --json
ocxd start --confirm --json
ocxd stop --confirm --json
ocxd restart --confirm --json
ocxd data-root show --json
ocxd data-root switch --target &lt;path&gt; [--migrate] --confirm --json
ocxd backup create --confirm --json
ocxd backup list --json
ocxd export --output &lt;path&gt; --password-stdin --confirm --json
ocxd import --input &lt;path&gt; --password-stdin --confirm --json
ocxd sync run --confirm --json
ocxd update check --json</code></pre><p>变更层命令需运行实例与显式确认；导出 / 导入不再需要口令，仅旧版加密容器导入可用 stdin 传入原口令，不进入 argv、日志或审计值。</p></div>
          <div class="cli-preview official"><h4>OpenCodex 配置 · 官方 CLI 速查</h4><pre><code>ocx --help
ocx status --json
ocx restore
ocx update</code></pre><p>提供方、路由、模型映射等自身配置不属于管理器 CLI；桌面壳只展示入口和检测到的接管 / 存在风险状态，不代理或包装写入动作。</p></div>
          <div class="cli-preview"><pre><code>管理器 CLI：ocxd（先 ocxd --help；仅限管理器自有域）
官方 CLI：ocx（提供方 / 路由 / 模型映射）
规则：变更先 --confirm；导出导入无需口令（旧容器可加 --password-stdin）；机读输出用 --json。</code></pre></div>
        </template>
      </article>
    </section>

    <section v-else-if="section === 'upgrade'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>OpenCodex 版本</h2><p>官方 npm 包与官方面板；桌面管理器不接管更新事务，只做升级前备份与官方引导。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">当前版本</div><div class="setting-desc">{{ officialVersion }} · {{ officialInstallLabel }}</div></div><div class="controls"><button class="btn ghost" :disabled="app.officialProjectLoading || app.officialRemoteLoading" @click="runOfficialCheck()">{{ app.officialProjectLoading || app.officialRemoteLoading ? '检查中' : '检查更新' }}</button></div></div>
          <div class="setting-row" data-testid="official-remote-latest"><div><div class="setting-title">远端最新版本</div><div class="setting-desc">{{ officialRemoteText }} · {{ officialUpdateText }}</div></div><div class="controls"><span class="tag" :class="{ danger: app.officialRemoteError, ok: officialUpdateAvailable === false }">{{ app.officialRemoteError ? '不可用' : officialUpdateAvailable === true ? '有更新' : officialUpdateAvailable === false ? '已最新' : '未比较' }}</span><button class="btn" :disabled="app.officialUpdateBusy || officialUpdateAvailable !== true || app.runtimeInstalling" @click="confirmOfficialUpdate()">{{ app.officialUpdateBusy ? '更新中' : '应用官方更新' }}</button></div></div>
          <div v-if="app.officialUpdateError" class="setting-row"><div><div class="setting-title">更新失败</div><div class="setting-desc">{{ app.officialUpdateError }}</div></div><div class="controls"><span class="tag danger">失败</span></div></div>
          <div class="setting-row"><div><div class="setting-title">升级前备份</div><div class="setting-desc">{{ app.upgradeLastBackup ? `最近备份 ${app.upgradeLastBackup.backupId}` : '先备份当前配置，再进入官方升级引导。' }}</div></div><div class="controls"><button class="btn" :disabled="app.upgradeBackupBusy" @click="app.createUpgradeBackup()">{{ app.upgradeBackupBusy ? '备份中' : '生成备份' }}</button></div></div>
          <div class="setting-row"><div><div class="setting-title">官方升级引导</div><div class="setting-desc">只展示并引导 <code>ocx update</code>；不重写更新事务。</div></div><div class="controls"><button class="btn ghost" @click="openUpgradeGuide">打开引导</button></div></div>
          <div class="setting-row"><div><div class="setting-title">失败后建议</div><div class="setting-desc">展示备份位置、错误摘要和建议恢复动作。</div></div><div class="controls"><button class="btn ghost" @click="openUpgradeAdvice">查看建议</button></div></div>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>桌面管理器版本</h2><p>OpenCodeX-Desktop 自身的应用更新；签名校验通过后才会安装，失败保留当前版本。</p></div></div>
        <div class="setting-list">
        <div class="setting-row"><div><div class="setting-title">当前版本</div><div class="setting-desc">v{{ updateStatus?.currentVersion ?? '0.1.0' }} · {{ updateStatus?.channel === 'beta' ? '测试' : '稳定' }}通道</div></div><div class="controls"><button class="btn ghost" :disabled="updateChecking" @click="runUpdateCheck">{{ updateChecking ? '检查中' : '检查应用更新' }}</button></div></div>
          <div class="setting-row"><div><div class="setting-title">更新结果</div><div class="setting-desc">{{ updateResultText }}</div></div><div class="controls"><span class="tag" :class="{ danger: updateStatus?.error }">{{ updateStatus?.error ? '失败' : updateStatus?.availableVersion ? '有更新' : '已检查' }}</span><button class="btn" :disabled="updateInstallDisabled" @click="installUpdate">{{ app.appUpdateBusy ? '安装中' : '安装更新' }}</button></div></div>
          <div class="setting-row"><div><div class="setting-title">安装完成后</div><div class="setting-desc">{{ app.appUpdateError || '签名校验通过后由桌面壳接管重启；不会停止托管代理。' }}</div></div><div class="controls"><button class="btn ghost" :disabled="updateInstallDisabled" @click="installUpdate">重新安装</button></div></div>
<div class="setting-row"><div><div class="setting-title">更新通道</div><div class="setting-desc">稳定通道或测试通道；检查、安装与自动调度共用同一通道来源。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'appUpdateChannel' }"><button class="select-trigger" :aria-expanded="openSelect === 'appUpdateChannel' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="应用更新通道" @click="toggleSelect('appUpdateChannel')"><span class="select-value">{{ preferences.appUpdateChannel === 'beta' ? '测试通道' : '稳定通道' }}</span></button><UiSelectMenu :open="openSelect === 'appUpdateChannel'" @close="openSelect = ''" role="listbox" aria-label="应用更新通道"><button class="select-option" :class="{ selected: preferences.appUpdateChannel === 'stable' }" type="button" role="option" :aria-selected="preferences.appUpdateChannel === 'stable' ? 'true' : 'false'" @click="chooseUpdateChannel('stable', '稳定通道')">稳定通道</button><button class="select-option" :class="{ selected: preferences.appUpdateChannel === 'beta' }" type="button" role="option" :aria-selected="preferences.appUpdateChannel === 'beta' ? 'true' : 'false'" @click="chooseUpdateChannel('beta', '测试通道')">测试通道</button></UiSelectMenu></div></div></div><div class="setting-row" data-testid="setting-network-shortcut"><div><div class="setting-title">网络连接</div><div class="setting-desc">检查更新失败多为网络问题；可在此跳转配置代理。</div></div><div class="controls"><button class="btn ghost" @click="goToNetwork">配置网络</button></div></div>
<div class="setting-row"><div><div class="setting-title">自动检查更新</div><div class="setting-desc">开启后按通道后台检查；关闭后不影响手动检查。</div></div><div class="controls"><button class="toggle" role="switch" :aria-checked="preferences.appUpdateAutoCheck" @click="toggleUpdateAutoCheck()"></button></div></div>
        </div>
      </article>
    </section>

    <section v-else-if="section === 'about'" class="settings-panel active">
      <article class="card about-card">
        <article class="about-hero">
          <div class="about-head">
            <div>
              <span class="about-kicker">官方项目</span>
              <h2>{{ officialProject?.displayName ?? 'OpenCodex' }}</h2>
              <p>被管理的外部项目；本应用仅负责本机发现、运行托管与状态观测。</p>
            </div>
            <span class="tag" :class="{ danger: app.officialProjectError }">
              {{ officialProjectState }}
            </span>
          </div>
          <dl class="about-facts">
            <div><dt>项目版本</dt><dd><span class="mono">{{ officialVersion }}</span></dd></div>
            <div><dt>发行包</dt><dd><span class="mono">@bitkyc08/opencodex</span></dd></div>
            <div><dt>代码作者</dt><dd><strong>lidge-jun</strong></dd></div>
            <div><dt>项目维护者</dt><dd><strong>bitkyc08</strong></dd></div>
          </dl>
          <div class="about-links">
            <button class="link-chip" @click="openExternal(OFFICIAL_HOME)">官网</button>
            <button class="link-chip" @click="openExternal(OFFICIAL_REPO)">GitHub</button>
            <button class="link-chip" @click="openExternal(OFFICIAL_ISSUES)">Issues</button>
          </div>
        </article>
        <article class="about-panel">
          <div class="about-head">
            <div>
              <span class="about-kicker">本应用</span>
              <h3>{{ app.aboutApp?.name ?? 'OpenCodeX-Desktop' }}</h3>
              <p>独立的桌面壳封装，不是 OpenCodex 官方作品。</p>
            </div>
            <span class="tag">{{ appVersion }}</span>
          </div>
          <dl class="about-facts">
            <div><dt>应用版本</dt><dd><span class="mono">{{ appVersion }}</span></dd></div>
            <div><dt>项目作者</dt><dd><strong>gzers</strong></dd></div>
            <div><dt>运行平台</dt><dd>{{ app.aboutApp?.platform ?? '检测中' }} · {{ app.aboutApp?.framework ?? 'Tauri v2' }}</dd></div>
            <div><dt>开源许可</dt><dd>{{ app.aboutApp?.license ?? 'MIT License' }}</dd></div>
          </dl>
          <div class="about-links">
            <button class="link-chip" @click="openExternal(APP_REPO)">本应用 GitHub</button>
            <button class="link-chip" @click="openExternal(APP_ISSUES)">Issues</button>
            <button class="link-chip" @click="openLocalDocument('license')">查看 MIT 许可</button>
          </div>
        </article>
        <article class="about-panel">
          <div class="about-head">
            <div>
              <span class="about-kicker">版权与归属</span>
              <h3>许可、归属与免责声明</h3>
            </div>
          </div>
          <p class="about-note">本项目与 OpenCodex 官方项目及其权利人不存在隶属、授权或背书关系。仅在必要范围内使用项目名称、版本与链接进行兼容性说明，不复制或重新分发 OpenCodex 源代码、构建产物、图标、界面素材或商标标识。OpenCodex 的源代码、名称、商标及其他权利仍归其权利人所有；使用官方软件时请以官方仓库中的 LICENSE、NOTICE 和服务条款为准。本项目自身代码与素材按 MIT License 发布，二者许可彼此独立。</p>
          <div class="about-actions">
            <button class="link-chip" @click="openExternal(OFFICIAL_LICENSE)">查看官方许可</button>
            <button class="link-chip" @click="openLocalDocument('third-party')">查看第三方声明</button>
          </div>
        </article>
      </article>
    </section>
  </section>

  <div v-if="installOpen" class="modal-mask" data-testid="runtime-install-modal">
    <div class="modal modal-wide wiz" role="dialog" aria-modal="true" aria-labelledby="wiz-title">
      <h3 id="wiz-title">安装 OpenCodex</h3>
      <ol class="wiz-steps">
        <li v-for="n in 4" :key="n" :class="{ active: installStep === n, done: installStep > n }"><span>{{ n }}</span></li>
      </ol>

      <div v-if="installStep === 1" class="wiz-body">
        <div class="field"><label for="wiz-prefix">安装位置</label><input id="wiz-prefix" v-model="installPrefix" class="input" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        <p class="wiz-hint">{{ installLandingText }}</p>
        <p v-if="installPrefixError" class="wiz-error">{{ installPrefixError }}</p>
        <div class="wiz-actions">
          <button class="btn ghost" type="button" @click="chooseInstallPrefix">选择…</button>
          <button class="btn ghost" type="button" @click="resetInstallPrefix">恢复默认</button>
        </div>
      </div>

      <div v-else-if="installStep === 2" class="wiz-body">
        <label class="wiz-choice"><input v-model="installSource" type="radio" value="registry">联网安装（官方包 @bitkyc08/opencodex）</label>
        <label class="wiz-choice"><input v-model="installSource" type="radio" value="offline">导入离线包（file.tgz）</label>

        <div v-if="installSource === 'offline'" class="wiz-drop" data-testid="wiz-offline">
          <p>把 <code>opencodex-&lt;version&gt;.tgz</code> 拖到这里，或选择文件</p>
          <div class="wiz-actions"><button class="btn ghost" type="button" @click="chooseOfflinePackage">选择文件…</button></div>
          <p v-if="app.offlinePreview" class="wiz-hint" :class="{ 'wiz-error': !app.offlinePreview.ok }">
            {{ app.offlinePreview.fileName }} · {{ formatBytes(app.offlinePreview.fileSize) }} ·
            {{ app.offlinePreview.ok ? `SHA-256 ${app.offlinePreview.sha256Prefix}…` : app.offlinePreview.message }}
          </p>
          <p class="wiz-hint">离线包不联网，但仍会写入数据根内的私有前缀；不匹配时拒绝，不回退成「任意 tgz 都装」。</p>
          <details class="wiz-more"><summary>下载地址</summary><p>npm 包页面：https://www.npmjs.com/package/@bitkyc08/opencodex</p></details>
        </div>

        <div v-else class="wiz-proxy" data-testid="wiz-proxy">
          <div class="field"><label>代理（可选）</label>
            <div class="wiz-inline">
              <select v-model="installProxy.scheme" class="input"><option value="http">HTTP</option><option value="socks5h">SOCKS5</option></select>
              <input v-model="installProxy.host" class="input" placeholder="host:port" type="text" autocapitalize="off" autocorrect="off" spellcheck="false">
            </div>
          </div>
          <details class="wiz-more"><summary>代理需要认证</summary>
            <div class="wiz-inline">
              <input v-model="installProxy.username" class="input" placeholder="用户名" type="text" autocapitalize="off" autocorrect="off" spellcheck="false">
              <input v-model="installProxy.secret" class="input" placeholder="密码" type="password">
            </div>
            <p class="wiz-hint">凭据只用于本次安装：写进临时配置文件（Unix 0600，用完即删），不进命令行参数、不写系统代理、不落盘。</p>
          </details>
          <p class="wiz-hint">{{ proxyHint }}</p>
        </div>
      </div>

      <div v-else-if="installStep === 3" class="wiz-body">
        <label class="wiz-choice"><input v-model="installUseSpecificVersion" type="checkbox">指定版本</label>
        <div v-if="installUseSpecificVersion" class="field"><input v-model="installVersion" class="input" placeholder="例如 2.50.0" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        <p v-else class="wiz-hint">安装最新版（latest）。</p>
        <label class="wiz-choice"><input v-model="installConfirmed" type="checkbox">我已确认安装位置与安装源</label>
        <details class="wiz-more"><summary>说明 / 安全边界</summary>
          <p class="wiz-hint">默认以 <code>--ignore-scripts</code> 安装；若包必须执行安装脚本才能生成入口，会先回来征求你的二次确认。</p>
        </details>
      </div>

      <div v-else class="wiz-body">
        <div class="wiz-progress"><div class="wiz-progress-bar" :style="{ width: `${installProgress?.percent ?? 0}%` }"></div></div>
        <p class="wiz-status" data-testid="wiz-status">
          {{ installPhaseLabel(installProgress?.phase ?? 'preparing') }}
          <template v-if="app.runtimeInstallOutcome"> · {{ app.runtimeInstallOutcome.version }} · {{ installSource === 'registry' ? '联网安装' : '离线导入' }}</template>
          · {{ installProgress?.percent ?? 0 }}%
        </p>
        <pre v-if="showInstallCommandLine" class="wiz-console" data-testid="wiz-console">{{ app.runtimeInstallLines.join('\n') || '等待 npm 输出…' }}</pre>
        <ul v-else class="wiz-substeps" data-testid="wiz-substeps">
          <li v-for="step in offlineStepList" :key="step.label" :class="`is-${step.state}`">{{ step.label }}</li>
        </ul>
        <p v-if="app.runtimeInstallError" class="wiz-error" data-testid="wiz-error">{{ app.runtimeInstallError }}</p>
        <p v-if="app.runtimeInstallOutcome" class="wiz-hint" data-testid="wiz-outcome">
          入口 {{ app.runtimeInstallOutcome.entry }} · SHA-256 {{ app.runtimeInstallOutcome.tarballSha256.slice(0, 16) }}…
          <template v-if="app.runtimeInstallOutcome.needsRestart"> · 代理正在运行，重启后生效。</template>
        </p>
      </div>

      <div class="wiz-foot">
        <button class="btn ghost" type="button" @click="cancelInstall()">{{ app.runtimeInstalling ? '取消安装' : '取消' }}</button>
        <button v-if="installStep < 4" class="btn primary" type="button" :disabled="!installCanAdvance" @click="advanceInstall()">下一步</button>
        <template v-else-if="app.runtimeInstallError">
          <button class="btn primary" type="button" :disabled="!installAllowScripts" data-testid="wiz-allow-scripts" @click="installAllowScripts = true; runInstall()">允许安装脚本并重试</button>
          <button class="btn ghost" type="button" @click="closeInstallModal()">关闭</button>
        </template>
        <button v-else class="btn primary" type="button" data-testid="wiz-finish" @click="finishInstall()">完成</button>
      </div>
    </div>
  </div>

  <div v-if="uninstallOpen" class="modal-mask" data-testid="runtime-uninstall-modal">
    <div class="modal wiz wiz-uninstall" role="dialog" aria-modal="true" aria-labelledby="uninstall-title">
      <h3 id="uninstall-title">卸载 OpenCodex</h3>

      <template v-if="!uninstallRunning && !uninstallDone">
        <p class="wiz-hint">按当前运行来源 <code>{{ app.runtimeUninstallPlan?.sourcePath ?? '未解析' }}</code> 卸载；范围越大后果越重。</p>
        <div class="wiz-choices">
          <label class="wiz-choice-block" :class="{ on: uninstallScope === 'full' }">
            <input v-model="uninstallScope" type="radio" value="full" data-testid="uninstall-scope-full">
            <span><b>完整卸载（推荐）</b><small>先由应用执行官方 <code>ocx uninstall</code> 清理 service / shim / config，再移除包体与入口。</small></span>
          </label>
          <label class="wiz-choice-block" :class="{ on: uninstallScope === 'body' }">
            <input v-model="uninstallScope" type="radio" value="body" data-testid="uninstall-scope-body">
            <span><b>仅移除包体与入口</b><small>只删包体与入口；<b>不碰</b> service / shim / config / OPENCODEX_HOME。</small></span>
          </label>
        </div>

        <p v-if="app.runtimeUninstallPlanLoading" class="wiz-hint">正在读取卸载方案…</p>
        <p v-else-if="app.runtimeUninstallPlanError" class="wiz-error" data-testid="uninstall-plan-error">{{ app.runtimeUninstallPlanError }}</p>
        <template v-else-if="app.runtimeUninstallPlan">
          <p v-if="!app.runtimeUninstallPlan.removable" class="wiz-error" data-testid="uninstall-not-removable">{{ app.runtimeUninstallPlan.reason }}</p>

          <details class="wiz-more" data-testid="uninstall-plan-objects">
            <summary>将移除的对象：包体 · 入口{{ uninstallScope === 'full' ? ' · 运行态' : '' }}</summary>
            <ul class="wiz-objects">
              <li v-for="item in app.runtimeUninstallPlan.removeObjects" :key="item"><code>{{ item }}</code></li>
              <li v-for="item in (uninstallScope === 'full' ? app.runtimeUninstallPlan.runtimeObjects : [])" :key="item"><span>{{ item }}</span></li>
            </ul>
            <p v-if="app.runtimeUninstallPlan.externalCommand" class="wiz-hint">将执行：<code>{{ app.runtimeUninstallPlan.externalCommand }}</code></p>
            <p v-if="uninstallScope === 'full' && app.runtimeUninstallPlan.officialCommand" class="wiz-hint">完整卸载将执行：<code>{{ app.runtimeUninstallPlan.officialCommand }}</code></p>
          </details>

          <p v-if="app.runtimeUninstallPlan.backupId" class="wiz-backup ok" data-testid="uninstall-backup">已检测到可恢复备份 <code>{{ app.runtimeUninstallPlan.backupId }}</code>。</p>
          <p v-else class="wiz-backup warn" data-testid="uninstall-backup">未检测到可恢复备份；卸载后不可恢复。<b>不阻断卸载</b>，但建议先备份。</p>

          <div class="wiz-opts">
            <label class="wiz-opt" data-testid="uninstall-auto-backup"><input v-model="uninstallAutoBackup" type="checkbox"> 卸载前自动生成备份</label>
            <label class="wiz-opt" :class="{ disabled: uninstallScope !== 'full' }" data-testid="uninstall-clean-data"><input v-model="uninstallCleanData" type="checkbox" :disabled="uninstallScope !== 'full'"> 清空 OPENCODEX_HOME 残留{{ uninstallScope === 'full' ? '' : '（仅完整卸载）' }}</label>
          </div>
        </template>

        <label class="wiz-ack wiz-ack-sticky" :class="{ on: uninstallAck }">
          <input v-model="uninstallAck" type="checkbox" data-testid="uninstall-ack">
          <span><b>我已阅读清单并确认继续</b><small>停代理 → 备份{{ uninstallScope === 'full' ? ' → 执行官方 ocx uninstall' : '' }} → 移除包体与入口 → 残留核验（全部由应用代执行）。</small></span>
        </label>
        <p v-if="!uninstallAck" class="wiz-ack-hint" data-testid="uninstall-ack-hint">勾选上面这一项，「卸载」才会亮起。</p>
      </template>

      <template v-else>
        <div class="wiz-uview">
          <div class="wiz-uhead">
            <span
              class="wiz-spin"
              :class="{ done: uninstallDone, faulted: uninstallPartialFailure }"
              aria-hidden="true"
            >{{ uninstallDone ? (uninstallPartialFailure ? '!' : '✓') : '' }}</span>
            <b data-testid="uninstall-progress-title">{{ uninstallDone ? (uninstallPartialFailure ? `卸载完成但有 ${uninstallFailedSteps} 步失败` : '卸载完成') : '正在卸载 OpenCodex' }}</b>
            <span class="wiz-umeta">{{ uninstallScope === 'full' ? '完整卸载' : '仅移除包体' }}<template v-if="uninstallDone">{{ uninstallPartialFailure ? ` · ${uninstallFailedSteps} 步失败` : ' · 100%' }}</template></span>
          </div>
          <div class="wiz-ubar" role="progressbar" aria-label="卸载进度"><div class="wiz-ufill" :class="{ done: uninstallDone && !uninstallPartialFailure, faulted: uninstallPartialFailure, run: !uninstallDone }"></div></div>
          <ul class="wiz-substeps" data-testid="uninstall-steps">
            <li v-for="row in uninstallStepRows" :key="row.name" :class="row.cls"><span><b>{{ row.name }}</b>{{ row.detail ? ' · ' + row.detail : '' }}</span></li>
          </ul>
          <pre v-if="app.runtimeUninstallResult && app.runtimeUninstallResult.officialOutput.length" class="wiz-console wiz-console-fixed" data-testid="uninstall-console">{{ app.runtimeUninstallResult.officialOutput.join('\n') }}</pre>
        </div>
        <div v-if="uninstallDone" class="residue-card" data-testid="uninstall-residue">
          <div class="residue-head"><span class="residue-badge" aria-hidden="true">{{ residueOk ? '✓' : '!' }}</span><b>残留核验</b><small>{{ residueOk ? '0 残留' : '有未清除项' }}</small></div>
          <ul class="residue-list">
            <li v-for="item in (app.runtimeUninstallResult?.residue ?? [])" :key="item.path" :class="item.status"><code>{{ item.path }}</code><span>{{ residueLabel(item.status) }}</span></li>
          </ul>
          <p v-if="app.runtimeUninstallResult?.backupId" class="wiz-hint">备份 <code>{{ app.runtimeUninstallResult.backupId }}</code> · <button class="cli-copy-btn" type="button" @click="copyPath(app.runtimeUninstallResult?.backupDirectory ?? '')"><code>{{ app.runtimeUninstallResult?.backupDirectory }}</code></button></p>
        </div>
      </template>

      <p v-if="app.runtimeUninstallError" class="wiz-error" data-testid="uninstall-error">{{ app.runtimeUninstallError }}</p>

      <div class="wiz-foot">
        <button class="btn ghost" type="button" :disabled="uninstallRunning" @click="closeUninstall()">{{ uninstallDone ? '关闭' : '取消' }}</button>
        <button
          v-if="!uninstallDone"
          class="btn danger"
          type="button"
          data-testid="uninstall-confirm"
          :disabled="!uninstallCanConfirm"
          @click="confirmUninstall()"
        >
          {{ uninstallRunning ? '卸载中…' : '卸载' }}
        </button>
        <button v-else class="btn primary" type="button" data-testid="uninstall-finish" @click="closeUninstall()">完成</button>
      </div>
    </div>
  </div>

</template>
