import type {AgentViewContext} from './agentView'
import { computed, onScopeDispose, ref } from 'vue'
import { defineStore } from 'pinia'
import {pendingModelRows,writeRow,type ModelRowsState} from './modelRows'
import {canonicalPolicy} from './policyEquality'
import { emptyContent, emptyRules, errorText, modelApi, savePlanErrorText, type CredentialEdit, type ModelSnapshot, type ModelPolicy, type SavePlan, type Template, type TemplateDraft } from './api'

interface HistoricalRecovery {
  id:string; sourceId:string; policy:ModelPolicy
  libraryDigest:string; policyDigest:string; workspaceDigest:string
  plan:SavePlan|null
}

type MergeValue = undefined|null|boolean|number|string|MergeValue[]|{[key:string]:MergeValue}
const copyTransfer = <T>(value:T):T => value === undefined ? value : JSON.parse(JSON.stringify(value))
const mergeObject = (value:MergeValue):value is {[key:string]:MergeValue} => !!value && typeof value === 'object' && !Array.isArray(value)
const transferField = (value:{[key:string]:MergeValue},key:string):MergeValue => Object.hasOwn(value,key)?value[key]:undefined
function sameTransferValue(left:MergeValue,right:MergeValue):boolean {
  if (left === right) return true
  if (Array.isArray(left)&&Array.isArray(right)) return left.length===right.length&&left.every((value,index)=>sameTransferValue(value,right[index]))
  if (!mergeObject(left)||!mergeObject(right)) return false
  const keys=[...new Set([...Object.keys(left),...Object.keys(right)])]
  return keys.every(key=>sameTransferValue(transferField(left,key),transferField(right,key)))
}
function transferConflict():never {
  throw {code:'transfer_draft_conflict',message:'导入内容与当前渠道、型号或 Key 修改冲突，请先保存或还原冲突修改后重新比较；当前输入已保留。'}
}
function mergeTransferValue(base:MergeValue,local:MergeValue,incoming:MergeValue,key='',path:string[]=[]):MergeValue {
  // Reads of the same file digest can mint different local receipts. They do
  // not constitute competing edits; raw importedConfig receipts remain atomic.
  if (path.length===4&&path[0]==='channels'&&path[2]==='connection'&&key==='sourceReadId') return copyTransfer(incoming??local)
  if (sameTransferValue(local,base)) return copyTransfer(incoming)
  if (sameTransferValue(incoming,base)||sameTransferValue(local,incoming)) return copyTransfer(local)
  if (Array.isArray(base)&&Array.isArray(local)&&Array.isArray(incoming)) {
    const keyed=(values:MergeValue[])=>values.every(value=>mergeObject(value)&&typeof value.id==='string')&&new Set(values.map(value=>(value as {id:string}).id)).size===values.length
    if (keyed(base)&&keyed(local)&&keyed(incoming)) {
      const byId=(values:MergeValue[])=>new Map(values.map(value=>[(value as {id:string}).id,value]))
      const before=byId(base),ours=byId(local),theirs=byId(incoming),merged=new Map<string,MergeValue>()
      for (const id of new Set([...before.keys(),...ours.keys(),...theirs.keys()])) {
        const value=mergeTransferValue(before.get(id),ours.get(id),theirs.get(id),'',[...path,id])
        if (value!==undefined) merged.set(id,value)
      }
      // Existing order is a separate field. New objects from either side can
      // share a gap, but incompatible ordering must never silently move input.
      const retained=(values:Map<string,MergeValue>)=>[...values.keys()].filter(id=>before.has(id)&&merged.has(id))
      const order=mergeTransferValue(retained(before),retained(ours),retained(theirs)) as string[]
      const edges=new Map([...merged.keys()].map(id=>[id,new Set<string>()]))
      for (let i=1;i<order.length;i++) edges.get(order[i-1]!)!.add(order[i]!)
      for (const values of [ours,theirs]) {
        const ids=[...values.keys()].filter(id=>merged.has(id))
        for (let i=1;i<ids.length;i++) if (!before.has(ids[i-1]!)||!before.has(ids[i]!)) edges.get(ids[i-1]!)!.add(ids[i]!)
      }
      const degree=new Map([...merged.keys()].map(id=>[id,0]))
      for (const targets of edges.values()) for (const id of targets) degree.set(id,degree.get(id)!+1)
      const ready=[...merged.keys()].filter(id=>degree.get(id)===0),result:MergeValue[]=[]
      for (let i=0;i<ready.length;i++) {
        const id=ready[i]!;result.push(merged.get(id)!)
        for (const target of edges.get(id)!) {
          degree.set(target,degree.get(target)!-1)
          if (degree.get(target)===0) ready.push(target)
        }
      }
      if (result.length!==merged.size) transferConflict()
      return result
    }
  }
  // Exact template revisions and raw import receipts are indivisible; merging
  // their individual identifiers could authorize unrelated material at Save.
  const receipt=path.length===4&&path[0]==='channels'&&path[2]==='connection'&&key==='importedConfig'
  const revision=(value:{[key:string]:MergeValue})=>typeof value.templateId==='string'&&typeof value.revisionId==='string'
  if ((base===undefined||mergeObject(base))&&mergeObject(local)&&mergeObject(incoming)&&!receipt&&!revision(local)&&!revision(incoming)&&!(base===undefined&&(typeof local.id==='string'||typeof incoming.id==='string'||typeof local.mode==='string'||typeof incoming.mode==='string'))) {
    // An absent per-model override inherits every rule. Independent fields can
    // both be the first edit to that model (or to its optional naming map).
    const rules=(value:{[key:string]:MergeValue})=>['protocol','contextWindow','fast'].every(name=>mergeObject(transferField(value,name))&&typeof (value[name] as {mode:MergeValue}).mode==='string')
    const previous=base??(rules(local)&&rules(incoming)?emptyRules() as unknown as {[key:string]:MergeValue}:{})
    const result:{[key:string]:MergeValue}={}
    for (const name of new Set([...Object.keys(previous),...Object.keys(local),...Object.keys(incoming)])) {
      const value=mergeTransferValue(transferField(previous,name),transferField(local,name),transferField(incoming,name),name,[...path,name])
      if (value!==undefined) Object.defineProperty(result,name,{value,enumerable:true,configurable:true,writable:true})
    }
    return result
  }
  return transferConflict()
}
function mergeTransferPolicy(base:ModelPolicy,local:ModelPolicy,incoming:ModelPolicy):ModelPolicy {
  return mergeTransferValue(base as unknown as MergeValue,local as unknown as MergeValue,incoming as unknown as MergeValue) as unknown as ModelPolicy
}

