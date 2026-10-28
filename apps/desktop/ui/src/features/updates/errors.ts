/** Backend query details are already sanitized; keep npm's original diagnostic
 * text and the command log reference while removing only the transport envelope. */
export function remoteQueryErrorText(error: unknown, fallback = '版本查询失败，请检查网络或运行来源后重试。'): string {
  const message = typeof error === 'string'
    ? error
    : error && typeof error === 'object' && 'message' in error && typeof error.message === 'string'
      ? error.message
      : ''
  if (!message) return fallback
  if (message === 'application is not configured yet') return '运行环境尚未配置，版本查询未执行。'
  return message.replace(/^(?:official metadata query|managed runtime operation) failed: [a-z_]+;\s*/, '')
}
