import type { ModelPolicy } from './api'

/**
 * 渠道草稿的“未保存修改”判定。
 *
 * 读取现有渠道会把官方快照的读取回执（sourceReadId）和接收来源（importedConfig）
 * 记到草稿上；这两个字段只用于审计本机来源，不代表用户改了配置。旧实现直接比较
 * JSON 字符串，键顺序和这两个字段都会让“只读取、未编辑”显示为未保存修改。
 *
 * 这里把对象键排序后比较，并跳过上述本机记账字段；数组顺序仍然有意义
 * （渠道顺序、型号顺序都是用户可见配置），不参与归一。
 */
const volatileConnectionFields = new Set(['sourceReadId', 'importedConfig'])

function normalize(value: unknown, key?: string): unknown {
  if (Array.isArray(value)) return value.map(item => normalize(item))
  if (value && typeof value === 'object') {
    const source = value as Record<string, unknown>
    const result: Record<string, unknown> = {}
    for (const name of Object.keys(source).sort()) {
      if (key === 'connection' && volatileConnectionFields.has(name)) continue
      if (source[name] === undefined) continue
      result[name] = normalize(source[name], name)
    }
    return result
  }
  return value
}

export function canonicalPolicy(policy: ModelPolicy | null): string {
  return policy ? JSON.stringify(normalize(policy)) : ''
}

export function samePolicyDraft(left: ModelPolicy | null, right: ModelPolicy | null): boolean {
  if (!left || !right) return left === right
  return canonicalPolicy(left) === canonicalPolicy(right)
}
