import { parse } from '@vue/compiler-sfc'

/**
 * 状态展示的源码门禁（2026-10-25 裁决 #2 / #4）。
 *
 * 拦两类回退：
 * 1. 在 SFC 里新写 `data-tone="…"` 语义别名 —— tone 唯一词表只有五类，
 *    对应 `styles/base.css` 的 `.state-*`；新别名会让同义色各写一遍。
 * 2. 直接渲染后端英文枚举 —— 必须经 `lib/labels.ts` 或 `lib/saveStates.ts`。
 *
 * 已知例外（登记在 EXEMPT）：
 * - `overview-check-value` / `motion-check-value` 的 ok/bad/busy 是**检查判定**文字色，
 *   不是状态标签，语义域不同；它们由 base.css 自带变体，本轮不动。
 */
const ALLOWED_TONE_KEYS = new Set(['tone', 'data-tone'])

/** 允许保留 data-tone 的既有检查判定类（非状态标签）。 */
const EXEMPT_CLASSES = ['overview-check-value', 'motion-check-value']

/** 后端状态枚举里不允许直接出现在模板文本插值中的值。 */
const RAW_ENUM = /\{\{\s*(?:[A-Za-z_$][\w$]*\.)+matrix\.(?:runtime|connection|operation)\b/g
/** 也拦直接插值 statusSnapshot.matrix.* 这类原始枚举。 */

export function statusToneViolations(source, filename = 'consumer.vue') {
  const { descriptor } = parse(source, { filename })
  const template = descriptor.template?.content ?? ''
  const violations = []

  // 1) data-tone 别名
  for (const match of template.matchAll(/data-tone="([^"]*)"/g)) {
    const value = match[1].trim()
    if (!value || value.includes('{{')) continue
    const line = template.slice(0, match.index).split('\n').length
    // 例外类：同一元素上有 EXEMPT_CLASSES 之一
    const tagStart = template.lastIndexOf('<', match.index)
    const tagText = template.slice(tagStart, template.indexOf('>', match.index) + 1)
    if (EXEMPT_CLASSES.some(cls => tagText.includes(cls))) continue
    violations.push({ file: filename, line, kind: 'data-tone-alias', value })
  }

  // 2) 直接渲染英文枚举
  for (const match of template.matchAll(RAW_ENUM)) {
    const line = template.slice(0, match.index).split('\n').length
    violations.push({ file: filename, line, kind: 'raw-enum', value: match[0].trim() })
  }

  return violations
}

export { ALLOWED_TONE_KEYS, EXEMPT_CLASSES }

