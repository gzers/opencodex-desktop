import { beforeAll, afterEach, describe, it, expect, vi } from 'vitest'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import ChannelNodeSelect from '@/features/models/ChannelNodeSelect.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {workspaceChannel,workspaceId,workspaceHash} from './helpers/modelWorkspaceV2'
import { installNativeDialogStub } from './helpers/nativeDialog'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
const channel=workspaceChannel('fixture');channel.connection.nodes=[{id:workspaceId('one'),name:'主节点',url:'https://example.test'}];channel.connection.activeNodeId=workspaceId('one')
const expected={dtoVersion:2 as const,draftId:workspaceId('draft'),draftGeneration:1,sourceDigest:workspaceHash(1),libraryDigest:workspaceHash(2),policyDigest:workspaceHash(3),rootRevision:workspaceHash(4)}
let wrapper: VueWrapper
beforeAll(installNativeDialogStub)
afterEach(() => { wrapper?.unmount(); document.body.innerHTML = '' })
describe('检测按钮的公共模态接入', () => {
  it('键盘在真实业务控件内循环，Esc归还检测按钮，重开仍有焦点', async () => {
    wrapper = mount(ChannelNodeSelect, { props: { channel,expected }, attachTo: document.body })
    const trigger = wrapper.get('button').element as HTMLButtonElement
    trigger.focus(); trigger.click(); await flushPromises();wrapper.findComponent(UiSelectField).vm.$emit('update:modelValue','openai_list');await flushPromises()
    const dialog = document.querySelector<HTMLDialogElement>('dialog[open]')!
    const first = dialog.querySelector<HTMLButtonElement>('.modal-close')!, last = dialog.querySelector<HTMLButtonElement>('[aria-label="检测节点 主节点"]')!
    last.focus(); last.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }))
    expect(document.activeElement).toBe(first)
    first.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }))
    expect(document.activeElement).toBe(last)
    last.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); await flushPromises()
    expect(dialog.open).toBe(false); expect(document.activeElement).toBe(trigger)
    trigger.click(); await flushPromises(); expect(dialog.contains(document.activeElement)).toBe(true)
  })
  it('没有节点时不能打开观察窗', () => {
    wrapper = mount(ChannelNodeSelect, { props: { channel: { ...channel, connection: { ...channel.connection, nodes: [] } } } })
    expect((wrapper.get('button').element as HTMLButtonElement).disabled).toBe(true)
    expect(document.querySelector('dialog[open]')).toBeNull()
  })
})
