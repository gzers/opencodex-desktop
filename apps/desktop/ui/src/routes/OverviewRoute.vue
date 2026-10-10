<script setup lang="ts">
// 概览：运行 / 环境准备双形态（UI规范 §25）。
//
// - 环境就绪（ready / ready_via_source）：无框居中状态区；Logo → 主线状态 → 必要说明 → 当前操作；
//   环境明细并入运行详情；下方版本升级 / 配置迁移 / WebDAV 同步三卡，最后为最近事件摘要。
// - 环境准备（检查中或真实前置缺失）：三卡与最近事件退出布局，顶部缩短，下方一张准备卡
//   （左纵向检查 / 右当前指引 + 命令 + 动作）。
// - 状态形象只消费既有运行投影（`projectMotion`），不引入新的状态来源；环境走既有 presentation。
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouteStore } from '@/stores/routes'
import { useAppStore } from '@/stores/app'
import { useAppController } from '@/composables/useAppController'
import AppTopbar from '@/components/AppTopbar.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UpdateCenter from '@/features/updates/UpdateCenter.vue'
import { managerUpdateStatus } from '@/features/updates/update'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import EnvironmentGate from '@/features/environment/components/EnvironmentGate.vue'
import { webdavStates } from '@/features/sync/states'
import { runtimeSourceLabel } from '@/lib/labels'
import { buildEnvironmentPresentation, environmentChecksOrdered } from '@/features/environment/presentation'
import { projectMotion } from '@/features/runtime/motion/projection'

const routes = useRouteStore()
const app = useAppStore()
const controller = useAppController()

const updateCenter = ref<InstanceType<typeof UpdateCenter> | null>(null)
const updateOpen = ref(false)
const updateTarget = ref<'manager' | 'official' | null>(null)
function openUpdates(target: 'manager' | 'official' | null = null, check = false) {
  updateTarget.value = target
  updateOpen.value = true
  if (check) void updateCenter.value?.check()
}
const detailOpen = ref(false)
const envOpen = ref(false)
const copiedCommand = ref<string | null>(null)
const documentVisible = ref(true)
// 按**应用窗口可用逻辑高度**切换紧凑版（参考阈值 680px，考虑界面缩放），不按外部浏览器尺寸猜测。
const compact = ref(false)

function onVisibilityChange() {
  documentVisible.value = document.visibilityState !== 'hidden'
}
function onKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return
  if (detailOpen.value) {
    event.preventDefault()
    detailOpen.value = false
  } else if (envOpen.value) {
    event.preventDefault()
    envOpen.value = false
  }
}

let sizeObserver: ResizeObserver | null = null
function updateSize() {
  const win = document.querySelector('.app-window')
  const measured = win?.getBoundingClientRect().height ?? window.innerHeight
  const zoom = Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom')) || 1
  const logical = measured / zoom
  const next = logical < 680
  if (next !== compact.value) compact.value = next
  document.documentElement.setAttribute('data-overview-size', next ? 'compact' : 'standard')
}

onMounted(() => {
  document.addEventListener('visibilitychange', onVisibilityChange)
  window.addEventListener('keydown', onKeydown)
  updateSize()
  if (typeof ResizeObserver !== 'undefined') {
    const win = document.querySelector('.app-window')
    if (win) {
      sizeObserver = new ResizeObserver(updateSize)
      sizeObserver.observe(win)
    }
  }
})
onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', onVisibilityChange)
  window.removeEventListener('keydown', onKeydown)
  sizeObserver?.disconnect()
  sizeObserver = null
  document.documentElement.removeAttribute('data-overview-size')
})

const scenario = controller.scenario

// 状态形象：主线 / 新鲜度 / 持续问题 → 一种内部结构。只读投影，不新建状态机。
const ongoingProblem = computed(() =>
  ['external_takeover', 'unreachable'].includes(app.runtimeState),
)
// 事实新鲜度接线：**最近一次采集失败**（`statusError`）且手上是**此前真实观测到的**快照
// （`source === 'live'`）时，保留旧事实但标记为「过期」；fixture / 未配置来源不冒充过期，
// 进行中的观测/操作也不受旧事实顶替（见 projection 的 IN_FLIGHT）。不伪造新鲜，也不伪造过期。
const fresh = computed(() => !(app.statusError && app.statusSnapshot?.source === 'live'))
const motion = computed(() =>
  projectMotion({ runtimeState: app.runtimeState, fresh: fresh.value, problem: ongoingProblem.value }),
)
// 只在概览可见且窗口在前台时逐帧更新。
const motionActive = computed(() => routes.current === 'overview' && documentVisible.value)

