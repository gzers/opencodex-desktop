import { beforeAll, beforeEach, afterEach, describe, it, expect, vi } from 'vitest'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { reactive } from 'vue'
import ChannelNodeSelect from '@/features/models/ChannelNodeSelect.vue'
import { emptyRules, modelApi, type ChannelPolicy, type NodeMeasurementSet } from '@/features/models/api'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@/features/models/api', async original => ({ ...await original<typeof import('@/features/models/api')>(), modelApi: { measureNodes: vi.fn(), stopNodeMeasurement: vi.fn(async () => {}) } }))
function channel(): ChannelPolicy { return { id: 'channel', label: '节点测试渠道', templateRefs: [], modelRefs: {}, connection: { providerId: 'provider', protocol: 'openai-responses', nodes: [{ id: 'a', name: '当前', url: 'https://a.example.test' }, { id: 'b', name: '备用', url: 'https://b.example.test' }], activeNodeId: 'a', credentialRef: 'fixture-reference', enabled: true, allowPrivateNetwork: false }, selection: { mode: 'all' }, modelOrder: ['m'], discovered: [], defaults: emptyRules(), overrides: {}, familyOverrides: {} } }
let wrapper: VueWrapper
beforeAll(installNativeDialogStub)
beforeEach(() => vi.clearAllMocks())
afterEach(() => { wrapper?.unmount(); document.body.innerHTML = '' })
async function open() { const trigger = wrapper.get('button').element as HTMLButtonElement; trigger.focus(); trigger.click(); await flushPromises(); return trigger }
const dialog = () => document.querySelector<HTMLDialogElement>('dialog[open]')!
const action = (label: string) => dialog().querySelector<HTMLButtonElement>('button[aria-label="' + label + '"]')!

describe('POST05 公共模态节点检测（全部为模拟API）', () => {
  it('两字检测打开公共模态，无下拉与底栏；选择只改当前草稿并归还焦点', async () => {
    const value = reactive(channel()), before = JSON.parse(JSON.stringify(value))
    wrapper = mount(ChannelNodeSelect, { props: { channel: value }, attachTo: document.body })
    const trigger = await open()
    expect(trigger.textContent?.trim()).toBe('检测')
    expect(trigger.getAttribute('aria-haspopup')).toBe('dialog')
    expect(dialog()?.classList.contains('ui-dialog')).toBe(true)
    expect(dialog().getAttribute('aria-modal')).toBe('true')
    expect(dialog().querySelector('footer')).toBeNull()
    expect(document.querySelector('[role="menu"]')).toBeNull()
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
    action('选择节点 备用').click(); await flushPromises()
    expect(value.connection.activeNodeId).toBe('b')
    before.connection.activeNodeId = 'b'; expect(JSON.parse(JSON.stringify(value))).toEqual(before)
    expect(dialog()).toBeNull(); expect(document.activeElement).toBe(trigger)
    expect(modelApi.measureNodes).not.toHaveBeenCalled()
    await open(); expect(action('选择节点 备用').getAttribute('aria-pressed')).toBe('true')
  })

  it('结果就地更新，焦点、节点身份与窗口保持；滚动和resize不触发关闭', async () => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await open(); const sheet = dialog(), choice = action('选择节点 备用')
    action('检测节点 备用').click(); await flushPromises(); choice.focus()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    expect(request.credentialRef).toBe('fixture-reference'); expect(request.apiKey).toBeNull()
    expect(request.nodes.map(n => n.id)).toEqual(['b'])
    window.dispatchEvent(new Event('resize')); window.dispatchEvent(new Event('scroll'))
    sheet.querySelector('.ui-dialog-content')!.dispatchEvent(new Event('scroll'))
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog()).toBe(sheet); expect(action('选择节点 备用')).toBe(choice)
    expect(document.activeElement).toBe(choice); expect(sheet.textContent).toContain('42 ms')
    expect(value.connection.activeNodeId).toBe('a'); expect(modelApi.stopNodeMeasurement).not.toHaveBeenCalled()
  })

  it.each(['X', 'Escape', '遮罩'])('%s关闭观察不停止请求，重开保留结果和选择', async exit => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const trigger = await open(); action('检测节点 备用').click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0], sheet = dialog()
    if (exit === 'X') sheet.querySelector<HTMLButtonElement>('.modal-close')!.click()
    else if (exit === 'Escape') sheet.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    else sheet.dispatchEvent(new MouseEvent('click', { clientX: -1, clientY: -1, bubbles: true }))
    await flushPromises(); expect(dialog()).toBeNull(); expect(document.activeElement).toBe(trigger)
    expect(modelApi.stopNodeMeasurement).not.toHaveBeenCalled()
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    await open(); expect(dialog().textContent).toContain('42 ms')
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
    expect(modelApi.measureNodes).toHaveBeenCalledOnce()
  })

  it('一键检测与停止分离，停止后保留已完成结果，不选择节点', async () => {
    wrapper = mount(ChannelNodeSelect, { props: { channel: channel() }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await open(); [...dialog().querySelectorAll('button')].find(b => b.textContent === '一键检测')!.click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    expect(request.nodes.map(n => n.id)).toEqual(['a', 'b'])
    expect(action('检测节点 备用').disabled).toBe(true)
    ;[...dialog().querySelectorAll('button')].find(b => b.textContent === '停止')!.click(); await flushPromises()
    expect(modelApi.stopNodeMeasurement).toHaveBeenCalledExactlyOnceWith(request.runId)
    finish({ runId: request.runId, stopped: true, results: [{ nodeId: 'a', elapsedMs: 12, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog().textContent).toContain('已完成结果保留'); expect(dialog().textContent).toContain('12 ms')
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
  })

  it('连接变更使晚回失效，独立只读示例默认不能检测或切换', async () => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value, readonly: true }, attachTo: document.body })
    await open(); expect(action('选择节点 备用').disabled).toBe(true); expect(action('检测节点 备用').disabled).toBe(true)
    await wrapper.setProps({ readonly: false })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    action('检测节点 备用').click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    value.connection.nodes[1]!.url = 'https://changed.example.test'; await flushPromises()
    expect(modelApi.stopNodeMeasurement).toHaveBeenCalledWith(request.runId)
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog().textContent).not.toContain('42 ms'); expect(value.connection.activeNodeId).toBe('a')
  })

  it('明确允许只读检测时，readonly只拦选择而不拦模型清单读取', async () => {
    const value = reactive(channel()), before = JSON.stringify(value)
    wrapper = mount(ChannelNodeSelect, { props: { channel: value, readonly: true, measurementDisabled: false }, attachTo: document.body })
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(async request => ({ runId: request.runId, stopped: false, results: [{ nodeId: 'b', elapsedMs: 8, modelCount: 1, error: null }] }))
    await open()
    expect(action('选择节点 备用').disabled).toBe(true)
    expect(action('检测节点 备用').disabled).toBe(false)
    action('检测节点 备用').click(); await flushPromises()
    expect(dialog().textContent).toContain('8 ms'); expect(JSON.stringify(value)).toBe(before)
  })
})
