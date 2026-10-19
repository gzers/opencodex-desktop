import {mount} from '@vue/test-utils'
import {describe,it,expect} from 'vitest'
import ChannelModelScope from '@/features/sync/ChannelModelScope.vue'
import {scopeFor,remoteScope} from '@/features/sync/modelTransfer'
describe('per-channel raw-model scope',()=>{
 it('searches without dropping hidden ids and distinguishes all from explicit none',async()=>{
  const wrapper=mount(ChannelModelScope,{props:{label:'测试渠道',models:[{id:'native/a',label:'A'},{id:'native/b',label:'B'}]}})
  await wrapper.get('input[type=search]').setValue('A');expect(wrapper.emitted('update:selected')).toBeUndefined()
  await wrapper.get('input[aria-label="测试渠道范围内型号 native/a"]').setValue(false);expect(wrapper.emitted('update:selected')?.[0]).toEqual([['native/b']])
  await wrapper.setProps({selected:[]});expect(wrapper.text()).toContain('所选 0 / 2 项');expect(wrapper.findAll('input[type=checkbox]').every(i=>!(i.element as HTMLInputElement).checked)).toBe(true)
  await wrapper.findAll('input[type=checkbox]')[0]!.setValue(true);expect(wrapper.emitted('update:selected')?.at(-1)).toEqual([undefined]);wrapper.unmount()
 })
 it('carries only selected channel masks into file and remote device scope',()=>{
  const snapshot={library:{templates:[]}} as never
  const scope=scopeFor(snapshot,[],['channel'],false,{}, {channel:['native/a'],other:['unselected']})
  expect(scope.channelModels).toEqual({channel:['native/a']});expect(remoteScope(scope).channelModels).toEqual(scope.channelModels)
  expect(scopeFor(snapshot,[],[],false,{}).channelModels).toBeUndefined()
 })
})
