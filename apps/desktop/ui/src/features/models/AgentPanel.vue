<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import UiIconButton from '@/components/ui/UiIconButton.vue'
import UiButton from '@/components/ui/UiButton.vue'
import AgentExecutionField from './AgentExecutionField.vue'
import type {AgentViewContext} from './agentView'
import { modelApi, errorText, type AgentSettings, type ModelSnapshot, type Proposal, type ResearchSource, type TemplateDraft } from './api'
import { acceptProposal, proposalFields } from './proposals'
const props = withDefaults(defineProps<{ draft: TemplateDraft; snapshot: ModelSnapshot;active?:boolean;viewContext?:AgentViewContext }>(),{active:true})
const emit = defineEmits<{ close: []; apply: [draft: TemplateDraft]; external: [draft:TemplateDraft] }>()
const prompt = ref(''); const urls = ref(''); const sources = ref<ResearchSource[]>([]); const settings = ref<AgentSettings | null>(null)
const query = ref('')
const lockContext=ref(props.viewContext?.lockContext??true),sourcesOpen=ref(props.viewContext?.sourcesOpen??false)
const sourceSummary=computed(()=>sources.value.length?'资料 '+sources.value.length+' 项':'资料')
function requestDraft(){const value=copy(props.draft);if(lockContext.value&&value.content.rules.contextWindow.mode==='custom'&&!value.content.lockedFields.includes('rules.contextWindow'))value.content.lockedFields.push('rules.contextWindow');return value}
function external(){emit('external',copy(props.draft))}
async function toggleSources(){sourcesOpen.value=!sourcesOpen.value;if(sourcesOpen.value){await nextTick();const detail=panel.value?.querySelector('#model-agent-sources');if(detail&&chat.value)chat.value.scrollTop+=detail.getBoundingClientRect().top-chat.value.getBoundingClientRect().top}}
const restoreStatus=ref('')
const error = ref(''); const busy = ref(false); const researching = ref(false)
const run = ref(''); const result = ref<Proposal | null>(null); const base = ref<TemplateDraft | null>(null); const chosen = ref<string[]>([])
const history = ref<{ role: string; text: string }[]>([])
const panel=ref<HTMLElement|null>(null),chat=ref<HTMLElement|null>(null),input=ref<HTMLTextAreaElement|null>(null),maximized=ref(false)
const copy=<T,>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
if(props.viewContext?.initialized){const view=props.viewContext;prompt.value=view.input;urls.value=view.urls;query.value=view.query;history.value=copy(view.history);sources.value=copy(view.sources);base.value=copy(view.base);result.value=copy(view.proposal);chosen.value=[...view.chosen];maximized.value=view.maximized}
function captureView(wasActive=props.active){
 const view=props.viewContext;if(!view)return
 Object.assign(view,{initialized:true,lockContext:lockContext.value,sourcesOpen:sourcesOpen.value,input:prompt.value,urls:urls.value,query:query.value,history:copy(history.value),sources:copy(sources.value),scrollTop:chat.value?.scrollTop??view.scrollTop,selectionStart:input.value?.selectionStart??view.selectionStart,selectionEnd:input.value?.selectionEnd??view.selectionEnd,focusInput:wasActive&&!!panel.value?.contains(document.activeElement)||view.focusInput,base:copy(base.value),proposal:copy(result.value),chosen:[...chosen.value],maximized:maximized.value})
}
watch([prompt,history,sources,urls,query,base,result,chosen,maximized,lockContext,sourcesOpen],()=>captureView(),{deep:true,flush:'sync'})
async function restoreView(){await nextTick();if(panelDisposed||!props.active)return;const view=props.viewContext;if(!view?.initialized)return;if(chat.value)chat.value.scrollTop=view.scrollTop;if(input.value){input.value.setSelectionRange(view.selectionStart,view.selectionEnd);if(view.focusInput)input.value.focus({preventScroll:true})}}
function collapse(){captureView();emit('close')}
watch(()=>props.active,(active,previous)=>{if(active)void restoreView();else captureView(previous)})

