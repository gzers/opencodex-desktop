import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()
const dialogOpen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

// 目录选择交给系统原生对话框；这里只验证调用契约，不测系统 UI。
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => dialogOpen(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

const SKILLS_SOURCE =
  '/Users/example/Library/Application Support/com.gzers.opencodex.desktop/manager-state/skills-store'

function mountSettingsExtensions() {
  const wrapper = mount(App, { global: { plugins: [createPinia()] } })
  useRouteStore().go('settings', { section: 'extensions' })
  return wrapper
}

async function flush(wrapper: VueWrapper) {
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
}

describe('settings · Skills 源目录 row', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    dialogOpen.mockReset()
    invoke.mockImplementation((command: string) => {
      if (command === 'get_managed_path_targets') {
        return Promise.resolve({
          managed: [
            { key: 'manager_state', path: '/data/manager-state', label: '管理器设置目录' },
          ],
          agent: [
            { kind: 'skills-source', client: 'agents', label: 'Skills 源目录', path: SKILLS_SOURCE },
            { kind: 'skills', client: 'codex', label: 'Codex Skills', path: '/home/.codex/skills' },
            { kind: 'mcp', client: 'codex', label: 'Codex MCP 配置', path: '/home/.codex/config.toml' },
          ],
        })
      }
      if (command === 'extension_config') {
        return Promise.resolve({
          skills: [],
          servers: [],
          enablement: {
            claude: true,
            codex: true,
            gemini: true,
            grok: true,
            opencode: true,
            hermes: true,
          },
          sourceDir: null,
          sourceDirCustom: false,
          syncMethod: 'symlink',
          revision: 1,
          fingerprint: 'a'.repeat(64),
          updatedAt: '2026-09-20T00:00:00Z',
          conflict: null,
          documentSha256: 'b'.repeat(64),
          backedUp: false,
        })
      }
      if (command === 'list_extensions') {
        return Promise.resolve({
          skills: [],
          servers: [],
          sourceDir: SKILLS_SOURCE,
          sourceDirNotice: null,
          sourceDirCustom: false,
          syncMethod: 'symlink',
        })
      }
      return Promise.resolve(undefined)
    })
    document.body.innerHTML = ''
  })

  // 回归：路径条此前被塞进右侧 .controls，长路径把「打开 / 选择」挤到错位。
  // 原型把 .cli-copy-btn 作为 .setting-row 的直接子元素（grid-column:1/-1）独占整行。
  it('keeps the path bar on its own full-width row instead of inside the controls', async () => {
    const wrapper = mountSettingsExtensions()
    await flush(wrapper)

    const sourceRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === 'Skills 源目录')
    expect(sourceRow, '未找到 Skills 源目录行').toBeTruthy()

    const bar = sourceRow!.find(':scope > .cli-copy-btn.skills-source')
    expect(bar.exists()).toBe(true)
    expect(bar.find('code').text()).toBe(SKILLS_SOURCE)

    // 仍不允许出现在右侧操作区。
    expect(sourceRow!.find('.controls .cli-copy-btn').exists()).toBe(false)
    // 「打开 / 选择自定义目录」保持在右侧操作区（源目录是配置值，允许自定义）。
    expect(sourceRow!.findAll('.controls .btn').map((btn) => btn.text())).toEqual([
      '打开',
      '选择自定义目录',
    ])
  })

  it('drives the sync method from the extension config, not from preferences', async () => {
    const wrapper = mountSettingsExtensions()
    await flush(wrapper)

    const methodRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === '同步方式')
    expect(methodRow, '未找到同步方式行').toBeTruthy()
    // 默认（软链接优先）来自 extension_config 的 syncMethod。
    expect(methodRow!.find('.select-value').text()).toBe('软链接优先')
    // 「测试同步」已删除，改为「检查并修正」。
    expect(methodRow!.find('.controls .btn').text()).toBe('检查并修正')
    expect(wrapper.text()).not.toContain('测试同步')

    // 选择「文件复制」走扩展写入命令，并带上扩展配置的快照。
    await methodRow!.findAll('.select-option')[1].trigger('click')
    await flush(wrapper)
    const write = invoke.mock.calls.find((call) => call[0] === 'execute_extension_write')
    expect(write).toBeTruthy()
    expect(write![1]).toEqual({ command: { kind: 'set_sync_method', method: 'copy' } })
  })

  it('marks the source directory as custom and offers restoring the default', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'get_managed_path_targets') {
        return Promise.resolve({ managed: [], agent: [
          { kind: 'skills-source', client: 'agents', label: 'Skills 源目录', path: '/custom/skills' },
        ] })
      }
      if (command === 'extension_config') {
        return Promise.resolve({
          skills: [],
          servers: [],
          enablement: {},
          sourceDir: '/custom/skills',
          sourceDirCustom: true,
          syncMethod: 'copy',
          revision: 2,
          fingerprint: 'a'.repeat(64),
          updatedAt: '2026-09-20T00:00:00Z',
          conflict: null,
          documentSha256: 'b'.repeat(64),
          backedUp: false,
        })
      }
      if (command === 'list_extensions') {
        return Promise.resolve({
          skills: [], servers: [],
          sourceDir: '/custom/skills', sourceDirNotice: null, sourceDirCustom: true,
          syncMethod: 'copy',
        })
      }
      return Promise.resolve(undefined)
    })
    const wrapper = mountSettingsExtensions()
    await flush(wrapper)

    const sourceRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === 'Skills 源目录')
    expect(sourceRow!.findAll('.controls .btn').map((btn) => btn.text())).toEqual([
      '打开',
      '选择自定义目录',
      '恢复默认',
    ])
    const methodRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === '同步方式')
    expect(methodRow!.find('.select-value').text()).toBe('文件复制')
  })

  // 回归：不再自造目录浏览器，改为调用系统原生目录选择器；
  // 选中的路径仍走扩展写入命令，由后端做 R-23 校验。
  it('opens the native folder picker and writes the chosen directory', async () => {
    dialogOpen.mockResolvedValue('/custom/agent-skills')
    const wrapper = mountSettingsExtensions()
    await flush(wrapper)

    const sourceRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === 'Skills 源目录')
    await sourceRow!.findAll('.controls .btn')[1].trigger('click')
    await flush(wrapper)

    expect(dialogOpen).toHaveBeenCalledTimes(1)
    expect(dialogOpen.mock.calls[0][0]).toMatchObject({ directory: true, multiple: false })
    const write = invoke.mock.calls.find((call) => call[0] === 'execute_extension_write')
    expect(write).toBeTruthy()
    expect(write![1]).toEqual({
      command: { kind: 'set_source_dir', path: '/custom/agent-skills' },
    })
  })

  it('keeps the current source directory when the native picker is cancelled', async () => {
    dialogOpen.mockResolvedValue(null)
    const wrapper = mountSettingsExtensions()
    await flush(wrapper)

    const sourceRow = wrapper
      .findAll('.setting-row')
      .find((row) => row.find('.setting-title').text() === 'Skills 源目录')
    await sourceRow!.findAll('.controls .btn')[1].trigger('click')
    await flush(wrapper)

    expect(dialogOpen).toHaveBeenCalledTimes(1)
    expect(invoke.mock.calls.find((call) => call[0] === 'execute_extension_write')).toBeUndefined()
  })
})
