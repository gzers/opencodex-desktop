import {mount} from '@vue/test-utils'
import {describe,expect,it} from 'vitest'
import UiIconButton from '@/components/ui/UiIconButton.vue'
import TaskProgressCard from '@/components/patterns/TaskProgressCard.vue'
import FeedbackToast from '@/components/patterns/FeedbackToast.vue'
describe('pure feedback presentation',()=>{
 it('gives icon controls a name, explicit notification size, and native busy disabling',async()=>{
  const wrapper=mount(UiIconButton,{props:{label:'删除通知',size:'notification',busy:true}})
  expect(wrapper.get('button').attributes()).toMatchObject({'aria-label':'删除通知','aria-busy':'true',disabled:''});expect(wrapper.get('button').classes()).toContain('icon-btn-notification')
  await wrapper.get('button').trigger('click');expect(wrapper.emitted('click')).toBeUndefined();wrapper.unmount()
 })
 it('reports collapse without needing a store or writing notification state',async()=>{
  const wrapper=mount(TaskProgressCard,{props:{title:'写入结果待核对',message:'保留原计划',phase:'error',steps:[{label:'校验完成',state:'done'},{label:'读回待核对',state:'pending'}]}})
  expect(wrapper.get('section').classes()).toContain('error');expect(wrapper.text()).toContain('读回待核对');await wrapper.get('button').trigger('click');expect(wrapper.emitted('collapse')).toEqual([[]]);wrapper.unmount()
  const toast=mount(FeedbackToast,{props:{message:'偏好已保存'}});await toast.get('button').trigger('click');expect(toast.emitted('collapse')).toEqual([[]]);expect(toast.text()).toContain('偏好已保存');toast.unmount()
 })
})
