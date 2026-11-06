import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { getActivePinia } from 'pinia'
import { reactive } from 'vue'
import AgentExecutionField from '@/features/models/AgentExecutionField.vue'
import ChannelModelTable from '@/features/models/ChannelModelTable.vue'
import {createModelRowsState,copy,rowFieldsV2} from '@/features/models/modelRows'
import {modelV2Api as modelApi,type AgentSettingsV2 as AgentSettings,type ExecutionDirectoryV2 as ExecutionDirectory} from '@/features/models/api-v2'
import {workspaceBackend,withWorkspaceModels,workspaceId} from './helpers/modelWorkspaceV2'
import { installNativeDialogStub } from './helpers/nativeDialog'

const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
const saved: AgentSettings = { providerId: 'original', modelId: 'old', reasoningEffort: null, fast: null, channelId: 'old-channel', connectionDigest: 'old-binding' }
const directory:ExecutionDirectory={dtoVersion:2,sourceDigest:'source',rootRevision:'root',libraryDigest:'library',channels:[{id:'new-channel',label:'新渠道',providerId:'new-provider'}],rows:[{channelId:'new-channel',providerId:'new-provider',modelId:'raw/new',label:'raw/new',selected:true,enabled:true,nativeModelId:null,unavailableReason:null,aliases:[],connectionDigest:'new-binding',pickerVisible:true,actualSelector:'new-provider/raw/new',rules:{protocol:{mode:'custom',value:'openai-chat'},contextWindow:{mode:'inherit'},inputModalities:{mode:'inherit'},reasoningEfforts:{mode:'inherit'},defaultEffort:{mode:'inherit'},fast:{mode:'inherit'}},protocol:{kind:'resolved',value:'openai-chat',origin:{kind:'unknown'}}}]}
const wrappers: VueWrapper[] = []
let backend:ReturnType<typeof workspaceBackend>,models:Awaited<ReturnType<ReturnType<typeof workspaceBackend>['load']>>
let pinia:NonNullable<ReturnType<typeof getActivePinia>>
beforeEach(async()=>{installNativeDialogStub();backend=workspaceBackend(ipc,['channel']);backend.current=withWorkspaceModels(backend.current,['raw/a']);models=await backend.load();pinia=getActivePinia()!;vi.spyOn(modelApi,'agentSettings').mockResolvedValue(saved);vi.spyOn(modelApi,'executionDirectory').mockResolvedValue(directory);vi.spyOn(modelApi,'bindAgent');vi.spyOn(modelApi,'fileProject')})
afterEach(()=>{wrappers.splice(0).forEach(w=>w.unmount());vi.restoreAllMocks();document.body.innerHTML=''})
function body() { return new DOMWrapper(document.body) }
function button(label: string, root: Pick<DOMWrapper<Element>, 'findAll'> = body()) { return root.findAll('button').find(b => b.text() === label)! }
async function exit(channel: string, selector: string) {
  const root = body().get(selector)
  if (channel === 'x') await root.get('header .modal-close').trigger('click')
  if (channel === 'escape') root.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'backdrop') await root.trigger('click', { clientX: -10, clientY: -10 })
  if (channel === 'cancel') await button('取消', root).trigger('click')
  await flushPromises()
}
async function agent() {
  const w = mount(AgentExecutionField, { global: { plugins: [pinia] }, attachTo: document.body }); wrappers.push(w)
  await flushPromises(); await button('更换执行模型').trigger('click'); await flushPromises(); return w
}
async function table(){const c=models.channels[0]!,state=reactive(createModelRowsState()),fields=rowFieldsV2(c,'raw/a');state.buffers['raw/a']={...copy(fields),base:copy(fields),error:''};state.buffers['raw/a']!.rules.fast={mode:'custom',value:false};const w=mount(ChannelModelTable,{props:{channel:c,state,effective:models.effectiveRows},global:{plugins:[pinia],stubs:{ModelRuleFields:true}},attachTo:document.body});wrappers.push(w);await flushPromises();return {c,state,w}}