function recheckEnvironment() {
  void controller.refreshEnvironment()
}

const knownInstall = computed(() => !['loading', 'not_found'].includes(app.runtimeState))
const version = computed(() =>
  knownInstall.value
    ? (app.statusSnapshot?.facts.version_drift ?? app.officialProject?.version ?? '—')
    : '—',
)
const runtime = computed(() =>
  knownInstall.value && app.statusSnapshot?.facts.runtime_label
    ? `Codex 运行时（${runtimeSourceLabel(app.statusSnapshot.facts.runtime_label)}）`
    : '—',
)
const managerDataRoot = computed(() => app.dataRootConfig?.currentDataRoot || app.dataRootPath || '')
const managerHome = computed(() => app.dataRootConfig?.currentOpencodexHome || '')
const dataRoot = computed(() => managerDataRoot.value || app.statusSnapshot?.facts.data_root || '—')
const home = computed(() => managerHome.value || app.statusSnapshot?.facts.opencodex_home || '—')

// 环境结论（就绪态并入运行详情；准备态由准备卡承载）。
const environment = computed(() =>
  buildEnvironmentPresentation(app.environment, app.environmentLoading, app.runtimeSource?.kind ?? null, app.aboutApp?.platform),
)
// 按检查顺序呈现：检查中只亮当前项，其后为「待检查」。
const envChecks = computed(() =>
  environmentChecksOrdered(environment.value.checks, environment.value.state === 'checking'),
)

// 双形态：环境未就绪（检查中或真实前置缺失）走「环境准备」，其余走「环境就绪」（UI规范 §25.1）。
// `blocked` 只覆盖真实前置缺失；检查中由 `state === 'checking'` 一并纳入准备态。
const gateBlocked = computed(() => environment.value.state === 'checking' || environment.value.blocked)
const mode = computed<'ready' | 'setup'>(() => (gateBlocked.value ? 'setup' : 'ready'))
// 准备态的主线说明与入口按 §25 调整：不充当无效详情入口。
const caption = computed(() => {
  if (!gateBlocked.value) return motion.value.caption
  return environment.value.state === 'checking' ? '正在确认启动条件' : '启动条件未满足 · 请按下方指引处理'
})
// 副标题按双形态改写（§25.1 / 原型 overview-dual.js）。
const pageSubtitle = computed(() =>
  gateBlocked.value ? '完成启动前的环境准备，再继续运行操作。' : '运行状态与常用操作；环境明细收进运行详情。',
)

// 最近事件：标准显示最近两条，紧凑一条；完整历史保留在诊断。
const recentEvents = computed(() => app.recentEvents.slice(0, compact.value ? 1 : 2))

// 动作：主操作直接露出，低频动作收进「更多」（UI规范 §19 / §20）。
// 动作层级对齐已确认原型 `actionsByState`（运行中＝打开面板/停止/重启/查看日志，全部内联）。
// 既有「刷新状态」不在原型主线动作内，但仍保留出口：收进「更多」，不删既有功能。
const PRIMARY = new Set(['start', 'panel'])
const SECONDARY = new Set(['stop', 'restart', 'logs', 'advice'])
const primaryAction = computed(() => scenario.value.actions.find(action => PRIMARY.has(action)) ?? null)
const secondaryActions = computed(() => scenario.value.actions.filter(action => SECONDARY.has(action)))
const moreActions = computed(() => scenario.value.actions.filter(action => !PRIMARY.has(action) && !SECONDARY.has(action)))

const lifecycleActions = new Set(['start', 'stop', 'restart'])
function isLifecycleAction(action: string) {
  return lifecycleActions.has(action)
}
function actionLabel(action: string) {
  if (app.processActionBusy && app.processAction === action) {
    return action === 'start' ? '启动中…' : action === 'stop' ? '停止中…' : '重启中…'
  }
  // 与原型一致（overview.html `render()`）：启动失败时启动动作读作「重试启动」，
  // 其余状态沿用同一份动作文案（`actionLabels`）。
  if (action === 'start' && app.runtimeState === 'starting_failed') return '重试启动'
  return controller.actionLabels[action]
}

