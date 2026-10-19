import {mount} from '@vue/test-utils'
import {describe,it,expect} from 'vitest'
import NodePicker from '@/features/models/NodePicker.vue'
const nodes=[{id:'one',name:'主节点',url:'https://one.example.test'},{id:'two',name:'备用节点',url:'https://two.example.test'}]
describe('pure node selection view',()=>{
 it('emits manual selection and bounded edits without mutating inputs or switching after a measurement',async()=>{
  const wrapper=mount(NodePicker,{props:{nodes:structuredClone(nodes),activeId:'one',results:{two:{nodeId:'two',elapsedMs:20,modelCount:5,error:null}}}})
  expect(wrapper.text()).toContain('20 ms · 5 个模型');expect(wrapper.emitted('update:activeId')).toBeUndefined()
  await wrapper.get('input[aria-label="节点2名称"]').setValue('新名称')
  expect(wrapper.emitted('update:nodes')?.[0]?.[0]).toEqual([nodes[0],{...nodes[1],name:'新名称'}]);expect(nodes[1]!.name).toBe('备用节点')
  await wrapper.get('button[aria-label="设节点2为当前"]').trigger('click');expect(wrapper.emitted('update:activeId')).toEqual([['two']])
  expect(wrapper.get('button[aria-label="删除节点1"]').attributes('disabled')).toBeDefined()
  await wrapper.get('button[aria-label="检测节点2"]').trigger('click');expect(wrapper.emitted('measure')).toEqual([[['two']]])
 })
 it('keeps completed results on cancel and disables unavailable actions while a run is active',async()=>{
  const wrapper=mount(NodePicker,{props:{nodes,activeId:'one',measuring:true,pending:['two'],stopped:true,results:{one:{nodeId:'one',elapsedMs:12,modelCount:8,error:null}}}})
  expect(wrapper.text()).toContain('已完成结果保留');expect(wrapper.text()).toContain('12 ms')
  expect(wrapper.get('button[aria-label="检测节点2"]').attributes('disabled')).toBeDefined()
  await wrapper.findAll('button').find(b=>b.text()==='取消检测')!.trigger('click');expect(wrapper.emitted('stop')).toEqual([[]])
 })
})
