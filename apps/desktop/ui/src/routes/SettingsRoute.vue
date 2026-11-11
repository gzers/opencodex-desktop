<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import UiCard from '@/components/ui/UiCard.vue'
import BackupManagement from '@/features/backup/BackupManagement.vue'
import NativeConfigRollback from '@/features/backup/NativeConfigRollback.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import SettingRow from '@/components/patterns/SettingRow.vue'
import { useAppStore } from '@/stores/app'
import { useRuntimeStore } from '@/features/runtime/store'
import { useRouteStore } from '@/stores/routes'
import UiSegmented from '@/components/ui/UiSegmented.vue'
import AgentExecutionField from '@/features/models/AgentExecutionField.vue'
import RuntimeActivationRecovery from '@/features/runtime/RuntimeActivationRecovery.vue'
import RuntimePortSettings from '@/features/runtime/RuntimePortSettings.vue'
import { settingsSections } from '@/navigation'
import { checkNetworkProxy, type PreferencesDto } from '@/features/preferences/api'
import { getCodexShimStatus, setCodexShim, type CodexShimDto } from '@/features/codex-shim/api'
import UpdateCenter from '@/features/updates/UpdateCenter.vue'
import { applyInterfaceScale, clampScale, DEFAULT_INTERFACE_SCALE } from '@/app/appearance/scale'
import { useGlowRenderStore } from '@/app/appearance/glowRender'
import type { InstallSourceKind } from '@/features/runtime/api'
import { displayPath } from '@/lib/paths'
import { useDataRootStore } from '@/features/data-root/store'
import { type PathKeyV2, type PathStatusV2 } from '@/features/data-root/api'

const routes = useRouteStore()
const app = useAppStore()
const runtimeProtection = useRuntimeStore()
const protectionBlocked = computed(() => runtimeProtection.protectionRetrying
  || !!runtimeProtection.protectionError
  || (!!runtimeProtection.protection && runtimeProtection.protection.state !== 'none'))
const protectionDescription = computed(() => {
  switch (runtimeProtection.protection?.state) {
    case 'pending': return '上次面板更新的安装证据尚未核验，关联备份继续保留。核对完成前无法安装、卸载、更换运行来源或切换数据目录。'
    case 'verified_pending': return '安装证据已持久核验，备份关联的完成记录仍待处理。重试只处理本地记录，不重新安装面板。'
    case 'unreadable': return '保护记录无法安全读取或内容无效。记录与备份均保留；重试不会删除记录或强制解除保护，请结合诊断日志排查。'
    default: return runtimeProtection.protectionLoading ? '正在读取本地保护记录…' : runtimeProtection.protectionFeedback
  }
})
const dataRoots = useDataRootStore()
const pathDialog = ref<'details' | 'initialize' | 'json' | null>(null)
const pathLabels: Record<PathKeyV2, string> = {
  aggregateRoot: '用户配置根', desktopRoot: 'Desktop 数据', opencodexHome: 'OpenCodex HOME',
  configPath: '渠道配置文件', installationRoot: 'Desktop 安装根', managerExecutable: 'Desktop 本体',
  runtimePrefix: '运行包目录', runtimeEntry: '程序入口',
}
const userPathKeys: PathKeyV2[] = ['aggregateRoot', 'desktopRoot', 'opencodexHome', 'configPath']
const programPathKeys: PathKeyV2[] = ['installationRoot', 'managerExecutable', 'runtimePrefix', 'runtimeEntry']
const pathStatusLabels: Record<PathStatusV2, string> = { valid: '位置有效', missing: '缺失', unreadable: '无法读取', corrupted: '记录损坏' }
const restartPending = computed(() => dataRoots.pendingRestart || dataRoots.reconciliationRequired || dataRoots.paths?.pendingRestart || dataRoots.config?.runtimeActive === false)
const dataRootBindingDisabled = computed(() => protectionBlocked.value || runtimeProtection.protectionLoading || !runtimeProtection.protection
  || app.runtimeInstalling || app.runtimeUninstallBusy || dataRoots.switching || dataRoots.homeSaving
  || dataRoots.saving || restartPending.value || !dataRoots.config || dataRoots.configError
  || dataRoots.pathsLoading || dataRoots.pathsStale || !dataRoots.paths)
