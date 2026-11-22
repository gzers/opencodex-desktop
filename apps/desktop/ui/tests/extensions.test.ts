import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  getExtensionConfig,
  listExtensions,
  type ExtensionConfigDto,
  type ExtensionsDto,
} from '@/features/extensions/api'
import {
  extensionChips,
  formatUpdatedAt,
  serverVersion,
  skillSourceLabels,
  skillVersion,
  targetEnabled,
} from '@/features/extensions/presentation'
import { projectExtensions } from '@/features/extensions/projection'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function dto(overrides: Partial<ExtensionsDto> = {}): ExtensionsDto {
  return {
    skills: [
      {
        id: 'design-studio',
        name: 'design-studio',
        source: 'agents',
        updatedAt: '2026-09-15T00:00:00Z',
        description: '设计',
        clients: ['codex'],
      },
    ],
    servers: [
      {
        id: 'node_repl',
        name: 'node_repl',
        transport: 'stdio',
        command: 'node',
        args: ['--foo'],
        envKeys: ['TOKEN'],
        description: 'node --foo',
        clients: ['codex'],
        updatedAt: '2026-09-15T00:00:00Z',
      },
    ],
    sourceDir: '/Users/example/.agents/skills',
    sourceDirNotice: null,
    sourceDirCustom: false,
    syncMethod: 'symlink',
    ...overrides,
  }
}


function configDto(overrides: Partial<ExtensionConfigDto> = {}): ExtensionConfigDto {
  return {
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
    updatedAt: '2026-09-16T00:00:00Z',
    conflict: null,
    documentSha256: 'b'.repeat(64),
    backedUp: false,
    ...overrides,
  }
}

