import {mount} from '@vue/test-utils'
import {describe,expect,it,vi} from 'vitest'
import SelectorPreviewDialog from '@/features/models/SelectorPreviewDialog.vue'
import {emptyRules,type DraftProjection,type ModelPolicy} from '@/features/models/api'

const dialog={props:['open'],template:'<div v-if="open"><slot/><slot name="actions"/></div>'}

function row(channelId:string,modelId:string,selected=true){
  return {channelId,providerId:channelId,modelId,label:modelId,enabled:true,selected,discovered:true,templateRef:null,rules:emptyRules(),source:{}}
}
function projection():DraftProjection{
  return {rows:[row('a','gpt-5.4'),row('a','glm-flash',false),row('b','claude-opus-5')],knownActiveModels:2,exactCatalogCount:3,notices:[]}
}
function saved():ModelPolicy{
  return {schemaVersion:1,templateRefs:[],channels:[
    {id:'a',label:'A渠道',templateRefs:[],modelRefs:{},connection:{providerId:'a',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['gpt-5.4','glm-flash']},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}},
    {id:'b',label:'B渠道',templateRefs:[],modelRefs:{},connection:{providerId:'b',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:[]},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}},
  ]}
}
function render(overrides:Record<string,unknown>={}){
  return mount(SelectorPreviewDialog,{props:{open:true,projection:projection(),saved:saved(),channels:[{id:'a',label:'A渠道',enabled:true},{id:'b',label:'B渠道',enabled:true,border:true}].map(c=>({id:c.id,label:c.label,enabled:c.enabled})),...overrides},global:{stubs:{UiDialog:dialog}}})
}

