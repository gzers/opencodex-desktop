import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ExtensionsRoute from '@/routes/ExtensionsRoute.vue'
import { renderMarkdown } from '@/lib/markdown'
import type { ExtensionsDto, SkillDetailDto } from '@/features/extensions/api'
import { useExtensionsStore } from '@/features/extensions/store'

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

const skillDetail: SkillDetailDto = {
  name: 'design-studio',
  source: 'agents',
  sourcePath: '/fixtures/home/.agents/skills/design-studio',
  entryFile: 'SKILL.md',
  // 后端已剥离 frontmatter（name / description 由信息区展示），因此这里只断言正文渲染。
  body: '# 设计工作台\n\n- 步骤一\n- 步骤二\n\n> 引用行\n\n```json\n{"type": "stdio"}\n```\n\n**加粗**\n<img src=x onerror=alert(1)>\n',
  bodyTruncated: false,
  fileCount: 28,
  totalBytes: 421888,
  statsNote: null,
}

function extensions(): ExtensionsDto {
  return {
    skills: [
      {
        id: 'design-studio',
        name: 'design-studio',
        source: 'agents',
        updatedAt: '2026-09-13T00:00:00Z',
        description: '产品设计、UX 原型、应用界面草图、交互流程、结构化图表。'.repeat(4),
        clients: ['claude'],
      },
    ],
    servers: [
      {
        id: 'node_repl',
        name: 'node_repl',
        transport: 'stdio',
        command: '/opt/cua_node/bin/node_repl',
        args: [],
        envKeys: ['CODEX_HOME', 'NODE_REPL_NODE_PATH', 'SKY_CUA_SERVICE_PATH'],
        description: '本机 Node REPL 服务；只同步服务器声明。',
        clients: ['codex'],
        updatedAt: '2026-09-23T00:00:00Z',
      },
    ],
    sourceDir: '/fixtures/home/.agents/skills',
    sourceDirNotice: null,
    sourceDirCustom: false,
    syncMethod: 'symlink',
  }
}

function config(skills: string[]) {
  return {
    skills,
    servers: [],
    enablement: { claude: true, codex: true, gemini: true, grok: true, opencode: true, hermes: true },
    sourceDir: null,
    sourceDirCustom: false,
    syncMethod: 'symlink',
    revision: 1,
    fingerprint: 'fp',
    updatedAt: '2026-09-23T00:00:00Z',
    conflict: null,
    documentSha256: 'sha',
    backedUp: true,
  }
}

function mountRoute() {
  const pinia = createPinia()
  setActivePinia(pinia)
  useExtensionsStore().discovered = extensions()
  useExtensionsStore().config = config([]) as never
  return mount(ExtensionsRoute, { global: { plugins: [pinia] } })
}

