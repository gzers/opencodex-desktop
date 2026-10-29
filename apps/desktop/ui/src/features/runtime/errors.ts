// 运行来源 / 安装 / 卸载域的后端错误投影（IMP-04 §19.4 E：随切片十二从总 store 迁出）。
// 后端错误统一投影成一句可读中文；Tauri 的 `{code, message}` 也一并处理。
export function runtimeErrorReason(error: unknown): string {
  if (!error || typeof error !== 'object') return ''
  const value = error as { code?: unknown; message?: unknown }
  if (typeof value.code === 'string') return value.code
  // Production AppErrorPayload retains the stable managed reason in its
  // numeric-14 envelope; never infer permission from localized prose.
  if (value.code === 14 && typeof value.message === 'string') {
    return /^managed runtime operation failed: ([a-z_]+);/.exec(value.message)?.[1] ?? ''
  }
  return ''
}

export function runtimeErrorText(error: unknown): string {
  const readable = (message: string) => message.replace(/^(?:managed runtime operation|official metadata query) failed: [a-z_]+;\s*/, '')
  if (typeof error === "string" && error) return readable(error)
  if (error && typeof error === "object") {
    const payload = error as { message?: unknown; detail?: unknown; code?: unknown }
    if (typeof payload.message === "string" && payload.message) return readable(payload.message)
    if (typeof payload.detail === "string" && payload.detail) return payload.detail
    if (typeof payload.code === "string" && payload.code) return `操作失败（${payload.code}）`
  }
  if (error instanceof Error && error.message) return error.message
  return "操作失败；请查看诊断中心日志。"
}
