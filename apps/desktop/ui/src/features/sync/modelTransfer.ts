import {open,type OpenDialogOptions} from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import type { ChannelPolicy, ModelPolicy, ModelSnapshot, Template, TemplateRef, DiscoveredModel } from '@/features/models/api'
export interface DraftCandidate {expectedLibrary:string;expectedPolicy:string;expectedWorkspace:string}
export interface TransferChoice {preferences?:boolean;extensions?:boolean;channelModels?:Record<string,string[]>;channelFamilies?:Record<string,FamilyScope>;selectedIds:string[];credentialIds:string[];versions:Record<string,string>;mcpSources:Record<string,string>;includeGlobal:boolean;webdavPasswords:boolean}
export const transferChoiceApi={load:(remote:boolean)=>invoke<TransferChoice|null>('transfer_choices',{remote}),save:(remote:boolean,choice:TransferChoice)=>invoke<TransferChoice>('save_transfer_choices',{remote,choice})}
export interface TransferProtection {enabled:boolean;hasPassword:boolean}
export const transferProtectionApi={load:()=>invoke<TransferProtection>('transfer_protection'),save:(enabled:boolean,passphrase?:string)=>invoke<TransferProtection>('save_transfer_protection',{enabled,passphrase:passphrase??null})}
export interface FamilyScope {templateIds:string[];includeChannelDefaults:boolean}
export interface TransferScope {channelFamilies?:Record<string,FamilyScope>;channelModels?:Record<string,string[]>; includePolicy: boolean; channelIds: string[]; templateRefs: TemplateRef[]; includeGlobal: boolean }
export interface AccountOrigin {version:number;accountPlatform:string;owner:string;authKind:string;sourceNamespace:string;sourceAccountId:string;tenant:string|null}
export interface PortableAccount {id:string;provider:string;subject:string|null;label:string;email:string|null;origin?:AccountOrigin}
export type LoginTransfer = 'manualImportDisabled'|'crossDeviceEvidenceMissing'|'dedicatedAdapterRequired'|'unsupportedProvider'
export interface AccountCatalogEntry {account:PortableAccount;loginTransfer:LoginTransfer;localSource:boolean;informationSupported:boolean}
export const accountPlatforms:Record<string,string>={chatgpt:'ChatGPT',antigravity:'Google Antigravity',kiro:'Kiro',muse:'Muse'}
export function accountInformationSupported(account:PortableAccount):boolean {
  const expected=({chatgpt:'chatgpt','google-antigravity':'antigravity',kiro:'kiro',muse:'muse'} as Record<string,string>)[account.provider]
  if(!expected)return false
  const origin=account.origin;if(!origin)return true
  return origin.version===1&&origin.accountPlatform===expected&&origin.owner==='opencodex'&&origin.sourceNamespace===(account.provider==='chatgpt'?'codex-account-pool':'oauth-account-pool')&&(account.provider==='kiro'?['unknown','kiro_desktop','aws_sso_oidc'].includes(origin.authKind):origin.authKind==='oauth')
}
export function accountInformationLabel(account:PortableAccount):string {
  const origin=account.origin
  const label=origin?[(accountPlatforms[origin.accountPlatform]??origin.accountPlatform),account.email].filter(Boolean).join(' · '):'旧格式 · 来源未登记'
  return label+' · '+(accountInformationSupported(account)?'仅账户信息 · 新设备需登录':'暂不支持此来源的信息同步')
}
export function accountTransferLabel(provider:string):string {
  if(provider==='chatgpt')return '需重新登录 · 官方不支持手工导入登录态'
  if(provider==='google-antigravity')return '需重新登录 · 免登录迁移尚未验证'
  if(provider==='kiro'||provider==='muse')return '需重新登录 · 需要专用设备授权适配'
  return '需重新登录 · 暂未支持此类型登录态迁移'
}
/** S18-03：由客户端应用维护、含本机路径的 MCP 默认不同步，可手动勾选；与后端 DEFAULT_EXCLUDED_MCPS 一致。 */
export const DEFAULT_EXCLUDED_MCPS:readonly string[]=['node_repl']
export const defaultIncludedMcp=(name:string)=>!DEFAULT_EXCLUDED_MCPS.includes(name)
/** 同名 MCP 在各客户端定义不同且未选来源；只含客户端 id 与差异字段名，不含值。 */
export interface McpConflict {name:string;clients:string[];fields:string[]}
export interface ManagerScope {accountIds?:string[];preferences:boolean;extensions:boolean;webdav:boolean;webdavPasswords:boolean;skillNames?:string[];mcpNames?:string[];mcpSources?:Record<string,string>;extensionCredentials?:boolean;credentialIds?:string[];assetsReviewed?:boolean}
export interface AssetNameMappings {skills:Record<string,string>;mcps:Record<string,string>}
export interface AssetSummary {skills:{name:string;digest:string;files:number;bytes:number;warnings:string[];protected:boolean}[];mcps:{name:string;digest:string;secretFields:string[];warnings:string[]}[];conflicts?:McpConflict[]}
export interface PortableEndpoint {id:string;url:string;remotePath:string;username:string;conflictPolicy:string}
export interface ManagerSettings {preferences:Record<string,unknown>|null;extensions:{skills:string[];servers:string[];sync_method:string}|null;webdav:PortableEndpoint[];accounts?:PortableAccount[]}
export interface ModelPackage { format: string; formatVersion: number; schemaVersion: number; createdAt: string; digest: string; payload: { candidate?:DraftCandidate; templates: Template[]; policy: { templateRefs: TemplateRef[] | null; channelOrder: string[] | null;defaultChannelId?:string|null; channels: (Pick<ChannelPolicy, 'id' | 'label' | 'selection' | 'naming'> & {familyScope?:FamilyScope;modelSubset?:string[]|null;discovered?:DiscoveredModel[];modelOrder?:string[];connection?:{nodes:{id:string;name:string;url:string}[];provider?:Record<string,unknown>|null}|null})[] } | null } }
export interface CredentialCounts {channelKeys:number;extensions:number;webdavPasswords:number}
export interface FilePlan {credentialCounts?:CredentialCounts|null; id: string; package: ModelPackage;protected?:boolean;fileDigest?:string;credentialCount?:number;manager?:ManagerSettings|null;assets?:AssetSummary|null }
/** S18-01：设备间同步方向。local 表示本机较新、上传；remote 表示云端较新、下载。 */
export interface SyncCheck {source:'local'|'remote';reason?:string;hasRemote:boolean;lastSyncedAt?:string}
export interface SyncChange {kind:string;id:string;label:string;mark:'new'|'update'|'conflict';version?:number;before?:number;after?:number;clients?:string[];filesAdded?:string[];filesUpdated?:string[];filesRemoved?:string[];fields?:string[]}
/** changes 为 null 表示无法比较变化，仍允许确认。 */
export interface SyncPreview {source:'local'|'remote'|'file';changes:SyncChange[]|null;unchanged:number;createdAt:string}
export interface UploadPlan extends FilePlan { phase: string;sync?:SyncPreview|null }
export interface IncomingPlan extends FilePlan { comparison?:SyncPreview|null;modelsAccepted?:boolean;sync?:SyncPreview|null;expectedLibrary: string; expectedPolicy: string; expectedWorkspace: string; mergedPolicy?:ModelPolicy|null; notices: string[];previewError?:string|null;availableKeys?:string[];availableWebdavPasswords?:string[];availableExtensionIds?:string[];availableExtensionCredentials?:boolean;assetMappings?:AssetNameMappings;channelBindings?:Record<string,{targetId:string;keepLocalConnection:boolean;label:string|null}>;adoption?:{keys:boolean;webdav:boolean;completed:boolean;extensionCredentials?:boolean}|null }
export type ChannelImportMapping = {mode:'new';label:string}|{mode:'local';targetId:string}
export interface ReceiveScope {channelModels?:Record<string,string[]>;assetMappings?:AssetNameMappings;channelMappings?:Record<string,ChannelImportMapping>;accountIds?:string[];templateRefs:TemplateRef[];channelIds:string[];includeGlobal:boolean;preferences?:boolean;extensions?:boolean;endpointIds?:string[];skillNames?:string[];mcpNames?:string[]}
export interface RemoteScope {channelFamilies?:Record<string,FamilyScope>;channelModels?:Record<string,string[]>;accountIds?:string[];templateIds:string[];channelIds:string[];includeGlobal:boolean;preferences:boolean;extensions:boolean;includeKeys:boolean;skillNames?:string[];mcpNames?:string[];mcpSources?:Record<string,string>;extensionCredentials?:boolean;credentialIds?:string[];assetsReviewed?:boolean;allTemplates?:boolean;allChannels?:boolean;allSkills?:boolean;allMcps?:boolean}
export interface RemoteOptions {accountIds?:string[];preferences:boolean;extensions:boolean;includeKeys:boolean;skillNames?:string[];mcpNames?:string[];mcpSources?:Record<string,string>;extensionCredentials?:boolean;credentialIds?:string[];assetsReviewed?:boolean;allTemplates?:boolean;allChannels?:boolean;allSkills?:boolean;allMcps?:boolean;passphrase?:string}
export function remoteScope(scope:TransferScope,options?:RemoteOptions):RemoteScope{return {...(scope.channelFamilies?{channelFamilies:scope.channelFamilies}:{}),...(scope.channelModels?{channelModels:scope.channelModels}:{}),...(options?.accountIds?{accountIds:options.accountIds}:{}),templateIds:[...new Set(scope.templateRefs.map(reference=>reference.templateId))],channelIds:scope.channelIds,includeGlobal:scope.includeGlobal,preferences:options?.preferences??true,extensions:options?.extensions??true,includeKeys:options?.includeKeys??false,...(options?.skillNames?{skillNames:options.skillNames}:{}),...(options?.mcpNames?{mcpNames:options.mcpNames}:{}),...(options?.mcpSources&&Object.keys(options.mcpSources).length?{mcpSources:options.mcpSources}:{}),...(options?.extensionCredentials!==undefined?{extensionCredentials:options.extensionCredentials,credentialIds:options.credentialIds,assetsReviewed:options.assetsReviewed}:{}),...(options?.allTemplates!==undefined?{allTemplates:options.allTemplates,allChannels:options.allChannels,allSkills:options.allSkills,allMcps:options.allMcps}:{})}}
export interface ExportedModels { path: string; digest: string; templates: number; channels: number }
export const modelTransferApi = {
  protect:(planId:string,passphrase?:string)=>invoke<FilePlan>('model_transfer_protect',{planId,passphrase:passphrase??null}),
  discardUnattempted:(planId:string)=>invoke<boolean>('model_transfer_discard_unattempted',{planId}),
  prepareDraft:(scope:TransferScope,draft:DraftCandidate,passphrase?:string,policy?:ModelPolicy)=>invoke<FilePlan>('model_transfer_prepare_draft',{scope,draft,passphrase:passphrase??null,policy:policy??null}),
  prepare: (scope: TransferScope,passphrase?:string,includeKeys=false,managerScope?:ManagerScope) => invoke<FilePlan>('model_transfer_prepare', { scope,passphrase:passphrase??null,includeKeys,managerScope:managerScope??null }),
  // targetDir 为空时写受管导出目录；用户另存时传绝对路径。
  export: (planId: string, targetDir?: string) => invoke<ExportedModels>('model_transfer_export', { planId, targetDir: targetDir ?? null }),
  receive: (path: string,passphrase?:string) => invoke<IncomingPlan>('model_transfer_receive', { path,passphrase:passphrase??null }),
  accept: (incomingId: string,options?:{adoptKeys:boolean;backupBeforeUpdate?:boolean;credentialIds?:string[];adoptWebdavPasswords?:boolean;adoptExtensionCredentials?:boolean;passphrase?:string}) => invoke<ModelSnapshot>('model_transfer_accept', { incomingId,backupBeforeUpdate:options?.backupBeforeUpdate??true,credentialIds:options?.credentialIds??null,adoptKeys:options?.adoptKeys??false,passphrase:options?.passphrase??null,adoptWebdavPasswords:options?.adoptWebdavPasswords??false,adoptExtensionCredentials:options?.adoptExtensionCredentials??false }),
  select:(incomingId:string,scope:ReceiveScope,passphrase?:string)=>invoke<IncomingPlan>('model_transfer_select',{incomingId,scope,passphrase:passphrase??null}),
  channelCatalog:()=>invoke<ChannelPolicy[]>('portable_channel_catalog'),
  accountCatalog:()=>invoke<AccountCatalogEntry[]>('portable_account_catalog'),
  extensionCatalog:()=>invoke<{skills:string[];mcps:string[];credentialSkills?:string[];credentialMcps?:string[]}>('portable_extension_catalog'),
  latestIncoming:()=>invoke<IncomingPlan|null>('portable_latest_incoming'),
  recompare:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_recompare',{incomingId,passphrase:passphrase??null}),
  openReceived:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_open_received',{incomingId,passphrase:passphrase??null}),
  loadRemoteScope:()=>invoke<RemoteScope>('config_sync_scope'),
  saveRemoteScope:(scope:RemoteScope)=>invoke<RemoteScope>('config_sync_save_scope',{scope}),
  prepareUpload: (scope: TransferScope,options?:RemoteOptions) => invoke<UploadPlan>('config_sync_prepare', { scope:remoteScope(scope,options),passphrase:options?.passphrase??null }),
  publish: (planId: string) => invoke<UploadPlan>('config_sync_publish', { planId }),
  download: (passphrase?:string) => invoke<IncomingPlan>('config_sync_download',{passphrase:passphrase??null}),
  latestUpload:()=>invoke<UploadPlan|null>('config_sync_latest_upload'),
  checkSync:()=>invoke<SyncCheck>('config_sync_check'),
}

