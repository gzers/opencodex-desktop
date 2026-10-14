import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRuntimeStore } from '@/features/runtime/store'
import { useAboutStore } from '@/features/about/store'
import { useDataRootStore } from '@/features/data-root/store'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
const pages: Array<ReturnType<typeof mount>> = []
beforeEach(() => invoke.mockReset())
afterEach(() => pages.splice(0).forEach(page => page.unmount()))

async function setup(platform: string, root: string, prefix: string) {
  const pinia = createPinia()
  setActivePinia(pinia)
  useAboutStore().app = { name: 'OpenCodeX Desktop', version: '0.1.10', platform,
    identifier: 'com.gzers.opencodex.desktop', framework: 'Tauri v2', license: 'MIT License' }
  const runtime = useRuntimeStore()
  runtime.source = { kind: 'managed', path: null, version: null, resolvedAt: null,
    managedEntry: null, managedPrefix: prefix, defaultPrefix: prefix,
    explicitPath: null, insideDataRoot: true, history: [] }
  useDataRootStore().config = { currentDataRoot: root, activeDataRoot: root,
    opencodexHome: root, currentOpencodexHome: root, opencodexHomeMode: 'inside', runtimeActive: true }
  invoke.mockImplementation(async command => {
    if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
    if (command === 'runtime_source') return runtime.source
    if (command === 'get_data_root_config') return useDataRootStore().config
    return null
  })
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] } })
  pages.push(page)
  await flushPromises()
  await page.get('[data-testid="runtime-install"]').trigger('click')
  const modal = page.get('[data-testid="runtime-install-modal"]')
  const next = () => modal.findAll('button').find(button => button.text() === '下一步')!
  return { modal, next }
}

describe('installation prefix platform syntax and landing hint', () => {
  it.each([
    ['C:\\Tools\\OpenCodeX\\data', 'C:\\Tools\\OpenCodeX\\data\\runtime\\opencodex'],
    ['F:\\Apps with spaces\\OpenCodeX\\data', 'F:\\Apps with spaces\\OpenCodeX\\data\\runtime\\opencodex'],
    ['D:\\Tools\\data', 'd:/tools/DATA/runtime/opencodex'],
    ['\\\\server\\share\\OpenCodeX\\data', '\\\\SERVER\\share\\opencodex\\DATA\\runtime'],
  ])('allows a fully qualified Windows prefix %s and only then advances', async (root, prefix) => {
    const { modal, next } = await setup('Windows', root, prefix)
    expect(next().attributes('disabled')).toBeUndefined()
    expect(modal.text()).toContain('落点：数据根内')
    expect(invoke.mock.calls.map(call => call[0])).not.toContain('install_runtime')
    await next().trigger('click')
    expect(modal.find('#wiz-prefix').exists()).toBe(false)
    expect(modal.text()).toContain('联网安装')
  })

  it.each([
    ['D:\\Tools\\data', 'D:\\Tools\\data-sibling\\runtime'],
    ['\\\\server\\share\\data', '\\\\server\\other-share\\data\\runtime'],
    ['/Users/fixture/data', '/Users/fixture/data-other/runtime'],
  ])('does not treat a string-prefix sibling as inside %s', async (root, prefix) => {
    const { modal } = await setup(root.startsWith('/') ? 'macOS' : 'Windows', root, prefix)
    expect(modal.text()).toContain('落点：数据根外')
  })

  it.each(['C:relative', '\\runtime', 'runtime', 'C:\\', '\\\\server\\share', '\\\\server',
    '\\\\?\\C:\\runtime', 'D:\\data\\..\\Windows', 'D:\\data\\.\\runtime'])
    ('keeps ambiguous, root or non-normal Windows path %s from advancing', async prefix => {
      const { next } = await setup('Windows', 'D:\\data', prefix)
      expect(next().attributes('disabled')).toBeDefined()
      expect(invoke.mock.calls.map(call => call[0])).not.toContain('install_runtime')
    })

  it.each(['/System/opencodex', '/usr/local/opencodex', '/', 'C:\\runtime'])
    ('retains POSIX protected/absolute checks for macOS %s', async prefix => {
      const { next } = await setup('macOS', '/Users/fixture/data', prefix)
      expect(next().attributes('disabled')).toBeDefined()
    })

  it('updates the syntax check when the host platform fact arrives without changing the selected prefix', async () => {
    const prefix = 'E:\\Applications\\OpenCodeX\\data\\runtime\\opencodex'
    const { modal, next } = await setup('unknown', 'E:\\Applications\\OpenCodeX\\data', prefix)
    expect(next().attributes('disabled')).toBeDefined()
    useAboutStore().app!.platform = 'Windows'
    await flushPromises()
    expect(next().attributes('disabled')).toBeUndefined()
    expect((modal.get('#wiz-prefix').element as HTMLInputElement).value).toBe(prefix)
  })

  it('passes the confirmed Windows path unchanged and displays a native permission rejection without retrying scripts', async () => {
    const prefix = 'E:\\Protected by host policy\\opencodex'
    const { modal, next } = await setup('Windows', 'D:\\OpenCodeX\\data', prefix)
    expect(next().attributes('disabled')).toBeUndefined()
    await next().trigger('click')
    await next().trigger('click')
    await modal.findAll('input[type="checkbox"]').at(-1)!.setValue(true)
    const read = invoke.getMockImplementation()!
    invoke.mockImplementation(async (command, ...args) => {
      if (command === 'install_runtime') throw { message: '安装位置落在系统保护目录内，已拒绝' }
      return read(command, ...args)
    })
    await next().trigger('click')
    await flushPromises()
    const requests = invoke.mock.calls.filter(([command]) => command === 'install_runtime')
    expect(requests).toHaveLength(1)
    expect(requests[0]![1].request).toMatchObject({ prefix, source: 'registry', allowScripts: false })
    expect(modal.get('[data-testid="wiz-error"]').text()).toContain('系统保护目录')
    expect(modal.get('[data-testid="wiz-status"]').text()).toContain('安装失败')
    expect(modal.find('[role="progressbar"]').exists()).toBe(false)
  })
})
