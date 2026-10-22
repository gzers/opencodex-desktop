import {mount} from '@vue/test-utils'
import {describe,expect,it} from 'vitest'
import SelectorPreviewDialog from '@/features/models/SelectorPreviewDialog.vue'
import {emptyRules,type DraftProjection} from '@/features/models/api'

const dialog={props:['open'],template:'<div v-if="open"><slot/><slot name="actions"/></div>'}
/** 用原生 select 代替公共下拉：这里只关心“切换来源会发出什么事件”。 */
const select={props:['modelValue','options'],template:'<select :value="modelValue"><option v-for="o in options" :key="o.value" :value="o.value">{{ o.label }}</option></select>'}

function projection(): DraftProjection {
  const row=(channelId:string,modelId:string,label:string)=>(
    {channelId,providerId:channelId,modelId,label,enabled:true,selected:true,discovered:true,templateRef:null,rules:emptyRules(),source:{}}
  )
  return {rows:[row('a','gpt-5.4','gpt-5.4'),row('a','glm-flash','glm-flash'),row('b','claude-opus-5','claude-opus-5')],knownActiveModels:3,exactCatalogCount:3,notices:[]}
}

function render(overrides:Record<string,unknown>={}){
  return mount(SelectorPreviewDialog,{props:{open:true,source:'draft',projection:projection(),channels:[{id:'a',label:'A渠道',enabled:true},{id:'b',label:'B渠道',enabled:true}],...overrides},global:{stubs:{UiDialog:dialog,UiSelectField:select}}})
}

describe('选择器预览是只读投影', () => {
  it('按渠道分组并保持投影顺序，不提供任何勾选', () => {
    const wrapper=render()
    const groups=wrapper.findAll('.selector-preview-group')
    expect(groups.map(g=>g.get('h3').text())).toEqual(['A渠道','B渠道'])
    expect(groups[0]!.findAll('.selector-preview-item span').map(s=>s.text())).toEqual(['gpt-5.4','glm-flash'])
    expect(wrapper.find('input[type=checkbox]').exists()).toBe(false)
    expect(wrapper.findAll('button').map(b=>b.text())).toEqual(['关闭'])
    wrapper.unmount()
  })

  it('展示激活预算与目录占用，超预算时明确提示', () => {
    const wrapper=render()
    expect(wrapper.text()).toContain('模型激活数 3/100')
    expect(wrapper.text()).toContain('目录占用 3')
    wrapper.unmount()
    const over=render({projection:{...projection(),knownActiveModels:128}})
    expect(over.text()).toContain('模型激活数 128/100')
    expect(over.text()).toContain('超出激活预算')
    over.unmount()
  })

  it('停用渠道不发布目录项，因此不出现在预览里', () => {
    const wrapper=render({channels:[{id:'a',label:'A渠道',enabled:true},{id:'b',label:'B渠道',enabled:false}]})
    expect(wrapper.findAll('.selector-preview-group h3').map(h=>h.text())).toEqual(['A渠道'])
    wrapper.unmount()
  })

  it('允许在草稿计划与已保存配置之间切换来源', async () => {
    const wrapper=render()
    const control=wrapper.getComponent(select)
    control.vm.$emit('update:modelValue','saved')
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('update:source')?.[0]).toEqual(['saved'])
    wrapper.unmount()
  })
})