describe('extensions command contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('invokes the frozen read-only discovery command', async () => {
    const payload = dto()
    invoke.mockResolvedValue(payload)
    await expect(listExtensions()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('list_extensions')
  })

  it('loads the unified extension config projection', async () => {
    const payload = configDto()
    invoke.mockResolvedValue(payload)
    await expect(getExtensionConfig()).resolves.toEqual(payload)
    expect(invoke).toHaveBeenCalledWith('extension_config')

    const app = useAppStore()
    await app.loadExtensionConfig()
    expect(app.extensionConfig).toEqual(payload)
    expect(app.extensionConfigError).toBe(false)
  })

  it('executes one unified extension write and refreshes discovery', async () => {
    const initial = configDto()
    invoke.mockResolvedValueOnce(initial)
    const app = useAppStore()
    await app.loadExtensionConfig()

    const updated = configDto({
      revision: 2,
      enablement: { ...initial.enablement, codex: false },
      documentSha256: 'c'.repeat(64),
    })
    invoke.mockResolvedValueOnce({ config: updated, skillsLinked: [], mcpWritten: [] })
    invoke.mockResolvedValueOnce(dto())
    await app.toggleExtensionClient('codex', false)
    expect(app.extensionConfig).toEqual(updated)
    expect(invoke).toHaveBeenNthCalledWith(2, 'execute_extension_write', {
      command: { kind: 'toggle_client', client: 'codex', enabled: false },
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'list_extensions')

    invoke.mockRejectedValueOnce(new Error('external modified'))
    await expect(app.toggleExtensionClient('codex', true)).resolves.toBe(false)
    expect(app.extensionConfigError).toBe(true)
    expect(app.extensionConfig).toEqual(updated)
  })

  it('loads extensions into the shared store', async () => {
    const payload = dto()
    invoke.mockResolvedValue(payload)
    const app = useAppStore()
    await app.loadExtensions()
    expect(app.extensions).toEqual(projectExtensions(payload))
    expect(app.extensionsError).toBe(false)
    expect(app.extensionsLoading).toBe(false)
  })

  it('preserves the previous result when IPC fails', async () => {
    invoke.mockResolvedValueOnce(dto())
    const app = useAppStore()
    await app.loadExtensions()
    invoke.mockRejectedValueOnce(new Error('backend unavailable'))
    await app.loadExtensions()
    expect(app.extensions).toEqual(projectExtensions(dto()))
    expect(app.extensionsError).toBe(true)
    expect(app.extensionsLoading).toBe(false)
  })

  it('projects client counts, labels, and target states', () => {
    const payload = dto()
    const chips = extensionChips('已安装', payload.skills.length, payload.skills)
    expect(chips.map(chip => chip.value)).toEqual([1, 0, 1, 0, 0, 0, 0])
    expect(skillSourceLabels.agents).toBe('本地')
    expect(skillVersion(payload.skills[0])).toBe('本地源')
    expect(serverVersion(payload.servers[0])).toBe('本机配置')
    expect(formatUpdatedAt(payload.skills[0].updatedAt)).toContain('更新')
    expect(targetEnabled(payload.skills[0].clients, 'codex')).toBe(true)
    expect(targetEnabled(payload.skills[0].clients, 'claude')).toBe(false)
  })

  it('keeps empty state separate from failure', () => {
    expect(projectExtensions(dto({ skills: [], servers: [] }))).toEqual({ skills: [], servers: [] })
    expect(projectExtensions(null)).toBeNull()
  })
})

  it('sends base64 archive bytes and archive name for skill import', async () => {
    const updated = configDto({ revision: 2, skills: ['sample'] })
    invoke.mockResolvedValueOnce({ config: updated, skillsLinked: [], mcpWritten: [] })
    invoke.mockResolvedValueOnce(dto({ skills: [] }))
    const app = useAppStore()
    await app.executeExtensionWrite({
      kind: 'import_skill_archive',
      archiveName: 'sample.zip',
      archivePayload: 'UEsDBA==',
    })
    expect(invoke).toHaveBeenNthCalledWith(1, 'execute_extension_write', {
      command: {
        kind: 'import_skill_archive',
        archiveName: 'sample.zip',
        archivePayload: 'UEsDBA==',
      },
    })
    expect(app.extensionConfig?.skills).toEqual(['sample'])
  })

  it('sends uninstall, restore, and MCP management command shapes', async () => {
    invoke.mockResolvedValue({ config: configDto({ revision: 3 }), skillsLinked: [], mcpWritten: [] })
    const app = useAppStore()
    await app.executeExtensionWrite({ kind: 'uninstall_skill', name: 'sample', confirm: true })
    expect(invoke).toHaveBeenNthCalledWith(1, 'execute_extension_write', {
      command: { kind: 'uninstall_skill', name: 'sample', confirm: true },
    })
    await app.executeExtensionWrite({ kind: 'restore_skill', name: 'sample' })
    expect(invoke).toHaveBeenNthCalledWith(3, 'execute_extension_write', {
      command: { kind: 'restore_skill', name: 'sample' },
    })
    await app.executeExtensionWrite({
      kind: 'add_mcp',
      definition: { name: 'local_repl', value: { command: 'node', args: ['--foo'] } },
    })
    expect(invoke).toHaveBeenNthCalledWith(5, 'execute_extension_write', {
      command: {
        kind: 'add_mcp',
        definition: { name: 'local_repl', value: { command: 'node', args: ['--foo'] } },
      },
    })
    await app.executeExtensionWrite({
      kind: 'edit_mcp',
      name: 'local_repl',
      definition: { name: 'local_repl_v2', value: { command: 'node', args: ['--bar'] } },
      confirm: true,
    })
    expect(invoke).toHaveBeenNthCalledWith(7, 'execute_extension_write', {
      command: {
        kind: 'edit_mcp',
        name: 'local_repl',
        definition: { name: 'local_repl_v2', value: { command: 'node', args: ['--bar'] } },
        confirm: true,
      },
    })
    // 删除定义 = `client: null`（作用于所有启用客户端）。
    await app.executeExtensionWrite({
      kind: 'remove_mcp',
      name: 'local_repl_v2',
      client: null,
      confirm: true,
    })
    expect(invoke).toHaveBeenNthCalledWith(9, 'execute_extension_write', {
      command: { kind: 'remove_mcp', name: 'local_repl_v2', client: null, confirm: true },
    })
  })
