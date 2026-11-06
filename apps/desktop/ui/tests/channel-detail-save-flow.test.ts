import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import ChannelsView from '@/features/models/ChannelsView.vue'
import {useModelsV2Store} from '@/features/models/store-v2'
import {useModelSaveFeedback} from '@/app/feedback/modelSave'
import {useFeedbackStore} from '@/app/feedback/store'
import {workspaceBackend,workspaceId as id,deferredWorkspace} from './helpers/modelWorkspaceV2'
import type {FileSnapshotV2,FileSaveRequestV2} from '@/features/models/api-v2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
const editor={props:['channel','dirty','saveDisabled'],emits:['save'],template:`<section class="detail"><span class="status">{{dirty ? "未保存修改" : "未修改"}}</span><button class="detail-save" :disabled="saveDisabled" @click="$emit('save')">保存</button></section>`}
function render(){return mount(ChannelsView,{global:{stubs:{ChannelEditor:editor,UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}})}
const saveCalls=()=>ipc.mock.calls.filter(c=>c[0]==='model_v2_file_save')
let backend:ReturnType<typeof workspaceBackend>
beforeEach(async()=>{vi.clearAllMocks();backend=workspaceBackend(ipc);const s=await backend.load();s.channelEditorId=id('a')})
describe('current-channel Save navigation and state',()=>{
 it.each(['detail','inactive route'])('invalidates a list review after entering %s, including an already queued confirmation',async target=>{
  const s=useModelsV2Store();s.channelEditorId='';s.queueChannel({kind:'rename',channelId:id('a'),name:'pending'});const w=render();await flushPromises();await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises()
  expect(w.text()).toContain('确认保存');const confirm=w.findAll('button').find(b=>b.text()==='确认保存')!.element
  if(target==='detail')s.channelEditorId=id('a');else await w.setProps({active:false});await flushPromises()
  confirm.dispatchEvent(new MouseEvent('click',{bubbles:true}));await flushPromises()
  expect(w.text()).not.toContain('确认保存');expect(saveCalls()).toEqual([]);expect(s.channelEditorId).toBe(target==='detail'?id('a'):'');expect(s.channels[0]!.label).toBe('pending');w.unmount()
 })
 it('keeps clean A disabled when B has edits, including stale emitted events',async()=>{
  const s=useModelsV2Store();s.queueChannel({kind:'rename',channelId:id('b'),name:'B pending'});const w=render();await flushPromises()
  expect(w.get('.detail-save').attributes('disabled')).toBeDefined();expect(w.get('.status').text()).toBe('未修改')
  w.getComponent(editor).vm.$emit('save');await flushPromises();expect(saveCalls()).toEqual([]);expect(s.channelEditorId).toBe(id('a'));w.unmount()
 })
 it('writes A without a preview and returns to list with B still dirty',async()=>{
  const s=useModelsV2Store();s.queueChannel({kind:'rename',channelId:id('a'),name:'A saved'});s.queueChannel({kind:'rename',channelId:id('b'),name:'B pending'})
  const shown=vi.spyOn(useModelSaveFeedback(),'showSaved');const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises()
  expect(saveCalls()).toHaveLength(1);expect(saveCalls()[0]![1].request).toMatchObject({scopeChannelId:id('a'),channelIntents:[{kind:'rename',channelId:id('a'),name:'A saved'}]});expect(s.channelEditorId).toBe('');expect(s.channelDirty(id('a'))).toBe(false);expect(s.channelDirty(id('b'))).toBe(true);expect(s.file!.savedPolicy.channels[1]!.label).toBe('b');expect(shown).toHaveBeenCalledOnce();expect(w.text()).not.toContain('确认保存');expect(w.get('[data-testid="policy-save"]').attributes('disabled')).toBeUndefined();w.unmount()
 })
 it('keeps the detail and input on file failure and permits retry',async()=>{
  const s=useModelsV2Store();s.queueChannel({kind:'rename',channelId:id('a'),name:'A retained'});ipc.mockImplementation(async(command,args)=>{if(command==='model_v2_file_save')throw {code:'model_source_changed'};return backend.handler(command,args)});const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises();expect(s.channelEditorId).toBe(id('a'));expect(s.channels[0]!.label).toBe('A retained');expect(useFeedbackStore().toast).toContain('未写入');expect(w.get('.detail-save').attributes('disabled')).toBeUndefined()
  ipc.mockImplementation(backend.handler);await w.get('.detail-save').trigger('click');await flushPromises();expect(s.file!.savedPolicy.channels[0]!.label).toBe('A retained');expect(s.channelEditorId).toBe('');w.unmount()
 })
 it.each(['other detail','inactive route','later input'])('does not navigate on a late receipt after %s',async change=>{
  const s=useModelsV2Store();s.queueChannel({kind:'rename',channelId:id('a'),name:'A submitting'});const pending=deferredWorkspace<FileSnapshotV2>();let request!:FileSaveRequestV2
  ipc.mockImplementation(async(command,args)=>{if(command==='model_v2_file_save'){request=args.request;return pending.promise}return backend.handler(command,args)});const w=render();await flushPromises();w.getComponent(editor).vm.$emit('save');await flushPromises();w.getComponent(editor).vm.$emit('save');await flushPromises();expect(saveCalls()).toHaveLength(1)
  if(change==='other detail')s.channelEditorId=id('b');else if(change==='inactive route')await w.setProps({active:false});else s.queueChannel({kind:'rename',channelId:id('a'),name:'later A input'})
  pending.resolve(backend.apply(request));await flushPromises();expect(s.channelEditorId).toBe(change==='other detail'?id('b'):id('a'));expect(s.file!.savedPolicy.channels[0]!.label).toBe('A submitting');if(change==='later input'){expect(s.channels[0]!.label).toBe('later A input');expect(s.channelDirty(id('a'))).toBe(true);expect(s.fileNeedsReconcile).toBe(false)}w.unmount()
 })
 it('consumes only submitted model edits while retaining newer values and unrelated channel edits',async()=>{
  const s=useModelsV2Store();s.queueModel({kind:'set_override',channelId:id('a'),rawModelId:'vendor/raw-model',field:'fast',value:true});s.queueModel({kind:'set_override',channelId:id('b'),rawModelId:'vendor/raw-model',field:'contextWindow',value:64000});const pending=deferredWorkspace<FileSnapshotV2>();let request!:FileSaveRequestV2
  ipc.mockImplementation(async(command,args)=>{if(command==='model_v2_file_save'){request=args.request;return pending.promise}return backend.handler(command,args)});const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises();s.queueModel({kind:'set_override',channelId:id('a'),rawModelId:'vendor/raw-model',field:'fast',value:false});pending.resolve(backend.apply(request));await flushPromises()
  expect(request.edits).toEqual([{kind:'set_override',channelId:id('a'),rawModelId:'vendor/raw-model',field:'fast',value:true}]);expect(s.file!.savedPolicy.channels[0]!.models[0]!.explicitOverrides.fast).toEqual({mode:'custom',value:true});expect(s.explicitRules(id('a'),'vendor/raw-model').fast).toEqual({mode:'custom',value:false});expect(s.modelEdits).toHaveLength(2);expect(s.channelDirty(id('b'))).toBe(true);expect(s.channelEditorId).toBe(id('a'));expect(s.fileNeedsReconcile).toBe(false);w.unmount()
 })
 it('keeps the clean list Save disabled after saving the only edited channel',async()=>{
  const s=useModelsV2Store();s.queueChannel({kind:'rename',channelId:id('a'),name:'A saved'});const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises();expect(w.get('[data-testid="policy-save"]').attributes('disabled')).toBeDefined();await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises();expect(saveCalls()).toHaveLength(1);expect(w.text()).not.toContain('确认保存');w.unmount()
 })
})
