import {computed,ref,shallowRef} from 'vue'
import {defineStore} from 'pinia'
import {useModelsV2Store} from '@/features/models/store-v2'
import {useAppStore} from '@/stores/app'
import {transferBackupV2Api,type BackupStatusV2,type RestorePreviewV2,type RestoreReceiptV2} from './api-v2'

const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
const sameGroups=(a:readonly string[],b:readonly string[])=>a.length===b.length&&new Set(a).size===a.length&&new Set(b).size===b.length&&[...a].sort().every((v,i)=>v===[...b].sort()[i])
const message=(cause:unknown)=>cause&&typeof cause==='object'&&'code'in cause?String(cause.code):cause instanceof Error?cause.message:'原范围备份尚未核对完成，原记录和输入保持。'

export interface TransferRestoreSessionV2 {
  originalOperationId:string;availableGroups:string[];groups:string[]
  preview:RestorePreviewV2|null;receipt:RestoreReceiptV2|null
  attempted:boolean;reviewContext:string;error:string;refreshError:string
}
export const useTransferBackupsV2Store=defineStore('transfer-backups-v2',()=>{
  const models=useModelsV2Store()
  const rows=shallowRef<BackupStatusV2[]>([]),sessions=ref<Record<string,TransferRestoreSessionV2>>({}),activeId=ref(''),open=ref(false)
  const loading=ref(false),checking=ref(false),restoring=ref(false),refreshing=ref(false),listError=ref('')
  let listSequence=0,previewSequence=0
  const active=computed(()=>sessions.value[activeId.value]??null)
  const busy=computed(()=>checking.value||restoring.value||refreshing.value)
  function modelContext(){return JSON.stringify({file:models.fileContext(),rows:models.modelRows,template:models.draft,templateGeneration:models.editGeneration})}
  function assertReady(){if(models.fileBusy||models.saving||models.fileNeedsReconcile||models.fileDirty||models.pendingRows.length||models.dirty)throw new Error('请先保存或放弃当前渠道、型号、模版或 Key 编辑，再恢复所选备份；当前输入已保留。')}
  const confirmationStale=computed(()=>!!active.value?.preview&&!active.value.attempted&&active.value.reviewContext!==modelContext())
  const canRestore=computed(()=>!!active.value?.preview&&!!active.value.groups.length&&!busy.value&&active.value.receipt?.status!=='applied'&&(active.value.attempted||active.value.reviewContext===modelContext()))
  async function load(){const seq=++listSequence;loading.value=true;listError.value='';try{const value=await transferBackupV2Api.list();if(seq!==listSequence)return;if(!Array.isArray(value)||value.some(row=>!row.id||!Array.isArray(row.groups)||new Set(row.groups).size!==row.groups.length))throw new Error('范围备份目录返回不完整，原记录保持。');rows.value=value}catch(cause){if(seq===listSequence)listError.value=message(cause)}finally{if(seq===listSequence)loading.value=false}}
  async function choose(row:BackupStatusV2){if(busy.value||!row.enabled||!row.ready)return;const known=rows.value.find(r=>r.id===row.id);if(!known||!known.enabled||!known.ready)return;activeId.value=row.id;open.value=true;if(!sessions.value[row.id])sessions.value[row.id]={originalOperationId:row.id,availableGroups:[...known.groups],groups:[...known.groups],preview:null,receipt:null,attempted:false,reviewContext:'',error:'',refreshError:''};if(!active.value!.attempted)await preview()}
  function setGroups(groups:string[]){const session=active.value;if(!session||busy.value||session.attempted)return;if(!groups.every(group=>session.availableGroups.includes(group))||new Set(groups).size!==groups.length){session.error='只能选择原备份已提供的恢复范围。';return}session.groups=[...groups];session.preview=null;session.receipt=null;session.reviewContext='';session.error='';if(groups.length)void preview()}
  async function preview(){
    const session=active.value;if(!session||busy.value||session.attempted||!session.groups.length)return
    checking.value=true;session.error='';session.preview=null;const seq=++previewSequence,groups=[...session.groups],context=modelContext()
    try{assertReady();const value=await transferBackupV2Api.preview(session.originalOperationId,groups);if(seq!==previewSequence||active.value!==session)return;assertReady();if(context!==modelContext()||!sameGroups(groups,session.groups))throw new Error('核对期间当前编辑或范围已变化，输入保持，请重新核对。');if(!value.restoreId||!value.planDigest||!sameGroups(value.groups,groups))throw new Error('恢复预览不属于所选原备份范围，未允许写入。');session.preview=copy(value);session.reviewContext=context}
    catch(cause){if(seq===previewSequence)session.error=message(cause)}finally{if(seq===previewSequence)checking.value=false}
  }
  async function refreshSaved(session=active.value){
    if(!session?.receipt||refreshing.value)return
    refreshing.value=true;session.refreshError=''
    try{assertReady();if(session.receipt.refreshFileSnapshot)await models.reloadSavedFacts();if(session.receipt.completedGroups.includes('preferences')){await useAppStore().loadPreferences();if(useAppStore().preferencesError)throw new Error('已恢复的偏好暂未读回，原恢复回执保持。')}}
    catch(cause){session.refreshError=message(cause)}finally{refreshing.value=false}
  }
  async function restore(){
    const session=active.value;if(!session?.preview||!canRestore.value)return
    restoring.value=true;session.error='';const plan=copy(session.preview),groups=[...session.groups]
    try{
      assertReady();if(!session.attempted&&session.reviewContext!==modelContext())throw new Error('恢复确认已过期，当前输入保持，请重新核对。')
      await models.releaseCleanSessionForSavedMutation()
      assertReady();if(!sameGroups(groups,session.groups))throw new Error('确认期间恢复范围已变化，未执行本次恢复。')

      session.attempted=true
      const value=await transferBackupV2Api.restore(plan.restoreId,plan.planDigest)
      if(value.restoreId!==plan.restoreId||value.originalOperationId!==session.originalOperationId||!sameGroups(value.groups,plan.groups)||new Set(value.completedGroups).size!==value.completedGroups.length||!value.completedGroups.every(group=>plan.groups.includes(group))||!['applied','partial'].includes(value.status)||value.status==='applied'&&(!sameGroups(value.completedGroups,plan.groups)||value.failureCode))throw new Error('恢复回执身份或完成范围不完整，原计划保持，请核对原操作。')
      session.receipt=copy(value)
      if(value.status==='partial')session.error=value.failureCode??'原恢复仅部分完成；已完成范围保持，可继续同一恢复操作。'
      await refreshSaved(session)
    }catch(cause){session.error=message(cause)}finally{restoring.value=false}
  }
  function close(){if(busy.value)return;open.value=false;previewSequence++}
  return {confirmationStale,rows,sessions,activeId,active,open,loading,checking,restoring,refreshing,busy,listError,canRestore,load,choose,setGroups,preview,restore,refreshSaved,close}
})
