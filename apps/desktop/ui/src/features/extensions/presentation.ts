import type {
  ExtensionClientId,
  ExtensionServerDto,
  ExtensionSkillDto,
} from '@/features/extensions/api'

const clientLabels: Record<ExtensionClientId, string> = {
  claude: 'Claude',
  codex: 'Codex',
  gemini: 'Gemini',
  grok: 'Grok Build',
  opencode: 'OpenCode',
  hermes: 'Hermes',
}

export const skillClientOrder: ExtensionClientId[] = [
  'claude', 'codex', 'gemini', 'grok', 'opencode', 'hermes',
]

export const skillSourceLabels: Record<ExtensionSkillDto['source'], string> = {
  agents: '本地',
  store: '管理器',
}

/** 只有日期，不带「更新」前缀：详情信息区的标签已经写明是更新时间。 */
export function formatDateOnly(value: string | null, now = new Date()): string {
  if (!value) return '未知'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '未知'
  const formatter = new Intl.DateTimeFormat('en-CA', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    timeZone: now.getTimezoneOffset() === 0 ? 'UTC' : undefined,
  })
  return formatter.format(date)
}

export function formatUpdatedAt(value: string | null, now = new Date()): string {
  const date = formatDateOnly(value, now)
  if (date === '未知') return '未知时间'
  return `更新 ${date}`
}

export function skillVersion(skill: ExtensionSkillDto): string {
  return skill.source === 'agents' ? '本地源' : '管理器源'
}

export function serverVersion(server: ExtensionServerDto): string {
  return server.updatedAt ? '本机配置' : '本机配置'
}

export interface ExtensionChip {
  /** 客户端筛选键；`null` 表示「全部 / 已安装」汇总项。 */
  key: ExtensionClientId | null
  label: string
  value: number | string
}

export function extensionChips(
  countLabel: string,
  count: number,
  items: Array<{ clients: ExtensionClientId[] }>,
): ExtensionChip[] {
  const chips: ExtensionChip[] = [{ key: null, label: countLabel, value: count }]
  for (const client of skillClientOrder) {
    chips.push({
      key: client,
      label: clientLabels[client],
      value: items.filter(item => item.clients.includes(client)).length,
    })
  }
  return chips
}

export function targetEnabled(
  clients: ExtensionClientId[],
  target: ExtensionClientId,
): boolean {
  return clients.includes(target)
}

export { clientLabels }
