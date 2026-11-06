import {afterEach,beforeAll,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import ChannelsView from '@/features/models/ChannelsView.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {modelV2Api,type NodeMeasurementSetV2} from '@/features/models/api-v2'
import {workspaceBackend,workspaceId as id,workspaceHash,copyWorkspace,deferredWorkspace} from './helpers/modelWorkspaceV2'
import {installNativeDialogStub} from './helpers/nativeDialog'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
let wrapper:VueWrapper,backend:ReturnType<typeof workspaceBackend>,models:Awaited<ReturnType<ReturnType<typeof workspaceBackend>['load']>>
const dialog=()=>document.querySelector<HTMLDialogElement>('dialog[open]')!
const action=(label:string)=>dialog().querySelector<HTMLButtonElement>('button[aria-label="'+label+'"]')!
async function open(){await wrapper.get('.channel-node-detect').trigger('click');await flushPromises();const method=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='本次节点清单获取方式')!;if(!method.props('modelValue'))method.vm.$emit('update:modelValue','openai_list');await flushPromises()}
beforeAll(installNativeDialogStub)
beforeEach(async()=>{backend=workspaceBackend(ipc,['a']);models=await backend.load();vi.spyOn(modelV2Api,'measureNodes');vi.spyOn(modelV2Api,'stopMeasurement').mockResolvedValue();wrapper=mount(ChannelsView,{attachTo:document.body,global:{stubs:{ChannelEditor:true,NewChannelDialog:true}}})})
afterEach(()=>{wrapper.unmount();vi.restoreAllMocks();document.body.innerHTML=''})
describe('POST05 列表实际消费者',()=>{
 it('保留选择列与四列，检测位于操作列，选择节点只改草稿',async()=>{
  await flushPromises();expect(wrapper.findAll('thead th').map(n=>n.text())).toEqual(['','渠道','模型','启用','操作']);const cells=wrapper.findAll('tbody td');expect(cells).toHaveLength(5);expect(cells[0]!.find('input[type="checkbox"]').exists()).toBe(true);expect(cells[4]!.find('.channel-node-detect').exists()).toBe(true);await open();action('选择节点 备用').click();await flushPromises();expect(models.channels[0]!.connection.activeNodeId).toBe(id('abackup'));expect(models.file!.savedPolicy.channels[0]!.connection.activeNodeId).toBe(id('anode'));expect(models.channelDirty(id('a'))).toBe(true);expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false);expect(modelV2Api.measureNodes).not.toHaveBeenCalled()
 })
 it('离开渠道页只关闭观察，回到渠道页可继续查看同一检测',async()=>{
  const pending=deferredWorkspace<NodeMeasurementSetV2>();vi.mocked(modelV2Api.measureNodes).mockReturnValueOnce(pending.promise);await open();action('检测节点 备用').click();await flushPromises();const request=vi.mocked(modelV2Api.measureNodes).mock.calls[0]![0];await wrapper.setProps({active:false});await flushPromises();expect(dialog()).toBeNull();expect(modelV2Api.stopMeasurement).not.toHaveBeenCalled();pending.resolve({runId:request.runId,stopped:false,results:[{nodeId:id('abackup'),elapsedMs:19,modelCount:2,error:null}]});await flushPromises();await wrapper.setProps({active:true});await open();expect(dialog().textContent).toContain('19 ms');expect(modelV2Api.measureNodes).toHaveBeenCalledOnce()
 })
 it.each(['搜索隐藏行','进入详情'])('%s不销毁检测，回到列表重开保留结果',async leave=>{
  const pending=deferredWorkspace<NodeMeasurementSetV2>();vi.mocked(modelV2Api.measureNodes).mockReturnValueOnce(pending.promise);await open();action('检测节点 备用').click();await flushPromises();const request=vi.mocked(modelV2Api.measureNodes).mock.calls[0]![0];dialog().querySelector<HTMLButtonElement>('.modal-close')!.click();await flushPromises();if(leave==='搜索隐藏行')models.channelSearch='not-found';else models.channelEditorId=id('a');await flushPromises();expect(modelV2Api.stopMeasurement).not.toHaveBeenCalled();pending.resolve({runId:request.runId,stopped:false,results:[{nodeId:id('abackup'),elapsedMs:23,modelCount:2,error:null}]});await flushPromises();models.channelSearch='';models.channelEditorId='';await flushPromises();await open();expect(dialog().textContent).toContain('23 ms');expect(modelV2Api.measureNodes).toHaveBeenCalledOnce()
 })
 it.each([{providerId:'openai',authMode:'key'},{providerId:'fixture',authMode:'oauth'},{providerId:'fixture',authMode:'forward'}])('保留既有官方/专用认证保护：$providerId/$authMode',async connection=>{
  const next=copyWorkspace(models.file!);Object.assign(next.draft!.policy.channels[0]!.connection,connection);models.file=next;await flushPromises();await open();expect(action('选择节点 备用').disabled).toBe(true);expect(action('检测节点 备用').disabled).toBe(true);dialog().querySelector<HTMLButtonElement>('.modal-close')!.click();await flushPromises();expect(modelV2Api.measureNodes).not.toHaveBeenCalled();expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false)
 })
 it('未保存改名检测仍绑定稳定渠道和可信基准，保存来源变化后旧结果失效',async()=>{
  models.queueChannel({kind:'rename',channelId:id('a'),name:'改名草稿'});vi.mocked(modelV2Api.measureNodes).mockImplementationOnce(async request=>({runId:request.runId,stopped:false,results:[{nodeId:id('abackup'),elapsedMs:17,modelCount:2,error:null}]}));await open();action('检测节点 备用').click();await flushPromises();expect(modelV2Api.measureNodes).toHaveBeenCalledWith({channelId:id('a'),expected:models.file!.draft!.base,nodeIds:[id('abackup')],discoveryKind:'openai_list',runId:expect.any(String)});expect(dialog().textContent).toContain('17 ms');expect(models.channels[0]!.label).toBe('改名草稿');expect(models.channels[0]!.connection.providerId).toBe('provider-a');expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false);const next=copyWorkspace(models.file!);next.sourceDigest=workspaceHash(7);next.draft!.base={...next.draft!.base,sourceDigest:next.sourceDigest,draftGeneration:2};models.file=next;await flushPromises();expect(dialog().textContent).not.toContain('17 ms')
 })
})
