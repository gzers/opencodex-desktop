import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ModelPicker from '@/features/models/ModelPicker.vue'
import { emptyRules, type ModelProjection } from '@/features/models/api'
import { mergeUpstream } from '@/features/models/upstream'
const row = (id: string): ModelProjection => ({ channelId: 'channel', providerId: 'provider', modelId: id, label: id, enabled: true, selected: true, discovered: true, templateRef: null, rules: { ...emptyRules(), fast: { mode: 'custom', value: true }, reasoningEfforts: { mode: 'custom', value: ['none','high'] } }, source: {} })
const menu = { props: ['open'], template: '<div v-if="open"><slot/></div>' }
const dialog = { template: '<div><slot/><slot name="actions"/></div>' }
describe('public model selection', () => {
  it('keeps a selected identity during load/error, blocks stale submission and delegates retry to the caller',async()=>{
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[row('a')],channels:[],multiple:true,current:[{channelId:'channel',modelId:'a'}],loading:true},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    expect(wrapper.text()).toContain('已选模型 · 1');expect(wrapper.text()).toContain('正在读取模型目录');expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined()
    await wrapper.setProps({loading:false,error:'服务读取失败'});await wrapper.findAll('button').find(b=>b.text()==='重新读取目录')!.trigger('click');expect(wrapper.emitted('retry')).toHaveLength(1);expect(wrapper.emitted('confirm')).toBeUndefined()
    await wrapper.setProps({error:undefined});await wrapper.findAll('button').find(b=>b.text()==='确定选择')!.trigger('click');expect(wrapper.emitted('confirm')?.[0]).toEqual([[{channelId:'channel',modelId:'a'}]]);wrapper.unmount()
  })
  it('preserves native identity without inventing a channel reference and respects caller unavailability',async()=>{
    const native={...row('native-id'),channelId:'native-display-group',nativeModelId:'native-id'}
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[native],channels:[{id:'native-display-group',label:'官方 / 原生'}]},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    await wrapper.get('.model-picker-row').trigger('click');await wrapper.findAll('button').find(b=>b.text()==='确定选择')!.trigger('click')
    expect(wrapper.emitted('confirm')?.[0]).toEqual([[{nativeModelId:'native-id'}]])
    await wrapper.setProps({rows:[{...native,unavailableReason:'原生目录尚无管理器执行绑定'}]});expect(wrapper.text()).toContain('原生目录尚无管理器执行绑定');expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined();wrapper.unmount()
  })
  it('shows and sorts recommendations only when caller supplies exact identities, without changing picked priority',async()=>{
    const wrapper=mount(ModelPicker,{props:{open:true,rows:[row('a'),row('b')],channels:[],multiple:true,ordered:true,recommendations:[{channelId:'channel',modelId:'b'}],current:[{channelId:'channel',modelId:'a'},{channelId:'channel',modelId:'b'}]},global:{stubs:{UiDialog:dialog,UiSelectMenu:menu}}})
    await wrapper.findAll('th button').find(b=>b.text().includes('推荐'))!.trigger('click');expect(wrapper.findAll('.model-picker-row strong').map(r=>r.text())).toEqual(['b','a'])
    await wrapper.findAll('button').find(b=>b.text()==='确定选择')!.trigger('click');expect(wrapper.emitted('confirm')?.[0]).toEqual([[{channelId:'channel',modelId:'a'},{channelId:'channel',modelId:'b'}]])
    await wrapper.setProps({recommendations:[]});expect(wrapper.findAll('th button').some(b=>b.text().includes('推荐'))).toBe(false);wrapper.unmount()
  })
  it('uses the public display-name field while leaving the exact native model id intact',()=>{
    const result=mergeUpstream({version:'2.84.0',providers:{providers:{test:{adapter:'openai-chat',baseUrl:'https://example.test/v1',modelDisplayNames:{'native/id':'我的显示名'},modelAliases:{'native/id':'routing-alias'}}}},models:[{provider:'test',id:'native/id',displayName:'上游名称'}],order:{}},[])
    expect(result[0]!.discovered[0]).toMatchObject({id:'native/id',label:'我的显示名'})
  })
  it('does not turn a generated namespaced routing label into an edited display name',()=>{
    const value=mergeUpstream({version:'2.84.0',providers:{providers:{test:{adapter:'openai-chat',baseUrl:'https://example.test/v1'}}},models:[{provider:'test',id:'native/id',namespaced:'test/native-id',displayName:'test/native-id'},{provider:'test',id:'b',namespaced:'test/b',displayName:'用户定义名称'}],order:{}},[])
    expect(value[0]!.discovered.map(m=>({id:m.id,label:m.label}))).toEqual([{id:'native/id',label:'native/id'},{id:'b',label:'用户定义名称'}])
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
  it('does not list or select a row the caller reports unavailable', async () => {
    const item = row('unknown'); item.rules = emptyRules(); item.enabled = false; item.unavailableReason = '渠道停用'
    const wrapper = mount(ModelPicker, { props: { open: true, rows: [item], channels: [] }, global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } } })
    // 不可用型号不进可浏览列表。
    expect(wrapper.find('.model-picker-row').exists()).toBe(false)
    expect(wrapper.findAll('button').find(b=>b.text()==='确定选择')!.attributes('disabled')).toBeDefined()
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
  it('lists only picker-visible models and hides channels left without any', async () => {
    const visible = row('visible')
    const hidden = { ...row('hidden'), pickerVisible: false, unavailableReason: '型号已隐藏，请重新选择', enabled: false }
    const wrapper = mount(ModelPicker, {
      props: {
        open: true,
        rows: [visible, hidden, { ...row('other'), channelId: 'hidden-channel', enabled: false, unavailableReason: '渠道停用' }],
        channels: [{ id: 'channel', label: '可见渠道' }, { id: 'hidden-channel', label: '全隐藏渠道' }],
        // 已保存但如今失效的选择：仍在“已选模型”里，并写明原因。
        current: [{ channelId: 'channel', modelId: 'hidden' }],
      },
      global: { stubs: { UiDialog: dialog, UiSelectMenu: menu } },
    })
    // 可浏览列表只列可选型号；隐藏型号与停用渠道的行都不出现。
    expect(wrapper.findAll('.model-picker-row').map(r => r.get('strong').text())).toEqual(['visible'])
    expect(wrapper.find('nav').text()).toContain('可见渠道')
    expect(wrapper.find('nav').text()).not.toContain('全隐藏渠道')
    expect(wrapper.text()).toContain('型号已隐藏，请重新选择')
    wrapper.unmount()
  })

  it('refreshes discovery without replacing existing identity, scope, node or order', () => {
    const first = mergeUpstream({ version: '2.84.0', providers: { providers: { test: { adapter: 'openai-chat', baseUrl: 'https://example.test/v1' } } }, models: [{ provider: 'test', id: 'one' }], order: {} }, [])
    first[0]!.label = '我的渠道'; first[0]!.selection = { mode: 'none' }; first[0]!.modelOrder = ['old','one']
    const next = mergeUpstream({ version: '2.84.0', providers: { providers: { test: { adapter: 'openai-responses', baseUrl: 'https://changed.test' } } }, models: [{ provider: 'test', id: 'two' }], order: {} }, first)
    expect(next[0]!.id).toBe(first[0]!.id); expect(next[0]!.label).toBe('我的渠道'); expect(next[0]!.selection).toEqual({ mode: 'none' })
    expect(next[0]!.connection.nodes[0]!.url).toBe('https://example.test/v1'); expect(next[0]!.modelOrder).toEqual(['old','one'])
    expect(next[0]!.discovered.map(m => m.id)).toEqual(['two'])
  })
})
