import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ReleaseNotes from '@/features/updates/ReleaseNotes.vue'
import { getReleaseDetails, type ReleaseDetails } from '@/features/updates/releaseDetails'
import { useAppStore } from '@/stores/app'
import { useDataRootStore } from '@/features/data-root/store'
const mocks = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mocks.invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
let wrappers: VueWrapper[] = []
function details(overrides: Partial<ReleaseDetails> = {}): ReleaseDetails {
  return { target: 'official', channel: 'official', candidateVersion: '2.84.0', notesState: 'ready',
    notes: '真实版本发布说明', notesSource: 'github_release', releaseUrl: 'https://github.com/lidge-jun/opencodex/releases/tag/v2.84.0',
    fetchedAt: 100, publishedAt: '2026-10-14T00:00:00Z', cached: false, ...overrides }
}
function render(props = {}) {
  const wrapper = mount(ReleaseNotes, { props: { target: 'official', channel: 'official', version: '2.84.0', ...props }, global: { plugins: [createPinia()] }, attachTo: document.body })
  wrappers.push(wrapper); return wrapper
}
beforeEach(() => { mocks.invoke.mockReset(); setActivePinia(createPinia()); mocks.invoke.mockResolvedValue(details()) })
afterEach(() => { wrappers.forEach(wrapper => wrapper.unmount()); wrappers = []; document.body.innerHTML = ''; vi.restoreAllMocks() })

describe('exact candidate release notes', () => {
  it('shows loading then literal scrollable notes and opens only the verified release page', async () => {
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const wrapper = render(); expect(wrapper.text()).toContain('正在加载')
    finish(details({ notes: '<img src=x onerror=evil()>'.repeat(400) })); await flushPromises()
    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.get('pre').attributes('tabindex')).toBe('0')
    expect(wrapper.get('pre').text()).toContain('<img src=x onerror=evil()>')
    const app = useAppStore(); const open = vi.spyOn(app, 'openExternalLink').mockResolvedValue(true)
    await wrapper.get('a').trigger('click')
    expect(open).toHaveBeenCalledWith('https://github.com/lidge-jun/opencodex/releases/tag/v2.84.0')
    expect(mocks.invoke).toHaveBeenCalledOnce()
    expect(mocks.invoke.mock.calls[0][0]).toBe('update_release_details')
  })
  it('missing content is distinct from transport failure and retries only the notes command', async () => {
    mocks.invoke.mockResolvedValueOnce(details({ notesState: 'failed', notes: null, releaseUrl: null }))
    const wrapper = render(); await flushPromises()
    expect(wrapper.text()).toContain('获取失败')
    await wrapper.get('button').trigger('click'); await flushPromises()
    expect(mocks.invoke).toHaveBeenCalledTimes(2)
    expect(mocks.invoke.mock.calls[1]).toEqual(['update_release_details', { target: 'official', channel: 'official', candidateVersion: '2.84.0', force: true }])
    expect(wrapper.text()).toContain('真实版本发布说明')
    expect(mocks.invoke.mock.calls.every(call => call[0] === 'update_release_details')).toBe(true)
  })
  it('actual publisher absence keeps a verified page and does not invent a failure', async () => {
    mocks.invoke.mockResolvedValueOnce(details({ notesState: 'unavailable', notes: null }))
    const wrapper = render(); await flushPromises()
    expect(wrapper.text()).toContain('尚未提供本版本更新说明')
    expect(wrapper.text()).not.toContain('获取失败')
    expect(wrapper.text()).not.toContain('重试说明')
    expect(wrapper.get('a').text()).toContain('查看官方包页面')
  })
  it('replacing the focused retry preserves focus inside the notes area', async () => {
    mocks.invoke.mockResolvedValueOnce(details({ notesState: 'failed', notes: null, releaseUrl: null }))
    const wrapper = render(); await flushPromises()
    const retry = wrapper.get('button'); (retry.element as HTMLButtonElement).focus()
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    await retry.trigger('click'); await flushPromises()
    expect(wrapper.element.contains(document.activeElement)).toBe(true)
    finish(details()); await flushPromises()
    expect(wrapper.element.contains(document.activeElement)).toBe(true)
  })
  it('a late retry never steals focus from another control', async () => {
    mocks.invoke.mockResolvedValueOnce(details({ notesState: 'failed', notes: null, releaseUrl: null }))
    const wrapper = render(); await flushPromises()
    const retry = wrapper.get('button'); (retry.element as HTMLButtonElement).focus()
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    await retry.trigger('click')
    const other = document.createElement('button'); document.body.append(other); other.focus()
    finish(details()); await flushPromises()
    expect(document.activeElement).toBe(other)
  })
  it('retains updater text when independent detail lookup fails', async () => {
    mocks.invoke.mockRejectedValueOnce(new Error('offline'))
    const wrapper = render({ target: 'manager', channel: 'stable', version: '0.1.10', updaterNotes: 'updater正文' }); await flushPromises()
    expect(wrapper.get('pre').text()).toBe('updater正文')
    expect(wrapper.text()).toContain('获取失败')
    expect(wrapper.text()).toContain('重试说明')
  })
  it('late old-version results cannot replace the selected candidate', async () => {
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const wrapper = render()
    mocks.invoke.mockResolvedValueOnce(details({ candidateVersion: '2.85.0', notes: '新版正文', releaseUrl: null }))
    await wrapper.setProps({ version: '2.85.0' }); await flushPromises()
    expect(wrapper.text()).toContain('新版正文')
    finish(details({ notes: '旧版迟到正文' })); await flushPromises()
    expect(wrapper.text()).not.toContain('旧版迟到正文')
  })
  it('closing an observer ignores late results and a root freeze prevents new queries', async () => {
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const wrapper = render(); await flushPromises(); wrapper.unmount(); wrappers = []
    finish(details()); await flushPromises()
    const pinia = createPinia(); setActivePinia(pinia); useDataRootStore().pendingRestart = true
    const frozen = mount(ReleaseNotes, { props: { target: 'official', channel: 'official', version: '2.84.0' }, global: { plugins: [pinia] } }); wrappers.push(frozen)
    await flushPromises(); expect(frozen.text()).toContain('请重启')
    expect(mocks.invoke).toHaveBeenCalledOnce()
  })
  it.each(['javascript:evil()', 'https://github.com.evil.example/x/releases/tag/v2.84.0', 'https://u:p@github.com/x/releases/tag/v2.84.0', 'https://github.com/x/releases/tag/v2.83.0', 'https://github.com/x/releases/tag/v2.84.0?x=1'])('rejects unsafe or different-version page %s', async releaseUrl => {
    mocks.invoke.mockResolvedValueOnce(details({ releaseUrl }))
    const wrapper = render(); await flushPromises(); expect(wrapper.find('button').exists()).toBe(false)
  })
  it('does not request notes without a verified candidate', async () => {
    const wrapper = render({ version: null }); await flushPromises()
    expect(wrapper.text()).toContain('尚无可核验'); expect(mocks.invoke).not.toHaveBeenCalled()
  })
  it('single-flight coalesces the same object/root while different roots remain independent', async () => {
    let finish!: (value: ReleaseDetails) => void
    mocks.invoke.mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const a = getReleaseDetails('official', 'official', '2.84.0', 'root-a')
    expect(getReleaseDetails('official', 'official', '2.84.0', 'root-a', true)).toBe(a)
    const b = getReleaseDetails('official', 'official', '2.84.0', 'root-b')
    expect(mocks.invoke).toHaveBeenCalledTimes(2)
    finish(details()); await Promise.all([a, b])
  })
})
