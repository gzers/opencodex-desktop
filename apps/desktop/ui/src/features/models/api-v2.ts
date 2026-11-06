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
export interface DraftV2 {schemaVersion:2;base:ReadBaseV2;readonly policy:PolicyV2}
export interface FileSnapshotV2 {dtoVersion:2;library:LibraryV2;libraryDigest:string;savedPolicy:PolicyV2;draft:DraftV2|null;sourceDigest:string;rootRevision:string;effective:readonly EffectiveModelV2[]}
export interface ReadExistingRequestV2 {expectedSource:string;expectedLibrary:string;expectedDraft:ReadBaseV2|null}
export interface ReadExistingResponseV2 {snapshot:FileSnapshotV2;results:ReadMatchDtoV2[]}
export type ChannelIntentV2={kind:'rename';channelId:string;name:string}|{kind:'set_selection';channelId:string;selection:ModelSelectionV2}|{kind:'set_enabled';channelId:string;enabled:boolean}|{kind:'set_order';channelId:string;rawModelIds:string[]}
export interface FileSaveRequestV2 {expected:ReadBaseV2;edits:ModelEditV2[];channelIntents:ChannelIntentV2[]}
export interface StageConnectionRequestV2 {expected:ReadBaseV2|null;expectedSource:string;expectedLibrary:string;channelId:string|null;name:string;url:string;nodes?:ConnectionV2['nodes'];activeNodeId?:string;authMode:'key'|'none';apiKey:string|null;rawModelIds:string[];selectedModelIds:string[]}
export interface DiscoverRequestV2 {discoveryKind:'openai_list'|'anthropic_list'|'google_list'|'ollama_list';url:string;apiKey:string|null}
export interface DiscoverResponseV2 {dtoVersion:2;models:{rawModelId:string;label:string}[];elapsedMs:number}

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
}
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

// Public command signatures supplied by L0. Preview references are in-memory only.
export const modelV2Api={
  library:()=>invoke<LibraryViewV2>('model_v2_library'),
  validateContent:(content:TemplateContentV2)=>invoke<void>('model_v2_validate_content',{content}),
  previewConditions:(content:TemplateContentV2,rawModelIds:string[],channelId:string)=>invoke<PreviewConditionsResponseV2>('model_v2_preview_conditions',{request:{content,rawModelIds,channelId}}),
  saveRevision:(request:RevisionEditIntentV2)=>invoke<LibraryViewV2>('model_v2_save_revision',{request}),
  fileSnapshot:()=>invoke<FileSnapshotV2>('model_v2_file_snapshot'),
  beginDraft:(request:ReadExistingRequestV2)=>invoke<FileSnapshotV2>('model_v2_begin_draft',{request}),
  readExisting:(request:ReadExistingRequestV2)=>invoke<ReadExistingResponseV2>('model_v2_read_existing',{request}),
  stageConnection:(request:StageConnectionRequestV2)=>invoke<FileSnapshotV2>('model_v2_stage_connection',{request}),
  fileSave:(request:FileSaveRequestV2)=>invoke<FileSnapshotV2>('model_v2_file_save',{request}),
  discover:(request:DiscoverRequestV2)=>invoke<DiscoverResponseV2>('model_v2_discover',{request}),
}
