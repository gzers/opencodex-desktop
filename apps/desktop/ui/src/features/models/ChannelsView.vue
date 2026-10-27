<script setup lang="ts">
import {escapeHtml} from '@/lib/html'
import { computed, onBeforeUnmount, ref, watch, toRef } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ChannelEditor from './ChannelEditor.vue'
import ChannelBatchDialog from './ChannelBatchDialog.vue'
import ChannelNodeSelect from './ChannelNodeSelect.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import NewChannelDialog from './NewChannelDialog.vue'
import {useModelSaveFeedback} from '@/app/feedback/modelSave'
import UiStatus from '@/components/ui/UiStatus.vue'
import {channelPolicyState} from '@/lib/saveStates'
import SelectorPreviewDialog from './SelectorPreviewDialog.vue'
import { inlineChanges, setModelActive } from './saveReview'
import {copyChannel,protectedChannel} from './channelActions'
import {reorderSlots} from './ordering'
import {usePointerDrag} from './pointerDrag'
import {createModelRowsState} from './modelRows'
import { useModelsStore } from './store'
import { errorText, modelApi, type ChannelPolicy, type DraftProjection, type ModelPolicy, type SaveBlockedReason } from './api'
import { useModalStore } from '@/app/modal/store'
import {useFeedbackStore} from '@/app/feedback/store'
const props=withDefaults(defineProps<{actionsTarget?:string;active?:boolean}>(),{active:true})
const models = useModelsStore(); const modal = useModalStore()
const search = toRef(models.view,'channelSearch'); const editorId = toRef(models.view,'channelEditor'); const error = ref('')
const editor = computed(() => models.channelDraft?.channels.find(c => c.id === editorId.value))
watch(editorId,id=>{if(id){models.view.channelFilters[id]??={search:'',selectedOnly:false,family:'all'};models.modelRows[id]??=createModelRowsState();models.view.navigation['channel:'+id]??={collapsed:false,activeId:'channel-config'}}},{immediate:true,flush:'sync'})
const previewOpen = ref(false);const previewProjection = ref<DraftProjection|null>(null);const previewBusy = ref(false);const previewError = ref('')
/**
 * 系统占用（原型 model-config.js:518 的「系统占用 只读」分组）：官方目录里已存在、
 * 但选择器不可见的条目（原生隐藏项与未纳入管理的占用）。数据源是官方运行目录
 * model_agent_directory 的 pickerVisible=false 行，不猜测、不补数。
 * 取不到时留空，预算摘要退回「未提供」，与原型 same-name 字段同义。
 */
const systemRows = ref<{id:string;label:string}[]>([])
async function loadSystemRows(){
  try{
    const directory = await modelApi.executionDirectory()
    systemRows.value = directory.rows
      .filter(row => row.pickerVisible === false)
      .map(row => ({id: row.channelId + '/' + row.modelId, label: row.label || row.channelId + '/' + row.modelId}))
  }catch{ systemRows.value = [] }
}