describe('extension entry detail (AC-11 / CAP-11.5)', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockImplementation(async (command: string, payload?: Record<string, unknown>) => {
      if (command === 'read_skill_detail') return skillDetail
      if (command === 'read_mcp_detail') {
        return {
          name: 'node_repl',
          transport: 'stdio',
          command: '/opt/cua_node/bin/node_repl',
          args: [],
          envKeys: ['CODEX_HOME', 'NODE_REPL_NODE_PATH', 'SKY_CUA_SERVICE_PATH'],
          description: '本机 Node REPL 服务；只同步服务器声明。',
          clients: ['codex'],
          updatedAt: '2026-09-23T00:00:00Z',
          configJson: '{\n  "type": "stdio",\n  "command": "/opt/cua_node/bin/node_repl"\n}',
          landings: [
            { client: 'codex', configPath: '/fixtures/.codex/config.toml', key: 'mcp_servers', present: true },
          ],
        }
      }
      if (command === 'list_extensions') return extensions()
      if (command === 'execute_extension_write') {
        const request = payload?.command as { kind: string; name?: string }
        const linked = request.kind === 'link_skill' ? [request.name ?? ''] : []
        return { config: config(linked), skillsLinked: linked, mcpWritten: [] }
      }
      return null
    })
  })

  it('opens a read-only detail from the name + description cell and renders markdown', async () => {
    const wrapper = mountRoute()
    await nextTick()
    const cell = wrapper.find('.skill-info[data-ext-detail]')
    expect(cell.attributes('role')).toBe('button')
    expect(cell.attributes('tabindex')).toBe('0')
    expect(cell.attributes('aria-label')).toBe('查看 design-studio 完整信息')

    await cell.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_skill_detail', { name: 'design-studio' }))
    await nextTick()

    const modal = wrapper.find('.modal.ext-detail')
    expect(modal.exists()).toBe(true)
    expect(modal.find('h3').text()).toBe('design-studio')
    // 正文按 Markdown 渲染：标题、列表、引用、围栏代码都成为真实元素。
    expect(modal.find('.ext-detail-md h1').text()).toBe('设计工作台')
    expect(modal.findAll('.ext-detail-md ul > li').length).toBe(2)
    expect(modal.find('.ext-detail-md blockquote').text()).toContain('引用行')
    expect(modal.find('.ext-detail-md pre code').text()).toContain('"type": "stdio"')
    expect(modal.find('.ext-detail-md strong').text()).toBe('加粗')
    // 信息区包含源目录与统计；不套外框（两列网格）。
    const facts = modal.findAll('.ext-detail-facts > div').map(node => node.text())
    expect(facts.some(text => text.includes('源目录'))).toBe(true)
    expect(facts.some(text => text.includes('28 个文件'))).toBe(true)
    expect(facts.some(text => text.includes('KB'))).toBe(true)
    // 标签已写明「更新时间」，取值只给日期（不出现「更新时间 更新 …」）。
    const updated = modal.findAll('.ext-detail-facts > div').find(row => row.text().includes('更新时间'))
    expect(updated?.text()).toBe('更新时间2026-09-13')
    // 只有「关闭」一个底部动作，且没有原型脚注与区块小标题。
    expect(modal.find('.modal-actions').findAll('button').map(button => button.text())).toEqual(['关闭'])
    expect(wrapper.find('.modal-prototype-note').exists()).toBe(false)
    // 标题动作按原型用图标按钮：2 个、有 svg、无文字；卸载为危险色。
    const headButtons = modal.findAll('.modal-head-actions button')
    expect(headButtons.length).toBe(2)
    expect(headButtons.every(button => button.find('svg').exists())).toBe(true)
    expect(headButtons.map(button => button.text())).toEqual(['', ''])
    expect(headButtons[1].classes()).toContain('danger')
    // 描述只出现在正文面板里：标题下不再有独立描述段。
    expect(modal.find('.ext-detail-lead').exists()).toBe(false)
    expect(modal.find('.ext-detail-md').text()).toContain('产品设计、UX 原型、应用界面草图')
    // 源目录整行（原型 .span-2）。
    const sourceRow = modal.findAll('.ext-detail-facts > div').find(row => row.text().includes('源目录'))
    expect(sourceRow?.classes()).toContain('span-2')
    // 同步目标格按原型：li 外框 + 15px 图标 + 名字 + 纯文字状态（不用胶囊标签，避免挤掉名字）。
    const targetCells = wrapper.findAll('.ext-detail-targets li')
    expect(targetCells.length).toBe(6)
    expect(targetCells[0].classes()).toContain('on')
    expect(targetCells[0].find('svg.target-svg').exists()).toBe(true)
    expect(targetCells[0].find('em').text()).toBe('已同步')
    expect(targetCells[1].find('em').text()).toBe('未同步')
    expect(wrapper.findAll('.ext-detail-targets .tag').length).toBe(0)
    await wrapper.unmount()
  })

  it('renders detail content as escaped markup instead of executing inline HTML', () => {
    const html = renderMarkdown(skillDetail.body ?? '')
    expect(html).toContain('<h1>设计工作台</h1>')
    expect(html).not.toContain('<img')
    expect(html).toContain('&lt;img src=x onerror=alert(1)&gt;')
    // 链接只保留文字，不生成 href。
    expect(renderMarkdown('[官方文档](https://example.com)')).toBe('<p>官方文档</p>')
  })

  it('writes back to the list row when a client cell is toggled', async () => {
    const wrapper = mountRoute()
    await nextTick()
    await wrapper.find('.skill-info[data-ext-detail]').trigger('click')
    // 先等详情读取落地，再取目标格，避免在重渲染过程中点击到旧节点。
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_skill_detail', { name: 'design-studio' }))
    await nextTick()

    const cells = wrapper.findAll('.ext-detail-target')
    expect(cells.length).toBe(6)
    const codexCell = cells.find(cell => cell.text().includes('Codex'))
    expect(codexCell?.attributes('aria-pressed')).toBe('false')
    await codexCell!.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'execute_extension_write',
      expect.objectContaining({
        command: expect.objectContaining({
          kind: 'link_skill',
          name: 'design-studio',
          client: 'codex',
        }),
      }),
    ))
    // 连/断是逐客户端的：绝不能把整条 Skill 卸载掉。
    expect(
      invoke.mock.calls.filter(([command, payload]) =>
        command === 'execute_extension_write' &&
        (payload?.command as { kind?: string })?.kind === 'uninstall_skill',
      ),
    ).toHaveLength(0)
    await wrapper.unmount()
  })

  it('disconnects only the clicked client from the detail targets', async () => {
    const wrapper = mountRoute()
    await nextTick()
    await wrapper.find('.skill-info[data-ext-detail]').trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_skill_detail', { name: 'design-studio' }))
    await nextTick()

    const claudeCell = wrapper
      .findAll('.ext-detail-target')
      .find(cell => cell.text().includes('Claude'))!
    // design-studio 已落在 Claude：再点一次 = 断开这一个客户端的链接。
    await claudeCell.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'execute_extension_write',
      expect.objectContaining({
        command: expect.objectContaining({
          kind: 'unlink_skill',
          name: 'design-studio',
          client: 'claude',
          confirm: true,
        }),
      }),
    ))
    expect(invoke).not.toHaveBeenCalledWith(
      'execute_extension_write',
      expect.objectContaining({ command: expect.objectContaining({ kind: 'uninstall_skill' }) }),
    )
    await wrapper.unmount()
  })

  it('opens the detail with Enter / Space from the keyboard', async () => {
    const wrapper = mountRoute()
    await nextTick()
    await wrapper.find('.skill-info[data-ext-detail]').trigger('keydown', { key: 'Enter' })
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_skill_detail', { name: 'design-studio' }))
    await wrapper.unmount()
  })

  it('fixes the label column width so values line up in both dialogs', () => {
    // jsdom 不做布局，这里按样式表断言固定列宽规则存在（真实对齐由原型浏览器 QA 按像素测量）。
    // vitest 以 apps/desktop/ui 为工作目录；直接读样式源文件。
    const css = readFileSync(resolve(process.cwd(), 'src/styles/base.css'), 'utf8')
    expect(css).toMatch(/\.ext-detail-facts dt \{[^}]*width: 4em/)
  })

  it('keeps the MCP description inside the panel and scrolls the env row on its own line', async () => {
    const wrapper = mountRoute()
    await nextTick()
    const { useRouteStore } = await import('@/stores/routes')
    useRouteStore().go('extensions', { tab: 'mcp' })
    await nextTick()

    const cell = wrapper.find('.skill-info[data-ext-detail]')
    expect(cell.attributes('aria-label')).toBe('查看 node_repl 完整信息')
    await cell.trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('read_mcp_detail', { name: 'node_repl' }))
    await nextTick()

    const modal = wrapper.find('.modal.ext-detail')
    // 描述 + 「完整配置」标题 + JSON 都在面板内；标题动作是图标按钮。
    expect(modal.find('.ext-detail-lead').exists()).toBe(false)
    const panel = modal.find('.ext-detail-md').text()
    expect(panel).toContain('本机 Node REPL 服务')
    expect(panel).toContain('完整配置')
    expect(panel).toContain('"type": "stdio"')
    expect(modal.findAll('.modal-head-actions button').map(button => button.text())).toEqual(['', ''])
    // 环境变量单独整行，且取值区自带内部滚动。
    const envRow = modal.findAll('.ext-detail-facts > div').find(row => row.text().includes('环境变量'))
    expect(envRow?.classes()).toContain('span-2')
    expect(envRow!.find('dd').classes()).toContain('is-scrollable')
    expect(envRow!.find('dd').text()).toContain('CODEX_HOME')
    // 字段与顺序对齐原型：类型 / 版本 / 更新时间 / 命令 / 参数 / 环境变量 / 配置落点。
    expect(modal.findAll('.ext-detail-facts dt').map(node => node.text())).toEqual([
      '类型', '版本', '更新时间', '命令', '参数', '环境变量', '配置落点',
    ])
    const mcpUpdated = modal.findAll('.ext-detail-facts > div').find(row => row.text().includes('更新时间'))
    expect(mcpUpdated?.text()).toBe('更新时间2026-09-23')
    await wrapper.unmount()
  })
})
