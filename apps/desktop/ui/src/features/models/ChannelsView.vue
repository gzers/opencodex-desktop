<script setup lang="ts">
import {escapeHtml} from '@/lib/html'
import { computed, onBeforeUnmount, onMounted, ref, watch, toRef } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ChannelEditor from './ChannelEditor.vue'
import ChannelBatchDialog from './ChannelBatchDialog.vue'
import ChannelNodeSelect from './ChannelNodeSelect.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import NewChannelDialog from './NewChannelDialog.vue'
import ClientLoadStatus from './ClientLoadStatus.vue'
import SelectorPreviewDialog from './SelectorPreviewDialog.vue'
import { mergeUpstream } from './upstream'
import { inlineChanges, setModelActive } from './saveReview'
import {copyChannel,protectedChannel} from './channelActions'
import {reorderSlots} from './ordering'
import {createModelRowsState} from './modelRows'
import { useModelsStore } from './store'
import { errorText, savePlanErrorText, modelApi, type ChannelPolicy, type DraftProjection, type ModelClientState, type ModelPolicy, type SaveBlockedReason, type SavePlan } from './api'
import { useModalStore } from '@/app/modal/store'
const props=withDefaults(defineProps<{actionsTarget?:string;active?:boolean}>(),{active:true})
const models = useModelsStore(); const modal = useModalStore()
const search = toRef(models.view,'channelSearch'); const editorId = toRef(models.view,'channelEditor'); const error = ref('')
const editor = computed(() => models.channelDraft?.channels.find(c => c.id === editorId.value))
watch(editorId,id=>{if(id){models.view.channelFilters[id]??={search:'',selectedOnly:false,family:'all'};models.modelRows[id]??=createModelRowsState();models.view.navigation['channel:'+id]??={collapsed:false,activeId:'channel-config'}}},{immediate:true,flush:'sync'})
const previewOpen = ref(false);const previewProjection = ref<DraftProjection|null>(null);const previewBusy = ref(false);const previewError = ref('')
let previewSequence = 0
const savePlan=ref<SavePlan|null>(null);const saveOpen=ref(false);const saving=ref(false)
const saveScopeOpen=ref(false),saveScopeIds=ref<string[]>([]),saveShared=ref(false),saveScopeError=ref('');const saveScopeTargets=ref<{id:string;label:string;kind:string}[]>([]);const saveSharedAvailable=ref(false)
let saveScopeBaseline:{policy:string;library:string;saved:string;workspace:string}|undefined
const changedTargets=computed(()=>{
 const saved=models.snapshot?.policy.channels??[],draft=models.channelDraft?.channels??[]
 const ids=[...new Set([...saved,...draft].map(c=>c.id))]
 return ids.flatMap(id=>{const before=saved.find(c=>c.id===id),after=draft.find(c=>c.id===id);return JSON.stringify(before)===JSON.stringify(after)?[]:[{id,label:after?.label??before?.label??id,kind:!before?'新增':!after?'删除':'修改'}]}).concat((models.channelDraft?.pendingRemovals??[]).filter(r=>!ids.includes(r.channelId)).map(r=>({id:r.channelId,label:r.providerId,kind:'删除'})))
})
// 共享项只含全局模版采用与渠道顺序；“默认渠道”不是本产品的概念，不参与差异与保存范围。
const sharedChanged=computed(()=>{const saved=models.snapshot?.policy,draft=models.channelDraft;if(!saved||!draft)return false;return JSON.stringify([saved.templateRefs,saved.channels.map(c=>c.id)])!==JSON.stringify([draft.templateRefs,draft.channels.map(c=>c.id)])})
async function adjustSaveScope(){
 if(!models.channelDraft||!models.snapshot||models.busy||saving.value)return
 if(models.pendingRows.length){error.value='请先处理未确认型号编辑，再调整提交范围';return}
 models.busy=true;saveScopeError.value=''
 try{await models.persistDraft();if(models.channelError)throw {message:models.channelError};
 saveScopeBaseline={policy:JSON.stringify(models.channelDraft),library:models.snapshot.libraryDigest,saved:models.channelBaseline,workspace:models.snapshot.workspaceDigest};saveScopeTargets.value=changedTargets.value.map(c=>({...c}));saveScopeIds.value=saveScopeTargets.value.map(c=>c.id);saveSharedAvailable.value=sharedChanged.value;saveShared.value=saveSharedAvailable.value;saveScopeOpen.value=true;saveOpen.value=false
 }catch(cause){error.value=errorText(cause)}finally{models.busy=false}
}
async function prepareSelectedScope(){
 if(!saveScopeBaseline||!models.channelDraft||models.busy)return
 models.busy=true;saveScopeError.value=''
 try{if(saveScopeBaseline.policy!==JSON.stringify(models.channelDraft)||saveScopeBaseline.library!==models.snapshot?.libraryDigest||saveScopeBaseline.saved!==models.channelBaseline||saveScopeBaseline.workspace!==models.snapshot.workspaceDigest)throw {message:'本次保存范围已过期，请关闭后重新核对'}
 savePlan.value=await modelApi.preparePolicyScoped(JSON.parse(saveScopeBaseline.policy),saveScopeBaseline.library,saveScopeBaseline.saved,saveScopeBaseline.workspace,{channelIds:[...saveScopeIds.value],includeShared:saveShared.value});saveScopeOpen.value=false;saveOpen.value=true;error.value=''
 }catch(cause){saveScopeError.value=errorText(cause)}finally{models.busy=false}
}
const clientState=ref<ModelClientState|null>(null)
const clientLabel=computed(()=>clientState.value?.status==='user_confirmed'?'已写入 · 用户已确认':clientState.value?.status==='pending_verification'?'已写入 · 待客户端验证':clientState.value?.status==='runtime_changed'?'运行配置变化 · 待核对':clientState.value?.status==='not_required'?'已写入 · 无需新增重启':'已写入 · 待重启 Codex')
const policyStatus=computed(()=>saving.value?{label:'正在核对',tone:'warning'}:savePlan.value&&!['prepared','committed','compensated'].includes(savePlan.value.phase)?{label:'保存待核对',tone:'danger'}:models.pendingRows.length?{label:models.pendingRows.length+' 项未确认',tone:'warning'}:models.channelDirty?{label:'未保存修改',tone:'warning'}:savePlan.value?.phase==='committed'?{label:clientLabel.value,tone:'success'}:savePlan.value?.phase==='compensated'?{label:'已撤回本次写入',tone:'neutral'}:{label:'未修改',tone:'neutral'})
const detailProjection=ref<DraftProjection|null>(null),detailProjectionError=ref(''),detailProjectionBusy=ref(false)
let detailSequence=0,detailTimer:ReturnType<typeof setTimeout>|undefined
watch(()=>[editorId.value,JSON.stringify(models.channelDraft),models.snapshot?.libraryDigest],(_next,previous)=>{
 const sequence=++detailSequence;if(detailTimer)clearTimeout(detailTimer);if(!previous||previous[0]!==editorId.value)detailProjection.value=null;detailProjectionError.value='';detailProjectionBusy.value=false
 if(!editorId.value||!models.channelDraft)return
 const policy=JSON.parse(JSON.stringify(models.channelDraft));detailProjectionBusy.value=true
 detailTimer=setTimeout(async()=>{try{const result=await modelApi.project(policy);if(!disposed&&sequence===detailSequence)detailProjection.value=result}catch(cause){if(!disposed&&sequence===detailSequence)detailProjectionError.value=errorText(cause)}finally{if(sequence===detailSequence)detailProjectionBusy.value=false}},200)
},{immediate:true})
let disposed=false
onBeforeUnmount(()=>{disposed=true;detailSequence++;if(detailTimer)clearTimeout(detailTimer)})
onMounted(async()=>{try {const plan=await modelApi.latestPolicy();if(!disposed&&!models.policyNotificationTarget)savePlan.value=plan}catch(cause){if(!disposed)error.value=errorText(cause)}})
watch(()=>models.policyNotificationTarget,async target=>{
 if(!target)return
 const requested=target
 saving.value=true;error.value=''
 try{const plan=await modelApi.policyResult(target.planId);if(!disposed&&models.policyNotificationTarget===requested){savePlan.value=plan;saveOpen.value=true}}
 catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)saving.value=false}
},{immediate:true})
/** 保存是唯一提交入口，也是预览入口（UI 规范 §38.1）。 */
const saveReviewBaseline=ref<{saved:string;library:string}|null>(null),saveConfirming=ref(false)
async function prepareSave(){
  if(!models.channelDraft||!models.snapshot||models.busy||models.staging||saving.value)return
  const sequence=++previewSequence
  previewBusy.value=true;previewError.value='';error.value=''
  try{const projection=await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft)))
    if(sequence!==previewSequence)return
    previewProjection.value=projection;saveReviewBaseline.value={saved:models.channelBaseline,library:models.snapshot.libraryDigest};previewOpen.value=true
  }catch(cause){if(sequence===previewSequence){previewProjection.value=null;previewError.value=errorText(cause);previewOpen.value=true}}
  finally{if(sequence===previewSequence)previewBusy.value=false}
}
async function refreshSavePreview(){
  if(!models.channelDraft)return
  const sequence=++previewSequence;previewBusy.value=true
  try{const projection=await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft)))
    if(sequence===previewSequence){previewProjection.value=projection;previewError.value=''}
  }catch(cause){if(sequence===previewSequence)previewError.value=errorText(cause)}
  finally{if(sequence===previewSequence)previewBusy.value=false}
  void models.persistDraft()
}
function togglePreviewChannel(id:string){
  const channel=models.channelDraft?.channels.find(c=>c.id===id)
  if(!channel||models.busy||saving.value||protectedChannel(channel))return
  channel.connection.enabled=!channel.connection.enabled
  void refreshSavePreview()
}
function togglePreviewModel(channelId:string,modelId:string){
  const channel=models.channelDraft?.channels.find(c=>c.id===channelId)
  if(!channel||models.busy||saving.value||protectedChannel(channel))return
  const active=channel.selection.mode==='all'||channel.selection.mode==='whitelist'&&channel.selection.models.includes(modelId)
  setModelActive(channel,[modelId],!active)
  void refreshSavePreview()
}
const saveReviewChanges=computed(()=>{
  const saved=models.snapshot?.policy,draft=models.channelDraft
  return saved&&draft?inlineChanges(saved,draft):[]
})
/** 门禁顺序与原型一致：待确认编辑 → 超预算 → 校验失败 → 无修改 → 基准变化。 */
function saveGateReason():SaveBlockedReason|''{
  if(models.pendingRows.length)return 'staged_edit'
  if((previewProjection.value?.knownActiveModels??0)>100)return 'over_budget'
  if(previewError.value)return 'validation'
  if(!models.channelDirty)return 'no_changes'
  const baseline=saveReviewBaseline.value
  if(baseline&&(baseline.saved!==models.channelBaseline||baseline.library!==models.snapshot?.libraryDigest))return 'stale'
  return ''
}
/** 确认即关闭：无论写入还是被拦下，弹窗都先消失（§38.4）。 */
async function confirmSave(){
  if(saveConfirming.value||saving.value||!models.channelDraft||!models.snapshot)return
  const frozen=JSON.parse(JSON.stringify(models.channelDraft)) as ModelPolicy
  const library=models.snapshot.libraryDigest,expected=models.channelBaseline,workspace=models.snapshot.workspaceDigest
  const reason=saveGateReason()
  saveConfirming.value=true;previewOpen.value=false;previewProjection.value=null;saveReviewBaseline.value=null
  if(reason){
    error.value=''
    try{await modelApi.policySaveBlocked(reason)}catch{/* 通知投递失败不回退为界面错误块，草稿保持可重试 */}
    saveConfirming.value=false
    return
  }
  saving.value=true;models.busy=true;error.value=''
  let stage:'prepare'|'apply'='prepare'
  try{
    await models.persistDraft();if(models.channelError)throw new Error(models.channelError)
    savePlan.value=await modelApi.preparePolicy(frozen,library,expected,workspace)
    stage='apply'
    savePlan.value=await modelApi.applyPolicy(savePlan.value.id)
    if(savePlan.value.phase==='committed')models.acceptTransfer(await modelApi.load())
  }catch(cause){
    error.value=errorText(cause)
    if(stage==='prepare'){
      const code=String((cause as {code?:string})?.code??'')
      const reason:SaveBlockedReason=code.includes('stale')?'stale':'validation'
      try{await modelApi.policySaveBlocked(reason)}catch{/* 草稿保留 */}
    }
    if(savePlan.value){try{savePlan.value=await modelApi.policyResult(savePlan.value.id)}catch{/* 保留原计划标识供核对 */}}
  }finally{saving.value=false;models.busy=false;saveConfirming.value=false}
}

