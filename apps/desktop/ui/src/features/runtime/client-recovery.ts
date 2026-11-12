import { invoke } from '@tauri-apps/api/core'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

export type ClientRecoveryStatus = 'not_required' | 'active' | 'pending' | 'restored' | 'unknown' | 'unsupported'
export type ClientRecoveryEvidence = 'not_recorded' | 'verified' | 'unregistered' | 'unknown'
export type ClientRecoveryPhase = 'preparing' | 'captured' | 'ready' | 'stop_prepared' | 'stopped' | 'restore_prepared' | 'restored' | 'failed'
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
  confirmationDigest: string | null
}
export interface ClientRecoveryFinalizeRequest { attemptId: string; revision: number; confirmationDigest: string }
export interface ClientRecoveryFinalizeOutcome { attemptId: string; revision: number; status: 'restored' }

// Inspection takes no file paths and never starts recovery implicitly.
export const clientRecoveryApi = {
  preview: () => invoke<ClientRecoveryPreview>('client_recovery_preview'),
  finalize: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryFinalizeOutcome>('client_recovery_finalize', { request }),
  continue: (request: ClientRecoveryFinalizeRequest) => invoke<ClientRecoveryFinalizeOutcome>('client_recovery_continue', { request }),
  status: () => invoke<StatusSnapshot>('get_status_snapshot'),
}
