import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ExtensionsRoute from '@/routes/ExtensionsRoute.vue'
import type { ExtensionsDto } from '@/features/extensions/api'
import {
  buildClientDirectory,
  clientAdaptation,
  clientStatusTag,
  filterClients,
} from '@/features/extensions/clients'
import { useExtensionsStore } from '@/features/extensions/store'
import { useAppStore } from '@/stores/app'
import { usePathsStore } from '@/app/paths/store'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function extensions(): ExtensionsDto {
  return {
    skills: [
      {
        id: 'design-studio',
        name: 'design-studio',
        source: 'agents',
        updatedAt: '2026-09-23T00:00:00Z',
        description: '设计',
        clients: ['codex', 'claude'],
      },
    ],
    servers: [
      {
        id: 'node_repl',
        name: 'node_repl',
        transport: 'stdio',
        command: 'node',
        args: [],
        envKeys: [],
        description: 'node',
        clients: ['codex'],
        updatedAt: null,
      },
    ],
    sourceDir: '/fixtures/.agents/skills',
    sourceDirNotice: null,
    sourceDirCustom: false,
    syncMethod: 'symlink',
  }
}

function mountClients() {
  window.location.hash = '#extensions?tab=clients'
  const pinia = createPinia()
  setActivePinia(pinia)
  useExtensionsStore().discovered = extensions()
  usePathsStore().agent = [
    { kind: 'skills', client: 'codex', label: 'Codex Skills', path: '/home/.codex/skills' },
    { kind: 'mcp', client: 'codex', label: 'Codex MCP 配置', path: '/home/.codex/config.toml' },
    { kind: 'skills', client: 'claude', label: 'Claude Skills', path: '/home/.claude/skills' },
    { kind: 'mcp', client: 'claude', label: 'Claude MCP 配置', path: '/home/.claude.json' },
  ]
  return mount(ExtensionsRoute, { global: { plugins: [pinia] } })
}

describe('extensions clients projection', () => {
  const entries = buildClientDirectory({
    skills: extensions().skills,
    servers: extensions().servers,
    skillsPaths: { codex: '/home/.codex/skills', claude: '/home/.claude/skills' },
    mcpPaths: { codex: '/home/.codex/config.toml', claude: '/home/.claude.json' },
    enablement: { codex: true, claude: false },
  })

  it('lists the same six client targets the extension contract already writes to', () => {
    expect(entries.map(entry => entry.id)).toEqual([
      'codex', 'claude', 'gemini', 'grok', 'opencode', 'hermes',
    ])
    expect(entries.find(entry => entry.id === 'codex')?.name).toBe('Codex')
  })

  it('counts managed resources per client instead of inventing new ones', () => {
    const codex = entries.find(entry => entry.id === 'codex')!
    expect(codex.managed.map(item => item.key)).toEqual(['skill:design-studio', 'mcp:node_repl'])
    expect(codex.skillsReady).toBe(true)
    expect(codex.mcpReady).toBe(true)
    expect(clientStatusTag(codex)).toEqual({ label: '可管理', tone: 'running' })
    expect(clientAdaptation(codex)).toBe('可管理')
  })

  it('falls back to the prototype placeholder wording when the software has no data', () => {
    const gemini = entries.find(entry => entry.id === 'gemini')!
    expect(gemini.skillsPath).toBeNull()
    expect(gemini.capability).toBe('unknown')
    expect(clientStatusTag(gemini)).toEqual({ label: '待核实适配', tone: 'warn' })
    expect(clientAdaptation(gemini)).toBe('待核实扩展适配')
  })

  it('reports a disabled client that still has landed resources as an exception', () => {
    const claude = entries.find(entry => entry.id === 'claude')!
    expect(claude.issue).toBe(true)
    expect(clientStatusTag(claude)).toEqual({ label: '有异常', tone: 'bad' })
    expect(clientAdaptation(claude)).toBe('已停用，仍有受管资源')
  })

  it('filters by the prototype status options and search box', () => {
    expect(filterClients(entries, '', 'installed').map(entry => entry.id)).toEqual(['codex', 'claude'])
    expect(filterClients(entries, '', 'issues').map(entry => entry.id)).toEqual(['claude'])
    expect(filterClients(entries, '', 'unknown').map(entry => entry.id)).toEqual([
      'gemini', 'grok', 'opencode', 'hermes',
    ])
    expect(filterClients(entries, 'claude', 'all').map(entry => entry.id)).toEqual(['claude'])
  })
})