describe('保存 · 选择器预览（UI 规范 §38）',()=>{
  it('固定当前草稿，没有预览来源与数量折叠',()=>{
    const wrapper=render()
    expect(wrapper.text()).not.toContain('预览来源')
    expect(wrapper.text()).not.toContain('按渠道查看数量')
    expect(wrapper.find('select').exists()).toBe(false)
    expect(wrapper.findAll('.preview-group')).toHaveLength(2)
    wrapper.unmount()
  })

  it('按渠道分组、组内保持顺序，并标出激活预算',()=>{
    const wrapper=render()
    expect(wrapper.findAll('.preview-group strong').map(n=>n.text())).toEqual(['A渠道','B渠道'])
    expect(wrapper.findAll('.preview-group')[0]!.findAll('.preview-item span').map(n=>n.text())).toEqual(['gpt-5.4','glm-flash'])
    expect(wrapper.text()).toContain('模型激活数 2/100')
    wrapper.unmount()
  })

  it('渠道标题行整行可点，用于停用与恢复',async()=>{
    const wrapper=render()
    await wrapper.findAll('.preview-group-title')[0]!.trigger('click')
    expect(wrapper.emitted('toggle-channel')?.[0]).toEqual(['a'])
    wrapper.unmount()
  })

  it('型号标签单独启停，并暴露 aria-pressed',async()=>{
    const wrapper=render()
    const items=wrapper.findAll('.preview-item')
    expect(items[0]!.attributes('aria-pressed')).toBe('true')
    expect(items[1]!.attributes('aria-pressed')).toBe('false')
    await items[0]!.trigger('click')
    expect(wrapper.emitted('toggle-model')?.[0]).toEqual(['a','gpt-5.4'])
    wrapper.unmount()
  })

  it('本次变更直接标在型号标签与渠道卡上，不另起差异清单',()=>{
    const wrapper=render()
    expect(wrapper.find('.preview-item.is-added').text()).toContain('claude-opus-5')
    // “新 / 停”由样式伪元素呈现，测试按语义类判断，不依赖装饰内容。
    expect(wrapper.find('.preview-item.is-added').classes()).toContain('is-added')
    expect(wrapper.find('.preview-item.is-removed').classes()).toContain('is-removed')
    expect(wrapper.text()).not.toContain('本次变更与写入')
    wrapper.unmount()
  })

  it('停用渠道保留恢复入口并隐藏型号，卡片置灰',()=>{
    const wrapper=render({channels:[{id:'a',label:'A渠道',enabled:false}]})
    const group=wrapper.get('.preview-group')
    expect(group.classes()).toContain('off')
    expect(group.text()).toContain('已停用 · 点击恢复')
    expect(group.findAll('.preview-item')).toHaveLength(0)
    wrapper.unmount()
  })

  it('标签表达不了的字段变化折叠为一行，超出部分不逐条铺开',()=>{
    const changes=Array.from({length:11},(_,i)=>({label:'字段'+(i+1),before:'前',after:'后'}))
    const wrapper=render({'other-changes':changes})
    expect(wrapper.text()).toContain('另有 11 项字段变化')
    expect(wrapper.findAll('.preview-other dd')).toHaveLength(8)
    wrapper.unmount()
  })

  it('触屏长按 500ms 只显示说明，抬手不触发启停（§38.3.1）', async()=>{
    vi.useFakeTimers()
    const wrapper=render()
    const title=wrapper.findAll('.preview-group-title')[0]!
    await title.trigger('pointerdown',{pointerType:'touch'})
    expect(wrapper.emitted('toggle-channel')).toBeUndefined()
    vi.advanceTimersByTime(500)
    await wrapper.vm.$nextTick()
    expect(title.classes()).toContain('show-note')
    await title.trigger('pointerup')
    await title.trigger('click')
    expect(wrapper.emitted('toggle-channel')).toBeUndefined()
    await title.trigger('click')
    expect(wrapper.emitted('toggle-channel')?.[0]).toEqual(['a'])
    wrapper.unmount()
    vi.useRealTimers()
  })

  it('长按未满 500ms 或移出后取消，正常点击仍可启停', async()=>{
    vi.useFakeTimers()
    const wrapper=render()
    const title=wrapper.findAll('.preview-group-title')[0]!
    await title.trigger('pointerdown',{pointerType:'touch'})
    vi.advanceTimersByTime(200)
    await title.trigger('pointerleave')
    await title.trigger('pointerup')
    await title.trigger('click')
    expect(wrapper.emitted('toggle-channel')).toHaveLength(1)
    wrapper.unmount()
    vi.useRealTimers()
  })

  it('滚动开始会取消长按计时，滑动不会被当成看说明（§38.3.1）', async()=>{
    vi.useFakeTimers()
    const wrapper=render()
    const title=wrapper.findAll('.preview-group-title')[0]!
    const order=wrapper.get('.selector-preview-order')
    await title.trigger('pointerdown',{pointerType:'touch'})
    // 手指一滑，滚动开始：计时器必须被清掉，说明也不能出现。
    await order.trigger('scroll')
    vi.advanceTimersByTime(800)
    await wrapper.vm.$nextTick()
    expect(title.classes()).not.toContain('show-note')
    // 抬手仍然不该启停（没有触发过长按，正常点击语义保留）。
    await title.trigger('pointerup')
    await title.trigger('click')
    expect(wrapper.emitted('toggle-channel')).toHaveLength(1)
    wrapper.unmount()
    vi.useRealTimers()
  })

  it('长按说明后点击空白处收起，且不影响其它渠道启停（§38.3.1）', async()=>{
    vi.useFakeTimers()
    const wrapper=render()
    const title=wrapper.findAll('.preview-group-title')[0]!
    await title.trigger('pointerdown',{pointerType:'touch'})
    vi.advanceTimersByTime(500)
    await wrapper.vm.$nextTick()
    expect(title.classes()).toContain('show-note')
    await title.trigger('pointerup')
    // 点击空白处（弹窗内容区）应收起说明。
    await wrapper.get('.selector-preview').trigger('click')
    await wrapper.vm.$nextTick()
    expect(title.classes()).not.toContain('show-note')
    wrapper.unmount()
    vi.useRealTimers()
  })

  it('确认按钮始终可点：被拦下的原因走通知，不靠灰按钮堵住',async()=>{
    const wrapper=render({projection:{...projection(),knownActiveModels:128}})
    expect(wrapper.text()).toContain('超出激活预算')
    const confirm=wrapper.findAll('button').find(button=>button.text()==='确认保存')!
    expect(confirm.attributes('disabled')).toBeUndefined()
    await confirm.trigger('click')
    expect(wrapper.emitted('confirm')).toHaveLength(1)
    wrapper.unmount()
  })
})

