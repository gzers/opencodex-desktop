// Wire types follow schema_v2.rs/source_v2.rs at 6e031e109. No v1 conversion.
import {invoke} from '@tauri-apps/api/core'
import type {ModelRules} from './api'

export type RulesV2 = ModelRules
export interface RefV2 {templateId:string; revisionId:string; digest:string}
export type ModelBindingV2 = {kind:'manual'|'auto_read';reference:Readonly<RefV2>}
export type ProtocolV2 = 'openai-chat'|'openai-responses'|'anthropic'|'google'|'ollama-native'
export interface SourceEvidenceV2 {kind:'imported_file'|'upstream_read'|'portable_import'|'external_file_edit';sourceDigest:string;providerId:string;pointer:string}
export interface SourceModelV2 {readonly rawModelId:string;readonly fields:Readonly<Record<string,{readonly value:unknown;readonly evidence:Readonly<SourceEvidenceV2>}>>}
export type FieldOriginV2 = {kind:'source';evidence:Readonly<SourceEvidenceV2>}|{kind:'template_common';reference:Readonly<RefV2>}|{kind:'template_condition';reference:Readonly<RefV2>;conditionId:string}|{kind:'channel_default'}|{kind:'channel_family';templateId:string}|{kind:'explicit_model'}|{kind:'unknown'}
export type ProtocolStateV2 = {kind:'resolved';value:ProtocolV2;origin:Readonly<FieldOriginV2>}|{kind:'missing'}|{kind:'unsupported';value:string;origin:Readonly<FieldOriginV2>}
export interface MatchIssueV2 {code:string;channelId:string|null;rawModelId:string|null;fieldPath:string|null;conditionIds:string[];reference:RefV2|null;message:string;recoveryActions:string[]}
export interface MatchErrorV2 {code:string;issues:MatchIssueV2[]}
export interface EffectiveModelV2 {
  readonly channelId:string;readonly rawModelId:string;readonly selected:boolean;readonly enabled:boolean;readonly observed:boolean
  readonly binding:Readonly<ModelBindingV2>|null;readonly rules:Readonly<RulesV2>;readonly protocol:Readonly<ProtocolStateV2>
  readonly fieldOrigins:Readonly<Record<string,Readonly<FieldOriginV2>>>;readonly matchedConditionId:string|null;readonly notices:readonly MatchIssueV2[]
}
export interface PatternV2 {kind:'exact'|'prefix'|'glob'|'ascii_numeric_prefix';value:string}
export interface MatchRulesV2 {channelMode:'all'|'whitelist'|'blacklist';channels:string[];caseSensitive:boolean;priority:number;include:PatternV2[];exclude:PatternV2[];rejectRepeatedLiteralPrefix:boolean}
export interface NameLayoutV2 {id:string;literalPrefix:string;versionCapture:'complete_numeric_components';numericJoiners:('.'|'-')[];tailCapture:'literal_remainder';trailingDatePolicy:'retain_valid_YYYYMMDD_after_hyphen';ambiguousNumericTail:'no_equivalence'}
export type SelectorAtomV2 = {kind:'exact_id';value:string}|{kind:'numeric_version';components:string[]}|{kind:'literal_segment';value:string;position:'any'|'before_version'|'after_version';boundary:'hyphen_or_string_edge'}
export interface SelectorV2 {all:SelectorAtomV2[];layoutId?:string;literalTail?:string}
export interface ConditionOverrideV2 {id:string;name:string;selector:SelectorV2;rules:RulesV2;lockedFields:string[];sourceExactId:string;scope:'known_identity_only'}
export interface TemplateSourceRecordV2 {field:string;url:string|null;note:string;recordedAt:string}
export interface TemplateContentV2 {description:string;matching:MatchRulesV2;nameLayouts:NameLayoutV2[];commonRules:RulesV2;conditionOverrides:ConditionOverrideV2[];lockedFields:string[];sources:TemplateSourceRecordV2[]}
export type SaveOriginV2 = {kind:'manual'|'builtin'}|{kind:'external_task'|'internal_agent';taskId:string}
export interface RevisionV2 {id:string;createdAt:string;savedLabel:string;contentVersion:2;content:TemplateContentV2;digest:string;baseRevision:string|null;changes:{field:string;before:unknown;after:unknown}[];origin:SaveOriginV2}
export interface TemplateV2 {id:string;label:string;source:'builtin'|'user';defaultRevision:string|null;revisions:RevisionV2[]}
export interface LibraryV2 {schemaVersion:2;templates:TemplateV2[];submissionReceipts:Readonly<Record<string,unknown>>}
export interface ReadBaseV2 {dtoVersion:2;draftId:string;draftGeneration:number;sourceDigest:string;policyDigest:string;libraryDigest:string;rootRevision:string}
export type ModelEditV2 = {kind:'set_override';channelId:string;rawModelId:string;field:string;value:unknown}|{kind:'clear_override';channelId:string;rawModelId:string;field:string}|{kind:'set_manual_binding';channelId:string;rawModelId:string;reference:RefV2}|{kind:'clear_manual_binding';channelId:string;rawModelId:string}
export interface SaveModelsRequestV2 {dtoVersion:2;expected:ReadBaseV2;edits:ModelEditV2[];scopeChannelId:string|null}
export interface ReadModelDtoV2 {rawModelId:string;outcome:'matched'|'manual_preserved'|'unmatched'|'missing_default'|'missing_dependency'|'conflict';changed:boolean;bindingBefore:ModelBindingV2|null;bindingAfter:ModelBindingV2|null;effective:EffectiveModelV2;issues:MatchIssueV2[]}
export interface ReadMatchDtoV2 {dtoVersion:2;readId:string;base:ReadBaseV2;nextDraftGeneration:number;changed:boolean;models:ReadModelDtoV2[];counts:{total:number;matched:number;manualPreserved:number;unmatched:number;missingDefault:number;missingDependency:number;conflict:number;changed:number;protocolPending:number}}
export type ModelSelectionV2={mode:'all'|'none'}|{mode:'whitelist';models:string[]}
export interface ConnectionV2 {providerId:string;nodes:{id:string;name:string;url:string}[];activeNodeId:string;credentialRef:string|null;enabled:boolean;allowPrivateNetwork:boolean;authMode:string|null;sourceReadId:string|null;importedConfig:{id:string;digest:string}|null}
export interface ModelConfigV2 {rawModelId:string;label:string;observed:boolean;readonly source:SourceModelV2;explicitOverrides:RulesV2;readonly binding:Readonly<ModelBindingV2>|null}
export interface ChannelV2 {id:string;label:string;connection:ConnectionV2;selection:ModelSelectionV2;modelOrder:string[];models:ModelConfigV2[];templateRefs:RefV2[];defaults:RulesV2;familyOverrides:Record<string,RulesV2>;naming:Record<string,{alias?:string;displayName?:string}>;namespace:{channelId:string;removeLeadingPrefixes:string[]}|null}
export interface PolicyV2 {schemaVersion:2;channels:ChannelV2[];templateRefs:RefV2[];defaultChannelId:string|null;pendingRemovals:readonly unknown[];recoveryId:string|null}
export interface DraftV2 {schemaVersion:2;base:ReadBaseV2;readonly policy:PolicyV2;readonly observedBaseline?:Readonly<PolicyV2>|null;readonly credentialEdits?:Readonly<Record<string,{kind:'replace';credentialRef:string}|{kind:'clear'}>>}
export interface FileSnapshotV2 {dtoVersion:2;library:LibraryV2;libraryDigest:string;savedPolicy:PolicyV2;draft:DraftV2|null;sourceDigest:string;rootRevision:string;effective:readonly EffectiveModelV2[]}
export interface ReadExistingRequestV2 {expectedSource:string;expectedLibrary:string;expectedDraft:ReadBaseV2|null}
export interface ReadExistingResponseV2 {snapshot:FileSnapshotV2;results:ReadMatchDtoV2[]}
export type ChannelIntentV2={kind:'rename';channelId:string;name:string}|{kind:'set_selection';channelId:string;selection:ModelSelectionV2}|{kind:'set_enabled';channelId:string;enabled:boolean}|{kind:'set_order';channelId:string;rawModelIds:string[]}|{kind:'set_defaults';channelId:string;rules:RulesV2}|{kind:'set_family_defaults';channelId:string;templateId:string;rules:RulesV2}|{kind:'set_naming';channelId:string;rawModelId:string;alias:string|null;displayName:string|null}|{kind:'set_template_refs';channelId:string;references:RefV2[]}|{kind:'delete';channelId:string}|{kind:'set_active_node';channelId:string;nodeId:string}
export interface FileSaveRequestV2 {expected:ReadBaseV2;edits:ModelEditV2[];channelIntents:ChannelIntentV2[];scopeChannelId?:string|null}
// L0/L1 frozen readonly contract. Registration/normal-crate SHA is tracked separately.
export interface DraftProjectionV2 {expected:ReadBaseV2;effective:EffectiveModelV2[];intended:PolicyV2;digest:string}
export interface StageConnectionRequestV2 {expected:ReadBaseV2|null;expectedSource:string;expectedLibrary:string;channelId:string|null;name:string;url:string;nodes?:ConnectionV2['nodes'];activeNodeId?:string;authMode:'key'|'none';apiKey:string|null;rawModelIds:string[];selectedModelIds:string[]}
export interface DiscoverRequestV2 {discoveryKind:'openai_list'|'anthropic_list'|'google_list'|'ollama_list';url:string;apiKey:string|null}
export interface DiscoverResponseV2 {dtoVersion:2;models:{rawModelId:string;label:string}[];elapsedMs:number}
export type DiscoveryKindV2=DiscoverRequestV2['discoveryKind']
export interface NodeMeasurementSetV2 {runId:string;stopped:boolean;results:{nodeId:string;elapsedMs:number|null;modelCount:number|null;error:string|null}[]}
export interface AgentSettingsV2 {providerId:string;modelId:string;reasoningEffort:string|null;fast:boolean|null;channelId?:string|null;connectionDigest?:string|null}
export interface ExecutionRowV2 {channelId:string;providerId:string;modelId:string;label:string;rules:RulesV2;protocol:ProtocolStateV2;selected:boolean;enabled:boolean;nativeModelId:string|null;unavailableReason:string|null;aliases:string[];connectionDigest:string|null;pickerVisible:boolean;actualSelector:string|null}
export interface ExecutionDirectoryV2 {dtoVersion:2;sourceDigest:string;rootRevision:string;libraryDigest:string;channels:{id:string;label:string;providerId:string}[];rows:ExecutionRowV2[]}
export type AgentSceneV2='from_sources'|'update_models'|'audit_and_revise'|'private_models'|'preferences'|'check_only'
export interface AgentMessageV2 {role:'user'|'assistant';text:string}
export interface ResearchSourceV2 {url:string;retrievedAt:string;text:string;links:string[];receiptId?:string;contentDigest?:string}
export interface EditingBufferInputV2 {target:RefV2|null;label:string;content:TemplateContentV2;generation:number}
export interface EditingBufferV2 extends EditingBufferInputV2 {digest:string}
export interface GenerateIntentV2 {scene:AgentSceneV2;targets:RefV2[];allowNew:boolean;expectedLibrary:string;requirements:string;history:AgentMessageV2[];sources:ResearchSourceV2[];buffer?:EditingBufferV2|null}
export interface ExternalBrokerBundleV2 {taskId:string;content:string;preparedTicket:string;expiresAt:string}
export interface ExternalRunViewV2 {dtoVersion:2;taskId:string;scene:AgentSceneV2;expectedLibrary:string;targets:RefV2[];allowNew:boolean;result:GeneratedReviewV2}
export type RunPhaseV2='prepared'|'running'|'review'|'stopped'|'failed'|'accepted'
export interface RunTicketV2 {dtoVersion:2;runId:string;phase:RunPhaseV2;expectedLibrary:string;rootRevision:string;sourceDigest:string}
export interface ContentProposalV2 {target:RefV2|null;label:string;contentVersion:2;content:TemplateContentV2}
export interface AgentProposalResponseV2 {contractVersion:2;expectedLibrary:string;proposals:ContentProposalV2[]}
export interface ProposalDiffV2 {fieldPath:string;before:unknown|null;after:unknown|null}
export interface ProposalResultV2 {index:number;proposal:ContentProposalV2;plannedRevision:RefV2|null;differences:ProposalDiffV2[];dependencies:RefV2[];issues:MatchIssueV2[]}
export interface ProposalReviewV2 {dtoVersion:2;expectedLibrary:string;valid:boolean;proposals:ProposalResultV2[];issues:MatchIssueV2[]}
export interface CheckReportV2 {summary:string;issues:{code:string;message:string;fieldPath:string|null}[]}
export interface GeneratedReviewV2 {dtoVersion:2;runId:string;phase:RunPhaseV2;expectedLibrary:string;review:ProposalReviewV2|null;report:CheckReportV2|null;errorCode:string|null;responseDigest:string|null}

