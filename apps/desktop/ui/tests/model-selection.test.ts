import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ModelPicker from '@/features/models/ModelPicker.vue'
import { emptyRules, type ModelProjection } from '@/features/models/api'
import { mergeUpstream } from '@/features/models/upstream'
const row = (id: string): ModelProjection => ({ channelId: 'channel', providerId: 'provider', modelId: id, label: id, enabled: true, selected: true, discovered: true, templateRef: null, rules: { ...emptyRules(), fast: { mode: 'custom', value: true }, reasoningEfforts: { mode: 'custom', value: ['none','high'] } }, source: {} })
const menu = { props: ['open'], template: '<div v-if="open"><slot/></div>' }
const dialog = { template: '<div><slot/><slot name="actions"/></div>' }
describe('public model selection', () => {
  it('keeps effort and Fast changes pending until the main confirmation', async () => {
    const wrapper = mount(ModelPicker, { props: { open: false, rows: [row('model-a')], channels: [{ id: 'channel', label: '测试渠道' }], reasoning: true }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    await wrapper.setProps({ open: true })
    await wrapper.get('input[type=radio]').setValue(true)
    const button = (text: string) => wrapper.findAll('button').find(b => b.text() === text)!
    await button('high').trigger('click'); await button('开启').trigger('click'); await button('关闭').trigger('click')
    expect(wrapper.emitted('confirm')).toBeUndefined()
    await button('确定选择').trigger('click')
    expect(wrapper.emitted('confirm')?.[0]).toEqual([[{ channelId: 'channel', modelId: 'model-a', reasoningEffort: 'high', fast: false }]])
  })
  it('keeps multiple selection across filtering and cancellation does not submit', async () => {
    const wrapper = mount(ModelPicker, { props: { open: false, rows: [row('model-a'), row('model-b')], channels: [], multiple: true }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    await wrapper.setProps({ open: true }); await wrapper.findAll('input[type=checkbox]')[0]!.setValue(true)
    await wrapper.get('input[type=search]').setValue('model-b'); await wrapper.get('input[type=checkbox]').setValue(true)
    expect(wrapper.text()).toContain('已选 2 项')
    await wrapper.findAll('button').find(b => b.text() === '取消')!.trigger('click')
    expect(wrapper.emitted('confirm')).toBeUndefined(); expect(wrapper.emitted('close')).toHaveLength(1)
  })
  it('does not infer support or select an unavailable row', async () => {
    const item = row('unknown'); item.rules = emptyRules(); item.enabled = false
    const wrapper = mount(ModelPicker, { props: { open: true, rows: [item], channels: [] }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    expect(wrapper.get('input[type=radio]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('渠道停用')
  })
})
describe('official snapshot adoption', () => {
  it('refreshes discovery without replacing existing identity, scope, node or order', () => {
    const first = mergeUpstream({ version: '2.84.0', providers: { providers: { test: { adapter: 'openai-chat', baseUrl: 'https://example.test/v1' } } }, models: [{ provider: 'test', id: 'one' }], order: {} }, [])
    first[0]!.label = '我的渠道'; first[0]!.selection = { mode: 'none' }; first[0]!.modelOrder = ['old','one']
    const next = mergeUpstream({ version: '2.84.0', providers: { providers: { test: { adapter: 'openai-responses', baseUrl: 'https://changed.test' } } }, models: [{ provider: 'test', id: 'two' }], order: {} }, first)
    expect(next[0]!.id).toBe(first[0]!.id); expect(next[0]!.label).toBe('我的渠道'); expect(next[0]!.selection).toEqual({ mode: 'none' })
    expect(next[0]!.connection.nodes[0]!.url).toBe('https://example.test/v1'); expect(next[0]!.modelOrder).toEqual(['old','one'])
    expect(next[0]!.discovered.map(m => m.id)).toEqual(['two'])
  })
})
