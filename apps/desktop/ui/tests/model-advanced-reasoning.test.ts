import {describe,expect,it} from 'vitest'
import {mount} from '@vue/test-utils'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {emptyRules} from '@/features/models/api'
describe('advanced reasoning controls',()=>{
 it('edits a pin independently of default and distinguishes inherited and explicit empty mapping',async()=>{
  const rules=emptyRules();rules.defaultEffort={mode:'custom',value:'low'};rules.reasoningEfforts={mode:'custom',value:['low','high']}
  const wrapper=mount(ModelRuleFields,{props:{modelValue:rules},global:{stubs:{UiSelectField:true}}})
  wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='强制推理档位')!.vm.$emit('update:modelValue','high')
  const changed=wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules;expect(changed.defaultEffort).toEqual({mode:'custom',value:'low'});expect(changed.pinnedEffort).toEqual({mode:'custom',value:'high'})
  await wrapper.findAll('label').find(l=>l.text()==='继承标签映射')!.get('input').setValue(false)
  const empty=wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules;expect(empty.effortMap).toEqual({mode:'custom',value:{}});await wrapper.setProps({modelValue:empty})
  await wrapper.findAll('label').find(l=>l.text()==='high 的上游标签')!.get('input').setValue('__omit__')
  const mapped=wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules;expect(mapped.effortMap).toEqual({mode:'custom',value:{high:'__omit__'}})
  await wrapper.setProps({modelValue:mapped});await wrapper.findAll('label').find(l=>l.text()==='继承标签映射')!.get('input').setValue(true)
  expect((wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules).effortMap).toEqual({mode:'inherit'});wrapper.unmount()
 })
})
