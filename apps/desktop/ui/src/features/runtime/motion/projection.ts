// 状态形象动画的**投影**：把既有的运行主线、刷新新鲜度与持续问题投影成一种
// 形象状态。它是纯函数、只读，绝不新建状态机，也不替换 `stores/app` 里的
// `runtimeState`（单一真源）。生产接入只消费这里的结果。
import type { RuntimeState } from '@/contracts/runtimeStatus'

export type MotionStateId =
  | 'not_ready'
  | 'stopped'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'failed'
  | 'problem'
  | 'confirming'
  | 'stale'

export interface MotionProjectionInput {
  /** 运行主线（来自既有状态投影；与列表/托盘同源，不在此重新推断）。 */
  runtimeState: RuntimeState
  /**
   * 事实是否新鲜。缺省视为新鲜——在真正的旧响应保护落地前，不得伪造「过期」。
   * 仅在明确判定事实过期（如最后成功时间超阈值）时才置 false。
   */
  fresh?: boolean
  /** 是否存在持续性待处理问题（来源接管/不可达/运行中问题）。 */
  problem?: boolean
}

export interface MotionProjection {
  state: MotionStateId
  /** 主线结论：待接入 / 未运行 / 运行中 / 正在确认。 */
  mainline: string
  /** 一句话现状说明；空串表示不需要额外说明。 */
  caption: string
  /** 是否表示「旧事实」。 */
  stale: boolean
}

/**
 * 主线标签按**运行事实**取，与已确认原型 `STATE_MODEL.main` 一致：
 * `pending`（进程已起、待健康/端口核验）归「运行中」，不是「未运行」。
 *
 * `at_risk` 由「代理**未在运行** + 官方 startup at-risk」折叠而来，主线只能说
 * 「未运行」——风险结论与成因走 `caption`（启动保护未启用），与托盘同一口径
 * （`runtime_label(AtRisk) = 未运行`，2026-09-24 决策）。曾把它归「运行中」，
 * 结果是启停两态都显示「运行中」、状态形象也不变（2026-09-29 用户报告）。
 */
const MAINLINE_BY_RUNTIME: Record<RuntimeState, string> = {
  loading: '正在确认',
  not_found: '待接入',
  stopped: '未运行',
  starting: '未运行',
  pending: '运行中',
  running: '运行中',
  stopping: '运行中',
  starting_failed: '未运行',
  at_risk: '未运行',
  external_takeover: '运行中',
  unreachable: '运行中',
}

/** 进行中的观测/操作不被旧事实顶替，不显示「过期」。 */
const IN_FLIGHT: ReadonlySet<RuntimeState> = new Set<RuntimeState>([
  'loading',
  'starting',
  'pending',
  'stopping',
])

/** 运行主线 → 形象状态（不含新鲜度覆盖）。 */
function baseState(runtimeState: RuntimeState, problem: boolean): MotionStateId {
  switch (runtimeState) {
    case 'not_found':
      return 'not_ready'
    case 'stopped':
      return 'stopped'
    case 'starting':
    case 'pending':
      return 'starting'
    case 'stopping':
      return 'stopping'
    case 'starting_failed':
      return 'failed'
    case 'external_takeover':
    case 'unreachable':
      return 'problem'
    case 'at_risk':
      // 代理未运行：保留「未运行」的结构（分离三球），风险只体现在说明与配色语义。
      return 'stopped'
    case 'running':
      return problem ? 'problem' : 'running'
    case 'loading':
    default:
      return 'confirming'
  }
}

export function projectMotion(input: MotionProjectionInput): MotionProjection {
  const { runtimeState, fresh = true, problem = false } = input
  // 过期只覆盖已落定的主线；进行中的操作/确认不被旧事实顶替。
  const stale = fresh === false && !IN_FLIGHT.has(runtimeState)
  const state = stale ? 'stale' : baseState(runtimeState, problem)
  return {
    state,
    // 过期时主线归「正在确认」，与原型 stale.main 一致。
    mainline: stale ? '正在确认' : MAINLINE_BY_RUNTIME[runtimeState],
    caption: captionFor(state, runtimeState),
    stale,
  }
}

function captionFor(state: MotionStateId, runtimeState: RuntimeState): string {
  switch (state) {
    case 'confirming':
      return runtimeState === 'loading' ? '等待首次观测' : '没有可用新鲜观测'
    case 'not_ready':
      return '接入后可启动'
    case 'starting':
      // UI规范 §25.1 确认文案：进程已起用「进程已起 · 待就绪」，启动指令阶段用「启动中 · 待核验」。
      return runtimeState === 'pending' ? '进程已起 · 待就绪' : '启动中 · 待核验'
    case 'stopping':
      return '停止中 · 待核验'
    case 'failed':
      return '启动失败'
    case 'stopped':
      // at_risk 的进程事实是「未运行」：概览形象与「未运行」逐字一致，不再另加说明句
      // （2026-10-03 用户确认：原型的「未运行」呈现才是对的）。at_risk 只体现在配色与托盘，
      // 成因走通知中心/诊断，不进概览 hero。
      return ''
    case 'problem':
      // 具体成因进入全局待处理/诊断，主线说明只给一句概括（§25.1、原型 sync()）。
      return '有待处理问题'
    case 'stale':
      return '事实已过期 · 等待重新采集'
    default:
      return ''
  }
}
