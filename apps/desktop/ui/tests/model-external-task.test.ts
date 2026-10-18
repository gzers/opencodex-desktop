import { mount, flushPromises } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ExternalTaskDialog from '@/features/models/ExternalTaskDialog.vue'
import { modelApi, type ModelSnapshot, type TaskBundle } from '@/features/models/api'
vi.mock('@/features/models/api', async original => ({ ...await original<object>(), modelApi: { external: vi.fn(), activate: vi.fn() } }))
const snapshot: ModelSnapshot = { library: { schemaVersion: 1, templates: [] }, policy: { schemaVersion: 1, templateRefs: [], channels: [] }, libraryDigest: 'baseline', policyDigest: 'policy', workspace: null, workspaceDigest: 'workspace' }
const bundle = { taskId: 'task', content: '完整指令 + 同源字段契约 + 目标', grant: { task: { id: 'task', contractVersion: 1, scene: 'from_sources', targets: [], allowNew: true, requirements: '', expectedLibrary: 'baseline', expiresAt: '2026-10-25', revoked: false }, token: 'capability' } } as TaskBundle
beforeEach(() => { vi.useFakeTimers(); vi.clearAllMocks(); vi.mocked(modelApi.external).mockResolvedValue(bundle); vi.mocked(modelApi.activate).mockResolvedValue() })
afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals() })
const dialog = { template: '<div><slot/><slot name="actions"/></div>' }
describe('single external-task dialog', () => {
  it('prepares locally without activation, then copies one complete payload', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const wrapper = mount(ExternalTaskDialog, { props: { open: false, snapshot }, global: { stubs: { UiDialog: dialog } } })
    await wrapper.setProps({ open: true }); await vi.advanceTimersByTimeAsync(250); await flushPromises()
    expect(modelApi.external).toHaveBeenCalled(); expect(modelApi.activate).not.toHaveBeenCalled()
    const copy = wrapper.findAll('button').find(b => b.text() === '复制完整任务')!
    await copy.trigger('click'); await flushPromises()
    expect(modelApi.activate).toHaveBeenCalledWith(bundle.grant); expect(writeText).toHaveBeenCalledExactlyOnceWith(bundle.content)
    wrapper.unmount()
  })
  it('preserves supplemental input and updates one preview without a second dialog or task queue', async () => {
    const wrapper = mount(ExternalTaskDialog, { props: { open: false, snapshot }, global: { stubs: { UiDialog: dialog } } })
    await wrapper.setProps({ open: true }); await wrapper.findAll('textarea')[0]!.setValue('保留我的资料')
    await vi.advanceTimersByTimeAsync(250); await flushPromises()
    expect(modelApi.external).toHaveBeenLastCalledWith(expect.objectContaining({ requirements: '保留我的资料', expectedLibrary: 'baseline' }))
    expect(wrapper.findAll('textarea')).toHaveLength(2); expect(wrapper.text()).not.toContain('创建任务')
    expect(modelApi.activate).not.toHaveBeenCalled(); wrapper.unmount()
  })
})
