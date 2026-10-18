import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, modelApi, versionLabel, type ModelSnapshot, type Revision } from '@/features/models/api'

vi.mock('@/features/models/api', async importOriginal => {
  const actual = await importOriginal<typeof import('@/features/models/api')>()
  return { ...actual, modelApi: { load: vi.fn(), save: vi.fn(), setDefault: vi.fn(), delete: vi.fn(), preview: vi.fn() } }
})
const revision = (): Revision => ({ id: '2f415ab0-e93f-400d-b4a8-8539bfc87903', createdAt: '2026-10-18T00:00:00Z', savedLabel: '测试', content: emptyContent(), digest: 'digest', baseRevision: null, changes: [] })
const snapshot = (): ModelSnapshot => ({ library: { schemaVersion: 1, templates: [{ id: 'template', source: 'user', label: '测试', defaultRevision: null, revisions: [revision()] }] }, libraryDigest: 'base', policyDigest: 'policy' })
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
