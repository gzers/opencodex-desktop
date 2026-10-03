export type LogKind = 'app' | 'agent' | 'audit' | 'sync'

export interface LogsDto {
  kind: LogKind
  lines: string[]
  redactedLineCount: number
  truncated: boolean
  fileMissing: boolean
  /** 请求的日志文件不存在、实际改读了别的文件时，这里是那个文件名。 */
  fallbackFile: string | null
}

export async function readLogs(kind: LogKind): Promise<LogsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<LogsDto>('read_logs', { kind })
}
