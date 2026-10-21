import {mount} from '@vue/test-utils'
import {describe,it,expect} from 'vitest'
import UiSegmented from '@/components/ui/UiSegmented.vue'
import InformationRow from '@/components/patterns/InformationRow.vue'
const options=[{value:'high',label:'高'},{value:'mid',label:'中'},{value:'low',label:'低',disabled:true}]
describe('0.2.1 public patterns',()=>{
 it('segmented uses a controlled radiogroup with arrow and home/end keyboard intent',async()=>{
  const w=mount(UiSegmented,{props:{label:'画质',modelValue:'high',options},attachTo:document.body})
  expect(w.get('[role=radiogroup]').attributes('aria-label')).toBe('画质')
  expect(w.findAll('[role=radio]').map(b=>b.attributes('aria-checked'))).toEqual(['true','false','false'])
  await w.findAll('button')[0]!.trigger('keydown',{key:'ArrowRight'})
  expect(w.emitted('update:modelValue')).toEqual([['mid']]);expect(w.findAll('button')[0]!.attributes('aria-checked')).toBe('true')
  expect(document.activeElement).toBe(w.findAll('button')[1]!.element)
  await w.setProps({modelValue:'mid'});await w.findAll('button')[1]!.trigger('keydown',{key:'Home'})
  expect(w.emitted('update:modelValue')!.at(-1)).toEqual(['high']);w.unmount()
 })
 it('disabled segmented has no input intent',async()=>{
  const w=mount(UiSegmented,{props:{label:'画质',modelValue:'high',options,disabled:true}})
  expect(w.findAll('button').every(b=>b.attributes('disabled')!==undefined)).toBe(true)
  await w.findAll('button')[0]!.trigger('keydown',{key:'End'});expect(w.emitted('update:modelValue')).toBeUndefined()
 })
 it('keeps an enabled Tab entry when the selected option becomes disabled or unknown',async()=>{
  const w=mount(UiSegmented,{props:{label:'画质',modelValue:'low',options}})
  expect(w.findAll('button').map(b=>b.attributes('tabindex'))).toEqual(['0','-1','-1'])
  await w.setProps({modelValue:'missing'});expect(w.get('button').attributes('tabindex')).toBe('0')
  await w.setProps({disabled:true});expect(w.findAll('button').every(b=>b.attributes('tabindex')==='-1')).toBe(true)
 })
 it('information rows share content classes, interactive rows use the actual UiButton',async()=>{
  const plain=mount(InformationRow,{props:{label:'连接状态',value:'未连接'}})
  const action=mount(InformationRow,{props:{interactive:true,label:'桌面管理器',value:'0.2.1'}})
  for(const w of [plain,action]){expect(w.get('.info-row-label').text()).toBeTruthy();expect(w.get('.info-row-value').text()).toBeTruthy()}
  expect(action.get('button').classes()).toContain('btn-plain');await action.get('button').trigger('click');expect(action.emitted('activate')).toHaveLength(1)
  expect(plain.find('button').exists()).toBe(false)
 })
})
