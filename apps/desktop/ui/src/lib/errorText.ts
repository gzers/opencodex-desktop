export function errorText(error: unknown): string {
  const message = typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string'
    ? error.message : typeof error === 'string' ? error : '操作未完成，请重新读取并核对。'
  if (message === 'application is not configured yet') return '运行环境尚未配置，请先在设置的安装配置中指定运行来源。'
  return message
}
