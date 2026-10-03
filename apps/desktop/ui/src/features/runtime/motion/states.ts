// 概览状态形象动画的**演示模型**：把既有的运行/操作/问题/新鲜度投影映射为
// 「一种内部聚合结构 + 配色 + 节奏」。它不采集状态、不推进业务结论，也不是新状态机。
import type { MotionStateId } from './projection'

export const BRAND_COLORS = ['#B1A7FF', '#7A9DFF', '#3941FF'] as const

/** 结构幅度固定为已确认的最大值（不随窗口/状态变化）。 */
export const STRUCTURE_AMPLITUDE = 1.25

/** 结构参数：`[节点散布增量, 连接显现度, 柔性弯曲正弦分量, 余弦分量]`。 */
export type MotionShape = readonly [number, number, number, number]

export interface MotionPalette {
  name: string
  colors: readonly [string, string, string]
}

export interface MotionEffect {
  /** 整体自转目标角速度（度/秒）；0 表示回正静止。 */
  outer: number
  /** 内核自转角速度（度/秒）。 */
  inner: number
  /** 摆动叠加角速度（度/秒）。 */
  rock: number
  /** 循环周期（秒）。 */
  period: number
  /** 背景能量强度 0–1。 */
  energy: number
  title: string
  body: string
  core: string
  light: string
}

export interface MotionState {
  id: MotionStateId
  name: string
  short: string
  /** 主线结论（与项目核心主线一致）。 */
  main: string
  /** 节点操作/结果摘要；`—` 表示无。 */
  op: string
  verb: string
  meaning: string
  note: string
  shape: MotionShape
  palette: MotionPalette
  effect: MotionEffect
}

export const PALETTES: Record<MotionStateId, MotionPalette> = {
  not_ready: { name: '雾蓝 · 待聚合', colors: ['#CCD3EA', '#A4B3D6', '#778CB7'] },
  stopped: { name: '柔紫 · 静置', colors: ['#C7B5F4', '#929ADF', '#6473BB'] },
  starting: { name: '鸢紫 → 电光蓝', colors: ['#E0A0FF', '#8775F5', '#398AF2'] },
  running: { name: '薄荷青 → 靛蓝', colors: ['#80E9CD', '#3BAED1', '#5861E8'] },
  stopping: { name: '暮蓝 · 收束', colors: ['#B7CAEF', '#9494DC', '#7675B3'] },
  failed: { name: '珊瑚 → 莓紫', colors: ['#FFC39E', '#F08093', '#B35BB4'] },
  problem: { name: '琥珀 → 薰衣草', colors: ['#FFE0A0', '#ECAF68', '#8A83D9'] },
  confirming: { name: '冰蓝 → 浅紫', colors: ['#B2DCF8', '#8EAAF2', '#9F80E7'] },
  stale: { name: '雾灰 · 缓动', colors: ['#CFD4DF', '#A2ABBE', '#828DA4'] },
}

