export type ExtensionClientId = 'codex' | 'claude' | 'gemini' | 'grok' | 'opencode' | 'hermes'
// 'agents' = 默认读源（Agent Skills 共享目录 ~/.agents/skills）；
// 'store' = 管理器写入区（数据根 manager-state/skills-store，导入 / 恢复产物）。
export type ExtensionSkillSource = 'agents' | 'store'

export interface ExtensionSkillDto {
  id: string
  name: string
  source: ExtensionSkillSource
  updatedAt: string | null
  description: string
  clients: ExtensionClientId[]
}

export interface ExtensionServerDto {
  id: string
  name: string
  transport: string
  command: string | null
  args: string[]
  envKeys: string[]
  description: string
  clients: ExtensionClientId[]
  updatedAt: string | null
}

export interface ExtensionServerDefinition {
  name: string
  value: Record<string, unknown>
}

export interface ExtensionsDto {
  skills: ExtensionSkillDto[]
  servers: ExtensionServerDto[]
  /** FZ-23 本次发现实际使用的源目录（默认目录或自定义目录的解析结果）。 */
  sourceDir: string
  /** 自定义源目录被忽略时的稳定原因码；正常为 null。 */
  sourceDirNotice: string | null
  /** 源目录是否为用户自定义。 */
  sourceDirCustom: boolean
  /** FZ-24 当前分发方式。 */
  syncMethod: ExtensionSyncMethod
}

export type ExtensionSyncMethod = 'symlink' | 'copy'

export async function listExtensions(): Promise<ExtensionsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ExtensionsDto>('list_extensions')
}

/** AC-11：单个 Skill 的只读详情（`SKILL.md` 正文按需读取，目录统计带上限）。 */
export interface SkillDetailDto {
  name: string
  source: ExtensionSkillSource
  sourcePath: string
  entryFile: string
  body: string | null
  bodyTruncated: boolean
  fileCount: number | null
  totalBytes: number | null
  /** 统计不可用时的稳定原因码（`statsUnavailable` / `statsLimitExceeded`）。 */
  statsNote: string | null
}

/** 某个客户端上的 MCP 配置落点。 */
export interface McpLandingDto {
  client: ExtensionClientId
  configPath: string
  key: string
  present: boolean
}

/** AC-11：MCP 条目的只读详情（命令 / 参数 / 落点；`env` 只给键名，`args` 掩码）。 */
export interface McpDetailDto {
  name: string
  transport: string
  command: string | null
  args: string[]
  envKeys: string[]
  description: string
  clients: ExtensionClientId[]
  updatedAt: string | null
  configJson: string | null
  landings: McpLandingDto[]
}

export async function readSkillDetail(name: string): Promise<SkillDetailDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SkillDetailDto>('read_skill_detail', { name })
}

export async function readMcpDetail(name: string): Promise<McpDetailDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<McpDetailDto>('read_mcp_detail', { name })
}

export interface ExtensionConfigDto {
  skills: string[]
  servers: string[]
  enablement: Record<ExtensionClientId, boolean>
  /** 配置的自定义源目录；null = 使用默认目录。 */
  sourceDir: string | null
  sourceDirCustom: boolean
  /** FZ-24 分发方式。 */
  syncMethod: ExtensionSyncMethod
  revision: number
  fingerprint: string
  updatedAt: string
  conflict: string | null
  documentSha256: string
  backedUp: boolean
}

export interface ExtensionWriteResultDto {
  config: ExtensionConfigDto
  skillsLinked: string[]
  mcpWritten: ExtensionClientId[]
}

export type ExtensionWriteCommand =
  | { kind: 'toggle_client'; client: ExtensionClientId; enabled: boolean }
  // 逐客户端：图标开关只连/断被点的那一个客户端，绝不删改源目录里的 Skill。
  | { kind: 'link_skill'; name: string; client: ExtensionClientId }
  | { kind: 'unlink_skill'; name: string; client: ExtensionClientId; confirm: boolean }
  | { kind: 'uninstall_skill'; name: string; confirm: boolean }
  | { kind: 'write_mcp'; name: string; client: ExtensionClientId; confirm: boolean }
  // `client: null` = 删除该 MCP 定义（所有启用客户端）。
  | { kind: 'remove_mcp'; name: string; client: ExtensionClientId | null; confirm: boolean }
  | { kind: 'import_skill_archive'; archiveName: string; archivePayload: string }
  | { kind: 'restore_skill'; name: string }
  | { kind: 'add_mcp'; definition: ExtensionServerDefinition }
  | { kind: 'edit_mcp'; name: string; definition: ExtensionServerDefinition; confirm: boolean }
  | { kind: 'set_source_dir'; path: string | null }
  | { kind: 'set_sync_method'; method: ExtensionSyncMethod }
  | { kind: 'resync_skills' }

export async function getExtensionConfig(): Promise<ExtensionConfigDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ExtensionConfigDto>('extension_config')
}

export async function setExtensionClientEnabled(
  client: ExtensionClientId,
  enabled: boolean,
): Promise<ExtensionConfigDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ExtensionConfigDto>('set_extension_client_enabled', {
    request: { client, enabled },
  })
}

export async function executeExtensionWrite(
  command: ExtensionWriteCommand,
): Promise<ExtensionWriteResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ExtensionWriteResultDto>('execute_extension_write', { command })
}
