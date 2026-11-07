import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h } from 'vue'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import AppModal from '@/components/AppModal.vue'
import {useModelsV2Store as useModelsStore} from '@/features/models/store-v2'
import { useModalStore } from '@/app/modal/store'
import {modelV2Api as modelApi,type LibraryViewV2,type UsageReportV2} from '@/features/models/api-v2'
import {agentLibrary,agentId,agentHash,agentReview,externalRunView} from './helpers/modelAgentV2'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
function snapshot():LibraryViewV2{const value=agentLibrary(),template=value.library.templates[0]!;template.revisions.push({...JSON.parse(JSON.stringify(template.revisions[0]!)),id:agentId(3),digest:agentHash(3)});return value}
function usage():UsageReportV2{return {dtoVersion:2,reference:{templateId:agentId(1),revisionId:agentId(3),digest:agentHash(3)},libraryDigest:agentHash(1),savedPolicyDigest:agentHash(4),draftPolicyDigest:null,isDefault:false,builtinInitial:false,references:[],models:[]}}
const wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
beforeEach(() => {
  pinia = createPinia(); setActivePinia(pinia); installNativeDialogStub(); vi.clearAllMocks()
  ipc.mockReset();ipc.mockImplementation(async(command)=>{if(command==='model_v2_library')return snapshot();if(command==='model_v2_template_usage')return usage();if(command==='model_v2_external_run')return externalRunView();throw new Error('Unexpected v1 command '+command)});vi.spyOn(modelApi,'changeLibrary');vi.spyOn(modelApi,'previewConditions');useModelsStore().view.tab='templates'
})
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount());vi.restoreAllMocks(); document.body.innerHTML = '' })
async function render(external=false) {
  const host = defineComponent({ setup: () => () => [h(ModelsRoute), h(AppModal)] })
  const w = mount(host, { global: { plugins: [pinia], stubs: { AppTopbar: true, ChannelsView: true, AgentExecutionField: true, AgentPanel: true, BatchAgentDialog: true, ExternalTaskDialog: !external, TemplateComparisonDialog: true, TemplateUsageDialog: true, ModelRuleFields: true } }, attachTo: document.body }); wrappers.push(w)
  await flushPromises(); return w
}
function body() { return new DOMWrapper(document.body) }
function button(label: string, within: Pick<DOMWrapper<Element>, 'findAll'> = body()) { return within.findAll('button').find(b => b.text() === label)! }
async function manager() { await render(); await button('版本管理').trigger('click'); await flushPromises(); return body().get('dialog.model-version-manager[open]') }
async function deleting() {
  const parent = await manager(); await parent.get('input[type=checkbox]:not(:disabled)').setValue(true)
  await button('删除所选版本', parent).trigger('click'); await flushPromises(); return parent
}
async function exit(channel: string, selector = 'dialog.model-version-delete[open]') {
  const root = body().get(selector)
  if (channel === 'x') await root.get('header .modal-close').trigger('click')
  if (channel === 'escape') root.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'native-cancel') root.element.dispatchEvent(new Event('cancel', { cancelable: true }))
  if (channel === 'backdrop') await root.trigger('click', { clientX: -10, clientY: -10 })
  if (channel === 'cancel') await button('取消', root).trigger('click')
  await flushPromises()
}

