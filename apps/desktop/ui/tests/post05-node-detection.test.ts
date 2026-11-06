import { beforeAll, beforeEach, afterEach, describe, it, expect, vi } from 'vitest'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { reactive } from 'vue'
import ChannelNodeSelect from '@/features/models/ChannelNodeSelect.vue'
import {modelV2Api as modelApi,type NodeMeasurementSetV2 as NodeMeasurementSet,type ReadBaseV2} from '@/features/models/api-v2'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {workspaceChannel,workspaceId,workspaceHash} from './helpers/modelWorkspaceV2'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}))
const nodeA=workspaceId('channelnode'),nodeB=workspaceId('channelbackup')
const expected:ReadBaseV2={dtoVersion:2,draftId:workspaceId('draft'),draftGeneration:1,sourceDigest:workspaceHash(1),libraryDigest:workspaceHash(2),policyDigest:workspaceHash(3),rootRevision:workspaceHash(4)}
function channel(){const value=workspaceChannel('channel');value.connection.nodes[0]!.name='当前';return value}
let wrapper: VueWrapper
beforeAll(installNativeDialogStub)
beforeEach(() => {vi.clearAllMocks();vi.spyOn(modelApi,'measureNodes');vi.spyOn(modelApi,'stopMeasurement').mockResolvedValue()})
afterEach(() => { wrapper?.unmount();vi.restoreAllMocks(); document.body.innerHTML = '' })
async function open() { const trigger = wrapper.get('button').element as HTMLButtonElement; trigger.focus(); trigger.click(); await flushPromises();const method=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='本次节点清单获取方式')!;if(!method.props('modelValue'))method.vm.$emit('update:modelValue','openai_list');await flushPromises();return trigger }
const dialog = () => document.querySelector<HTMLDialogElement>('dialog[open]')!
const action = (label: string) => dialog().querySelector<HTMLButtonElement>('button[aria-label="' + label + '"]')!