describe('extensions clients section', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
    window.location.hash = ''
  })

  it('adds the third tab and does not disturb Skills / MCP', async () => {
    const wrapper = mountClients()
    await nextTick()
    const tabs = wrapper.findAll('.ext-tabs button').map(tab => tab.text())
    expect(tabs).toEqual(['Skills', 'MCP', '客户端'])
    expect(wrapper.find('.ext-tabs button.active').text()).toBe('客户端')
    await wrapper.unmount()
  })

  it('renders the four columns with aligned single-line rows', async () => {
    const wrapper = mountClients()
    await nextTick()
    const head = wrapper.findAll('.ext-client-head span').map(node => node.text())
    expect(head).toEqual(['客户端', '扩展能力', '受管资源', '状态'])
    const rows = wrapper.findAll('.ext-client-row')
    expect(rows).toHaveLength(6)
    // 每一行都是四个单元格（客户端 / 扩展能力 / 受管资源 / 状态）。
    expect(rows[0].findAll(':scope > *')).toHaveLength(4)
    expect(rows[0].find('.ext-client-cap').text()).toContain('Skills · 文档有依据')
    expect(rows[0].find('.ext-client-count').text()).toBe('2 项')
    expect(rows[0].find('.ext-client-state .state-pill').text()).toBe('可管理')
    await wrapper.unmount()
  })

  it('opens the client detail skeleton and goes back', async () => {
    const wrapper = mountClients()
    await nextTick()
    await wrapper.findAll('.ext-client-link')[0].trigger('click')
    await nextTick()
    const facts = wrapper.findAll('.ext-client-facts dt').map(node => node.text())
    expect(facts).toEqual(['范围', 'Skills', 'MCP', '适配状态'])
    expect(wrapper.find('.ext-client-facts dd').text()).toBe('用户级')
    expect(wrapper.text()).toContain('/home/.codex/skills')
    expect(wrapper.text()).toContain('受管资源 2 项')

    await wrapper.find('.ext-client-back').trigger('click')
    await nextTick()
    expect(wrapper.findAll('.ext-client-row')).toHaveLength(6)
    await wrapper.unmount()
  })

  it('jumps from the client detail to the Skills tab with the client filter applied', async () => {
    const wrapper = mountClients()
    await nextTick()
    await wrapper.findAll('.ext-client-link')[0].trigger('click')
    await nextTick()
    const skillsButton = wrapper.findAll('.ext-clients-actions button').find(button => button.text() === '查看 Skills')
    expect(skillsButton).toBeTruthy()
    await skillsButton!.trigger('click')
    await nextTick()
    expect(wrapper.find('.ext-tabs button.active').text()).toBe('Skills')
    expect(wrapper.find('.skills-chip.active').text()).toContain('Codex')
    await wrapper.unmount()
  })

  it('refreshes discovery and path projection from 发现客户端', async () => {
    const wrapper = mountClients()
    await nextTick()
    const discover = wrapper.findAll('.ext-clients-head button').find(button => button.text() === '发现客户端')!
    // 首屏会先读一次统一配置；按钮在读取期间禁用，等它可用再点。
    await vi.waitFor(() => expect(discover.attributes('disabled')).toBeUndefined())
    await discover.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('get_managed_path_targets'))
    expect(invoke).toHaveBeenCalledWith('extension_config')
    await nextTick()
    expect(wrapper.find('.ext-clients-note').text()).toContain('已发现')
    await wrapper.unmount()
  })

  it('filters the table through the prototype status menu', async () => {
    const wrapper = mountClients()
    await nextTick()
    expect(wrapper.findAll('.ext-client-row')).toHaveLength(6)
    await wrapper.find('.ext-clients-filter .select-trigger').trigger('click')
    await nextTick()
    // 弹层经 Teleport 到 body：按文案选出「待核实适配」。
    const option = Array.from(document.querySelectorAll<HTMLButtonElement>('.select-menu-portal .select-option'))
      .find(node => node.textContent?.trim() === '待核实适配')
    expect(option).toBeTruthy()
    option!.click()
    await nextTick()
    const rows = wrapper.findAll('.ext-client-row')
    expect(rows).toHaveLength(4)
    expect(rows.every(row => row.find('.ext-client-state .state-pill').text() === '待核实适配')).toBe(true)
    await wrapper.unmount()
  })

  it('opens the read-only 配置说明 modal with the same facts', async () => {
    const wrapper = mountClients()
    await nextTick()
    await wrapper.findAll('.ext-client-link')[0].trigger('click')
    await nextTick()
    const config = wrapper.findAll('.ext-clients-head button').find(button => button.text() === '配置说明')!
    await config.trigger('click')
    const modal = useAppStore().modal
    expect(modal?.title).toBe('Codex · 配置说明')
    expect(modal?.body).toContain('能力依据')
    expect(modal?.body).toContain('/home/.codex/skills')
    expect(modal?.body).toContain('/home/.codex/config.toml')
    await wrapper.unmount()
  })

  it('opens the entry detail from 管理目标 instead of adding a second dispatch path', async () => {
    const wrapper = mountClients()
    await nextTick()
    await wrapper.findAll('.ext-client-link')[0].trigger('click')
    await nextTick()
    const manage = wrapper.findAll('.ext-client-row-detail button').find(button => button.text() === '管理目标')!
    await manage.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_skill_detail', { name: 'design-studio' }))
    // 详情弹窗由 ExtensionDetailModal 打开；标题就是被管理的条目。
    expect(wrapper.find('.ext-detail h3').text()).toBe('design-studio')
    await wrapper.unmount()
  })
})