let previewSequence = 0
let previewRefreshSequence = 0
const saving=ref(false)
const editingLocked=computed(()=>models.busy||saving.value||!!models.historicalRecovery)
const policyStatus=computed(()=>channelPolicyState({saving:saving.value,pendingRows:models.pendingRows.length,draftDirty:models.channelDirty}))
const policyHint=computed(()=>saving.value?'正在保存':undefined)
const detailPending=computed(()=>models.pendingRows.filter(row=>row.channel===editorId.value))
const detailDirty=computed(()=>!!editorId.value&&models.channelIsDirty(editorId.value))
const detailStatus=computed(()=>channelPolicyState({saving:saving.value,pendingRows:detailPending.value.length,draftDirty:detailDirty.value}))
const detailProjection=ref<DraftProjection|null>(null),detailProjectionError=ref(''),detailProjectionBusy=ref(false)
let detailSequence=0,detailTimer:ReturnType<typeof setTimeout>|undefined
watch(()=>[editorId.value,JSON.stringify(models.channelDraft),models.snapshot?.libraryDigest,models.snapshot?.policyDigest],(_next,previous)=>{
 const sequence=++detailSequence;if(detailTimer)clearTimeout(detailTimer);if(!previous||previous[0]!==editorId.value)detailProjection.value=null;detailProjectionError.value='';detailProjectionBusy.value=false
 if(!editorId.value||!models.channelDraft)return
 let policy:ModelPolicy
 try{policy=models.channelSaveCandidate(editorId.value)}catch(cause){detailProjectionError.value=errorText(cause);return}
 detailProjectionBusy.value=true
 detailTimer=setTimeout(async()=>{try{const result=await modelApi.project(policy);if(!disposed&&sequence===detailSequence)detailProjection.value=result}catch(cause){if(!disposed&&sequence===detailSequence)detailProjectionError.value=errorText(cause)}finally{if(sequence===detailSequence)detailProjectionBusy.value=false}},200)
},{immediate:true})
let disposed=false
onBeforeUnmount(()=>{disposed=true;detailSequence++;if(detailTimer)clearTimeout(detailTimer)})
/** 列表保存先预览全部剩余修改；详情保存独立按当前渠道提交（UI 规范 §47）。 */
const saveReviewBaseline=ref<{saved:string;library:string;recovery:string|null}|null>(null),saveConfirming=ref(false)
async function prepareSave(){
  // §38.4「再次点击不会重复提交」：除按钮禁用外，函数自身也要挡住并发与重入，
  // 列表预览与详情写入也共享互斥状态，旧事件不能绕过按钮态。
  if(!models.channelDraft||!models.snapshot||!models.channelDirty&&!models.pendingRows.length||models.busy||models.staging||saving.value||saveConfirming.value||previewBusy.value||previewOpen.value)return
  const sequence=++previewSequence
  previewBusy.value=true;previewError.value='';error.value=''
  try{const projection=await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft)))
    if(sequence!==previewSequence)return
    previewProjection.value=projection;saveReviewBaseline.value={saved:models.channelBaseline,library:models.snapshot.libraryDigest,recovery:models.historicalRecovery?.id??null};previewOpen.value=true
    void loadSystemRows()
  }catch(cause){if(sequence===previewSequence){previewProjection.value=null;previewError.value=errorText(cause);previewOpen.value=true}}
  finally{if(sequence===previewSequence)previewBusy.value=false}
}
/**
 * 弹窗内启停后的重算。用独立序号：打开预览与刷新预览共用序号会让“打开过程中的
 * 一次启停”把打开结果判成过期，弹窗静默不出现（真实缺陷，已由 parity 对照发现）。
 */
