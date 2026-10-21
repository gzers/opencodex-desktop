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
 it('does not select disabled objects through category or header and keeps their prior choices',async()=>{
  const wrapper=mount(ObjectScopeTree,{props:{objects:[...objects,{id:'locked',category:'templates',categoryLabel:'模版',label:'锁定',disabled:true}],selectedIds:['locked'],label:'同步对象'}})
  await wrapper.get('.scope-tree-toolbar input[type=checkbox]').setValue(true);expect(wrapper.emitted('update:selectedIds')?.[0]).toEqual([['locked','one','two','three']]);wrapper.unmount()
 })
 it('search keeps hidden selections and selects only the current matching objects',async()=>{
  const wrapper=mount(ObjectScopeTree,{props:{objects,selectedIds:['one'],label:'同步对象'}})
  await wrapper.get('input[type=search]').setValue('Skill');expect(wrapper.text()).toContain('筛选外仍有 1 个已选对象')
  await wrapper.get('.scope-tree-toolbar input[type=checkbox]').setValue(true);expect(wrapper.emitted('update:selectedIds')?.[0]).toEqual([['one','three']]);wrapper.unmount()
 })
})
