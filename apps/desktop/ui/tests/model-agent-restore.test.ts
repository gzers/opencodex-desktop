import { beforeEach,describe,expect,it,vi } from 'vitest'
import { mount,flushPromises } from '@vue/test-utils'
import AgentPanel from '@/features/models/AgentPanel.vue'
import { emptyContent,modelApi,type TemplateDraft,type ModelSnapshot } from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestAgentResult:vi.fn(),stopGeneration:vi.fn()}}))
beforeEach(()=>vi.clearAllMocks())
describe('restore a persisted suggestion',()=>{
 it('starts with no selected fields and leaves the current human buffer untouched',async()=>{
  const content=emptyContent();content.description='原始说明';const draft:TemplateDraft={templateId:'family',baseRevision:'version',label:'人工当前名称',content}
  const revision={id:'version',createdAt:'2026-10-13T00:00:00Z',savedLabel:'人工当前名称',content,digest:'digest',baseRevision:null,changes:[]}
  const snapshot:ModelSnapshot={library:{schemaVersion:1,templates:[{id:'family',label:'人工当前名称',source:'user',defaultRevision:null,revisions:[revision]}]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'base',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'}
  vi.mocked(modelApi.latestAgentResult).mockResolvedValue({runId:'run',createdAt:'2026-10-18T00:00:00Z',expectedLibrary:'base',scene:'from_sources',reviewed:false,targets:[{templateId:'family',label:'人工当前名称',revision,buffer:null}],result:{submissionId:'run',contractVersion:1,report:'上次建议',proposals:[{...draft,content:{...emptyContent(),description:'助手建议'}}]}})
  const wrapper=mount(AgentPanel,{props:{draft,snapshot},global:{stubs:{AgentExecutionField:true}}})
  await wrapper.findAll('button').find(b=>b.text()==='恢复上次建议')!.trigger('click');await flushPromises()
  expect(modelApi.latestAgentResult).toHaveBeenCalledExactlyOnceWith('family');expect(wrapper.emitted('apply')).toBeUndefined();expect(draft.content.description).toBe('原始说明')
  expect(wrapper.get<HTMLInputElement>('.model-agent-diff input').element.checked).toBe(false)
  await wrapper.get('.model-agent-diff input').setValue(true);await wrapper.findAll('button').find(b=>b.text()==='接受所选建议')!.trigger('click')
  expect(wrapper.emitted('apply')![0]![0]).toMatchObject({label:'人工当前名称',content:{description:'助手建议'}});expect(draft.content.description).toBe('原始说明');wrapper.unmount()
 })
})
