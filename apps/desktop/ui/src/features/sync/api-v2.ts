// Frozen sync_v2_types.rs / objects_overlay_v2.rs; no v1 ModelPackage/ETag fallback.
import {invoke} from '@tauri-apps/api/core'
import type {RefV2} from '@/features/models/api-v2'

export type LocalRelationV2 = 'not_compared'|'observed_equal'|'changed'|'unknown'
export type RemoteRelationV2 = 'unknown'|'observed_equal'|'descendant'|'conflict'|'initialization_ambiguous'|'generation_changed'
export type RecoveryKindV2 = 'verify_original_upload'|'refresh_remote'|'repreview_upload'|'repreview_download'|'choose_conflict_source'|'prepare_recovery_download'|'prepare_recovery_export'|'select_empty_directory'|'resume_original_upload'|'return_to_original_connection'|'unlock_payload'|'inspect_local_protection'|'free_remote_space'|'retry_later'|'inspect_error_details'
export type ErrorCodeV2 = 'sync_generation_ambiguous'|'sync_generation_changed'|'sync_listing_incomplete'|'sync_remote_invalid'|'sync_remote_conflict'|'sync_operation_unknown'|'sync_operation_collision'|'sync_source_changed'|'sync_scope_changed'|'sync_endpoint_changed'|'sync_payload_schema_unsupported'|'sync_payload_invalid'|'sync_limit'|'sync_remote_full'|'sync_auth_required'|'sync_network'|'sync_collection_missing'|'sync_passphrase_required'|'sync_passphrase_invalid'|'sync_plan_expired'|'sync_plan_changed'|'sync_local_changed'|'sync_backup_failed'|'sync_journal_invalid'|'sync_local_state_io'|'sync_cancelled'
export type StepV2 = 'observe'|'initialize'|'source_guard'|'payload_put'|'payload_get'|'commit_put'|'commit_get'|'verify'|'download_guard'|'apply_local'|'persist'|'decode'|'recovery'
export type RetryabilityV2 = 'never'|'after_refresh'|'same_operation_read_only'|'same_operation_guarded'
export type CauseV2 = 'missing'|'unauthorized'|'forbidden'|'timeout'|'disconnected'|'tls'|'redirect'|'invalid_listing'|'invalid_href'|'partial_response'|'too_large'|'insufficient_storage'|'server'|'protocol'|'invalid_json'|'invalid_digest'|'missing_parent'|'cycle'|'unsupported_schema'|'source_changed'|'scope_changed'|'binding_changed'|'local_io'|'backup'|'passphrase'|'cancelled'|'expired'|'collision'|'generation_changed'|'generation_ambiguous'
export type RemoteStateV2 = 'empty'|'generation_empty'|'ready'|'conflict'|'initialization_ambiguous'|'incomplete'|'invalid'
export type OperationKindV2 = 'upload'|'download'
export type PhaseV2 = 'prepared'|'initialization_may_be_written'|'generation_verified'|'payload_may_be_written'|'payload_verified'|'commit_may_be_written'|'settled'|'cancelled_before_write'
export type WriteStepV2 = 'none'|'generation'|'payload'|'commit'
export type StorageStatusV2 = 'not_attempted'|'unknown'|'stored'|'invalid'
export type LocalApplyStatusV2 = 'not_applicable'|'not_attempted'|'unknown'|'applied'|'invalid'
export interface GenerationRefV2 {generationId:string;digest:string}
export interface CommitRefV2 {operationId:string;digest:string}
export interface RecoveryActionV2 {kind:RecoveryKindV2;operationId?:string;recoveryPlanId?:string;generation?:GenerationRefV2;head?:CommitRefV2}
export interface SyncFailureV2 {code:ErrorCodeV2;step:StepV2;retryability:RetryabilityV2;cause?:CauseV2;httpStatus?:number;recoveryActions:RecoveryActionV2[]}
export interface RemoteObservationV2 {dtoVersion:1;bindingId:string;state:RemoteStateV2;generation?:GenerationRefV2;generations:GenerationRefV2[];heads:CommitRefV2[];orphanCount:number;staleCount:number;viewDigest:string;observedAt:string;problems:SyncFailureV2[];recoveryCandidates:{generation:GenerationRefV2;head:CommitRefV2}[];recoveryActions:RecoveryActionV2[]}
export interface OperationFactV2 {dtoVersion:1;operationId:string;kind:OperationKindV2;phase:PhaseV2;lastWriteStep:WriteStepV2;storageStatus:StorageStatusV2;localApplyStatus:LocalApplyStatusV2;relation:{local:LocalRelationV2;remote:RemoteRelationV2};generation?:GenerationRefV2;payloadDigest:string;commit?:CommitRefV2;storedAt?:string;observation?:RemoteObservationV2;failure?:SyncFailureV2;recoveryActions:RecoveryActionV2[]}
export interface PayloadSchemaRefV2 {envelope:{format:string;version:number};document:{format:string;version:number};model:{format:string;formatVersion:number;schemaVersion:number}}
export interface SourceGuardV2 {savedSourceDigest:string;scopeDigest:string;mcpSourcesDigest:string}
export interface UploadPlanV2 {dtoVersion:1;operationId:string;planDigest:string;bindingId:string;initialization:'none'|'on_confirm';generation?:GenerationRefV2;parents:CommitRefV2[];intent:'update'|'resolve';resolution?:{selectionMode:'head'|'merged';selectedHead?:CommitRefV2;scopeDigest:string;previewDigest:string};payloadDigest:string;payloadBytes:number;payloadSchema:PayloadSchemaRefV2;protectionMode:'plain'|'protected';sourceGuard:SourceGuardV2;expectedViewDigest:string;preparedAt:string;expiresAt:string}
export interface DownloadPlanV2 {dtoVersion:1;operationId:string;planDigest:string;bindingId:string;generation:GenerationRefV2;head:CommitRefV2;payloadDigest:string;scopeDigest:string;targetDigest:string;expectedViewDigest:string;backupBeforeUpdate:boolean;preparedAt:string;expiresAt:string}
export interface ResumeUploadRequestV2 {operationId:string;expectedViewDigest:string;planDigest:string}
export interface SelectionV2 {channels:{portableChannelId:string;modelIds:string[]|null;familyIds:string[]|null;includeConnection:boolean;includeKey:boolean;includeAuthorization:boolean}[];templateRefs:RefV2[];adoptTemplateDefaults:RefV2[];includeGlobalTemplateRefs:boolean;manager:{preferences:boolean;extensions:boolean;accountIds:string[];webdavIds:string[];webdavPasswordIds:string[]};skillNames:string[];mcpSources:{name:string;definitionDigest:string}[];extensionCredentials:{skillNames:string[];mcpNames:string[]};usage:'remote_sync'|'local_file'}
export const emptySelectionV2=():SelectionV2=>({channels:[],templateRefs:[],adoptTemplateDefaults:[],includeGlobalTemplateRefs:false,manager:{preferences:false,extensions:false,accountIds:[],webdavIds:[],webdavPasswordIds:[]},skillNames:[],mcpSources:[],extensionCredentials:{skillNames:[],mcpNames:[]},usage:'remote_sync'})
// Service DTOs read from the L2 contribution source. Final source SHA is pending;
// only public command names explicitly frozen by L0 are called here.
export type ObjectKindV2='channel'|'template'|'preferences'|'extensions'|'account'|'skill'|'mcp'
export interface ModelInventoryV2 {rawModelId:string;label:string;observed:boolean;selected:boolean;availableLocal:boolean;availableCloud:boolean;familyId:string|null}
export interface ObjectEntryV2 {kind:ObjectKindV2;id:string;label:string;availableLocal:boolean;availableCloud:boolean;modelIds:string[];models:ModelInventoryV2[];familyIds:string[];revisions:RefV2[];defaultRevision:RefV2|null;secretAvailable:boolean}
export interface McpSourceEntryV2 {name:string;sources:{source:string;definitionDigest:string}[];differingFields:string[]}
export interface ScopeRequestV2 {selection:SelectionV2;mcpClients:Record<string,string>;assetsReviewed:boolean}
export interface CatalogV2 {connectionState:string;entries:ObjectEntryV2[];mcpSources:McpSourceEntryV2[];selected:ScopeRequestV2;initialized:boolean;cloudState:RemoteStateV2;heads:CommitRefV2[];recoveryActions:RecoveryActionV2[]}
export interface PreviewRequestV2 extends ScopeRequestV2 {selectedHead:CommitRefV2|null;forcedDirection:'upload'|'download'|null;backupBeforeUpdate:boolean}
export interface DiffEntryV2 {kind:ObjectKindV2;id:string;status:'add'|'update'|'unchanged'|'unavailable'|'source_required';beforeDigest:string|null;afterDigest:string|null}
export interface PublicSummaryV2 {schema:PayloadSchemaRefV2;templateIds:string[];channelIds:string[];accountCount:number;skillCount:number;mcpCount:number;hasPreferences:boolean;hasExtensions:boolean;webdavCount:number;channelKeyCount:number;channelAuthorizationCount:number;webdavPasswordCount:number;extensionSecretCount:number;contentVersionCount:number}
export interface PreviewV2 {sessionId:string;planDigest:string;chosenDirection:'upload'|'download'|'no_change';automaticReason:'same_semantic_content'|'both_empty'|'cloud_empty'|'local_empty'|'local_saved_later'|'cloud_saved_later'|'unreliable_save_time'|'equal_time_different_content'|'explicit_upload'|'explicit_download';localSavedTime:string|null;cloudSavedTime:string|null;noChange:boolean;backupBeforeUpdate:boolean;entries:DiffEntryV2[];localSummary:PublicSummaryV2;cloudSummary:PublicSummaryV2|null;plan:{kind:'upload';plan:UploadPlanV2}|{kind:'download';plan:DownloadPlanV2}|{kind:'no_change'}}
export const configV2Api={
 catalog:(readPassphrase:string|null)=>invoke<CatalogV2>('config_v2_catalog',{readPassphrase}),
 preview:(request:PreviewRequestV2,readPassphrase:string|null,writePassphrase:string|null)=>invoke<PreviewV2>('config_v2_preview',{request,readPassphrase,writePassphrase}),
 confirm:(request:{sessionId:string;planDigest:string})=>invoke<OperationFactV2>('config_v2_confirm',{request}),
 result:(operationId:string)=>invoke<OperationFactV2>('config_v2_result',{operationId}),
 verify:(operationId:string,readPassphrase:string|null)=>invoke<OperationFactV2>('config_v2_verify',{operationId,readPassphrase}),
 resume:(request:ResumeUploadRequestV2,readPassphrase:string|null)=>invoke<OperationFactV2>('config_v2_resume',{request,readPassphrase}),
 cancel:(sessionId:string)=>invoke<OperationFactV2|null>('config_v2_cancel',{sessionId}),
 readScope:()=>invoke<ScopeRequestV2|null>('config_v2_read_scope'),
 saveScope:(request:ScopeRequestV2)=>invoke<void>('config_v2_save_scope',{selection:request.selection,mcpClients:request.mcpClients,assetsReviewed:request.assetsReviewed}),
}
export const recoveryLabelsV2:Record<RecoveryKindV2,string>={verify_original_upload:'核对原上传（只读）',refresh_remote:'重新读取云端',repreview_upload:'重新预览上传',repreview_download:'重新预览下载',choose_conflict_source:'选择并发配置来源',prepare_recovery_download:'查看专用恢复下载方案',prepare_recovery_export:'查看专用恢复导出方案',select_empty_directory:'选择另一个空目录',resume_original_upload:'确认续传原上传',return_to_original_connection:'核对原连接',unlock_payload:'解锁所选配置',inspect_local_protection:'查看本机保护记录',free_remote_space:'查看容量处理说明',retry_later:'稍后重新核对',inspect_error_details:'查看错误详情'}
