import { invoke } from '@tauri-apps/api/core'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

export type ClientRecoveryStatus = 'not_required' | 'active' | 'pending' | 'restored' | 'unknown' | 'unsupported'
export type ClientRecoveryEvidence = 'not_recorded' | 'verified' | 'unregistered' | 'unknown'
export type ClientRecoveryPhase = 'preparing' | 'captured' | 'ready' | 'stop_prepared' | 'stopped' | 'restore_prepared' | 'restore_rolled_back' | 'restored' | 'unchanged' | 'cancelled_before_capture' | 'failed'
export interface ClientRecoveryPreview {
  status: ClientRecoveryStatus
  phase: ClientRecoveryPhase | null
  revision: number | null
  attemptId: string | null
  originalEvidence: ClientRecoveryEvidence
  planEvidence: ClientRecoveryEvidence
  completionEvidence: ClientRecoveryEvidence
  phaseCount: number
  canContinue: boolean
  canRollback: boolean
  canFinalize: boolean
  canRearm: boolean
  canCancelPreparation: boolean
  preparationDigest: string | null
  canFinishUnchanged: boolean
  unchangedDigest: string | null
  rollbackDigest: string | null
  rearmDigest: string | null
  confirmationDigest: string | null
}
export interface ClientRecoveryFinalizeRequest { attemptId: string; revision: number; confirmationDigest: string }
export interface ClientRecoveryFinalizeOutcome { attemptId: string; revision: number; status: 'restored' }
export type ClientRecoveryAction = 'finalize' | 'continue' | 'rollback' | 'rearm' | 'finishUnchanged' | 'cancelPreparation'
export interface ClientRecoveryOutcome { attemptId: string; revision: number; status: 'restored' | 'rolled_back' | 'restore_prepared' | 'unchanged' | 'cancelled_before_capture' }

// Inspection takes no file paths and never starts recovery implicitly.
export const clientRecoveryApi = {
  preview: () => invoke<ClientRecoveryPreview>('client_recovery_preview'),
  finalize: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryFinalizeOutcome>('client_recovery_finalize', { request }),
  continue: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryFinalizeOutcome>('client_recovery_continue', { request }),
  rollback: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryOutcome>('client_recovery_rollback', { request }),
  rearm: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryOutcome>('client_recovery_rearm', { request }),
  finishUnchanged: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryOutcome>('client_recovery_finish_unchanged', { request }),
  cancelPreparation: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryOutcome>('client_recovery_cancel_preparation', { request }),
  status: () => invoke<StatusSnapshot>('get_status_snapshot'),
}
