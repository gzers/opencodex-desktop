import {createAgentViewContext} from '@/features/models/agentView'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, modelApi, versionLabel, type ModelSnapshot, type Revision } from '@/features/models/api'

vi.mock('@/features/models/api', async importOriginal => {
  const actual = await importOriginal<typeof import('@/features/models/api')>()
  return { ...actual, modelApi: { load: vi.fn(), save: vi.fn(), setDefault: vi.fn(), delete: vi.fn(), preview: vi.fn() } }
})
const revision = (): Revision => ({ id: '2f415ab0-e93f-400d-b4a8-8539bfc87903', createdAt: '2026-10-18T00:00:00Z', savedLabel: '测试', content: emptyContent(), digest: 'digest', baseRevision: null, changes: [] })
const snapshot = (): ModelSnapshot => ({ library: { schemaVersion: 1, templates: [{ id: 'template', source: 'user', label: '测试', defaultRevision: null, revisions: [revision()] }] }, policy: { schemaVersion: 1, templateRefs: [], channels: [] }, libraryDigest: 'base', policyDigest: 'policy', workspace: null, workspaceDigest: 'workspace' })
beforeEach(() => { setActivePinia(createPinia()); vi.clearAllMocks() })
describe('model template working buffer', () => {
  it('refreshes saved facts without replacing uncommitted input', async () => {
    const store = useModelsStore(); store.snapshot = snapshot(); store.edit(store.snapshot.library.templates[0])
    store.draft!.label = '保留人工输入'
    vi.mocked(modelApi.load).mockResolvedValue({ ...snapshot(), libraryDigest: 'new' })
    await store.load()
    expect(store.draft!.label).toBe('保留人工输入'); expect(store.dirty).toBe(true)
    expect(store.snapshot!.libraryDigest).toBe('new')
    vi.mocked(modelApi.save).mockRejectedValue({ message: '基准已变化' })
    await store.save()
    expect(modelApi.save).toHaveBeenCalledWith(expect.anything(), 'base')
  })
  it('preserves dirty input and exposes the actual error on a rejected save', async () => {
    const store = useModelsStore(); store.snapshot = snapshot(); store.edit(store.snapshot.library.templates[0])
    store.draft!.content.rules.fast = { mode: 'custom', value: false }
    vi.mocked(modelApi.save).mockRejectedValue({ code: 'stale_baseline', message: '基准已变化' })
    await store.save()
    expect(store.draft!.content.rules.fast).toEqual({ mode: 'custom', value: false })
    expect(store.dirty).toBe(true); expect(store.error).toBe('基准已变化'); expect(store.busy).toBe(false)
    expect(modelApi.save).toHaveBeenCalledWith(expect.objectContaining({ baseRevision: revision().id }), 'base')
  })
  it('prevents duplicate submission and uses the committed identity to clean the editor', async () => {
    const store = useModelsStore(); store.snapshot = snapshot(); store.edit(); store.draft!.label = '新建'
    let finish!: (value: Awaited<ReturnType<typeof modelApi.save>>) => void
    vi.mocked(modelApi.save).mockReturnValue(new Promise(resolve => { finish = resolve }))
    const first = store.save(); await store.save(); expect(modelApi.save).toHaveBeenCalledTimes(1)
    const saved = snapshot(); saved.library.templates[0]!.label = '新建'
    finish({ snapshot: saved, templateId: 'template', revisionId: revision().id, changed: true }); await first
    expect(store.dirty).toBe(false); expect(store.draft!.templateId).toBe('template'); expect(store.status).toBe('已保存')
  })
  it('carries this logical editor conversation into its newly saved version without applying old proposals',async()=>{
    const store=useModelsStore();await store.load();const template=store.snapshot!.library.templates[0]!;store.edit(template,template.revisions[0]!.id)
    const oldSession=store.draftSession,context=createAgentViewContext();context.initialized=true;context.input='未发送的纠偏';context.scrollTop=150;context.selectionStart=2;context.selectionEnd=4
    store.view.agentViews[oldSession]=context;store.draft!.label='saved new name'
    const saved=JSON.parse(JSON.stringify(store.snapshot!));const previous=saved.library.templates[0].revisions[0];saved.library.templates[0].revisions.push({...previous,id:'new-revision',savedLabel:'saved new name'});saved.libraryDigest='new-library'
    vi.mocked(modelApi.save).mockResolvedValue({snapshot:saved,templateId:template.id,revisionId:'new-revision',changed:true});await store.save()
    expect(store.draftSession).not.toBe(oldSession);expect(store.view.agentViews[store.draftSession]).toMatchObject({input:'未发送的纠偏',scrollTop:150,selectionStart:2,selectionEnd:4});expect(store.view.agentViews[store.draftSession]).not.toBe(context)
    expect(store.draft!.baseRevision).toBe('new-revision');expect(store.dirty).toBe(false)
  })
  it('retains separate target buffers and their original optimistic baselines when switching templates',async()=>{
    const store=useModelsStore();store.snapshot=snapshot();const first=store.snapshot.library.templates[0]!
    const second={...JSON.parse(JSON.stringify(first)),id:'second',label:'另一模版'};store.snapshot.library.templates.push(second)
    store.edit(first);const session=store.draftSession;store.draft!.content.description='第一处未保存'
    store.snapshot.libraryDigest='later';store.edit(second);store.draft!.content.description='第二处未保存'
    store.edit(first);expect(store.protectsRevision(first.id,first.revisions[0]!.id)).toBe(true);expect(store.protectsRevision(second.id,second.revisions[0]!.id)).toBe(true);expect(store.draft!.content.description).toBe('第一处未保存');expect(store.draftSession).toBe(session);expect(store.dirty).toBe(true)
    vi.mocked(modelApi.save).mockRejectedValue({message:'基准已变化'});await store.save();expect(modelApi.save).toHaveBeenCalledWith(expect.anything(),'base')
    store.edit(second);expect(store.draft!.content.description).toBe('第二处未保存');store.leave();store.edit(second);expect(store.draft!.content.description).toBe('')
  })
  it('copies the complete selected content to a fresh identity without defaults', () => {
    const store = useModelsStore(); const source = snapshot().library.templates[0]!
    source.defaultRevision = revision().id; source.revisions[0]!.content.rules.fast = { mode: 'custom', value: false }
    store.edit(source, revision().id, true)
    expect(store.draft!.templateId).toBeNull(); expect(store.draft!.baseRevision).toBeNull()
    expect(store.draft!.content.rules.fast).toEqual({ mode: 'custom', value: false })
    store.draft!.content.description = '独立副本'; expect(source.revisions[0]!.content.description).toBe('')
  })
  it('renders baseline as the sole suffix with a fixed timezone', () => {
    const text = versionLabel(revision(), true)
    expect(text).toContain('2026-10-18 08:00:00'); expect(text).toContain(' · 内置基线')
    expect(text).not.toContain('以后默认')
  })
})