// View-only props and explicit editing intents below are not IPC wire contracts.
// The L0 adapter will supply these services after public commands are registered.
export interface ConditionPreviewView {
  rawModelId:string;matchingView:string|null;matched:boolean;reason:string
  conditions:readonly {conditionId:string;matched:boolean;reason:string;atoms:readonly {index:number;matched:boolean;reason:string}[]}[]
  effective:Readonly<EffectiveModelV2>|null;issues:readonly MatchIssueV2[]
  configuration?:Readonly<Pick<EffectiveModelV2,'rules'|'protocol'|'fieldOrigins'>>
}
export interface ConditionPreviewInput {content:TemplateContentV2;channelId:string;rawModelId:string}
export interface LibraryViewV2 {dtoVersion:2;library:LibraryV2;libraryDigest:string}
export type LibraryIntentV2={kind:'set_default';reference:RefV2}|{kind:'clear_default';reference:RefV2}|{kind:'delete_revisions';templateId:string;revisionIds:string[]}
export interface LibraryChangeV2 {expectedLibrary:string;intent:LibraryIntentV2}
export type UsageStateV2='saved'|'draft'|'additional'|'library'
export type UsageKindV2='global_reference'|'channel_reference'|'manual'|'auto_read'|'additional_exact'|'base_revision'
export interface RevisionUseV2 {state:UsageStateV2;kind:UsageKindV2;reference:RefV2;channelId:string|null;rawModelId:string|null;dependentRevision:RefV2|null}
export interface UsageModelV2 {state:UsageStateV2;channelId:string;channelLabel:string;rawModelId:string;matched:boolean;matchedConditionId:string|null;bindingKind:UsageKindV2|null;usingQueriedRevision:boolean;issues:MatchIssueV2[]}
export interface UsageReportV2 {dtoVersion:2;reference:RefV2;libraryDigest:string;savedPolicyDigest:string;draftPolicyDigest:string|null;isDefault:boolean;builtinInitial:boolean;references:RevisionUseV2[];models:UsageModelV2[]}
export interface RevisionEditIntentV2 {expectedLibrary:string;templateId:string;baseRevision:string|null;label:string;content:TemplateContentV2;makeDefault:boolean}
export interface PreviewConditionsResponseV2 {dtoVersion:2;previewOnly:true;contentDigest:string;rows:{rawModelId:string;familyMatched:boolean;matchedConditionId:string|null;rules:RulesV2;protocol:ProtocolStateV2;fieldOrigins:Record<string,FieldOriginV2>}[]}
export type TemporaryDiscoveryKind = 'compatible_models'|'anthropic_models'|'google_models'|'ollama_models'
export interface ChannelConnectionEditIntentV2 {
  channelId:string;kind:'create'|'edit_connection';label:string
  nodes:{id:string;name:string;url:string}[];activeNodeId:string;authMode:'key'|'none'
  selectedModelIds:string[];modelOrder:string[];modelLabels:Record<string,string>;discoveryReadId:string|null
}
export interface ChannelDiscoveryInputV2 {kind:TemporaryDiscoveryKind;url:string;authMode:'key'|'none';apiKey:string|null}
export interface ChannelDiscoveryViewV2 {readId?:string;models:{id:string;label:string}[];elapsedMs:number}
export interface ModelV2Services {
  loadLibrary?:()=>Promise<LibraryViewV2>
  validateContent?:(content:TemplateContentV2)=>Promise<readonly MatchIssueV2[]>
  previewConditions?:(input:ConditionPreviewInput)=>Promise<ConditionPreviewView>
  saveRevision?:(intent:RevisionEditIntentV2)=>Promise<LibraryViewV2>
  previewChannel?:(input:ChannelDiscoveryInputV2)=>Promise<ChannelDiscoveryViewV2>
  stageConnection?:(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'})=>Promise<{channelId:string}>
  // UI adapter intent, not an invented RPC. L0 supplies the actual v2 command DTOs.
  templateAction?:(intent:TemplateActionIntentV2)=>Promise<TemplateActionViewV2>
}
export interface TemplateActionIntentV2 {kind:'set_default'|'delete_versions'|'agent_fill'|'agent_audit'|'external_task'|'usage';expectedLibrary:string;templateId:string;references:RefV2[];content:TemplateContentV2|null;label:string;targets?:readonly {templateId:string;label:string;reference:RefV2|null}[]}
export interface TemplateActionViewV2 {library?:LibraryViewV2;report?:string;proposal?:{label:string;content:TemplateContentV2}}
export const protocolOptionsV2 = [
  {value:'openai-chat',label:'OpenAI Chat'},{value:'openai-responses',label:'OpenAI Responses'},
  {value:'anthropic',label:'Anthropic'},{value:'google',label:'Google'},{value:'ollama-native',label:'Ollama'}
] as const
export function originLabelV2(origin:Readonly<FieldOriginV2>|undefined):string {
  if(!origin)return '无可核验来源'
  switch(origin.kind){case 'source':return '实际导入事实';case 'template_common':return '模版通用';case 'template_condition':return '条件覆盖';case 'channel_default':return '明确的非协议渠道设置';case 'channel_family':return '明确的非协议族设置';case 'explicit_model':return '单模型人工覆盖';default:return '来源待核对'}
}
export const emptyRulesV2=():RulesV2=>({protocol:{mode:'inherit'},contextWindow:{mode:'inherit'},inputModalities:{mode:'inherit'},reasoningEfforts:{mode:'inherit'},defaultEffort:{mode:'inherit'},fast:{mode:'inherit'}})
export const emptyContentV2=():TemplateContentV2=>({description:'',matching:{channelMode:'all',channels:[],caseSensitive:false,priority:0,include:[{kind:'prefix',value:''}],exclude:[],rejectRepeatedLiteralPrefix:true},nameLayouts:[],commonRules:emptyRulesV2(),conditionOverrides:[],lockedFields:[],sources:[]})
export function serviceUnavailableV2():Error {return new Error('此操作暂不可用，当前输入已保留。请稍后重新读取。')}
export function reviewedAgentResponseV2(result:GeneratedReviewV2):string {
  if(result.phase!=='review'||result.report||!result.review?.valid||result.review.dtoVersion!==2||result.review.expectedLibrary!==result.expectedLibrary)throw new Error('当前结果不是可接受的有效建议；检查报告不能用于保存。')
  const items=[...result.review.proposals].sort((a,b)=>a.index-b.index)
  if(!items.length||items.some((item,index)=>item.index!==index||item.proposal.contentVersion!==2||item.issues.length))throw new Error('建议顺序或完整性尚未通过核对，未生成保存请求。')
  return JSON.stringify({contractVersion:2,expectedLibrary:result.expectedLibrary,proposals:items.map(item=>item.proposal)} satisfies AgentProposalResponseV2)
}

// Public command signatures supplied by L0. Preview references are in-memory only.
export const modelV2Api={
  library:()=>invoke<LibraryViewV2>('model_v2_library'),
  changeLibrary:(request:LibraryChangeV2)=>invoke<LibraryViewV2>('model_v2_change_library',{request}),
  templateUsage:(templateId:string,revisionId:string)=>invoke<UsageReportV2>('model_v2_template_usage',{templateId,revisionId}),
  validateContent:(content:TemplateContentV2)=>invoke<void>('model_v2_validate_content',{content}),
  previewConditions:(content:TemplateContentV2,rawModelIds:string[],channelId:string)=>invoke<PreviewConditionsResponseV2>('model_v2_preview_conditions',{request:{content,rawModelIds,channelId}}),
  saveRevision:(request:RevisionEditIntentV2)=>invoke<LibraryViewV2>('model_v2_save_revision',{request}),
  fileSnapshot:()=>invoke<FileSnapshotV2>('model_v2_file_snapshot'),
  beginDraft:(request:ReadExistingRequestV2)=>invoke<FileSnapshotV2>('model_v2_begin_draft',{request}),
  readExisting:(request:ReadExistingRequestV2)=>invoke<ReadExistingResponseV2>('model_v2_read_existing',{request}),
  stageConnection:(request:StageConnectionRequestV2)=>invoke<FileSnapshotV2>('model_v2_stage_connection',{request}),
  fileSave:(request:FileSaveRequestV2)=>invoke<FileSnapshotV2>('model_v2_file_save',{request}),
  fileProject:(request:FileSaveRequestV2)=>invoke<DraftProjectionV2>('model_v2_file_project',{request}),
  copyChannel:(request:{expected:ReadBaseV2;channelId:string;name:string;reuseSourceKey:boolean})=>invoke<FileSnapshotV2>('model_v2_copy_channel',{request}),
  credentialView:(channelId:string)=>invoke<{configured:boolean;value:string;source:string;viewable:boolean}>('model_v2_credential_view',{channelId}),
  stageCredential:(request:{expected:ReadBaseV2;channelId:string;action:'replace'|'clear';value?:string})=>invoke<FileSnapshotV2>('model_v2_stage_credential',{request}),
  measureNodes:(request:{expected:ReadBaseV2;channelId:string;nodeIds:string[];discoveryKind:DiscoveryKindV2;runId:string})=>invoke<NodeMeasurementSetV2>('model_v2_measure_nodes',{request}),
  stopMeasurement:(runId:string)=>invoke<void>('model_node_measure_stop',{runId}),
  discover:(request:DiscoverRequestV2)=>invoke<DiscoverResponseV2>('model_v2_discover',{request}),
  executionDirectory:()=>invoke<ExecutionDirectoryV2>('model_v2_execution_directory'),
  agentSettings:()=>invoke<AgentSettingsV2|null>('model_agent_preferences'),
  bindAgent:(settings:AgentSettingsV2)=>invoke<AgentSettingsV2>('model_v2_bind_agent',{settings}),
  externalPrepare:(request:GenerateIntentV2)=>invoke<ExternalBrokerBundleV2>('model_v2_external_prepare',{request}),
  externalActivate:(taskId:string,preparedTicket:string)=>invoke<void>('model_v2_external_activate',{taskId,preparedTicket}),
  externalRevoke:(taskId:string)=>invoke<void>('model_v2_external_revoke',{taskId}),
  externalRun:(taskId:string)=>invoke<ExternalRunViewV2>('model_v2_external_run',{taskId}),
  agentBuffer:(request:EditingBufferInputV2)=>invoke<EditingBufferV2>('model_v2_agent_buffer',{request}),
  agentPrepare:(request:GenerateIntentV2)=>invoke<RunTicketV2>('model_v2_agent_prepare',{request}),
  agentGenerate:(runId:string)=>invoke<GeneratedReviewV2>('model_v2_agent_generate',{runId}),
  agentStop:(runId:string)=>invoke<void>('model_v2_agent_stop',{runId}),
  agentLatest:(templateId:string|null)=>invoke<GeneratedReviewV2|null>('model_v2_agent_latest',{templateId}),
  agentAccept:(request:{runId:string;expectedLibrary:string;reviewedResponse:string})=>invoke<GeneratedReviewV2>('model_v2_agent_accept',{request}),
  researchSources:(urls:string[])=>invoke<ResearchSourceV2[]>('model_research_sources',{urls}),
  searchSources:(query:string)=>invoke<ResearchSourceV2[]>('model_research_search',{query}),
}
