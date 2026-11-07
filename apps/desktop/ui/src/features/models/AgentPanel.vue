<script setup lang="ts">
import {computed,nextTick,onBeforeUnmount,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiIconButton from '@/components/ui/UiIconButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import AgentExecutionField from './AgentExecutionField.vue'
import {useModelsV2Store,type RevisionBufferV2} from './store-v2'
import {modelV2Api,type AgentSceneV2,type LibraryViewV2,type RefV2,type GenerateIntentV2} from './api-v2'
import {useModalStore} from '@/app/modal/store'
const props=withDefaults(defineProps<{draft?:RevisionBufferV2;snapshot:LibraryViewV2;generation?:number;targets?:readonly RefV2[];sessionKey?:string;active?:boolean;embedded?:boolean;disabled?:boolean}>(),{active:true,embedded:false})
const emit=defineEmits<{close:[];external:[draft:RevisionBufferV2]}>()
const models=useModelsV2Store(),modal=useModalStore(),chat=ref<HTMLElement|null>(null),input=ref<HTMLTextAreaElement|null>(null),maximized=ref(false)
const scene=ref<AgentSceneV2>(props.embedded?'check_only':'from_sources'),confirming=ref(false),researching=ref(false)
let researchSequence=0
const key=computed(()=>props.sessionKey??'template:'+props.draft?.templateId),session=computed(()=>models.agentSession(key.value))
const busy=computed(()=>session.value.preparing||session.value.generating||session.value.stopping||session.value.accepting)
const target=computed<RefV2|null>(()=>{const d=props.draft;if(!d?.baseRevision)return null;const r=props.snapshot.library.templates.find(t=>t.id===d.templateId)?.revisions.find(r=>r.id===d.baseRevision);return r?{templateId:d.templateId,revisionId:r.id,digest:r.digest}:null})
const exactTargets=computed(()=>props.targets?[...props.targets]:target.value?[target.value]:[])
const missingTarget=computed(()=>!!props.draft?.baseRevision&&!target.value)
const result=computed(()=>session.value.result)
const reportOnly=computed(()=>!!result.value?.report)
const stale=computed(()=>!!result.value&&result.value.expectedLibrary!==props.snapshot.libraryDigest)
const proposals=computed(()=>result.value?.review?.proposals??[])
const scopeMatches=computed(()=>proposals.value.every(item=>item.proposal.target?exactTargets.value.some(r=>JSON.stringify(r)===JSON.stringify(item.proposal.target)):!!props.draft&&!props.draft.baseRevision&&exactTargets.value.length===0))
const canAccept=computed(()=>scene.value!=='check_only'&&result.value?.phase==='review'&&!reportOnly.value&&result.value.review?.valid&&scopeMatches.value&&(!stale.value||!!session.value.acceptRequest)&&!busy.value&&!props.disabled)
const editChanged=computed(()=>props.generation!==undefined&&session.value.inputGeneration!==null&&props.generation!==session.value.inputGeneration)
const sceneOptions=computed(()=>props.embedded?[{value:'check_only',label:'仅检查，返回报告'},{value:'update_models',label:'生成更新建议'}]:[{value:'from_sources',label:'按当前编辑生成建议'},{value:'check_only',label:'仅检查已保存目标'}])
function capture(){const s=session.value;s.view??={scrollTop:0,selectionStart:0,selectionEnd:0,maximized:false};s.view.scrollTop=chat.value?.scrollTop??s.view.scrollTop;s.view.selectionStart=input.value?.selectionStart??0;s.view.selectionEnd=input.value?.selectionEnd??0;s.view.maximized=maximized.value}
async function restoreView(){await nextTick();const view=session.value.view;if(!view||!props.active)return;if(chat.value)chat.value.scrollTop=view.scrollTop;maximized.value=view.maximized;if(input.value)input.value.setSelectionRange(view.selectionStart,view.selectionEnd)}
function close(){capture();emit('close')}
watch(()=>props.active,value=>{if(value)void restoreView();else capture()},{immediate:true})
onBeforeUnmount(()=>{capture();researchSequence++})
async function research(mode:'read'|'search'){if(researching.value||busy.value)return;const sessionKey=key.value,request=++researchSequence;researching.value=true;session.value.error='';try{const values=mode==='read'?await modelV2Api.researchSources((session.value.researchUrls??'').split('\n').map(s=>s.trim()).filter(Boolean)):await modelV2Api.searchSources((session.value.researchQuery??'').trim());if(request!==researchSequence||key.value!==sessionKey)return;session.value.sources=values}catch(cause){if(request===researchSequence)session.value.error=cause instanceof Error?cause.message:'资料读取未完成，原有来源保持。'}finally{if(request===researchSequence)researching.value=false}}
async function generate(){
 if(busy.value||researching.value||props.disabled||missingTarget.value||!models.executionPreference)return
 const check=scene.value==='check_only'
 if(check&&!exactTargets.value.length){session.value.error='仅检查需要明确的已保存版本，新建缓冲不会被伪装成保存目标。';return}
 const message=session.value.input.trim()||(check?'检查所选保存版本的规则与参数':'按当前明确编辑补全规则，保留人工锁与原始 ID')
 const history=[...session.value.history,{role:'user' as const,text:message}]
 session.value.history=history
 const request:GenerateIntentV2={scene:scene.value,targets:exactTargets.value,allowNew:!check&&!!props.draft&&!props.draft.baseRevision,expectedLibrary:props.snapshot.libraryDigest,requirements:message,history:history.slice(-40),sources:JSON.parse(JSON.stringify(session.value.sources))}
 const buffer=!check&&props.draft?{target:target.value,label:props.draft.label,content:JSON.parse(JSON.stringify(props.draft.content)),generation:props.generation??models.editGeneration}:undefined
 await models.runAgent(key.value,request,buffer)
 const value=session.value.result;if(value?.report)session.value.history.push({role:'assistant',text:value.report.summary});else if(value?.review)session.value.history.push({role:'assistant',text:value.review.valid?'建议已返回，请审阅后明确保存。':'建议未通过完整校验，问题已保留。'})
}
async function restore(){await models.restoreAgent(key.value,props.draft?.baseRevision?props.draft.templateId:null);void restoreView()}
function accept(){if(!canAccept.value||confirming.value)return;confirming.value=true;const runId=result.value!.runId,digest=result.value!.responseDigest,generation=props.generation;modal.open({title:'接受本次已审阅建议',body:(editChanged.value?'当前编辑已经变化；本次只另存原任务建议，不覆盖当前输入。':'确认后按本次精确目标另存建议版本，当前输入、默认与已有采用保持。')+' 检查报告不会进入保存。',confirmLabel:'确认接受并保存',onConfirm:async()=>{try{if(result.value?.runId!==runId||result.value.responseDigest!==digest||props.generation!==generation){session.value.error='审阅确认期间结果或编辑已变化，原输入保持，请重新核对。';return}await models.acceptAgent(key.value,props.draft?.baseRevision?props.draft.templateId:null)}finally{confirming.value=false}},onCancel:()=>{confirming.value=false}})}
</script>
<template>
  <aside id="model-agent-panel" class="model-agent-panel" :class="{'is-maximized':maximized,'is-embedded':embedded}" role="region" aria-label="模版助手" v-show="active">
    <header><strong>{{embedded?'检查模版更新':'模版助手'}}</strong><div class="model-inline"><UiIconButton v-if="!embedded" :label="maximized?'还原大小':'放大'" @click="maximized=!maximized"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 9V4h5m6 0h5v5M4 15v5h5m6 0h5v-5" /></svg></UiIconButton><UiButton v-if="!embedded" size="sm" @click="close">收起助手</UiButton></div></header>
    <div class="model-agent-execution"><AgentExecutionField :disabled="busy" /></div>
    <div ref="chat" class="model-agent-scroll" @scroll.passive="capture">
      <div class="model-inline"><UiButton size="sm" :disabled="busy" @click="restore">恢复上次建议</UiButton><UiSelectField v-model="scene" label="Agent 本次任务" :options="sceneOptions" :disabled="busy" /></div>
      <p v-if="missingTarget" class="model-error" role="alert">当前编辑的精确保存依赖已缺失，未改选其它版本。</p>
      <p v-if="session.status" role="status">{{session.status}}</p><p v-if="session.ticket" class="model-muted">原任务 · {{session.ticket.runId}}</p>
      <div role="log" aria-label="Agent对话"><article v-for="(message,index) in session.history" :key="index" class="model-agent-message"><strong>{{message.role==='user'?'你':'助手'}}</strong><p>{{message.text}}</p></article></div>
      <details><summary>本次已核验资料 · {{session.sources.length}} 项</summary><label>检索关键词<input v-model="session.researchQuery" class="input" aria-label="Agent资料关键词" /></label><UiButton size="sm" :disabled="busy||researching||!session.researchQuery?.trim()" @click="research('search')">检索公开资料</UiButton><label>公开 HTTPS 资料地址（每行一个）<textarea v-model="session.researchUrls" class="input" aria-label="Agent公开资料地址" rows="3" /></label><UiButton size="sm" :disabled="busy||researching||!session.researchUrls?.trim()" @click="research('read')">读取在线资料</UiButton><p class="model-muted">只发送有后端来源回执的资料；本页不会把任意 URL 文本当成已核验来源。</p><p v-for="source in session.sources" :key="source.receiptId??source.url">{{source.url}} · {{source.retrievedAt}}</p></details>
      <template v-if="result?.report"><h3>检查报告</h3><p class="model-agent-report">{{result.report.summary}}</p><ul><li v-for="(issue,index) in result.report.issues" :key="index">{{issue.code}} · {{issue.message}}<span v-if="issue.fieldPath"> · {{issue.fieldPath}}</span></li></ul><p>仅检查返回报告，没有建议版本或配置写入。</p></template>
      <template v-if="result?.review"><h3>待审阅建议 · {{proposals.length}} 项</h3><p v-if="stale" class="model-error" role="alert">已保存库已变化，当前建议保留；不能把旧结果应用到新基准。</p><p v-if="!scopeMatches" class="model-error" role="alert">该结果不属于当前精确目标范围，未允许接受。</p><p v-if="editChanged" role="status">当前编辑已变化；审阅原建议不会覆盖当前输入，保存前会再次确认。</p><article v-for="item in proposals" :key="item.index" class="model-agent-diff"><h4>{{item.proposal.label}} · {{item.proposal.target?.revisionId??'明确新建目标'}}</h4><p v-if="item.plannedRevision" class="model-muted">后端计划版本 {{item.plannedRevision.revisionId}}</p><details v-for="(difference,index) in item.differences" :key="index"><summary>{{difference.fieldPath}}</summary><pre>{{JSON.stringify(difference.before,null,2)}}</pre><pre>{{JSON.stringify(difference.after,null,2)}}</pre></details><details><summary>完整 v2 条件内容</summary><pre>{{JSON.stringify(item.proposal.content,null,2)}}</pre></details><p v-for="(issue,index) in item.issues" :key="index" class="model-error">{{issue.code}} · {{issue.message}}</p></article><p v-for="(issue,index) in result.review.issues" :key="index" role="alert" class="model-error">{{issue.code}} · {{issue.message}}</p><UiButton v-if="result.phase==='review'&&scene!=='check_only'" variant="primary" :disabled="!canAccept||confirming" :loading="session.accepting" @click="accept">{{session.acceptRequest?'核对原接受结果':'接受本次完整建议'}}</UiButton><p v-if="result.phase==='accepted'" role="status">原任务建议已接受保存，当前编辑缓冲保持。</p></template>
      <p v-if="session.error" class="model-error" role="alert">{{session.error}}</p>
    </div>
    <footer><label>制模要求<textarea ref="input" v-model="session.input" class="input" aria-label="制模要求" rows="2" @select="capture" @blur="capture" /></label><div class="model-inline"><UiButton v-if="session.preparing||session.generating" :disabled="session.stopping" @click="models.stopAgent(key)">停止</UiButton><UiButton v-else variant="primary" :disabled="busy||researching||disabled||!models.executionPreference||missingTarget||confirming" @click="generate">{{scene==='check_only'?'开始检查':'生成建议'}}</UiButton><UiButton v-if="draft" size="sm" :disabled="busy" @click="emit('external',draft)">外部任务</UiButton></div></footer>
  </aside>
</template>
<style scoped>
.model-agent-panel{position:fixed;right:12px;bottom:12px;z-index:var(--z-overlay);width:min(400px,calc(100vw / var(--ui-zoom) - 24px));max-height:calc(100vh / var(--ui-zoom) - 48px);display:flex;flex-direction:column;background:var(--surface-overlay-bg,var(--panel));backdrop-filter:var(--surface-overlay-blur);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow);overflow:hidden}.model-agent-panel.is-maximized{width:calc(100vw / var(--ui-zoom) - 24px);height:calc(100vh / var(--ui-zoom) - 48px)}.model-agent-panel.is-embedded{position:static;width:100%;max-height:none;border:0;box-shadow:none;background:none}.model-agent-panel>header,.model-agent-panel>footer{flex:none;padding:14px;display:flex;gap:12px;align-items:center;justify-content:space-between}.model-agent-panel>header{border-bottom:1px solid var(--border)}.model-agent-panel>footer{border-top:1px solid var(--border);display:grid;gap:8px}.model-agent-panel>footer label{display:grid;gap:8px}.model-agent-execution{padding:10px 14px;border-bottom:1px solid var(--border);flex:none}.model-agent-scroll{min-height:0;flex:1;padding:14px;overflow:auto;display:grid;gap:14px}.model-agent-message p,.model-agent-report{white-space:pre-wrap;overflow-wrap:anywhere}.model-agent-diff{padding:10px 0;border-bottom:1px solid var(--border)}.model-agent-diff pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:240px;overflow:auto}.model-agent-panel textarea{width:100%;min-height:56px;max-height:140px;resize:vertical;box-sizing:border-box}.model-error{color:var(--red)}.model-agent-panel svg{fill:none;stroke:currentColor;stroke-width:1.8}
</style>
