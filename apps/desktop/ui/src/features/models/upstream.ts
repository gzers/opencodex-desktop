import { emptyRules, type ChannelPolicy, type DiscoveredModel, type UpstreamSnapshot } from './api'
const record = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null && !Array.isArray(value)
const strings = (value: unknown): string[] => Array.isArray(value) && value.every(item => typeof item === 'string') ? value : []

/** Official public snapshots seed a draft. Existing local ids/nodes/adoptions stay intact. */
export function mergeUpstream(snapshot: UpstreamSnapshot, existing: ChannelPolicy[]): ChannelPolicy[] {
  if (!record(snapshot.providers) || !record(snapshot.providers.providers) || !Array.isArray(snapshot.models)) throw new Error('官方快照结构不受支持')
  const result: ChannelPolicy[] = []
  for (const [providerId, raw] of Object.entries(snapshot.providers.providers)) {
    if (!record(raw) || typeof raw.baseUrl !== 'string' || typeof raw.adapter !== 'string') throw new Error('官方渠道缺少连接字段，未修改草稿')
    const old = existing.find(c => c.connection.providerId === providerId)
    const rows = snapshot.models.filter((row): row is Record<string, unknown> => record(row) && row.provider === providerId && typeof row.id === 'string' && row.native !== true)
    const discovered: DiscoveredModel[] = rows.map(row => {
      const rules = emptyRules()
      if (typeof row.contextWindow === 'number' && row.contextWindow > 0) rules.contextWindow = { mode: 'custom', value: row.contextWindow }
      if (Array.isArray(row.inputModalities)) rules.inputModalities = { mode: 'custom', value: strings(row.inputModalities) }
      if (Array.isArray(row.reasoningEfforts)) rules.reasoningEfforts = { mode: 'custom', value: strings(row.reasoningEfforts) }
      if (typeof row.defaultReasoningEffort === 'string') rules.defaultEffort = { mode: 'custom', value: row.defaultReasoningEffort }
      if (typeof row.fastRowAvailable === 'boolean') rules.fast = { mode: 'custom', value: row.fastRowAvailable }
      return { id: String(row.id), label: record(raw.modelDisplayNames)&&typeof raw.modelDisplayNames[String(row.id)]==='string'?String(raw.modelDisplayNames[String(row.id)]):typeof row.displayName === 'string' && row.displayName!==row.namespaced ? row.displayName : String(row.id), rules }
    })
    if (old) { result.push({ ...old, connection:{...old.connection,sourceReadId:snapshot.readId??old.connection.sourceReadId,authMode:typeof raw.authMode==='string'?raw.authMode:old.connection.authMode}, discovered }); continue }
    const node = { id: crypto.randomUUID(), name: '当前节点', url: raw.baseUrl }
    const selected = strings(raw.selectedModels)
    const visible = rows.filter(row => row.disabled !== true && row.initialSelectionPending !== true).map(row => String(row.id))
    result.push({ id: crypto.randomUUID(), label: providerId, templateRefs: [], modelRefs: {}, connection: { sourceReadId:snapshot.readId,authMode:typeof raw.authMode==='string'?raw.authMode:'key',providerId, protocol: raw.adapter, nodes: [node], activeNodeId: node.id, credentialRef: null, enabled: raw.disabled !== true, allowPrivateNetwork: raw.allowPrivateNetwork === true }, selection: selected.length ? { mode: 'whitelist', models: selected } : visible.length === rows.length && rows.length ? { mode: 'all' } : visible.length ? { mode: 'whitelist', models: visible } : { mode: 'none' }, modelOrder: discovered.map(row => row.id), discovered, defaults: emptyRules(), overrides: {}, familyOverrides: {} })
  }
  // A temporarily missing upstream channel is not a deletion instruction.
  for (const old of existing) if (!result.some(c => c.connection.providerId === old.connection.providerId)) result.push(old)
  // Existing channel order is authoritative; newly found channels append.
  const rank = new Map(existing.map((c, index) => [c.id, index]))
  return result.sort((a,b) => (rank.get(a.id) ?? Number.MAX_SAFE_INTEGER) - (rank.get(b.id) ?? Number.MAX_SAFE_INTEGER))
}
