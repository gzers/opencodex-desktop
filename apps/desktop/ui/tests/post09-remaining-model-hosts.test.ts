import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h } from 'vue'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import AppModal from '@/components/AppModal.vue'
import { useModelsStore } from '@/features/models/store'
import { useModalStore } from '@/app/modal/store'
import { emptyContent, modelApi, type ModelSnapshot } from '@/features/models/api'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@/features/models/api', async original => ({ ...await original<typeof import('@/features/models/api')>(), modelApi: { load: vi.fn(), delete: vi.fn(), setDefault: vi.fn(), preview: vi.fn(), externalResult: vi.fn() } }))
function snapshot(): ModelSnapshot {
  return { library: { schemaVersion: 1, templates: [{ id: 'family', label: '模版族', source: 'user', defaultRevision: 'old', revisions: ['old', 'new'].map(id => ({ id, createdAt: '2026-11-04T00:00:00Z', savedLabel: id, digest: id, baseRevision: null, changes: [], content: emptyContent() })) }] }, policy: { schemaVersion: 1, templateRefs: [], channels: [] }, libraryDigest: 'library', policyDigest: 'policy', workspaceDigest: 'workspace', workspace: null }
}
const wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
beforeEach(() => {
  pinia = createPinia(); setActivePinia(pinia); installNativeDialogStub(); vi.clearAllMocks()
  vi.mocked(modelApi.load).mockResolvedValue(snapshot()); useModelsStore().view.tab = 'templates'
})
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); document.body.innerHTML = '' })
async function render() {
  const host = defineComponent({ setup: () => () => [h(ModelsRoute), h(AppModal)] })
  const w = mount(host, { global: { plugins: [pinia], stubs: { AppTopbar: true, ChannelsView: true, AgentExecutionField: true, AgentPanel: true, BatchAgentDialog: true, ExternalTaskDialog: true, TemplateComparisonDialog: true, TemplateUsageDialog: true, ModelRuleFields: true } }, attachTo: document.body }); wrappers.push(w)
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
    expect(modelApi.delete).not.toHaveBeenCalled(); expect(useModelsStore().snapshot!.library.templates[0]!.revisions).toHaveLength(2)
  })

  it('guards an actual pending delete and keeps the captured target/digests', async () => {
    await deleting(); let finish!: (value: ModelSnapshot) => void
    vi.mocked(modelApi.delete).mockReturnValue(new Promise(resolve => { finish = resolve }))
    await button('确认删除', body().get('dialog.model-version-delete[open]')).trigger('click'); await flushPromises()
    for (const channel of ['x', 'escape', 'native-cancel', 'backdrop', 'cancel']) await exit(channel)
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    expect(modelApi.delete).toHaveBeenCalledExactlyOnceWith('library', 'policy', 'workspace', 'family', ['new'])
    const remaining = snapshot(); remaining.library.templates[0]!.revisions = remaining.library.templates[0]!.revisions.slice(0, 1)
    finish(remaining); await flushPromises()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1)
    expect(useModelsStore().snapshot!.library.templates[0]!.revisions.map(r => r.id)).toEqual(['old'])
  })

  it('keeps the manager under default confirmation and returns focus to its actual trigger', async () => {
    const parent = await manager(); const trigger = button('设为默认', parent)
    ;(trigger.element as HTMLButtonElement).focus(); await trigger.trigger('click'); await flushPromises()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    await exit('escape', 'dialog.app-modal-dialog[open]')
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1)
    expect(document.activeElement).toBe(trigger.element); expect(modelApi.setDefault).not.toHaveBeenCalled()
  })

  it('does not let a dismissed match request populate a reopened preview or change the template buffer', async () => {
    await render(); const store = useModelsStore(); store.edit(store.snapshot!.library.templates[0]); store.view.templateEditor = true
    store.draft!.label = '未保存模版'; await flushPromises(); await button('匹配预览').trigger('click'); await flushPromises()
    let finish!: (value: Awaited<ReturnType<typeof modelApi.preview>>) => void
    vi.mocked(modelApi.preview).mockReturnValue(new Promise(resolve => { finish = resolve }))
    await body().get('dialog.model-match-preview[open] input[aria-label="原始模型ID"]').setValue('fixture/model')
    await button('检查匹配').trigger('click'); await flushPromises(); await exit('escape', 'dialog.model-match-preview[open]')
    await button('匹配预览').trigger('click'); await flushPromises()
    finish({ matched: true, reason: '过期响应', variantId: null } as never); await flushPromises()
    expect(body().get('dialog.model-match-preview[open]').text()).not.toContain('过期响应')
    expect(store.draft!.label).toBe('未保存模版'); expect(modelApi.delete).not.toHaveBeenCalled()
  })

  it('closes an external result observer without changing saved or draft state', async () => {
    await render(); const store = useModelsStore(); const before = JSON.stringify(store.snapshot)
    vi.mocked(modelApi.externalResult).mockResolvedValue({ task: { targets: [] }, receipt: null, outcomes: [] } as never)
    store.notificationTarget = { taskId: 'fixture-task', submissionId: 'fixture-submit' }; await flushPromises()
    const result = body().get('dialog.model-task-result[open]'); expect(result.find('footer').exists()).toBe(false)
    await exit('x', 'dialog.model-task-result[open]')
    expect(JSON.stringify(store.snapshot)).toBe(before); expect(useModalStore().current).toBeNull()
  })
})