export const EFFECTS: Record<MotionStateId, MotionEffect> = {
  not_ready: { outer: 0, inner: 0, rock: 18, period: 7, energy: 0.25, title: 'B · 离散漂浮', body: '轻缓上下漂浮', core: '三球分离、低速探寻', light: '雾蓝散光' },
  stopped: { outer: 0, inner: 0, rock: 7, period: 6, energy: 0.2, title: 'B · 低位待命', body: '较低悬浮位置、小幅起伏', core: '三球待命、无连接线', light: '柔紫光晕与近地投影' },
  starting: { outer: 20, inner: -120, rock: 0, period: 3.2, energy: 1, title: 'B → C · 轻跃蓄能', body: '小幅跃起＋整体慢转', core: '柔性连接建立、内核反转', light: '蓝紫聚光、投影随高度变化' },
  running: { outer: 0, inner: 60, rock: 0, period: 6, energy: 0.75, title: 'A · 稳定悬浮', body: '平稳悬浮、不改变比例', core: '完整内核固定形状 · 6s/圈', light: '青蓝流光与柔和投影' },
  stopping: { outer: 0, inner: 0, rock: 6, period: 5, energy: 0.35, title: 'A → C → B · 下沉', body: '缓慢下沉后低位浮动', core: '减速、曲线柔和退去', light: '暮蓝收束、投影聚拢' },
  failed: { outer: 0, inner: 0, rock: 18, period: 5, energy: 0.4, title: 'C → B · 回落', body: '一次短促回落、随后低浮', core: '连接退去、三球回摆', light: '珊瑚柔光，无闪烁' },
  problem: { outer: 0, inner: 34, rock: 0, period: 5, energy: 0.55, title: 'C · 柔性牵动', body: '轻微横向浮动', core: '连续曲线轻弯、保持连接', light: '琥珀局部流光' },
  confirming: { outer: 0, inner: 0, rock: 65, period: 4.5, energy: 0.55, title: 'B · 悬浮探寻', body: '缓慢横向漂移', core: '分离三球、双向寻位', light: '冰蓝往返光场' },
  stale: { outer: 0, inner: 0, rock: 0, period: 9, energy: 0.15, title: '旧结构 · 失重漂移', body: '低幅漂浮、去饱和', core: '保留旧结构、平滑减速停住', light: '雾灰残光' },
}

export const MOTION_STATES: MotionState[] = [
  { id: 'not_ready', name: '待接入', short: 'B · 离散漂浮', main: '待接入', op: '—', verb: '三球分离，云朵轻轻漂浮', meaning: '没有连接线。三个球围绕统一中心均衡分布，漂浮表达等待接入。', note: '尚未接入运行来源，不视为故障。', shape: [0.55, 0, 0, 0], palette: PALETTES.not_ready, effect: EFFECTS.not_ready },
  { id: 'stopped', name: '未运行', short: 'B · 低位待命', main: '未运行', op: '—', verb: '三球规整待命，云朵落在较低的悬浮位置', meaning: '小幅起伏保持轻盈感，不使用正常运行的整圈旋转。', note: '来源已接入，可启动。', shape: [0.2, 0, 0, 0], palette: PALETTES.stopped, effect: EFFECTS.stopped },
  { id: 'starting', name: '正在启动', short: 'B → C · 轻跃聚合', main: '未运行', op: '启动 / 执行中', verb: '轻跃蓄能，分离三球逐渐建立柔性连接', meaning: '内外反向旋转，柔性曲线保持完整圆润；只有运行核验通过才进入 A 完整内核。', note: '启动执行中，尚未核验运行。', shape: [0.16, 0.65, 0.16, 0], palette: PALETTES.starting, effect: EFFECTS.starting },
  { id: 'running', name: '运行中', short: 'A · 稳定悬浮', main: '运行中', op: '—', verb: '完整内核固定形状旋转，云朵稳定悬浮', meaning: '连接线不再伸缩或碎裂；三球与连接整体匀速转动，外形保持原始比例。', note: '新鲜观测确认运行。', shape: [0, 1, 0, 0], palette: PALETTES.running, effect: EFFECTS.running },
  { id: 'stopping', name: '正在停止', short: 'A → C → B · 下沉', main: '运行中', op: '停止 / 核验中', verb: '内核减速，连接柔和退去，云朵缓慢下沉', meaning: '过渡结束后保持低位浮动。动画不自行宣布停止，主线等待核验。', note: '等待停止核验。', shape: [0.2, 0, 0, 0], palette: PALETTES.stopping, effect: EFFECTS.stopping },
  { id: 'failed', name: '启动失败', short: 'C → B · 回落', main: '未运行', op: '启动 / 失败', verb: '接合退回，云朵短促回落后低位漂浮', meaning: '回落只在进入失败时发生一次，之后三球有限角回摆；不循环假装重新启动。', note: '失败属于启动操作；可查看原因与重试。', shape: [0.3, 0, 0, 0], palette: PALETTES.failed, effect: EFFECTS.failed },
  { id: 'problem', name: '运行中 · 待处理', short: 'C · 柔性牵动', main: '运行中', op: '—', verb: '连续曲线轻微牵动，云朵不规则但平缓地悬浮', meaning: '保持完整连接和持续转动，以柔性张力、琥珀光和文字表达运行中问题。', note: '仍在运行，存在待处理问题。', shape: [0, 1, 0.22, 0], palette: PALETTES.problem, effect: EFFECTS.problem },
  { id: 'confirming', name: '正在确认', short: 'B · 悬浮探寻', main: '正在确认', op: '读取状态', verb: '分离三球双向寻位，云朵缓慢横向漂移', meaning: '无连接的内核往返扫描，不表示正常运行，也不按时间猜测结果。', note: '没有可用新鲜观测。', shape: [0.35, 0, 0, 0], palette: PALETTES.confirming, effect: EFFECTS.confirming },
  { id: 'stale', name: '状态已过期', short: '保留结构 · 失重漂移', main: '正在确认', op: '等待重新采集', verb: '保留上一结构，褪色后缓缓漂移', meaning: '保留最后的内部结构，转速平滑衰减后停住，只留下低幅整体漂移和雾灰残光。', note: '上次结构仅为旧事实，当前状态已过期。', shape: [0, 1, 0, 0], palette: PALETTES.stale, effect: EFFECTS.stale },
]

