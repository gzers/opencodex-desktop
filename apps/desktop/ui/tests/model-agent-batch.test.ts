import { beforeEach,describe,expect,it,vi } from 'vitest'
import { mount,flushPromises } from '@vue/test-utils'
import { createPinia,setActivePinia } from 'pinia'
import { defineComponent,onMounted } from 'vue'
import BatchAgentDialog from '@/features/models/BatchAgentDialog.vue'
import { emptyContent,modelApi,type ModelSnapshot,type Submission } from '@/features/models/api'
import { useModelsStore } from '@/features/models/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{generate:vi.fn(),acceptAgentReview:vi.fn(),stopGeneration:vi.fn(),latestAgentResult:vi.fn()}}))
const settings=defineComponent({emits:['update:modelValue'],setup(_,{emit}){onMounted(()=>emit('update:modelValue',{providerId:'fixture',modelId:'fixture-model',reasoningEffort:null,fast:null}));return()=>null}})
const dialog={template:'<div><slot/><slot name="actions"/></div>'}
function snapshot():ModelSnapshot{return {library:{schemaVersion:1,templates:['one','two'].map(id=>({id,label:id,source:'user' as const,defaultRevision:id+'-v',revisions:[{id:id+'-v',createdAt:'2026-10-13T00:00:00Z',savedLabel:id,content:emptyContent(),digest:id,baseRevision:null,changes:[]}]}))},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'base',policyDigest:'policy',workspaceDigest:'workspace',workspace:null}}
function result(base:ModelSnapshot):Submission{return {submissionId:'fixture-run',contractVersion:1,report:'两个模版有建议',proposals:base.library.templates.map(t=>({templateId:t.id,baseRevision:t.revisions[0]!.id,label:t.label,content:{...emptyContent(),description:'建议说明'}}))}}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks()})
function render(base:ModelSnapshot,selected:string[]=[]){useModelsStore().snapshot=base;return mount(BatchAgentDialog,{props:{open:true,snapshot:base,selected,versions:{}},global:{stubs:{UiDialog:dialog,AgentExecutionField:settings}}})}
describe('internal batch suggestions require review',()=>{
 it('freezes saved targets and does not save or adopt anything before confirmation',async()=>{
  const base=snapshot();const store=useModelsStore();const wrapper=render(base);store.edit();store.draft!.label='保留未保存人工输入'
  vi.mocked(modelApi.generate).mockResolvedValue(result(base));vi.mocked(modelApi.acceptAgentReview).mockResolvedValue({snapshot:{...base,libraryDigest:'saved'},outcomes:[{templateId:'one',revisionId:'new-one',changed:true},{templateId:'two',revisionId:'new-two',changed:true}]})
  await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='检查更新')!.trigger('click');await flushPromises()
  expect(modelApi.generate).toHaveBeenCalledWith(expect.any(String),expect.objectContaining({targets:[{templateId:'one',revisionId:'one-v'},{templateId:'two',revisionId:'two-v'}],buffers:[],allowNew:false,expectedLibrary:'base'}),[])
  expect(modelApi.acceptAgentReview).not.toHaveBeenCalled();expect(store.snapshot!.policy).toEqual(base.policy)
  await wrapper.findAll('button').find(b=>b.text()==='另存所选建议版本')!.trigger('click');await flushPromises()
  expect(modelApi.acceptAgentReview).toHaveBeenCalledWith(expect.objectContaining({expectedLibrary:'base',proposals:expect.arrayContaining([expect.objectContaining({templateId:'one'}),expect.objectContaining({templateId:'two'})])}));expect(store.draft!.label).toBe('保留未保存人工输入');wrapper.unmount()
 })
 it('retries the identical reviewed run after an uncertain save instead of generating a second operation',async()=>{
  const base=snapshot();const wrapper=render(base,['one']);const generated=result(base);generated.proposals=generated.proposals.slice(0,1);vi.mocked(modelApi.generate).mockResolvedValue(generated)
  vi.mocked(modelApi.acceptAgentReview).mockRejectedValueOnce({message:'结果待核对'}).mockResolvedValue({snapshot:base,outcomes:[{templateId:'one',revisionId:'new',changed:true}]})
  await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='检查更新')!.trigger('click');await flushPromises()
  expect(vi.mocked(modelApi.generate).mock.calls[0]![1].targets).toEqual([{templateId:'one',revisionId:'one-v'}])
  await wrapper.findAll('button').find(b=>b.text()==='另存所选建议版本')!.trigger('click');await flushPromises()
  expect(wrapper.text()).toContain('结果待核对');await wrapper.findAll('button').find(b=>b.text()==='核对原保存结果')!.trigger('click');await flushPromises()
  expect(modelApi.generate).toHaveBeenCalledTimes(1);expect(vi.mocked(modelApi.acceptAgentReview).mock.calls[0]).toEqual(vi.mocked(modelApi.acceptAgentReview).mock.calls[1]);wrapper.unmount()
 })
 it('keeps suggestions visible but refuses adoption when the local saved baseline advances',async()=>{
  const base=snapshot();const wrapper=render(base);vi.mocked(modelApi.generate).mockResolvedValue(result(base));await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='检查更新')!.trigger('click');await flushPromises()
  await wrapper.setProps({snapshot:{...base,libraryDigest:'newer'}})
  expect(wrapper.text()).toContain('当前建议保留');expect(wrapper.findAll('button').find(b=>b.text()==='另存所选建议版本')!.attributes('disabled')).toBeDefined();expect(modelApi.acceptAgentReview).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('recovers the persisted saved target snapshots without making a model request or applying them',async()=>{
  const base=snapshot();vi.mocked(modelApi.latestAgentResult).mockResolvedValue({runId:'recovered',createdAt:'2026-10-18T00:00:00Z',expectedLibrary:'base',targets:base.library.templates.map(t=>({templateId:t.id,label:t.label,revision:t.revisions[0]!,buffer:null})),scene:'update_models',result:result(base),reviewed:false})
  const wrapper=render(base);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='恢复上次建议')!.trigger('click');await flushPromises()
  expect(modelApi.latestAgentResult).toHaveBeenCalledExactlyOnceWith(null);expect(wrapper.text()).toContain('建议说明');expect(modelApi.generate).not.toHaveBeenCalled();expect(modelApi.acceptAgentReview).not.toHaveBeenCalled();wrapper.unmount()
 })
})
