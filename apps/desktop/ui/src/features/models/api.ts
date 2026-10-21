import { invoke } from '@tauri-apps/api/core'

export type Field<T> = { mode: 'inherit' } | { mode: 'custom'; value: T }
export interface FastWire {kind:'service-tier'|'anthropic-speed'|'cursor-variant';canonicalToWire:Record<string,string>;foreignCallerTiers:'verbatim'|'drop';betas?:string[]}
export interface ModelRules {
  protocol: Field<string>
  contextWindow: Field<number>
  inputModalities: Field<string[]>
  reasoningEfforts: Field<string[]>
  defaultEffort: Field<string>
  fast: Field<boolean>
  pinnedEffort?: Field<string>
  reasoningAuthoritative?:Field<boolean>
  fastWire?:Field<FastWire|null>
  noReasoning?: Field<boolean>
  effortMap?: Field<Record<string,string>>
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
export interface TemplateContent { description: string; matching: MatchRules; rules: ModelRules; variants: Variant[]; lockedFields: string[]; sources: { field: string; url: string | null; note: string; recordedAt: string }[] }
export interface ChangeFact { field: string; before: unknown; after: unknown }
export interface Revision {
  id: string; createdAt: string; savedLabel: string; content: TemplateContent
  digest: string; baseRevision: string | null; changes: ChangeFact[]
}
export interface Template { id: string; label: string; source: 'builtin' | 'user'; defaultRevision: string | null; revisions: Revision[] }
export interface ModelSnapshot {
  library: { schemaVersion: number; templates: Template[] }
  policy: ModelPolicy
  libraryDigest: string
  policyDigest: string
  workspace: { basePolicy: string; policy: ModelPolicy } | null
  workspaceDigest: string
}
export interface TemplateRef { templateId: string; revisionId: string }
export interface ServiceNode { id: string; name: string; url: string }
export interface NodeMeasurement {nodeId:string;elapsedMs:number|null;modelCount:number|null;error:string|null}
export interface NodeMeasurementSet {runId:string;stopped:boolean;results:NodeMeasurement[]}
export interface DiscoveredModel { id: string; label: string; rules: ModelRules }
export type ModelSelection = { mode: 'all' | 'none' } | { mode: 'whitelist'; models: string[] }
export interface ModelNaming {alias?:string;displayName?:string}
export interface ChannelPolicy {
  id: string; label: string; templateRefs: TemplateRef[]; modelRefs: Record<string, TemplateRef>
  connection: { providerId: string; protocol: string; nodes: ServiceNode[]; activeNodeId: string; credentialRef: string | null; enabled: boolean; allowPrivateNetwork: boolean; authMode?:string|null;sourceReadId?:string|null;importedConfig?:{id:string;digest:string}|null }
  selection: ModelSelection; modelOrder: string[]; discovered: DiscoveredModel[]
  naming?:Record<string,ModelNaming>
  defaults: ModelRules; overrides: Record<string, ModelRules>; familyOverrides: Record<string, ModelRules>
}
export interface ModelPolicy { schemaVersion: number; templateRefs: TemplateRef[]; channels: ChannelPolicy[];defaultChannelId?:string|null;recoveryId?:string|null;pendingRemovals?:{readId:string;channelId:string;providerId:string;credentialRef:string|null}[] }
export interface ModelProjection {connectionDigest?:string|null;nativeModelId?:string|null;unavailableReason?:string|null;aliases?:string[]; channelId: string; providerId: string; modelId: string; label: string; enabled: boolean; selected: boolean; discovered: boolean; templateRef: TemplateRef | null; rules: ModelRules; source: Record<string, string> }
export interface DraftProjection { rows: ModelProjection[]; knownActiveModels: number; exactCatalogCount: number | null; notices: string[] }
export interface TemplateUsage {templateId:string;revisionId:string;libraryDigest:string;policyDigest:string;workspaceDigest:string;references:{state:string;location:string}[];models:{state:string;channelId:string;channelLabel:string;modelId:string;matched:boolean;reason:string;adopted:boolean;resolutionError:string|null}[];historicalDependency:boolean}
export interface SaveScope {channelIds:string[];includeShared:boolean}
export interface SavePlan { id:string;phase:string;catalogCount:number;currentCatalogCount:number;changedChannels:string[];deferredChannels?:string[];error:string|null;protocolGroups?:string[];routeChanges?:string[];removedProviders?:string[];historySource?:string|null;runtimeChanges?:ChangeFact[] }
export interface ModelClientState {planId:string|null;status:'not_required'|'restart_required'|'pending_verification'|'user_confirmed'|'runtime_changed';checkedAt:string|null;checkId:string|null;confirmedAt:string|null;runtimeMatches:boolean;automaticClientVerification:boolean}
export interface ModelHistoryEntry {id:string;createdAt:string;phase:string;channelLabels:string[];recoverable:boolean}
export interface ModelRecoveryRequest {sourceId:string;side:'before'|'after';scope:'affected'|'all';restoreDefaults:boolean;expectedLibrary:string;expectedPolicy:string;expectedWorkspace:string}
export interface ModelRecoveryPreview {requiresRuntimeReview?:boolean;id:string;sourceId:string;side:'before'|'after';scope:'affected'|'all';restoreDefaults:boolean;changes:ChangeFact[];defaultChanges:ChangeFact[];channelLabels:string[];notices:string[]}
export type ExternalScene = 'from_sources' | 'update_models' | 'audit_and_revise' | 'private_models' | 'preferences' | 'check_only'
export interface ExternalTaskRequest { scene: ExternalScene; targets: TemplateRef[]; allowNew: boolean; requirements: string; expectedLibrary: string; buffers: { templateId: string; baseRevision: string; label: string; content: TemplateContent }[] }
export interface TaskGrant { task: { id: string; contractVersion: number; scene: ExternalScene; targets: unknown[]; allowNew: boolean; requirements: string; expectedLibrary: string; expiresAt: string; revoked: boolean }; token: string }
export interface TaskBundle { taskId: string; content: string; grant: TaskGrant }
export interface ExecutionDirectory {channels:{id:string;label:string;providerId:string}[];rows:ModelProjection[]}
export interface AgentSettings {channelId?:string|null;connectionDigest?:string|null; providerId: string; modelId: string; reasoningEffort: string | null; fast: boolean | null }
export interface AgentConversationContent {baseRevision:string|null;input:string;history:AgentMessage[];sources:ResearchSource[]}
export interface AgentConversation {templateId:string|null;digest:string;content:AgentConversationContent}
export interface AgentMessage {role:'user'|'assistant';text:string}
export interface ResearchSource {receiptId?:string|null;contentDigest?:string|null; url: string; retrievedAt: string; text: string; links?: string[] }
export interface Proposal { templateId: string | null; baseRevision: string | null; label: string; content: TemplateContent }
export interface Submission { submissionId: string; contractVersion: number; proposals: Proposal[]; report: string | null }
export interface PendingSuggestion {history?:AgentMessage[];sources?:ResearchSource[];runId:string;createdAt:string;expectedLibrary:string;scene:ExternalScene;targets:{templateId:string;label:string;revision:Revision;buffer:TemplateDraft|null}[];result:Submission;reviewed:boolean}
export interface OwnerTaskResult { task: { id: string; targets: { templateId: string; label: string }[]; requirements: string }; receipt: { submissionId: string; state: string; outcomes: {templateId:string;revisionId:string;changed:boolean}[]; report: string | null } | null }
export interface UpstreamSnapshot {readId?:string; version: string; providers: unknown; models: unknown; order: unknown }
export interface TemplateDraft { templateId: string | null; baseRevision: string | null; label: string; content: TemplateContent }
export interface Commit { snapshot: ModelSnapshot; templateId: string; revisionId: string; changed: boolean }
export interface MatchPreview { matched: boolean; reason: string; variantId: string | null; rules: ModelRules }
export const emptyRules = (): ModelRules => ({ protocol: { mode: 'inherit' }, contextWindow: { mode: 'inherit' }, inputModalities: { mode: 'inherit' }, reasoningEfforts: { mode: 'inherit' }, defaultEffort: { mode: 'inherit' }, fast: { mode: 'inherit' }, pinnedEffort:{mode:'inherit'}, effortMap:{mode:'inherit'}, noReasoning:{mode:'inherit'},reasoningAuthoritative:{mode:'inherit'},fastWire:{mode:'inherit'} })
export const emptyContent = (): TemplateContent => ({ description: '', matching: { include: [{ kind: 'prefix', value: '' }], exclude: [], caseSensitive: false, channelMode: 'all', channels: [], priority: 0 }, rules: emptyRules(), variants: [], lockedFields: [], sources: [] })
export interface NewChannelRequest {id:string;expectedPolicy:string;expectedWorkspace:string;channel:ChannelPolicy;apiKey:string|null;replace?:ChannelPolicy}
export interface NewChannelOutcome {id:string;channel:ChannelPolicy;phase:'prepared'|'staging'|'committed'|'failed'|'conflict'|'cleanup_pending';error:{code:string;message:string}|null;snapshot:ModelSnapshot|null}
export const modelApi = {
  previewNewChannel:(request:{protocol:string;url:string;apiKey:string|null;credentialRef:null;allowPrivateNetwork:boolean})=>invoke<{models:DiscoveredModel[];credentialRef:null;elapsedMs:number}>('model_new_channel_preview',{request}),
  addNewChannel:(request:NewChannelRequest)=>invoke<NewChannelOutcome>('model_new_channel_add',{request}),
  newChannelResult:(id:string)=>invoke<NewChannelOutcome>('model_new_channel_result',{id}),
  latestNewChannel:()=>invoke<NewChannelOutcome|null>('model_new_channel_latest'),
  load: () => invoke<ModelSnapshot>('model_config_snapshot'),
  save: (draft: TemplateDraft, expectedLibrary: string) => invoke<Commit>('model_template_save', { request: { ...draft, expectedLibrary } }),
  setDefault: (expectedLibrary: string, templateId: string, revisionId: string) => invoke<ModelSnapshot>('model_template_set_default', { expectedLibrary, templateId, revisionId }),
  delete: (expectedLibrary: string, expectedPolicy: string, expectedWorkspace: string, templateId: string, revisionIds: string[]) => invoke<ModelSnapshot>('model_template_delete', { expectedLibrary, expectedPolicy, expectedWorkspace, templateId, revisionIds }),
  templateUsage:(templateId:string,revisionId:string,expectedLibrary:string)=>invoke<TemplateUsage>('model_template_usage',{templateId,revisionId,expectedLibrary}),
  preview: (content: TemplateContent, channelId: string, modelId: string) => invoke<MatchPreview>('model_match_preview', { content, channelId, modelId }),
  project: (policy: ModelPolicy) => invoke<DraftProjection>('model_policy_preview', { policy }),
  stage: (expectedPolicy: string, expectedWorkspace: string, policy: ModelPolicy) => invoke<ModelSnapshot>('model_policy_stage', { expectedPolicy, expectedWorkspace, policy }),
  preparePolicy: (policy:ModelPolicy,expectedLibrary:string,expectedPolicy:string,expectedWorkspace:string) => invoke<SavePlan>('model_policy_prepare',{policy,expectedLibrary,expectedPolicy,expectedWorkspace}),
  preparePolicyScoped: (policy:ModelPolicy,expectedLibrary:string,expectedPolicy:string,expectedWorkspace:string,scope:SaveScope) => invoke<SavePlan>('model_policy_prepare',{policy,expectedLibrary,expectedPolicy,expectedWorkspace,scope}),
  applyPolicy: (planId:string) => invoke<SavePlan>('model_policy_apply',{planId}),
  policyResult: (planId:string) => invoke<SavePlan>('model_policy_result',{planId}),
  latestPolicy: () => invoke<SavePlan|null>('model_policy_latest'),
  clientStatus:()=>invoke<ModelClientState>('model_policy_client_status'),
  checkClient:(planId:string,confirmationCheck?:string)=>invoke<ModelClientState>('model_policy_client_check',{request:{planId,confirmationCheck:confirmationCheck??null}}),
  policyHistory:()=>invoke<ModelHistoryEntry[]>('model_policy_history'),
  prepareRecovery:(request:ModelRecoveryRequest)=>invoke<ModelRecoveryPreview>('model_policy_recovery_prepare',{request}),
  acceptRecovery:(planId:string)=>invoke<ModelSnapshot>('model_policy_recovery_accept',{planId}),
  compensatePolicy: (planId:string) => invoke<SavePlan>('model_policy_compensate',{planId}),
  upstream: () => invoke<UpstreamSnapshot>('model_upstream_snapshot'),
  discover: (request:{protocol:string;url:string;apiKey:string|null;credentialRef:string|null;allowPrivateNetwork:boolean}) => invoke<{models:DiscoveredModel[];credentialRef:string|null;elapsedMs:number}>('model_channel_discover',{request}),
  forgetCredential: (reference:string) => invoke<void>('model_channel_forget_credential',{reference}),
  measureNodes:(request:{runId:string;protocol:string;nodes:ServiceNode[];apiKey:string|null;credentialRef:string|null;allowPrivateNetwork:boolean})=>invoke<NodeMeasurementSet>('model_node_measure',{request}),
  stopNodeMeasurement:(runId:string)=>invoke<void>('model_node_measure_stop',{runId}),
  external: (request: ExternalTaskRequest) => invoke<TaskBundle>('model_external_task', { request }),
  activate: (grant: TaskGrant) => invoke<void>('model_external_activate', { grant }),
  externalResult: (taskId: string, submissionId: string) => invoke<OwnerTaskResult>('model_external_result', { taskId, submissionId }),
  executionDirectory:()=>invoke<ExecutionDirectory>('model_agent_directory'),
  agentSettings: () => invoke<AgentSettings | null>('model_agent_preferences'),
  saveAgentSettings: (settings: AgentSettings) => invoke<AgentSettings>('model_agent_preferences_save', { settings }),
  research: (urls: string[]) => invoke<ResearchSource[]>('model_research_sources', { urls }),
  searchSources: (query: string) => invoke<ResearchSource[]>('model_research_search', { query }),
  generate: (runId: string, task: ExternalTaskRequest, sources: ResearchSource[], history:AgentMessage[]=[]) => invoke<Submission>('model_agent_generate', { request: { runId, task, sources,history } }),
  stopGeneration: (runId: string) => invoke<void>('model_agent_stop', { runId }),
  acceptAgentReview:(request:{runId:string;expectedLibrary:string;proposals:Proposal[]})=>invoke<{snapshot:ModelSnapshot;outcomes:{templateId:string;revisionId:string;changed:boolean}[]}>('model_agent_accept_review',{request}),
  agentConversation:(templateId:string|null)=>invoke<AgentConversation|null>('model_agent_conversation',{templateId}),
  saveAgentConversation:(request:{templateId:string|null;expectedDigest:string|null;content:AgentConversationContent})=>invoke<AgentConversation>('model_agent_conversation_save',{request}),
  latestAgentResult:(templateId:string|null)=>invoke<PendingSuggestion|null>('model_agent_latest_result',{templateId}),
}

export function errorText(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string') return error.message
  return typeof error === 'string' ? error : '操作未完成，请重新读取并核对。'
}
export function savePlanErrorText(code:string|null|undefined):string {
  if(!code)return ''
  if(code==='catalog_write')return '模型目录刷新结果尚未确认，请核对原计划。'
  if(code==='catalog_readback'||code==='catalog_projection')return '模型目录与确认内容不同，请核对原计划和当前模型范围。'
  if(code.includes('drift')||code.startsWith('stale_'))return '检测到后来的配置修改，已停止继续写入；请核对原计划。'
  if(code.startsWith('credential_')||code.startsWith('provider_auth_'))return '渠道认证配置未完成核对，请检查系统凭据与原保存计划。'
  if(code==='provider_readback'||code==='registration_readback'||code==='history_readback')return '渠道配置读回与确认内容不同，请核对原计划。'
  if(code==='upstream_write_unknown'||code==='management_unknown')return '官方接口结果尚未确认，可能已有部分写入；请核对原计划。'
  return '本次保存未完成，请核对原计划；已保留执行记录。'
}
export function versionLabel(revision: Revision, baseline = false): string {
  const date = new Intl.DateTimeFormat('sv-SE', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }).format(new Date(revision.createdAt))
  return `${date} · ${revision.id.slice(0, 4).toUpperCase()}${baseline ? ' · 内置基线' : ''}`
}
