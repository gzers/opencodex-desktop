import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import {getActivePinia} from 'pinia'
import { defineComponent } from 'vue'
import ChannelsView from '@/features/models/ChannelsView.vue'
import TaskProgressHost from '@/components/TaskProgressHost.vue'
import {useModelsV2Store as useModelsStore} from '@/features/models/store-v2'
import {workspaceBackend} from './helpers/modelWorkspaceV2'

import { useModelSaveFeedback } from '@/app/feedback/modelSave'
import { useOperationTasks } from '@/app/feedback/tasks'
import { useNotificationsStore } from '@/features/notifications/store'

const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }))
const Harness = defineComponent({
  components: { ChannelsView, TaskProgressHost },
  template: '<div><main><ChannelsView /></main><div class="notif-layer"><TaskProgressHost /></div></div>',
})
let wrapper: VueWrapper

beforeAll(() => {
  Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value() { this.setAttribute('open', '') } })
  Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value() { this.removeAttribute('open') } })
})
beforeEach(async()=>{const backend=workspaceBackend(invoke,['channel']);await backend.load();wrapper=mount(Harness,{attachTo:document.body,global:{plugins:[getActivePinia()!]}})})
afterEach(() => { wrapper.unmount(); useOperationTasks().dispose(); vi.restoreAllMocks(); document.body.innerHTML = '' })

describe('渠道确认保存到全局结果宿主的完整点击链（模拟文件 API）', () => {
  it('实际编辑、打开预览、确认保存后只出现全局卡；稍后彻底结束，新编辑仍显示 dirty', async () => {
    await flushPromises()
    await wrapper.get('input[aria-label="启用渠道 channel"]').setValue(!useModelsStore().channels[0]!.connection.enabled)
    expect(useModelsStore().fileDirty).toBe(true)
    await wrapper.findAll('button').find(node => node.text() === '保存配置')!.trigger('click')
    await flushPromises()
    const preview = document.querySelector('dialog[open]')!
    expect(preview.textContent).toContain('保存当前渠道草稿')
    const confirm = Array.from(preview.querySelectorAll<HTMLButtonElement>('button')).find(node => node.textContent?.trim() === '确认保存')!
    confirm.click()
    await flushPromises()
    expect(invoke.mock.calls.filter(c=>c[0]==='model_v2_file_save')).toHaveLength(1)
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(wrapper.findAll('.notif-layer .process-progress')).toHaveLength(1)
    expect(wrapper.find('main .process-progress').exists()).toBe(false)
    expect(wrapper.find('main .save-result').exists()).toBe(false)
    expect(useModelsStore().fileDirty).toBe(false)
    expect(document.activeElement?.textContent?.trim()).not.toBe('立即重启 Codex')
    await wrapper.findAll('.notif-layer button').find(node => node.text() === '稍后')!.trigger('click')
    expect(useModelSaveFeedback().open).toBe(false)
    expect(wrapper.find('.notif-layer .process-progress').exists()).toBe(false)
    expect(useNotificationsStore().list).toHaveLength(0)
    expect(useOperationTasks().tasks).toHaveLength(0)
    await wrapper.get('input[aria-label="启用渠道 channel"]').setValue(!useModelsStore().channels[0]!.connection.enabled)
    expect(useModelsStore().fileDirty).toBe(true);expect(wrapper.get('[data-testid="policy-save"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.text()).not.toContain('待重启')
  })
})
