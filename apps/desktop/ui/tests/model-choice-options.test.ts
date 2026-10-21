import {mount} from '@vue/test-utils'
import {describe,it,expect,vi} from 'vitest'
import {nextTick} from 'vue'
import ModelChoiceOptions from '@/features/models/ModelChoiceOptions.vue'
import {emptyRules} from '@/features/models/api'
describe('nested model options escape',()=>{
 it('reverts from the trigger before native dialog cancellation and restores focus without reopening',async()=>{
  const reopen=vi.fn();const wrapper=mount(ModelChoiceOptions,{props:{open:true,label:'推理 / Fast',row:{channelId:'c',providerId:'p',modelId:'m',label:'model',enabled:true,selected:true,discovered:true,templateRef:null,rules:emptyRules(),source:{}},onOpen:reopen},attachTo:document.body})
  await nextTick();const trigger=wrapper.get('button.select-trigger');(trigger.element as HTMLButtonElement).focus();reopen.mockClear()
  const event=new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true});trigger.element.dispatchEvent(event)
  expect(event.defaultPrevented).toBe(true);expect(wrapper.emitted('revert')).toEqual([[]]);expect(reopen).not.toHaveBeenCalled();expect(document.activeElement).toBe(trigger.element)
  wrapper.unmount()
 })
})
