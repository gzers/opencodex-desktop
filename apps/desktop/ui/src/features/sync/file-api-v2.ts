import {invoke} from '@tauri-apps/api/core'
import type {ReadBaseV2,RefV2,ModelEditV2,ChannelIntentV2} from '@/features/models/api-v2'
import type {SelectionV2,ObjectEntryV2,McpSourceEntryV2,PublicSummaryV2,PayloadSchemaRefV2,ScopeRequestV2} from './api-v2'
export interface AccountInfoV2 {id:string;provider:string;subject:string|null;label:string;email:string|null;origin?:{version:number;accountPlatform:string;owner:string;authKind:string;sourceNamespace:string;sourceAccountId:string;tenant:string|null}}
export interface AccountCatalogEntryV2 {account:AccountInfoV2;loginTransfer:'manualImportDisabled'|'crossDeviceEvidenceMissing'|'dedicatedAdapterRequired'|'unsupportedProvider';localSource:boolean;informationSupported:boolean}
export interface FileConnectionV2 {id:string;label:string;url:string;remotePath:string;username:string;passwordAvailable:boolean}
export interface LocalFileCatalogV2 {accounts:AccountCatalogEntryV2[];entries:ObjectEntryV2[];mcpSources:McpSourceEntryV2[];webdavConnections:FileConnectionV2[];source:'saved'}
export interface FileExportPlanV2 {id:string;scope:SelectionV2;summary:PublicSummaryV2;encrypted:boolean;filename:string;schema:PayloadSchemaRefV2;digest:string;length:number}
export interface FileReceiptV2 {operationId:string;planDigest:string;fileDigest:string;length:number}
export type FileChannelBindingV2={kind:'existing';portableChannelId:string;localChannelId:string}|{kind:'new';portableChannelId:string}
export interface FileImportBindingsV2 {channels:FileChannelBindingV2[];mcpClients:Record<string,string>;assetsReviewed:boolean;adoptNamespaceChannelIds:string[]}
export const emptyFileBindingsV2=():FileImportBindingsV2=>({channels:[],mcpClients:{},assetsReviewed:false,adoptNamespaceChannelIds:[]})
export interface FileImportObjectV2 {kind:string;id:string;label:string;templateReferences:RefV2[];savedDefaultRevisionId:string|null;connectionAvailable:boolean;namespaceRequiresAdoption:boolean;exportedModelIds:string[]|null;exportedFamilyIds:string[]|null;modelIds:string[];familyIds:string[];revisionIds:string[];existingChannelIds:string[];sameIdExists:boolean;newIdentityAvailable:boolean;hasKey:boolean;hasAuthorization:boolean;hasCredentials:boolean;definitionDigest:string|null;accountInfo?:AccountInfoV2|null;informationSupported?:boolean}
export interface FileImportPreviewV2 {operationId:string;planDigest:string;summary:PublicSummaryV2;objects:FileImportObjectV2[];selectedScope:SelectionV2;differences:{changedChannelIds:string[];changedTemplateRefs:RefV2[];selectedSummary:PublicSummaryV2|null};requiresBackup:boolean;canConfirm:boolean;reasonCode:string|null}
export interface RecoverableFileOperationV2 {operationId:string;kind:string;attempted:boolean;completed:boolean}
export const configFileV2Api={
 catalog:()=>invoke<LocalFileCatalogV2>('catalog_config_file_v2'),
 readScope:()=>invoke<ScopeRequestV2|null>('file_v2_read_scope'),
 saveScope:(request:ScopeRequestV2)=>invoke<void>('file_v2_save_scope',{selection:request.selection,mcpClients:request.mcpClients,assetsReviewed:request.assetsReviewed}),
 prepareExport:(request:ScopeRequestV2,passphrase:string|null)=>invoke<FileExportPlanV2>('prepare_config_file_export_v2',{request,passphrase}),
 prepareDraftExport:(expected:ReadBaseV2,request:ScopeRequestV2,passphrase:string|null,edits:ModelEditV2[]=[],channelIntents:ChannelIntentV2[]=[])=>invoke<FileExportPlanV2>('prepare_draft_config_file_export_v2',{expected,request,passphrase,edits,channelIntents}),
 saveExport:(operationId:string)=>invoke<FileReceiptV2|null>('save_config_file_export_v2',{operationId}),
 cancelExport:(operationId:string)=>invoke<boolean>('cancel_config_file_export_v2',{operationId}),
 openImport:(passphrase:string|null,scope:SelectionV2|null,bindings:FileImportBindingsV2)=>invoke<FileImportPreviewV2|null>('open_config_file_import_v2',{passphrase,scope,bindings}),
 reselectImport:(operationId:string,expectedPlanDigest:string,scope:SelectionV2,bindings:FileImportBindingsV2)=>invoke<FileImportPreviewV2>('reselect_config_file_import_v2',{operationId,expectedPlanDigest,scope,bindings}),
 confirmImport:(operationId:string,expectedPlanDigest:string,backupBeforeUpdate:boolean)=>invoke<FileReceiptV2>('confirm_config_file_import_v2',{operationId,expectedPlanDigest,backupBeforeUpdate}),
 cancelImport:(operationId:string)=>invoke<boolean>('cancel_config_file_import_v2',{operationId}),
 reopenExport:(operationId:string)=>invoke<FileExportPlanV2>('reopen_config_file_export_v2',{operationId}),
 reopenImport:(operationId:string)=>invoke<FileImportPreviewV2>('reopen_config_file_import_v2',{operationId}),
 recoverable:()=>invoke<RecoverableFileOperationV2[]>('recoverable_config_file_operations_v2'),
}
