import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import NewChannelDialog from '@/features/models/NewChannelDialog.vue'
import {modelApi,type ModelSnapshot,type NewChannelOutcome} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestNewChannel:vi.fn(),previewNewChannel:vi.fn(),addNewChannel:vi.fn(),newChannelResult:vi.fn(),stage:vi.fn(),discover:vi.fn(),forgetCredential:vi.fn()}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
function render(){return mount(NewChannelDialog,{global:{stubs:{UiDialog:dialog,UiSelectField:true}}})}
function button(wrapper:ReturnType<typeof render>,name:string){return wrapper.findAll('button').find(b=>b.text()===name)!}
async function fill(wrapper:ReturnType<typeof render>,label:string,value:string){await wrapper.findAll('label').find(l=>l.text()===label)!.get('input').setValue(value)}
async function firstStep(wrapper:ReturnType<typeof render>){await fill(wrapper,'渠道名称','独立渠道');await fill(wrapper,'运行渠道标识','custom-new');await fill(wrapper,'服务基地址','https://example.test/v1');await fill(wrapper,'API Key','fixture-new-SECRET');await button(wrapper,'下一步').trigger('click');await flushPromises()}
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.clearAllMocks();vi.mocked(modelApi.latestNewChannel).mockResolvedValue(null);const snapshot={policy:{schemaVersion:1,templateRefs:[],channels:[]},workspace:null,workspaceDigest:'w',policyDigest:'p',libraryDigest:'l',library:{schemaVersion:1,templates:[]}} as ModelSnapshot;useModelsStore().acceptTransfer(snapshot);vi.spyOn(useModelsStore(),'persistDraft').mockResolvedValue()})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))
describe('new channel two-step buffer',()=>{
 it('next validates locally and closing never inserts an empty channel or persists a key',async()=>{
  const wrapper=render();await flushPromises();await firstStep(wrapper)
  expect(wrapper.text()).toContain('2 / 2');expect(modelApi.previewNewChannel).not.toHaveBeenCalled();expect(modelApi.addNewChannel).not.toHaveBeenCalled();expect(modelApi.discover).not.toHaveBeenCalled()
  await button(wrapper,'关闭').trigger('click');expect(wrapper.emitted('close')).toHaveLength(1);expect(useModelsStore().channelDraft!.channels).toHaveLength(0);expect(modelApi.forgetCredential).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('discovery is read-only, starts unselected and a cancelled late result cannot change the buffer',async()=>{
  let finish!:(value:Awaited<ReturnType<typeof modelApi.previewNewChannel>>)=>void
  vi.mocked(modelApi.previewNewChannel).mockReturnValue(new Promise(resolve=>finish=resolve))
  const wrapper=render();await flushPromises();await firstStep(wrapper);await button(wrapper,'获取模型列表').trigger('click')
  expect(modelApi.previewNewChannel).toHaveBeenCalledWith(expect.objectContaining({apiKey:'fixture-new-SECRET',credentialRef:null}));await button(wrapper,'取消等待').trigger('click')
  finish({models:[{id:'late',label:'late',rules:{} as never}],credentialRef:null,elapsedMs:1});await flushPromises();expect(wrapper.text()).toContain('已选 0 / 0');expect(modelApi.forgetCredential).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('manual exact IDs join once and uncertain transport uses only the original result query',async()=>{
  vi.mocked(modelApi.addNewChannel).mockRejectedValue({message:'response lost'})
  const wrapper=render();await flushPromises();await firstStep(wrapper);await fill(wrapper,'原始模型ID','native/manual');await button(wrapper,'添加ID').trigger('click')
  await wrapper.get('input[aria-label="选择新渠道型号 native/manual"]').setValue(true);await button(wrapper,'加入草稿').trigger('click');await flushPromises()
  const request=vi.mocked(modelApi.addNewChannel).mock.calls[0]![0]
  expect(request.channel.selection).toEqual({mode:'whitelist',models:['native/manual']});expect(request.channel.connection.enabled).toBe(false);expect(useModelsStore().channelDraft!.channels).toHaveLength(0)
  const snapshot={...useModelsStore().snapshot!,workspace:{basePolicy:'p',policy:{...useModelsStore().snapshot!.policy,channels:[request.channel]}},workspaceDigest:'joined'}
  vi.mocked(modelApi.newChannelResult).mockResolvedValue({id:request.id,channel:request.channel,phase:'committed',snapshot,error:null})
  await button(wrapper,'核对原加入结果').trigger('click');await flushPromises();expect(modelApi.addNewChannel).toHaveBeenCalledTimes(1);expect(modelApi.newChannelResult).toHaveBeenCalledExactlyOnceWith(request.id);expect(useModelsStore().channelDraft!.channels).toHaveLength(1);expect(wrapper.emitted('added')?.[0]).toEqual([request.channel.id]);wrapper.unmount()
 })
 it('same connection back/next keeps selection but a changed target clears old IDs',async()=>{
  vi.mocked(modelApi.previewNewChannel).mockResolvedValue({models:[{id:'first',label:'first',rules:{} as never}],credentialRef:null,elapsedMs:1})
  const wrapper=render();await flushPromises();await firstStep(wrapper);await button(wrapper,'获取模型列表').trigger('click');await flushPromises();await wrapper.get('input[aria-label="选择新渠道型号 first"]').setValue(true)
  await button(wrapper,'上一步').trigger('click');await button(wrapper,'下一步').trigger('click');expect(wrapper.text()).toContain('已选 1 / 1')
  await button(wrapper,'上一步').trigger('click');await fill(wrapper,'服务基地址','https://other.example.test/v1');await button(wrapper,'下一步').trigger('click');expect(wrapper.text()).toContain('已选 0 / 0');wrapper.unmount()
 })
 it('restores an unfinished operation without collecting or creating another credential',async()=>{
  const channel={id:'prior',label:'原渠道'} as NewChannelOutcome['channel'];vi.mocked(modelApi.latestNewChannel).mockResolvedValue({id:'original',phase:'cleanup_pending',channel,error:{code:'credential_store',message:'清理待核对'},snapshot:null})
  const wrapper=render();await flushPromises();expect(wrapper.text()).toContain('清理待核对');expect(button(wrapper,'核对原加入结果').exists()).toBe(true);expect(wrapper.find('input[type=password]').exists()).toBe(false);expect(modelApi.addNewChannel).not.toHaveBeenCalled();wrapper.unmount()
 })
})
