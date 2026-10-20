<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch, toRef } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ChannelEditor from './ChannelEditor.vue'
import NewChannelDialog from './NewChannelDialog.vue'
import ClientLoadStatus from './ClientLoadStatus.vue'
import ModelPicker from './ModelPicker.vue'
import { mergeUpstream } from './upstream'
import {copyChannel,protectedChannel} from './channelActions'
import {reorderSlots} from './ordering'
import { useModelsStore } from './store'
import { errorText, savePlanErrorText, modelApi, type ChannelPolicy, type DraftProjection, type SavePlan, type ModelClientState } from './api'
import { useModalStore } from '@/app/modal/store'
const props=defineProps<{actionsTarget?:string;active?:boolean}>()
const models = useModelsStore(); const modal = useModalStore()
const search = toRef(models.view,'channelSearch'); const editorId = toRef(models.view,'channelEditor'); const preview = ref<DraftProjection | null>(null); const error = ref(''); const previewBusy = ref(false)
const editor = computed(() => models.channelDraft?.channels.find(c => c.id === editorId.value))
watch(editorId,id=>{if(id)models.view.channelFilters[id]??={search:'',selectedOnly:false,family:'all'}},{immediate:true,flush:'sync'})
const pickerOpen = ref(false)
const savePlan=ref<SavePlan|null>(null);const saveOpen=ref(false);const saving=ref(false)
const clientState=ref<ModelClientState|null>(null)
const clientLabel=computed(()=>clientState.value?.status==='user_confirmed'?'已写入 · 用户已确认':clientState.value?.status==='pending_verification'?'已写入 · 待客户端验证':clientState.value?.status==='runtime_changed'?'运行配置变化 · 待核对':clientState.value?.status==='not_required'?'已写入 · 无需新增重启':'已写入 · 待重启 Codex')
const policyStatus=computed(()=>saving.value?{label:'正在核对',tone:'warning'}:savePlan.value&&!['prepared','committed','compensated'].includes(savePlan.value.phase)?{label:'保存待核对',tone:'danger'}:models.channelDirty?{label:'未保存修改',tone:'warning'}:savePlan.value?.phase==='committed'?{label:clientLabel.value,tone:'success'}:savePlan.value?.phase==='compensated'?{label:'已撤回本次写入',tone:'neutral'}:{label:'未修改',tone:'neutral'})
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
async function prepareSave(){
  if(!models.channelDraft||!models.snapshot||models.busy||models.staging)return
  models.busy=true;error.value=''
  try{await models.persistDraft();if(models.channelError)throw new Error(models.channelError)
    savePlan.value=await modelApi.preparePolicy(JSON.parse(JSON.stringify(models.channelDraft)),models.snapshot.libraryDigest,models.channelBaseline,models.snapshot.workspaceDigest);saveOpen.value=true
  }catch(cause){error.value=errorText(cause)}finally{models.busy=false}
}
async function applySave(){
  if(!savePlan.value||saving.value)return
  saving.value=true;models.busy=true;error.value=''
  try{savePlan.value=await modelApi.applyPolicy(savePlan.value.id)
    if(savePlan.value.phase==='committed')models.acceptTransfer(await modelApi.load())
  }catch(cause){error.value=errorText(cause);try{savePlan.value=await modelApi.policyResult(savePlan.value.id)}catch{/* The original plan id remains visible for explicit readback. */}}
  finally{saving.value=false;models.busy=false}
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
    const sourceDefault=typeof snapshot.providers==='object'&&snapshot.providers!==null&&'defaultProvider' in snapshot.providers?String(snapshot.providers.defaultProvider):''
    if(models.channelDraft&&!models.channelDraft.defaultChannelId){const defaultChannel=channels.find(c=>c.connection.providerId===sourceDefault);if(defaultChannel)models.channelDraft.defaultChannelId=defaultChannel.id}
    upstreamVersion.value = snapshot.version
  } catch (cause) { error.value = errorText(cause) } finally { upstreamBusy.value = false }
}
const selectedIds=toRef(models.view,'selectedChannels')
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
 modal.open({title:'删除渠道',body:'将 '+values.map(c=>c.label).join('、')+' 从草稿移除。保存时一并移除对应官方运行渠道及其协议分组；此前可用还原修改撤回。',confirmLabel:'从草稿删除',onConfirm:()=>{
  if(JSON.stringify(models.channelDraft)!==baseline){error.value='删除确认已过期，请重新核对当前草稿';return}
  if(!models.channelDraft)return
  for(const channel of models.channelDraft.channels.filter(c=>ids.includes(c.id))){if(channel.connection.sourceReadId&&!models.snapshot?.policy.channels.some(c=>c.id===channel.id)){
    const pending=models.channelDraft.pendingRemovals??[];models.channelDraft.pendingRemovals=[...pending.filter(r=>r.channelId!==channel.id),{readId:channel.connection.sourceReadId,channelId:channel.id,providerId:channel.connection.providerId,credentialRef:channel.connection.credentialRef}]
  }}
  models.channelDraft.channels=models.channelDraft.channels.filter(c=>!ids.includes(c.id));if(ids.includes(models.channelDraft.defaultChannelId??''))models.channelDraft.defaultChannelId=null
  selectedIds.value=selectedIds.value.filter(id=>!ids.includes(id));if(ids.includes(editorId.value))editorId.value=''
 }})
}
const channels = computed(() => models.channelDraft?.channels.filter(c => c.label.toLowerCase().includes(search.value.toLowerCase())) ?? [])
const newChannelOpen=ref(false)
function create(){if(!models.busy&&!saving.value)newChannelOpen.value=true}
function restore() {
  const baseline = JSON.stringify(models.channelDraft); const policy = models.snapshot?.policyDigest
  modal.open({ title: '还原修改', body: '将全部渠道恢复到最近成功保存的配置？', confirmLabel: '还原修改', onConfirm: () => {
    if (baseline !== JSON.stringify(models.channelDraft) || policy !== models.snapshot?.policyDigest) { error.value = '还原确认已过期，请重新比较当前修改'; return }
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
async function showPreview() {
  if (!models.channelDraft) return
  previewBusy.value = true; error.value = ''
  try { preview.value = await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft))); pickerOpen.value = true }
  catch (cause) { error.value = errorText(cause) } finally { previewBusy.value = false }
}
</script>
<template>
  <div class="channels-view">
    <div class="model-toolbar"><input class="input" v-model="search" type="search" placeholder="搜索渠道" aria-label="搜索渠道" /><div class="model-inline"><UiButton :loading="upstreamBusy" :disabled="models.busy||saving" @click="readUpstream">读取现有渠道</UiButton><UiButton :disabled="models.busy||saving" @click="create">新增渠道</UiButton></div></div>
    <Teleport defer :to="actionsTarget??'body'" :disabled="!actionsTarget"><div v-if="props.active!==false" class="model-inline model-policy-actions"><UiButton :loading="previewBusy" @click="showPreview">预览选择器</UiButton><span class="model-policy-state" :data-tone="policyStatus.tone" role="status">● {{ policyStatus.label }}</span><UiButton :disabled="!models.channelDirty||models.busy||saving" @click="restore">还原修改</UiButton><UiButton :variant="models.channelDirty?'primary':'default'" :disabled="!models.channelDirty || models.staging||saving" :loading="models.busy" @click="prepareSave">保存</UiButton></div></Teleport>
    <p v-if="upstreamVersion" class="model-muted">来源：{{ upstreamVersion }} · 已读取到草稿</p>
    <p v-if="search.trim()" class="model-muted">清除搜索后可调整渠道顺序</p>
    <ClientLoadStatus @change="clientState=$event" :refresh-key="(models.snapshot?.policyDigest??'')+':'+(savePlan?.phase??'')" />
    <div v-if="savePlan && !['prepared','compensated'].includes(savePlan.phase)" class="model-inline"><span class="tag">{{ savePlan.phase === 'committed' ? '最近保存已写入' : '上次保存尚未完成核对' }}</span><UiButton size="sm" @click="saveOpen=true">查看原保存结果</UiButton></div>
    <p v-if="error || models.channelError" class="model-error" role="alert">{{ error || models.channelError }}</p><p v-if="models.staging" class="model-muted" role="status">正在暂存渠道草稿…</p>
    <label v-if="models.channelDraft" class="model-inline">默认渠道<UiSelectField :model-value="models.channelDraft.defaultChannelId??''" label="默认渠道" :disabled="models.busy||saving" :options="[{value:'',label:'保持官方当前默认'},...models.channelDraft.channels.filter(c=>c.connection.enabled).map(c=>({value:c.id,label:c.label||'未命名渠道'}))]" @update:model-value="models.channelDraft!.defaultChannelId=$event||null" /></label>
    <div v-if="selected.length" class="model-inline"><span>已选 {{ selected.length }} 个渠道</span><UiButton size="sm" :disabled="models.busy||saving" @click="enableSelected(true)">启用</UiButton><UiButton size="sm" :disabled="models.busy||saving" @click="enableSelected(false)">停用</UiButton><UiButton size="sm" variant="danger" :disabled="models.busy||saving" @click="deleteChannels(selected)">删除所选渠道</UiButton></div>
    <ChannelEditor v-if="editor" :key="editor.id" :view-state="models.view.channelFilters[editor.id]" :existing="models.snapshot?.policy.channels.some(c=>c.id===editorId)" :channel="editor" :global-refs="models.channelDraft?.templateRefs??[]" :projection="detailProjection?.rows.filter(row=>row.channelId===editorId)??[]" :projection-busy="detailProjectionBusy" :projection-error="detailProjectionError" :templates="models.snapshot?.library.templates ?? []" :busy="models.busy||protectedChannel(editor)" @close="editorId = ''" />
    <template v-else><div class="model-table-scroll"><table class="model-table"><thead><tr><th><input type="checkbox" aria-label="选择当前结果中的可编辑渠道" :checked="allSelected" :indeterminate="someSelected" @change="selectAll" /></th><th>渠道</th><th>服务节点</th><th>模型</th><th>操作</th></tr></thead><tbody><tr v-for="(c,channelIndex) in channels" :key="c.id" @dragover.prevent @drop.prevent="dropChannel(c.id)"><td><input type="checkbox" :aria-label="'选择渠道 '+c.label" :checked="selectedIds.includes(c.id)" :disabled="protectedChannel(c)||models.busy||saving" @change="selectOne(c.id)" /></td><td><strong>{{ c.label || '未命名渠道' }}</strong><small>{{ c.connection.nodes.length }} 节点 · {{ c.connection.enabled ? '启用' : '停用' }}</small></td><td><UiSelectField v-model="c.connection.activeNodeId" :label="`${c.label}当前节点`" :options="c.connection.nodes.map(n => ({ value: n.id, label: n.name || '未命名节点' }))" size="sm" /></td><td>{{ c.selection.mode === 'all' ? c.discovered.length : c.selection.mode === 'whitelist' ? c.selection.models.length : 0 }} / {{ c.discovered.length }}</td><td><div class="model-inline"><UiButton size="sm" variant="ghost" :draggable="reorderAllowed" :disabled="!reorderAllowed" :aria-label="c.label+'拖动排序'" @dragstart="dragChannel($event,c.id)" @dragend="draggedChannel=''">↕</UiButton><UiButton size="sm" :data-model-focus="'channel:'+c.id" @click="editorId = c.id">{{ protectedChannel(c)?'查看':'编辑' }}</UiButton><UiButton size="sm" :disabled="protectedChannel(c)||models.busy||saving" @click="beginCopy(c)">复制</UiButton><UiButton size="sm" variant="danger" :disabled="protectedChannel(c)||models.busy||saving" @click="deleteChannels([c])">删除</UiButton><UiButton size="sm" variant="ghost" :disabled="!reorderAllowed||channelIndex===0" :aria-label="c.label+'上移'" @click="move(c.id,-1)">↑</UiButton><UiButton size="sm" variant="ghost" :disabled="!reorderAllowed||channelIndex===channels.length-1" :aria-label="c.label+'下移'" @click="move(c.id,1)">↓</UiButton><UiButton size="sm" variant="ghost" :disabled="!reorderAllowed||channelIndex===0" :aria-label="c.label+'置顶'" @click="reorderChannel(c.id,channels[0]!.id)">置顶</UiButton></div></td></tr></tbody></table></div><p v-if="!channels.length" class="model-empty">暂无渠道</p></template>
    <NewChannelDialog v-if="newChannelOpen" @close="newChannelOpen=false" @added="editorId=$event" />
    <UiDialog :open="saveOpen" title="确认保存渠道配置" :busy="saving" @close="saveOpen = false"><template v-if="savePlan"><p v-if="savePlan.historySource">本次保存来自成功历史恢复，按原始官方字段、目录顺序和受保护授权执行。</p><div v-if="savePlan.removedProviders?.length"><h3>将移除的运行渠道</h3><p>{{ savePlan.removedProviders.join('、') }}</p></div><p>完整目录条目：{{ savePlan.currentCatalogCount }} → {{ savePlan.catalogCount }} / 100（当前兼容预算）</p><p>范围：{{ savePlan.changedChannels.join('、') }}</p><div v-if="savePlan.protocolGroups?.length"><h3>协议分组</h3><p v-for="group in savePlan.protocolGroups" :key="group" class="mono">{{ group }}</p></div><div v-if="savePlan.routeChanges?.length"><h3>模型运行标识或快捷入口变更</h3><p>以下分组或别名变更会影响旧选择入口，已有会话需按对应提示重新选择型号。</p><p v-for="change in savePlan.routeChanges" :key="change" class="mono">{{ change }}</p></div><details v-if="savePlan.runtimeChanges?.length"><summary>核对原始官方字段差异（{{ savePlan.runtimeChanges.length }} 项）</summary><article v-for="change in savePlan.runtimeChanges" :key="change.field"><strong>{{ change.field }}</strong><pre>当前：{{ JSON.stringify(change.before,null,2) }}</pre><pre>恢复后：{{ JSON.stringify(change.after,null,2) }}</pre></article></details><p v-if="savePlan.phase === 'prepared'">确认后备份并调用官方接口写入，完成后仍需核对客户端加载状态。</p><p v-else>执行阶段：{{ ({applying:'执行中',providers_written:'渠道已写入',visibility_written:'可见性已写入',order_written:'顺序已写入',runtime_verified:'运行读回已核对',committed:'保存完成，待客户端核对'}[savePlan.phase] ?? '待核对') }}</p><p v-if="compensationConfirm">将核对本次最后读回的运行态，只撤回本次写入；若发现后来修改会拒绝补偿。确认继续？</p><p v-if="savePlan.error || error" class="model-error" role="alert">{{ error || savePlanErrorText(savePlan.error) }}</p></template><template #actions><UiButton :disabled="saving" @click="saveOpen = false">{{ savePlan?.phase === 'prepared' ? '取消' : '关闭' }}</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" :disabled="saving" @click="querySave">核对原计划</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" variant="danger" :loading="saving" @click="compensate">{{ compensationConfirm ? '确认撤回本次写入' : '撤回本次未完成写入' }}</UiButton><UiButton v-if="savePlan?.phase === 'prepared'" variant="primary" :loading="saving" @click="applySave">确认保存</UiButton></template></UiDialog>
    <UiDialog :open="!!copySource" title="复制渠道" :busy="models.busy" @close="copySource=null"><label>副本名称<input v-model="copyName" class="input" maxlength="256" /></label><label v-if="copySource?.connection.credentialRef"><input v-model="reuseCopyKey" type="checkbox" />副本复用此本机渠道Key</label><p>副本采用独立渠道和节点身份，保持停用。模版匹配范围需在预览中核对。</p><template #actions><UiButton @click="copySource=null">取消</UiButton><UiButton variant="primary" :disabled="!copyName.trim()" @click="confirmCopy">建立副本草稿</UiButton></template></UiDialog>
    <ModelPicker :open="pickerOpen" :rows="preview?.rows ?? []" :channels="models.channelDraft?.channels ?? []" reasoning @close="pickerOpen = false" @confirm="pickerOpen = false" />
  </div>
</template>
