// 扩展写入失败的可读原因。
//
// 后端返回的是 `{ code, message }`，`message` 是英文技术描述（例如
// `file system operation failed: parse extension projection; projection is invalid or corrupted`）。
// 界面此前把这些一律说成「同名冲突或配置不可写未覆盖」，等于把真因吞掉；
// 这里按已知形态翻成一句中文原因，并保留原始描述便于排查。
export interface ExtensionFailure {
  /** 面向用户的一句话原因。 */
  reason: string
  /** 后端原始描述；为空表示没有可展示的技术信息。 */
  technical: string
}

const RULES: { match: RegExp; reason: string }[] = [
  // 首装不再需要「先建立配置」（缺配置按默认配置执行），这条只剩数据目录不可用时可命中。
  { match: /not configured yet/i, reason: '扩展配置不可用；数据目录无效或不可访问。' },
  { match: /not registered or not installed/i, reason: '目标客户端未安装或当前不可写。' },
  { match: /is not a regular directory|not a regular directory/i, reason: '目标不是常规目录（可能是符号链接），已拒绝。' },
  {
    match: /invalid or corrupted/i,
    reason: '目标配置或统一配置解析失败，文件可能被外部改动或写坏；请先修复该文件再试。',
  },
  { match: /external target changed|conflict is pending/i, reason: '目标配置在管理器之外被改动，需要先处理冲突。' },
  { match: /half written|marker is missing/i, reason: '上次写入未完成，需要先恢复统一配置。' },
  { match: /exceeds frozen limit/i, reason: '统一配置超过冻结上限。' },
  { match: /lock timed out|timed out/i, reason: '目标文件被占用或超时，请稍后重试。' },
  { match: /requires confirmation/i, reason: '同名内容需要显式确认后才能覆盖。' },
  { match: /unsupported/i, reason: '该客户端不支持这种写法。' },
  { match: /escapes|path/i, reason: '路径越界，已拒绝。' },
  { match: /atomic write failed/i, reason: '写入未完成（原子写失败）。' },
  { match: /another application instance is already running/i, reason: '已有管理器实例在运行。' },
]

export function describeExtensionFailure(error: unknown): ExtensionFailure {
  const payload = error as { code?: number; message?: string } | null | undefined
  const message = typeof payload?.message === 'string' ? payload.message : ''
  const rule = RULES.find(item => item.match.test(message))
  return {
    reason: rule?.reason ?? '未归类的失败；请把原始信息一并反馈。',
    technical: message,
  }
}

/** 失败提示正文：原因 + （有则）原始描述片段。 */
export function extensionFailureText(error: unknown): string {
  const { reason, technical } = describeExtensionFailure(error)
  const trimmed = technical.trim()
  return trimmed ? `${reason}（${trimmed}）` : reason
}
