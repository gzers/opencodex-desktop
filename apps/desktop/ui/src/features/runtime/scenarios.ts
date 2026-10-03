// 运行时场景与动作文案（原 data/mock 的运行时部分，IMP-04 §5 按功能拆归）。
import type { RuntimeState } from '@/types/ui'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
import { healthLabel } from '@/lib/labels'

export interface RuntimeScenario {
  label: string
  pillClass: string
  badgeClass: string
  health: string
  port: string
  pid: string
  actions: string[]
  logKind: 'normal' | 'risk' | 'error'
}

export function statusScenario(base: RuntimeScenario, snapshot: StatusSnapshot | null): RuntimeScenario {
  const port = snapshot?.port
  const pid = snapshot?.pid
  const facts = snapshot?.facts
  return {
    ...base,
    // 概览状态动作严格对齐已确认原型 `actionsByState`（2026-10-03 用户确认：原型对「未运行」的
    // 呈现才是对的）。at_risk 的进程事实就是「未运行」，概览与 stopped 同样只暴露「启动 OpenCodex」，
    // 不再注入「查看建议」；成因走通知中心/诊断，托盘另按自己的门控显示。
    actions: base.actions,
    // 后端 health 为英文枚举（healthy/degraded/…），统一转中文。
    health: facts?.health ? healthLabel(facts.health) : base.health,
    port: port === null || port === undefined ? base.port : String(port),
    pid: pid ?? base.pid,
  }
}

// 概览状态卡只呈现事实与动作，不再承载说明句（2026-09-25，UI规范 §19）；
// 说明句按性质分流：需关注 → 持久通知，稳定事实 → 一次 Toast，瞬时进度 → 不写。
export const runtimeScenarios: Record<RuntimeState, RuntimeScenario> = {
  // port / pid 是未观测到快照时的占位；`statusScenario` 会用真实快照覆盖它们。
  loading: { label: '加载中', pillClass: 'state-starting', badgeClass: 'busy', health: '探测中', port: '—', pid: '—', actions: [], logKind: 'normal' },
  // 刷新是**全局**能力（顶栏右上角图标，IMP-06），不再挂在某个状态的动作里（IMP-07 对齐原型
  // `actionsByState`）：状态动作只保留与运行态同源的能力，避免出现「更多里只有刷新」的空壳折叠。
  not_found: { label: '未发现', pillClass: 'state-muted', badgeClass: '', health: '未检测', port: '—', pid: '—', actions: [], logKind: 'normal' },
  stopped: { label: '未运行', pillClass: 'state-muted', badgeClass: '', health: '未知', port: '—', pid: '—', actions: ['start'], logKind: 'normal' },
  starting: { label: '启动中', pillClass: 'state-starting', badgeClass: 'busy', health: '等待确认', port: '—', pid: '—', actions: [], logKind: 'normal' },
  pending: { label: '待就绪', pillClass: 'state-starting', badgeClass: 'busy', health: '等待就绪', port: '—', pid: '—', actions: [], logKind: 'normal' },
  running: { label: '运行中', pillClass: 'state-running', badgeClass: 'running', health: '正常', port: '—', pid: '—', actions: ['panel', 'stop', 'restart', 'logs'], logKind: 'normal' },
  stopping: { label: '停止中', pillClass: 'state-starting', badgeClass: 'busy', health: '未知', port: '—', pid: '—', actions: [], logKind: 'normal' },
  starting_failed: { label: '启动失败', pillClass: 'state-bad', badgeClass: 'bad', health: '失败', port: '—', pid: '—', actions: ['start', 'logs'], logKind: 'error' },
  // at_risk：进程事实「未运行」，概览动作与 stopped 一致（只「启动 OpenCodex」）；配色仍走 amber。
  at_risk: { label: '存在风险', pillClass: 'state-warn', badgeClass: 'warn', health: '正常', port: '—', pid: '—', actions: ['start'], logKind: 'risk' },
  external_takeover: { label: '外部接管', pillClass: 'state-warn', badgeClass: 'warn', health: '正常', port: '—', pid: '—', actions: ['advice'], logKind: 'risk' },
  unreachable: { label: '不可达', pillClass: 'state-bad', badgeClass: 'bad', health: '不可达', port: '—', pid: '—', actions: ['logs'], logKind: 'error' },
}

// 稳定事实的说明句（迁移到该状态时播报**一次**，不进通知中心）。
export const runtimeAnnouncements: Partial<Record<RuntimeState, string>> = {
  stopped: '已发现安装，代理未运行，可以直接启动。',
  running: 'OpenCodex 正在运行，可停止或重启。',
}

// 瞬时进度：不播报、不写通知，由状态点与按钮 spinner 表达。
export const transitionalRuntimeStates: RuntimeState[] = [
  'loading',
  'starting',
  'pending',
  'stopping',
]

export const actionLabels: Record<string, string> = {
  start: '启动 OpenCodex',
  panel: '打开面板',
  stop: '停止',
  restart: '重启',
  advice: '查看建议',
  refresh: '刷新状态',
  logs: '查看日志',
}