function openRuntimeDetail() {
  if (gateBlocked.value) return
  detailOpen.value = true
}
function openEnvironment() {
  envOpen.value = true
}

async function copyValue(value: string, event?: MouseEvent) {
  try {
    if (!navigator.clipboard) {
      app.showToast('剪贴板不可用；请手动复制该值。')
      return
    }
    await navigator.clipboard.writeText(value)
    app.showToast(`已复制：${value}`)
  } catch {
    app.showToast('复制失败；请手动复制该值。')
  } finally {
    if (event) (event.currentTarget as HTMLButtonElement | null)?.blur()
  }
}

async function copyCommand(command: string) {
  try {
    if (!navigator.clipboard) {
      app.showToast(`剪贴板不可用；不会执行 ${command}，请手动运行。`)
      return
    }
    await navigator.clipboard.writeText(command)
    copiedCommand.value = command
    window.setTimeout(() => {
      if (copiedCommand.value === command) copiedCommand.value = null
    }, 1400)
    app.showToast(`已复制 ${command}；不会代你执行，请手动运行。`)
  } catch {
    app.showToast(`复制失败；不会执行 ${command}，请手动运行。`)
  }
}

function installManaged(source: 'registry' | 'offline') {
  app.requestInstallModal(source)
  envOpen.value = false
  routes.go('settings', { section: 'installation' })
}

function guide() {
  envOpen.value = false
  routes.go('settings', { section: 'installation' })
}

const mods = computed(() => [
  {
    key: 'upgrade',
    title: '版本升级',
    tag: '管理器 · 面板',
    rows: [],
    actions: [
      { label: '检查更新', cls: 'btn primary', run: () => openUpdates(null, true) },
    ],
  },
  { key: 'migration', title: '配置迁移', tag: '已接入', rows: [['导出配置', '加密容器'], ['导入校验', '备份后替换']], actions: [{ label: '导出配置', cls: 'btn', run: () => routes.go('settings', { section: 'migration' }) }, { label: '导入配置', cls: 'btn', run: () => routes.go('settings', { section: 'migration' }) }] },
  { key: 'sync', title: 'WebDAV 同步', tag: webdavStates[app.webdavState].tag, rows: [['连接状态', webdavStates[app.webdavState].conn], ['冲突策略', webdavStates[app.webdavState].conflict]], actions: webdavStates[app.webdavState].actions.map(a => ({ label: a.label, cls: a.cls, run: () => routes.go('settings', { section: 'sync' }) })) },
])
</script>

