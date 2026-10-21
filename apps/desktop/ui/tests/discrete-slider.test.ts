import { mount } from '@vue/test-utils'
import { describe,expect,it } from 'vitest'
import UiDiscreteSlider from '@/components/ui/UiDiscreteSlider.vue'
describe('declared discrete slider values',()=>{
 it('uses the actual default without emitting an explicit preference until edited',async()=>{
  const wrapper=mount(UiDiscreteSlider,{props:{values:['none','high','max'],defaultValue:'high',label:'推理档位'}})
  expect(wrapper.get('input').element.value).toBe('1');expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  await wrapper.get('input').setValue('2');expect(wrapper.emitted('update:modelValue')!.at(-1)).toEqual(['max'])
  await wrapper.setProps({modelValue:'max'});await wrapper.get('button[aria-label="恢复默认推理档位"]').trigger('click')
  expect(wrapper.emitted('update:modelValue')!.at(-1)).toEqual([undefined]);wrapper.unmount()
 })
 it('does not turn unknown capability into a default value and steps only to a declared value',async()=>{
  const wrapper=mount(UiDiscreteSlider,{props:{values:['low','high'],label:'推理档位'}})
  expect(wrapper.get('input').attributes('aria-valuetext')).toBe('未知');expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  await wrapper.get('input').trigger('keydown',{key:'ArrowRight'});expect(wrapper.emitted('update:modelValue')!.at(-1)).toEqual(['low'])
  const empty=mount(UiDiscreteSlider,{props:{values:[],label:'无可调档位'}});expect(empty.find('input').exists()).toBe(false);expect(empty.text()).toContain('未提供可选档位');wrapper.unmount();empty.unmount()
 })
})
