import { invoke } from '@tauri-apps/api/core'
import type { ChannelPolicy, ModelSnapshot, Template, TemplateRef } from '@/features/models/api'
export interface TransferScope { includePolicy: boolean; channelIds: string[]; templateRefs: TemplateRef[]; includeGlobal: boolean }
export interface ModelPackage { format: string; formatVersion: number; schemaVersion: number; createdAt: string; digest: string; payload: { templates: Template[]; policy: { templateRefs: TemplateRef[] | null; channelOrder: string[] | null; channels: Pick<ChannelPolicy, 'id' | 'label' | 'selection'>[] } | null } }
export interface FilePlan { id: string; package: ModelPackage }
export interface UploadPlan extends FilePlan { phase: string }
export interface IncomingPlan extends FilePlan { expectedLibrary: string; expectedPolicy: string; expectedWorkspace: string; notices: string[] }
export interface ExportedModels { path: string; digest: string; templates: number; channels: number }
export const modelTransferApi = {
  prepare: (scope: TransferScope) => invoke<FilePlan>('model_transfer_prepare', { scope }),
  export: (planId: string) => invoke<ExportedModels>('model_transfer_export', { planId }),
  receive: (path: string) => invoke<IncomingPlan>('model_transfer_receive', { path }),
  accept: (incomingId: string) => invoke<ModelSnapshot>('model_transfer_accept', { incomingId }),
  prepareUpload: (scope: TransferScope) => invoke<UploadPlan>('model_remote_prepare', { scope }),
  publish: (planId: string) => invoke<UploadPlan>('model_remote_publish', { planId }),
  download: () => invoke<IncomingPlan>('model_remote_download'),
}

export function scopeFor(snapshot: ModelSnapshot, templateIds: string[], channelIds: string[], global: boolean, versions: Record<string,string>): TransferScope {
  return { includePolicy: global || channelIds.length > 0, channelIds, includeGlobal: global,
    templateRefs: templateIds.flatMap(id => { const template = snapshot.library.templates.find(t => t.id === id); const revision = template?.revisions.find(r => r.id === versions[id]) ?? template?.revisions.at(-1); return revision ? [{templateId: id, revisionId: revision.id}] : [] }) }
}