describe('POST05 公共模态节点检测（全部为模拟API）', () => {
  it('两字检测打开公共模态，无下拉与底栏；选择只改当前草稿并归还焦点', async () => {
    const value = reactive(channel()), before = JSON.parse(JSON.stringify(value))
    wrapper = mount(ChannelNodeSelect, { props: { channel: value,expected,onSelect:(id:string)=>{value.connection.activeNodeId=id} }, attachTo: document.body })
    const trigger = await open()
    expect(trigger.textContent?.trim()).toBe('检测')
    expect(trigger.getAttribute('aria-haspopup')).toBe('dialog')
    expect(dialog()?.classList.contains('ui-dialog')).toBe(true)
    expect(dialog().getAttribute('aria-modal')).toBe('true')
    expect(dialog().querySelector('footer')).toBeNull()
    expect(document.querySelector('[role="menu"]')).toBeNull()
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
    action('选择节点 备用').click(); await flushPromises()
    expect(value.connection.activeNodeId).toBe(nodeB)
    before.connection.activeNodeId = nodeB; expect(JSON.parse(JSON.stringify(value))).toEqual(before)
    expect(dialog()).toBeNull(); expect(document.activeElement).toBe(trigger)
    expect(modelApi.measureNodes).not.toHaveBeenCalled()
    await open(); expect(action('选择节点 备用').getAttribute('aria-pressed')).toBe('true')
  })

  it('结果就地更新，焦点、节点身份与窗口保持；滚动和resize不触发关闭', async () => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value,expected,onSelect:(id:string)=>{value.connection.activeNodeId=id} }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await open(); const sheet = dialog(), choice = action('选择节点 备用')
    action('检测节点 备用').click(); await flushPromises(); choice.focus()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    expect(Object.keys(request).sort()).toEqual(['channelId','discoveryKind','expected','nodeIds','runId']);expect(request.expected).toEqual(expected);expect(request.channelId).toBe(workspaceId('channel'));expect(request.discoveryKind).toBe('openai_list')
    expect(request.nodeIds).toEqual([nodeB])
    window.dispatchEvent(new Event('resize')); window.dispatchEvent(new Event('scroll'))
    sheet.querySelector('.ui-dialog-content')!.dispatchEvent(new Event('scroll'))
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: nodeB, elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog()).toBe(sheet); expect(action('选择节点 备用')).toBe(choice)
    expect(document.activeElement).toBe(choice); expect(sheet.textContent).toContain('42 ms')
    expect(value.connection.activeNodeId).toBe(nodeA); expect(modelApi.stopMeasurement).not.toHaveBeenCalled()
  })

  it.each(['X', 'Escape', '遮罩'])('%s关闭观察不停止请求，重开保留结果和选择', async exit => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value,expected,onSelect:(id:string)=>{value.connection.activeNodeId=id} }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const trigger = await open(); action('检测节点 备用').click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0], sheet = dialog()
    if (exit === 'X') sheet.querySelector<HTMLButtonElement>('.modal-close')!.click()
    else if (exit === 'Escape') sheet.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    else sheet.dispatchEvent(new MouseEvent('click', { clientX: -1, clientY: -1, bubbles: true }))
    await flushPromises(); expect(dialog()).toBeNull(); expect(document.activeElement).toBe(trigger)
    expect(modelApi.stopMeasurement).not.toHaveBeenCalled()
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: nodeB, elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    await open(); expect(dialog().textContent).toContain('42 ms')
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
    expect(modelApi.measureNodes).toHaveBeenCalledOnce()
  })

  it('一键检测与停止分离，停止后保留已完成结果，不选择节点', async () => {
    wrapper = mount(ChannelNodeSelect, { props: { channel: channel(),expected }, attachTo: document.body })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    await open(); [...dialog().querySelectorAll('button')].find(b => b.textContent === '一键检测')!.click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    expect(request.nodeIds).toEqual([nodeA, nodeB])
    expect(action('检测节点 备用').disabled).toBe(true)
    ;[...dialog().querySelectorAll('button')].find(b => b.textContent === '停止')!.click(); await flushPromises()
    expect(modelApi.stopMeasurement).toHaveBeenCalledExactlyOnceWith(request.runId)
    finish({ runId: request.runId, stopped: true, results: [{ nodeId: nodeA, elapsedMs: 12, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog().textContent).toContain('已完成结果保留'); expect(dialog().textContent).toContain('12 ms')
    expect(action('选择节点 当前').getAttribute('aria-pressed')).toBe('true')
  })

  it('连接变更使晚回失效，独立只读示例默认不能检测或切换', async () => {
    const value = reactive(channel())
    wrapper = mount(ChannelNodeSelect, { props: { channel: value,expected,onSelect:(id:string)=>{value.connection.activeNodeId=id}, readonly: true }, attachTo: document.body })
    await open(); expect(action('选择节点 备用').disabled).toBe(true); expect(action('检测节点 备用').disabled).toBe(true)
    await wrapper.setProps({ readonly: false })
    let finish!: (r: NodeMeasurementSet) => void
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    action('检测节点 备用').click(); await flushPromises()
    const request = vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
    value.connection.nodes[1]!.url = 'https://changed.example.test'; await flushPromises()
    expect(modelApi.stopMeasurement).toHaveBeenCalledWith(request.runId)
    finish({ runId: request.runId, stopped: false, results: [{ nodeId: nodeB, elapsedMs: 42, modelCount: 3, error: null }] }); await flushPromises()
    expect(dialog().textContent).not.toContain('42 ms'); expect(value.connection.activeNodeId).toBe(nodeA)
  })

  it('明确允许只读检测时，readonly只拦选择而不拦模型清单读取', async () => {
    const value = reactive(channel()), before = JSON.stringify(value)
    wrapper = mount(ChannelNodeSelect, { props: { channel: value,expected,onSelect:(id:string)=>{value.connection.activeNodeId=id}, readonly: true, measurementDisabled: false }, attachTo: document.body })
    vi.mocked(modelApi.measureNodes).mockImplementationOnce(async request => ({ runId: request.runId, stopped: false, results: [{ nodeId: nodeB, elapsedMs: 8, modelCount: 1, error: null }] }))
    await open()
    expect(action('选择节点 备用').disabled).toBe(true)
    expect(action('检测节点 备用').disabled).toBe(false)
    action('检测节点 备用').click(); await flushPromises()
    expect(dialog().textContent).toContain('8 ms'); expect(JSON.stringify(value)).toBe(before)
  })
})