const compensationConfirm=ref(false)
async function querySave(){
 if(!savePlan.value)return
 saving.value=true;error.value=''
 try{savePlan.value=await modelApi.policyResult(savePlan.value.id);if(savePlan.value.phase==='committed')models.acceptTransfer(await modelApi.load())}
 catch(cause){error.value=errorText(cause)}finally{saving.value=false}
}
async function compensate(){
 if(!savePlan.value)return
 if(!compensationConfirm.value){compensationConfirm.value=true;return}
 saving.value=true;models.busy=true;error.value=''
 try{savePlan.value=await modelApi.compensatePolicy(savePlan.value.id);compensationConfirm.value=false}
 catch(cause){error.value=errorText(cause)}finally{saving.value=false;models.busy=false}
}

const upstreamBusy = ref(false)
const upstreamVersion = ref('')
async function readUpstream() {
  if (upstreamBusy.value) return
  upstreamBusy.value = true; error.value = ''
  try {
    const snapshot = await modelApi.upstream()
    const channels = mergeUpstream(snapshot, models.channelDraft?.channels ?? [])
    if (!models.channelDraft) models.restoreChannels()
    if (models.channelDraft) models.channelDraft.channels = channels
    upstreamVersion.value = snapshot.version
  } catch (cause) { error.value = errorText(cause) } finally { upstreamBusy.value = false }
}
const selectedIds=toRef(models.view,'selectedChannels')
const batchChannels=ref<{mode:'family'|'adopt';ids:string[]}>()
const copySource=ref<ChannelPolicy|null>(null);const copyName=ref('');const reuseCopyKey=ref(false)
const selectable=computed(()=>channels.value.filter(c=>!protectedChannel(c)))
const selected=computed(()=>(models.channelDraft?.channels??[]).filter(c=>selectedIds.value.includes(c.id)&&!protectedChannel(c)))
const allSelected=computed(()=>selectable.value.length>0&&selectable.value.every(c=>selectedIds.value.includes(c.id)))
const someSelected=computed(()=>selectable.value.some(c=>selectedIds.value.includes(c.id))&&!allSelected.value)
function selectAll(){const visible=new Set(selectable.value.map(c=>c.id));selectedIds.value=allSelected.value?selectedIds.value.filter(id=>!visible.has(id)):[...new Set([...selectedIds.value,...visible])]}
function selectOne(id:string){selectedIds.value=selectedIds.value.includes(id)?selectedIds.value.filter(value=>value!==id):[...selectedIds.value,id]}
function beginCopy(channel:ChannelPolicy){if(protectedChannel(channel))return;copySource.value=channel;copyName.value=channel.label+' 副本';reuseCopyKey.value=false}
function confirmCopy(){if(!copySource.value)return;try{const current=models.channelDraft?.channels.find(c=>c.id===copySource.value!.id);if(!current)throw {message:'来源渠道已变化'};const copy=copyChannel(JSON.parse(JSON.stringify(current)),copyName.value,reuseCopyKey.value);models.channelDraft?.channels.push(copy);copySource.value=null;editorId.value=copy.id}catch(cause){error.value=errorText(cause)}}
function enableSelected(enabled:boolean){for(const channel of selected.value)channel.connection.enabled=enabled}
function deleteChannels(values:ChannelPolicy[]){
 const ids=values.filter(c=>!protectedChannel(c)).map(c=>c.id);if(!ids.length)return
 const baseline=JSON.stringify(models.channelDraft)
 modal.open({title:'删除渠道',body:'将 '+values.map(c=>escapeHtml(c.label)).join('、')+' 从草稿移除（包含这些渠道未确认的型号输入）。保存时一并移除对应官方运行渠道及其协议分组；此前可用还原修改撤回。',confirmLabel:'从草稿删除',onConfirm:()=>{
  if(JSON.stringify(models.channelDraft)!==baseline){error.value='删除确认已过期，请重新核对当前草稿';return}
  if(!models.channelDraft)return
  for(const channel of models.channelDraft.channels.filter(c=>ids.includes(c.id))){if(channel.connection.sourceReadId&&!models.snapshot?.policy.channels.some(c=>c.id===channel.id)){
    const pending=models.channelDraft.pendingRemovals??[];models.channelDraft.pendingRemovals=[...pending.filter(r=>r.channelId!==channel.id),{readId:channel.connection.sourceReadId,channelId:channel.id,providerId:channel.connection.providerId,credentialRef:channel.connection.credentialRef}]
  }}
  models.channelDraft.channels=models.channelDraft.channels.filter(c=>!ids.includes(c.id));// 旧草稿可能残留 defaultChannelId；删除渠道时一并清掉，避免带失效引用进入保存
  models.channelDraft.defaultChannelId=null
  for(const id of ids)delete models.modelRows[id];selectedIds.value=selectedIds.value.filter(id=>!ids.includes(id));if(ids.includes(editorId.value))editorId.value=''
 }})
}
const channels = computed(() => models.channelDraft?.channels.filter(c => c.label.toLowerCase().includes(search.value.toLowerCase())) ?? [])
const newChannelOpen=ref(false)
function create(){if(!models.busy&&!saving.value)newChannelOpen.value=true}
function restore() {
  const baseline = JSON.stringify(models.channelDraft); const rows=JSON.stringify(models.modelRows); const policy = models.snapshot?.policyDigest
  modal.open({ title: '还原修改', body: '将全部渠道恢复到最近成功保存的配置？', confirmLabel: '还原修改', onConfirm: () => {
    if (rows!==JSON.stringify(models.modelRows)||baseline !== JSON.stringify(models.channelDraft) || policy !== models.snapshot?.policyDigest) { error.value = '还原确认已过期，请重新比较当前修改'; return }
    models.restoreChannels(); editorId.value = ''; error.value = ''
  } })
}
const draggedChannel=ref('')
const reorderAllowed=computed(()=>!search.value.trim()&&!models.busy&&!saving.value)
function reorderChannel(id:string,target:string){
 if(!reorderAllowed.value||!models.channelDraft)return
 const values=models.channelDraft.channels;const order=reorderSlots(values.map(c=>c.id),values.map(c=>c.id),id,target)
 models.channelDraft.channels=order.map(id=>values.find(c=>c.id===id)!)
}
function move(id:string,direction:number){const values=models.channelDraft?.channels??[];const index=values.findIndex(c=>c.id===id);const target=values[index+direction];if(target)reorderChannel(id,target.id)}
function dragChannel(event:DragEvent,id:string){if(!reorderAllowed.value){event.preventDefault();return}draggedChannel.value=id;event.dataTransfer?.setData('text/plain',id);if(event.dataTransfer)event.dataTransfer.effectAllowed='move'}
function dropChannel(target:string){if(draggedChannel.value)reorderChannel(draggedChannel.value,target);draggedChannel.value=''}
</script>
<template>
  <div class="channels-view">
    <div v-if="!editor" class="model-toolbar"><input class="input" v-model="search" type="search" placeholder="搜索渠道" aria-label="搜索渠道" /><div class="model-inline"><UiButton :loading="upstreamBusy" :disabled="models.busy||saving" @click="readUpstream">读取现有渠道</UiButton><UiButton :disabled="models.busy||saving" @click="create">新增渠道</UiButton></div></div>
    <Teleport defer :to="actionsTarget??'body'" :disabled="!actionsTarget"><div v-if="props.active!==false&&!editor" class="model-inline model-policy-actions"><span class="model-policy-state" :data-tone="policyStatus.tone" role="status">● {{ policyStatus.label }}</span><UiButton :disabled="(!models.channelDirty&&!models.pendingRows.length)||models.busy||saving" @click="restore">还原修改</UiButton><UiButton :variant="models.channelDirty?'primary':'default'" :disabled="models.busy||saving||saveConfirming" :loading="models.busy" title="保存 · 先核对选择器预览，确认后写入" @click="prepareSave">保存</UiButton></div></Teleport>
    <p v-if="upstreamVersion" class="model-muted">来源：{{ upstreamVersion }} · 已读取到草稿</p>
    <p v-if="search.trim()" class="model-muted">清除搜索后可调整渠道顺序</p>
    <p v-if="models.pendingRows.length" role="status">{{ models.pendingRows.length }} 项型号编辑未确认；全局保存暂不可用。<UiButton v-for="target in [...new Set(models.pendingRows.map(r=>r.channel))]" :key="target" size="sm" variant="ghost" @click="editorId=target">查看 {{ models.channelDraft?.channels.find(c=>c.id===target)?.label??target }}</UiButton></p>
    <ClientLoadStatus @change="clientState=$event" :refresh-key="(models.snapshot?.policyDigest??'')+':'+(savePlan?.phase??'')" />
    <div v-if="savePlan && !['prepared','compensated'].includes(savePlan.phase)" class="model-inline"><span class="tag">{{ savePlan.phase === 'committed' ? '最近保存已写入' : '上次保存尚未完成核对' }}</span><UiButton size="sm" @click="saveOpen=true">查看原保存结果</UiButton></div>
    <p v-if="error || models.channelError" class="model-error" role="alert">{{ error || models.channelError }}</p><UiButton v-if="error&&models.channelDirty&&!models.channelDraft?.recoveryId" size="sm" variant="ghost" :disabled="models.busy||saving" @click="adjustSaveScope">调整本次保存范围</UiButton><p v-if="models.staging" class="model-muted" role="status">正在暂存渠道草稿…</p>
    <div v-if="selected.length&&!editor" class="model-inline"><span>已选 {{ selected.length }} 个渠道<span v-if="selected.some(c=>!channels.some(row=>row.id===c.id))">（含搜索外 {{ selected.filter(c=>!channels.some(row=>row.id===c.id)).length }} 个）</span></span><UiButton size="sm" :disabled="models.busy||saving" @click="enableSelected(true)">启用</UiButton><UiButton size="sm" :disabled="models.busy||saving" @click="enableSelected(false)">停用</UiButton><UiButton size="sm" variant="danger" :disabled="models.busy||saving" @click="deleteChannels(selected)">删除所选渠道</UiButton><UiButton size="sm" :disabled="models.busy||saving||!!models.pendingRows.length" @click="batchChannels={mode:'family',ids:selected.map(c=>c.id)}">按族批量配置</UiButton><UiButton size="sm" :disabled="models.busy||saving||!!models.pendingRows.length" @click="batchChannels={mode:'adopt',ids:selected.map(c=>c.id)}">套用模版</UiButton><UiButton size="sm" variant="ghost" @click="selectedIds=[]">清除选择</UiButton></div>
    <ChannelEditor v-if="editor" :key="editor.id" :navigation="models.view.navigation['channel:'+editor.id]" :channels="models.channelDraft?.channels" @switch="editorId=$event" @save="prepareSave" @restore="restore" :dirty="models.channelDirty||!!models.pendingRows.length" :save-status="policyStatus.label" :save-disabled="models.busy||saving||saveConfirming" :active-count="detailProjection?.knownActiveModels" :view-state="models.view.channelFilters[editor.id]" :row-state="models.modelRows[editor.id]" :policy="models.channelDraft??undefined" :existing="models.snapshot?.policy.channels.some(c=>c.id===editorId)" :channel="editor" :global-refs="models.channelDraft?.templateRefs??[]" :projection="detailProjection?.rows.filter(row=>row.channelId===editorId)??[]" :projection-busy="detailProjectionBusy" :projection-error="detailProjectionError" :templates="models.snapshot?.library.templates ?? []" :busy="models.busy||protectedChannel(editor)" @close="editorId = ''" />
    <template v-else><div class="model-table-scroll"><table class="model-table ui-table ui-table-fixed channel-table"><colgroup><col class="ui-selection-col"/><col class="channel-name-col"/><col class="channel-node-col"/><col class="channel-count-col"/><col class="channel-enable-col"/><col class="channel-actions-col"/></colgroup><thead><tr><th class="ui-selection-cell"><input type="checkbox" aria-label="选择当前结果中的可编辑渠道" :checked="allSelected" :indeterminate="someSelected" @change="selectAll" /></th><th>渠道</th><th>服务节点</th><th>模型</th><th>启用</th><th>操作</th></tr></thead><tbody><tr v-for="(c,channelIndex) in channels" :key="c.id" :draggable="reorderAllowed" @dragstart="dragChannel($event,c.id)" @dragend="draggedChannel=''" @dragover.prevent @drop.prevent="dropChannel(c.id)"><td class="ui-selection-cell"><input type="checkbox" :aria-label="'选择渠道 '+c.label" :checked="selectedIds.includes(c.id)" :disabled="protectedChannel(c)||models.busy||saving" @change="selectOne(c.id)" /></td><td><strong>{{ c.label || '未命名渠道' }}</strong><small>{{ c.connection.nodes.length }} 节点 · {{ c.connection.enabled ? '启用' : '停用' }}</small></td><td><ChannelNodeSelect :channel="c" :disabled="models.busy||saving" :readonly="protectedChannel(c)" /></td><td class="ui-nowrap">{{ c.selection.mode === 'all' ? c.discovered.length : c.selection.mode === 'whitelist' ? c.selection.models.length : 0 }} / {{ c.discovered.length }}</td><td><span v-if="protectedChannel(c)">官方管理</span><div v-else class="channel-enable-cell"><UiSwitch :model-value="c.connection.enabled" :label="c.label+'渠道启用状态'" :disabled="models.busy||saving" @update:model-value="c.connection.enabled=$event" /><span>{{ c.connection.enabled?'启用':'停用' }}</span></div></td><td><div class="channel-row-actions" :class="{'is-readonly':protectedChannel(c)}"><UiButton size="sm" :data-model-focus="'channel:'+c.id" @click="editorId = c.id">{{ protectedChannel(c)?'查看':'编辑' }}</UiButton><UiButton size="sm" :disabled="protectedChannel(c)||models.busy||saving" @click="beginCopy(c)">复制</UiButton><UiButton size="sm" variant="danger" :disabled="protectedChannel(c)||models.busy||saving" @click="deleteChannels([c])">删除</UiButton><span class="channel-order" role="group" :aria-label="c.label+'排序'"><button type="button" class="channel-order-button" :disabled="!reorderAllowed||channelIndex===0" :aria-label="c.label+'上移'" :title="c.label+'上移'" @click="move(c.id,-1)">↑</button><button type="button" class="channel-order-button" :disabled="!reorderAllowed||channelIndex===channels.length-1" :aria-label="c.label+'下移'" :title="c.label+'下移'" @click="move(c.id,1)">↓</button></span></div></td></tr></tbody></table></div><p v-if="!channels.length" class="model-empty">暂无渠道</p></template>
    <ChannelBatchDialog v-if="batchChannels" :mode="batchChannels.mode" :targets="batchChannels.ids" @close="batchChannels=undefined" />
    <NewChannelDialog v-if="newChannelOpen" @close="newChannelOpen=false" @added="editorId=$event" />
    <UiDialog :open="saveScopeOpen" title="调整本次保存范围" :busy="models.busy" @close="saveScopeOpen=false">
      <p>未勾选渠道留待稍后，仍保留在草稿。本次会完整核对共享影响和全局目录预算。</p><p v-if="saveScopeError" class="model-error" role="alert">{{ saveScopeError }}</p>
      <label v-for="channel in saveScopeTargets" :key="channel.id" class="model-inline"><input v-model="saveScopeIds" type="checkbox" :value="channel.id" :disabled="models.busy" />{{ channel.label }} · {{ channel.kind }}<span v-if="!saveScopeIds.includes(channel.id)"> · 留待稍后</span></label>
      <label v-if="saveSharedAvailable" class="model-inline"><input v-model="saveShared" type="checkbox" :disabled="models.busy" />包含全局模版采用与渠道顺序（影响全部渠道）</label>
      <template #actions><UiButton @click="saveScopeOpen=false">取消</UiButton><UiButton variant="primary" :loading="models.busy" :disabled="!saveScopeIds.length&&!saveShared" @click="prepareSelectedScope">比较本次范围</UiButton></template>
    </UiDialog>
    <UiDialog :open="saveOpen" title="保存结果" :busy="saving" @close="saveOpen = false"><template v-if="savePlan"><p v-if="savePlan.historySource">本次保存来自成功历史恢复，按原始官方字段、目录顺序和受保护授权执行。</p><div v-if="savePlan.removedProviders?.length"><h3>将移除的运行渠道</h3><p>{{ savePlan.removedProviders.join('、') }}</p></div><p>完整目录条目：{{ savePlan.currentCatalogCount }} → {{ savePlan.catalogCount }} / 100（当前兼容预算）</p><p>范围：{{ savePlan.changedChannels.join('、') }}</p><div v-if="savePlan.deferredChannels?.length"><h3>留待稍后</h3><p>{{ savePlan.deferredChannels.join('、') }}；本次不提交，草稿保留。</p></div><div v-if="savePlan.protocolGroups?.length"><h3>协议分组</h3><p v-for="group in savePlan.protocolGroups" :key="group" class="mono">{{ group }}</p></div><div v-if="savePlan.routeChanges?.length"><h3>模型运行标识或快捷入口变更</h3><p>以下分组或别名变更会影响旧选择入口，已有会话需按对应提示重新选择型号。</p><p v-for="change in savePlan.routeChanges" :key="change" class="mono">{{ change }}</p></div><details v-if="savePlan.runtimeChanges?.length"><summary>核对原始官方字段差异（{{ savePlan.runtimeChanges.length }} 项）</summary><article v-for="change in savePlan.runtimeChanges" :key="change.field"><strong>{{ change.field }}</strong><pre>当前：{{ JSON.stringify(change.before,null,2) }}</pre><pre>恢复后：{{ JSON.stringify(change.after,null,2) }}</pre></article></details><p v-if="savePlan.phase === 'prepared'">本次已准备但尚未写入；草稿保留，可重新保存。</p><p v-else>执行阶段：{{ ({applying:'执行中',providers_written:'渠道已写入',visibility_written:'可见性已写入',order_written:'顺序已写入',runtime_verified:'运行读回已核对',committed:'保存完成，待客户端核对'}[savePlan.phase] ?? '待核对') }}</p><p v-if="compensationConfirm">将核对本次最后读回的运行态，只撤回本次写入；若发现后来修改会拒绝补偿。确认继续？</p><p v-if="savePlan.error || error" class="model-error" role="alert">{{ error || savePlanErrorText(savePlan.error) }}</p></template><template #actions><UiButton v-if="savePlan?.phase==='prepared'&&!savePlan.historySource" :disabled="models.busy||saving" @click="adjustSaveScope">调整范围</UiButton><UiButton :disabled="saving" @click="saveOpen = false">关闭</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" :disabled="saving" @click="querySave">核对原计划</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" variant="danger" :loading="saving" @click="compensate">{{ compensationConfirm ? '确认撤回本次写入' : '撤回本次未完成写入' }}</UiButton></template></UiDialog>
    <UiDialog :open="!!copySource" title="复制渠道" :busy="models.busy" @close="copySource=null"><label>副本名称<input v-model="copyName" class="input" maxlength="256" /></label><label v-if="copySource?.connection.credentialRef"><input v-model="reuseCopyKey" type="checkbox" />副本复用此本机渠道Key</label><p>副本采用独立渠道和节点身份，保持停用。模版匹配范围需在预览中核对。</p><template #actions><UiButton @click="copySource=null">取消</UiButton><UiButton variant="primary" :disabled="!copyName.trim()" @click="confirmCopy">建立副本草稿</UiButton></template></UiDialog>
    <SelectorPreviewDialog :open="previewOpen" :projection="previewProjection" :saved="models.snapshot?.policy??null" :other-changes="saveReviewChanges" :channels="(models.channelDraft?.channels ?? []).map(c=>({id:c.id,label:c.label||c.id,enabled:c.connection.enabled,protected:protectedChannel(c)}))" :busy="previewBusy" :saving="saveConfirming" :error="previewError||undefined" @toggle-channel="togglePreviewChannel" @toggle-model="togglePreviewModel" @confirm="confirmSave" @close="previewOpen = false" />
  </div>
