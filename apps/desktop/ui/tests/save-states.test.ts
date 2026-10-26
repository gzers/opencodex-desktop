import {describe,expect,it} from 'vitest'
import {
  TONE_CLASS,
  applyingPhaseLabel,
  channelPolicyState,
  saveStateOf,
  type SaveStateId,
} from '@/lib/saveStates'

/** 后端 runtime.rs 实际出现的全部 23 个 phase（20 个赋值 + 3 个字面量）。 */
const ALL_PHASES = [
  'applying','committed','compensated','compensating','compensating_direction',
  'credentials_registered','credentials_verified','hiding_none_selection',
  'history_authorizations','history_providers','history_registering','new_registered',
  'order_written','pending_active','prepared','providers_written','registering_new',
  'runtime_verified','staging','verified_partial','visibility_written',
  'writing_credentials','writing_imported_authorizations',
]

const EXPECTED: Record<string, SaveStateId> = {
  prepared: 'prepared',
  staging: 'staging',
  applying: 'applying', hiding_none_selection: 'applying', providers_written: 'applying',
  writing_imported_authorizations: 'applying', order_written: 'applying', visibility_written: 'applying',
  writing_credentials: 'applying', credentials_registered: 'applying', registering_new: 'applying',
  new_registered: 'applying', history_providers: 'applying', history_authorizations: 'applying',
  history_registering: 'applying', pending_active: 'applying',
  committed: 'verified', runtime_verified: 'verified', credentials_verified: 'verified',
  verified_partial: 'partial',
  compensating: 'rolling-back', compensating_direction: 'rolling-back',
  compensated: 'rolled-back',
}

describe('保存状态归并（23 phase → 结论级）', () => {
  it('后端全部 23 个 phase 都有归属，没有一个落到 unknown', () => {
    for (const phase of ALL_PHASES) {
      const state = saveStateOf(phase)
      expect(state.id, phase).toBe(EXPECTED[phase])
      expect(state.id, phase).not.toBe('unknown')
    }
    expect(ALL_PHASES).toHaveLength(23)
  })

  it('14 个写入阶段统一显示「正在保存」，不把内部节奏暴露为常驻文案', () => {
    const applying = ALL_PHASES.filter(p => EXPECTED[p] === 'applying')
    expect(applying).toHaveLength(14)
    for (const phase of applying) {
      const state = saveStateOf(phase)
      expect(state.label).toBe('正在保存')
      expect(state.tone).toBe('starting')
    }
  })

  it('写入阶段有 hover 用的阶段名；非写入阶段没有', () => {
    expect(applyingPhaseLabel('writing_credentials')).toBe('写入凭据')
    expect(applyingPhaseLabel('order_written')).toBe('写入顺序')
    expect(applyingPhaseLabel('committed')).toBeUndefined()
    expect(applyingPhaseLabel(null)).toBeUndefined()
  })

  it('未知 phase 落到「未完整写入」，不静默当成成功或失败', () => {
    const state = saveStateOf('some_future_phase')
    expect(state.id).toBe('unknown')
    expect(state.label).toBe('未完整写入')
    expect(state.tone).toBe('warn')
    expect(state.actionable).toBe(true)
  })

  it('没有 plan 是未修改；verified / partial / failed 都是需要处理的结果', () => {
    expect(saveStateOf(null).id).toBe('idle')
    expect(saveStateOf(undefined).label).toBe('未修改')
    expect(saveStateOf('committed').actionable).toBe(true)
    expect(saveStateOf('verified_partial').actionable).toBe(true)
    expect(saveStateOf('verified_partial').label).toBe('未完整写入')
  })
})

describe('tone 唯一词表', () => {
  it('只有五类语义，类名全部落在既有 .state-* 上', () => {
    expect(Object.keys(TONE_CLASS).sort()).toEqual(['bad','muted','running','starting','warn'])
    for (const cls of Object.values(TONE_CLASS)) expect(cls).toMatch(/^state-(running|starting|warn|bad|muted)$/)
  })

  it('不产生 data-tone 别名（success/ok/info/busy/warning/danger/bad/neutral/muted/warn）', () => {
    const values = Object.values(TONE_CLASS).join(' ')
    for (const alias of ['state-success','state-ok','state-info','state-busy','state-warning','state-danger','state-neutral']) {
      expect(values).not.toContain(alias)
    }
  })
})


describe('列表页唯一状态（3 处收敛为 1 处）', () => {
  it('进行中的保存优先于草稿修改', () => {
    const state = channelPolicyState({ saving: true, draftDirty: true, pendingRows: 2 })
    expect(state.label).toBe('正在保存')
  })

  it('待确认编辑先于未保存修改（与原型 channelPolicyState 优先级一致）', () => {
    expect(channelPolicyState({ pendingRows: 3, draftDirty: true }).label).toBe('有待确认编辑')
    expect(channelPolicyState({ draftDirty: true }).label).toBe('未保存修改')
    expect(channelPolicyState({ overBudget: true }).label).toBe('超出激活预算')
  })

  it('没有改动时是未修改', () => {
    expect(channelPolicyState({}).label).toBe('未修改')
  })
})