const programValidity = computed(() => {
  const p = dataRoots.paths
  if (!p) return '尚未核验'
  if (p.program.consistency === 'missing') return '尚未安装（安装证据缺失）'
  if (p.program.consistency === 'mismatch') return '包 / 清单 / 来源不一致，待核对'
  if (p.program.consistency === 'unreadable') return '安装证据无法读取，待核对'
  if (programPathKeys.some(key => p[key].status !== 'valid')) return '安装位置异常，待核对'
  return '安装证据一致；运行状态另行核验'
})
function refreshPaths() {
  void dataRoots.loadPaths()
  void dataRoots.loadConfig()
}
function closePathDialog() { pathDialog.value=null }
async function copyActualPaths() {
  const p = dataRoots.paths
  if (!p) return
  try {
    await navigator.clipboard.writeText([...userPathKeys, ...programPathKeys].map(key => p[key].path).join('\n'))
    app.showToast('完整路径已复制。')
  } catch { app.showToast('复制失败；可选择文本手动复制。') }
}
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
const glowRender = useGlowRenderStore()
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
const updateCenter = ref<InstanceType<typeof UpdateCenter> | null>(null)
const updateOpen = ref(false)
const updateTarget = ref<'manager' | 'official' | null>(null)
function openUpdates(target: 'manager' | 'official' | null = null, check = false) {
  updateTarget.value = target
  updateOpen.value = true
  if (check) void updateCenter.value?.check()
}
// 设置页每次进入都会重新挂载，而偏好通常在启动阶段就已加载完成；
// 若只监听「变化」，挂载后再也不会触发，界面会停留在默认值（100）而与实际/持久化值不符。
// 因此立即同步一次已有的偏好，保证「显示值 = 实际比例 = 持久化值」。
watch(() => app.preferences, value => {
  if (!value) return
  preferences.value = { ...preferences.value, ...value }
}, { immediate: true })
// 界面缩放的 DOM 镜像跟随生效偏好（含加载、保存成功与还原默认值）。
watch(() => routes.settingsSection, section => {
  if (section === 'general') void loadCodexShim()
  if (section === 'installation') {
    void app.loadDataRootConfig()
    void dataRoots.loadPaths()
  }
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
  if (dataRootBindingDisabled.value || !dataRoots.paths) return
  const saved = await dataRoots.save(dataRoots.paths.aggregateRoot.path)
  if (saved) { closePathDialog(); app.showToast('新版用户目录已初始化。') }
  await dataRoots.loadPaths()
}
const officialProject = computed(() => app.officialProject)
const officialVersion = computed(() => {
  if (app.officialProjectLoading && !app.officialProject) return '检查中…'
  if (app.officialProjectError && !app.officialProject) return '未发现'
  const version = app.officialProject?.version ?? app.environment?.ocx.version
  return version ? `v${version}` : '未发现'
})
const officialProjectState = computed(() => {
  if (app.officialProjectLoading && !app.officialProject) return '检查中'
  if (app.officialProjectError) return app.officialProject ? '上次结果' : '未发现'
  return app.officialProject?.truncated ? '已截断' : '外部项目'
})
const appVersion = computed(() => app.aboutApp ? `v${app.aboutApp.version}` : 'v0.1.0')
// 引导安装事件在挂载时注册、卸载时成对移除（IMP-04 §13.3 A03）：
// 原来在模块作用域匿名注册且从不移除，反复进入/离开设置页会持续累积监听。
function onGuideInstallation() {
  routes.go('settings', { section: 'installation' })
}
onMounted(() => {
  window.addEventListener('opencodex:guide-installation', onGuideInstallation)
})

const sectionLabels: Record<string, string> = {
  agent: 'Agent 配置', general: '通用', installation: '安装配置', backup: '数据与备份', extensions: '扩展管理', migration: '文件同步', sync: 'WebDAV 同步', cleanup: '日志与通知', cli: 'CLI 控制面', upgrade: '版本升级', about: '关于',
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

async function copyPath(path: string) {
  try {
    await navigator.clipboard.writeText(displayPath(path))
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
function openInstallModal(source: InstallSourceKind) { app.requestInstallModal(source) }

const uninstallOpen = ref(false)
const uninstallScope = ref<'body' | 'full'>('full')
const uninstallAck = ref(false)
const uninstallAutoBackup = ref(true)
const uninstallCleanData = ref(true)
const uninstallRunning = ref(false)
const uninstallDone = ref(false)

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
    ? ['停止代理', '生成备份', '执行官方 ocx uninstall', '移除包体与入口', '清空 OPENCODEX_HOME 非官方自有残留', '残留核验']
    : ['停止代理', '生成备份', '移除包体与入口', '残留核验']
  return planned.map(name => ({ name, detail: '', cls: 'on' }))
})

const residueOk = computed(() => {
  const residue = app.runtimeUninstallResult?.residue ?? []
  return residue.length > 0 && residue.every(item => item.status === 'cleared')
})

// 与后端共用核验条件：失败、残留未知和证据缺失均不能显示成功。
const uninstallFailedSteps = computed(() =>
  (app.runtimeUninstallResult?.steps ?? []).filter(step => step.status === 'failed').length,
)
const uninstallVerified = computed(() => {
  const steps = app.runtimeUninstallResult?.steps ?? []
  return residueOk.value && steps.some(step => step.status === 'ok')
    && steps.every(step => step.status !== 'failed')
})
const uninstallPartialFailure = computed(() => uninstallDone.value && !uninstallVerified.value)

function residueLabel(status: string) {
  if (status === 'cleared') return '已清除'
  if (status === 'present') return '残留'
  return '未知（未确认清除）'
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

watch(() => routes.settingsSection, section => {
  if (section === 'installation') {
    if (!app.runtimeSource) void app.loadRuntimeSource()
    void runtimeProtection.loadProtection()
    void app.loadOfficialUninstallObservation()
  }
}, { immediate: true })
watch(() => runtimeProtection.source, () => {
  if (routes.settingsSection === 'installation') {
    void runtimeProtection.loadProtection()
    void dataRoots.loadPaths()
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
  <UpdateCenter ref="updateCenter" v-model:open="updateOpen" :target="updateTarget" />
  <section class="route-section active">
    <AppTopbar />
    <StickyTabs><div class="settings-tabs" role="tablist">
      <button v-for="item in settingsSections" :key="item" :class="{ active: section === item }" @click="routes.go('settings', { section: item })">{{ sectionLabels[item] }}</button>
    </div></StickyTabs>

    <section v-if="section === 'general'" class="settings-panel active">
      <UiCard>
        <UiCardHeader><h2>桌面壳偏好</h2><p>管理器自有行为、面板承载与通知。</p></UiCardHeader>
        <div class="setting-list">
          <SettingRow title="界面缩放" description="外壳与官方面板使用同一缩放因子；支持 50%–200% 自定义，默认 100%。">
            <template #actions>
              <div class="controls"><div class="range"><div class="range-slider"><input type="range" min="50" max="200" step="1" :value="preferences.interfaceScale" aria-label="界面缩放" @input="previewScale(Number(($event.target as HTMLInputElement).value))"><div class="range-ticks" role="group" aria-label="界面缩放刻度"><button v-for="tick in scaleTicks" :key="tick.value" type="button" class="range-tick" :class="{ active: preferences.interfaceScale === tick.value }" :style="{ '--pos': tick.pos }" :aria-pressed="preferences.interfaceScale === tick.value ? 'true' : 'false'" @click="setScale(tick.value)">{{ tick.value }}</button></div></div><input class="range-value" type="number" inputmode="numeric" min="50" max="200" step="1" :value="preferences.interfaceScale" aria-label="界面缩放数值" @change="commitScale(($event.target as HTMLInputElement).value, $event)"></div></div>
            </template>
          </SettingRow>
          <div class="setting-row"><div><div class="setting-title">启动时打开主界面</div><div class="setting-desc">默认打开；关闭后启动只保留托盘，不自动显示主窗口，也不自动启动代理。</div></div><div class="controls"><UiSwitch label="启动时打开主界面" :model-value="preferences.launchMain" @update:model-value="toggle('launchMain', '启动时打开主界面')" /></div></div>
          <div class="setting-row"><div><div class="setting-title">面板打开方式</div><div class="setting-desc">默认内嵌到 App 窗口；加载异常时提供浏览器兜底入口。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'panelMode' }"><button class="select-trigger" :aria-expanded="openSelect === 'panelMode' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="面板打开方式" @click="toggleSelect('panelMode')"><span class="select-value">{{ preferences.panelMode === 'embedded' ? '内嵌优先' : '浏览器兜底' }}</span></button><UiSelectMenu :open="openSelect === 'panelMode'" @close="openSelect = ''" role="listbox" aria-label="面板打开方式"><button class="select-option" :class="{ selected: preferences.panelMode === 'embedded' }" type="button" role="option" :aria-selected="preferences.panelMode === 'embedded' ? 'true' : 'false'" @click="chooseOption('panelMode', 'embedded', '内嵌优先')">内嵌优先</button><button class="select-option" :class="{ selected: preferences.panelMode === 'browser' }" type="button" role="option" :aria-selected="preferences.panelMode === 'browser' ? 'true' : 'false'" @click="chooseOption('panelMode', 'browser', '浏览器兜底')">浏览器兜底</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">关闭窗口后保持代理运行</div><div class="setting-desc">关闭窗口只隐藏桌面壳；从托盘退出会完全退出并可能中断由桌面壳托管的代理。</div></div><div class="controls"><UiSwitch label="关闭窗口后保持代理运行" :model-value="preferences.keepProxyOnClose" @update:model-value="toggle('keepProxyOnClose', '关闭窗口后保持代理运行')" /></div></div>
          <div class="setting-row"><div><div class="setting-title">启停结果通知</div><div class="setting-desc">通知启动、停止、重启和失败结果；不展示敏感配置。</div></div><div class="controls"><UiSwitch label="启停结果通知" :model-value="preferences.lifecycleNotifications" @update:model-value="toggle('lifecycleNotifications', '启停结果通知')" /></div></div>
          <div class="setting-row"><div><div class="setting-title">同步冲突提醒</div><div class="setting-desc">检测到本地与远端冲突时暂停覆盖，并要求显式确认。</div></div><div class="controls"><UiSwitch label="同步冲突提醒" :model-value="preferences.syncConflictAlerts" @update:model-value="toggle('syncConflictAlerts', '同步冲突提醒')" /></div></div>
        </div>
      </UiCard>
      <UiCard data-testid="card-quality">
        <UiCardHeader><h2>画质</h2><p>背景光与视觉效果的渲染方式；按设备能力自动降级。</p></UiCardHeader>
        <div class="setting-list">
          <div class="setting-row" data-testid="setting-visual-effects"><div><div class="setting-title">视觉特效</div><div class="setting-desc">高档为通知玻璃与动态光场；中档玻璃静态；低档统一实底、无光场。画质由所选档位决定，不跟随系统减少动态。即时生效并按偏好恢复。</div></div><div class="controls"><UiSegmented :model-value="preferences.visualEffects" label="视觉特效" :options="[{value:'high',label:'高'},{value:'mid',label:'中'},{value:'low',label:'低'}]" @update:model-value="chooseVisualEffects($event as 'high'|'mid'|'low', $event==='high'?'高':$event==='mid'?'中':'低')" /></div></div>
          <div class="setting-row" data-testid="setting-glow-render"><div><div class="setting-title">背景光渲染</div><div class="setting-desc">WEBGL 用网格渐变＋颗粒着色器，过渡最平滑、色带最少；CSS 为纯极光兜底，不依赖 GPU。<span v-if="glowRender.fellBack" class="setting-note" data-testid="glow-render-fallback">当前环境不支持 WEBGL，已自动回退纯 CSS 极光。</span></div></div><div class="controls"><UiSegmented :model-value="preferences.glowRender" label="背景光渲染" :options="[{value:'mesh',label:'WEBGL'},{value:'css',label:'CSS'}]" @update:model-value="chooseGlowRender($event as 'mesh'|'css', $event==='mesh'?'WEBGL':'CSS')" /></div></div>
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
          <div class="setting-row" data-testid="setting-network-probe"><div><div class="setting-title">检查连接</div><div class="setting-desc">{{ proxyProbeResult || '用当前配置对固定地址发一次带超时请求；只回报结果，不改写任何设置。' }}</div></div><div class="controls"><span v-if="proxyProbeResult" class="tag" :class="{ danger: !proxyProbeOk, ok: proxyProbeOk }">{{ proxyProbeOk ? '可用' : '不可用' }}</span><UiButton class="btn ghost" type="button" :disabled="proxyProbeBusy" @click="runProxyProbe()">{{ proxyProbeBusy ? '检查中' : '检查连接' }}</UiButton></div></div>
        </div>
      </UiCard>

<article class="card"><div class="card-head"><div><h2>官方共享配置</h2><p>与官方 OpenCodex 使用同一配置源；模型、路由与提供方仍由官方面板或官方 CLI 管理。</p></div></div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">随 Codex 启动 OpenCodex <span class="tag">官方共享</span></div><div class="setting-desc">与官方面板同一配置；开关的读写都经官方 shim <code>ocx codex-shim</code>，管理器不旁路改写官方配置。</div><div v-if="codexShimNotice" class="setting-note" data-testid="codex-shim-notice">{{ codexShimNotice }}</div></div><div class="controls"><span v-if="codexShimUnreachable" class="setting-readonly">官方 CLI 未就绪</span><UiSwitch label="检查连接" v-else   :model-value="codexShimInstalled" :disabled="codexShimLoading || codexShimBusy" aria-label="随 Codex 启动 OpenCodex" @update:model-value="toggleCodexShim()" /></div><div v-if="codexShimPhase !== 'idle'" class="shim-panel" :class="{ 'is-close': codexShimClosing, 'is-done': codexShimPhase === 'done', 'is-error': !codexShimResultOk }" data-testid="codex-shim-panel"><div class="shim-panel-head"><span class="shim-panel-spinner" aria-hidden="true"></span><b data-testid="codex-shim-panel-title">{{ codexShimPanelTitle }}</b><span class="shim-panel-meta">{{ codexShimElapsedText }}</span></div><div v-if="!codexShimClosing" class="shim-panel-bar" role="progressbar" aria-label="官方 shim 写入进度"><div class="shim-panel-fill"></div></div><div v-if="codexShimPanelHint" class="shim-panel-hint">{{ codexShimPanelHint }}</div><div v-if="codexShimResult" class="shim-panel-result" :class="{ error: !codexShimResultOk }" data-testid="codex-shim-result">{{ codexShimResult }}</div><button class="shim-details-toggle" type="button" :aria-expanded="codexShimShowDetails" @click="codexShimShowDetails = !codexShimShowDetails"><span class="chev" :class="{ open: codexShimShowDetails }" aria-hidden="true">▸</span> 查看执行的具体指令</button><div v-if="codexShimShowDetails" class="shim-details" data-testid="codex-shim-details"><ol class="shim-cmds"><li v-for="(c, i) in codexShimPlan.commands" :key="i" :data-cmd-state="codexShimPhase === 'done' && codexShimResultOk ? 'done' : 'todo'"><code>{{ c.cmd }}</code><span>{{ c.note }}</span></li></ol><p class="shim-details-note">写入严格经官方 CLI 转发，管理器不直接改写官方配置或启动器二进制。</p></div></div></div></div></article>
      <div class="section-actions"><UiButton class="btn ghost" @click="restoreGeneralDefaults">还原通用默认</UiButton></div>
    </section>

    <!-- 页面标题在卡片外，卡片内是自己的标题行（模型模版助手 · 本机共用 + 摘要 + 按钮），与原型层级一致。 -->
    <section v-else-if="section === 'agent'" class="settings-panel active"><h2 class="settings-panel-title">Agent 配置</h2><UiCard><AgentExecutionField /></UiCard></section>
    <section v-else-if="section === 'installation'" class="settings-panel active">
      <article class="card paths-v2" data-testid="settings-program-installation">
        <div class="card-head"><div><h2>程序安装与卸载</h2><p>程序位于 Desktop 安装目录。</p></div></div>
        <p v-if="dataRoots.pathsLoading" role="status">正在读取实际位置…{{ dataRoots.paths ? '下方保留上次快照（非最新）。' : '' }}</p>
        <p v-if="dataRoots.pathsError" role="alert">{{ dataRoots.pathsError }} {{ dataRoots.paths ? '已保留上次快照（非最新），请刷新后核对。' : '尚无可用快照；未推断路径或缺失状态。' }}</p>
        <section class="directory-group">
        <dl class="paths-facts">
          <template v-for="key in programPathKeys" :key="key"><dt>{{ pathLabels[key] }}</dt><dd :data-path-key="key" :data-path-status="dataRoots.paths?.[key].status"><template v-if="dataRoots.paths"><code class="local-path">{{ dataRoots.paths[key].path }}</code><span v-if="dataRoots.paths[key].status!=='valid'" class="tag" :data-path-status="dataRoots.paths[key].status">{{ pathStatusLabels[dataRoots.paths[key].status] }}</span></template><span v-else>尚未读取</span></dd></template>
        </dl><div class="directory-status"><dl class="paths-facts"><dt>运行状态</dt><dd>{{ app.runtimeState === 'running' ? '运行中' : app.runtimeState === 'stopped' ? '已停止' : app.runtimeState === 'starting_failed' ? '启动失败' : '待核对' }}</dd>
          <dt>包 / 清单 / 来源</dt><dd v-if="dataRoots.paths">{{ dataRoots.paths.program.packageVersion ?? '未记录' }} / {{ dataRoots.paths.program.manifestVersion ?? '未记录' }} / {{ dataRoots.paths.program.sourceVersion ?? '未记录' }}</dd><dd v-else>尚未读取</dd>
          <dt>程序有效性</dt><dd data-testid="program-validity">{{ programValidity }}</dd>
        </dl>
        </div><div class="runtime-actions"><UiButton class="btn primary" data-testid="runtime-install" :disabled="protectionBlocked || runtimeProtection.protectionLoading || !runtimeProtection.protection || restartPending" @click="openInstallModal('registry')">安装程序</UiButton><RuntimeActivationRecovery :disabled="protectionBlocked||restartPending||runtimeProtection.protectionLoading||app.runtimeInstalling||app.runtimeUninstallBusy" /><UiButton class="btn ghost" data-testid="paths-refresh" @click="refreshPaths">刷新位置</UiButton></div>
        <div v-if="protectionBlocked || runtimeProtection.protectionLoading || runtimeProtection.protectionFeedback" class="setting-row" data-testid="runtime-protection" :aria-busy="runtimeProtection.protectionLoading || runtimeProtection.protectionRetrying">
          <div>
            <div class="setting-title">面板更新保护核对</div>
            <p class="setting-desc" role="status" aria-live="polite">{{ protectionDescription }}</p>
            <p v-if="runtimeProtection.protection?.version" class="setting-desc">候选版本 {{ runtimeProtection.protection.version }} · 关联备份 {{ runtimeProtection.protection.backupId }}</p>
            <p v-if="runtimeProtection.protectionError" role="alert" class="setting-note">{{ runtimeProtection.protectionError }}</p>
          </div>
          <div class="controls">
            <UiButton v-if="protectionBlocked" type="button" class="btn ghost" data-testid="runtime-protection-retry" :disabled="runtimeProtection.protectionRetrying || runtimeProtection.protectionLoading || app.runtimeInstalling || app.runtimeUninstallBusy || app.dataRootPendingRestart || app.dataRootReconciliationRequired" @click="runtimeProtection.retryProtection()">{{ runtimeProtection.protectionRetrying ? '核对中…' : '重试本地核对' }}</UiButton>
            <UiButton v-if="protectionBlocked" type="button" class="btn ghost" @click="routes.go('logs', { tab: 'logs' })">查看诊断</UiButton>
          </div>
        </div>
        <div v-if="app.runtimeSource" class="runtime-actions" data-testid="runtime-source-card">
          <UiButton class="btn ghost" type="button" data-testid="runtime-import-offline" :disabled="protectionBlocked||runtimeProtection.protectionLoading||!runtimeProtection.protection||restartPending" @click="openInstallModal('offline')">导入离线包</UiButton>
          <UiButton v-if="app.runtimeSource.kind==='managed'" class="btn ghost" type="button" @click="routes.go('settings',{section:'upgrade'})">更新面板</UiButton>
          <UiButton v-if="app.runtimeSource.kind!=='unresolved'" class="btn danger" type="button" data-testid="runtime-uninstall" :disabled="protectionBlocked||runtimeProtection.protectionLoading||!runtimeProtection.protection||restartPending" @click="openUninstall">卸载</UiButton>
        </div>
        </section>
      </article>
      <RuntimePortSettings @saved="refreshPaths" :disabled="protectionBlocked||restartPending||runtimeProtection.protectionLoading||app.runtimeInstalling||app.runtimeUninstallBusy" />
      <article class="card paths-v2" data-testid="settings-paths-v2">
        <div class="card-head"><div><h2>用户数据与配置{{ restartPending ? ' · 待生效位置' : '' }}</h2><p>配置保存在当前用户目录。</p></div></div>
        <p v-if="dataRoots.pathsLoading" role="status">正在读取实际位置…{{ dataRoots.paths ? '下方保留上次快照（非最新）。' : '' }}</p>
        <p v-if="dataRoots.pathsError" role="alert">{{ dataRoots.pathsError }} {{ dataRoots.paths ? '已保留上次快照（非最新），请刷新后核对。' : '尚无可用快照；未推断路径或缺失状态。' }}</p>
        <section class="directory-group">
        <dl class="paths-facts">
          <template v-for="key in userPathKeys" :key="key"><dt>{{ pathLabels[key] }}</dt><dd :data-path-key="key" :data-path-status="dataRoots.paths?.[key].status"><template v-if="dataRoots.paths"><code class="local-path">{{ dataRoots.paths[key].path }}</code><span v-if="dataRoots.paths[key].status!=='valid'" class="tag" :data-path-status="dataRoots.paths[key].status">{{ pathStatusLabels[dataRoots.paths[key].status] }}</span></template><span v-else>尚未读取</span></dd></template>
        </dl>
        <div class="runtime-actions">
          <UiButton class="btn ghost" data-testid="paths-details" :disabled="!dataRoots.paths" @click="pathDialog = 'details'">完整查看与复制路径</UiButton>
          <UiButton class="btn ghost" :disabled="!dataRoots.paths" @click="pathDialog = 'json'">读取渠道配置说明</UiButton>
          <UiButton v-if="dataRoots.paths?.aggregateRoot.status === 'missing' || dataRoots.paths?.desktopRoot.status === 'missing'" class="btn" data-testid="paths-initialize" :disabled="dataRootBindingDisabled" @click="pathDialog = 'initialize'">初始化用户目录</UiButton>
        </div>
        <div v-if="restartPending" data-testid="paths-pending">
          <h3>等待重启生效</h3>
          <dl class="paths-facts"><dt>当前用户集合根</dt><dd>{{ dataRoots.paths?.currentAggregateRoot ?? '待读取' }}</dd><dt>待生效用户集合根</dt><dd>{{ dataRoots.config?.aggregateRoot ?? dataRoots.paths?.aggregateRoot.path ?? '待读取' }}</dd><dt>当前 Desktop 工作根</dt><dd>{{ dataRoots.config?.currentDataRoot ?? '待读取' }}</dd><dt>待生效 Desktop 工作根</dt><dd>{{ dataRoots.config?.activeDataRoot ?? '待读取' }}</dd><dt>当前 OpenCodex HOME</dt><dd>{{ dataRoots.config?.currentOpencodexHome ?? '待读取' }}</dd><dt>待生效 OpenCodex HOME</dt><dd>{{ dataRoots.config?.opencodexHome ?? '待读取' }}</dd></dl>
          <p v-if="dataRoots.reconciliationRequired" role="alert">绑定可能已保存，持久性核对尚未通过；待生效位置仅为预期目标。写入已暂停，请重启管理器核对。</p>
          <p v-else role="status">已保存新位置，写入已暂停；当前服务仍使用当前路径，请重启管理器后回读确认。</p>
        </div>
        <p v-if="dataRoots.configError" role="alert">用户目录绑定配置读取失败；路径修改已暂停，请刷新核对。</p>
        <p v-if="dataRoots.error" role="alert">{{ dataRoots.error }}</p>
        </section>
      </article>

    </section>

    <section v-else-if="section === 'backup'" class="settings-panel active">
      <NativeConfigRollback />
      <article class="card"><div class="card-head"><div><h2>数据与备份</h2><p>备份边界基于安装配置里的数据目录。</p></div><UiButton class="btn ghost" @click="routes.go('settings', { section: 'installation' })">打开安装配置</UiButton></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">升级前备份</div><div class="setting-desc">面板更新前必须备份，失败时阻断安装；管理器备份选项在更新详情中确认。</div></div><div class="controls"><span class="tag">面板必需</span></div></div>
          <div class="setting-row"><div><div class="setting-title">导入前自动备份</div><div class="setting-desc">在导入窗口确认后，先备份所选范围再更新；每次默认开启，可在本次窗口关闭。</div></div><div class="controls"><span class="setting-readonly">导入时选择</span></div></div>
          <div class="setting-row"><div><div class="setting-title">同步覆盖前自动备份</div><div class="setting-desc">在同步预览确认后，按本次“更新前备份”选择保护所选资源；取消预览不生成备份。</div></div><div class="controls"><span class="setting-readonly">确认时选择</span></div></div>
          <div class="setting-row"><div><div class="setting-title">备份完整性校验</div><div class="setting-desc">生成备份后写入 SHA-256 校验摘要，恢复前先校验；算法在备份契约中固定。</div></div><div class="controls"><span class="setting-readonly">SHA-256</span></div></div>
        </div>
        <BackupManagement />
      </article>
    </section>

    <section v-else-if="section === 'extensions'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>Skills 管理</h2><p>控制源目录、同步与资产参与范围。</p></div><UiButton class="btn ghost" @click="routes.go('extensions', { tab: 'skills' })">打开 Skills 列表</UiButton></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">Skills 源目录</div><div class="setting-desc">默认目录；也可以自定义目录，管理器只读该目录。导入与恢复的 Skill 由管理器自行保存。</div></div><div class="controls"><UiButton class="btn ghost" @click="openPath(skillsSourcePath, 'Skills 源目录')">打开</UiButton><UiButton class="btn" @click="chooseSkillsSourceDir()">选择自定义目录</UiButton><UiButton v-if="skillsSourceIsCustom" class="btn ghost" @click="restoreDefaultSkillsSourceDir()">恢复默认</UiButton></div><button class="cli-copy-btn skills-source" title="复制 Skills 源目录" @click="copyPath(skillsSourcePath)"><code class="local-path">{{ displayPath(skillsSourcePath) }}</code></button></div>
          <div class="setting-row"><div><div class="setting-title">同步方式</div><div class="setting-desc">软链接优先：省空间、与源实时联动；创建失败或安全校验不通过时自动回退为复制。文件复制更稳定，适合迁移和备份快照。改动源目录或同步方式后，已落地的目标不会自动重做——用「检查并修正」对齐。</div></div><div class="controls"><UiButton class="btn ghost" @click="resyncSkills()" title="按当前源目录与同步方式，重新对齐已落地的客户端目标">检查并修正</UiButton><div class="select" :class="{ open: openSelect === 'skillsSyncMode' }"><button class="select-trigger" :aria-expanded="openSelect === 'skillsSyncMode' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="Skills 同步方式" @click="toggleSelect('skillsSyncMode')"><span class="select-value">{{ skillsSyncMethod === 'copy' ? '文件复制' : '软链接优先' }}</span></button><UiSelectMenu :open="openSelect === 'skillsSyncMode'" @close="openSelect = ''" role="listbox" aria-label="Skills 同步方式"><button class="select-option" :class="{ selected: skillsSyncMethod === 'symlink' }" type="button" role="option" :aria-selected="skillsSyncMethod === 'symlink' ? 'true' : 'false'" @click="chooseSkillsSyncMethod('symlink', '软链接优先')">软链接优先</button><button class="select-option" :class="{ selected: skillsSyncMethod === 'copy' }" type="button" role="option" :aria-selected="skillsSyncMethod === 'copy' ? 'true' : 'false'" @click="chooseSkillsSyncMethod('copy', '文件复制')">文件复制</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">备份包含 Skills 目录</div><div class="setting-desc">选择 Skills 并开启更新前备份，即保护所选 Skill 的实际内容；偏好备份不包含这些资源。</div></div><div class="controls"><span class="setting-readonly">按所选范围</span></div></div>
          <div class="setting-row"><div><div class="setting-title">导出包含 Skills 目录</div><div class="setting-desc">在同步页选择要导出的 Skills；实际正文与资源随包导出，可能的凭据另行选择。</div></div><div class="controls"><span class="setting-readonly">同步页选择</span></div></div>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>MCP 管理</h2><p>统一管理 MCP 写入策略与目标。</p></div><UiButton class="btn ghost" @click="routes.go('extensions', { tab: 'mcp' })">打开 MCP 列表</UiButton></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">同名冲突策略</div><div class="setting-desc">目标客户端已有同名 MCP 时一律暂停写入、展示差异并要求确认；不提供静默保留策略。</div></div><div class="controls"><span class="setting-readonly">每次询问</span></div></div>
          <div class="setting-row"><div><div class="setting-title">MCP 配置落点</div><div class="setting-desc">按客户端适配器写入各自 MCP 配置节点。</div></div><div class="controls"><UiButton class="btn ghost" @click="routes.go('extensions', { tab: 'mcp' })">查看落点</UiButton></div></div>
          <div class="setting-row"><div><div class="setting-title">敏感值脱敏</div><div class="setting-desc">URL 令牌、请求头、环境变量与 API Key 默认掩码展示。</div></div><div class="controls"><UiSwitch label="敏感值脱敏" :model-value="preferences.mcpMask" @update:model-value="toggle('mcpMask', 'MCP 敏感值脱敏')" /></div></div>
          <div class="setting-row"><div><div class="setting-title">备份包含 MCP 配置</div><div class="setting-desc">选择 MCP 并开启更新前备份，即保护本次定义；未选对象与当前客户端分发保持。</div></div><div class="controls"><span class="setting-readonly">按所选范围</span></div></div>
          <div class="setting-row"><div><div class="setting-title">导出包含 MCP 配置</div><div class="setting-desc">在同步页选择要导出的 MCP；同名定义可选择来源，凭据另行选择。</div></div><div class="controls"><span class="setting-readonly">同步页选择</span></div></div>
        </div>
      </article>
      <article class="card"><div class="card-head"><div><h2>Skills 路径</h2><p>集中查看并打开各客户端 Skills 目录。</p></div></div><div class="agent-path-list"><div v-for="row in skillsPaths" :key="row[1]" class="agent-path-row"><div class="agent-path-app"><span>{{ row[0] }}</span></div><button class="cli-copy-btn agent-path-copy" @click="copyPath(row[1])"><code class="local-path">{{ displayPath(row[1]) }}</code></button><div class="agent-path-actions"><UiButton size="sm" class="btn ghost" @click="openPath(row[1], `${row[0]} Skills`)">打开目录</UiButton></div></div></div></article>
      <article class="card"><div class="card-head"><div><h2>MCP 路径</h2><p>集中查看并打开各客户端 MCP 配置。</p></div></div><div class="agent-path-list"><div v-for="row in mcpPaths" :key="row[1]" class="agent-path-row"><div class="agent-path-app"><span>{{ row[0] }}</span></div><button class="cli-copy-btn agent-path-copy" @click="copyPath(row[1])"><code class="local-path">{{ displayPath(row[1]) }}</code></button><div class="agent-path-actions"><UiButton size="sm" class="btn ghost" @click="openPath(row[1], `${row[0]} MCP 配置`)">打开配置</UiButton></div></div></div></article>
    </section>

    <section v-else-if="section === 'cleanup'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>日志与通知清理</h2><p>自动清理只作用于管理器本地记录；不会清理官方 OpenCodex 服务日志。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">日志自动清理</div><div class="setting-desc">本地日志超过上限或保留期后自动截断。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'logRetention' }"><button class="select-trigger" :aria-expanded="openSelect === 'logRetention' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="日志自动清理" @click="toggleSelect('logRetention')"><span class="select-value">{{ preferences.logRetention === '7d-5000' ? '7 天 / 5,000 行' : preferences.logRetention === '90d-30000' ? '90 天 / 30,000 行' : '30 天 / 10,000 行' }}</span></button><UiSelectMenu :open="openSelect === 'logRetention'" @close="openSelect = ''" role="listbox" aria-label="日志自动清理"><button v-for="option in [['7d-5000','7 天 / 5,000 行'],['30d-10000','30 天 / 10,000 行'],['90d-30000','90 天 / 30,000 行']]" :key="option[0]" class="select-option" :class="{ selected: preferences.logRetention === option[0] }" type="button" role="option" :aria-selected="preferences.logRetention === option[0] ? 'true' : 'false'" @click="chooseOption('logRetention', option[0] as PreferencesDto['logRetention'], option[1])">{{ option[1] }}</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">通知自动清理</div><div class="setting-desc">已读通知超过保留期后自动清理。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'notificationRetention' }"><button class="select-trigger" :aria-expanded="openSelect === 'notificationRetention' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="通知自动清理" @click="toggleSelect('notificationRetention')"><span class="select-value">{{ preferences.notificationRetention === '7' ? '7 天' : preferences.notificationRetention === '90' ? '90 天' : '30 天' }}</span></button><UiSelectMenu :open="openSelect === 'notificationRetention'" @close="openSelect = ''" role="listbox" aria-label="通知自动清理"><button v-for="option in [['7','7 天'],['30','30 天'],['90','90 天']]" :key="option[0]" class="select-option" :class="{ selected: preferences.notificationRetention === option[0] }" type="button" role="option" :aria-selected="preferences.notificationRetention === option[0] ? 'true' : 'false'" @click="chooseOption('notificationRetention', option[0] as PreferencesDto['notificationRetention'], option[1])">{{ option[1] }}</button></UiSelectMenu></div></div></div>
          <div class="setting-row"><div><div class="setting-title">启动时清理</div><div class="setting-desc">应用启动时按策略执行一次轻量清理，不阻塞主窗口。</div></div><div class="controls"><UiSwitch label="启动时清理" :model-value="preferences.startupCleanup" @update:model-value="toggle('startupCleanup', '启动时清理')" /></div></div>
          <div class="setting-row"><div><div class="setting-title">清理前备份摘要</div><div class="setting-desc">danger 日志和重要通知会先生成脱敏摘要。</div></div><div class="controls"><UiSwitch label="清理前备份摘要" :model-value="preferences.cleanupBackupSummary" @update:model-value="toggle('cleanupBackupSummary', '清理前备份摘要')" /></div></div>
        </div>
        <div class="controls"><UiButton class="btn ghost" @click="routes.go('logs')">打开诊断中心</UiButton><UiButton class="btn" @click="cleanupLogs()">立即清理日志</UiButton><UiButton class="btn" :disabled="!app.notifications.length" @click="cleanupNotifications()">立即清理通知</UiButton></div>
      </article>
    </section>

    <section v-else-if="section === 'cli'" class="settings-panel active">
      <article class="card"><div class="card-head"><div><h2>CLI 控制面</h2><p>让终端、脚本或 AI 代理控制本应用的自有域；不提供改写 OpenCodex 配置的通道。</p></div></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">启用 CLI 控制面</div><div class="setting-desc">{{ cliSupported ? '默认关闭；开启后注册本机命令入口。CLI 仅作为运行中实例的客户端，经本地 IPC 委托执行。' : 'Windows 暂不开放公开 CLI 控制面；本机 IPC 传输已实现，但公开命令权限与验收尚未完成。' }}</div></div><div class="controls"><UiSwitch label="启用 CLI 控制面" :disabled="!cliSupported" :model-value="cliSupported && preferences.cliEnabled" @update:model-value="toggleCli()" /></div></div>
          <template v-if="cliSupported && preferences.cliEnabled">
            <div class="setting-row"><div><div class="setting-title">本机 IPC 端点</div><div class="setting-desc"><code>~/Library/Caches/OpenCodex Desktop/ipc/opencodex.ipc</code>；目录 0700，socket 0600，仅限当前用户 UID。</div></div><div class="controls"><span class="cli-badge ok">在线</span></div></div>
            <div class="setting-row"><div><div class="setting-title">PATH 注册</div><div class="setting-desc">默认关闭；注册为指向应用内 <code>ocxd</code> 的 symlink，不会覆盖他人目标。</div></div><div class="controls"><span class="cli-badge">未注册</span></div></div>
            <div class="setting-row"><div><div class="setting-title">调用日志</div><div class="setting-desc">记录命令、参数键、请求 ID 与结果；保留 90 天 / 5,000 条，单文件 5 MB × 5 份。在「诊断中心 → 日志历史 → 调用日志」中查看。</div></div><div class="controls"><span class="cli-badge">已启用</span></div></div>
            <div class="setting-row"><div><div class="setting-title">Agent 提示</div><div class="setting-desc">管理器用 <code>ocxd</code>；OpenCodex 自身配置仍用官方 <code>ocx</code>。命令能力先 <code>ocxd --help</code>。</div></div><div class="controls"><UiButton class="btn ghost" @click="copyAgentPrompt()">复制 Agent 提示</UiButton></div></div>
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
ocxd data-root switch --target &lt;path&gt; --confirm --json
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
      <article class="card">
        <div class="card-head"><h2>软件更新</h2><UiButton class="btn primary" type="button" @click="openUpdates(null, true)">检查更新</UiButton></div>
        <div class="setting-list">
          <div class="setting-row"><div><div class="setting-title">桌面管理器</div><div class="setting-desc">查看通道版本、更新说明与实时进度；准备就绪后选择重启时机。</div></div><div class="controls"><UiButton class="btn ghost" type="button" @click="openUpdates('manager')">查看管理器更新</UiButton></div></div>
          <div class="setting-row"><div><div class="setting-title">OpenCodex 面板</div><div class="setting-desc">当前 {{ officialVersion }}；升级前必须备份配置。</div></div><div class="controls"><UiButton class="btn ghost" type="button" @click="openUpdates('official')">查看面板更新</UiButton></div></div>
<div class="setting-row"><div><div class="setting-title">更新通道</div><div class="setting-desc">稳定通道或测试通道；检查、安装与自动调度共用同一通道来源。</div></div><div class="controls"><div class="select" :class="{ open: openSelect === 'appUpdateChannel' }"><button class="select-trigger" :aria-expanded="openSelect === 'appUpdateChannel' ? 'true' : 'false'" aria-haspopup="listbox" aria-label="应用更新通道" @click="toggleSelect('appUpdateChannel')"><span class="select-value">{{ preferences.appUpdateChannel === 'beta' ? '测试通道' : '稳定通道' }}</span></button><UiSelectMenu :open="openSelect === 'appUpdateChannel'" @close="openSelect = ''" role="listbox" aria-label="应用更新通道"><button class="select-option" :class="{ selected: preferences.appUpdateChannel === 'stable' }" type="button" role="option" :aria-selected="preferences.appUpdateChannel === 'stable' ? 'true' : 'false'" @click="chooseUpdateChannel('stable', '稳定通道')">稳定通道</button><button class="select-option" :class="{ selected: preferences.appUpdateChannel === 'beta' }" type="button" role="option" :aria-selected="preferences.appUpdateChannel === 'beta' ? 'true' : 'false'" @click="chooseUpdateChannel('beta', '测试通道')">测试通道</button></UiSelectMenu></div></div></div><div class="setting-row" data-testid="setting-network-shortcut"><div><div class="setting-title">网络连接</div><div class="setting-desc">检查更新失败多为网络问题；可在此跳转配置代理。</div></div><div class="controls"><UiButton class="btn ghost" @click="goToNetwork">配置网络</UiButton></div></div>
<div class="setting-row"><div><div class="setting-title">自动检查更新</div><div class="setting-desc">开启后按通道后台检查；关闭后不影响手动检查。</div></div><div class="controls"><UiSwitch label="自动检查更新" :model-value="preferences.appUpdateAutoCheck" @update:model-value="toggleUpdateAutoCheck()" /></div></div>
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

  <UiDialog :open="uninstallOpen" title="卸载 OpenCodex" :busy="uninstallRunning || app.runtimeUninstallBusy" panel-class="wiz wiz-uninstall" data-testid="runtime-uninstall-modal" @close="closeUninstall">

      <template v-if="!uninstallRunning && !uninstallDone">
        <p class="wiz-hint">按当前运行来源 <code class="local-path">{{ displayPath(app.runtimeUninstallPlan?.sourcePath) || '未解析' }}</code> 卸载；范围越大后果越重。</p>
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
              <li v-for="item in app.runtimeUninstallPlan.removeObjects" :key="item"><code class="local-path">{{ displayPath(item) }}</code></li>
              <li v-for="item in (uninstallScope === 'full' ? app.runtimeUninstallPlan.runtimeObjects : [])" :key="item"><span class="local-path">{{ displayPath(item) }}</span></li>
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


      </template>

      <template v-else>
        <div class="wiz-uview">
          <div class="wiz-uhead">
            <span
              class="wiz-spin"
              :class="{ done: uninstallDone, faulted: uninstallPartialFailure }"
              aria-hidden="true"
            >{{ uninstallDone ? (uninstallPartialFailure ? '!' : '✓') : '' }}</span>
            <b data-testid="uninstall-progress-title">{{ uninstallDone ? (uninstallPartialFailure ? (uninstallFailedSteps > 0 ? `卸载未核验完成：${uninstallFailedSteps} 步失败` : '卸载待核验') : '卸载完成') : '正在卸载 OpenCodex' }}</b>
            <span class="wiz-umeta">{{ uninstallScope === 'full' ? '完整卸载' : '仅移除包体' }}<template v-if="uninstallDone">{{ uninstallPartialFailure ? (uninstallFailedSteps > 0 ? ` · ${uninstallFailedSteps} 步失败` : ' · 未核验完成') : ' · 100%' }}</template></span>
          </div>
          <div class="wiz-ubar" role="progressbar" aria-label="卸载进度"><div class="wiz-ufill" :class="{ done: uninstallDone && !uninstallPartialFailure, faulted: uninstallPartialFailure, run: !uninstallDone }"></div></div>
          <ul class="wiz-substeps" data-testid="uninstall-steps">
            <li v-for="row in uninstallStepRows" :key="row.name" :class="row.cls"><span><b>{{ row.name }}</b>{{ row.detail ? ' · ' + row.detail : '' }}</span></li>
          </ul>
          <pre v-if="app.runtimeUninstallResult && app.runtimeUninstallResult.officialOutput.length" class="wiz-console wiz-console-fixed" data-testid="uninstall-console">{{ app.runtimeUninstallResult.officialOutput.join('\n') }}</pre>
        </div>
        <div v-if="uninstallDone" class="residue-card" data-testid="uninstall-residue">
          <div class="residue-head"><span class="residue-badge" aria-hidden="true">{{ residueOk ? '✓' : '!' }}</span><b>残留核验</b><small>{{ residueOk ? '0 残留' : ((app.runtimeUninstallResult?.residue ?? []).length ? '有未清除项' : '缺少核验证据') }}</small></div>
          <ul class="residue-list">
            <li v-for="item in (app.runtimeUninstallResult?.residue ?? [])" :key="item.path" :class="item.status"><code class="local-path">{{ displayPath(item.path) }}</code><span>{{ residueLabel(item.status) }}</span></li>
          </ul>
          <p v-if="app.runtimeUninstallResult?.backupId" class="wiz-hint">备份 <code>{{ app.runtimeUninstallResult.backupId }}</code> · <button class="cli-copy-btn" type="button" @click="copyPath(app.runtimeUninstallResult?.backupDirectory ?? '')"><code class="local-path">{{ displayPath(app.runtimeUninstallResult?.backupDirectory) }}</code></button></p>
        </div>
      </template>

      <p v-if="app.runtimeUninstallError" class="wiz-error" data-testid="uninstall-error">{{ app.runtimeUninstallError }}</p>

      <template v-if="!uninstallRunning && !uninstallDone" #actions="{ close }">
        <div class="uninstall-confirmation">
        <label class="wiz-ack" :class="{ on: uninstallAck }">
          <input v-model="uninstallAck" type="checkbox" data-testid="uninstall-ack">
          <span><b>我已阅读清单并确认继续</b><small>停代理 → 备份{{ uninstallScope === 'full' ? ' → 执行官方 ocx uninstall' : '' }} → 移除包体与入口 → 残留核验（全部由应用代执行）。</small></span>
        </label>
        <p v-if="!uninstallAck" class="wiz-ack-hint" data-testid="uninstall-ack-hint">勾选上面这一项，「卸载」才会亮起。</p>
          <div class="uninstall-actions">
            <UiButton variant="ghost" type="button" :disabled="app.runtimeUninstallBusy" @click="close">取消</UiButton>
            <UiButton variant="danger" type="button" data-testid="uninstall-confirm" :disabled="!uninstallCanConfirm" @click="confirmUninstall">卸载</UiButton>
          </div>
        </div>
      </template>
  </UiDialog>
  <UiDialog v-if="pathDialog" :open="true" :title="pathDialog === 'details' ? '完整路径' : pathDialog === 'initialize' ? '初始化用户目录' : '读取渠道配置说明'" :busy="dataRoots.switching || dataRoots.saving || dataRoots.homeSaving" panel-class="paths-v2-dialog" data-testid="paths-dialog" @close="closePathDialog">
    <div class="paths-v2">
      <p v-if="dataRoots.pathsStale" role="status">上次快照（非最新），请刷新后核对。</p>
      <template v-if="pathDialog === 'details' && dataRoots.paths">
        <dl class="paths-facts"><template v-for="key in [...userPathKeys, ...programPathKeys]" :key="key"><dt>{{ pathLabels[key] }}</dt><dd><code class="local-path">{{ dataRoots.paths[key].path }}</code><span class="tag">{{ pathStatusLabels[dataRoots.paths[key].status] }}</span></dd></template></dl>
        <UiButton class="btn ghost" data-testid="paths-copy" @click="copyActualPaths">复制完整路径</UiButton>
      </template>
      <template v-else-if="pathDialog === 'initialize'"><p>将仅初始化下方明确的新版用户集合根；不读取或搬移旧目录。初始化不会搬移其它目录内容。</p><code class="local-path">{{ dataRoots.paths?.aggregateRoot.path }}</code><p v-if="dataRoots.error" role="alert">{{ dataRoots.error }}</p></template>
      <template v-else-if="pathDialog === 'json'"><dl class="paths-facts"><dt>放入位置</dt><dd>{{ dataRoots.paths?.configPath.path }}</dd></dl><p>将当前支持格式的 config.json 放入此位置，再前往模型页“读取现有渠道”。格式错误、认证引用缺失或空结果均保留原草稿。</p></template>
    </div>
    <template v-if="pathDialog !== 'details' && pathDialog !== 'json'" #actions>
      <UiButton class="btn ghost" :disabled="dataRoots.switching || dataRoots.saving || dataRoots.homeSaving" @click="closePathDialog">取消</UiButton>
      <UiButton v-if="pathDialog === 'initialize'" class="btn primary" :disabled="dataRootBindingDisabled" @click="saveDataRoot">确认初始化</UiButton>
    </template>
  </UiDialog>

</template>

<style scoped>
.paths-v2 { min-width:0; font-size:13px; }
.paths-v2 h3 { margin:20px 0 12px; font-size:13px; }
.paths-v2 p, .paths-v2 code, .paths-v2 input, .paths-v2 button { font-size:13px; }
.paths-facts { display:grid; grid-template-columns:minmax(100px, 140px) minmax(0, 1fr); gap:10px 16px; margin:12px 0 16px; }
.paths-facts dt { color:var(--muted); }
.paths-facts dd { margin:0; min-width:0; overflow-wrap:anywhere; white-space:pre-wrap; }
.paths-v2 .local-path { overflow-wrap:anywhere; white-space:pre-wrap; word-break:break-word; }
.paths-facts .tag { margin-left:8px; white-space:normal; }
.paths-v2 {container-type:inline-size;min-width:0}
.paths-v2>.card-head{margin-bottom:22px}
.paths-v2>.card-head p{margin-top:6px;line-height:1.6}
.paths-v2 .directory-group+.directory-group{margin-top:24px;padding-top:24px;border-top:1px solid var(--border)}
.paths-v2 .directory-group h3{margin:0 0 16px;font-size:var(--text-body,13px);font-weight:600;line-height:1.6}
.paths-v2 .paths-facts{margin:0;grid-template-columns:144px minmax(0,1fr);gap:14px 20px;align-items:start}
.paths-v2 .paths-facts dt,.paths-v2 .paths-facts dd{line-height:1.65;margin:0;min-width:0}
.paths-v2 .paths-facts dt{white-space:nowrap}
.paths-v2 .paths-facts code{font:inherit;white-space:pre-wrap;overflow-wrap:anywhere}
.paths-v2 .runtime-actions {display:flex;gap:10px;margin-top:18px;flex-wrap:wrap}
.paths-v2 .directory-status{margin-top:22px;padding-top:18px;border-top:1px solid var(--border)}
@container(max-width:520px){.paths-v2 .paths-facts{grid-template-columns:128px minmax(0,1fr);column-gap:14px}}
@container(max-width:380px){.paths-v2 .paths-facts{grid-template-columns:minmax(0,1fr);row-gap:6px}.paths-v2 .paths-facts dd:not(:last-child){margin-bottom:10px}}
@media (max-width:600px) { .paths-facts { grid-template-columns:minmax(0, 1fr); gap:6px; } .paths-facts dd { margin-bottom:8px; } }

.uninstall-confirmation { display:grid; gap:10px; width:100%; }
.uninstall-confirmation .wiz-ack { margin:0; }
.uninstall-confirmation .wiz-ack span { display:grid; gap:4px; }
.uninstall-confirmation .wiz-ack input { flex:none; }
.uninstall-confirmation .wiz-ack-hint { margin:0; }
.uninstall-actions { display:flex; justify-content:flex-end; gap:8px; }

/* 统一要求：设置页表格行内的路径、用途与操作按钮垂直居中，不因长路径互相挤压。 */
.data-root-table th,
.data-root-table td { vertical-align: middle; }
/* 列表行（shim 指令 / 卸载残留）同样垂直居中，命令与说明不上下错位。 */
.shim-cmds li,
.residue-list li { align-items: center; }
</style>
