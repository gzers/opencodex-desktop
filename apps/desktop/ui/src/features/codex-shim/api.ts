// 「随 Codex 启动 OpenCodex」开关：读写都经官方 shim（`ocx codex-shim`）。
export type CodexShimState = 'installed' | 'not_installed' | 'unreachable'

export interface CodexShimDto {
  state: CodexShimState
  installed: boolean
  summary: string
  /** 不可达的稳定原因码（unresolved / timeout / failed / locked）；可读时为 ''。 */
  reason: string
}

export async function getCodexShimStatus(): Promise<CodexShimDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<CodexShimDto>('codex_shim_status')
}

export async function setCodexShim(enabled: boolean): Promise<CodexShimDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<CodexShimDto>('set_codex_shim', { enabled })
}
