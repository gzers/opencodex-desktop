import { describe, expect, it } from 'vitest'
import { emptyRules, type ChannelPolicy, type ModelPolicy } from '@/features/models/api'
import { canonicalPolicy, samePolicyDraft } from '@/features/models/policyEquality'

function channel(overrides: Partial<ChannelPolicy> = {}): ChannelPolicy {
  return {
    id: 'channel-a',
    label: 'A6api',
    templateRefs: [],
    modelRefs: {},
    connection: {
      providerId: 'a6api',
      protocol: 'openai-chat',
      nodes: [{ id: 'node-a', name: '当前节点', url: 'https://example.test/v1' }],
      activeNodeId: 'node-a',
      credentialRef: null,
      enabled: true,
      allowPrivateNetwork: false,
    },
    selection: { mode: 'all' },
    modelOrder: ['gpt-a', 'gpt-b'],
    discovered: [{ id: 'gpt-a', label: 'GPT A', rules: emptyRules() }],
    defaults: emptyRules(),
    overrides: {},
    familyOverrides: {},
    ...overrides,
  }
}

const policy = (channels: ChannelPolicy[]): ModelPolicy => ({ schemaVersion: 1, templateRefs: [], channels })

describe('渠道草稿等价判定', () => {
  it('读取现有渠道带回的本机读取回执不算修改', () => {
    const saved = channel()
    const read = channel({ connection: { ...saved.connection, sourceReadId: 'read-2026-10-22', importedConfig: { id: 'import-1', digest: 'abc' } } })
    expect(samePolicyDraft(policy([saved]), policy([read]))).toBe(true)
  })

  it('对象键顺序不同不算修改', () => {
    const left = policy([channel()])
    const right: ModelPolicy = { channels: [channel()], templateRefs: [], schemaVersion: 1 }
    expect(canonicalPolicy(left)).toBe(canonicalPolicy(right))
  })

  it('渠道顺序与型号顺序仍然算修改', () => {
    const a = channel()
    const b = channel({ id: 'channel-b', connection: { ...a.connection, providerId: 'codekey' } })
    expect(samePolicyDraft(policy([a, b]), policy([b, a]))).toBe(false)
    expect(samePolicyDraft(policy([a]), policy([{ ...a, modelOrder: ['gpt-b', 'gpt-a'] }]))).toBe(false)
  })

  it('真实字段变化仍然算修改', () => {
    const saved = channel()
    expect(samePolicyDraft(policy([saved]), policy([{ ...saved, connection: { ...saved.connection, enabled: false } }]))).toBe(false)
    expect(samePolicyDraft(policy([saved]), policy([{ ...saved, selection: { mode: 'none' } }]))).toBe(false)
  })

  it('空草稿与未加载状态可区分', () => {
    expect(samePolicyDraft(null, null)).toBe(true)
    expect(samePolicyDraft(policy([]), null)).toBe(false)
  })

  it('保存后重新读取不误报未保存（WP-42 闭环）', () => {
    // 保存写入的草稿被重新读回时，会带回来源回执等记账字段，并可能改变对象键顺序；
    // 这些差异都不能让“刚保存完”的页面重新变成有未保存修改。
    const saved = channel({ selection: { mode: 'whitelist', models: ['gpt-a'] } })
    const reread = channel({
      templateRefs: [],
      modelRefs: {},
      selection: { mode: 'whitelist', models: ['gpt-a'] },
      connection: { ...saved.connection, sourceReadId: 'read-after-save', importedConfig: { id: 'import-after-save', digest: 'digest' } },
    })
    expect(canonicalPolicy(policy([saved]))).toBe(canonicalPolicy(policy([saved])))
    expect(samePolicyDraft(policy([saved]), policy([reread]))).toBe(true)
    // 读回后真正改动仍必须报出来，不能因为放宽等价而吞掉编辑。
    expect(samePolicyDraft(policy([saved]), policy([{ ...reread, selection: { mode: 'none' } }]))).toBe(false)
  })
})
