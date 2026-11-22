import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import ImportedConnections from '@/features/sync/ImportedConnections.vue'
import { listImportedConnections, activateImportedConnection } from '@/features/sync/api'
vi.mock('@/features/sync/api', () => ({ listImportedConnections: vi.fn(), activateImportedConnection: vi.fn() }))
const dialog = { props: ['open'], emits: ['close'], template: '<section v-if="open"><button @click="$emit(\'close\')">×</button><slot/><slot name="actions" :close="()=> $emit(\'close\')"/></section>' }
const render = () => mount(ImportedConnections, { global: { stubs: { UiDialog: dialog } } })
const fixture = () => ({ baseline: 'exact', items: [{ id: 'candidate', endpoint: { id: 'e', url: 'https://dav.example.test', remotePath: 'portable', username: 'fixture', conflictPolicy: 'ask' }, hasPassword: true }] })
beforeEach(() => { vi.clearAllMocks() })
describe('imported connection lifecycle', () => {
  it('hides only after a successful empty read; loading and failed reads stay visible and retryable', async () => {
    let resolve!: (value: { baseline: string; items: never[] }) => void
    vi.mocked(listImportedConnections).mockReturnValueOnce(new Promise(r => { resolve = r }))
    const w = render(); expect(w.text()).toContain('正在读取导入的连接'); resolve({ baseline: 'empty', items: [] }); await flushPromises()
    expect(w.find('.imported-connections').exists()).toBe(false); expect(w.find('summary').exists()).toBe(false); w.unmount()
    vi.mocked(listImportedConnections).mockRejectedValueOnce(new Error('cannot read')).mockResolvedValueOnce({ baseline: 'empty', items: [] })
    const failed = render(); await flushPromises(); expect(failed.text()).toContain('cannot read')
    await failed.find('button').trigger('click'); await flushPromises(); expect(failed.find('.imported-connections').exists()).toBe(false); failed.unmount()
  })
  it('shows count and purpose and closing confirmation never activates', async () => {
    vi.mocked(listImportedConnections).mockResolvedValue(fixture()); const w = render(); await flushPromises()
    expect(w.find('summary').text()).toBe('导入的连接（1）'); expect(w.text()).toContain('尚未启用')
    await w.findAll('button').find(b => b.text() === '启用此连接')!.trigger('click'); await w.findAll('button').find(b => b.text() === '取消')!.trigger('click')
    expect(activateImportedConnection).not.toHaveBeenCalled(); w.unmount()
  })
  it('ignores a list or activation response after unmount', async () => {
    let resolve!: (value: ReturnType<typeof fixture>) => void
    vi.mocked(listImportedConnections).mockReturnValueOnce(new Promise(r => { resolve = r }))
    const loading = render(); loading.unmount(); resolve(fixture()); await flushPromises(); expect(activateImportedConnection).not.toHaveBeenCalled()
    vi.mocked(listImportedConnections).mockResolvedValue(fixture()); let finish!: () => void
    vi.mocked(activateImportedConnection).mockReturnValue(new Promise<void>(r => { finish = r }))
    const w = render(); await flushPromises(); await w.findAll('button').find(b => b.text() === '启用此连接')!.trigger('click')
    await w.findAll('button').find(b => b.text() === '确认启用')!.trigger('click'); w.unmount(); finish(); await flushPromises()
    expect(w.emitted('activated')).toBeUndefined()
  })
})
