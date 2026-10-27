// 模型保存域的结论级状态：把后端 23 个 `plan.phase` 收敛成用户能行动的少数几种。
//
// 与 lib/labels.ts 同一职责边界：
// - 界面**不得**直接渲染后端 phase 原值（英文中间态），一律经本模块。
// - tone 只取 5 类语义（running / starting / warn / bad / muted），对应 styles/base.css
//   既有 `.state-*` 六类类名；不再引入 data-tone 别名（见 2026-10-25 裁决 #2）。
// - 状态**结论**归前端：phase 是事务内部节奏，可能继续增删（2026-10-25 裁决）。
//
// 依据：docs/…/20261025-状态展示统一化提案.md §2.1；原型 model-config.js:431 九态。

/** 五类语义 tone → 既有 CSS 类名。新增别名一律不进来。 */
export type StatusTone = 'running' | 'starting' | 'warn' | 'bad' | 'muted'

/** tone → `.state-*` 类名，供 .state-pill 组合使用。 */
export const TONE_CLASS: Record<StatusTone, string> = {
  running: 'state-running',
  starting: 'state-starting',
  warn: 'state-warn',
  bad: 'state-bad',
  muted: 'state-muted',
}

/**
 * 结论级保存状态。前 7 个对应原型九态的业务语义，后 3 个是终态。
 * `id` 与原型 data-channel-state 对齐（除 prepared/staging 为软件侧细分）。
 */
export type SaveStateId =
  | 'idle'
  | 'prepared'
  | 'staging'
  | 'applying'
  | 'verified'
  | 'partial'
  | 'rolling-back'
  | 'rolled-back'
  | 'failed'
  | 'cancelled'
  | 'unknown'

export interface SaveState {
  id: SaveStateId
  label: string
  tone: StatusTone
  /** 是否仍需用户处理（决定要不要进通知 / 要不要显示核对入口）。 */
  actionable: boolean
}

/**
 * 23 个后端 phase → 结论级状态。
 *
 * 归并原则（裁决 #1 = 选项 C）：写入过程中的 14 个内部阶段统一显示「正在保存」，
 * 阶段名只在 hover / focus 出现（由调用方从 `applyingPhaseLabel` 取），不占常驻宽度。
 */
const PHASE_TO_STATE: Record<string, SaveStateId> = {
  prepared: 'prepared',
  staging: 'staging',

  // —— 正在写入：14 个内部阶段 ——
  applying: 'applying',
  hiding_none_selection: 'applying',
  providers_written: 'applying',
  writing_imported_authorizations: 'applying',
  order_written: 'applying',
  visibility_written: 'applying',
  writing_credentials: 'applying',
  credentials_registered: 'applying',
  registering_new: 'applying',
  new_registered: 'applying',
  history_providers: 'applying',
  history_authorizations: 'applying',
  history_registering: 'applying',
  pending_active: 'applying',

  // —— 写入完成并核实 ——
  committed: 'verified',
  runtime_verified: 'verified',
  credentials_verified: 'verified',

  // —— 部分核实：需核对 ——
  verified_partial: 'partial',

  // —— 回滚 ——
  compensating: 'rolling-back',
  compensating_direction: 'rolling-back',
  compensated: 'rolled-back',
}

/** 结论级状态的文案与 tone。文案逐字沿用原型九态，不另造词。 */
const STATE_META: Record<SaveStateId, Omit<SaveState, 'id'>> = {
  idle: { label: '未修改', tone: 'muted', actionable: false },
  prepared: { label: '待写入', tone: 'muted', actionable: false },
  staging: { label: '正在准备', tone: 'starting', actionable: false },
  applying: { label: '正在保存', tone: 'starting', actionable: false },
  verified: { label: '已保存', tone: 'muted', actionable: false },
  // 流程第 ④ 态（0.2.11）：运行目录与那次写入不一致，或写入未完整结束。
  partial: { label: '未完整写入', tone: 'warn', actionable: true },
  'rolling-back': { label: '正在回滚', tone: 'warn', actionable: false },
  'rolled-back': { label: '已撤回本次写入', tone: 'muted', actionable: false },
  failed: { label: '保存失败', tone: 'bad', actionable: true },
  cancelled: { label: '保存已取消', tone: 'muted', actionable: true },
  // 未登记阶段：同样落进第 ④ 态，不静默当成成功或失败。
  unknown: { label: '未完整写入', tone: 'warn', actionable: true },
}