describe('POST-09 remaining model editors', () => {
  it.each(['x', 'escape', 'backdrop', 'cancel'])('discards only the temporary execution choice through %s', async channel => {
    const w = await agent()
    await button('original / old').trigger('click'); await flushPromises()
    await body().get('dialog.model-picker-dialog[open] .model-picker-row').trigger('click')
    await button('确定选择').trigger('click'); await flushPromises()
    expect(document.querySelector('dialog.model-picker-dialog[open]')).toBeNull()
    expect(body().get('dialog.agent-execution-dialog[open]').text()).toContain('new-provider / raw/new')
    await exit(channel, 'dialog.agent-execution-dialog[open]')
    expect(modelApi.bindAgent).not.toHaveBeenCalled(); expect(w.emitted('update:modelValue')?.at(-1)).toEqual([saved])
    await button('更换执行模型').trigger('click'); await flushPromises()
    expect(body().get('dialog.agent-execution-dialog[open]').text()).toContain('original / old')
  })

  it('closes a loading picker first and ignores its late directory without closing settings', async () => {
    await agent(); let finish!: (value: ExecutionDirectory) => void
    vi.mocked(modelApi.executionDirectory).mockReturnValue(new Promise(resolve => { finish = resolve }))
    const trigger = button('original / old'); (trigger.element as HTMLButtonElement).focus()
    await trigger.trigger('click'); await flushPromises()
    await exit('escape', 'dialog.model-picker-dialog[open]')
    expect(document.querySelector('dialog.agent-execution-dialog[open]')).not.toBeNull(); expect(document.activeElement).toBe(trigger.element)
    finish({ ...directory, channels: [{ id: 'late', label: '过期目录', providerId: 'late' }] }); await flushPromises()
    expect(body().text()).not.toContain('过期目录'); expect(modelApi.bindAgent).not.toHaveBeenCalled()
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('keeps an explicitly submitted execution preference save guarded against %s', async channel => {
    await agent(); let finish!: (value: AgentSettings) => void
    vi.mocked(modelApi.bindAgent).mockReturnValue(new Promise(resolve => { finish = resolve }))
    await button('保存共用设置').trigger('click'); await flushPromises(); await exit(channel, 'dialog.agent-execution-dialog[open]')
    expect(document.querySelector('dialog.agent-execution-dialog[open]')).not.toBeNull(); expect(modelApi.bindAgent).toHaveBeenCalledExactlyOnceWith(saved)
    finish(saved); await flushPromises(); expect(document.querySelector('dialog.agent-execution-dialog[open]')).toBeNull()
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('cancels a detail through %s while preserving pre-existing row input and the channel draft', async channel => {
    const { c, state } = await table(); const before = JSON.stringify(c), buffer = JSON.stringify(state.buffers)
    await button('编辑明细').trigger('click'); await flushPromises()
    await body().get('dialog.model-detail-dialog[open] input[maxlength="128"]').setValue('discard-this-detail-alias')
    await exit(channel, 'dialog.model-detail-dialog[open]')
    expect(JSON.stringify(c)).toBe(before); expect(JSON.stringify(state.buffers)).toBe(buffer); expect(modelApi.fileProject).not.toHaveBeenCalled()
  })

  it.each(['detail', 'batch'])('keeps %s validation guarded and applies the draft only once after it passes', async kind => {
    const { c, state, w } = await table()
    if (kind === 'detail') await button('编辑明细').trigger('click')
    else {
      state.buffers = {}; await w.get('input[aria-label="操作选择 raw/a"]').setValue(true); await button('批量停用').trigger('click')
    }
    await flushPromises(); const selector = kind === 'detail' ? 'dialog.model-detail-dialog[open]' : 'dialog.model-batch-dialog[open]'
    let finish!: (value: Awaited<ReturnType<typeof modelApi.fileProject>>) => void
    vi.mocked(modelApi.fileProject).mockReturnValue(new Promise(resolve => { finish = resolve }))
    await button(kind === 'detail' ? '加入草稿' : '整批加入草稿', body().get(selector)).trigger('click'); await flushPromises()
    const before = JSON.stringify(c)
    for (const channel of ['x', 'escape', 'backdrop', 'cancel']) await exit(channel, selector)
    expect(JSON.stringify(c)).toBe(before); expect(modelApi.fileProject).toHaveBeenCalledTimes(1)
    finish(backend.projectResult(vi.mocked(modelApi.fileProject).mock.calls[0]![0])); await flushPromises()
    expect(document.querySelector(selector)).toBeNull()
    if (kind === 'detail') expect(models.explicitRules(workspaceId('channel'),'raw/a').fast).toEqual({ mode: 'custom', value: false })
    else expect(models.channels[0]!.selection).toEqual({ mode: 'none' })
  })
})
