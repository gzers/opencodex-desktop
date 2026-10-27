<script setup lang="ts">
import { computed,onBeforeUnmount,ref,watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import AgentExecutionField from './AgentExecutionField.vue'
import { modelApi,errorText,type AgentSettings,type ModelSnapshot,type Proposal,type ResearchSource,type TemplateDraft,type Submission } from './api'
import { proposalFields,acceptProposal } from './proposals'
import { useModelsStore } from './store'
const props=defineProps<{open:boolean;snapshot:ModelSnapshot;selected:string[];versions:Record<string,string>}>()
const emit=defineEmits<{close:[]}>()
const models=useModelsStore()
const scope=ref<'all'|'selected'>('all');const settings=ref<AgentSettings|null>(null);const requirements=ref('');const query=ref('');const sources=ref<ResearchSource[]>([])
const reading=ref(false);const busy=ref(false);const saving=ref(false);const error=ref('');const report=ref('');const run=ref('');const resultRun=ref('');const result=ref<Submission|null>(null);const frozen=ref<ModelSnapshot|null>(null)
const chosen=ref<Record<string,string[]>>({});const saved=ref(false)
type Review=Parameters<typeof modelApi.acceptAgentReview>[0]
const review=ref<Review|null>(null)
let sequence=0
watch(()=>props.open,value=>{if(value){scope.value=props.selected.length?'selected':'all';error.value=''}},{immediate:true})
const targets=computed(()=>props.snapshot.library.templates.filter(t=>scope.value==='all'||props.selected.includes(t.id)).map(t=>({templateId:t.id,revisionId:t.revisions.find(r=>r.id===props.versions[t.id])?.id??t.revisions.at(-1)!.id})))
function draft(snapshot:ModelSnapshot,proposal:Proposal):TemplateDraft|null {const family=snapshot.library.templates.find(t=>t.id===proposal.templateId);const revision=family?.revisions.find(r=>r.id===proposal.baseRevision);return family&&revision?{templateId:family.id,baseRevision:revision.id,label:revision.savedLabel,content:revision.content}:null}
const items=computed(()=>result.value?.proposals.flatMap(proposal=>{const base=frozen.value&&draft(frozen.value,proposal);if(!base)return[];const fields=proposalFields(base,base,proposal);return[{id:proposal.templateId!,proposal,base,fields}];})??[])
const stale=computed(()=>frozen.value&&frozen.value.libraryDigest!==props.snapshot.libraryDigest)
const selectedCount=computed(()=>Object.values(chosen.value).reduce((count,fields)=>count+fields.length,0))
async function research(){if(reading.value)return;reading.value=true;error.value='';try{sources.value=await modelApi.searchSources(query.value)}catch(cause){error.value=errorText(cause)}finally{reading.value=false}}
async function restore(){
 if(busy.value||saving.value)return
 error.value=''
 try{const pending=await modelApi.latestAgentResult(null);if(!pending){report.value='没有可恢复的内部建议';return}if(pending.result.proposals.some(p=>!p.templateId))throw new Error('上次为新建模版建议，请在编辑器中恢复')
  const captured=JSON.parse(JSON.stringify(props.snapshot)) as ModelSnapshot;captured.libraryDigest=pending.expectedLibrary;captured.library.templates=pending.targets.map(t=>({id:t.templateId,label:t.label,source:'user',defaultRevision:null,revisions:[t.revision]}));frozen.value=captured;result.value=pending.result;resultRun.value=pending.runId;review.value=null;saved.value=pending.reviewed;chosen.value={};report.value=new Date(pending.createdAt).toLocaleString('zh-CN')+' · '+(pending.reviewed?'该建议已另存保存版本':pending.result.report??'上次建议已恢复，尚未保存');for(const item of items.value)chosen.value[item.id]=item.fields.filter(f=>!f.locked&&!f.conflict).map(f=>f.path.join('.'))
 }catch(cause){error.value=errorText(cause)}
}
async function generate(){
 if(busy.value||saving.value||!settings.value||!targets.value.length||review.value)return
 const current=++sequence;const id=crypto.randomUUID();const captured=structuredClone(JSON.parse(JSON.stringify(props.snapshot))) as ModelSnapshot;const selected=targets.value.map(t=>({...t}));run.value=id;busy.value=true;error.value=''
 try{const value=await modelApi.generate(id,{scene:'update_models',targets:selected,allowNew:false,requirements:requirements.value,expectedLibrary:captured.libraryDigest,buffers:[]},sources.value)
  if(current!==sequence)return
  if(value.proposals.some(p=>!selected.some(t=>t.templateId===p.templateId&&t.revisionId===p.baseRevision)))throw new Error('内部结果超出本次已保存范围')
  frozen.value=captured;result.value=value;resultRun.value=id;report.value=value.report??'';saved.value=false;chosen.value={}
  for(const item of items.value)chosen.value[item.id]=item.fields.filter(f=>!f.locked&&!f.conflict).map(f=>f.path.join('.'))
 }catch(cause){if(current===sequence)error.value=errorText(cause)}finally{if(current===sequence){busy.value=false;run.value=''}}
}
async function stop(){const id=run.value;if(!id)return;sequence++;run.value='';busy.value=false;try{await modelApi.stopGeneration(id)}catch(cause){error.value=errorText(cause)}}
function close(){if(saving.value)return;if(run.value)void stop();emit('close')}
onBeforeUnmount(()=>{sequence++;if(run.value)void modelApi.stopGeneration(run.value)})
async function save(){
 if(saving.value||busy.value||saved.value)return
 if(!review.value){if(stale.value||!frozen.value||!selectedCount.value)return;const proposals=items.value.filter(item=>chosen.value[item.id]?.length).map(item=>{const accepted=acceptProposal(item.base,item.fields,chosen.value[item.id]??[]);return{templateId:accepted.templateId,baseRevision:accepted.baseRevision,label:accepted.label,content:accepted.content}});review.value={runId:resultRun.value,expectedLibrary:frozen.value.libraryDigest,proposals}}
 saving.value=true;error.value=''
 try{
  const committed=await modelApi.acceptAgentReview(JSON.parse(JSON.stringify(review.value)))
  // Agent commits carry the legacy manager policy. Only the library changed;
  // retain the file snapshot and every channel/Key edit still in memory.
  if(models.snapshot)models.snapshot={...models.snapshot,library:committed.snapshot.library,libraryDigest:committed.snapshot.libraryDigest}
  saved.value=true;report.value='已另存 '+committed.outcomes.filter(o=>o.changed).length+' 个版本；默认与已有采用保持。'
 }catch(cause){error.value=errorText(cause)}finally{saving.value=false}
}
</script>
<template>
 <UiDialog :open="open" title="检查模版更新" :busy="saving" wide @close="close">
  <div class="batch-agent-form"><AgentExecutionField v-model="settings" :disabled="busy||saving" />
   <div class="model-inline"><label><input v-model="scope" type="radio" value="all" :disabled="busy||!!review" />全部已保存模版</label><label><input v-model="scope" type="radio" value="selected" :disabled="busy||!!review||!selected.length" />已勾选 {{ selected.length }} 项</label><span>{{ targets.length }} 个目标</span></div>
   <label>补充要求<textarea v-model="requirements" class="input" rows="3" :disabled="busy||!!review" /></label>
   <details><summary>在线资料</summary><div class="model-inline"><input v-model="query" class="input" type="search" aria-label="批量更新资料关键词" placeholder="官方资料关键词" :disabled="busy||!!review" /><UiButton :loading="reading" :disabled="!query.trim()||busy||!!review" @click="research">检索</UiButton></div><p v-for="source in sources" :key="source.url" class="model-muted">{{ source.url }}</p></details>
   <p v-if="report" class="batch-agent-report">{{ report }}</p><p v-if="stale&&!saved" role="alert" class="model-error">本机已保存内容已变化，当前建议保留；请按新基准重新检查。</p>
   <article v-for="item in items" :key="item.id" class="batch-agent-result"><h3>{{ item.proposal.label }}</h3><label v-for="field in item.fields" :key="field.path.join('.')"><input v-model="chosen[item.id]" type="checkbox" :value="field.path.join('.')" :disabled="field.locked||busy||!!review" />{{ field.label }}{{ field.locked?' · 人工锁':'' }}<details><summary>查看差异</summary><pre>{{ JSON.stringify(field.before,null,2) }}</pre><pre>{{ JSON.stringify(field.after,null,2) }}</pre></details></label><p v-if="!item.fields.length" class="model-muted">未提出内容变化</p></article>
   <p v-if="error" role="alert" class="model-error">{{ error }}</p>
  </div>
  <template #actions><UiButton :disabled="busy||saving||!!review" @click="restore">恢复上次建议</UiButton><UiButton v-if="busy" @click="stop">停止</UiButton><UiButton :disabled="saving" @click="close">关闭</UiButton><UiButton v-if="!review" :disabled="!settings||!targets.length||reading" :loading="busy" @click="generate">{{ result?'重新检查':'检查更新' }}</UiButton><UiButton v-if="result&&!saved" variant="primary" :loading="saving" :disabled="busy||(!review&&(!selectedCount||!!stale))" @click="save">{{ review?'核对原保存结果':'另存所选建议版本' }}</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.batch-agent-form{display:grid;gap:18px}.batch-agent-form>label{display:grid;gap:8px}.batch-agent-result{padding:14px 0;border-bottom:1px solid var(--border)}.batch-agent-result h3{font-size:var(--text-body)}.batch-agent-result>label{display:block;margin:10px 0}.batch-agent-result details{margin:8px 0 8px 20px}.batch-agent-result pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:180px;overflow:auto}.batch-agent-report{white-space:pre-wrap;overflow-wrap:anywhere}.model-error{color:var(--red)}
</style>
