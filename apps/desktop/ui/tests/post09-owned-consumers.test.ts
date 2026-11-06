import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import SelectorPreviewDialog from '@/features/models/SelectorPreviewDialog.vue'
import TemplateComparisonDialog from '@/features/models/TemplateComparisonDialog.vue'
import ExtensionDetailModal from '@/features/extensions/components/ExtensionDetailModal.vue'
import TransferBackups from '@/features/backup/TransferBackups.vue'
import NotificationCenter from '@/features/notifications/components/NotificationCenter.vue'
import {transferBackupV2Api,type RestorePreviewV2} from '@/features/backup/api-v2'
import { useOperationTasks } from '@/app/feedback/tasks'
import { installNativeDialogStub } from './helpers/nativeDialog'

const invoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
const mounted: VueWrapper[] = []
beforeEach(() => { setActivePinia(createPinia()); installNativeDialogStub(); vi.clearAllMocks() })
afterEach(() => { mounted.splice(0).forEach(w => w.unmount()); vi.restoreAllMocks();document.body.innerHTML = '' })
function render(component: Parameters<typeof mount>[0], props = {}) { const w = mount(component, { props, attachTo: document.body }); mounted.push(w); return w }
function dialog() { return new DOMWrapper(document.body).get('dialog[open]') }
async function exit(channel: string) {
  const root = dialog()
  if (channel === 'x') await root.get('header .modal-close').trigger('click')
  if (channel === 'escape') root.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'backdrop') await root.trigger('click', { clientX: -10, clientY: -10 })
  if (channel === 'cancel') await root.findAll('footer button').find(b => b.text() === '取消')!.trigger('click')
  await flushPromises()
}

describe('POST-09 remaining owned consumer guards', () => {
  it.each(['x', 'escape', 'backdrop', 'cancel'])('uses the saving guard for preview %s and allows exit after completion', async channel => {
    const w = render(SelectorPreviewDialog, { open: true, saving: true, projection: null, channels: [], saved: null })
    await flushPromises(); await exit(channel)
    expect(w.emitted('close')).toBeUndefined(); expect(w.emitted('confirm')).toBeUndefined()
    await w.setProps({ saving: false } as never); await exit(channel)
    expect(w.emitted('close')).toHaveLength(1)
    expect(w.emitted('toggle-channel')).toBeUndefined(); expect(w.emitted('toggle-model')).toBeUndefined()
  })

  it('allows closing a preview computation without confirming a save', async () => {
    const w = render(SelectorPreviewDialog, { open: true, busy: true, projection: null, channels: [], saved: null })
    await flushPromises(); await exit('escape')
    expect(w.emitted('close')).toHaveLength(1); expect(w.emitted('confirm')).toBeUndefined()
  })

  it('has no exit footer in a comparison and keeps extension actions to the left of X', async () => {
    const comparison = render(TemplateComparisonDialog, { open: true, template: null, draft: null, libraryDigest: 'lib' })
    await flushPromises(); expect(dialog().find('footer').exists()).toBe(false)
    comparison.unmount()
    const detail = render(ExtensionDetailModal, { title: 'Fixture', markdown: '# 内容', facts: [], enabledClients: [], actions: [{ key: 'refresh', title: '刷新详情', icon: 'refresh' }], busy: true })
    await flushPromises()
    expect(dialog().findAll('header button').map(b => b.attributes('aria-label'))).toEqual(['刷新详情', '关闭'])
    expect(dialog().find('footer').exists()).toBe(false)
    await exit('x'); expect(detail.emitted('close')).toHaveLength(1); expect(detail.emitted('action')).toBeUndefined()
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('guards restore %s while checking its baseline, then cancels without writing', async channel => {
    vi.spyOn(transferBackupV2Api,'list').mockResolvedValue([{id:'fixture',groups:['channels'],enabled:true,ready:true}]);vi.spyOn(transferBackupV2Api,'restore');vi.spyOn(transferBackupV2Api,'operations').mockResolvedValue([])
    let finish!:(value:RestorePreviewV2)=>void
    vi.spyOn(transferBackupV2Api,'preview').mockReturnValue(new Promise(resolve => { finish = resolve }))
    const w = render(TransferBackups); await flushPromises()
    await w.findAll('button').find(b => b.text() === '恢复所选范围')!.trigger('click'); await flushPromises()
    await exit(channel); expect(document.querySelector('dialog[open]')).not.toBeNull()
    finish({restoreId:'fixture-restore',planDigest:'fixture-digest',groups:['channels']}); await flushPromises(); await exit(channel)
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(transferBackupV2Api.restore).not.toHaveBeenCalled()
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('cancels notification cleanup through %s without clearing tasks or calling IPC', async channel => {
    const operations = useOperationTasks()
    operations.tasks = [{ id: 'fixture', kind: 'research', phase: 'failed', collapsed: true }]
    const w = render(NotificationCenter, { open: true })
    await w.findAll('button').find(b => b.text() === '清理')!.trigger('click'); await flushPromises()
    await exit(channel)
    expect(document.querySelector('dialog[open]')).toBeNull(); expect(operations.tasks).toHaveLength(1)
    expect(invoke).not.toHaveBeenCalled(); expect(w.emitted('close')).toBeUndefined()
  })
})
