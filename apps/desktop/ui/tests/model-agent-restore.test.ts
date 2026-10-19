import { beforeEach,describe,expect,it,vi } from 'vitest'
import { mount,flushPromises } from '@vue/test-utils'
import AgentPanel from '@/features/models/AgentPanel.vue'
import { emptyContent,modelApi,type TemplateDraft,type ModelSnapshot } from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{agentConversation:vi.fn(),saveAgentConversation:vi.fn(),latestAgentResult:vi.fn(),stopGeneration:vi.fn()}}))
beforeEach(()=>{vi.clearAllMocks();vi.mocked(modelApi.agentConversation).mockResolvedValue(null);vi.mocked(modelApi.saveAgentConversation).mockImplementation(async request=>({templateId:request.templateId,digest:'saved-dialogue',content:request.content}))})
describe('restore a persisted suggestion',()=>{
 it('restores unsent conversation explicitly while keeping the human template form independent',async()=>{
  const draft:TemplateDraft={templateId:'family',baseRevision:'version',label:'当前人工模版',content:emptyContent()}
  const snapshot:ModelSnapshot={library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'library',policyDigest:'policy',workspaceDigest:'workspace',workspace:null}
  const saved={templateId:'family',digest:'previous-conversation',content:{baseRevision:'older-version',input:'尚未发送的纠偏要求',history:[{role:'user' as const,text:'已发送的前一轮'}],sources:[]}}
  vi.mocked(modelApi.agentConversation).mockResolvedValue(saved);vi.mocked(modelApi.latestAgentResult).mockResolvedValue(null)
  const wrapper=mount(AgentPanel,{props:{draft,snapshot},global:{stubs:{AgentExecutionField:true}}});await flushPromises()
  expect(wrapper.get<HTMLTextAreaElement>('textarea[aria-label="制模要求"]').element.value).toBe('')
  await wrapper.findAll('button').find(b=>b.text()==='恢复上次对话与建议')!.trigger('click');await flushPromises()
  expect(wrapper.get<HTMLTextAreaElement>('textarea[aria-label="制模要求"]').element.value).toBe('尚未发送的纠偏要求');expect(wrapper.text()).toContain('已发送的前一轮');expect(wrapper.emitted('apply')).toBeUndefined();expect(draft.label).toBe('当前人工模版')
  wrapper.unmount();await flushPromises();expect(modelApi.saveAgentConversation).toHaveBeenCalledWith(expect.objectContaining({expectedDigest:'previous-conversation',content:expect.objectContaining({input:'尚未发送的纠偏要求'})}))
 })
 it('starts with no selected fields and leaves the current human buffer untouched',async()=>{
  const content=emptyContent();content.description='原始说明';const draft:TemplateDraft={templateId:'family',baseRevision:'version',label:'人工当前名称',content}
  const revision={id:'version',createdAt:'2026-10-13T00:00:00Z',savedLabel:'人工当前名称',content,digest:'digest',baseRevision:null,changes:[]}
  const snapshot:ModelSnapshot={library:{schemaVersion:1,templates:[{id:'family',label:'人工当前名称',source:'user',defaultRevision:null,revisions:[revision]}]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'base',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'}
  vi.mocked(modelApi.latestAgentResult).mockResolvedValue({runId:'run',createdAt:'2026-10-18T00:00:00Z',expectedLibrary:'base',scene:'from_sources',history:[{role:'user',text:'此前纠偏要求：不要推断Fast'},{role:'assistant',text:'此前待审回复'}],sources:[{url:'https://official.example.test/model',retrievedAt:'2026-10-18T00:00:00Z',text:'retrieved fixture',receiptId:'source-receipt',contentDigest:'source-digest'}],reviewed:false,targets:[{templateId:'family',label:'人工当前名称',revision,buffer:null}],result:{submissionId:'run',contractVersion:1,report:'上次建议',proposals:[{...draft,content:{...emptyContent(),description:'助手建议'}}]}})
  const wrapper=mount(AgentPanel,{props:{draft,snapshot},global:{stubs:{AgentExecutionField:true}}})
  await wrapper.findAll('button').find(b=>b.text()==='恢复上次对话与建议')!.trigger('click');await flushPromises()
  expect(modelApi.latestAgentResult).toHaveBeenCalledExactlyOnceWith('family');expect(wrapper.text()).toContain('此前纠偏要求：不要推断Fast');expect(wrapper.text()).toContain('此前待审回复');expect(wrapper.text()).toContain('https://official.example.test/model');expect(wrapper.emitted('apply')).toBeUndefined();expect(draft.content.description).toBe('原始说明')
  expect(wrapper.get<HTMLInputElement>('.model-agent-diff input').element.checked).toBe(false)
  await wrapper.get('.model-agent-diff input').setValue(true);await wrapper.findAll('button').find(b=>b.text()==='接受所选建议')!.trigger('click')
  expect(wrapper.emitted('apply')![0]![0]).toMatchObject({label:'人工当前名称',content:{description:'助手建议'}});expect(draft.content.description).toBe('原始说明');wrapper.unmount()
 })
})