/** 写入中 14 个阶段的阶段名（裁决 #1：只在 hover / focus 显示）。 */
const PHASE_LABEL: Record<string, string> = {
  applying: '校验',
  hiding_none_selection: '校验',
  providers_written: '写入渠道',
  writing_imported_authorizations: '写入授权',
  order_written: '写入顺序',
  visibility_written: '写入可见性',
  writing_credentials: '写入凭据',
  credentials_registered: '登记凭据',
  registering_new: '登记新渠道',
  new_registered: '登记新渠道',
  history_providers: '记录历史',
  history_authorizations: '记录历史',
  history_registering: '记录历史',
  pending_active: '核对目录',
}

/**
 * 把后端 phase 归约为结论级状态。
 *
 * - 未知 phase 一律落到 `unknown`（不静默当成成功或失败；与 §5.8「不假称」一致）。
 * - 没有 plan（undefined/null）表示没有进行中的保存 → `idle`。
 */
export function saveStateOf(phase: string | null | undefined): SaveState {
  if (!phase) return { id: 'idle', ...STATE_META.idle }
  // 失败也可能只是「结果未知」，不据 error 字段改写 phase 结论；失败由 saveFailureState() 显式表达。
  const id = PHASE_TO_STATE[phase] ?? 'unknown'
  return { id, ...STATE_META[id] }
}

/** 失败 / 拦下的终态：错误已定，覆盖 phase 结论。 */
export function saveFailureState(): SaveState {
  return { id: 'failed', ...STATE_META.failed }
}

/** 取消：草稿保留，可继续保存。 */
export function saveCancelledState(): SaveState {
  return { id: 'cancelled', ...STATE_META.cancelled }
}

/**
 * 写入中的阶段名，供 hover / focus 的 title 使用。
 * 非写入阶段返回 undefined，调用方据此不挂 title。
 */
export function applyingPhaseLabel(phase: string | null | undefined): string | undefined {
  if (!phase) return undefined
  return PHASE_LABEL[phase]
}

/**
 * 0.2.12：客户端加载态（restart_required / pending_verification / user_confirmed /
 * runtime_changed）整组删除。保存成功后只给一条通知，点「稍后」即彻底结束——
 * 不再有「之后还要再确认一次」的状态，这正是 0.2.11 死锁的根源。
 */
/** 草稿本身的编辑状态（未保存修改 / 待确认编辑 / 超限），与保存事务分开判定。 */
export type DraftStateId = 'clean' | 'dirty' | 'staged' | 'over-budget'

export interface DraftState {
  id: DraftStateId
  label: string
  tone: StatusTone
}

const DRAFT_META: Record<DraftStateId, Omit<DraftState, 'id'>> = {
  clean: { label: '未修改', tone: 'muted' },
  dirty: { label: '未保存修改', tone: 'warn' },
  staged: { label: '有待确认编辑', tone: 'warn' },
  'over-budget': { label: '超出激活预算', tone: 'bad' },
}

/**
 * 渠道列表顶部要显示的**唯一**一个状态（裁决：3 处 → 1 处）。
 *
 * 进行中的保存与未完整写入优先；已完成的旧计划不能遮住后来编辑的草稿。
 *
 * 0.2.12：不再有「客户端待处理」一档——保存成功后只给一条通知，不置状态。
 */
export function channelPolicyState(input: {
  saving?: boolean
  plan?: { phase: string | null; error?: string | null } | null
  pendingRows?: number
  draftDirty?: boolean
  overBudget?: boolean
}): SaveState {
  if (input.saving) return { id: 'applying', ...STATE_META.applying }

  const plan = input.plan
  if (plan?.phase) {
    const state = saveStateOf(plan.phase)
    if (!['idle', 'verified', 'rolled-back'].includes(state.id)) return state
  }

  if (input.pendingRows) return { id: 'staged', ...DRAFT_META.staged } as unknown as SaveState
  if (input.overBudget) return { id: 'over-budget', ...DRAFT_META['over-budget'] } as unknown as SaveState
  if (input.draftDirty) return { id: 'dirty', ...DRAFT_META.dirty } as unknown as SaveState
  if (plan?.phase) return saveStateOf(plan.phase)
  return { id: 'idle', ...STATE_META.idle }
}

