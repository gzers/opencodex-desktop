// Lexical UI feedback only. The backend remains authoritative for protected
// OS paths, permissions, links, actual containment and installation admission.
interface PathIdentity { root: string; parts: string[]; windows: boolean }
const POSIX_PROTECTED = ['bin', 'sbin', 'usr', 'etc', 'System', 'Library', 'private', 'Applications']

function identity(value: string, platform?: string): PathIdentity | null {
  const path = value.trim()
  if (!path || /[\u0000-\u001f]/.test(path)) return null
  const windows = platform?.toLowerCase() === 'windows'
  let root: string, remaining: string
  if (windows) {
    const normalized = path.replaceAll('\\', '/')
    const extendedDrive = /^\/\/\?\/([a-z]):\/(.*)$/i.exec(normalized)
    const extendedShare = /^\/\/\?\/unc\/([^/]+)\/([^/]+)(?:\/(.*))?$/i.exec(normalized)
    const drive = /^([a-z]):\/(.*)$/i.exec(normalized)
    const share = /^\/\/([^/]+)\/([^/]+)(?:\/(.*))?$/.exec(normalized)
    if (extendedDrive) {
      root = 'drive:' + extendedDrive[1]!.toLowerCase()
      remaining = extendedDrive[2]!
    } else if (extendedShare && !['.', '..', '?'].includes(extendedShare[1]!) && !['.', '..', '?'].includes(extendedShare[2]!)) {
      root = 'unc:' + extendedShare[1]!.toLowerCase() + '/' + extendedShare[2]!.toLowerCase()
      remaining = extendedShare[3] ?? ''
    } else if (drive) {
      root = 'drive:' + drive[1]!.toLowerCase()
      remaining = drive[2]!
    } else if (share && !['.', '..', '?'].includes(share[1]!) && !['.', '..', '?'].includes(share[2]!)) {
      root = 'unc:' + share[1]!.toLowerCase() + '/' + share[2]!.toLowerCase()
      remaining = share[3] ?? ''
    } else return null
  } else {
    if (!path.startsWith('/')) return null
    root = 'posix'
    remaining = path.slice(1)
  }
  const parts = remaining.split('/').filter(Boolean)
  if (parts.some(part => part === '.' || part === '..')) return null
  if (windows && parts.some(part => /[:<>"|?*]/.test(part) || /[. ]$/.test(part) || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part))) return null
  return { root, parts: windows ? parts.map(part => part.toLowerCase()) : parts, windows }
}

export function installPrefixIssue(value: string, platform?: string): string {
  if (!value.trim()) return ''
  const parsed = identity(value, platform)
  if (!parsed) return '安装位置必须是当前平台的完整绝对路径，且不能包含 . 或 ..'
  if (!parsed.parts.length || (!parsed.windows && POSIX_PROTECTED.includes(parsed.parts[0]!))) {
    return '安装位置落在系统保护目录内，已拒绝'
  }
  // Windows protected roots come from the host environment, not a guessed drive.
  // Their definitive check is kept in validate_install_target.
  return ''
}

/** Component hint only, never proof of canonical filesystem ownership. */
export function prefixInsideDataRoot(root: string, prefix: string, platform?: string): boolean | null {
  const parent = identity(root, platform), candidate = identity(prefix, platform)
  if (!parent || !candidate) return null
  return parent.root === candidate.root && parent.parts.length <= candidate.parts.length
    && parent.parts.every((part, index) => part === candidate.parts[index])
}

/** Display only ordinary Windows directory identities; retain device/long-path semantics. */
export { displayPath as displayInstallPrefix } from '@/lib/paths'