export function scopeFor(snapshot: ModelSnapshot, templateIds: string[], channelIds: string[], global: boolean, versions: Record<string,string>,channelModels:Record<string,string[]>={},channelFamilies:Record<string,FamilyScope>={}): TransferScope {
  return {...(Object.keys(channelFamilies).length?{channelFamilies:Object.fromEntries(Object.entries(channelFamilies).filter(([id])=>channelIds.includes(id)))}:{}),...(Object.keys(channelModels).length?{channelModels:Object.fromEntries(Object.entries(channelModels).filter(([id])=>channelIds.includes(id)))}:{}), includePolicy: global || channelIds.length > 0, channelIds, includeGlobal: global,
    templateRefs: templateIds.flatMap(id => { const template = snapshot.library.templates.find(t => t.id === id); const chosen=versions[id];const revisions=chosen?template?.revisions.filter(r=>r.id===chosen)??[]:template?.revisions??[];return revisions.map(r=>({templateId:id,revisionId:r.id})) }) }
}

export const importCredentialChoiceApi={load:()=>invoke<{keys:boolean;extensions:boolean}>("import_credential_choices",{preferences:null}),save:(preferences:{keys:boolean;extensions:boolean})=>invoke("import_credential_choices",{preferences})}

export interface TransferRetry {adoptKeys:boolean;adoptWebdavPasswords:boolean;adoptExtensionCredentials:boolean;backupBeforeUpdate:boolean;credentialIds:string[]|null}
export interface TransferResult {id:string;kind:'export'|'import'|'upload'|'download';phase:'running'|'complete'|'partial'|'failed';startedAt:string;finishedAt:string|null;items:{id:string;label:string;kind:string;status:'pending'|'completed'|'unchanged'|'skipped'}[];errorCode:string|null;exportPath:string|null;sourceDigest:string;retry:TransferRetry|null}
export interface TransferBackupStatus {id:string;enabled:boolean;ready:boolean;groups:string[]}
export const transferResultApi={backups:()=>invoke<TransferBackupStatus[]>('list_transfer_backups'),previewRestore:(incomingId:string,groups:string[])=>invoke<string>('preview_transfer_restore',{incomingId,groups}),readMode:(mode:'file'|'webdav')=>invoke<TransferResult|null>('transfer_result',{operationId:null,mode}),read:(operationId?:string)=>invoke<TransferResult|null>('transfer_result',{operationId:operationId??null}),openDirectory:(planId:string)=>invoke<void>('open_transfer_export_directory',{planId}),backup:(incomingId:string)=>invoke<TransferBackupStatus|null>('transfer_backup_status',{incomingId}),restore:(incomingId:string,groups:string[],baseline:string)=>invoke<ModelSnapshot>('restore_transfer_backup',{incomingId,groups,baseline})}

export interface CloudSyncCatalog{binding:string;objects:{id:string;category:string;categoryLabel:string;label:string;detail:string;mark:string;files?:string[]}[];accounts?:PortableAccount[];credentialIds:string[];createdAt:string}
export const cloudCatalogApi={load:()=>invoke<CloudSyncCatalog|null>('config_sync_catalog')}

export const transferFileDialog={open:(options:OpenDialogOptions):Promise<string|string[]|null>=>open(options)}

export interface AssetDetail {source:string;fields:string[];files:string[];bytes:number;clients:string[];conflicts:string[];warnings:string[]}
export const assetDetailApi={load:(kind:string,name:string,source?:string)=>invoke<AssetDetail>('portable_asset_detail',{kind,name,source:source??null})}
