import {invoke} from '@tauri-apps/api/core'

// Public names frozen by L0; production registration is verified against its
// final immutable command commit separately from deterministic UI fixtures.
export interface BackupStatusV2 {id:string;enabled:boolean;ready:boolean;groups:string[]}
export interface RestorePreviewV2 {restoreId:string;planDigest:string;groups:string[]}
export interface RestoreReceiptV2 {
  restoreId:string;originalOperationId:string;groups:string[]
  status:'applied'|'partial';completedGroups:string[]
  refreshFileSnapshot:boolean;refreshSyncScope?:boolean;failureCode:string|null
}
export interface RestoreOperationV2 {restoreId:string;originalOperationId:string;planDigest:string;groups:string[];receipt:RestoreReceiptV2|null}
export const transferBackupV2Api={
  list:()=>invoke<BackupStatusV2[]>('transfer_v2_backup_list'),
  preview:(originalOperationId:string,groups:string[])=>invoke<RestorePreviewV2>('transfer_v2_backup_preview',{originalOperationId,groups}),
  restore:(restoreId:string,planDigest:string)=>invoke<RestoreReceiptV2>('transfer_v2_backup_restore',{restoreId,planDigest}),
  operations:(originalOperationId:string|null=null)=>invoke<RestoreOperationV2[]>('transfer_v2_backup_restore_operations',{originalOperationId}),
}
