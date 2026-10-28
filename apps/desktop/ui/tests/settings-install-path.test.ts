import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRuntimeStore } from '@/features/runtime/store'
import { useAboutStore } from '@/features/about/store'
import { useDataRootStore } from '@/features/data-root/store'
import { useRouteStore } from '@/stores/routes'
import { displayInstallPrefix } from '@/features/runtime/installPath'

const invoke = vi.fn()
const selectDirectory = vi.hoisted(() => vi.fn(async (): Promise<string | null> => null))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: selectDirectory }))
const pages: Array<ReturnType<typeof mount>> = []
beforeEach(() => { invoke.mockReset(); selectDirectory.mockReset(); selectDirectory.mockResolvedValue(null) })
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
    ['\\\\?\\D:\\Software\\OpenCodeX Desktop\\data/runtime/opencodex', 'D:\\Software\\OpenCodeX Desktop\\data\\runtime\\opencodex'],
    ['\\\\?\\UNC\\server\\share\\OpenCodeX Desktop\\runtime', '\\\\server\\share\\OpenCodeX Desktop\\runtime'],
  ])('displays the same readable identity on prefill and reset: %s', async (prefix, expected) => {
    const { modal } = await setup('Windows', 'D:\\Software\\OpenCodeX Desktop\\data', prefix)
    expect((modal.get('#wiz-prefix').element as HTMLInputElement).value).toBe(expected)
    await modal.get('#wiz-prefix').setValue('relative')
    await modal.findAll('button').find(button => button.text() === '恢复默认')!.trigger('click')
    expect((modal.get('#wiz-prefix').element as HTMLInputElement).value).toBe(expected)
  })

  it('preserves spaces, device identities and extended long paths', () => {
    for (const prefix of ['\\\\?\\GLOBALROOT\\Device\\HarddiskVolume1', '\\\\?\\D:\\trailing.\\runtime',
      '\\\\?\\D:\\' + 'long directory\\'.repeat(25) + 'runtime', '\\\\?\\D:\\NUL\\runtime']) {
      expect(displayInstallPrefix(prefix, 'Windows')).toBe(prefix)
    }
    expect(displayInstallPrefix('D:\\My App\\runtime', 'Windows')).toBe('D:\\My App\\runtime')
    expect(displayInstallPrefix('/Users/My App/runtime', 'macOS')).toBe('/Users/My App/runtime')
  })

  it('uses the same readable conversion after selecting a directory', async () => {
    const { modal } = await setup('Windows', 'D:\\data', 'D:\\data\\runtime')
    selectDirectory.mockResolvedValue('\\\\?\\D:\\My App\\Selected Folder/runtime')
    await modal.findAll('button').find(button => button.text() === '选择…')!.trigger('click')
    await flushPromises()
    expect((modal.get('#wiz-prefix').element as HTMLInputElement).value).toBe('D:\\My App\\Selected Folder\\runtime')
  })
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
    '\\\\?\\GLOBALROOT\\Device\\HarddiskVolume1', 'D:\\data\\..\\Windows', 'D:\\data\\.\\runtime'])
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

  it.each([
    ' \\\\?\\D:\\Applications\\OpenCodeX\\data\\runtime\\opencodex',
    '\\\\?\\UNC\\server\\share\\OpenCodeX\\data\\runtime\\opencodex',
  ])('accepts a Windows extended path returned by the host for %s', async prefix => {
    const { next } = await setup('Windows', 'D:\\Applications\\OpenCodeX\\data', prefix.trim())
    expect(next().attributes('disabled')).toBeUndefined()
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
    expect(modal.get('[data-testid="wiz-allow-scripts"]').attributes('disabled')).toBeDefined()
  })

  it('permits exactly the explicit scripts-required retry from the real numeric error envelope', async () => {
    const { modal, next } = await setup('Windows', 'D:\\data', 'D:\\data\\runtime\\opencodex')
    await next().trigger('click')
    await next().trigger('click')
    await modal.findAll('input[type="checkbox"]').at(-1)!.setValue(true)
    const read = invoke.getMockImplementation()!
    invoke.mockImplementation(async (command, ...args) => {
      if (command === 'install_runtime') throw { code: 14, message: 'managed runtime operation failed: scripts_required; 该包需要执行安装脚本才能生成入口；需要你的二次确认' }
      return read(command, ...args)
    })
    await next().trigger('click')
    await flushPromises()
    expect(invoke.mock.calls.filter(([name]) => name === 'install_runtime')).toHaveLength(1)
    const retry = modal.get('[data-testid="wiz-allow-scripts"]')
    expect(retry.attributes('disabled')).toBeUndefined()
    await retry.trigger('click')
    await flushPromises()
    const writes = invoke.mock.calls.filter(([name]) => name === 'install_runtime')
    expect(writes).toHaveLength(2)
    expect(writes[0]![1].request.allowScripts).toBe(false)
    expect(writes[1]![1].request.allowScripts).toBe(true)
    await modal.findAll('button').find(b => b.text() === '关闭')!.trigger('click')
    await pages.at(-1)!.get('[data-testid="runtime-install"]').trigger('click')
    expect(useRuntimeStore().installing).toBe(false)
  })
})
