<script setup lang="ts">
import {computed,onBeforeUnmount,ref,watch} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {useModelsStore} from './store'
import {protectedChannel} from './channelActions'
import {emptyRules,errorText,modelApi,versionLabel,type ModelPolicy,type ChangeFact,type DraftProjection} from './api'
const props=defineProps<{mode:'family'|'adopt';targets:string[]}>(),emit=defineEmits<{close:[]}>(),models=useModelsStore()
const templateId=ref(''),revisionId=ref(''),scope=ref('channels'),clearOverrides=ref(false),busy=ref(false),error=ref('')
const rules=ref(emptyRules()),modes=ref<Record<string,string>>({})
const fields=[['protocol','协议'],['contextWindow','上下文窗口'],['inputModalities','输入模态'],['reasoningEfforts','可选推理档位'],['defaultEffort','默认推理档位'],['fast','Fast支持'],['pinnedEffort','强制推理档位'],['effortMap','标签映射'],['noReasoning','不传推理参数'],['reasoningAuthoritative','推理列表权威'],['fastWire','Fast wire']] as const
const selectedTemplate=computed(()=>models.snapshot?.library.templates.find(t=>t.id===templateId.value))
watch(templateId,()=>{const t=selectedTemplate.value;revisionId.value=t?.defaultRevision??t?.revisions.at(-1)?.id??'';review.value=undefined})
const targetChannels=computed(()=>models.channelDraft?.channels.filter(c=>props.targets.includes(c.id))??[])
const frozenTargets=props.targets.map(id=>({id,label:models.channelDraft?.channels.find(c=>c.id===id)?.label??id}))
const review=ref<{policy:ModelPolicy;baseline:string;library:string;workspace:string;settings:string;projection:DraftProjection;changes:ChangeFact[];matches:{channel:string;ids:string[]}[]}>()
let generation=0,disposed=false
function settings(){return JSON.stringify([props.mode,templateId.value,revisionId.value,scope.value,clearOverrides.value,rules.value,modes.value])}
function current(){return JSON.stringify(models.channelDraft)}
function assertAvailable(){
 if(!models.snapshot||!models.channelDraft)throw {message:'请先读取渠道工作区'}
 if(models.busy||models.staging)throw {message:'工作区正在处理，请稍后重新比较'}
 if(models.pendingRows.length)throw {message:'请先处理未确认型号编辑；批量范围不会覆盖行缓冲'}
 if(new Set(props.targets).size!==props.targets.length||targetChannels.value.length!==props.targets.length||targetChannels.value.some(protectedChannel))throw {message:'渠道范围已变化或包含官方管理来源，请重新选择'}
}
async function compare(){
 if(busy.value)return
 const sequence=++generation;busy.value=true;error.value='';review.value=undefined
 try{
  assertAvailable();const baseline=current(),library=models.snapshot!.libraryDigest,workspace=models.snapshot!.workspaceDigest,options=settings()
  const candidate=JSON.parse(baseline) as ModelPolicy,t=selectedTemplate.value
  if(!t)throw {message:'请选择已保存的模型族模版'}
  const matches:{channel:string;ids:string[]}[]=[]
  if(props.mode==='adopt'){
   const revision=t.revisions.find(r=>r.id===revisionId.value);if(!revision)throw {message:'请选择精确保存版本'}
   const reference={templateId:t.id,revisionId:revision.id}
   if(scope.value==='global')candidate.templateRefs=[...candidate.templateRefs.filter(r=>r.templateId!==t.id),reference]
   else for(const channel of candidate.channels.filter(c=>props.targets.includes(c.id)))channel.templateRefs=[...channel.templateRefs.filter(r=>r.templateId!==t.id),reference]
   // Explicitly clearing overrides uses the exact selected revision, not its default pointer
   // or the post-override projection. Exact per-model references remain independent.
   if(clearOverrides.value)for(const channel of candidate.channels.filter(c=>scope.value==='global'?!protectedChannel(c):props.targets.includes(c.id))){
    const ids=[...new Set([...channel.discovered.map(m=>m.id),...channel.modelOrder,...Object.keys(channel.overrides)])]
    if(ids.length>20000)throw {message:'型号范围超过批量比较上限，请先收窄渠道范围'}
    const hit:string[]=[]
    for(const id of ids){if(sequence!==generation||disposed)return;const match=await modelApi.preview(revision.content,channel.id,id);if(match.matched){delete channel.overrides[id];hit.push(id)}}
    matches.push({channel:channel.label,ids:hit})
   }
  }else{
   const before=await modelApi.project(candidate)
   const rows=before.rows.filter(row=>props.targets.includes(row.channelId)&&row.templateRef?.templateId===t.id)
   if(!rows.length)throw {message:'所选渠道没有实际采用并匹配此族的型号，请先采用精确版本'}
   const missing=targetChannels.value.filter(c=>![...candidate.templateRefs,...c.templateRefs,...Object.values(c.modelRefs)].some(r=>r.templateId===t.id))
   if(missing.length)throw {message:'以下渠道尚未采用此族，请先套用模版或重新选择范围：'+missing.map(c=>c.label).join('、')}
   if(!fields.some(([key])=>modes.value[key]&&modes.value[key]!=='keep'))throw {message:'请选择至少一个需要调整的字段'}
   for(const channel of candidate.channels.filter(c=>props.targets.includes(c.id))){
    const override={...(channel.familyOverrides[t.id]??emptyRules())}
    for(const [key] of fields){const mode=modes.value[key]??'keep';if(mode==='inherit')Object.assign(override,{[key]:{mode:'inherit'}});else if(mode==='custom'){const value=rules.value[key];if(!value||value.mode!=='custom')throw {message:'请填写 '+fields.find(f=>f[0]===key)![1]+' 的自定义值'};Object.assign(override,{[key]:value})}}
    channel.familyOverrides={...channel.familyOverrides,[t.id]:override}
    matches.push({channel:channel.label,ids:rows.filter(r=>r.channelId===channel.id).map(r=>r.modelId)})
   }
  }
  const projection=await modelApi.project(candidate)
  if(disposed||sequence!==generation)return
  assertAvailable()
  if(current()!==baseline||models.snapshot!.libraryDigest!==library||models.snapshot!.workspaceDigest!==workspace||settings()!==options)throw {message:'比较期间工作区、模版或范围变化，请重新比较'}
  const previous=JSON.parse(baseline) as ModelPolicy,changes:ChangeFact[]=[]
  const add=(field:string,before:unknown,after:unknown)=>{if(JSON.stringify(before)!==JSON.stringify(after))changes.push({field,before,after})}
  add('全局族采用',previous.templateRefs,candidate.templateRefs)
  for(const channel of candidate.channels){const old=previous.channels.find(c=>c.id===channel.id)!;add(channel.label+' · 族采用',old.templateRefs,channel.templateRefs);add(channel.label+' · 族覆盖',old.familyOverrides,channel.familyOverrides);add(channel.label+' · 单模型覆盖',old.overrides,channel.overrides)}
  review.value={policy:candidate,baseline,library,workspace,settings:options,projection,matches,changes}
 }catch(cause){if(!disposed&&sequence===generation)error.value=errorText(cause)}finally{if(sequence===generation)busy.value=false}
}
function stop(){generation++;busy.value=false;review.value=undefined;error.value='比较已停止，原草稿保持'}
function accept(){
 const plan=review.value;if(!plan)return
 try{assertAvailable();if(plan.baseline!==current()||plan.library!==models.snapshot!.libraryDigest||plan.workspace!==models.snapshot!.workspaceDigest||plan.settings!==settings())throw {message:'确认已过期，请重新比较，原草稿保持'}
  models.channelDraft=plan.policy;emit('close')
 }catch(cause){error.value=errorText(cause);review.value=undefined}
}
onBeforeUnmount(()=>{disposed=true;generation++})
</script>
<template>
 <UiDialog :open="true" :title="mode==='family'?'按族批量配置':'套用模版精确版本'" wide :busy="busy" @close="emit('close')">
  <p v-if="error" role="alert" class="model-error">{{ error }}</p>
  <p>当前所选 {{ frozenTargets.length }} 个渠道（包含搜索外所选）：{{ frozenTargets.map(c=>c.label).join('、') }}</p>
  <UiSelectField v-model="templateId" label="模型族 / 模版" :options="[{value:'',label:'选择已保存模版'},...(models.snapshot?.library.templates??[]).map(t=>({value:t.id,label:t.label}))]" :disabled="busy" />
  <template v-if="mode==='adopt'">
   <UiSelectField v-model="revisionId" label="精确保存版本" :options="(selectedTemplate?.revisions??[]).map(r=>({value:r.id,label:versionLabel(r)}))" :disabled="busy||!selectedTemplate" />
   <UiSelectField v-model="scope" label="采用范围" :options="[{value:'channels',label:'所选渠道的该模型族'},{value:'global',label:'全部渠道的全局族默认'}]" :disabled="busy" />
   <p v-if="scope==='global'">将修改全局族采用，影响全部渠道的继承；已有渠道同族采用优先，保持不变。</p><details v-if="scope==='global'"><summary>全部渠道范围（{{ models.channelDraft?.channels.length??0 }}）</summary><p v-for="channel in models.channelDraft?.channels??[]" :key="channel.id">{{ channel.label }} · {{ channel.connection.enabled?'启用':'停用' }}</p></details>
   <label><input v-model="clearOverrides" type="checkbox" :disabled="busy" />明确清除目标匹配型号的单模型规则覆盖</label>
   <p>默认保留族覆盖、单模型规则和精确引用。清除只针对此保存版本匹配的型号规则，精确引用、命名、白名单、连接与顺序保持；模版默认指针在版本管理中独立设置。</p>
  </template>
  <template v-else>
   <p>按已经采用的精确版本匹配；只修改所选族的指定字段。未调整字段、单模型例外、选择和顺序保持。</p>
   <div class="model-columns"><label v-for="[key,label] in fields" :key="key" class="model-batch-field">{{ label }}<UiSelectField :model-value="modes[key]??'keep'" :label="label+'批量操作'" :options="[{value:'keep',label:'不修改'},{value:'inherit',label:'恢复继承'},{value:'custom',label:'自定义'}]" :disabled="busy" @update:model-value="modes[key]=$event" /></label></div>
   <ModelRuleFields v-if="fields.some(([key])=>modes[key]==='custom')" v-model="rules" :disabled="busy" />
  </template>
  <div v-if="review&&review.settings===settings()" class="model-batch-review"><h3>确认本次范围</h3><p v-if="!review.changes.length">与当前草稿一致，没有配置变化。</p><details v-else><summary>完整字段差异（{{ review.changes.length }} 项）</summary><article v-for="change in review.changes" :key="change.field"><strong>{{ change.field }}</strong><pre>当前：{{ JSON.stringify(change.before,null,2) }}</pre><pre>拟修改：{{ JSON.stringify(change.after,null,2) }}</pre></article></details><p>完整草稿激活数 {{ review.projection.knownActiveModels }} / 100；实际目录 {{ review.projection.exactCatalogCount??'保存前核对' }}。这里只加入草稿，运行配置在页级保存时写入。</p><details v-for="item in review.matches" :key="item.channel"><summary>{{ item.channel }} · {{ item.ids.length }} 个型号</summary><code v-for="id in item.ids" :key="id">{{ id }}</code></details><p v-if="mode==='adopt'">精确版本：{{ revisionId }}；{{ scope==='global'?'全局族默认':'所选渠道' }}；{{ clearOverrides?'明确清除匹配型号规则覆盖':'保留单模型例外' }}</p></div>
  <template #actions><UiButton v-if="busy" @click="stop">停止比较</UiButton><UiButton :disabled="busy||!templateId" :loading="busy" @click="compare">比较本次修改</UiButton><UiButton variant="primary" :disabled="busy||!review||review.settings!==settings()" @click="accept">加入草稿</UiButton></template>
 </UiDialog>
</template>
<style scoped>
label.model-batch-field{display:grid;align-items:stretch;gap:var(--space-2);margin:0}label{display:flex;align-items:flex-start;gap:var(--space-2);margin-block:var(--space-3)}.model-batch-review{border-top:1px solid var(--border);padding-top:var(--space-4);margin-top:var(--space-4)}code{display:block;overflow-wrap:anywhere}pre{white-space:pre-wrap;overflow-wrap:anywhere}p{overflow-wrap:anywhere}
</style>
