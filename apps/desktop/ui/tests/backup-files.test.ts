import { mount } from '@vue/test-utils'
import { flushPromises } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import BackupFiles from '../src/features/backup/BackupFiles.vue'
import type { BackupFileNode } from '../src/features/backup/api'

const query = vi.hoisted(() => vi.fn())
vi.mock('../src/features/backup/api', () => ({ getBackupFiles: query }))
function node(id: string, label: string, children: BackupFileNode[] = [], directory = true): BackupFileNode {
  return { id, label, directory, purpose: '实际事务内容', canOpen: true, children }
}
function result() {
  return { rootPath: '/isolated/backups', nodes: [node('.', 'backups', [node('2026', '2026', [node('2026/10', '10', [node('2026/10/upgrade', 'upgrade', [node('2026/10/upgrade/file.json', 'file.json', [], false)])])])])] }
}
describe('actual backup file browser', () => {
  beforeEach(() => { query.mockReset(); query.mockResolvedValue(result()) })
  it('always renders the section and initially stops at month', async () => {
    const wrapper = mount(BackupFiles)
    await flushPromises()
    expect(wrapper.find('details').exists()).toBe(false)
    expect(wrapper.text()).toContain('/isolated/backups')
    expect(wrapper.findAll('tbody tr').map(r => r.attributes('data-node-id'))).toEqual(['.', '2026', '2026/10'])
    await wrapper.get('[aria-label="展开 10"]').trigger('click')
    await wrapper.get('[aria-label="打开目录 upgrade"]').trigger('click')
    expect(wrapper.emitted('open')?.[0]?.[0]).toMatchObject({ id: '2026/10/upgrade' })
    // Refresh keeps the user's expansion, rather than resetting the table.
    await wrapper.get('.backup-files-head button').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-node-id="2026/10/upgrade"]').exists()).toBe(true)
  })
  it('initializes at month when the first backup arrives after an empty root', async () => {
    query.mockResolvedValueOnce({ rootPath: '/isolated/backups', nodes: [node('.', 'backups')] })
    const wrapper = mount(BackupFiles)
    await flushPromises()
    expect(wrapper.findAll('tbody tr')).toHaveLength(1)
    await wrapper.get('.backup-files-head button').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('tbody tr').map(r => r.attributes('data-node-id'))).toEqual(['.', '2026', '2026/10'])
    await wrapper.get('[aria-label="收起 backups"]').trigger('click')
    await wrapper.get('.backup-files-head button').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('tbody tr')).toHaveLength(1)
  })
  it('shows no sample files on error; retry can populate a previously empty tree', async () => {
    query.mockRejectedValueOnce(new Error('secret/path/token'))
    const wrapper = mount(BackupFiles)
    await flushPromises()
    expect(wrapper.get('[role="alert"]').text()).toContain('无法读取备份文件')
    expect(wrapper.text()).not.toContain('secret/path/token')
    expect(wrapper.find('[data-node-id]').exists()).toBe(false)
    await wrapper.get('.tree-message button').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('tbody tr')).toHaveLength(3)
  })
  it('keeps the refresh control focusable while suppressing duplicate reads', async () => {
    const wrapper = mount(BackupFiles, { attachTo: document.body })
    await flushPromises()
    let finish!: (value: ReturnType<typeof result>) => void
    query.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const refresh = wrapper.get('.backup-files-head button')
    ;(refresh.element as HTMLButtonElement).focus()
    await refresh.trigger('click')
    expect(refresh.attributes('aria-disabled')).toBe('true')
    expect(refresh.attributes('aria-busy')).toBe('true')
    // Native disabled drops focus in macOS WebView; aria-disabled keeps the
    // same control while the request guard blocks mouse/keyboard re-entry.
    expect((refresh.element as HTMLButtonElement).disabled).toBe(false)
    expect(refresh.text()).toBe('正在读取…')
    await refresh.trigger('click')
    expect(query).toHaveBeenCalledTimes(2)
    expect(document.activeElement).toBe(refresh.element)
    finish(result())
    await flushPromises()
    expect(document.activeElement).toBe(refresh.element)
    expect(refresh.text()).toBe('刷新')
    expect(refresh.attributes('aria-disabled')).toBeUndefined()
    wrapper.unmount()
  })
})
