<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import ModelPicker, { type ModelChoice } from './ModelPicker.vue'
import { modelApi, errorText, type AgentSettings, type DraftProjection, type ModelSnapshot, type Proposal, type ResearchSource, type TemplateDraft, type ChannelPolicy } from './api'
import { mergeUpstream } from './upstream'
import { acceptProposal, proposalFields } from './proposals'
const props = defineProps<{ draft: TemplateDraft; snapshot: ModelSnapshot }>()
const emit = defineEmits<{ close: []; apply: [draft: TemplateDraft] }>()
const prompt = ref(''); const urls = ref(''); const sources = ref<ResearchSource[]>([]); const settings = ref<AgentSettings | null>(null)
const query = ref('')
const error = ref(''); const busy = ref(false); const researching = ref(false); const configuring = ref(false); const picker = ref(false)
const directory = ref<DraftProjection | null>(null); const channels = ref<ChannelPolicy[]>([])
const run = ref(''); const result = ref<Proposal | null>(null); const base = ref<TemplateDraft | null>(null); const chosen = ref<string[]>([])
const history = ref<{ role: string; text: string }[]>([])
const sameTarget = computed(() => !!base.value && base.value.templateId === props.draft.templateId && base.value.baseRevision === props.draft.baseRevision)
const fields = computed(() => base.value && result.value && sameTarget.value ? proposalFields(base.value, props.draft, result.value) : [])
watch(fields, values => { chosen.value = chosen.value.filter(path => values.some(field => field.path.join('.') === path && !field.conflict && !field.locked)) })
onMounted(async () => { try { settings.value = await modelApi.agentSettings() } catch (cause) { error.value = errorText(cause) } })
let sequence = 0
onBeforeUnmount(() => { sequence++; if (run.value) void modelApi.stopGeneration(run.value) })
async function configure() {
  configuring.value = true; error.value = ''
  try { const value = await modelApi.upstream(); channels.value = mergeUpstream(value, []); directory.value = await modelApi.project({ schemaVersion: 1, templateRefs: [], channels: channels.value }); picker.value = true }
  catch (cause) { error.value = errorText(cause) } finally { configuring.value = false }
}
async function saveExecution(picks: ModelChoice[]) {
  const pick = picks[0]; if (!pick) return
  const channel = channels.value.find(c => c.id === pick.channelId); if (!channel) return
  try { settings.value = await modelApi.saveAgentSettings({ providerId: channel.connection.providerId, modelId: pick.modelId, reasoningEffort: pick.reasoningEffort ?? null, fast: pick.fast ?? null }); picker.value = false }
  catch (cause) { error.value = errorText(cause) }
}
async function readSources() {
  researching.value = true; error.value = ''
  try { sources.value = await modelApi.research(urls.value.split('\n').map(v => v.trim()).filter(Boolean)) }
  catch (cause) { error.value = errorText(cause) } finally { researching.value = false }
}
async function searchSources() {
  researching.value = true; error.value = ''
  try { sources.value = await modelApi.searchSources(query.value) }
  catch (cause) { error.value = errorText(cause) } finally { researching.value = false }
}
async function generate() {
  if (busy.value || !settings.value) return
  const current = ++sequence; const captured: TemplateDraft = JSON.parse(JSON.stringify(props.draft)); const id = crypto.randomUUID(); run.value = id
  busy.value = true; error.value = ''; result.value = null; chosen.value = []
  const input = prompt.value.trim(); history.value.push({ role: '你', text: input || '根据当前模版与资料补全规则' })
  try {
    const value = await modelApi.generate(id, { scene: 'from_sources', targets: captured.templateId && captured.baseRevision ? [{ templateId: captured.templateId, revisionId: captured.baseRevision }] : [], allowNew: !captured.templateId, requirements: [...history.value.slice(-8).map(item => item.role+'：'+item.text), '当前表单：'+JSON.stringify(captured)].join('\n'), expectedLibrary: props.snapshot.libraryDigest, buffers: captured.templateId && captured.baseRevision ? [{ templateId: captured.templateId, baseRevision: captured.baseRevision, label: captured.label, content: captured.content }] : [] }, sources.value)
    if (current !== sequence) return
    if (value.proposals.length !== 1) throw new Error('当前编辑仅接收一个目标，请缩小范围')
    const suggestion = value.proposals[0]!
    if (suggestion.templateId !== captured.templateId || suggestion.baseRevision !== captured.baseRevision) throw new Error('建议目标与当前编辑基准不一致')
    base.value = captured; result.value = suggestion
    chosen.value = proposalFields(captured, props.draft, suggestion).filter(f => !f.conflict && !f.locked).map(f => f.path.join('.'))
    history.value.push({ role: '助手', text: value.report || '已生成建议，请比较后接受；尚未保存。' })
  } catch (cause) { if (current === sequence) error.value = errorText(cause) }
  finally { if (current === sequence) { busy.value = false; run.value = '' } }
}
async function stop() { const id = run.value; sequence++; busy.value = false; run.value = ''; if (id) await modelApi.stopGeneration(id) }
function accept() { if (sameTarget.value) { emit('apply', acceptProposal(props.draft, fields.value, chosen.value)); result.value = null; history.value.push({ role:'助手',text:'所选建议已放入表单，仍需保存。' }) } }
</script>
<template>
  <aside class="model-agent-panel" aria-label="模版助手"><header><strong>模版助手</strong><UiButton size="sm" variant="ghost" @click="emit('close')">收起</UiButton></header>
    <div class="model-agent-scroll"><UiButton :loading="configuring" @click="configure">{{ settings ? settings.providerId+' / '+settings.modelId : '配置执行模型' }}</UiButton><p v-if="settings" class="model-muted">{{ settings.reasoningEffort ?? '默认推理' }} · {{ settings.fast === null ? '默认Fast' : settings.fast ? 'Fast开' : 'Fast关' }}</p>
      <details><summary>在线资料</summary><label>检索关键词<input class="input" v-model="query" /></label><UiButton size="sm" :loading="researching" :disabled="!query.trim()" @click="searchSources">检索公开资料</UiButton><label>公开HTTPS资料地址（每行一个）<textarea class="input" v-model="urls" rows="3" /></label><UiButton size="sm" :loading="researching" @click="readSources">读取在线资料</UiButton><p v-for="source in sources" :key="source.url" class="model-muted">{{ source.url }} · {{ source.retrievedAt }}</p></details>
      <div v-for="(item,i) in history" :key="i" class="model-agent-message"><strong>{{ item.role }}</strong><p>{{ item.text }}</p></div>
      <template v-if="result"><p v-if="!sameTarget" class="model-error">该建议属于先前编辑对象，未应用到当前表单。</p><article v-for="field in fields" :key="field.path.join('.')" class="model-agent-diff"><label><input v-model="chosen" type="checkbox" :value="field.path.join('.')" :disabled="field.locked" />{{ field.label }}{{ field.locked ? ' · 人工锁' : field.conflict ? ' · 生成期间有人工修改，默认保留' : '' }}</label><details><summary>查看前后内容</summary><pre>{{ JSON.stringify(field.current,null,2) }}</pre><pre>{{ JSON.stringify(field.after,null,2) }}</pre></details></article><UiButton :disabled="!sameTarget || !chosen.length" @click="accept">接受所选建议</UiButton></template>
      <p v-if="error" role="alert" class="model-error">{{ error }}</p>
    </div>
    <footer><textarea class="input" v-model="prompt" rows="3" placeholder="填写资料或继续纠偏" aria-label="制模要求" /><div class="model-inline"><UiButton v-if="busy" @click="stop">停止</UiButton><UiButton v-else variant="primary" :disabled="!settings || researching" @click="generate">发送</UiButton></div></footer>
    <ModelPicker :open="picker" :rows="directory?.rows ?? []" :channels="channels" reasoning title="执行模型" @close="picker = false" @confirm="saveExecution" />
  </aside>
</template>
<style scoped>
.model-agent-panel{position:fixed;right:24px;bottom:24px;z-index:var(--z-overlay);width:min(400px,calc(100vw - 48px));max-height:calc(100vh - 90px);display:flex;flex-direction:column;background:var(--surface-overlay-bg,var(--panel));backdrop-filter:var(--surface-overlay-blur);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow);overflow:hidden}.model-agent-panel>header,.model-agent-panel>footer{padding:14px;display:flex;gap:12px;align-items:center;justify-content:space-between}.model-agent-panel>header{border-bottom:1px solid var(--border)}.model-agent-panel>footer{border-top:1px solid var(--border);display:grid}.model-agent-scroll{padding:14px;overflow:auto;display:grid;gap:14px}.model-agent-scroll label{display:grid;gap:6px}.model-agent-message p{white-space:pre-wrap;overflow-wrap:anywhere}.model-agent-diff{padding:10px 0;border-bottom:1px solid var(--border)}.model-agent-diff label{display:flex;align-items:center;gap:8px}.model-agent-diff pre{white-space:pre-wrap;max-height:160px;overflow:auto}.model-agent-panel textarea{width:100%;resize:vertical}
</style>
