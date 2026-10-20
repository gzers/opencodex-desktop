<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch, toRef } from 'vue'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import ChannelsView from '@/features/models/ChannelsView.vue'
import ExternalTaskDialog from '@/features/models/ExternalTaskDialog.vue'
import AgentPanel from '@/features/models/AgentPanel.vue'
import BatchAgentDialog from '@/features/models/BatchAgentDialog.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyRules, errorText, modelApi, versionLabel, type MatchPreview, type Pattern, type Revision, type Template, type OwnerTaskResult } from '@/features/models/api'
import { useModelViewPosition } from '@/features/models/useViewPosition'
import { useModalStore } from '@/app/modal/store'
const models = useModelsStore()
const modal=useModalStore()
const page = ref<HTMLElement | null>(null)
const detailHead = ref<HTMLElement | null>(null)
let headerObserver: ResizeObserver | undefined
watch(detailHead, head => {
  headerObserver?.disconnect()
  const measure = () => page.value?.style.setProperty('--model-header-height', `${head?.offsetHeight ?? 0}px`)
  if (head && typeof ResizeObserver !== 'undefined') { headerObserver = new ResizeObserver(measure); headerObserver.observe(head) }
  measure()
})
onBeforeUnmount(() => headerObserver?.disconnect())
const tab = toRef(models.view,'tab')
useModelViewPosition(page,()=>tab.value==='templates'?(models.view.templateEditor&&models.draft?'template:'+models.view.templatePosition:'templates'):(models.view.channelEditor?'channel:'+models.view.channelEditor:'channels'))
watch(()=>models.policyNotificationTarget,target=>{if(target)tab.value='channels'},{immediate:true})
const agentOpen = ref(false)
const batchOpen=ref(false)
const externalOpen = ref(false)
const receiptOpen = ref(false)
const receiptResult = ref<OwnerTaskResult | null>(null)
const receiptError = ref('')
let receiptSequence = 0
watch(() => models.notificationTarget, async target => {
  if (!target) return
  const sequence = ++receiptSequence; receiptOpen.value = true; receiptError.value = ''; receiptResult.value = null
  try { const result = await modelApi.externalResult(target.taskId,target.submissionId); if (sequence === receiptSequence) receiptResult.value = result }
  catch (cause) { if (sequence === receiptSequence) receiptError.value = errorText(cause) }
}, { immediate: true })
const selectedTemplates = toRef(models.view,'selectedTemplates')
const search = toRef(models.view,'templateSearch')
const draft = computed(() => models.draft)
const templates = computed(() => models.snapshot?.library.templates.filter(t => t.label.toLowerCase().includes(search.value.toLowerCase())) ?? [])
const managerId = ref('')
const manager = computed(() => models.snapshot?.library.templates.find(t => t.id === managerId.value))
const selectedIds = ref<string[]>([])
const historySearch = ref('')
const confirmDelete = ref(false)
const deletion = ref<{ library: string; policy: string; workspace: string; template: string; ids: string[] } | null>(null)
const historyError = ref('')
const versions = computed(() => [...(manager.value?.revisions ?? [])].reverse().filter(r => versionLabel(r).includes(historySearch.value) || r.changes.some(c => c.field.includes(historySearch.value))))
const selectedVersions = computed(() => manager.value?.revisions.filter(r => (deletion.value?.ids ?? selectedIds.value).includes(r.id)) ?? [])
const previewOpen = ref(false)
const sampleChannel = ref('')
const sampleModel = ref('')
const preview = ref<MatchPreview | null>(null)
const previewError = ref('')
const previewBusy = ref(false)
const kinds = [{ value: 'prefix', label: '前缀' }, { value: 'exact', label: '精确' }, { value: 'glob', label: '通配符' }]
let stopChanges: (() => void) | undefined
let disposed = false
onMounted(async () => {
  void models.load()
  try {
    const { listen } = await import('@tauri-apps/api/event')
    const stop = await listen('model-config-changed', () => { void models.load() })
    if (disposed) stop(); else stopChanges = stop
  } catch { /* Explicit IPC errors remain visible through the saved-state read. */ }
})
onBeforeUnmount(() => { disposed = true; stopChanges?.() })
function edit(t?: Template, revisionId?: string, copy = false) { if(models.busy)return;models.edit(t, revisionId, copy);models.view.templatePosition=models.draftSession;models.view.templateEditor=true;managerId.value='' }
function restoreTemplate(){
  const baseline=JSON.stringify(models.draft),session=models.draftSession
  modal.open({title:'还原模版修改',body:'还原当前模版到本次编辑的起点？其他模版与渠道草稿保持。',confirmLabel:'还原修改',onConfirm:()=>{if(JSON.stringify(models.draft)!==baseline||models.draftSession!==session){models.error='还原确认已过期，请重新核对';return}models.restoreTemplate()}})
}
function current(t: Template) { return t.revisions.find(r => r.id === models.selected[t.id]) ?? t.revisions[t.revisions.length - 1]! }
function choices(t: Template) {
  const ids = new Set([...t.revisions.slice(-5).map(r => r.id), current(t).id, t.defaultRevision])
  return [...t.revisions].reverse().filter(r => ids.has(r.id)).map(r => ({ value: r.id, label: versionLabel(r, t.source === 'builtin' && r === t.revisions[0]) }))
}
function manage(t: Template) { managerId.value = t.id; selectedIds.value = []; historySearch.value = ''; confirmDelete.value = false; deletion.value = null; historyError.value = '' }
const commonLocks=computed(()=>draft.value?.content.lockedFields.filter(key=>key.startsWith('rules.')).map(key=>key.slice(6))??[])
function setCommonLocks(values:string[]){if(draft.value)draft.value.content.lockedFields=[...draft.value.content.lockedFields.filter(key=>!key.startsWith('rules.')),...values.map(key=>'rules.'+key)]}
function contentLock(field:string){if(!draft.value)return;const fields=draft.value.content.lockedFields;draft.value.content.lockedFields=fields.includes(field)?fields.filter(key=>key!==field):[...fields,field]}
function protectedReason(t: Template, r: Revision) {
  if (t.source === 'builtin' && r === t.revisions[0]) return '内置基线'
  if (t.defaultRevision === r.id) return '默认版本'
  if (models.protectsRevision(t.id,r.id)) return '未保存编辑基准'
  return ''
}
async function makeDefault(t: Template, r: Revision) {
  if (!models.snapshot || models.busy) return
  models.busy = true; historyError.value = ''
  try { models.snapshot = await modelApi.setDefault(models.snapshot.libraryDigest, t.id, r.id) }
  catch (e) { historyError.value = errorText(e) } finally { models.busy = false }
}
async function remove() {
  if (!models.snapshot || !manager.value || models.busy) return
  if (!deletion.value) return
  const { library: libraryDigest, policy: policyDigest, workspace: workspaceDigest, template: id, ids } = deletion.value
  models.busy = true; historyError.value = ''
  try {
    models.snapshot = await modelApi.delete(libraryDigest, policyDigest, workspaceDigest, id, ids)
    if (draft.value?.templateId === id && ids.includes(draft.value.baseRevision ?? '')) models.leave()
    selectedIds.value = []; confirmDelete.value = false; deletion.value = null
    if (!models.snapshot.library.templates.some(t => t.id === id)) managerId.value = ''
  } catch (e) { historyError.value = errorText(e) } finally { models.busy = false }
}
async function checkMatch() {
  if (!draft.value) return
  previewBusy.value = true; previewError.value = ''; preview.value = null
  try { preview.value = await modelApi.preview(JSON.parse(JSON.stringify(draft.value.content)), sampleChannel.value, sampleModel.value) }
  catch (e) { previewError.value = errorText(e) } finally { previewBusy.value = false }
}
function anchor(id: string) { document.getElementById(`model-${id}`)?.scrollIntoView({ block: 'start' }) }
function addVariant() { draft.value?.content.variants.push({ id: crypto.randomUUID(), name: '', suffix: '', rules: emptyRules(), lockedFields: [] }) }
function prepareDelete() {
  if (!models.snapshot || !manager.value) return
  deletion.value = { library: models.snapshot.libraryDigest, policy: models.snapshot.policyDigest, workspace: models.snapshot.workspaceDigest, template: manager.value.id, ids: [...selectedIds.value] }
  confirmDelete.value = true
}
</script>
<template>
  <section ref="page" class="models-page">
    <AppTopbar />
    <StickyTabs><div class="model-tabs-line"><div class="model-tabs ext-tabs" role="tablist" aria-label="模型配置"><button class="tab" :class="{ active: tab === 'channels' }" role="tab" :aria-selected="tab === 'channels'" @click="tab = 'channels'">渠道模型</button><button class="tab" :class="{ active: tab === 'templates' }" role="tab" :aria-selected="tab === 'templates'" @click="tab = 'templates'">模型模版</button></div><div id="model-tabs-actions"></div></div></StickyTabs>
    <p v-if="models.error" class="model-error" role="alert">{{ models.error }}</p>
    <div v-if="!models.snapshot" class="model-empty"><p>{{ models.busy ? '正在读取模型配置…' : '模型配置读取未完成' }}</p><UiButton :disabled="models.busy" @click="models.load">重新读取</UiButton></div>
    <ChannelsView v-if="models.snapshot" v-show="tab==='channels'" :active="tab==='channels'" actions-target="#model-tabs-actions" />
    <template v-if="models.snapshot&&tab==='templates'&&(!draft||!models.view.templateEditor)">
      <div class="model-toolbar"><input class="input" v-model="search" type="search" placeholder="搜索模版" aria-label="搜索模版" /><div class="model-inline"><UiButton v-if="draft" @click="models.view.templateEditor=true">继续编辑</UiButton><UiButton @click="batchOpen=true">{{ selectedTemplates.length?'检查勾选更新':'检查全部更新' }}</UiButton><UiButton @click="externalOpen = true">外部任务</UiButton><UiButton variant="primary" @click="edit()">新建模版</UiButton></div></div>
      <div class="model-table-scroll"><table class="model-table"><thead><tr><th>模版</th><th>版本</th><th>操作</th></tr></thead><tbody><tr v-for="t in templates" :key="t.id">
        <td><label class="model-inline"><input v-model="selectedTemplates" type="checkbox" :value="t.id" :aria-label="`勾选${t.label}`" /><strong>{{ t.label }}</strong></label><span class="tag">{{ t.source === 'builtin' ? '内置' : '自建' }}</span><small>{{ t.source === 'builtin' && current(t) === t.revisions[0] ? '内置基线' : current(t).changes.map(c => c.field).slice(0, 3).join('、') || '内容变更依据缺失' }}</small></td>
        <td><UiSelectField :model-value="current(t).id" :options="choices(t)" :label="`${t.label}保存版本`" size="sm" @update:model-value="models.selected[t.id] = $event" /></td>
        <td><div class="model-inline"><UiButton size="sm" :data-model-focus="'template:'+t.id" @click="edit(t, current(t).id)">编辑</UiButton><UiButton size="sm" @click="edit(t, current(t).id, true)">复制</UiButton><UiButton size="sm" @click="manage(t)">版本管理</UiButton></div></td>
      </tr></tbody></table></div><p v-if="!templates.length" class="model-empty">没有匹配的模版</p>
    </template>
    <template v-if="models.snapshot&&tab==='templates'&&draft&&models.view.templateEditor">
      <header ref="detailHead" class="model-detail-head"><UiButton variant="ghost" @click="models.view.templateEditor=false">返回</UiButton><input v-model="draft.label" aria-label="模版名称" placeholder="模版名称" class="input model-name" :disabled="models.busy" /><span class="tag">{{ models.dirty ? '未保存修改' : models.status || (draft.templateId ? '未修改' : '新建未保存') }}</span><UiButton :disabled="models.busy" @click="agentOpen = true">Agent自动填充</UiButton><UiButton :disabled="models.busy" @click="externalOpen = true">外部任务</UiButton><UiButton :disabled="models.busy" @click="previewOpen = true">匹配预览</UiButton><UiButton variant="ghost" :disabled="!models.dirty||models.busy" @click="restoreTemplate">还原修改</UiButton><UiButton variant="primary" :loading="models.busy" :disabled="!models.dirty && !!draft.templateId" @click="models.save">保存</UiButton></header>
      <div class="model-editor"><nav class="model-anchors" aria-label="模版分区"><a v-for="item in [['basic','基本信息'],['matching','匹配规则'],['rules','通用配置'],['variants','版本或后缀'],['source','来源与版本']]" :key="item[0]" :href="`#model-${item[0]}`" @click.prevent="anchor(item[0]!)">{{ item[1] }}</a></nav><div class="model-form">
        <section id="model-basic"><h2>基本信息</h2><label>说明<textarea class="input" v-model="draft.content.description" rows="3" :disabled="models.busy" /></label><label class="model-inline"><input type="checkbox" :checked="draft.content.lockedFields.includes('description')" :disabled="models.busy" @change="contentLock('description')" />锁定说明</label></section>
        <section id="model-matching"><h2>匹配规则</h2><div v-for="kind in (['include','exclude'] as const)" :key="kind"><div class="model-field-title"><strong>{{ kind === 'include' ? '包含' : '排除' }}</strong><UiButton size="sm" :disabled="models.busy" @click="draft.content.matching[kind].push({ kind: 'prefix', value: '' })">添加规则</UiButton></div>
          <div v-for="(p, i) in draft.content.matching[kind]" :key="i" class="model-pattern"><UiSelectField :model-value="p.kind" :options="kinds" label="匹配类型" :disabled="models.busy" @update:model-value="p.kind = $event as Pattern['kind']" /><input class="input" v-model="p.value" :aria-label="`${kind}规则${i + 1}`" placeholder="原始模型ID或匹配文本" :disabled="models.busy" /><UiButton size="sm" variant="ghost" :disabled="models.busy" @click="draft.content.matching[kind].splice(i, 1)">移除</UiButton></div></div>
          <label class="model-inline"><input v-model="draft.content.matching.caseSensitive" type="checkbox" :disabled="models.busy" />区分大小写</label>
          <div class="model-columns"><label>渠道范围<UiSelectField :model-value="draft.content.matching.channelMode" :options="[{value:'all',label:'全部渠道'},{value:'whitelist',label:'仅所选渠道'},{value:'blacklist',label:'排除所选渠道'}]" label="渠道范围" :disabled="models.busy" @update:model-value="draft.content.matching.channelMode = $event as 'all' | 'whitelist' | 'blacklist'" /></label><label>优先级<input class="input" v-model.number="draft.content.matching.priority" type="number" :disabled="models.busy" /></label></div>
          <fieldset v-if="draft.content.matching.channelMode !== 'all'"><legend>选择渠道</legend><label v-for="channel in models.channelDraft?.channels ?? []" :key="channel.id" class="model-inline"><input v-model="draft.content.matching.channels" type="checkbox" :value="channel.id" :disabled="models.busy" />{{ channel.label }}</label><p v-if="!models.channelDraft?.channels.length" class="model-muted">暂无渠道，可先保存规则。</p><p v-for="id in draft.content.matching.channels.filter(id => !models.channelDraft?.channels.some(c => c.id === id))" :key="id" class="model-muted">已失效渠道引用 · {{ id.slice(0,8) }}</p></fieldset>
        </section>
        <section id="model-rules"><h2>通用配置</h2><ModelRuleFields v-model="draft.content.rules" :locked-fields="commonLocks" :disabled="models.busy" @update:locked-fields="setCommonLocks" /></section>
        <section id="model-variants"><div class="model-field-title"><h2>版本或后缀</h2><UiButton size="sm" :disabled="models.busy" @click="addVariant">新增版本或后缀</UiButton></div><p v-if="!draft.content.variants.length" class="model-muted">暂无型号差异规则</p>
          <article v-for="(v, i) in draft.content.variants" :key="v.id" class="model-variant"><div class="model-columns"><label>名称<input class="input" v-model="v.name" :disabled="models.busy" /></label><label>原始ID字面后缀<input class="input" v-model="v.suffix" :disabled="models.busy" /></label></div><ModelRuleFields v-model="v.rules" v-model:locked-fields="v.lockedFields" :disabled="models.busy" /><UiButton size="sm" variant="danger" :disabled="models.busy" @click="draft.content.variants.splice(i, 1)">移除此项</UiButton></article>
        </section>
        <section id="model-source"><h2>来源与版本</h2><p>{{ draft.templateId ? '基于已保存版本编辑；保存产生独立版本。' : '自建模版，保存后生成独立身份。' }}</p><p v-if="draft.baseRevision" class="mono">{{ draft.baseRevision }}</p><details v-for="(source,index) in draft.content.sources" :key="index" class="model-source-record"><summary>{{ source.field }} · {{ source.recordedAt.slice(0,10) }}</summary><p v-if="source.url" class="mono">{{ source.url }}</p><p>{{ source.note }}</p></details></section>
      </div></div>
    </template>
    <UiDialog :open="!!manager" :title="`版本管理 · ${manager?.label ?? ''}`" :busy="models.busy" wide @close="managerId = ''"><template v-if="manager">
      <p v-if="historyError" role="alert" class="model-error">{{ historyError }}</p>
      <template v-if="confirmDelete"><h3>确认删除所选版本？</h3><ul><li v-for="r in selectedVersions" :key="r.id">{{ versionLabel(r) }}</li></ul><p>确认后删除这些版本，已有依赖会再次核对。</p></template>
      <template v-else><div class="model-toolbar"><input class="input" v-model="historySearch" type="search" placeholder="搜索时间、标识或变更字段" aria-label="搜索版本" /><span>已选 {{ selectedIds.length }} 项</span></div><article v-for="r in versions" :key="r.id" class="model-version"><label><input v-model="selectedIds" type="checkbox" :value="r.id" :disabled="!!protectedReason(manager, r) || models.busy" /><span>{{ versionLabel(r, manager.source === 'builtin' && r === manager.revisions[0]) }}<small>{{ protectedReason(manager,r) || r.changes.map(c => c.field).join('、') || '内容变更依据缺失' }}</small></span></label><div class="model-inline"><UiButton size="sm" :disabled="models.busy" @click="edit(manager, r.id)">编辑</UiButton><UiButton size="sm" :disabled="models.busy || manager.defaultRevision === r.id" @click="makeDefault(manager,r)">{{ manager.defaultRevision === r.id ? '当前默认' : '设为默认' }}</UiButton></div></article></template>
    </template><template #actions><UiButton v-if="confirmDelete" :disabled="models.busy" @click="confirmDelete = false; deletion = null">取消</UiButton><UiButton variant="danger" :disabled="!selectedIds.length" :loading="models.busy" @click="confirmDelete ? remove() : prepareDelete()">{{ confirmDelete ? '确认删除' : '删除所选版本' }}</UiButton></template></UiDialog>
    <UiDialog :open="previewOpen" title="匹配预览" @close="previewOpen = false"><div class="model-form"><label>渠道ID<input class="input" v-model="sampleChannel" /></label><label>原始模型ID<input class="input" v-model="sampleModel" /></label><p v-if="previewError" class="model-error" role="alert">{{ previewError }}</p><p v-if="preview" role="status">{{ preview.reason }}{{ preview.variantId ? ' · 已叠加最长后缀规则' : '' }}</p></div><template #actions><UiButton variant="primary" :loading="previewBusy" :disabled="!sampleModel" @click="checkMatch">检查匹配</UiButton></template></UiDialog>
    <AgentPanel v-if="models.snapshot && models.draft" :key="models.draftSession" v-show="agentOpen" :draft="models.draft" :snapshot="models.snapshot" @close="agentOpen = false" @apply="models.draft = $event" />
    <UiDialog :open="receiptOpen" title="外部任务结果" @close="receiptOpen = false"><p v-if="receiptError" class="model-error" role="alert">{{ receiptError }}</p><template v-if="receiptResult"><p>目标：{{ receiptResult.task.targets.map(t => t.label).join('、') || '新建模版' }}</p><p>状态：{{ !receiptResult.receipt ? '结果待核对' : ({saved:'已保存',unchanged:'无内容变化',report:'检查报告',rejected:'提交被拒绝，未保存'}[receiptResult.receipt.state] ?? '待核对') }}</p><p v-if="receiptResult.receipt?.report" class="model-task-report">{{ receiptResult.receipt.report }}</p><p v-for="outcome in receiptResult.receipt?.outcomes ?? []" :key="outcome.revisionId">保存版本 · {{ outcome.revisionId }}</p></template></UiDialog>
    <ExternalTaskDialog v-if="models.snapshot" :open="externalOpen" :snapshot="models.snapshot" :draft="models.draft" :dirty="models.dirty" :selected="selectedTemplates" @close="externalOpen = false" />
    <BatchAgentDialog v-if="models.snapshot" :open="batchOpen" :snapshot="models.snapshot" :selected="selectedTemplates" :versions="models.selected" @close="batchOpen=false" />
  </section>
