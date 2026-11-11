import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import NativeConfigRollback from '@/features/backup/NativeConfigRollback.vue'
import { useModelsV2Store } from '@/features/models/store-v2'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
const backupId = 'bk_20261111010203_11223344'
const targetPath = '/isolated/opencodex/home/config.json'
function render() {
  return mount(NativeConfigRollback, {
    global: { stubs: {
      UiDialog: { props: ['open', 'title'], emits: ['close'], template: '<section v-if="open"><h2>{{ title }}</h2><slot/><slot name="actions" :close="() => $emit(&quot;close&quot;)"/></section>' },
    } },
  })
}

async function preview() {
  const view = render()
  const button = (label: string) => view.findAll('button').find(b => b.text() === label)!
  await button('查看备份与回退').trigger('click')
  await flushPromises()
  await view.get('select').setValue(backupId)
  expect(button('备份当前配置并回退')).toBeUndefined()
  await button('查看整文件回退预览').trigger('click')
  await flushPromises()
  expect(view.text()).toContain('43456 → 41234')
  expect(view.text()).toContain(targetPath)
  return { view, button }
}

describe('native config rollback', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValueOnce([{ id: backupId, createdAt: '2026-11-11T01:02:03Z', bytes: 512 }])
    invoke.mockResolvedValueOnce({ id: 'reviewed-rollback', preview: { currentPort: 43456, backupPort: 41234, targetPath } })
  })

  it('writes only after the dedicated rollback confirmation', async () => {
    const { view, button } = await preview()
    expect(invoke.mock.calls).toEqual([
      ['model_v2_native_rollback_list'],
      ['model_v2_native_rollback_select', { backupId }],
    ])
    invoke.mockResolvedValueOnce({ changed: true })
    await button('备份当前配置并回退').trigger('click')
    await flushPromises()
    expect(invoke.mock.calls[2]).toEqual(['model_v2_native_rollback_apply', { id: 'reviewed-rollback' }])
    expect(view.find('select').exists()).toBe(false)
    view.unmount()
  })

  it('cancel closes the preview without applying it', async () => {
    const { view, button } = await preview()
    await button('取消').trigger('click')
    expect(view.find('select').exists()).toBe(false)
    expect(invoke).toHaveBeenCalledTimes(2)
    view.unmount()
  })

  it('preserves edits made after preview and invalidates confirmation', async () => {
    const { view, button } = await preview()
    const models = useModelsV2Store()
    models.channelIntents.push({ kind: 'rename', channelId: 'fixture', name: 'Edited' })
    await button('备份当前配置并回退').trigger('click')
    expect(view.get('[role="alert"]').text()).toContain('重新核对')
    expect(button('备份当前配置并回退')).toBeUndefined()
    expect(models.channelIntents).toHaveLength(1)
    expect(invoke).toHaveBeenCalledTimes(2)
    view.unmount()
  })
})
