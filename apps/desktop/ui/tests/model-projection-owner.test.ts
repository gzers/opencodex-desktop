import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import ChannelsView from '@/features/models/ChannelsView.vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {modelV2Api,type DraftProjectionV2} from '@/features/models/api-v2'
import {workspaceBackend,workspaceId as id,copyWorkspace,deferredWorkspace} from './helpers/modelWorkspaceV2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
let backend:ReturnType<typeof workspaceBackend>
beforeEach(()=>{backend=workspaceBackend(ipc,['first','second'])})
describe('channel projection owner',()=>{
 it('ignores the previous channel validation after switching targets and never mutates the server draft or queues',async()=>{
  const store=await backend.load(),frozen=copyWorkspace(store.file),pending=deferredWorkspace<DraftProjectionV2>(),spy=vi.spyOn(modelV2Api,'fileProject').mockReturnValueOnce(pending.promise)
  const w=mount(ChannelsView,{global:{stubs:{NewChannelDialog:true,ModelRuleFields:true,ModelPicker:true,UiSelectField:true,UiDialog:true}}})
  try{await flushPromises();await w.findAll('button').find(b=>b.text()==='编辑')!.trigger('click');await w.get('input[aria-label="vendor/raw-model路由别名"]').setValue('first-pending');await w.get('input[aria-label="vendor/raw-model路由别名"]').trigger('blur');await flushPromises();expect(spy).toHaveBeenCalledOnce();const response=backend.projectResult(spy.mock.calls[0]![0]);await w.get('button[aria-label="返回渠道列表"]').trigger('click');await w.findAll('button').filter(b=>b.text()==='编辑')[1]!.trigger('click');await flushPromises();pending.resolve(response);await flushPromises();expect(w.getComponent(ChannelEditor).props('channel').id).toBe(id('second'));expect(w.get<HTMLInputElement>('input[aria-label="vendor/raw-model路由别名"]').element.value).toBe('');expect(store.file).toEqual(frozen);expect(store.channelIntents).toEqual([]);expect(store.modelEdits).toEqual([]);expect(store.effectiveRows).toEqual(frozen!.effective);expect(store.modelRows[id('first')]!.buffers['vendor/raw-model']!.naming.alias).toBe('first-pending');expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false)}finally{w.unmount();spy.mockRestore()}
 })
})