const sameTarget = computed(() => !!base.value && base.value.templateId === props.draft.templateId && base.value.baseRevision === props.draft.baseRevision)
const fields = computed(() => base.value && result.value && sameTarget.value ? proposalFields(base.value, props.draft, result.value) : [])
watch(fields, values => { chosen.value = chosen.value.filter(path => values.some(field => field.path.join('.') === path && !field.conflict && !field.locked)) })
let panelDisposed=false;let conversationDigest:string|null=null;let conversationReady=false;let conversationDirty=false;let conversationTimer:ReturnType<typeof setTimeout>|undefined;let conversationWrite:Promise<void>|null=null
function conversationContent(){return {baseRevision:props.draft.baseRevision,input:prompt.value,history:history.value.map(item=>({role:item.role==='你'?'user' as const:'assistant' as const,text:item.text})),sources:JSON.parse(JSON.stringify(sources.value)) as ResearchSource[]}}
async function persistConversation(){
 if(conversationWrite)return conversationWrite
 if(!conversationReady||!conversationDirty)return
 conversationWrite=(async()=>{while(conversationDirty){conversationDirty=false;try{const saved=await modelApi.saveAgentConversation({templateId:props.draft.templateId,expectedDigest:conversationDigest,content:conversationContent()});conversationDigest=saved.digest}catch(cause){conversationDirty=true;error.value=errorText(cause);break}}})().finally(()=>{conversationWrite=null})
 return conversationWrite
}
watch([prompt,history,sources],()=>{conversationDirty=true;if(conversationTimer)clearTimeout(conversationTimer);conversationTimer=setTimeout(()=>void persistConversation(),350)},{deep:true})
onMounted(async()=>{void restoreView();try{const previous=await modelApi.agentConversation(props.draft.templateId);if(panelDisposed)return;conversationDigest=previous?.digest??null;conversationReady=true;if(conversationDirty)void persistConversation()}catch(cause){if(!panelDisposed)error.value=errorText(cause)}})
let sequence = 0;let researchSequence=0
onBeforeUnmount(() => { captureView();if(conversationTimer)clearTimeout(conversationTimer);void persistConversation();panelDisposed=true;sequence++;researchSequence++; if (run.value) void modelApi.stopGeneration(run.value) })
async function readSources() {
  const current=++researchSequence;researching.value = true; error.value = ''
  try { const found=await modelApi.research(urls.value.split('\n').map(v => v.trim()).filter(Boolean));if(current===researchSequence)sources.value=found }
  catch (cause) {if(current===researchSequence) error.value = errorText(cause) } finally { if(current===researchSequence)researching.value = false }
}
async function searchSources() {
  const current=++researchSequence;researching.value = true; error.value = ''
  try {const found=await modelApi.searchSources(query.value);if(current===researchSequence)sources.value=found }
  catch (cause) { if(current===researchSequence)error.value = errorText(cause) } finally {if(current===researchSequence) researching.value = false }
}
async function restoreSuggestion(){
 if(busy.value)return
 const current=++sequence;busy.value=true;error.value=''
 try{const inputBefore=prompt.value;const previous=await modelApi.agentConversation(props.draft.templateId);const pending=await modelApi.latestAgentResult(props.draft.templateId);if(current!==sequence)return;
 if(previous){conversationDigest=previous.digest;conversationReady=true;history.value=previous.content.history.map(item=>({role:item.role==='user'?'你':'助手',text:item.text}));sources.value=previous.content.sources;if(prompt.value===inputBefore)prompt.value=previous.content.input}
 if(!pending){restoreStatus.value=previous?'本机对话与未发送输入已恢复，当前没有待审建议':'没有可恢复的内部对话或建议';return}
 const suggestion=pending.result.proposals.find(p=>p.templateId===props.draft.templateId);if(!suggestion)throw new Error('上次结果不包含当前编辑对象')
 const target=pending.targets.find(t=>t.templateId===props.draft.templateId)
 base.value=target?(target.buffer??{templateId:target.templateId,baseRevision:target.revision.id,label:target.revision.savedLabel,content:target.revision.content}):JSON.parse(JSON.stringify(props.draft))
 result.value=suggestion;chosen.value=[];sources.value=previous?.content.sources??pending.sources??[];history.value=previous?.content.history.length?previous.content.history.map(item=>({role:item.role==='user'?'你':'助手',text:item.text})):pending.history?.length?pending.history.map(item=>({role:item.role==='user'?'你':'助手',text:item.text})):[{role:'助手',text:pending.result.report??'上次待审建议'}];restoreStatus.value=new Date(pending.createdAt).toLocaleString('zh-CN')+' · '+(pending.reviewed?'该建议已有另存版本，当前编辑缓冲保持':'对话与建议已恢复，请逐项比较')
 }catch(cause){if(current===sequence)error.value=errorText(cause)}finally{if(current===sequence)busy.value=false}
}
async function generate() {
  if (busy.value || !settings.value) return
  const current = ++sequence; const captured: TemplateDraft = requestDraft(); const id = crypto.randomUUID(); run.value = id
  busy.value = true; error.value = '';restoreStatus.value=''; result.value = null; chosen.value = []
  const input = prompt.value.trim(); history.value.push({ role: '你', text: input || '根据当前模版与资料补全规则' })
  try {
    await persistConversation()
    const value = await modelApi.generate(id, { scene: 'from_sources', targets: captured.templateId && captured.baseRevision ? [{ templateId: captured.templateId, revisionId: captured.baseRevision }] : [], allowNew: !captured.templateId, requirements: [input||'根据当前模版与资料补全规则', ...(!captured.templateId?['新建表单：'+JSON.stringify(captured)]:[])].join('\n'), expectedLibrary: props.snapshot.libraryDigest, buffers: captured.templateId && captured.baseRevision ? [{ templateId: captured.templateId, baseRevision: captured.baseRevision, label: captured.label, content: captured.content }] : [] }, sources.value,history.value.slice(-40).map(item=>({role:item.role==='你'?'user':'assistant',text:item.text})))
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
  <aside id="model-agent-panel" ref="panel" class="model-agent-panel" role="dialog" aria-modal="false" :class="{'is-maximized':maximized}" aria-label="模版助手"><header><strong>模版助手</strong><div class="model-inline"><UiIconButton :label="maximized?'还原大小':'放大'" :title="maximized?'还原大小':'放大'" :aria-pressed="maximized" @click="maximized=!maximized"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true"><path v-if="maximized" d="M9 3v6H3m18 0h-6V3M3 15h6v6m6 0v-6h6"/><path v-else d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5"/></svg></UiIconButton><UiIconButton label="收起" title="收起助手" @click="collapse"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg></UiIconButton></div></header>
    <div class="model-agent-execution"><AgentExecutionField v-model="settings" /></div><div ref="chat" class="model-agent-scroll" @scroll.passive="captureView()"><UiButton size="sm" :disabled="busy" @click="restoreSuggestion">恢复上次对话与建议</UiButton>
      <p v-if="restoreStatus" class="model-muted" role="status">{{ restoreStatus }}</p>
      <details id="model-agent-sources" :open="sourcesOpen" @toggle="sourcesOpen=($event.target as HTMLDetailsElement).open"><summary>在线资料</summary><label>检索关键词<input class="input" v-model="query" /></label><UiButton size="sm" :loading="researching" :disabled="!query.trim()" @click="searchSources">检索公开资料</UiButton><label>公开HTTPS资料地址（每行一个）<textarea class="input" v-model="urls" rows="3" /></label><UiButton size="sm" :loading="researching" @click="readSources">读取在线资料</UiButton><p v-for="source in sources" :key="source.url" class="model-muted">{{ source.url }} · {{ source.retrievedAt }}</p></details>
      <div role="log" aria-live="polite" aria-relevant="additions text" aria-label="Agent对话"><div v-for="(item,i) in history" :key="i" class="model-agent-message"><strong>{{ item.role }}</strong><p>{{ item.text }}</p></div></div>
      <p v-if="history.length>40" class="model-muted">本次发送最近 40 条消息；完整对话继续保留在本机。</p>
      <template v-if="result"><p v-if="!sameTarget" class="model-error">该建议属于先前编辑对象，未应用到当前表单。</p><article v-for="field in fields" :key="field.path.join('.')" class="model-agent-diff"><label><input v-model="chosen" type="checkbox" :value="field.path.join('.')" :disabled="field.locked" />{{ field.label }}{{ field.locked ? ' · 人工锁' : field.conflict ? ' · 生成期间有人工修改，默认保留' : '' }}</label><details><summary>查看前后内容</summary><pre>{{ JSON.stringify(field.current,null,2) }}</pre><pre>{{ JSON.stringify(field.after,null,2) }}</pre></details></article><UiButton :disabled="!sameTarget || !chosen.length" @click="accept">接受所选建议</UiButton></template>
      <p v-if="error" role="alert" class="model-error">{{ error }}</p>
    </div>
    <footer><div class="model-agent-context"><UiButton size="sm" variant="ghost" :aria-expanded="sourcesOpen" aria-controls="model-agent-sources" @click="toggleSources">{{ sourceSummary }}</UiButton><label title="保留本轮内部建议前已有的人工上下文窗口"><input v-model="lockContext" type="checkbox" :disabled="busy" />锁定上下文</label><UiIconButton label="导出上下文" title="导出上下文到外部任务" :disabled="busy" @click="external"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true"><path d="M12 16V3m-5 5 5-5 5 5M4 14v6h16v-6"/></svg></UiIconButton></div><textarea ref="input" class="input" v-model="prompt" rows="2" placeholder="填写资料或继续纠偏" aria-label="制模要求" @select="captureView()" @keyup="captureView()" @blur="captureView()" /><div class="model-inline"><UiButton v-if="busy" @click="stop">停止</UiButton><UiButton v-else variant="primary" :disabled="!settings || researching" @click="generate">发送</UiButton></div></footer>
  </aside>
</template>
<style scoped>
.model-agent-panel{position:fixed;right:12px;bottom:12px;z-index:var(--z-overlay);width:min(400px,calc(100vw / var(--ui-zoom) - 24px));max-height:calc(100vh / var(--ui-zoom) - 48px);display:flex;flex-direction:column;background:var(--surface-overlay-bg,var(--panel));backdrop-filter:var(--surface-overlay-blur);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow);overflow:hidden}.model-agent-panel.is-maximized{width:calc(100vw / var(--ui-zoom) - 24px);height:calc(100vh / var(--ui-zoom) - 48px)}.model-agent-panel>header,.model-agent-panel>footer{flex:none;padding:14px;display:flex;gap:12px;align-items:center;justify-content:space-between}.model-agent-panel>header{border-bottom:1px solid var(--border)}.model-agent-panel>footer{border-top:1px solid var(--border);display:grid;grid-template-columns:minmax(0,1fr) auto;justify-content:normal;align-items:end;gap:8px;padding:10px}.model-agent-context{grid-column:1/-1;display:flex;flex-wrap:wrap;gap:8px;align-items:center;min-width:0}.model-agent-context label{display:flex;gap:6px;align-items:center;font-size:var(--text-control)}.model-agent-context svg,.model-agent-panel>header svg{width:16px;height:16px;stroke-width:1.8}.model-agent-panel>footer>.model-inline{justify-self:end}.model-agent-execution{padding:10px 14px;border-bottom:1px solid var(--border);flex:none}.model-agent-scroll{min-height:0;flex:1;padding:14px;overflow:auto;display:grid;gap:14px}.model-agent-scroll label{display:grid;gap:6px}.model-agent-message p{white-space:pre-wrap;overflow-wrap:anywhere}.model-agent-diff{padding:10px 0;border-bottom:1px solid var(--border)}.model-agent-diff label{display:flex;align-items:center;gap:8px}.model-agent-diff pre{white-space:pre-wrap;max-height:160px;overflow:auto}.model-agent-panel textarea{width:100%;min-height:56px;max-height:96px;resize:vertical}
</style>