// Route changes retain drafts; a refresh never discards an editor buffer.
export const useModelsStore = defineStore('model-config', () => {
  const modelRows=ref<Record<string,ModelRowsState>>({})
  const pendingRows=computed(()=>Object.entries(modelRows.value).flatMap(([channel,state])=>pendingModelRows(state).map(model=>({channel,model}))))
  const snapshot = ref<ModelSnapshot | null>(null)
  const draft = ref<TemplateDraft | null>(null)
  const draftSession = ref('')
  const clean = ref('')
  const editBaseline = ref('')
  const busy = ref(false)
  const error = ref('')
  const status = ref('')
  const selected = ref<Record<string, string>>({})
  const view = ref({tab:'channels' as 'channels'|'templates',templateSearch:'',selectedTemplates:[] as string[],templateEditor:false,templatePosition:'',channelSearch:'',channelEditor:'',selectedChannels:[] as string[],channelFilters:{} as Record<string,{search:string;selectedOnly:boolean;family:string}>,agentViews:{} as Record<string,AgentViewContext>,navigation:{} as Record<string,{collapsed:boolean;activeId:string}>,positions:{} as Record<string,{top:number;focus:string|null}>})
  let bufferKey = ''
  const buffers = new Map<string,{draft:TemplateDraft;clean:string;baseline:string;session:string}>()
  function retainTemplate() {
    if (draft.value && bufferKey) buffers.set(bufferKey,{draft:JSON.parse(JSON.stringify(draft.value)),clean:clean.value,baseline:editBaseline.value,session:draftSession.value})
  }
  function protectsRevision(templateId:string,revisionId:string){
    if(dirty.value&&draft.value?.templateId===templateId&&draft.value.baseRevision===revisionId)return true
    return [...buffers.values()].some(buffer=>buffer.draft.templateId===templateId&&buffer.draft.baseRevision===revisionId&&JSON.stringify(buffer.draft)!==buffer.clean)
  }
  const notificationTarget = ref<{taskId:string;submissionId:string} | null>(null)
  const policyNotificationTarget=ref<{planId:string}|null>(null)
  const channelDraft = ref<ModelPolicy | null>(null)
  const channelClean = ref('')
  const channelBaseline = ref('')
  const workspaceBaseline = ref('')
  const transferAdopted = ref(false)
  const channelError = ref('')
  const staging = ref(false)
  // Explicit historical restoration has its own legacy baselines and runtime
  // plan. None of these may become the ordinary config.json save baseline.
  const historicalRecovery = ref<HistoricalRecovery|null>(null)
  // Channel edits live in this process until explicit Save. In particular, Key
  // input must never enter the retired disk draft or an audit/event payload.
  const credentialEdits = ref<Record<string, CredentialEdit>>({})
  function editCredential(channelId:string,value:string,original:string) {
    // An emptied text field keeps the previous saved credential. Deletion is
    // an explicit action, matching the prototype's “清空 Key” control.
    if(value===original||!value)delete credentialEdits.value[channelId]
    else credentialEdits.value[channelId]={channelId,action:'replace',value}
  }
  function clearCredential(channelId:string) {
    credentialEdits.value[channelId]={channelId,action:'clear',value:null}
  }
  onScopeDispose(() => { credentialEdits.value={} })
  // 读取现有渠道会带上本机读取回执等记账字段；按等价草稿比较，避免“只读取未编辑”误报。
  // An explicitly adopted raw provider can contain changes beyond the modeled
  // fields. Its receipt must reach Save even when canonicalPolicy is unchanged.
  const transferMaterialDirty=computed(()=>transferAdopted.value&&channelDraft.value?.channels.some(channel=>{
    const receipt=channel.connection.importedConfig
    const saved=snapshot.value?.policy.channels.find(saved=>saved.id===channel.id)?.connection.importedConfig
    return !!receipt&&!sameTransferValue(receipt,saved)
  }))
  const channelDirty = computed(() => historicalRecovery.value !== null || transferMaterialDirty.value || Object.keys(credentialEdits.value).length>0 || channelDraft.value !== null && canonicalPolicy(channelDraft.value) !== channelClean.value)
  const dirty = computed(() => draft.value !== null && JSON.stringify(draft.value) !== clean.value)
  async function load() {
    if (busy.value) return
    busy.value = true; error.value = ''
    try {
      snapshot.value = await modelApi.load()
      if (!historicalRecovery.value&&(!channelDraft.value||!channelDirty.value&&!pendingRows.value.length)) {
        channelDraft.value = JSON.parse(JSON.stringify(snapshot.value.policy))
        channelClean.value = canonicalPolicy(snapshot.value.policy)
        channelBaseline.value = snapshot.value.policyDigest
        workspaceBaseline.value = snapshot.value.workspaceDigest
      }
    } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
  }
  function edit(template?: Template, revisionId?: string, copy = false) {
    const revision = template?.revisions.find(item => item.id === revisionId) ?? template?.revisions.at(-1)
    const nextKey=JSON.stringify([template?.id??'new',revision?.id??'',copy])
    if((copy||!template)&&hasNewTemplate.value&&nextKey!==bufferKey&&!buffers.has(nextKey)){error.value='请先保存或放弃已有的新建模版，再创建';return}
    retainTemplate()
    bufferKey=nextKey
    const retained=buffers.get(bufferKey)
    if(retained){draft.value=JSON.parse(JSON.stringify(retained.draft));clean.value=retained.clean;editBaseline.value=retained.baseline;draftSession.value=retained.session;error.value='';status.value='';return}
    draftSession.value = crypto.randomUUID()
    draft.value = { templateId: copy ? null : template?.id ?? null, baseRevision: copy ? null : revision?.id ?? null, label: template ? `${template.label}${copy ? ' 副本' : ''}` : '', content: revision ? JSON.parse(JSON.stringify(revision.content)) : emptyContent() }
    clean.value = JSON.stringify(draft.value); error.value = ''; status.value = ''
    editBaseline.value = snapshot.value?.libraryDigest ?? ''
  }
  const hasNewTemplate=computed(()=>draft.value?.templateId===null||[...buffers.values()].some(buffer=>buffer.draft.templateId===null))
  function copyCurrentTemplate(){
    if(busy.value||!draft.value?.templateId)return
    if(hasNewTemplate.value){error.value='请先保存或放弃已有的新建模版，再复制';return}
    const current=JSON.parse(JSON.stringify(draft.value)) as TemplateDraft
    retainTemplate();bufferKey=JSON.stringify([current.templateId,current.baseRevision??'',true]);draftSession.value=crypto.randomUUID()
    draft.value={templateId:null,baseRevision:null,label:current.label+' 副本',content:current.content}
    clean.value=JSON.stringify(draft.value);editBaseline.value=snapshot.value?.libraryDigest??'';error.value='';status.value=''
  }
  function leave() { buffers.delete(bufferKey);bufferKey='';draft.value = null; clean.value = ''; status.value = ''; error.value = '';view.value.templateEditor=false }
  function restoreTemplate(){
    if(!draft.value||busy.value||!clean.value)return
    draft.value=JSON.parse(clean.value);buffers.delete(bufferKey);draftSession.value=crypto.randomUUID();error.value='';status.value=''
  }
  async function save() {
    if (!snapshot.value || !draft.value || busy.value) return
    const request = JSON.parse(JSON.stringify(draft.value)) as TemplateDraft
    const expected = editBaseline.value
    const agentContext=view.value.agentViews[draftSession.value]
    busy.value = true; error.value = ''
    try {
      const result = await modelApi.save(request, expected)
      // Template commits do not replace the authoritative file-backed channel
      // snapshot with the old manager-policy copy returned by legacy storage.
      snapshot.value = {...snapshot.value,library:result.snapshot.library,libraryDigest:result.snapshot.libraryDigest}
      const template = result.snapshot.library.templates.find(item => item.id === result.templateId)
      buffers.delete(bufferKey);draft.value=null
      edit(template, result.revisionId)
      if(agentContext?.initialized)view.value.agentViews[draftSession.value]=JSON.parse(JSON.stringify(agentContext))
      selected.value[result.templateId] = result.revisionId
      status.value = result.changed ? '已保存' : '无内容变化'
    } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
  }
  function restoreChannels() {
    historicalRecovery.value=null
    transferAdopted.value=false
    modelRows.value={}
    credentialEdits.value={}
    if (!snapshot.value) return
    channelDraft.value = JSON.parse(JSON.stringify(snapshot.value.policy))
    channelClean.value = canonicalPolicy(channelDraft.value); channelBaseline.value = snapshot.value.policyDigest
    workspaceBaseline.value = snapshot.value.workspaceDigest
  }
  function acceptTransfer(saved: ModelSnapshot) {
    historicalRecovery.value=null
    transferAdopted.value=false
    credentialEdits.value={}
    snapshot.value = saved
    channelDraft.value = JSON.parse(JSON.stringify(saved.workspace?.policy ?? saved.policy))
    channelClean.value = canonicalPolicy(saved.policy)
    channelBaseline.value = saved.workspace?.basePolicy ?? saved.policyDigest
    workspaceBaseline.value = saved.workspaceDigest
    // The template editor keeps its original buffer and optimistic baseline.
  }
  async function adoptTransfer(plan:{expectedPolicy:string;mergedPolicy?:ModelPolicy|null},accept:()=>Promise<ModelSnapshot>) {
    if (busy.value||staging.value) throw {message:'模型配置正在处理，请稍后重试；当前输入已保留。'}
    if (!snapshot.value) throw {message:'无法读取当前模型配置'}
    const before=copyTransfer(snapshot.value)
    function mergeIncoming(policy:ModelPolicy,expectedFile:string) {
      if (historicalRecovery.value||channelDraft.value?.recoveryId||policy.recoveryId) throw {message:'请先保存或还原已有的历史恢复草稿；当前草稿已保留。'}
      if (expectedFile!==before.policyDigest||snapshot.value?.policyDigest!==before.policyDigest||channelDraft.value&&channelBaseline.value!==before.policyDigest) throw {message:'配置文件基准已变化，请重新比较；当前输入已保留。'}
      const local=channelDraft.value??before.policy
      const merged=mergeTransferPolicy(before.policy,local,policy)
      if (Object.keys(credentialEdits.value).some(id=>!merged.channels.some(channel=>channel.id===id))) transferConflict()
      if (pendingRows.value.length) {
        const pending=copyTransfer(local)
        for (const row of pendingRows.value) {
          const channel=pending.channels.find(channel=>channel.id===row.channel)
          if (!channel||!merged.channels.some(channel=>channel.id===row.channel)) transferConflict()
          writeRow(channel,row.model,modelRows.value[row.channel]!.buffers[row.model]!)
        }
        mergeTransferPolicy(before.policy,pending,policy)
      }
      return merged
    }
    // Fail before credentials, settings or templates can be adopted by IPC.
    if (plan.mergedPolicy) mergeIncoming(plan.mergedPolicy,plan.expectedPolicy)
    else if (plan.mergedPolicy===undefined&&historicalRecovery.value) throw {message:'请先保存或还原已有的历史恢复草稿；当前草稿已保留。'}
    busy.value=true
    try {
      const saved=await accept()
      // Template-only adoption never promotes an unrelated disk workspace or
      // replaces the actual-file snapshot/baseline, including during recovery.
      snapshot.value={...snapshot.value!,library:saved.library,libraryDigest:saved.libraryDigest}
      if (plan.mergedPolicy===null||!saved.workspace) return
      if (saved.workspace.basePolicy!==saved.policyDigest) throw {message:'导入草稿与当前配置文件不符，请重新比较；当前输入已保留。'}
      // Re-read live input after IPC: even a field typed while awaiting the
      // backend participates in the merge and can block this adoption.
      const merged=mergeIncoming(saved.workspace.policy,saved.policyDigest)
      snapshot.value=saved
      channelDraft.value=merged
      transferAdopted.value=true
      channelClean.value=canonicalPolicy(saved.policy)
      channelBaseline.value=saved.policyDigest
      workspaceBaseline.value=saved.workspaceDigest
    } finally { busy.value=false }
  }
  function assertRecoveryInputs() {
    if (historicalRecovery.value) throw {message:'请先保存或还原已有的历史恢复草稿；当前草稿已保留。'}
    if (channelDirty.value||pendingRows.value.length) throw {message:'请先保存或还原当前渠道、型号或 Key 修改，再生成历史恢复草稿；当前输入已保留。'}
  }
  function assertRecoveryReady() {
    if (busy.value||staging.value) throw {message:'模型配置正在处理，请稍后重试；当前输入已保留。'}
    assertRecoveryInputs()
  }
  async function acceptHistoricalRecovery(id:string,sourceId:string) {
    assertRecoveryReady()
    if (!snapshot.value) throw {message:'无法读取当前模型配置'}
    busy.value=true
    try {
      const saved=await modelApi.acceptRecovery(id)
      // Check again after IPC: another mounted editor must never lose input.
      assertRecoveryInputs()
      const workspace=saved.workspace
      if (!workspace||workspace.policy.recoveryId!==id||workspace.basePolicy!==saved.policyDigest) throw {message:'恢复草稿与所选记录不符，请重新比较；当前输入已保留。'}
      historicalRecovery.value={id,sourceId,policy:JSON.parse(JSON.stringify(workspace.policy)),libraryDigest:saved.libraryDigest,policyDigest:saved.policyDigest,workspaceDigest:saved.workspaceDigest,plan:null}
      channelDraft.value=JSON.parse(JSON.stringify(workspace.policy))
      // Acceptance can restore template dependencies/defaults, but its legacy
      // channel policy is never a view of the actual config file.
      snapshot.value={...snapshot.value,library:saved.library,libraryDigest:saved.libraryDigest}
    } finally { busy.value=false }
  }
  async function saveHistoricalRecovery(policy:ModelPolicy) {
    const recovery=historicalRecovery.value
    if (!recovery) throw {message:'历史恢复草稿已改变，请重新比较。'}
    const checkDraft=()=>{
      if (historicalRecovery.value!==recovery||pendingRows.value.length||Object.keys(credentialEdits.value).length||canonicalPolicy(policy)!==canonicalPolicy(recovery.policy)||canonicalPolicy(channelDraft.value)!==canonicalPolicy(recovery.policy)) throw {message:'历史恢复草稿中有新的渠道、型号或 Key 修改，未提交；当前输入已保留。请还原修改后继续普通编辑，或重新生成历史恢复草稿。'}
    }
    checkDraft()
    // A retry examines only this explicit recovery's original plan; ordinary
    // edits never query latestPolicy, reconcile old plans or enter this path.
    if (recovery.plan&&recovery.plan.phase!=='committed') recovery.plan=await modelApi.policyResult(recovery.plan.id)
    if (!recovery.plan) recovery.plan=await modelApi.preparePolicy(JSON.parse(JSON.stringify(recovery.policy)),recovery.libraryDigest,recovery.policyDigest,recovery.workspaceDigest)
    checkDraft()
    if (recovery.plan.historySource!==recovery.sourceId) throw {message:'保存计划不属于所选历史恢复，未继续写入；恢复草稿已保留。'}
    if (recovery.plan.phase==='prepared') recovery.plan=await modelApi.applyPolicy(recovery.plan.id)
    if (recovery.plan.phase!=='committed') throw {message:savePlanErrorText(recovery.plan.error)||'历史恢复尚未提交；草稿与原计划已保留，请重新核对。'}
    let current:ModelSnapshot
    try { current=await modelApi.load() }
    catch { throw {message:'历史恢复已提交，但读取当前配置失败；恢复草稿与原计划已保留，请再次保存以重新读取。'} }
    checkDraft()
    // Clear provenance only after a successful actual-file read. A failed or
    // uncertain runtime operation retains both draft and original plan id.
    acceptTransfer(current)
  }

  // 模型选择器的启用渠道口径与执行目录一致：渠道未停用即可见（含官方/原生）。
  // 这里只做同步读取，真正的“可用型号”判定仍在选择器里按每一行解释。
  const enabledChannels = computed(() => snapshot.value?.policy.channels.filter(c => c.connection.enabled) ?? [])

  return { historicalRecovery,assertRecoveryReady,acceptHistoricalRecovery,saveHistoricalRecovery,credentialEdits,editCredential,clearCredential,modelRows, pendingRows, view, hasNewTemplate, copyCurrentTemplate, protectsRevision, restoreTemplate, snapshot, draft, draftSession, notificationTarget, policyNotificationTarget, dirty, busy, error, status, selected, channelDraft, channelDirty, channelBaseline, channelError, staging, adoptTransfer, acceptTransfer, load, edit, leave, save, restoreChannels, enabledChannels }
})