const BY_ID = new Map(MOTION_STATES.map((state) => [state.id, state]))
export function motionState(id: MotionStateId): MotionState {
  return BY_ID.get(id) ?? MOTION_STATES[0]
}

export const BRAND_RGB = BRAND_COLORS.map(rgb)

export function rgb(hex: string): [number, number, number] {
  const parts = hex.match(/[a-f\d]{2}/gi) ?? ['00', '00', '00']
  return [parseInt(parts[0], 16), parseInt(parts[1], 16), parseInt(parts[2], 16)]
}

export function paletteRgb(id: MotionStateId): Array<[number, number, number]> {
  return PALETTES[id].colors.map(rgb)
}

export function colorText(color: readonly number[]): string {
  return `rgb(${color.map((value) => Math.round(value)).join(',')})`
}

export interface MarkGeometry {
  nodes: Array<{ x: number; y: number }>
  links: string[]
  connection: number
}

/** 由结构参数计算节点位置与三段柔性连接路径（纯函数，供渲染与测试）。 */
export function computeGeometry(
  shape: MotionShape,
  amount: number,
  center: { x: number; y: number } = { x: 12, y: 12 },
  baseRadius = 3.5,
): MarkGeometry {
  const [spread, connection, bendSin, bendCos] = shape
  const radius = baseRadius + spread * amount
  const points = Array.from({ length: 3 }, (_, index) => {
    const angle = -Math.PI / 2 + (index * Math.PI * 2) / 3
    return { a: angle, x: center.x + radius * Math.cos(angle), y: center.y + radius * Math.sin(angle) }
  })
  // 120° 圆弧的三次贝塞尔近似；曲线延伸至圆心、被圆节点覆盖，没有短线碎片。
  const tangent = (4 / 3) * Math.tan(Math.PI / 6) * radius
  const links = points.map((point, index) => {
    const next = points[(index + 1) % 3]
    const bend = amount * (bendSin * Math.cos((index * Math.PI * 2) / 3) + bendCos * Math.sin((index * Math.PI * 2) / 3))
    const mid = point.a + Math.PI / 3
    const bx = Math.cos(mid) * bend
    const by = Math.sin(mid) * bend
    return (
      `M${point.x} ${point.y} ` +
      `C${point.x - Math.sin(point.a) * tangent + bx} ${point.y + Math.cos(point.a) * tangent + by} ` +
      `${next.x + Math.sin(next.a) * tangent + bx} ${next.y - Math.cos(next.a) * tangent + by} ` +
      `${next.x} ${next.y}`
    )
  })
  return {
    nodes: points.map((point) => ({ x: point.x, y: point.y })),
    links,
    connection: Math.max(0, Math.min(1, connection)),
  }
}