async function refreshSavePreview(){
  if(!models.channelDraft)return
  const sequence=++previewRefreshSequence;previewBusy.value=true
  try{const projection=await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft)))
    if(sequence===previewRefreshSequence){previewProjection.value=projection;previewError.value=''}
  }catch(cause){if(sequence===previewRefreshSequence)previewError.value=errorText(cause)}
  finally{if(sequence===previewRefreshSequence)previewBusy.value=false}
}
function togglePreviewChannel(id:string){
  const channel=models.channelDraft?.channels.find(c=>c.id===id)
  if(!channel||editingLocked.value||protectedChannel(channel))return
  channel.connection.enabled=!channel.connection.enabled
  void refreshSavePreview()
}
function togglePreviewModel(channelId:string,modelId:string){
  const channel=models.channelDraft?.channels.find(c=>c.id===channelId)
  if(!channel||editingLocked.value||protectedChannel(channel))return
  const active=channel.selection.mode==='all'||channel.selection.mode==='whitelist'&&channel.selection.models.includes(modelId)
  setModelActive(channel,[modelId],!active)
  void refreshSavePreview()
}
const saveReviewChanges=computed(()=>{
  const saved=models.snapshot?.policy,draft=models.channelDraft
  const changes=saved&&draft?inlineChanges(saved,draft):[]
  return changes.concat(Object.values(models.credentialEdits).map(edit=>({label:(draft?.channels.find(c=>c.id===edit.channelId)?.label??'渠道')+' · API Key',before:'已保存值',after:edit.action==='clear'?'清空':'修改'})))
})
/** 门禁顺序与原型一致：待确认编辑 → 超预算 → 校验失败 → 无修改 → 基准变化。 */
function saveGateReason():SaveBlockedReason|''{
  if(models.pendingRows.length)return 'staged_edit'
  if((previewProjection.value?.knownActiveModels??0)>100)return 'over_budget'
  if(previewError.value)return 'validation'
  if(!models.channelDirty)return 'no_changes'
  const baseline=saveReviewBaseline.value
  if(baseline&&(baseline.saved!==models.channelBaseline||baseline.library!==models.snapshot?.libraryDigest||baseline.recovery!==(models.historicalRecovery?.id??null)))return 'stale'
  return ''
}
/** 确认即关闭：无论写入还是被拦下，弹窗都先消失（§38.4）。 */
async function confirmSave(){
  if(saveConfirming.value||saving.value||!models.channelDraft||!models.snapshot)return
  const frozen=JSON.parse(JSON.stringify(models.channelDraft)) as ModelPolicy
  const expected=models.channelBaseline
  const recovery=models.historicalRecovery
  const reason=saveGateReason()
  saveConfirming.value=true;previewOpen.value=false;previewProjection.value=null;saveReviewBaseline.value=null
  if(reason){
    try{await modelApi.policySaveBlocked(reason)}catch{useFeedbackStore().showToast('本次没有写入，当前编辑已保留。',{kind:'error'})}
    saveConfirming.value=false;return
  }
  saving.value=true;models.busy=true;error.value=''
  try{
    if(recovery)await models.saveHistoricalRecovery(frozen)
    else{
      const edits=Object.values(models.credentialEdits).filter(e=>frozen.channels.some(c=>c.id===e.channelId)).map(e=>({...e}))
      const result=await modelApi.writeFile(expected,frozen,edits)
      models.acceptTransfer(result.snapshot)
    }
    useModelSaveFeedback().showSaved()
  }catch(cause){const message=(recovery?'历史恢复未完成：':'未写入：')+errorText(cause);if(recovery)error.value=message;useFeedbackStore().showToast(message,{kind:'error'})}
  finally{saving.value=false;models.busy=false;saveConfirming.value=false}
}

/** Detail Save shares the file writer but commits only this channel. Navigation
 * belongs to this still-current editor; a late response never closes another. */
async function saveDetail(){
  const id=editorId.value
  if(!id||!editor.value||protectedChannel(editor.value)||editingLocked.value||saveConfirming.value||!detailDirty.value&&!detailPending.value.length)return
  saving.value=true;error.value=''
  try{
    await models.saveCurrentChannel(id)
    useModelSaveFeedback().showSaved()
    if(props.active!==false&&editorId.value===id&&!models.channelIsDirty(id)&&!models.pendingRows.some(row=>row.channel===id))editorId.value=''
  }catch(cause){
    useFeedbackStore().showToast('未写入：'+errorText(cause),{kind:'error'})
  }finally{saving.value=false}
}

