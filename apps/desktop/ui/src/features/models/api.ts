import { invoke } from '@tauri-apps/api/core'

export type Field<T> = { mode: 'inherit' } | { mode: 'custom'; value: T }
export interface ModelRules {
  protocol: Field<string>
  contextWindow: Field<number>
  inputModalities: Field<string[]>
  reasoningEfforts: Field<string[]>
  defaultEffort: Field<string>
  fast: Field<boolean>
}
export interface Pattern { kind: 'exact' | 'prefix' | 'glob'; value: string }
export interface MatchRules {
  include: Pattern[]
  exclude: Pattern[]
  caseSensitive: boolean
  channelMode: 'all' | 'whitelist' | 'blacklist'
  channels: string[]
  priority: number
}
export interface Variant { id: string; name: string; suffix: string; rules: ModelRules; lockedFields: string[] }
export interface TemplateContent { description: string; matching: MatchRules; rules: ModelRules; variants: Variant[]; lockedFields: string[] }
export interface ChangeFact { field: string; before: unknown; after: unknown }
export interface Revision {
  id: string; createdAt: string; savedLabel: string; content: TemplateContent
  digest: string; baseRevision: string | null; changes: ChangeFact[]
}
export interface Template { id: string; label: string; source: 'builtin' | 'user'; defaultRevision: string | null; revisions: Revision[] }
export interface ModelSnapshot {
  library: { schemaVersion: number; templates: Template[] }
  libraryDigest: string
  policyDigest: string
}
export interface TemplateDraft { templateId: string | null; baseRevision: string | null; label: string; content: TemplateContent }
export interface Commit { snapshot: ModelSnapshot; templateId: string; revisionId: string; changed: boolean }
export interface MatchPreview { matched: boolean; reason: string; variantId: string | null; rules: ModelRules }
export const emptyRules = (): ModelRules => ({ protocol: { mode: 'inherit' }, contextWindow: { mode: 'inherit' }, inputModalities: { mode: 'inherit' }, reasoningEfforts: { mode: 'inherit' }, defaultEffort: { mode: 'inherit' }, fast: { mode: 'inherit' } })
export const emptyContent = (): TemplateContent => ({ description: '', matching: { include: [{ kind: 'prefix', value: '' }], exclude: [], caseSensitive: false, channelMode: 'all', channels: [], priority: 0 }, rules: emptyRules(), variants: [], lockedFields: [] })
export const modelApi = {
  load: () => invoke<ModelSnapshot>('model_config_snapshot'),
  save: (draft: TemplateDraft, expectedLibrary: string) => invoke<Commit>('model_template_save', { request: { ...draft, expectedLibrary } }),
  setDefault: (expectedLibrary: string, templateId: string, revisionId: string) => invoke<ModelSnapshot>('model_template_set_default', { expectedLibrary, templateId, revisionId }),
  delete: (expectedLibrary: string, expectedPolicy: string, templateId: string, revisionIds: string[]) => invoke<ModelSnapshot>('model_template_delete', { expectedLibrary, expectedPolicy, templateId, revisionIds }),
  preview: (content: TemplateContent, channelId: string, modelId: string) => invoke<MatchPreview>('model_match_preview', { content, channelId, modelId }),
}

export function errorText(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string') return error.message
  return typeof error === 'string' ? error : '操作未完成，请重新读取并核对。'
}
export function versionLabel(revision: Revision, baseline = false): string {
  const date = new Intl.DateTimeFormat('sv-SE', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }).format(new Date(revision.createdAt))
  return `${date} · ${revision.id.slice(0, 4).toUpperCase()}${baseline ? ' · 内置基线' : ''}`
}
