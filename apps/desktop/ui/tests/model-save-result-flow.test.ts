import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ChannelsView from '@/features/models/ChannelsView.vue'
import TaskProgressHost from '@/components/TaskProgressHost.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyRules, modelApi, type ModelSnapshot } from '@/features/models/api'
import { useModelSaveFeedback } from '@/app/feedback/modelSave'
import { useOperationTasks } from '@/app/feedback/tasks'
import { useNotificationsStore } from '@/features/notifications/store'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
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
beforeEach(() => {
  const pinia = createPinia()
  setActivePinia(pinia)
  const models = useModelsStore()
  models.snapshot = {
    library: { schemaVersion: 1, templates: [] }, libraryDigest: 'library',
    policy: { schemaVersion: 1, templateRefs: [], channels: [{
      id: crypto.randomUUID(), label: '测试渠道', templateRefs: [], modelRefs: {},
      connection: { providerId: 'custom', protocol: 'openai-chat', nodes: [], activeNodeId: '', credentialRef: null, enabled: true, allowPrivateNetwork: false },
      selection: { mode: 'none' }, modelOrder: [], discovered: [], defaults: emptyRules(), overrides: {}, familyOverrides: {},
    }] }, policyDigest: 'current-file', workspaceDigest: 'workspace', workspace: null,
  } satisfies ModelSnapshot
  models.restoreChannels()
  vi.spyOn(modelApi, 'latestPolicy').mockResolvedValue(null)
  vi.spyOn(modelApi, 'project').mockResolvedValue({ rows: [], knownActiveModels: 0, exactCatalogCount: 0, notices: [] })
  vi.spyOn(modelApi, 'executionDirectory').mockResolvedValue({ channels: [], rows: [] })
  vi.spyOn(modelApi, 'writeFile').mockImplementation(async (expectedFile, policy) => {
    expect(expectedFile).toBe('current-file')
    return { changed: true, snapshot: { ...JSON.parse(JSON.stringify(models.snapshot)), policy: JSON.parse(JSON.stringify(policy)), policyDigest: 'saved-file', workspace: null } }
  })
  wrapper = mount(Harness, { attachTo: document.body, global: { plugins: [pinia] } })
})
afterEach(() => { wrapper.unmount(); useOperationTasks().dispose(); vi.restoreAllMocks(); document.body.innerHTML = '' })

describe('渠道确认保存到全局结果宿主的完整点击链（模拟文件 API）', () => {
  it('实际编辑、打开预览、确认保存后只出现全局卡；稍后彻底结束，新编辑仍显示 dirty', async () => {
    await flushPromises()
    await wrapper.get('.channel-enable-cell button').trigger('click')
    expect(useModelsStore().channelDirty).toBe(true)
    await wrapper.findAll('button').find(node => node.text() === '保存')!.trigger('click')
    await flushPromises()
    const preview = document.querySelector('dialog[open]')!
    expect(preview.textContent).toContain('保存预览')
    const confirm = Array.from(preview.querySelectorAll<HTMLButtonElement>('button')).find(node => node.textContent?.trim() === '确认保存')!
    confirm.click()
    await flushPromises()
    expect(modelApi.writeFile).toHaveBeenCalledOnce()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(wrapper.findAll('.notif-layer .process-progress')).toHaveLength(1)
    expect(wrapper.find('main .process-progress').exists()).toBe(false)
    expect(wrapper.find('main .save-result').exists()).toBe(false)
    expect(useModelsStore().channelDirty).toBe(false)
    expect(document.activeElement?.textContent?.trim()).toBe('立即重启 Codex')
    await wrapper.findAll('.notif-layer button').find(node => node.text() === '稍后')!.trigger('click')
    expect(useModelSaveFeedback().open).toBe(false)
    expect(wrapper.find('.notif-layer .process-progress').exists()).toBe(false)
    expect(useNotificationsStore().list).toHaveLength(0)
    expect(useOperationTasks().tasks).toHaveLength(0)
    await wrapper.get('.channel-enable-cell button').trigger('click')
    expect(wrapper.get('main').text()).toContain('未保存修改')
    expect(wrapper.text()).not.toContain('待重启')
  })
})
