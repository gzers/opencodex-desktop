export type EnvironmentGate =
  | 'checking'
  | 'ready'
  | 'missing_node'
  | 'missing_npm'
  | 'missing_ocx'

export interface DiscoveryCheck {
  found: boolean
  path: string | null
  version: string | null
}

export interface EnvironmentReport {
  node: DiscoveryCheck
  npm: DiscoveryCheck
  ocx: DiscoveryCheck
  gate: Exclude<EnvironmentGate, 'checking'>
  shortCircuited: boolean
  brewFound: boolean
}

export interface DiscoveryPaths {
  nodePath: string
  npmPath: string
  ocxPath: string
}

export async function discoverEnvironment(paths?: DiscoveryPaths): Promise<EnvironmentReport> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<EnvironmentReport>('discover_environment', { request: paths ?? null })
}
