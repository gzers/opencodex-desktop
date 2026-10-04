// 语义化版本比较（U-03）：只用于「本地 vs 远端」的只读判断，不产生安装副作用。
// 取数字段逐段比较；缺失段按 0，预发布后缀按「小于」同版本正式版处理（保守，不误报更新）。

interface ParsedVersion {
  core: number[]
  prerelease: string | null
}

function parseVersion(value: string): ParsedVersion | null {
  const trimmed = value.trim().replace(/^v/i, '')
  if (!trimmed) return null
  const [corePart, prerelease = null] = trimmed.split('-', 2) as [string, string?]
  const core = corePart.split('.').map(part => {
    const digits = part.match(/^\d+/)?.[0]
    return digits === undefined ? Number.NaN : Number.parseInt(digits, 10)
  })
  if (core.length === 0 || core.some(segment => Number.isNaN(segment))) return null
  return { core, prerelease }
}

/** 返回 a 相对 b 的大小：>0 表示 a 更新，<0 表示更旧，0 表示同版本。无法解析返回 null。 */
export function compareVersions(a: string, b: string): number | null {
  const left = parseVersion(a)
  const right = parseVersion(b)
  if (!left || !right) return null
  const length = Math.max(left.core.length, right.core.length)
  for (let index = 0; index < length; index += 1) {
    const diff = (left.core[index] ?? 0) - (right.core[index] ?? 0)
    if (diff !== 0) return diff
  }
  if (left.prerelease === right.prerelease) return 0
  if (left.prerelease === null) return 1
  if (right.prerelease === null) return -1
  return left.prerelease < right.prerelease ? -1 : 1
}

/** 远端版本是否严格高于本地版本；任一方不可解析时返回 null（界面显示「无法比较」）。 */
export function hasNewerVersion(local: string | null, remote: string | null): boolean | null {
  if (!local || !remote) return null
  const comparison = compareVersions(remote, local)
  return comparison === null ? null : comparison > 0
}
