import { invoke } from '@tauri-apps/api/core'
import type { ChannelPolicy, ModelSnapshot, Template, TemplateRef } from '@/features/models/api'
export interface TransferScope { includePolicy: boolean; channelIds: string[]; templateRefs: TemplateRef[]; includeGlobal: boolean }
export interface ManagerScope {preferences:boolean;extensions:boolean;webdav:boolean;webdavPasswords:boolean}
export interface PortableEndpoint {id:string;url:string;remotePath:string;username:string;conflictPolicy:string}
export interface ManagerSettings {preferences:Record<string,unknown>|null;extensions:{skills:string[];servers:string[];sync_method:string}|null;webdav:PortableEndpoint[]}
export interface ModelPackage { format: string; formatVersion: number; schemaVersion: number; createdAt: string; digest: string; payload: { templates: Template[]; policy: { templateRefs: TemplateRef[] | null; channelOrder: string[] | null; channels: Pick<ChannelPolicy, 'id' | 'label' | 'selection'>[] } | null } }
export interface FilePlan { id: string; package: ModelPackage;protected?:boolean;fileDigest?:string;credentialCount?:number;manager?:ManagerSettings|null }
export interface UploadPlan extends FilePlan { phase: string }
export interface IncomingPlan extends FilePlan { expectedLibrary: string; expectedPolicy: string; expectedWorkspace: string; notices: string[];previewError?:string|null;availableKeys?:string[];availableWebdavPasswords?:string[];adoption?:{keys:boolean;webdav:boolean;completed:boolean}|null }
export interface ReceiveScope {templateRefs:TemplateRef[];channelIds:string[];includeGlobal:boolean;preferences?:boolean;extensions?:boolean;endpointIds?:string[]}
export interface ExportedModels { path: string; digest: string; templates: number; channels: number }
export const modelTransferApi = {
  prepare: (scope: TransferScope,passphrase?:string,includeKeys=false,managerScope?:ManagerScope) => invoke<FilePlan>('model_transfer_prepare', { scope,passphrase:passphrase??null,includeKeys,managerScope:managerScope??null }),
  export: (planId: string) => invoke<ExportedModels>('model_transfer_export', { planId }),
  receive: (path: string,passphrase?:string) => invoke<IncomingPlan>('model_transfer_receive', { path,passphrase:passphrase??null }),
  accept: (incomingId: string,options?:{adoptKeys:boolean;adoptWebdavPasswords?:boolean;passphrase?:string}) => invoke<ModelSnapshot>('model_transfer_accept', { incomingId,adoptKeys:options?.adoptKeys??false,passphrase:options?.passphrase??null,adoptWebdavPasswords:options?.adoptWebdavPasswords??false }),
  select:(incomingId:string,scope:ReceiveScope)=>invoke<IncomingPlan>('model_transfer_select',{incomingId,scope}),
  latestIncoming:()=>invoke<IncomingPlan|null>('portable_latest_incoming'),
  recompare:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_recompare',{incomingId,passphrase:passphrase??null}),
  openReceived:(incomingId:string,passphrase?:string)=>invoke<IncomingPlan>('portable_open_received',{incomingId,passphrase:passphrase??null}),
  prepareUpload: (scope: TransferScope) => invoke<UploadPlan>('model_remote_prepare', { scope }),
  publish: (planId: string) => invoke<UploadPlan>('model_remote_publish', { planId }),
  download: () => invoke<IncomingPlan>('model_remote_download'),
}

export function scopeFor(snapshot: ModelSnapshot, templateIds: string[], channelIds: string[], global: boolean, versions: Record<string,string>): TransferScope {
  return { includePolicy: global || channelIds.length > 0, channelIds, includeGlobal: global,
    templateRefs: templateIds.flatMap(id => { const template = snapshot.library.templates.find(t => t.id === id); const revision = template?.revisions.find(r => r.id === versions[id]) ?? template?.revisions.at(-1); return revision ? [{templateId: id, revisionId: revision.id}] : [] }) }
}
