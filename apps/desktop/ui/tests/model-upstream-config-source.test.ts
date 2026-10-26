import { describe, expect, it } from 'vitest'
import { mergeUpstream } from '@/features/models/upstream'
import { emptyRules, type ChannelPolicy, type UpstreamSnapshot } from '@/features/models/api'

/**
 * 0.2.13 / WP-74：config.json 是唯一事实源（UI 规范 §43）。
 *
 * 旧行为：`discovered` 只来自 snapshot.models（官方目录）。当官方目录对该渠道为空，
 * 界面就显示 0 个型号，而 config.json 里其实声明了一百多个；保存时还会被
 * `model_unverified` 拦下。实测：A6api 在 config.json 有 116 个型号，界面显示 0 个。
 */
function snapshot(providers: Record<string, unknown>, models: Record<string, unknown>[]): UpstreamSnapshot {
  return { readId: 'read-1', providers: { providers }, models } as unknown as UpstreamSnapshot
}

describe('config.json 是渠道型号的事实源', () => {
  it('官方目录为空时，仍按 config.json 声明的 models 列出型号', () => {
    const next = mergeUpstream(
      snapshot(
        { A6api: { adapter: 'openai-responses', baseUrl: 'https://api.example.test', models: ['a', 'b', 'c'], selectedModels: ['a', 'c'] } },
        [],
      ),
      [],
    )
    expect(next).toHaveLength(1)
    expect(next[0]!.discovered.map(m => m.id)).toEqual(['a', 'b', 'c'])
    // 已选型号原样保留，不做「是否核对过」的判断。
    expect(next[0]!.selection).toEqual({ mode: 'whitelist', models: ['a', 'c'] })
  })

  it('官方目录只含部分型号时，补上仅声明的那些', () => {
    const next = mergeUpstream(
      snapshot(
        { demo: { adapter: 'openai-responses', baseUrl: 'https://api.example.test', models: ['online', 'offline'] } },
        [{ id: 'online', provider: 'demo', namespaced: 'demo/online' }],
      ),
      [],
    )
    expect(next[0]!.discovered.map(m => m.id)).toEqual(['online', 'offline'])
  })

  it('不重复：官方目录与声明都有的型号只出现一次', () => {
    const next = mergeUpstream(
      snapshot(
        { demo: { adapter: 'openai-responses', baseUrl: 'https://api.example.test', models: ['x'] } },
        [{ id: 'x', provider: 'demo', namespaced: 'demo/x' }],
      ),
      [],
    )
    expect(next[0]!.discovered.filter(m => m.id === 'x')).toHaveLength(1)
  })

  it('仅声明的型号不编造能力：规则留空，继承渠道默认', () => {
    const next = mergeUpstream(
      snapshot(
        { demo: { adapter: 'openai-responses', baseUrl: 'https://api.example.test', models: ['only-declared'] } },
        [],
      ),
      [],
    )
    expect(next[0]!.discovered[0]!.rules).toEqual(emptyRules())
  })

  it('已有渠道只读取、不改配置时，保留原 id 与选择', () => {
    const existing: ChannelPolicy = {
      id: 'keep-id', label: 'demo', templateRefs: [], modelRefs: {},
      connection: { providerId: 'demo', protocol: 'openai-responses', nodes: [{ id: 'n1', name: '当前节点', url: 'https://api.example.test' }], activeNodeId: 'n1', credentialRef: null, enabled: true, allowPrivateNetwork: false },
      selection: { mode: 'whitelist', models: ['x'] }, modelOrder: ['x'], discovered: [],
      defaults: emptyRules(), overrides: {}, familyOverrides: {},
    }
    const next = mergeUpstream(
      snapshot(
        { demo: { adapter: 'openai-responses', baseUrl: 'https://api.example.test', models: ['x', 'y'] } },
        [],
      ),
      [existing],
    )
    expect(next[0]!.id).toBe('keep-id')
    expect(next[0]!.selection).toEqual({ mode: 'whitelist', models: ['x'] })
    expect(next[0]!.discovered.map(m => m.id)).toEqual(['x', 'y'])
  })
})