const upstreamBusy = ref(false)
async function readUpstream() {
  if(upstreamBusy.value)return
  upstreamBusy.value=true;error.value=''
  try{await models.load();if(models.error)error.value=models.error}
  finally{upstreamBusy.value=false}
}
const selectedIds=toRef(models.view,'selectedChannels')
const batchChannels=ref<{mode:'family'|'adopt';ids:string[]}>()
const copySource=ref<ChannelPolicy|null>(null);const copyName=ref('');const reuseCopyKey=ref(false);const copyBusy=ref(false)
const selectable=computed(()=>channels.value.filter(c=>!protectedChannel(c)))
const selected=computed(()=>(models.channelDraft?.channels??[]).filter(c=>selectedIds.value.includes(c.id)&&!protectedChannel(c)))
const allSelected=computed(()=>selectable.value.length>0&&selectable.value.every(c=>selectedIds.value.includes(c.id)))
const someSelected=computed(()=>selectable.value.some(c=>selectedIds.value.includes(c.id))&&!allSelected.value)
function selectAll(){const visible=new Set(selectable.value.map(c=>c.id));selectedIds.value=allSelected.value?selectedIds.value.filter(id=>!visible.has(id)):[...new Set([...selectedIds.value,...visible])]}
function selectOne(id:string){selectedIds.value=selectedIds.value.includes(id)?selectedIds.value.filter(value=>value!==id):[...selectedIds.value,id]}
function beginCopy(channel:ChannelPolicy){if(editingLocked.value||copyBusy.value||protectedChannel(channel))return;copySource.value=channel;copyName.value=channel.label+' 副本';reuseCopyKey.value=false;error.value=''}
async function confirmCopy(){
 if(!copySource.value||copyBusy.value||editingLocked.value)return
 copyBusy.value=true;error.value=''
 try{
  const current=models.channelDraft?.channels.find(c=>c.id===copySource.value!.id)
  if(!current)throw {message:'来源渠道已变化'}
  const baseline=JSON.stringify(current),pending=JSON.stringify(models.credentialEdits[current.id])
  const label=copyName.value,reuse=reuseCopyKey.value
  let key=''
  if(reuse){
   const edit=models.credentialEdits[current.id]
   if(edit)key=edit.action==='replace'?(edit.value??''):''
   else if(!['none','local'].includes(current.connection.authMode??'')){
    let saved
    try{saved=await modelApi.credential(current.connection.providerId)}catch{throw {message:'无法读取来源渠道 Key，副本尚未建立；请重试或取消复用。'}}
    if(saved.configured&&(!saved.viewable||!saved.value))throw {message:'来源渠道已配置 Key，但当前无法读取；副本尚未建立，请取消复用或稍后重试。'}
    if(saved.configured)key=saved.value
   }
  }
  const source=models.channelDraft?.channels.find(c=>c.id===current.id)
  if(editingLocked.value||JSON.stringify(source)!==baseline||JSON.stringify(models.credentialEdits[current.id])!==pending)throw {message:'来源渠道或 Key 已变化，副本尚未建立，请重新核对。'}
  const copy=copyChannel(JSON.parse(JSON.stringify(source)),label,false)
  models.channelDraft!.channels.push(copy)
  if(key)models.editCredential(copy.id,key,'')
  copySource.value=null;editorId.value=copy.id
 }catch(cause){error.value=errorText(cause)}finally{copyBusy.value=false}
}
function enableSelected(enabled:boolean){if(editingLocked.value)return;for(const channel of selected.value)channel.connection.enabled=enabled}
function deleteChannels(values:ChannelPolicy[]){
 if(editingLocked.value)return
 const ids=values.filter(c=>!protectedChannel(c)).map(c=>c.id);if(!ids.length)return
 const baseline=JSON.stringify(models.channelDraft)
 modal.open({title:'删除渠道',body:'将 '+values.map(c=>escapeHtml(c.label)).join('、')+' 从草稿移除（包含这些渠道未确认的型号输入）。保存时一并移除对应官方运行渠道及其协议分组；此前可用还原修改撤回。',confirmLabel:'从草稿删除',onConfirm:()=>{
  if(JSON.stringify(models.channelDraft)!==baseline){error.value='删除确认已过期，请重新核对当前草稿';return}
  if(!models.channelDraft)return
  for(const channel of models.channelDraft.channels.filter(c=>ids.includes(c.id))){if(channel.connection.sourceReadId&&!models.snapshot?.policy.channels.some(c=>c.id===channel.id)){
    const pending=models.channelDraft.pendingRemovals??[];models.channelDraft.pendingRemovals=[...pending.filter(r=>r.channelId!==channel.id),{readId:channel.connection.sourceReadId,channelId:channel.id,providerId:channel.connection.providerId,credentialRef:channel.connection.credentialRef}]
  }}
  models.channelDraft.channels=models.channelDraft.channels.filter(c=>!ids.includes(c.id))
  // Deleting an unrelated channel must not change the official defaultProvider.
  if(models.channelDraft.defaultChannelId&&ids.includes(models.channelDraft.defaultChannelId))models.channelDraft.defaultChannelId=null
  for(const id of ids)delete models.modelRows[id];selectedIds.value=selectedIds.value.filter(id=>!ids.includes(id));if(ids.includes(editorId.value))editorId.value=''
 }})
}
const channels = computed(() => models.channelDraft?.channels.filter(c => c.label.toLowerCase().includes(search.value.toLowerCase())) ?? [])
const newChannelOpen=ref(false)
function create(){if(!editingLocked.value)newChannelOpen.value=true}
function restore() {
  const baseline = JSON.stringify(models.channelDraft); const rows=JSON.stringify(models.modelRows); const policy = models.snapshot?.policyDigest
  modal.open({ title: '还原修改', body: models.historicalRecovery?'放弃此历史恢复草稿并重新读取当前配置？已提交的运行写入不会被撤回。':'将全部渠道恢复到最近成功保存的配置？', confirmLabel: '还原修改', onConfirm: async () => {
    if (rows!==JSON.stringify(models.modelRows)||baseline !== JSON.stringify(models.channelDraft) || policy !== models.snapshot?.policyDigest) { error.value = '还原确认已过期，请重新比较当前修改'; return }
    if(models.historicalRecovery){await models.load();if(models.error){error.value=models.error;return}}
    models.restoreChannels(); editorId.value = ''; error.value = ''
  } })
}
const draggedChannel=ref('')
const reorderAllowed=computed(()=>!search.value.trim()&&!editingLocked.value)
function reorderChannel(id:string,target:string){
 if(!reorderAllowed.value||!models.channelDraft)return
 const values=models.channelDraft.channels;const order=reorderSlots(values.map(c=>c.id),values.map(c=>c.id),id,target)
 models.channelDraft.channels=order.map(id=>values.find(c=>c.id===id)!)
}
function move(id:string,direction:number){const values=models.channelDraft?.channels??[];const index=values.findIndex(c=>c.id===id);const target=values[index+direction];if(target)reorderChannel(id,target.id)}
// 与模型行同一套指针拖拽：WKWebView 里 HTML5 拖放的 drop 常不触发，还会顺带框选文字。
const draggedOver=ref('')
const channelDrag=usePointerDrag({
 rowSelector:'tr[data-channel]',itemAttr:'data-channel',
 enabled:()=>reorderAllowed.value,
 onReorder:(id,target)=>reorderChannel(id,target),
 onActive:id=>{draggedChannel.value=id??''},
 onOver:id=>{draggedOver.value=id??''},
})
function dragChannel(event:PointerEvent,id:string){draggedOver.value='';channelDrag.start(event,id)}
</script>
<template>
  <div class="channels-view">
    <div v-if="!editor" class="model-toolbar"><input class="input" v-model="search" type="search" placeholder="搜索渠道" aria-label="搜索渠道" /><div class="model-inline"><UiButton :loading="upstreamBusy" :disabled="models.busy||saving" @click="readUpstream">读取现有渠道</UiButton><UiButton :disabled="editingLocked" @click="create">新增渠道</UiButton></div></div>
    <Teleport defer :to="actionsTarget??'body'" :disabled="!actionsTarget"><div v-if="props.active!==false&&!editor" class="model-inline model-policy-actions"><UiStatus :state="policyStatus" :hint="policyHint" /><UiButton :disabled="(!models.channelDirty&&!models.pendingRows.length)||models.busy||saving" @click="restore">还原修改</UiButton><UiButton data-testid="policy-save" :variant="models.channelDirty?'primary':'default'" :disabled="!models.channelDirty&&!models.pendingRows.length||models.busy||saving||saveConfirming" :loading="models.busy" :title="models.channelDirty||models.pendingRows.length?'保存 · 查看剩余修改的保存预览，确认后写入':'没有未保存修改'" @click="prepareSave">保存</UiButton></div></Teleport>
    <p v-if="search.trim()" class="model-muted">清除搜索后可调整渠道顺序</p>
    <p v-if="!editor&&models.pendingRows.length" role="status">{{ models.pendingRows.length }} 项型号编辑未确认；全局保存暂不可用。<UiButton v-for="target in [...new Set(models.pendingRows.map(r=>r.channel))]" :key="target" size="sm" variant="ghost" @click="editorId=target">查看 {{ models.channelDraft?.channels.find(c=>c.id===target)?.label??target }}</UiButton></p>

    <!-- 保存结果详情不再在列表上方占一行：入口走通知中心（model_policy action），见提案 §4.1。 -->
    <p v-if="error || models.channelError" class="model-error" role="alert">{{ error || models.channelError }}</p>
    <div v-if="selected.length&&!editor" class="model-inline"><span>已选 {{ selected.length }} 个渠道<span v-if="selected.some(c=>!channels.some(row=>row.id===c.id))">（含搜索外 {{ selected.filter(c=>!channels.some(row=>row.id===c.id)).length }} 个）</span></span><UiButton size="sm" :disabled="editingLocked" @click="enableSelected(true)">启用</UiButton><UiButton size="sm" :disabled="editingLocked" @click="enableSelected(false)">停用</UiButton><UiButton size="sm" variant="danger" :disabled="editingLocked" @click="deleteChannels(selected)">删除所选渠道</UiButton><UiButton size="sm" :disabled="editingLocked||!!models.pendingRows.length" @click="batchChannels={mode:'family',ids:selected.map(c=>c.id)}">按族批量配置</UiButton><UiButton size="sm" :disabled="editingLocked||!!models.pendingRows.length" @click="batchChannels={mode:'adopt',ids:selected.map(c=>c.id)}">套用模版</UiButton><UiButton size="sm" variant="ghost" @click="selectedIds=[]">清除选择</UiButton></div>
    <ChannelEditor v-if="editor" :key="editor.id" :navigation="models.view.navigation['channel:'+editor.id]" :channels="models.channelDraft?.channels" @switch="editorId=$event" @save="saveDetail" :dirty="detailDirty||!!detailPending.length" :save-status="detailStatus.label" :save-tone="detailStatus.tone" :save-disabled="!detailDirty&&!detailPending.length||models.busy||saving||saveConfirming||!!models.historicalRecovery" :active-count="detailProjection?.knownActiveModels" :view-state="models.view.channelFilters[editor.id]" :row-state="models.modelRows[editor.id]" :policy="models.channelDraft??undefined" :existing="models.snapshot?.policy.channels.some(c=>c.id===editorId)" :channel="editor" :global-refs="models.channelDraft?.templateRefs??[]" :projection="detailProjection?.rows.filter(row=>row.channelId===editorId)??[]" :projection-busy="detailProjectionBusy" :projection-error="detailProjectionError" :templates="models.snapshot?.library.templates ?? []" :busy="editingLocked||protectedChannel(editor)" @close="editorId = ''" />
    <template v-else><div class="model-table-scroll"><table class="model-table ui-table ui-table-fixed channel-table"><colgroup><col class="ui-selection-col"/><col class="channel-name-col"/><col class="channel-node-col"/><col class="channel-count-col"/><col class="channel-enable-col"/><col class="channel-actions-col"/></colgroup><thead><tr><th class="ui-selection-cell"><input type="checkbox" aria-label="选择当前结果中的可编辑渠道" :checked="allSelected" :indeterminate="someSelected" @change="selectAll" /></th><th>渠道</th><th>服务节点</th><th>模型</th><th>启用</th><th>操作</th></tr></thead><tbody><tr v-for="(c,channelIndex) in channels" :key="c.id" :data-channel="c.id" :class="{dragging:draggedChannel===c.id,'drag-over':draggedOver===c.id}" @pointerdown="dragChannel($event,c.id)"><td class="ui-selection-cell"><input type="checkbox" :aria-label="'选择渠道 '+c.label" :checked="selectedIds.includes(c.id)" :disabled="protectedChannel(c)||editingLocked" @change="selectOne(c.id)" /></td><td><strong>{{ c.label || '未命名渠道' }}</strong><small>{{ c.connection.nodes.length }} 节点 · {{ c.connection.enabled ? '启用' : '停用' }}</small></td><td><ChannelNodeSelect :channel="c" :disabled="editingLocked" :readonly="protectedChannel(c)" /></td><td class="ui-nowrap"><span class="channel-model-count">{{ c.selection.mode === 'all' ? c.discovered.length : c.selection.mode === 'whitelist' ? c.selection.models.length : 0 }} / {{ c.discovered.length }}</span><small class="channel-model-count-hint">已选 / 发现</small></td><td><span v-if="protectedChannel(c)">官方管理</span><div v-else class="channel-enable-cell"><UiSwitch :model-value="c.connection.enabled" :label="c.label+'渠道启用状态'" :disabled="editingLocked" @update:model-value="c.connection.enabled=$event" /><span>{{ c.connection.enabled?'启用':'停用' }}</span></div></td><td><div class="channel-row-actions" :class="{'is-readonly':protectedChannel(c)}"><UiButton size="sm" :data-model-focus="'channel:'+c.id" @click="editorId = c.id">{{ protectedChannel(c)?'查看':'编辑' }}</UiButton><UiButton size="sm" :disabled="protectedChannel(c)||editingLocked" @click="beginCopy(c)">复制</UiButton><UiButton size="sm" variant="danger" :disabled="protectedChannel(c)||editingLocked" @click="deleteChannels([c])">删除</UiButton><span class="channel-order" role="group" :aria-label="c.label+'排序'"><button type="button" class="channel-order-button" :disabled="!reorderAllowed||channelIndex===0" :aria-label="c.label+'上移'" :title="c.label+'上移'" @click="move(c.id,-1)">↑</button><button type="button" class="channel-order-button" :disabled="!reorderAllowed||channelIndex===channels.length-1" :aria-label="c.label+'下移'" :title="c.label+'下移'" @click="move(c.id,1)">↓</button></span></div></td></tr></tbody></table></div><p v-if="!channels.length" class="model-empty">暂无渠道</p>
  <p class="model-footer-note">启用开关修改当前编辑；停用保留已选模型与配置。拖动渠道调整整组顺序。</p></template>
    <ChannelBatchDialog v-if="batchChannels" :mode="batchChannels.mode" :targets="batchChannels.ids" @close="batchChannels=undefined" />
    <NewChannelDialog v-if="newChannelOpen" @close="newChannelOpen=false" @added="editorId=$event" />
    <UiDialog :open="!!copySource" title="复制渠道" :busy="copyBusy" @close="copySource=null"><label>副本名称<input v-model="copyName" class="input" maxlength="256" :disabled="copyBusy" /></label><label><input v-model="reuseCopyKey" type="checkbox" :disabled="copyBusy" />副本复用此本机渠道Key</label><p>副本采用独立渠道和节点身份，保持停用。模版匹配结果可在保存预览中查看。</p><p v-if="error" class="model-error" role="alert">{{ error }}</p><template #actions><UiButton :disabled="copyBusy" @click="copySource=null">取消</UiButton><UiButton variant="primary" :disabled="!copyName.trim()||copyBusy" :loading="copyBusy" @click="confirmCopy">建立副本草稿</UiButton></template></UiDialog>
    <SelectorPreviewDialog :open="previewOpen" :system-rows="systemRows" :projection="previewProjection" :saved="models.snapshot?.policy??null" :other-changes="saveReviewChanges" :channels="(models.channelDraft?.channels ?? []).map(c=>({id:c.id,label:c.label||c.id,enabled:c.connection.enabled,protected:protectedChannel(c)}))" :busy="previewBusy" :saving="saveConfirming" :error="previewError||undefined" @toggle-channel="togglePreviewChannel" @toggle-model="togglePreviewModel" @confirm="confirmSave" @close="previewOpen = false" />
  </div>
</template>

<style scoped>
/* 列宽对齐原型 .mp-provider-columns：36px 选择 + 弹性名称 + 190/76/76/200 固定列。 */
.channel-table{min-width:800px}.channel-name-col{width:auto}.channel-node-col{width:190px}.channel-count-col{width:76px}.channel-enable-col{width:76px}.channel-actions-col{width:200px}
.channel-table strong{overflow-wrap:anywhere}.channel-table td:nth-child(5) .model-inline{flex-wrap:nowrap}
/* WP-40：与原型一致，模型列在数值下给出“已选 / 发现”说明，避免 12/12 含义不明。 */
.channel-model-count{display:block;font-variant-numeric:tabular-nums}
.channel-model-count-hint{display:block;color:var(--muted);font-size:var(--text-caption);font-weight:400;white-space:nowrap}
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
/* 指针拖拽：拖起行半透明、目标行描边；拖拽期间整体禁选，避免一路框住文字。 */
.channel-table tbody>tr.dragging{opacity:.45}
.channel-table tbody>tr.drag-over{outline:2px solid var(--accent);outline-offset:-2px}
:global(html.ocx-pointer-drag),:global(html.ocx-pointer-drag *){-webkit-user-select:none!important;user-select:none!important}
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
