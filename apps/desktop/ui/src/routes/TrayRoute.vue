<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { trayRuntimeLabel } from '@/lib/labels'
import type { RuntimeState, TrayOs } from '@/types/ui'

const routes = useRouteStore()
const app = useAppStore()
const controller = useAppController()
const trayOs = ref<TrayOs>('mac')
const trayOpen = ref(false)
const trayState = computed(() => app.runtimeState)

const trayHints: Record<string, Partial<Record<RuntimeState, string>>> = {
  start: { stopped: '可用', starting: '启动中', running: '已运行', starting_failed: '可重试', at_risk: '可用', external_takeover: '先确认' },
  stop: { stopped: '未运行', starting: '稍后停止', running: '可用', starting_failed: '未运行', external_takeover: '需先确认' },
  restart: { stopped: '未运行', starting: '稍后重启', running: '可用', starting_failed: '可重试', external_takeover: '需先确认' },
  panel: { stopped: '需运行', starting: '等待就绪', running: '可用', starting_failed: '不可用', external_takeover: '只读' },
  doctor: { stopped: '可选', starting: '等待', running: '可选', starting_failed: '建议', external_takeover: '建议' },
}

// 托盘标签只讲进程事实：`at_risk` 在托盘显示「未运行」，不写「存在风险」（2026-09-24 用户决策）。
const label = computed(() => trayRuntimeLabel(trayState.value))
const dotClass = computed(() => trayState.value === 'pending' ? 'starting' : trayState.value)
// 地址来自真实快照；未运行时显示中性占位，不使用原型固定端口。
const trayAddress = computed(() => {
  const port = app.statusSnapshot?.port
  if (app.statusSnapshot?.matrix.runtime === 'running' && port) return `OpenCodex · 127.0.0.1:${port}`
  return 'OpenCodex · 未运行'
})
const trayItems = [
  { action: 'start', label: '启动 OpenCodex', hintKey: 'start' },
  { action: 'stop', label: '停止', hintKey: 'stop' },
  { action: 'restart', label: '重启', hintKey: 'restart' },
] as const
const toggleItems = ref([
  { key: 'launchWithCodex', label: '随 Codex 启动 OpenCodex', checked: true },
  { key: 'showMainWindow', label: '启动时显示主界面', checked: true },
])

function toggle() {
  trayOpen.value = !trayOpen.value
  if (trayOpen.value) setTimeout(() => document.addEventListener('click', onDocumentClick))
  else document.removeEventListener('click', onDocumentClick)
}
function close() {
  trayOpen.value = false
  document.removeEventListener('click', onDocumentClick)
}
function onDocumentClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.tray-menu') && !target.closest('.tray-icon')) close()
}
function trayRun(action: string) {
  controller.runAction(action)
}
onBeforeUnmount(close)
</script>

<template>
  <section class="route-section active tray-route">
    <article class="card">
      <div class="card-head">
        <div><h2>托盘菜单预览</h2><p>展示 macOS 与 Windows 的样式差异；菜单动作全部由管理器自有域触发，不直接读写 OpenCodex 配置。</p></div>
      </div>
      <div class="tray-os" role="group">
        <button :class="{ active: trayOs === 'mac' }" @click="trayOs='mac'; trayOpen=false">macOS 菜单栏</button>
        <button :class="{ active: trayOs === 'windows' }" @click="trayOs='windows'; trayOpen=false">Windows 通知区</button>
      </div>
      <div class="tray-menu-wrap">
        <div class="tray-desktop-area" aria-hidden="true"></div>
        <div v-if="trayOs === 'mac'" class="tray-menubar mac">
          <span class="menu-name">Finder</span>
          <span>File</span><span>Edit</span><span>View</span><span>Window</span><span>Help</span>
          <span class="right-area">
            <span class="mini">Wi-Fi</span><span class="mini">电池</span><span class="battery-box"></span><span class="battery-cap"></span>
            <button class="tray-icon" :class="{ active: trayOpen }" :aria-expanded="trayOpen ? 'true' : 'false'" @click.stop="toggle">
              <svg class="tray-logo" :class="'state-' + dotClass" width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" fill="currentColor"/></svg>
              <span class="tray-state"><span class="tray-dot" :class="dotClass"></span><span>{{ label }}</span></span>
            </button>
          </span>
        </div>
        <div v-else class="tray-statusbar windows">
          <span class="left-area"><span class="sys-icon">⊞</span><span class="sys-icon">搜索</span><span class="sys-icon">任务视图</span><span class="sys-icon">Edge</span><span class="sys-icon">资源管理器</span></span>
          <span class="right-area">
            <button class="tray-icon" :class="{ active: trayOpen }" :aria-expanded="trayOpen ? 'true' : 'false'" @click.stop="toggle">
              <svg class="tray-logo" :class="'state-' + dotClass" width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" fill="currentColor"/></svg>
              <span class="tray-state"><span class="tray-dot" :class="dotClass"></span><span>{{ label }}</span></span>
            </button>
            <svg class="sys-icon" viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 8.5A15 15 0 0 1 22 8.5"/><path d="M6 12a9.5 9.5 0 0 1 12 0"/><path d="M9.5 15.5a4.5 4.5 0 0 1 5 0"/><circle cx="12" cy="19" r="1" fill="currentColor"/></svg>
            <svg class="sys-icon" viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 9v6"/><path d="M8 6v12"/><path d="M12 8v8"/><path d="M16 5v14"/><path d="M20 10v4"/></svg>
            <span class="sys-icon">09:41</span>
          </span>
        </div>
        <div v-if="trayOpen" class="tray-menu" :class="trayOs">
          <div class="tray-title"><b>{{ label }}</b><div>{{ trayAddress }}</div></div>
          <div class="tray-sep"></div>
          <button v-for="item in trayItems" :key="item.action" class="tray-item" @click="trayRun(item.action)">
            {{ item.label }}<span class="hint">{{ trayHints[item.hintKey]?.[trayState] ?? '—' }}</span>
          </button>
          <div class="tray-sep"></div>
          <button class="tray-item" @click="routes.go('overview')">打开主界面<span class="hint">App</span></button>
          <button class="tray-item" @click="routes.go('panel')">打开官方面板<span class="hint">{{ trayHints.panel[trayState] ?? '—' }}</span></button>
          <div class="tray-sep"></div>
          <button class="tray-item" @click="routes.go('logs')">打开诊断中心</button>
          <button class="tray-item" @click="routes.go('logs', { tab: 'doctor' })">运行 Doctor<span class="hint">{{ trayHints.doctor[trayState] ?? '—' }}</span></button>
          <div class="tray-sep"></div>
          <button v-for="toggleItem in toggleItems" :key="toggleItem.key" class="tray-item toggle-item" :aria-checked="toggleItem.checked ? 'true' : 'false'" @click="toggleItem.checked = !toggleItem.checked">
            {{ toggleItem.label }}<span class="toggle-mark">{{ toggleItem.checked ? '开启' : '关闭' }}</span>
          </button>
          <div class="tray-sep"></div>
          <button class="tray-item danger" @click="app.exitApp()">完全退出桌面壳<span class="hint">⌘Q</span></button>
        </div>
      </div>
    </article>
  </section>
</template>
