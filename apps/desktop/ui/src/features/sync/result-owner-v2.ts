import {invoke} from '@tauri-apps/api/core'

export interface ResultOwnerV2 {operationId:string;kind:'file'|'sync'|'recovery'|'scope_restore'}
export const resultOwnerV2Api={read:(operationId:string)=>invoke<ResultOwnerV2|null>('result_owner_v2',{operationId})}
