import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {nextTick} from 'vue'
import {reorderSlots} from '@/features/models/ordering'
import {emptyRules,type ChannelPolicy} from '@/features/models/api'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModalStore} from '@/app/modal/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{stage:vi.fn(),discover:vi.fn(),forgetCredential:vi.fn()}}))
const channel=():ChannelPolicy=>({id:'channel',label:'测试渠道',templateRefs:[],modelRefs:{},connection:{providerId:'provider',protocol:'openai-chat',nodes:[{id:'node',name:'主站',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['a','c']},modelOrder:['a','absent','b','c'],discovered:['a','b','c'].map(id=>({id,label:id,rules:emptyRules()})),defaults:emptyRules(),overrides:{},familyOverrides:{}})
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.clearAllMocks()})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))
describe('model draft ordering and explicit selection',()=>{
 it('reorders eligible slots without dropping absent or unselected model stubs',()=>{
  expect(reorderSlots(['a','absent','b','c'],['a','c'],'c','a')).toEqual(['c','absent','b','a'])
  expect(reorderSlots(['a','b'],['a','b'],'external','a')).toEqual(['a','b'])
 })
 it('requires confirmation when leaving all and restores the checkbox if cancelled',async()=>{
  const value=channel();value.selection={mode:'all'}
  const wrapper=mount(ChannelEditor,{props:{channel:value,templates:[]},global:{stubs:{ModelRuleFields:true,NodePicker:true,UiSelectField:true}}})
  const checkbox=wrapper.get('button[aria-label="b启用状态"]')
  await checkbox.trigger('click');expect(value.selection).toEqual({mode:'all'});expect(checkbox.attributes('aria-checked')).toBe('true')
  const modal=useModalStore();expect(modal.current?.title).toBe('改为仅公开所选型号')
  await modal.current?.onConfirm?.();await nextTick();expect(value.selection).toEqual({mode:'whitelist',models:['a','c']})
  wrapper.unmount()
 })
 it('blocks reordering while search or an enabled-only filter is active and retains absent slots',async()=>{
  const value=channel();const wrapper=mount(ChannelEditor,{props:{channel:value,templates:[]},global:{stubs:{ModelRuleFields:true,NodePicker:true,UiSelectField:true}}})
  await wrapper.get('input[aria-label="搜索渠道内型号"]').setValue('c')
  expect(wrapper.get('button[aria-label="c上移"]').attributes('disabled')).toBeDefined();expect(value.modelOrder).toEqual(['a','absent','b','c'])
  await wrapper.get('input[aria-label="搜索渠道内型号"]').setValue('')
  await wrapper.findAll('label').find(label=>label.text()==='仅启用')!.get('input').setValue(true)
  expect(wrapper.get('button[aria-label="c上移"]').attributes('disabled')).toBeDefined();expect(value.modelOrder).toEqual(['a','absent','b','c'])
  await wrapper.findAll('label').find(label=>label.text()==='仅启用')!.get('input').setValue(false);await wrapper.get('button[aria-label="c上移"]').trigger('click');await wrapper.get('button[aria-label="c上移"]').trigger('click');expect(value.modelOrder).toEqual(['c','absent','a','b'])
  expect(value.selection).toEqual({mode:'whitelist',models:['a','c']});await flushPromises();wrapper.unmount()
 })
})
