import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {nextTick} from 'vue'
import {reorderSlots} from '@/features/models/ordering'
import {useModelsStore} from '@/features/models/store'
import {emptyRules,modelApi,type ChannelPolicy,type ModelSnapshot} from '@/features/models/api'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModalStore} from '@/app/modal/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{stage:vi.fn(),discover:vi.fn(),forgetCredential:vi.fn()}}))
const channel=():ChannelPolicy=>({id:'channel',label:'测试渠道',templateRefs:[],modelRefs:{},connection:{providerId:'provider',protocol:'openai-chat',nodes:[{id:'node',name:'主站',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['a','c']},modelOrder:['a','absent','b','c'],discovered:['a','b','c'].map(id=>({id,label:id,rules:emptyRules()})),defaults:emptyRules(),overrides:{},familyOverrides:{}})
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.clearAllMocks()})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))
describe('model draft ordering and undo',()=>{
 it('reorders eligible slots without dropping absent or unselected model stubs',()=>{
  expect(reorderSlots(['a','absent','b','c'],['a','c'],'c','a')).toEqual(['c','absent','b','a'])
  expect(reorderSlots(['a','b'],['a','b'],'external','a')).toEqual(['a','b'])
 })
 it('keeps a shared cross-page undo/redo history and resets it after accepted saved state',async()=>{
  const store=useModelsStore();const policy={schemaVersion:1,templateRefs:[],channels:[channel()]}
  const snapshot={policy,workspace:null,policyDigest:'p',workspaceDigest:'w',libraryDigest:'l',library:{schemaVersion:1,templates:[]}} as ModelSnapshot
  store.acceptTransfer(snapshot);await nextTick();expect(store.canUndoChannels).toBe(false)
  store.channelDraft!.channels[0]!.label='changed';await nextTick();store.undoChannels();await nextTick()
  expect(store.channelDraft!.channels[0]!.label).toBe('测试渠道');expect(store.canRedoChannels).toBe(true)
  store.redoChannels();await nextTick();expect(store.channelDraft!.channels[0]!.label).toBe('changed')
  store.undoChannels();await nextTick();store.channelDraft!.channels[0]!.label='another';await nextTick();expect(store.canRedoChannels).toBe(false)
  store.acceptTransfer(snapshot);await nextTick();expect(store.canUndoChannels).toBe(false);expect(store.canRedoChannels).toBe(false)
  expect(modelApi.stage).not.toHaveBeenCalled()
 })
 it('requires confirmation when leaving all and restores the checkbox if cancelled',async()=>{
  const value=channel();value.selection={mode:'all'}
  const wrapper=mount(ChannelEditor,{props:{channel:value,templates:[]},global:{stubs:{ModelRuleFields:true,NodePicker:true,UiSelectField:true}}})
  const checkbox=wrapper.findAll('article.model-variant')[1]!.get<HTMLInputElement>('input[type=checkbox]')
  await checkbox.setValue(false);expect(value.selection).toEqual({mode:'all'});expect(checkbox.element.checked).toBe(true)
  const modal=useModalStore();expect(modal.current?.title).toBe('改为仅公开所选型号')
  await modal.current?.onConfirm?.();await nextTick();expect(value.selection).toEqual({mode:'whitelist',models:['a','c']})
  wrapper.unmount()
 })
 it('blocks reordering searched results and preserves hidden slots when ordering the complete selected view',async()=>{
  const value=channel();const wrapper=mount(ChannelEditor,{props:{channel:value,templates:[]},global:{stubs:{ModelRuleFields:true,NodePicker:true,UiSelectField:true}}})
  await wrapper.get('input[aria-label="搜索渠道内型号"]').setValue('c')
  expect(wrapper.get('button[aria-label="c置顶"]').attributes('disabled')).toBeDefined();expect(value.modelOrder).toEqual(['a','absent','b','c'])
  await wrapper.get('input[aria-label="搜索渠道内型号"]').setValue('')
  await wrapper.findAll('label').find(label=>label.text()==='仅已选')!.get('input').setValue(true)
  await wrapper.get('button[aria-label="c置顶"]').trigger('click');expect(value.modelOrder).toEqual(['c','absent','b','a'])
  expect(value.selection).toEqual({mode:'whitelist',models:['a','c']});await flushPromises();wrapper.unmount()
 })
})
