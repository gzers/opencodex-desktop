// 「拓展 → 客户端」列表与详情的数据投影（N-EX-1）。
// 只使用软件已有的目标客户端、发现结果、启用矩阵与路径投影；
// 取不到的字段按原型口径落到「待核实」，不臆造新的客户端或能力概念。
import type {
  ExtensionClientId,
  ExtensionServerDto,
  ExtensionSkillDto,
} from '@/features/extensions/api'
import { clientLabels } from '@/features/extensions/presentation'

/** 客户端列表顺序与原型 catalog 一致（Codex 在前）。 */
export const clientDirectoryOrder: readonly ExtensionClientId[] = [
  'codex',
  'claude',
  'gemini',
  'grok',
  'opencode',
  'hermes',
]

export type ClientCapability = 'ready' | 'unknown'
export type ClientStatusFilter = 'all' | 'installed' | 'issues' | 'unknown'
export type ClientTagTone = 'running' | 'starting' | 'warn' | 'bad' | 'muted'

export interface ClientManagedResource {
  /** 同时含 Skill 与 MCP 时名称可能相同，键必须带类型。 */
  key: string
  kind: 'skill' | 'mcp'
  id: string
  name: string
  status: string
}

export interface ClientDirectoryEntry {
  id: ExtensionClientId
  name: string
  monogram: string
  scope: string
  evidence: string
  skillsPath: string | null
  mcpPath: string | null
  skillsReady: boolean
  mcpReady: boolean
  capability: ClientCapability
  installed: boolean
  disabled: boolean
  issue: boolean
  issueReason: string
  managed: ClientManagedResource[]
}

export interface ClientDirectorySource {
  skills: ExtensionSkillDto[]
  servers: ExtensionServerDto[]
  /** get_managed_path_targets 的 Skills 落点，按客户端索引。 */
  skillsPaths?: Partial<Record<ExtensionClientId, string>>
  /** 同上，MCP 配置落点。 */
  mcpPaths?: Partial<Record<ExtensionClientId, string>>
  /** 统一配置的六客户端启用矩阵。 */
  enablement?: Partial<Record<ExtensionClientId, boolean>> | null
}

export function isExtensionClientId(value: string): value is ExtensionClientId {
  return (clientDirectoryOrder as readonly string[]).includes(value)
}

export function buildClientDirectory(source: ClientDirectorySource): ClientDirectoryEntry[] {
  return clientDirectoryOrder.map(id => {
    const name = clientLabels[id]
    const skillsPath = source.skillsPaths?.[id] ?? null
    const mcpPath = source.mcpPaths?.[id] ?? null
    const skillsReady = skillsPath !== null
    const mcpReady = mcpPath !== null
    const managed: ClientManagedResource[] = [
      ...source.skills
        .filter(skill => skill.clients.includes(id))
        .map(skill => ({
          key: 'skill:' + skill.id,
          kind: 'skill' as const,
          id: skill.id,
          name: skill.name,
          status: '已同步',
        })),
      ...source.servers
        .filter(server => server.clients.includes(id))
        .map(server => ({
          key: 'mcp:' + server.id,
          kind: 'mcp' as const,
          id: server.id,
          name: server.name,
          status: '已同步',
        })),
    ]
    const disabled = source.enablement?.[id] === false
    // 软件没有客户端级投影状态；唯一可核实的异常是「配置已停用，但仍有已落地资源」。
    const issue = disabled && managed.length > 0
    return {
      id,
      name,
      monogram: name.slice(0, 2),
      scope: '用户级',
      evidence: '既有六平台契约',
      skillsPath,
      mcpPath,
      skillsReady,
      mcpReady,
      capability: skillsReady && mcpReady ? 'ready' : 'unknown',
      // 两个落点都由后端路径投影给出，才算「本机已发现」；缺一个就退回「待核实」。
      installed: skillsReady && mcpReady,
      disabled,
      issue,
      issueReason: issue ? '已停用，仍有受管资源' : '',
      managed,
    }
  })
}

export function clientStatusTag(entry: ClientDirectoryEntry): { label: string; tone: ClientTagTone } {
  // tone 沿用原型同名状态口径：可管理 / 待核实 / 有异常；颜色由状态标签统一承载。
  if (entry.capability === 'unknown') return { label: '待核实适配', tone: 'warn' }
  if (entry.issue) return { label: '有异常', tone: 'bad' }
  if (entry.disabled) return { label: '已停用', tone: 'muted' }
  // 可管理 = 语义上的「正常运行」，与 tone 唯一词表一致（不再出现 ok 别名）。
  return { label: '可管理', tone: 'running' }
}

/** 详情「适配状态」的完整说法；与原型 reason(t) || 可管理 同口径。 */
export function clientAdaptation(entry: ClientDirectoryEntry): string {
  if (entry.capability === 'unknown') return '待核实扩展适配'
  if (entry.issue) return entry.issueReason
  if (entry.disabled) return '已停用'
  return '可管理'
}

/** 软件不暴露客户端安装探测；没有落地资源时只能说「待核实」。 */
export function clientDiscoveryLabel(entry: ClientDirectoryEntry): string {
  return entry.installed ? '本机已发现' : '待核实'
}

export function clientCapabilityLabel(ready: boolean): string {
  return ready ? '文档有依据' : '待核实'
}

/** 原型 renderClients() 的列表筛选：名称 / id 命中，再按状态口径过滤。 */
export function filterClients(
  entries: readonly ClientDirectoryEntry[],
  query: string,
  status: ClientStatusFilter,
): ClientDirectoryEntry[] {
  const needle = query.trim().toLowerCase()
  return entries.filter(entry => {
    if (needle && !(entry.name + ' ' + entry.id).toLowerCase().includes(needle)) return false
    // 「已发现」= 该客户端确实有已落地资源，这是软件能核实的口径。
    if (status === 'installed') return entry.installed
    if (status === 'issues') return entry.issue
    if (status === 'unknown') return entry.capability === 'unknown'
    return true
  })
}