describe('POST-09 remaining version/preview/result consumers', () => {
  it.each(['x', 'escape', 'native-cancel', 'backdrop', 'cancel'])('cancels deletion through %s without closing version management or deleting selected versions', async channel => {
    const parent = await deleting()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    expect(parent.findAll('footer button').map(b => b.text())).toEqual(['删除所选版本'])
    await exit(channel)
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1)
    expect(parent.get<HTMLInputElement>('input[type=checkbox]:not(:disabled)').element.checked).toBe(true)
    expect(modelApi.changeLibrary).not.toHaveBeenCalled(); expect(useModelsStore().snapshot!.library.templates[0]!.revisions).toHaveLength(2)
  })

  it('guards an actual pending delete and keeps the captured target/digests', async () => {
    await deleting(); let finish!:(value:LibraryViewV2)=>void
    vi.mocked(modelApi.changeLibrary).mockReturnValue(new Promise(resolve => { finish = resolve }))
    await button('确认操作', body().get('dialog.model-version-delete[open]')).trigger('click'); await flushPromises()
    for (const channel of ['x', 'escape', 'native-cancel', 'backdrop', 'cancel']) await exit(channel)
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    expect(modelApi.changeLibrary).toHaveBeenCalledExactlyOnceWith({expectedLibrary:agentHash(1),intent:{kind:'delete_revisions',templateId:agentId(1),revisionIds:[agentId(3)]}})
    const remaining = snapshot(); remaining.library.templates[0]!.revisions = remaining.library.templates[0]!.revisions.slice(0, 1)
    finish(remaining); await flushPromises()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1)
    expect(useModelsStore().snapshot!.library.templates[0]!.revisions.map(r => r.id)).toEqual([agentId(2)])
  })

  it('keeps the manager under default confirmation and returns focus to its actual trigger', async () => {
    const parent = await manager(); const trigger = button('设为默认', parent)
    ;(trigger.element as HTMLButtonElement).focus(); await trigger.trigger('click'); await flushPromises()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    await exit('escape', 'dialog.model-version-default[open]')
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1)
    expect(document.activeElement).toBe(trigger.element); expect(modelApi.changeLibrary).not.toHaveBeenCalled()
  })

  it('does not let a dismissed inline match request populate a reopened editor or change its buffer',async()=>{
    await render();const store=useModelsStore();store.start(store.snapshot!.library.templates[0]!);store.view.templateEditor=true;store.setLabel('未保存模版');await flushPromises();let finish!:(value:Awaited<ReturnType<typeof modelApi.previewConditions>>)=>void;vi.spyOn(modelApi,'validateContent').mockResolvedValue();vi.mocked(modelApi.previewConditions).mockReturnValueOnce(new Promise(resolve=>{finish=resolve}));await body().get('input[aria-label="原始模型ID"]').setValue('fixture/model');await button('检查匹配').trigger('click');await flushPromises();await button('返回模版列表').trigger('click');await button('继续编辑').trigger('click');await flushPromises();finish({dtoVersion:2,previewOnly:true,contentDigest:agentHash(7),rows:[{rawModelId:'fixture/model',matchingView:'fixture/model',familyMatched:true,family:{matched:true,reasonCode:'family_matched',reason:'fixture family'},reasonCode:'family_matched',reason:'fixture summary',conditions:[],matchedConditionId:null,rules:agentLibrary().library.templates[0]!.revisions[0]!.content.commonRules,protocol:{kind:'missing'},fieldOrigins:{}}]});await flushPromises();expect(store.preview).toBeNull();expect(store.previewBusy).toBe(false);expect(store.draft!.label).toBe('未保存模版');expect(modelApi.changeLibrary).not.toHaveBeenCalled()
  })

  it('closes an external result observer without changing saved or draft state',async()=>{
    const store=useModelsStore();store.externalCurrentId=agentId(9);store.externalTasks[agentId(9)]={request:{scene:'from_sources',targets:[usage().reference],allowNew:false,expectedLibrary:agentHash(1)},expiresAt:'2026-11-07',activated:true,revoked:false};const original=agentReview();original.review!.proposals[0]!.proposal.target=usage().reference;vi.spyOn(modelApi,'externalRun').mockResolvedValueOnce({...externalRunView(original),targets:[usage().reference]});await render(true);const before=JSON.stringify(store.snapshot);store.externalNotificationTarget={taskId:agentId(9)};await flushPromises();const result=body().findAll('dialog[open]').find(d=>d.text().includes('外部 CLI'))!;await button('读取原任务结果',result).trigger('click');await flushPromises();expect(result.text()).toContain('待审阅建议');await result.get('header .modal-close').trigger('click');await flushPromises();expect(document.querySelector('dialog[open]')).toBeNull();expect(JSON.stringify(store.snapshot)).toBe(before);expect(store.draft).toBeNull();expect(useModalStore().current).toBeNull();expect(ipc.mock.calls.some(c=>c[0]==='model_v2_agent_accept')).toBe(false)
  })
})
