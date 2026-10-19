import {mount} from '@vue/test-utils'
import {describe,expect,it} from 'vitest'
import UiSwitch from '@/components/ui/UiSwitch.vue'
describe('shared controlled switch',()=>{
 it('has an explicit accessible name and reports intent without replacing the persisted value',async()=>{
  const wrapper=mount(UiSwitch,{props:{label:'启停结果通知',modelValue:false}})
  expect(wrapper.get('button').attributes()).toMatchObject({type:'button',role:'switch','aria-label':'启停结果通知','aria-checked':'false'})
  await wrapper.get('button').trigger('click');expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  expect(wrapper.get('button').attributes('aria-checked')).toBe('false')
  await wrapper.setProps({modelValue:true});expect(wrapper.get('button').attributes('aria-checked')).toBe('true')
 })
 it('uses native disabled semantics during an in-flight update and exposes busy state',async()=>{
  const wrapper=mount(UiSwitch,{props:{label:'自动检查更新',modelValue:true,busy:true}})
  expect(wrapper.get('button').attributes('disabled')).toBeDefined();expect(wrapper.get('button').attributes('aria-busy')).toBe('true')
  await wrapper.get('button').trigger('click');expect(wrapper.emitted('update:modelValue')).toBeUndefined()
 })
})
