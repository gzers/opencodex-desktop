import {describe,expect,it} from 'vitest'
import {mount} from '@vue/test-utils'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelChoiceOptions from '@/features/models/ModelChoiceOptions.vue'
import {emptyRules} from '@/features/models/api'
describe('advanced reasoning controls',()=>{
 it('keeps inherited, disabled and explicit Fast wire separate without enabling capability',async()=>{
  const wrapper=mount(ModelRuleFields,{props:{modelValue:emptyRules()},global:{stubs:{UiSelectField:true}}})
  const selector=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='Fast wire模式')!
  selector.vm.$emit('update:modelValue','disabled');await wrapper.vm.$nextTick();let rules=wrapper.emitted('update:modelValue')!.at(-1)![0] as ReturnType<typeof emptyRules>;expect(rules.fastWire).toEqual({mode:'custom',value:null});expect(rules.fast).toEqual({mode:'inherit'});
  await wrapper.setProps({modelValue:rules});selector.vm.$emit('update:modelValue','custom');await wrapper.vm.$nextTick();rules=wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules;expect(rules.fastWire).toEqual({mode:'custom',value:{kind:'service-tier',canonicalToWire:{priority:'priority'},foreignCallerTiers:'drop'}})
  await wrapper.setProps({modelValue:rules});await wrapper.findAll('label').find(l=>l.text()==='priority wire值')!.get('input').setValue('fast');expect((wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules).fastWire).toMatchObject({value:{canonicalToWire:{priority:'fast'}}});
  selector.vm.$emit('update:modelValue','inherit');await wrapper.vm.$nextTick();expect((wrapper.emitted('update:modelValue')!.at(-1)![0] as typeof rules).fastWire).toEqual({mode:'inherit'});wrapper.unmount()
 })
 it('does not expose a reasoning ladder when the resolved rule forbids the parameter',()=>{
  const rules=emptyRules();rules.reasoningEfforts={mode:'custom',value:['low','high']};rules.noReasoning={mode:'custom',value:true}
  const wrapper=mount(ModelChoiceOptions,{props:{row:{channelId:'c',providerId:'p',modelId:'m',label:'m',enabled:true,selected:true,discovered:true,templateRef:null,rules,source:{}},open:true,label:'options'},global:{stubs:{UiSelectMenu:{template:'<div><slot/></div>'},UiDiscreteSlider:true}}})
  expect(wrapper.findComponent({name:'UiDiscreteSlider'}).props('values')).toEqual([]);wrapper.unmount()
 })
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
