import type {AgentViewContext} from './agentView'
import { computed, onScopeDispose, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import {pendingModelRows,type ModelRowsState} from './modelRows'
import { emptyContent, errorText, modelApi, type ModelSnapshot, type ModelPolicy, type Template, type TemplateDraft } from './api'

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
  const channelError = ref('')
  const staging = ref(false)
  let stageWork: Promise<void> | undefined
  let stageTimer: ReturnType<typeof setTimeout> | undefined
  function persistDraft(): Promise<void> {
    if (stageTimer) { clearTimeout(stageTimer); stageTimer = undefined }
    if (stageWork) return stageWork
    stageWork = stageDraftLoop().finally(() => { stageWork = undefined })
    return stageWork
  }
  async function stageDraftLoop() {
    if (!channelDraft.value || !snapshot.value) return
    staging.value = true; channelError.value = ''
    try {
      while (channelDraft.value && snapshot.value) {
        const serialized = JSON.stringify(channelDraft.value)
        const saved = await modelApi.stage(channelBaseline.value, workspaceBaseline.value, JSON.parse(serialized))
        workspaceBaseline.value = saved.workspaceDigest
        snapshot.value.workspace = saved.workspace; snapshot.value.workspaceDigest = saved.workspaceDigest
        if (JSON.stringify(channelDraft.value) === serialized) break
      }
    } catch (cause) { channelError.value = errorText(cause) }
    finally { staging.value = false }
  }
  function queueDraft(force = false) {
    if (stageTimer) clearTimeout(stageTimer)
    if (!force && !channelDirty.value) return
    stageTimer = setTimeout(() => { void persistDraft() }, 350)
  }
  watch(channelDraft, () => queueDraft(), { deep: true })
  onScopeDispose(() => { if (stageTimer) clearTimeout(stageTimer) })
  const channelDirty = computed(() => channelDraft.value !== null && JSON.stringify(channelDraft.value) !== channelClean.value)
  const dirty = computed(() => draft.value !== null && JSON.stringify(draft.value) !== clean.value)
  async function load() {
    if (busy.value) return
    busy.value = true; error.value = ''
    try {
      snapshot.value = await modelApi.load()
      if (!channelDraft.value) {
        channelDraft.value = JSON.parse(JSON.stringify(snapshot.value.workspace?.policy ?? snapshot.value.policy))
        channelClean.value = JSON.stringify(snapshot.value.policy)
        channelBaseline.value = snapshot.value.workspace?.basePolicy ?? snapshot.value.policyDigest
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
      snapshot.value = result.snapshot
      const template = result.snapshot.library.templates.find(item => item.id === result.templateId)
      buffers.delete(bufferKey);draft.value=null
      edit(template, result.revisionId)
      if(agentContext?.initialized)view.value.agentViews[draftSession.value]=JSON.parse(JSON.stringify(agentContext))
      selected.value[result.templateId] = result.revisionId
      status.value = result.changed ? '已保存' : '无内容变化'
    } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
  }
  function restoreChannels() {
    modelRows.value={}
    if (!snapshot.value) return
    channelDraft.value = JSON.parse(JSON.stringify(snapshot.value.policy))
    channelClean.value = JSON.stringify(channelDraft.value); channelBaseline.value = snapshot.value.policyDigest
    workspaceBaseline.value = snapshot.value.workspaceDigest
    queueDraft(true)
  }
  function acceptTransfer(saved: ModelSnapshot) {
    snapshot.value = saved
    channelDraft.value = JSON.parse(JSON.stringify(saved.workspace?.policy ?? saved.policy))
    channelClean.value = JSON.stringify(saved.policy)
    channelBaseline.value = saved.workspace?.basePolicy ?? saved.policyDigest
    workspaceBaseline.value = saved.workspaceDigest
    // The template editor keeps its original buffer and optimistic baseline.
  }

  // WP-25, WP-18, WP-22: 可见渠道与模型（用于模型选择器性能优化）
  // 可见渠道：过滤隐藏和停用的渠道
  const visibleChannels = computed(() => 
    snapshot.value?.policy?.channels.filter(c => c.enabled && !c.hidden) ?? []
  )

  // 可见模型：过滤隐藏、停用的模型，且渠道也必须可见
  const visibleModels = computed(() => {
    const channelIds = new Set(visibleChannels.value.map(c => c.id))
    return snapshot.value?.policy?.models.filter(m => 
      !m.hidden && m.enabled && channelIds.has(m.channelId)
    ) ?? []
  })

  return { modelRows, pendingRows, view, hasNewTemplate, copyCurrentTemplate, protectsRevision, restoreTemplate, snapshot, draft, draftSession, notificationTarget, policyNotificationTarget, dirty, busy, error, status, selected, channelDraft, channelDirty, channelBaseline, channelError, staging, persistDraft, acceptTransfer, load, edit, leave, save, restoreChannels, visibleChannels, visibleModels }
})

