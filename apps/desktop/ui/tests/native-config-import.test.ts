import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import NativeConfigImportDialog from '@/features/models/NativeConfigImportDialog.vue'
import { useModelsV2Store } from '@/features/models/store-v2'
import { useFeedbackStore } from '@/app/feedback/store'

const { invoke, chooseFile } = vi.hoisted(() => ({ invoke: vi.fn(), chooseFile: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: chooseFile }))

function render() {
  return mount(NativeConfigImportDialog, {
    props: { open: true },
    global: { stubs: {
      UiDialog: { props: ['open', 'title'], emits: ['close'], template: '<section v-if="open"><h2>{{ title }}</h2><slot/><slot name="actions" :close="() => $emit(&quot;close&quot;)"/></section>' },
    } },
  })
}

describe('native config import dialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    chooseFile.mockReset()
  })

  it('only pins the file on selection, and applies once in the same dialog', async () => {
    const load = vi.spyOn(useModelsV2Store(), 'loadChannels').mockResolvedValue(undefined)
    chooseFile.mockResolvedValue('/fixtures/config.json')
    invoke.mockResolvedValueOnce('fixture-id').mockResolvedValueOnce({ changed: true, backupId: 'fixture-backup', externalAuth: false })
    const view = render()
    expect(view.findAll('button').find(b => b.text() === '导入')!.attributes('disabled')).toBeDefined()
    await view.findAll('button').find(b => b.text() === '选择文件')!.trigger('click')
    await flushPromises()
    expect(view.text()).toContain('config.json')
    expect(view.text()).not.toContain('/fixtures/')
    expect(invoke.mock.calls).toEqual([['model_v2_native_import_select', { path: '/fixtures/config.json' }]])
    await view.findAll('button').find(b => b.text() === '导入')!.trigger('click')
    await flushPromises()
    expect(invoke.mock.calls[1]).toEqual(['model_v2_native_import_apply', { id: 'fixture-id' }])
    expect(view.emitted('close')).toHaveLength(1)
    expect(load).toHaveBeenCalledOnce()
    expect(useFeedbackStore().toast).toContain('原文件已备份')
    view.unmount()
  })

  it('keeps stale errors in the same dialog and disables retry until selection', async () => {
    chooseFile.mockResolvedValue('/fixtures/config.json')
    invoke.mockResolvedValueOnce('old-id').mockRejectedValueOnce({ code: 'native_import_stale' })
    const view = render()
    await view.findAll('button').find(b => b.text() === '选择文件')!.trigger('click')
    await flushPromises()
    await view.findAll('button').find(b => b.text() === '导入')!.trigger('click')
    await flushPromises()
    expect(view.get('[role="alert"]').text()).toContain('重新选择文件')
    expect(view.findAll('button').find(b => b.text() === '导入')!.attributes('disabled')).toBeDefined()
    expect(view.emitted('close')).toBeUndefined()
    expect(invoke).toHaveBeenCalledTimes(2)
    view.unmount()
  })

  it('does not prepare or apply anything when the picker is cancelled', async () => {
    chooseFile.mockResolvedValue(null)
    const view = render()
    await view.findAll('button').find(b => b.text() === '选择文件')!.trigger('click')
    await flushPromises()
    await view.findAll('button').find(b => b.text() === '取消')!.trigger('click')
    expect(invoke).not.toHaveBeenCalled()
    expect(view.emitted('close')).toHaveLength(1)
    view.unmount()
  })
})
