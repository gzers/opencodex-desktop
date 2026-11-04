import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyRules, modelApi, type ChannelPolicy } from '@/features/models/api'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
function channel(): ChannelPolicy { return { id: 'one', label: '测试渠道', templateRefs: [], modelRefs: {}, connection: { providerId: 'fixture', protocol: 'openai-chat', nodes: [{ id: 'a', name: '当前', url: 'https://a.example.test' }, { id: 'b', name: '备用', url: 'https://b.example.test' }], activeNodeId: 'a', credentialRef: null, enabled: true, allowPrivateNetwork: false }, selection: { mode: 'none' }, modelOrder: [], discovered: [], defaults: emptyRules(), overrides: {}, familyOverrides: {} } }
let wrapper: VueWrapper
beforeAll(installNativeDialogStub)
beforeEach(() => {
  setActivePinia(createPinia())
  const models = useModelsStore()
  models.snapshot = { library: { schemaVersion: 1, templates: [] }, libraryDigest: 'library', policy: { schemaVersion: 1, templateRefs: [], channels: [channel()] }, policyDigest: 'policy', workspaceDigest: 'workspace', workspace: null }
  models.restoreChannels()
  vi.spyOn(modelApi, 'latestPolicy').mockResolvedValue(null)
  vi.spyOn(modelApi, 'project').mockResolvedValue({ rows: [], knownActiveModels: 0, exactCatalogCount: 0, notices: [] })
  vi.spyOn(modelApi, 'executionDirectory').mockResolvedValue({ channels: [], rows: [] })
  vi.spyOn(modelApi, 'writeFile')
  vi.spyOn(modelApi, 'measureNodes')
  vi.spyOn(modelApi, 'stopNodeMeasurement').mockResolvedValue(undefined)
  wrapper = mount(ChannelsView, { attachTo: document.body, global: { stubs: { ChannelEditor: true, NewChannelDialog: true, ChannelBatchDialog: true, SelectorPreviewDialog: true } } })
})
afterEach(() => { wrapper.unmount(); vi.restoreAllMocks(); document.body.innerHTML = '' })

describe('POST05 列表实际消费者', () => {
  it('保留选择列与四列，检测位于操作列，选择节点只改草稿', async () => {
    await flushPromises()
    expect(wrapper.findAll('thead th').map(node => node.text())).toEqual(['', '渠道', '模型', '启用', '操作'])
    const cells = wrapper.findAll('tbody td')
    expect(cells).toHaveLength(5)
    expect(cells[0]!.find('input[type="checkbox"]').exists()).toBe(true)
    await cells[4]!.get('.channel-node-detect').trigger('click'); await flushPromises()
    document.querySelector<HTMLButtonElement>('dialog[open] [aria-label="选择节点 备用"]')!.click(); await flushPromises()
    const models = useModelsStore()
    expect(models.channelDraft!.channels[0]!.connection.activeNodeId).toBe('b')
    expect(models.snapshot!.policy.channels[0]!.connection.activeNodeId).toBe('a')
    expect(models.channelDirty).toBe(true)
    expect(modelApi.writeFile).not.toHaveBeenCalled(); expect(modelApi.measureNodes).not.toHaveBeenCalled()
  })

  it('离开渠道页只关闭观察，回到渠道页可继续查看同一检测', async () => {
    let finish!: (value: Awaited<ReturnType<typeof modelApi.measureNodes>>) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    document.querySelector<HTMLButtonElement>('dialog[open] [aria-label="检测节点 备用"]')!.click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    await wrapper.setProps({ active: false }); await flushPromises()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(modelApi.stopNodeMeasurement).not.toHaveBeenCalled()
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 19, modelCount: 2, error: null }] }); await flushPromises()
    await wrapper.setProps({ active: true }); await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    expect(document.querySelector('dialog[open]')!.textContent).toContain('19 ms')
    expect(modelApi.measureNodes).toHaveBeenCalledOnce()
  })

  it.each(['搜索隐藏行', '进入详情'])('%s不销毁检测，回到列表重开保留结果', async leave => {
    let finish!: (value: Awaited<ReturnType<typeof modelApi.measureNodes>>) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    document.querySelector<HTMLButtonElement>('dialog[open] [aria-label="检测节点 备用"]')!.click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    document.querySelector<HTMLButtonElement>('dialog[open] .modal-close')!.click(); await flushPromises()
    const models = useModelsStore()
    if (leave === '搜索隐藏行') models.view.channelSearch = 'not-found'
    else models.view.channelEditor = 'one'
    await flushPromises()
    expect(modelApi.stopNodeMeasurement).not.toHaveBeenCalled()
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 23, modelCount: 2, error: null }] }); await flushPromises()
    models.view.channelSearch = ''; models.view.channelEditor = ''; await flushPromises()
    await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    expect(document.querySelector('dialog[open]')!.textContent).toContain('23 ms')
    expect(modelApi.measureNodes).toHaveBeenCalledOnce()
  })

  it.each([{ providerId: 'openai', authMode: 'key' }, { providerId: 'fixture', authMode: 'oauth' }, { providerId: 'fixture', authMode: 'forward' }])('保留既有官方/专用认证保护：$providerId/$authMode', async connection => {
    const channel = useModelsStore().channelDraft!.channels[0]!
    Object.assign(channel.connection, connection)
    await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    const dialog = document.querySelector<HTMLDialogElement>('dialog[open]')!
    expect(dialog.querySelector<HTMLButtonElement>('[aria-label="选择节点 备用"]')!.disabled).toBe(true)
    expect(dialog.querySelector<HTMLButtonElement>('[aria-label="检测节点 备用"]')!.disabled).toBe(true)
    dialog.querySelector<HTMLButtonElement>('.modal-close')!.click(); await flushPromises()
    expect(modelApi.measureNodes).not.toHaveBeenCalled(); expect(modelApi.writeFile).not.toHaveBeenCalled()
  })

  it('未保存改名检测仍绑定已保存凭据归属，保存来源变化后旧结果失效', async () => {
    const models = useModelsStore(), current = models.channelDraft!.channels[0]!
    current.label = '改名草稿'; current.connection.providerId = 'renamed-provider'
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(async request => ({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 17, modelCount: 2, error: null }] }))
    await wrapper.get('.channel-node-detect').trigger('click'); await flushPromises()
    document.querySelector<HTMLButtonElement>('dialog[open] [aria-label="检测节点 备用"]')!.click(); await flushPromises()
    expect(modelApi.measureNodes).toHaveBeenCalledWith(expect.objectContaining({ providerId: 'fixture', apiKey: null }))
    expect(document.querySelector('dialog[open]')!.textContent).toContain('17 ms')
    expect(current.connection.providerId).toBe('renamed-provider'); expect(modelApi.writeFile).not.toHaveBeenCalled()
    models.snapshot!.policy.channels[0]!.connection.providerId = 'renamed-provider'; await flushPromises()
    expect(document.querySelector('dialog[open]')!.textContent).not.toContain('17 ms')
  })
})