</template>
<style>
.model-tabs,.model-detail-head{display:flex;align-items:center;gap:var(--space-2);border-bottom:1px solid var(--border);padding:10px 0}
.model-tabs{position:static;min-height:48px;background:transparent}
.model-tabs-line{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);flex-wrap:wrap}
.model-policy-actions{justify-content:flex-end;padding:6px 0}
.model-policy-state{display:inline-flex;align-items:center;gap:6px;min-height:var(--control-md);padding:0 10px;border:1px solid var(--border);border-radius:var(--radius-sm);font-size:var(--text-caption);white-space:nowrap;background:var(--surface-panel-bg,var(--panel))}
.model-policy-state[data-tone=warning]{color:var(--amber);border-color:color-mix(in srgb,var(--amber) 30%,var(--border))}
.model-policy-state[data-tone=danger]{color:var(--red);border-color:color-mix(in srgb,var(--red) 30%,var(--border))}
.model-policy-state[data-tone=success]{color:var(--green)}
.model-source-record{padding:10px 0}.model-source-record p{white-space:pre-wrap;overflow-wrap:anywhere}
.model-detail-head{position:sticky;z-index:8;top:var(--page-tabs-height,48px);flex-wrap:wrap;background:var(--surface-sticky-bg,var(--surface-panel-bg));-webkit-backdrop-filter:var(--surface-sticky-blur);backdrop-filter:var(--surface-sticky-blur)}
@supports not ((backdrop-filter:blur(1px)) or (-webkit-backdrop-filter:blur(1px))){.model-detail-head{background:var(--panel)}}
.model-name{flex:1;min-width:160px;font-weight:650}
.model-toolbar{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);margin:20px 0;flex-wrap:wrap}
.model-toolbar input{flex:1;max-width:360px;min-width:180px}
.model-table-scroll{overflow-x:auto}
.model-table{width:100%;border-collapse:collapse}
.model-table th{color:var(--muted);font-weight:500;text-align:left}
.model-table th,.model-table td{padding:14px 8px;border-bottom:1px solid var(--border);vertical-align:middle}
.model-table td:first-child{min-width:140px}
.model-table td:nth-child(2){min-width:250px}
.model-table .tag{margin-left:8px}
.model-table small,.model-version small{display:block;margin-top:5px;color:var(--muted)}
.model-inline{display:flex;flex-wrap:wrap;align-items:center;gap:var(--space-2)}
.model-editor{display:grid;grid-template-columns:145px minmax(0,1fr);gap:28px;margin-top:20px}
.model-anchors{position:sticky;top:calc(var(--page-tabs-height,48px) + var(--model-header-height,48px) + 20px);align-self:start;display:grid;gap:4px}
.model-anchors a{color:var(--muted);padding:8px;border-radius:var(--radius-sm);text-decoration:none}
.model-anchors a:hover{color:var(--text);background:var(--hover)}
.model-form{display:grid;gap:20px}
.model-form section{scroll-margin-top:calc(var(--page-tabs-height,48px) + var(--model-header-height,48px) + 20px);padding-bottom:24px;border-bottom:1px solid var(--border)}
.model-form h2{font-size:var(--text-title);margin:0 0 16px}
.model-form label,.model-rule-fields>label,.model-columns>label{display:grid;gap:7px}
.model-form label.model-inline,.model-inline label{display:flex;align-items:center;gap:7px}
.model-rule-fields{margin:0;padding:0;border:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px;min-width:0}
.model-field-wide{grid-column:1/-1}
.model-field-title{display:flex;justify-content:space-between;align-items:center;gap:12px;margin:12px 0}
.model-field-title h2{margin:0}
.model-field-title label{display:flex;gap:6px;align-items:center}
.model-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin:14px 0}
.model-pattern{display:grid;grid-template-columns:110px minmax(0,1fr) auto;gap:8px;margin:8px 0}
.model-variant{border:1px solid var(--border);border-radius:var(--radius);padding:16px;margin-top:16px}
.model-variant>button{margin-top:16px}
.model-error{color:var(--red);overflow-wrap:anywhere}
.model-empty{padding:32px 0;text-align:center;color:var(--muted)}
.model-muted{color:var(--muted)}
.model-version{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:14px;padding:16px 0;border-bottom:1px solid var(--border)}
.model-version>label{display:flex;align-items:center;gap:10px}@media(max-width:800px){.model-editor{grid-template-columns:minmax(0,1fr)}
.model-anchors{position:static;display:flex;flex-wrap:wrap}
.model-rule-fields,.model-columns{grid-template-columns:minmax(0,1fr)}
.model-pattern{grid-template-columns:90px minmax(0,1fr)}
.model-pattern>button{justify-self:end;grid-column:2}}
</style>
