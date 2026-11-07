<script setup lang="ts">
import {computed,onBeforeUnmount,ref,watch} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsV2Store,type RevisionBufferV2} from './store-v2'
import {modelV2Api,type AgentSceneV2,type LibraryViewV2,type RefV2,type GenerateIntentV2,type ExternalBrokerBundleV2} from './api-v2'
import {useModalStore} from '@/app/modal/store'
const props=defineProps<{open:boolean;snapshot:LibraryViewV2;draft?:RevisionBufferV2|null;generation?:number;selected?:string[];versions?:Record<string,string>}>()
const emit=defineEmits<{close:[]}>()
const store=useModelsV2Store(),scope=ref<'current'|'selected'|'all'|'new'>('all'),scene=ref<AgentSceneV2>('from_sources'),allowNew=ref(false),requirements=ref('')
const preparing=ref(false),copying=ref(false),reading=ref(false),revoking=ref(false),error=ref(''),status=ref(''),content=ref(''),taskId=ref(''),expiresAt=ref(''),activated=ref(false),revoked=ref(false),confirmation=ref(false)
const output=ref<HTMLTextAreaElement|null>(null)
// Activation ticket is session-private and is never rendered, copied, serialized to Pinia or logged.
let ticket='',sequence=0,disposed=false,preparedContext=''
const frozenRequest=ref<GenerateIntentV2|null>(null)
const key=computed(()=>taskId.value?'external:'+taskId.value:''),session=computed(()=>key.value?store.agentSession(key.value):null),result=computed(()=>session.value?.result)
const busy=computed(()=>copying.value||revoking.value||!!session.value?.accepting)
const scenes=[{value:'from_sources',label:'从资料制模'},{value:'update_models',label:'更新型号资料'},{value:'audit_and_revise',label:'批量检查与修订'},{value:'private_models',label:'私有模型配置'},{value:'preferences',label:'按偏好调整'},{value:'check_only',label:'仅检查，不保存'}]
function context(){return JSON.stringify({scope:scope.value,scene:scene.value,allowNew:allowNew.value,requirements:requirements.value,library:props.snapshot.libraryDigest,draft:props.draft,generation:props.generation,selected:props.selected,versions:props.versions,source:store.file?.sourceDigest,root:store.file?.rootRevision})}
function reason(cause:unknown){return cause instanceof Error?cause.message:typeof cause==='object'&&cause!==null&&'code'in cause?String(cause.code):'操作未完成，原任务与输入保持。'}
function exactReference(id:string,revisionId?:string|null):RefV2{const t=props.snapshot.library.templates.find(t=>t.id===id),revision=t?.revisions.find(r=>r.id===revisionId);if(!t||!revision)throw new Error('请选择可核验的精确保存版本；不会采用任意最新版本。');return {templateId:t.id,revisionId:revision.id,digest:revision.digest}}
function request():GenerateIntentV2{
 let targets:RefV2[]=[]
 if(scope.value==='current'){if(!props.draft?.baseRevision)throw new Error('当前模版没有已保存基准，请选择仅新建。');targets=[exactReference(props.draft.templateId,props.draft.baseRevision)]}
 if(scope.value==='selected'){if(!props.selected?.length)throw new Error('请先勾选明确保存模版。');targets=props.selected.map(id=>{const t=props.snapshot.library.templates.find(t=>t.id===id);return exactReference(id,props.versions?.[id]??t?.defaultRevision)})}
 if(scope.value==='all')targets=props.snapshot.library.templates.map(t=>exactReference(t.id,props.versions?.[t.id]??t.defaultRevision))
 if(scene.value==='check_only'&&!targets.length)throw new Error('仅检查需要明确保存版本，不能把新建缓冲当成保存目标。')
 if(!targets.length&&scope.value!=='new')throw new Error('当前范围没有明确目标，请选择仅新建或已保存模版。')
 return {scene:scene.value,targets,allowNew:scene.value!=='check_only'&&(scope.value==='new'||allowNew.value),expectedLibrary:props.snapshot.libraryDigest,requirements:requirements.value,history:[],sources:[]}
}
function restoreTask(){const id=store.externalNotificationTarget?.taskId??store.externalCurrentId,meta=store.externalTasks[id??''];if(!id)return false;taskId.value=id;frozenRequest.value=meta?{...JSON.parse(JSON.stringify(meta.request)),requirements:'',history:[],sources:[]}:null;expiresAt.value=meta?.expiresAt??'待核对';activated.value=meta?.activated??false;revoked.value=meta?.revoked??false;ticket='';content.value='';status.value='原任务已恢复，可读取原结果；复制授权内容需重新准备。';return true}
watch(()=>props.open,open=>{if(open&&!taskId.value&&!restoreTask()){scope.value=props.draft?(props.draft.baseRevision?'current':'new'):props.selected?.length?'selected':'all';error.value=''}if(!open&&!activated.value){sequence++;preparing.value=false}},{immediate:true})
watch(()=>store.externalNotificationTarget,value=>{if(value&&value.taskId!==taskId.value&&!busy.value){sequence++;preparing.value=false;taskId.value='';if(!restoreTask())error.value='原任务目标元数据尚未取得，未借用其他任务结果。'}})
watch(context,()=>{if(preparing.value){sequence++;preparing.value=false}if(taskId.value&&!activated.value&&preparedContext!==context()){ticket='';status.value='任务基准或输入已变化，请重新准备；原任务尚未激活。'}})
onBeforeUnmount(()=>{disposed=true;sequence++;ticket=''})
async function prepare(){
 if(preparing.value||busy.value||reading.value)return
 const seq=++sequence,guard=context();preparing.value=true;error.value='';let bundle:ExternalBrokerBundleV2|undefined
 try{
  const intent=request(),d=props.draft
  if(taskId.value&&!revoked.value){await modelV2Api.externalRevoke(taskId.value);revoked.value=true;activated.value=false;ticket=''}
  if(scene.value!=='check_only'&&d&&(scope.value==='current'||scope.value==='new')){const input={target:scope.value==='current'?intent.targets[0]!:null,label:d.label,content:JSON.parse(JSON.stringify(d.content)),generation:props.generation??store.editGeneration};const buffer=await modelV2Api.agentBuffer(input);const {digest,...actual}=buffer;if(!digest||JSON.stringify(actual)!==JSON.stringify(input))throw new Error('缓冲回执不属于当前编辑，未准备任务。');intent.buffer=buffer}
  if(seq!==sequence||guard!==context()||disposed)return
  bundle=await modelV2Api.externalPrepare(intent)
  if(seq!==sequence||guard!==context()||disposed){await modelV2Api.externalRevoke(bundle.taskId);return}
  ticket=bundle.preparedTicket;taskId.value=bundle.taskId;content.value=bundle.content;expiresAt.value=bundle.expiresAt;frozenRequest.value=intent;store.externalCurrentId=bundle.taskId;store.externalTasks[bundle.taskId]={request:{targets:JSON.parse(JSON.stringify(intent.targets)),scene:intent.scene,allowNew:intent.allowNew,expectedLibrary:intent.expectedLibrary},expiresAt:bundle.expiresAt,activated:false,revoked:false};preparedContext=guard;activated.value=false;revoked.value=false;status.value='任务已准备，尚未激活。复制完整任务成功后才授权提交。'
 }catch(cause){if(seq===sequence&&!disposed)error.value=reason(cause)}finally{if(seq===sequence)preparing.value=false}
}
async function copyTask(){
 if(!taskId.value||!ticket||busy.value||preparing.value||preparedContext!==context())return
 const id=taskId.value,activationTicket=ticket,text=content.value;copying.value=true;error.value=''
 try{
  await navigator.clipboard.writeText(text)
  if(disposed||id!==taskId.value||preparedContext!==context()){status.value='已复制，但任务基准已变化，未激活；请重新准备。';return}
  try{await modelV2Api.externalActivate(id,activationTicket);activated.value=true;if(store.externalTasks[id])store.externalTasks[id]!.activated=true;status.value='完整任务已复制并激活。外部提交只形成待审建议，不会自动保存。'}catch(cause){status.value='已复制但未激活，需重新准备。';ticket='';error.value=reason(cause)}
 }catch(cause){status.value='复制未完成，任务未激活；请重试复制。';error.value=reason(cause);output.value?.focus();output.value?.select()}finally{copying.value=false}
}
async function revoke(){if(!taskId.value||busy.value)return;revoking.value=true;error.value='';try{await modelV2Api.externalRevoke(taskId.value);ticket='';revoked.value=true;activated.value=false;if(store.externalTasks[taskId.value]){store.externalTasks[taskId.value]!.revoked=true;store.externalTasks[taskId.value]!.activated=false}status.value='原任务授权已撤销，已接受的版本保持。'}catch(cause){error.value=reason(cause)}finally{revoking.value=false}}
async function readResult(){
 if(!taskId.value||reading.value||busy.value)return
 const id=taskId.value,sessionKey=key.value;reading.value=true;error.value=''
 try{const view=await modelV2Api.externalRun(id);if(disposed||id!==taskId.value)return;const value=view.result;if(view.dtoVersion!==2||view.taskId!==id||value.dtoVersion!==2||value.runId!==id||view.expectedLibrary!==value.expectedLibrary)throw new Error('结果不属于原外部任务，当前任务保持。');frozenRequest.value={scene:view.scene,targets:JSON.parse(JSON.stringify(view.targets)),allowNew:view.allowNew,expectedLibrary:view.expectedLibrary,requirements:'',history:[],sources:[]};const meta=store.externalTasks[id];store.externalTasks[id]={request:{scene:view.scene,targets:JSON.parse(JSON.stringify(view.targets)),allowNew:view.allowNew,expectedLibrary:view.expectedLibrary},expiresAt:meta?.expiresAt??'',activated:meta?.activated??false,revoked:meta?.revoked??false};store.agentSession(sessionKey).result=value;status.value=value.phase==='accepted'?'原任务已接受保存。':value.report?'检查报告已返回，没有保存入口。':value.review?'原任务结果已读回，请完整审阅后确认。':'原任务尚无可审阅提交结果。'}catch(cause){error.value=reason(cause)}finally{reading.value=false}
}
const scopeMatches=computed(()=>{const r=frozenRequest.value,review=result.value?.review;if(!r||!review)return false;return r.targets.every(target=>review.proposals.some(p=>p.proposal.target&&JSON.stringify(target)===JSON.stringify(p.proposal.target)))&&review.proposals.every(p=>p.proposal.target?r.targets.some(t=>JSON.stringify(t)===JSON.stringify(p.proposal.target)):r.allowNew)})
const canAccept=computed(()=>!!frozenRequest.value&&frozenRequest.value.scene!=='check_only'&&result.value?.runId===taskId.value&&result.value.phase==='review'&&!result.value.report&&result.value.review?.valid&&scopeMatches.value&&!busy.value&&(result.value.expectedLibrary===props.snapshot.libraryDigest||!!session.value?.acceptRequest))
function accept(){if(!canAccept.value||confirmation.value||!result.value)return;const id=taskId.value,digest=result.value.responseDigest;confirmation.value=true;useModalStore().open({title:'接受外部任务完整建议',body:'按原任务完整精确目标另存建议版本。当前编辑、默认版本与已有采用保持；检查报告不会保存。',confirmLabel:'确认接受并保存',onConfirm:async()=>{try{if(taskId.value!==id||result.value?.responseDigest!==digest){error.value='原审阅结果已变化，请重新核对。';return}await store.acceptAgent(key.value,frozenRequest.value?.targets[0]?.templateId??null)}finally{confirmation.value=false}},onCancel:()=>{confirmation.value=false}})}
function close(){if(!busy.value)emit('close')}
</script>
<template>
 <UiDialog :open="open" title="外部任务" wide :busy="busy" @close="close">
  <p>外部 CLI 按完整任务中的提交入口回交结果；回交只形成待审内容，接受保存仍需在本机确认。</p>
  <div class="model-columns"><UiSelectField v-model="scope" label="任务范围" :disabled="busy||preparing" :options="[{value:'current',label:'当前模版',disabled:!draft?.baseRevision},{value:'selected',label:'勾选模版',disabled:!selected?.length},{value:'all',label:'全部模版'},{value:'new',label:'仅新建'}]" /><UiSelectField :model-value="scene" label="任务场景" :options="scenes" :disabled="busy||preparing" @update:model-value="scene=$event as AgentSceneV2" /></div>
  <label v-if="scope!=='new'&&scene!=='check_only'" class="model-inline"><input v-model="allowNew" type="checkbox" :disabled="busy||preparing" />同时允许新增模版</label><label>补充要求<textarea v-model="requirements" class="input" aria-label="外部任务补充要求" :disabled="busy||preparing" rows="3" /></label>
  <UiButton :disabled="busy||preparing||reading" :loading="preparing" @click="prepare">{{taskId?'重新准备任务':'准备完整任务'}}</UiButton><label>完整任务内容<textarea ref="output" readonly :value="content" class="input mono" aria-label="完整外部任务内容" rows="10" /></label>
  <p v-if="taskId">原任务 {{taskId}} · 到期 {{expiresAt}}</p><p v-if="status" role="status">{{status}}</p><p v-if="error" role="alert" class="model-error">{{error}}</p><p v-if="session?.status" role="status">{{session.status}}</p><p v-if="session?.error" role="alert" class="model-error">{{session.error}}</p>
  <template v-if="result?.report"><h3>检查报告</h3><p>{{result.report.summary}}</p><p v-for="(issue,index) in result.report.issues" :key="index">{{issue.code}} · {{issue.message}}</p></template>
  <template v-if="result?.review"><h3>待审阅建议 · {{result.review.proposals.length}} 项</h3><p v-if="!scopeMatches" role="alert" class="model-error">结果与原任务目标不符，未允许接受。</p><p v-if="result.expectedLibrary!==snapshot.libraryDigest" role="alert" class="model-error">已保存库已变化；建议保持，接受前需核对原任务。</p><article v-for="item in result.review.proposals" :key="item.index"><h4>{{item.proposal.label}} · {{item.proposal.target?.revisionId??'明确新建目标'}}</h4><details v-for="(difference,index) in item.differences" :key="index"><summary>{{difference.fieldPath}}</summary><pre>{{JSON.stringify(difference.before,null,2)}}</pre><pre>{{JSON.stringify(difference.after,null,2)}}</pre></details><details><summary>完整 v2 条件内容</summary><pre>{{JSON.stringify(item.proposal.content,null,2)}}</pre></details><p v-for="(issue,index) in item.issues" :key="index" class="model-error">{{issue.code}} · {{issue.message}}</p></article><UiButton v-if="frozenRequest?.scene!=='check_only'&&result.phase==='review'" variant="primary" :disabled="!canAccept||confirmation" @click="accept">{{session?.acceptRequest?'核对原接受结果':'接受外部完整建议'}}</UiButton><p v-if="result.phase==='accepted'" role="status">原任务建议已接受保存，当前编辑保持。</p></template>
  <template #actions="{close:requestClose}"><UiButton :disabled="busy" @click="requestClose">关闭</UiButton><UiButton v-if="taskId" :disabled="busy||reading" @click="readResult">读取原任务结果</UiButton><UiButton v-if="taskId&&!revoked" :disabled="busy" @click="revoke">撤销原任务授权</UiButton><UiButton variant="primary" :disabled="!ticket||busy||preparing||preparedContext!==context()" :loading="copying" @click="copyTask">{{activated?'再次复制完整任务':'复制完整任务并激活'}}</UiButton></template>
 </UiDialog>
</template>
<style scoped>
label{display:grid;gap:var(--space-2);margin-block:var(--space-3)}textarea{width:100%;box-sizing:border-box}pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:320px;overflow:auto}.model-error{color:var(--red)}
</style>
