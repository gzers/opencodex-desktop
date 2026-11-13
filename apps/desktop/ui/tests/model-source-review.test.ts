import { mount, flushPromises } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import FileSourceReview from '@/features/models/FileSourceReview.vue'
import { copyWorkspace, workspaceBackend, workspaceHash, workspaceId } from './helpers/modelWorkspaceV2'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

function render() {
  const view = mount(FileSourceReview, {
    global: { stubs: { UiDialog: { props: ['open'], emits: ['close'], template: '<section v-if="open"><slot/><slot name="actions" :close="() => $emit(&quot;close&quot;)" /></section>' } } },
  })
  const button = (label: string) => view.findAll('button').find(b => b.text() === label)
  return { view, button }
}

describe('explicit source review', () => {
  it('holds local edits through a stale save and only writes config after the next explicit save', async () => {
    const backend = workspaceBackend(invoke)
    const models = await backend.load()
    const id = workspaceId('a')
    const old = copyWorkspace(models.file!.draft!.base)
    models.queueChannel({ kind: 'set_enabled', channelId: id, enabled: false })
    models.queueModel({ kind: 'set_override', channelId: id, rawModelId: 'vendor/raw-model', field: 'fast', value: true })
    const external = copyWorkspace(backend.current)
    external.sourceDigest = workspaceHash(7)
    backend.current = external
    invoke.mockImplementation(async (command, args) => {
      if (command === 'model_v2_file_save' && args.request.expected.sourceDigest === old.sourceDigest) throw { code: 'model_save_stale' }
      if (command === 'model_v2_file_reconcile_preview') return { sourceDigest: external.sourceDigest, reviewDigest:workspaceHash(8), changedChannels: ['b'], conflicts: [] }
      if (command === 'model_v2_file_reconcile_apply') {
        external.draft!.base = { ...old, sourceDigest: external.sourceDigest, draftGeneration: old.draftGeneration + 1 }
        backend.current = external
        return copyWorkspace(external)
      }
      return backend.handler(command, args)
    })
    await models.saveChannels(id)
    expect(models.fileSourceStale).toBe(true)
    const retainedIntents = copyWorkspace(models.channelIntents)
    models.setKeyBuffer(id, { action: 'replace', value: 'fixture-not-in-review' })
    const { view, button } = render()
    await button('核对配置来源')!.trigger('click'); await flushPromises()
    expect(view.text()).toContain('磁盘上变化的渠道：b')
    expect(invoke.mock.calls.at(-1)).toEqual(['model_v2_file_reconcile_preview', { request: { expected: old, localChannelIds: [id], modelOnlyChannelIds: [] } }])
    expect(invoke.mock.calls.filter(([command]) => command === 'model_v2_file_save')).toHaveLength(1)
    await button('确认重核')!.trigger('click'); await flushPromises()
    expect(models.channelIntents).toEqual(retainedIntents)
    expect(models.modelEdits).toHaveLength(1)
    expect(models.keyBuffer(id)?.value).toBe('fixture-not-in-review')
    expect(models.fileSourceStale).toBe(false)
    expect(invoke.mock.calls.filter(([command]) => command === 'model_v2_file_save')).toHaveLength(1)
    expect(JSON.stringify(invoke.mock.calls.filter(([command]) => String(command).includes('reconcile')))).not.toContain('fixture-not-in-review')
    await models.saveChannels(id)
    expect(backend.keys.get(id)).toBe('fixture-not-in-review')
    expect(backend.current.savedPolicy.channels[0]!.connection.enabled).toBe(false)
    expect(backend.current.savedPolicy.channels[0]!.models[0]!.explicitOverrides.fast).toEqual({mode:'custom',value:true})
    view.unmount()
  })

  it('shows the affected channel and never applies an overlapping review', async () => {
    const backend = workspaceBackend(invoke)
    const models = await backend.load()
    models.fileSourceStale = true
    models.queueChannel({kind:'rename',channelId:workspaceId('a'),name:'Keep draft'})
    invoke.mockImplementation(async (command, args) => command === 'model_v2_file_reconcile_preview'
      ? {sourceDigest:workspaceHash(7),reviewDigest:workspaceHash(8), changedChannels:['a'],conflicts:[{label:'a',reason:'overlap'}]}
      : backend.handler(command, args))
    const { view, button } = render()
    await button('核对配置来源')!.trigger('click'); await flushPromises()
    expect(view.text()).toContain('a：磁盘和草稿都修改了此渠道')
    expect(button('确认重核')).toBeUndefined()
    await button('取消')!.trigger('click')
    expect(models.channels[0]!.label).toBe('Keep draft')
    expect(invoke.mock.calls.some(([command]) => command === 'model_v2_file_reconcile_apply')).toBe(false)
    view.unmount()
  })
})

it('classifies a model-only draft without sending its values and rejects new edits during confirmation', async () => {
  const backend = workspaceBackend(invoke)
  const models = await backend.load()
  const id = workspaceId('a')
  models.fileSourceStale = true
  models.queueModel({kind:'set_override',channelId:id,rawModelId:'vendor/raw-model',field:'contextWindow',value:8192})
  invoke.mockImplementation(async (command, args) => command === 'model_v2_file_reconcile_preview'
    ? {sourceDigest:workspaceHash(7),reviewDigest:workspaceHash(8),changedChannels:['a'],conflicts:[]}
    : backend.handler(command, args))
  const review = await models.reviewFileSource()
  expect(review.request.modelOnlyChannelIds).toEqual([id])
  expect(JSON.stringify(review.request)).not.toContain('contextWindow')
  models.queueChannel({kind:'set_enabled',channelId:id,enabled:false})
  await expect(models.applyFileSource(review)).rejects.toThrow('过期')
  expect(invoke.mock.calls.some(([command]) => command === 'model_v2_file_reconcile_apply')).toBe(false)
  expect(models.modelEdits).toHaveLength(1)
  expect(models.channelIntents).toHaveLength(1)
})
