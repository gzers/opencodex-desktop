import {mount} from '@vue/test-utils'
import {describe,it,expect} from 'vitest'
import ObjectScopeTree from '@/features/sync/ObjectScopeTree.vue'
const objects=[{id:'one',category:'templates',categoryLabel:'模版',label:'GPT模版',detail:'saved version'},{id:'two',category:'templates',categoryLabel:'模版',label:'Claude模版'},{id:'three',category:'skills',categoryLabel:'Skills',label:'目标Skill'}]
describe('shallow controlled object scope',()=>{
 it('starts collapsed; expansion never changes selection and category partial state remains honest',async()=>{
  const wrapper=mount(ObjectScopeTree,{props:{objects,selectedIds:['one'],label:'同步对象'}})
  expect(wrapper.find('input[aria-label="选择 GPT模版"]').exists()).toBe(false);expect(wrapper.get('input[aria-label="选择 模版"]').attributes('aria-checked')).toBe('mixed')
  await wrapper.get('button[aria-label="展开 模版"]').trigger('click');expect(wrapper.emitted('update:selectedIds')).toBeUndefined();expect(wrapper.find('input[aria-label="选择 GPT模版"]').exists()).toBe(true)
  await wrapper.get('input[aria-label="选择 模版"]').setValue(true);expect(wrapper.emitted('update:selectedIds')?.[0]).toEqual([['one','two']]);wrapper.unmount()
 })
 it('category selection never includes disabled objects and keeps their prior choices',async()=>{
  const wrapper=mount(ObjectScopeTree,{props:{objects:[...objects,{id:'locked',category:'templates',categoryLabel:'模版',label:'锁定',disabled:true}],selectedIds:['locked'],label:'同步对象'}})
  await wrapper.get('button[aria-label="展开 模版"]').trigger('click')
  await wrapper.get('input[aria-label="选择 模版"]').setValue(true)
  expect(wrapper.emitted('update:selectedIds')?.[0]).toEqual([['locked','one','two']]);wrapper.unmount()
 })
 it('search is owned by the page: query narrows the visible groups and keeps hidden selections counted',async()=>{
  const wrapper=mount(ObjectScopeTree,{props:{objects,selectedIds:['one'],label:'同步对象',query:'Skill'}})
  expect(wrapper.text()).toContain('筛选外仍有 1 个已选对象')
  expect(wrapper.find('input[aria-label="选择 GPT模版"]').exists()).toBe(false)
  expect(wrapper.find('input[aria-label="选择 目标Skill"]').exists()).toBe(true)
  wrapper.unmount()
 })
})