<template>
  <UpdateCenter ref="updateCenter" v-model:open="updateOpen" :target="updateTarget" />
  <section class="route-section active ov" :data-mode="mode">
    <AppTopbar :subtitle-override="pageSubtitle" />

    <section class="motion-overview" data-testid="overview-motion">
      <div class="motion-fixed">
        <div class="motion-stage">
          <RuntimeMotionMark :state="motion.state" :active="motionActive" />
        </div>
        <div class="motion-identity">
          <button
            class="motion-mainline"
            type="button"
            :disabled="gateBlocked"
            :title="gateBlocked ? '当前运行状态；请先完成环境准备' : '查看运行详情（含运行环境）'"
            :aria-label="gateBlocked ? `${motion.mainline}，环境准备中` : '查看运行详情，包含运行环境'"
            @click="openRuntimeDetail"
          >
            <span>{{ motion.mainline }}</span>
            <svg v-if="!gateBlocked" viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path d="m9 5 7 7-7 7" /></svg>
          </button>
          <p class="motion-caption" :title="caption">{{ caption }}</p>
        </div>
        <div v-if="!gateBlocked" class="motion-actions">
          <UiButton
            v-if="primaryAction"
            variant="primary"
            :disabled="isLifecycleAction(primaryAction) && app.processActionBusy"
            :loading="isLifecycleAction(primaryAction) && app.processActionBusy && app.processAction === primaryAction"
            @click="controller.runAction(primaryAction)"
          >{{ actionLabel(primaryAction) }}</UiButton>
          <UiButton
            v-for="action in secondaryActions"
            :key="action"
            :variant="action === 'stop' ? 'danger' : (action === 'logs' || action === 'advice') ? 'ghost' : 'default'"
            :disabled="app.processActionBusy"
            :loading="app.processActionBusy && app.processAction === action"
            @click="controller.runAction(action)"
          >{{ actionLabel(action) }}</UiButton>
          <details v-if="moreActions.length" class="motion-more">
            <summary>更多</summary>
            <div class="motion-more-menu">
              <button
                v-for="action in moreActions"
                :key="action"
                type="button"
                class="btn ghost"
                @click="controller.runAction(action)"
              >{{ actionLabel(action) }}</button>
            </div>
          </details>
        </div>
      </div>
    </section>

    <EnvironmentGate v-if="gateBlocked" :on-recheck="recheckEnvironment" />

    <div v-if="!gateBlocked" class="mods">
      <UiCard v-for="mod in mods" :key="mod.key" class="mod">
        <div class="mod-head">
          <h3>{{ mod.title }}</h3>
          <span class="tag">{{ mod.tag }}</span>
          <button
            class="mod-go"
            type="button"
            :aria-label="`进入「${mod.title}」设置`"
            @click="routes.go('settings', { section: mod.key })"
          ><svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5.2" cy="12" r="1.7"/><circle cx="12" cy="12" r="1.7"/><circle cx="18.8" cy="12" r="1.7"/></svg></button>
        </div>
        <div class="mod-status">
          <template v-if="mod.key === 'upgrade'">
            <button class="btn ghost overview-update-row" type="button" aria-label="查看桌面管理器更新详情" @click="openUpdates('manager')"><span>桌面管理器</span><b>{{ managerUpdateStatus?.currentVersion ?? app.aboutApp?.version ?? '—' }}</b><span aria-hidden="true">›</span></button>
            <button class="btn ghost overview-update-row" type="button" aria-label="查看 OpenCodex 面板更新详情" @click="openUpdates('official')"><span>OpenCodex 面板</span><b>{{ app.officialProject?.version ?? '—' }}</b><span aria-hidden="true">›</span></button>
          </template>
          <template v-else><span v-for="row in mod.rows" :key="row[0]">{{ row[0] }} <b>{{ row[1] }}</b></span></template>
        </div>
        <div class="mod-actions">
          <UiButton
            v-for="action in mod.actions"
            :key="action.label"
            :variant="action.cls.includes('primary') ? 'primary' : 'default'"
            :disabled="mod.key === 'upgrade' && action.label === '检查更新' && app.officialProjectLoading"
            @click="action.run?.()"
          >{{ action.label }}</UiButton>
        </div>
      </UiCard>
    </div>

    <UiCard v-if="!gateBlocked" class="recent-events-card" data-testid="overview-recent-events">
      <div class="recent-events-head">
        <h2>最近事件</h2>
        <button class="btn ghost" type="button" @click="routes.go('logs')">诊断中心</button>
      </div>
      <ul class="recent-events-list">
        <li v-if="!recentEvents.length" class="recent-events-empty">暂无事件记录。</li>
        <li v-for="(event, index) in recentEvents" :key="`${event.time}-${index}`">
          <time>{{ event.time }}</time>
          <span>{{ event.message }}</span>
        </li>
      </ul>
    </UiCard>

    <!-- 运行详情弹窗（三段式：标题 / 正文滚动 / 底部关闭）；上部两列进程与环境，下部整宽来源目录。 -->
    <div v-if="detailOpen" class="modal-mask" @click.self="detailOpen = false">
      <div class="modal motion-modal motion-runtime-modal" role="dialog" aria-modal="true" aria-label="运行详情">
        <div class="motion-modal-head">
          <h3>运行详情</h3>
          <button class="btn ghost motion-modal-close" type="button" aria-label="关闭" @click="detailOpen = false">✕</button>
        </div>
        <div class="motion-modal-body">
          <div class="motion-runtime-grid">
            <section aria-labelledby="runtimeProcessTitle">
              <h4 class="motion-section-title" id="runtimeProcessTitle">进程与就绪</h4>
              <dl class="motion-facts">
                <div><dt>运行状态</dt><dd>{{ motion.mainline }}</dd></div>
                <div><dt>健康</dt><dd>{{ scenario.health }}</dd></div>
                <div><dt>PID / 进程</dt><dd>{{ scenario.pid }}</dd></div>
                <div><dt>端口</dt><dd>{{ scenario.port }}</dd></div>
                <div><dt>当前操作</dt><dd>{{ motion.caption || '无' }}</dd></div>
              </dl>
            </section>
            <section class="motion-runtime-environment" aria-labelledby="runtimeEnvironmentTitle">
              <h4 class="motion-section-title" id="runtimeEnvironmentTitle">运行环境</h4>
              <ol class="motion-check-list">
                <li v-for="(check, index) in envChecks" :key="check.name">
                  <span class="motion-step">{{ index + 1 }}</span>
                  <b>{{ check.name }}</b>
                  <span class="motion-check-value" :data-tone="check.tone">{{ check.value }}</span>
                </li>
              </ol>
              <button v-if="gateBlocked" class="btn ghost" type="button" @click="openEnvironment">完整环境指引</button>
            </section>
          </div>
          <section class="motion-runtime-source" aria-labelledby="runtimeSourceTitle">
            <h4 class="motion-section-title" id="runtimeSourceTitle">来源与目录</h4>
            <dl class="motion-runtime-meta">
              <div><dt>版本</dt><dd><span class="motion-runtime-value" :title="version">{{ version }}</span></dd></div>
              <div><dt>运行时</dt><dd><span class="motion-runtime-value" :title="runtime">{{ runtime }}</span></dd></div>
            </dl>
            <dl class="motion-runtime-paths">
              <div>
                <dt>数据目录</dt>
                <dd><span class="motion-runtime-value" :title="dataRoot">{{ dataRoot }}</span><button class="motion-value-copy" type="button" aria-label="复制数据目录" @click="copyValue(dataRoot, $event)">复制</button></dd>
              </div>
              <div>
                <dt>OPENCODEX_HOME</dt>
                <dd><span class="motion-runtime-value" :title="home">{{ home }}</span><button class="motion-value-copy" type="button" aria-label="复制 OPENCODEX_HOME" @click="copyValue(home, $event)">复制</button></dd>
              </div>
            </dl>
          </section>
        </div>
        <div class="motion-modal-actions">
          <button class="btn ghost" type="button" @click="app.openOverviewDataRoot()">打开数据目录</button>
          <button class="btn ghost" type="button" @click="app.openOverviewOpencodexHome()">打开 OPENCODEX_HOME</button>
          <button class="btn primary" type="button" @click="detailOpen = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- 完整环境指引弹窗（可独立从准备卡打开；与运行详情共用三段式框架）。 -->
    <div v-if="envOpen" class="modal-mask" @click.self="envOpen = false">
      <div class="modal motion-modal" role="dialog" aria-modal="true" aria-label="运行环境">
        <div class="motion-modal-head">
          <h3>运行环境</h3>
          <button class="btn ghost motion-modal-close" type="button" aria-label="关闭" @click="envOpen = false">✕</button>
        </div>
        <div class="motion-modal-body">
          <p class="motion-modal-sub">{{ environment.title }}</p>
          <ol class="motion-check-list">
            <li v-for="(check, index) in envChecks" :key="check.name">
              <span class="motion-step">{{ index + 1 }}</span>
              <b>{{ check.name }}</b>
              <span class="motion-check-value" :data-tone="check.tone">{{ check.value }}</span>
            </li>
          </ol>
          <template v-if="environment.state !== 'ready' && environment.state !== 'ready_via_source'">
            <h4 class="motion-section-title">{{ environment.state === 'checking' ? '检查顺序' : '下一步' }}</h4>
            <p class="motion-modal-copy">{{ environment.description }}</p>
            <div v-for="item in environment.commands" :key="item.command" class="motion-command">
              <span>{{ item.label }}</span>
              <button class="cli-copy-btn" :class="{ copied: copiedCommand === item.command }" type="button" @click="copyCommand(item.command)"><code>{{ item.command }}</code></button>
            </div>
          </template>
        </div>
        <div class="motion-modal-actions">
          <template v-if="environment.state === 'missing_ocx'">
            <button class="btn primary" type="button" @click="installManaged('registry')">安装 OpenCodex</button>
            <button class="btn" type="button" @click="installManaged('offline')">导入离线包</button>
          </template>
          <button class="btn ghost" type="button" @click="guide">查看安装指引</button>
          <button class="btn ghost" type="button" @click="recheckEnvironment">重新检查</button>
          <button class="btn primary" type="button" @click="envOpen = false">关闭</button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.overview-update-row { display: flex; justify-content: space-between; width: 100%; gap: var(--space-2); min-height: 26px; padding: var(--space-1) 0; font-size: var(--text-label); }
.overview-update-row > b { margin-left: auto; overflow-wrap: anywhere; }
</style>
