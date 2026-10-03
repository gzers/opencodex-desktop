// 运行来源 / 安装 / 卸载域的后端错误投影（IMP-04 §19.4 E：随切片十二从总 store 迁出）。
// 后端错误统一投影成一句可读中文；Tauri 的 `{code, message}` 也一并处理。
export function runtimeErrorText(error: unknown): string {
  if (typeof error === "string" && error) return error
  if (error && typeof error === "object") {
    const payload = error as { message?: unknown; detail?: unknown; code?: unknown }
    if (typeof payload.message === "string" && payload.message) return payload.message
    if (typeof payload.detail === "string" && payload.detail) return payload.detail
    if (typeof payload.code === "string" && payload.code) return `操作失败（${payload.code}）`
  }
  if (error instanceof Error && error.message) return error.message
  return "操作失败；请查看诊断中心日志。"
}
