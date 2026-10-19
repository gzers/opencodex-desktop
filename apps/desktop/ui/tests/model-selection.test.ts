import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ModelPicker from '@/features/models/ModelPicker.vue'
import { emptyRules, type ModelProjection } from '@/features/models/api'
import { mergeUpstream } from '@/features/models/upstream'
const row = (id: string): ModelProjection => ({ channelId: 'channel', providerId: 'provider', modelId: id, label: id, enabled: true, selected: true, discovered: true, templateRef: null, rules: { ...emptyRules(), fast: { mode: 'custom', value: true }, reasoningEfforts: { mode: 'custom', value: ['none','high'] } }, source: {} })
const menu = { props: ['open'], template: '<div v-if="open"><slot/></div>' }
const dialog = { template: '<div><slot/><slot name="actions"/></div>' }
describe('public model selection', () => {
  it('uses the public display-name field while leaving the exact native model id intact',()=>{
    const result=mergeUpstream({version:'2.84.0',providers:{providers:{test:{adapter:'openai-chat',baseUrl:'https://example.test/v1',modelDisplayNames:{'native/id':'我的显示名'},modelAliases:{'native/id':'routing-alias'}}}},models:[{provider:'test',id:'native/id',displayName:'上游名称'}],order:{}},[])
    expect(result[0]!.discovered[0]).toMatchObject({id:'native/id',label:'我的显示名'})
  })
  it('keeps effort and Fast changes pending until the main confirmation', async () => {
    const wrapper = mount(ModelPicker, { props: { open: false, rows: [row('model-a')], channels: [{ id: 'channel', label: '测试渠道' }], reasoning: true }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    await wrapper.setProps({ open: true })
    await wrapper.get('.model-picker-row').trigger('click')
    expect(wrapper.find('input[type=radio]').exists()).toBe(false)
    const button = (text: string) => wrapper.findAll('button').find(b => b.text() === text)!
    await button('high').trigger('click'); await wrapper.get('input[type=checkbox]').setValue(true); await wrapper.get('input[type=checkbox]').setValue(false)
    expect(wrapper.emitted('confirm')).toBeUndefined()
    await button('确定选择').trigger('click')
    expect(wrapper.emitted('confirm')?.[0]).toEqual([[{ channelId: 'channel', modelId: 'model-a', reasoningEffort: 'high', fast: false }]])
  })
  it('keeps multiple selection across filtering and cancellation does not submit', async () => {
    const wrapper = mount(ModelPicker, { props: { open: false, rows: [row('model-a'), row('model-b')], channels: [], multiple: true }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    await wrapper.setProps({ open: true }); await wrapper.findAll('.model-picker-row input[type=checkbox]')[0]!.setValue(true)
    await wrapper.get('input[aria-label="搜索模型"]').setValue('model-b'); await wrapper.get('.model-picker-row input[type=checkbox]').setValue(true)
    expect(wrapper.text()).toContain('已选 2 项')
    await wrapper.findAll('button').find(b => b.text() === '取消')!.trigger('click')
    expect(wrapper.emitted('confirm')).toBeUndefined(); expect(wrapper.emitted('close')).toHaveLength(1)
  })
  it('does not infer support or select an unavailable row', async () => {
    const item = row('unknown'); item.rules = emptyRules(); item.enabled = false
    const wrapper = mount(ModelPicker, { props: { open: true, rows: [item], channels: [] }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    await wrapper.get('.model-picker-row').trigger('click')
    expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('渠道停用')
  })
  it('cycles numeric sorting through both directions and original order with unknown values last',async()=>{
    const unknown=row('unknown');const small=row('small');small.rules.contextWindow={mode:'custom',value:1000};const large=row('large');large.rules.contextWindow={mode:'custom',value:9000}
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[unknown,large,small],channels:[]},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    const names=()=>wrapper.findAll('.model-picker-row strong').map(item=>item.text())
    const button=()=>wrapper.findAll('th button').find(b=>b.text().startsWith('上下文'))!
    await button().trigger('click');expect(names()).toEqual(['small','large','unknown'])
    await button().trigger('click');expect(names()).toEqual(['large','small','unknown'])
    await button().trigger('click');expect(names()).toEqual(['unknown','large','small']);wrapper.unmount()
  })
  it('keeps missing and incompatible selected references visible and prevents confirmation',async()=>{
    const item=row('present')
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[item],channels:[],current:[{channelId:'channel',modelId:'missing'}],reasoning:true},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    expect(wrapper.text()).toContain('型号已缺失');expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined()
    await wrapper.get('.model-picker-row').trigger('click')
    expect(wrapper.text()).not.toContain('型号已缺失');expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeUndefined()
    const changed={...item,enabled:false};await wrapper.setProps({rows:[changed]})
    expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined();wrapper.unmount()
  })
  it('bulk-selects only the filtered enabled result and keeps picks outside that result',async()=>{
    const a=row('a');const b=row('b');const disabled={...row('b-disabled'),enabled:false}
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[a,b,disabled],channels:[],multiple:true,current:[{channelId:'channel',modelId:'a'}]},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    await wrapper.get('input[aria-label="搜索模型"]').setValue('b');await wrapper.get('thead input[type=checkbox]').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()==='确定选择')!.trigger('click')
    expect(wrapper.emitted('confirm')![0]).toEqual([[{channelId:'channel',modelId:'a'},{channelId:'channel',modelId:'b'}]])
    await wrapper.findAll('button').find(b=>b.text()==='清空')!.trigger('click')
    expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeUndefined();wrapper.unmount()
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
