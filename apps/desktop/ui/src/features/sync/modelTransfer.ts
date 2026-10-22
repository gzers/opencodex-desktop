import { invoke } from '@tauri-apps/api/core'
import type { ChannelPolicy, ModelSnapshot, Template, TemplateRef, DiscoveredModel } from '@/features/models/api'
export interface DraftCandidate {expectedLibrary:string;expectedPolicy:string;expectedWorkspace:string}
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
export interface ManagerScope {accountIds?:string[];preferences:boolean;extensions:boolean;webdav:boolean;webdavPasswords:boolean;skillNames?:string[];mcpNames?:string[];extensionCredentials?:boolean;assetsReviewed?:boolean}
export interface AssetNameMappings {skills:Record<string,string>;mcps:Record<string,string>}
export interface AssetSummary {skills:{name:string;digest:string;files:number;bytes:number;warnings:string[];protected:boolean}[];mcps:{name:string;digest:string;secretFields:string[];warnings:string[]}[]}
export interface PortableEndpoint {id:string;url:string;remotePath:string;username:string;conflictPolicy:string}
export interface ManagerSettings {preferences:Record<string,unknown>|null;extensions:{skills:string[];servers:string[];sync_method:string}|null;webdav:PortableEndpoint[];accounts?:PortableAccount[]}
export interface ModelPackage { format: string; formatVersion: number; schemaVersion: number; createdAt: string; digest: string; payload: { candidate?:DraftCandidate; templates: Template[]; policy: { templateRefs: TemplateRef[] | null; channelOrder: string[] | null;defaultChannelId?:string|null; channels: (Pick<ChannelPolicy, 'id' | 'label' | 'selection' | 'naming'> & {familyScope?:FamilyScope;modelSubset?:string[]|null;discovered?:DiscoveredModel[];modelOrder?:string[];connection?:{nodes:{id:string;name:string;url:string}[];provider?:Record<string,unknown>|null}|null})[] } | null } }
export interface FilePlan { id: string; package: ModelPackage;protected?:boolean;fileDigest?:string;credentialCount?:number;manager?:ManagerSettings|null;assets?:AssetSummary|null }
export interface UploadPlan extends FilePlan { phase: string }
export interface IncomingPlan extends FilePlan { expectedLibrary: string; expectedPolicy: string; expectedWorkspace: string; notices: string[];previewError?:string|null;availableKeys?:string[];availableWebdavPasswords?:string[];availableExtensionCredentials?:boolean;assetMappings?:AssetNameMappings;channelBindings?:Record<string,{targetId:string;keepLocalConnection:boolean;label:string|null}>;adoption?:{keys:boolean;webdav:boolean;completed:boolean;extensionCredentials?:boolean}|null }
export type ChannelImportMapping = {mode:'new';label:string}|{mode:'local';targetId:string}
export interface ReceiveScope {channelModels?:Record<string,string[]>;assetMappings?:AssetNameMappings;channelMappings?:Record<string,ChannelImportMapping>;accountIds?:string[];templateRefs:TemplateRef[];channelIds:string[];includeGlobal:boolean;preferences?:boolean;extensions?:boolean;endpointIds?:string[];skillNames?:string[];mcpNames?:string[]}
export interface RemoteScope {channelFamilies?:Record<string,FamilyScope>;channelModels?:Record<string,string[]>;accountIds?:string[];templateIds:string[];channelIds:string[];includeGlobal:boolean;preferences:boolean;extensions:boolean;includeKeys:boolean;skillNames?:string[];mcpNames?:string[];extensionCredentials?:boolean;assetsReviewed?:boolean;allTemplates?:boolean;allChannels?:boolean;allSkills?:boolean;allMcps?:boolean}
export interface RemoteOptions {accountIds?:string[];preferences:boolean;extensions:boolean;includeKeys:boolean;skillNames?:string[];mcpNames?:string[];extensionCredentials?:boolean;assetsReviewed?:boolean;allTemplates?:boolean;allChannels?:boolean;allSkills?:boolean;allMcps?:boolean;passphrase?:string}
export function remoteScope(scope:TransferScope,options?:RemoteOptions):RemoteScope{return {...(scope.channelFamilies?{channelFamilies:scope.channelFamilies}:{}),...(scope.channelModels?{channelModels:scope.channelModels}:{}),...(options?.accountIds?{accountIds:options.accountIds}:{}),templateIds:[...new Set(scope.templateRefs.map(reference=>reference.templateId))],channelIds:scope.channelIds,includeGlobal:scope.includeGlobal,preferences:options?.preferences??true,extensions:options?.extensions??true,includeKeys:options?.includeKeys??false,...(options?.skillNames?{skillNames:options.skillNames}:{}),...(options?.mcpNames?{mcpNames:options.mcpNames}:{}),...(options?.extensionCredentials!==undefined?{extensionCredentials:options.extensionCredentials,assetsReviewed:options.assetsReviewed}:{}),...(options?.allTemplates!==undefined?{allTemplates:options.allTemplates,allChannels:options.allChannels,allSkills:options.allSkills,allMcps:options.allMcps}:{})}}
export interface ExportedModels { path: string; digest: string; templates: number; channels: number }
export const modelTransferApi = {
  prepareDraft:(scope:TransferScope,draft:DraftCandidate,passphrase?:string)=>invoke<FilePlan>('model_transfer_prepare_draft',{scope,draft,passphrase:passphrase??null}),
  prepare: (scope: TransferScope,passphrase?:string,includeKeys=false,managerScope?:ManagerScope) => invoke<FilePlan>('model_transfer_prepare', { scope,passphrase:passphrase??null,includeKeys,managerScope:managerScope??null }),
  // targetDir 为空时写受管导出目录；用户另存时传绝对路径。
  export: (planId: string, targetDir?: string) => invoke<ExportedModels>('model_transfer_export', { planId, targetDir: targetDir ?? null }),
  receive: (path: string,passphrase?:string) => invoke<IncomingPlan>('model_transfer_receive', { path,passphrase:passphrase??null }),
  accept: (incomingId: string,options?:{adoptKeys:boolean;adoptWebdavPasswords?:boolean;adoptExtensionCredentials?:boolean;passphrase?:string}) => invoke<ModelSnapshot>('model_transfer_accept', { incomingId,adoptKeys:options?.adoptKeys??false,passphrase:options?.passphrase??null,adoptWebdavPasswords:options?.adoptWebdavPasswords??false,adoptExtensionCredentials:options?.adoptExtensionCredentials??false }),
  select:(incomingId:string,scope:ReceiveScope,passphrase?:string)=>invoke<IncomingPlan>('model_transfer_select',{incomingId,scope,passphrase:passphrase??null}),
  channelCatalog:()=>invoke<ChannelPolicy[]>('portable_channel_catalog'),
  accountCatalog:()=>invoke<AccountCatalogEntry[]>('portable_account_catalog'),
  extensionCatalog:()=>invoke<{skills:string[];mcps:string[]}>('portable_extension_catalog'),
  latestIncoming:()=>invoke<IncomingPlan|null>('portable_latest_incoming'),
  recompare:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_recompare',{incomingId,passphrase:passphrase??null}),
  openReceived:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_open_received',{incomingId,passphrase:passphrase??null}),
  loadRemoteScope:()=>invoke<RemoteScope>('config_sync_scope'),
  saveRemoteScope:(scope:RemoteScope)=>invoke<RemoteScope>('config_sync_save_scope',{scope}),
  prepareUpload: (scope: TransferScope,options?:RemoteOptions) => invoke<UploadPlan>('config_sync_prepare', { scope:remoteScope(scope,options),passphrase:options?.passphrase??null }),
  publish: (planId: string) => invoke<UploadPlan>('config_sync_publish', { planId }),
  download: (passphrase?:string) => invoke<IncomingPlan>('config_sync_download',{passphrase:passphrase??null}),
  latestUpload:()=>invoke<UploadPlan|null>('config_sync_latest_upload'),
}

export function scopeFor(snapshot: ModelSnapshot, templateIds: string[], channelIds: string[], global: boolean, versions: Record<string,string>,channelModels:Record<string,string[]>={},channelFamilies:Record<string,FamilyScope>={}): TransferScope {
  return {...(Object.keys(channelFamilies).length?{channelFamilies:Object.fromEntries(Object.entries(channelFamilies).filter(([id])=>channelIds.includes(id)))}:{}),...(Object.keys(channelModels).length?{channelModels:Object.fromEntries(Object.entries(channelModels).filter(([id])=>channelIds.includes(id)))}:{}), includePolicy: global || channelIds.length > 0, channelIds, includeGlobal: global,
    templateRefs: templateIds.flatMap(id => { const template = snapshot.library.templates.find(t => t.id === id); const revision = template?.revisions.find(r => r.id === versions[id]) ?? template?.revisions.at(-1); return revision ? [{templateId: id, revisionId: revision.id}] : [] }) }
}