</template>

<style scoped>
/* 列宽对齐原型 .mp-provider-columns：36px 选择 + 弹性名称 + 190/76/76/200 固定列。 */
.channel-table{min-width:800px}.channel-name-col{width:auto}.channel-node-col{width:190px}.channel-count-col{width:76px}.channel-enable-col{width:76px}.channel-actions-col{width:200px}
.channel-table strong{overflow-wrap:anywhere}.channel-table td:nth-child(5) .model-inline{flex-wrap:nowrap}
/* WP-40：编辑 / 复制 / 删除 + 竖排 ↑↓ 的迷你按钮对；整组贴右且不超出 200px 列。 */
.channel-row-actions{display:flex;align-items:center;justify-content:flex-end;gap:var(--space-1);flex-wrap:nowrap;min-width:0;max-width:100%}
.channel-row-actions .btn{white-space:nowrap}
/* 原型 .mp-mini-pair：14px 高的迷你箭头，两个叠起仍小于行内按钮，不撑高行。 */
.channel-order{display:inline-grid;gap:1px;flex:none;align-content:center}
.channel-order-button{display:grid;place-items:center;width:22px;min-height:14px;height:14px;padding:0;border:0;border-radius:var(--radius-2xs);background:transparent;color:var(--muted);font:inherit;font-size:var(--text-caption);line-height:1;cursor:pointer}
.channel-order-button:hover:not(:disabled){background:var(--raised);color:var(--text)}
.channel-order-button:focus-visible{outline:2px solid var(--accent);outline-offset:1px}
.channel-order-button:disabled{opacity:.35;cursor:default}
.channel-table td:last-child{padding-inline-end:var(--space-1)}
/* WP-40：启用列 76px 内横排开关与文字；竖排会撑高整行。 */
.channel-enable-cell{display:flex;align-items:center;gap:var(--space-1);flex-wrap:nowrap;white-space:nowrap;min-width:0}
.channel-enable-cell>span{font-size:var(--text-label);white-space:nowrap}
/* 窄窗：操作组两行排布（编辑/复制 + 删除 + 箭头），行内留白而不是挤到列外。 */
@container(max-width:860px){
 .channel-table{min-width:0}
 .channel-node-col{width:140px}
 .channel-count-col{width:60px}
 .channel-enable-col{width:60px}
 .channel-actions-col{width:160px}
 .channel-row-actions{flex-wrap:wrap;justify-content:flex-end;gap:var(--space-1);row-gap:2px}
 .channel-enable-cell span{display:none}
 .channel-table td:last-child{padding-block:var(--space-1)}
}
@container(max-width:700px){
 .channel-node-col{width:132px}
 .channel-actions-col{width:auto}
 .channel-row-actions{justify-content:flex-start}
}
</style>
