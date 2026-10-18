import { computed, onScopeDispose, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { emptyContent, errorText, modelApi, type ModelSnapshot, type ModelPolicy, type Template, type TemplateDraft } from './api'

// Route changes retain drafts; a refresh never discards an editor buffer.
export const useModelsStore = defineStore('model-config', () => {
  const snapshot = ref<ModelSnapshot | null>(null)
  const draft = ref<TemplateDraft | null>(null)
  const draftSession = ref('')
  const clean = ref('')
  const editBaseline = ref('')
  const busy = ref(false)
  const error = ref('')
  const status = ref('')
  const selected = ref<Record<string, string>>({})
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
    draftSession.value = crypto.randomUUID()
    const revision = template?.revisions.find(item => item.id === revisionId) ?? template?.revisions.at(-1)
    draft.value = { templateId: copy ? null : template?.id ?? null, baseRevision: copy ? null : revision?.id ?? null, label: template ? `${template.label}${copy ? ' 副本' : ''}` : '', content: revision ? JSON.parse(JSON.stringify(revision.content)) : emptyContent() }
    clean.value = JSON.stringify(draft.value); error.value = ''; status.value = ''
    editBaseline.value = snapshot.value?.libraryDigest ?? ''
  }
  function leave() { draft.value = null; clean.value = ''; status.value = ''; error.value = '' }
  async function save() {
    if (!snapshot.value || !draft.value || busy.value) return
    const request = JSON.parse(JSON.stringify(draft.value)) as TemplateDraft
    const expected = editBaseline.value
    busy.value = true; error.value = ''
    try {
      const result = await modelApi.save(request, expected)
      snapshot.value = result.snapshot
      const template = result.snapshot.library.templates.find(item => item.id === result.templateId)
      edit(template, result.revisionId)
      selected.value[result.templateId] = result.revisionId
      status.value = result.changed ? '已保存' : '无内容变化'
    } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
  }
  function restoreChannels() {
    if (!snapshot.value) return
    channelDraft.value = JSON.parse(JSON.stringify(snapshot.value.policy))
    channelClean.value = JSON.stringify(channelDraft.value); channelBaseline.value = snapshot.value.policyDigest
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
  return { snapshot, draft, draftSession, notificationTarget, policyNotificationTarget, dirty, busy, error, status, selected, channelDraft, channelDirty, channelBaseline, channelError, staging, persistDraft, acceptTransfer, load, edit, leave, save, restoreChannels }
})
