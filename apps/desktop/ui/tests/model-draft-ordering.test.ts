import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {defineComponent,h,nextTick} from 'vue'
import {reorderSlots} from '@/features/models/ordering'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModalStore} from '@/app/modal/store'
import {workspaceBackend,withWorkspaceModels,copyWorkspace} from './helpers/modelWorkspaceV2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
let backend:ReturnType<typeof workspaceBackend>,store:Awaited<ReturnType<ReturnType<typeof workspaceBackend>['load']>>,wrappers:VueWrapper[]=[]
beforeEach(async()=>{backend=workspaceBackend(ipc,['channel']);const fixture=withWorkspaceModels(backend.current,['a','b','c']);fixture.savedPolicy.channels[0]!.modelOrder=['a','absent','b','c'];fixture.savedPolicy.channels[0]!.selection={mode:'whitelist',models:['a','c']};backend.current=fixture;store=await backend.load()})
afterEach(()=>wrappers.splice(0).forEach(w=>w.unmount()))
function render(){const w=mount(defineComponent({setup:()=>()=>h(ChannelEditor,{channel:store.channels[0]!,effective:store.effectiveRows,onIntent:store.queueChannel})}),{global:{stubs:{ModelRuleFields:true,NodePicker:true,UiSelectField:true}}});wrappers.push(w);return w}
describe('model draft ordering and explicit selection',()=>{
 it('reorders eligible slots without dropping absent or unselected model stubs',()=>{expect(reorderSlots(['a','absent','b','c'],['a','c'],'c','a')).toEqual(['c','absent','b','a']);expect(reorderSlots(['a','b'],['a','b'],'external','a')).toEqual(['a','b'])})
 it('requires confirmation when leaving all and restores the checkbox if cancelled',async()=>{
  const fixture=copyWorkspace(store.file!);fixture.draft!.policy.channels[0]!.selection={mode:'all'};store.file=fixture;const w=render(),checkbox=w.get<HTMLInputElement>('input[aria-label="公开模型 b"]');await checkbox.setValue(false);expect(store.channels[0]!.selection).toEqual({mode:'all'});expect(checkbox.element.checked).toBe(true);const modal=useModalStore();expect(modal.current?.title).toBe('改为仅公开所选型号');modal.resolve('cancel');await nextTick();expect(checkbox.element.checked).toBe(true);await checkbox.setValue(false);modal.resolve('confirm');await nextTick();expect(store.channels[0]!.selection).toEqual({mode:'whitelist',models:['a','c']});expect(checkbox.element.checked).toBe(false)
 })
 it('blocks reordering while search or a selected-only filter is active and retains absent slots',async()=>{
  const w=render();await w.get('input[aria-label="渠道模型搜索"]').setValue('c');expect(w.get('button[aria-label="c上移"]').attributes('disabled')).toBeDefined();expect(store.channels[0]!.modelOrder).toEqual(['a','absent','b','c']);await w.get('input[aria-label="渠道模型搜索"]').setValue('');await w.findAll('label').find(l=>l.text()==='仅已选')!.get('input').setValue(true);expect(w.get('button[aria-label="c上移"]').attributes('disabled')).toBeDefined();expect(store.channels[0]!.modelOrder).toEqual(['a','absent','b','c']);await w.findAll('label').find(l=>l.text()==='仅已选')!.get('input').setValue(false);await w.get('button[aria-label="c上移"]').trigger('click');await w.get('button[aria-label="c上移"]').trigger('click');expect(store.channels[0]!.modelOrder).toEqual(['c','absent','a','b']);expect(store.channels[0]!.selection).toEqual({mode:'whitelist',models:['a','c']});expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false);await flushPromises()
 })
})
