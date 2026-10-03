// 桌面生命周期与恢复引导控制器（IMP-04 §19.4 E：从 useAppController 拆出）。
// 运行状态设置、启停动作、托盘请求分派、恢复前确认与官方恢复引导。
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { actionLabels } from '@/features/runtime/scenarios'
import { healthLabel, runtimeLabel } from '@/lib/labels'
import type { TrayAction } from '@/features/tray/api'
import type { RuntimeState } from '@/types/ui'

export function useLifecycleController(deps: { runDoctor: () => void | Promise<void>; loadLogs: () => void | Promise<void> }) {
  const app = useAppStore()
  const routes = useRouteStore()

  function setRuntimeState(next: RuntimeState) {
    app.setRuntimeState(next)
  }

  // 「启停结果通知」关闭时不弹启停 Toast；失败通知由后端同一偏好单独门控。
  function lifecycleNotificationsEnabled(): boolean {
    return app.preferences?.lifecycleNotifications !== false
  }

  async function runLifecycleAction(action: 'start' | 'stop' | 'restart') {
    try {
      const result = await app.requestProcessAction(action)
      const status = result?.result ?? 'cancelled'
      const succeeded = status === 'started' || status === 'stopped'
      const label = actionLabels[action]
      if (lifecycleNotificationsEnabled()) {
        app.showToast(
          succeeded
            ? action === 'start' || action === 'restart'
              ? `${label}指令已发出，正在等待就绪。`
              : `${label}已完成。`
            : `${label}指令未确认完成，请刷新状态。`,
        )
      }
      await app.loadStatusSnapshot()
    } catch {
      if (lifecycleNotificationsEnabled()) {
        app.showToast(`${actionLabels[action]}执行失败；已保留当前状态。`)
      }
      await app.loadStatusSnapshot()
    }
  }

  function runAction(action: string) {
    if (action === 'start' || action === 'stop' || action === 'restart') {
      void runLifecycleAction(action)
      return
    }
    if (action === 'refresh') {
      void app.loadStatusSnapshot().then(() => {
        void deps.loadLogs()
      })
      return
    }
    if (action === 'panel') {
      // 面板是桌面壳内的固定路由；不要再创建第二个原生窗口。
      routes.go('panel')
    }
    if (action === 'logs') routes.go('logs')
    if (action === 'advice') openRestoreConfirm(app.runtimeState)
  }

  async function openRestoreConfirm(target: RuntimeState) {
    app.openModal({
      title: '正在准备恢复确认',
      body: '<p>正在读取当前状态与配置影响摘要；不会读取配置内容或凭据。</p>',
      cancelLabel: '取消',
    })
    const loaded = await app.loadRestoreRiskSummary()
    if (!loaded) {
      app.openModal({
        title: '影响摘要不可用',
        body: '<p>无法读取当前状态或配置存在性；已保留当前配置，不会进入恢复引导。</p>',
        confirmLabel: '重试',
        cancelLabel: '稍后处理',
        onConfirm: () => {
          void openRestoreConfirm(target)
        },
      })
      return
    }
    const summary = app.restoreRiskSummary
    if (!summary) return
    const facts = summary.guidance
    const flag = (value: boolean) => (value ? '已配置' : '未发现')
    const risk =
      target === 'external_takeover'
        ? '检测到 OpenCodex 配置已被外部接管。继续前请先确认以下信息：'
        : '检测到 OpenCodex 启动状态存在风险。继续前请先确认以下信息：'
    const protectionLabel = (value: string | null) => {
      if (value === null) return '未知'
      const normalized = value.trim().toLowerCase()
      if (['none', 'off', 'disabled', 'no'].includes(normalized)) return '未开启'
      if (['enabled', 'on', 'active', 'yes'].includes(normalized)) return '已开启'
      // 未识别的官方取值不直接展示英文原文。
      return '未知'
    }
    const startupLabel =
      facts.startupStatus === null
        ? '未知'
        : facts.startupStatus.trim().toLowerCase().replace(/[-_]/g, ' ') === 'at risk'
          ? '存在风险'
          : '未知'
    const body = `<p class="modal-lead">${risk}</p>
<ul>
<li>运行状态：${runtimeLabel(facts.runtimeState)}</li>
<li>健康状态：${healthLabel(facts.health)}</li>
<li>启动状态：${startupLabel}</li>
<li>启动保护：${protectionLabel(facts.protection)}</li>
<li>重启安全：${facts.rebootSafe === null ? '未知' : facts.rebootSafe ? '是' : '否'}</li>
<li>系统服务：${facts.servicePresent === null ? '未知' : facts.servicePresent ? '已配置' : '未配置'}；启动辅助程序：${facts.shimPresent === null ? '未知' : facts.shimPresent ? '已配置' : '未配置'}</li>
<li>管理器偏好设置：${flag(summary.preferencesConfigured)}</li>
<li>扩展配置：${flag(summary.extensionConfigured)}</li>
<li>同步端点：${flag(summary.syncEndpointConfigured)}</li>
<li>数据根目录：${facts.dataRoot ? `<code>${facts.dataRoot}</code>` : '未获取'}</li>
<li>OPENCODEX_HOME 配置目录：${summary.opencodexHome ? `<code>${summary.opencodexHome}</code>` : '未获取'}</li>
</ul>
<p>本应用不会修改外部配置，只会说明当前影响。你可以先生成备份，再按官方指引执行恢复命令；操作可随时取消。</p>`
    app.openModal({
      title: '恢复前确认',
      body,
      wide: true,
      confirmLabel: '生成备份',
      cancelLabel: '稍后处理',
      onConfirm: () => {
        void app.createRestoreBackup().then(backup => {
          if (!backup) return
          const summaryState = target === 'external_takeover' ? '配置外部接管' : '启动状态存在风险'
          app.openModal({
            title: '官方恢复引导',
            body: `<p class="modal-lead">备份已生成。请复制下方命令，在终端确认后执行。</p>
<ul>
<li>备份 ID：<code>${backup.backupId}</code></li>
<li>备份目录：<code>${backup.directory}</code></li>
<li>备份来源：<code>${backup.targetPath}</code></li>
<li>当前摘要：<code>${summaryState}</code></li>
</ul>
<pre class="modal-code"><code>ocx restore</code></pre>
<p>本应用不会代为执行恢复，也不会包装或接管官方恢复命令。</p>`,
            wide: true,
            confirmLabel: '复制命令',
            cancelLabel: '稍后处理',
            onConfirm: () => {
              void navigator.clipboard?.writeText('ocx restore')
              app.showToast('已复制官方 restore 命令。')
            },
          })
        })
      },
    })
  }

  function handleTrayRequests(actions: TrayAction[]) {
    for (const action of actions) {
      if (action === 'start' || action === 'stop' || action === 'restart') {
        runAction(action)
        continue
      }
      if (action === 'open_main') {
        routes.go('overview')
        continue
      }
      if (action === 'open_panel') {
        routes.go('panel')
      }
      if (action === 'open_logs') routes.go('logs')
      if (action === 'open_settings') {
        routes.go('settings', { section: 'general' })
        continue
      }
      if (action === 'open_data_dir') {
        void app.loadManagedPathTargets().then(loaded => {
          if (!loaded) {
            app.showToast('数据目录不可用；已保留当前窗口。')
            return
          }
          const target = app.managedPathTargets?.find(item => item.key === 'manager_state')
          if (!target) {
            app.showToast('数据目录不可用；已保留当前窗口。')
            return
          }
          void app.openManagedPath(target.path).then(opened => {
            if (!opened) app.showToast('数据目录不可用；已保留当前窗口。')
          })
        })
        continue
      }
      // Doctor 现在是「环境诊断」Tab 的内容：先切到该 Tab，再运行，否则用户看不到结果。
      if (action === 'run_doctor') {
        routes.go('logs', { tab: 'doctor' })
        void deps.runDoctor()
      }
    }
  }

  return { setRuntimeState, runAction, handleTrayRequests, openRestoreConfirm }
}
