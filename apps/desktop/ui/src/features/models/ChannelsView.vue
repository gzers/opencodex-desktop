<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ChannelEditor from './ChannelEditor.vue'
import ClientLoadStatus from './ClientLoadStatus.vue'
import ModelPicker from './ModelPicker.vue'
import { mergeUpstream } from './upstream'
import { useModelsStore } from './store'
import { emptyRules, errorText, modelApi, type ChannelPolicy, type DraftProjection, type SavePlan, type ModelClientState } from './api'
import { useModalStore } from '@/app/modal/store'
defineProps<{actionsTarget?:string}>()
const models = useModelsStore(); const modal = useModalStore()
const search = ref(''); const editorId = ref(''); const preview = ref<DraftProjection | null>(null); const error = ref(''); const previewBusy = ref(false)
const editor = computed(() => models.channelDraft?.channels.find(c => c.id === editorId.value))
const pickerOpen = ref(false)
const savePlan=ref<SavePlan|null>(null);const saveOpen=ref(false);const saving=ref(false)
const clientState=ref<ModelClientState|null>(null)
const clientLabel=computed(()=>clientState.value?.status==='user_confirmed'?'已写入 · 用户已确认':clientState.value?.status==='pending_verification'?'已写入 · 待客户端验证':clientState.value?.status==='runtime_changed'?'运行配置变化 · 待核对':clientState.value?.status==='not_required'?'已写入 · 无需新增重启':'已写入 · 待重启 Codex')
const policyStatus=computed(()=>saving.value?{label:'正在核对',tone:'warning'}:savePlan.value&&!['prepared','committed','compensated'].includes(savePlan.value.phase)?{label:'保存待核对',tone:'danger'}:models.channelDirty?{label:'未保存修改',tone:'warning'}:savePlan.value?.phase==='committed'?{label:clientLabel.value,tone:'success'}:savePlan.value?.phase==='compensated'?{label:'已撤回本次写入',tone:'neutral'}:{label:'未修改',tone:'neutral'})
let disposed=false
onBeforeUnmount(()=>{disposed=true})
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
    upstreamVersion.value = snapshot.version
  } catch (cause) { error.value = errorText(cause) } finally { upstreamBusy.value = false }
}
const channels = computed(() => models.channelDraft?.channels.filter(c => c.label.toLowerCase().includes(search.value.toLowerCase())) ?? [])
function create() {
  if (!models.channelDraft) models.restoreChannels()
  const node = { id: crypto.randomUUID(), name: '默认节点', url: '' }
  const channel: ChannelPolicy = { id: crypto.randomUUID(), label: '', templateRefs: [], modelRefs: {}, connection: { providerId: '', protocol: 'openai-responses', nodes: [node], activeNodeId: node.id, credentialRef: null, enabled: false, allowPrivateNetwork: false }, selection: { mode: 'none' }, modelOrder: [], discovered: [], defaults: emptyRules(), overrides: {}, familyOverrides: {} }
  models.channelDraft?.channels.push(channel); editorId.value = channel.id
}
function restore() {
  const baseline = JSON.stringify(models.channelDraft); const policy = models.snapshot?.policyDigest
  modal.open({ title: '还原修改', body: '将全部渠道恢复到最近成功保存的配置？', confirmLabel: '还原修改', onConfirm: () => {
    if (baseline !== JSON.stringify(models.channelDraft) || policy !== models.snapshot?.policyDigest) { error.value = '还原确认已过期，请重新比较当前修改'; return }
    models.restoreChannels(); editorId.value = ''; error.value = ''
  } })
}
function move(id: string, direction: number) {
  const values = models.channelDraft?.channels; if (!values) return
  const index = values.findIndex(c => c.id === id); const target = index + direction
  if (index < 0 || target < 0 || target >= values.length) return
  const moved = values.splice(index, 1)[0]!; values.splice(target, 0, moved)
}
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
    <Teleport defer :to="actionsTarget??'body'" :disabled="!actionsTarget"><div class="model-inline model-policy-actions"><UiButton :loading="previewBusy" @click="showPreview">预览选择器</UiButton><span class="model-policy-state" :data-tone="policyStatus.tone" role="status">● {{ policyStatus.label }}</span><UiButton :disabled="!models.channelDirty||models.busy||saving" @click="restore">还原修改</UiButton><UiButton :variant="models.channelDirty?'primary':'default'" :disabled="!models.channelDirty || models.staging||saving" :loading="models.busy" @click="prepareSave">保存</UiButton></div></Teleport>
    <p v-if="upstreamVersion" class="model-muted">来源：{{ upstreamVersion }} · 已读取到草稿</p>
    <ClientLoadStatus @change="clientState=$event" :refresh-key="(models.snapshot?.policyDigest??'')+':'+(savePlan?.phase??'')" />
    <div v-if="savePlan && !['prepared','compensated'].includes(savePlan.phase)" class="model-inline"><span class="tag">{{ savePlan.phase === 'committed' ? '最近保存已写入' : '上次保存尚未完成核对' }}</span><UiButton size="sm" @click="saveOpen=true">查看原保存结果</UiButton></div>
    <p v-if="error || models.channelError" class="model-error" role="alert">{{ error || models.channelError }}</p><p v-if="models.staging" class="model-muted" role="status">正在暂存渠道草稿…</p>
    <ChannelEditor v-if="editor" :channel="editor" :templates="models.snapshot?.library.templates ?? []" :busy="models.busy" @close="editorId = ''" />
    <template v-else><div class="model-table-scroll"><table class="model-table"><thead><tr><th>渠道</th><th>服务节点</th><th>模型</th><th>操作</th></tr></thead><tbody><tr v-for="c in channels" :key="c.id"><td><strong>{{ c.label || '未命名渠道' }}</strong><small>{{ c.connection.nodes.length }} 节点 · {{ c.connection.enabled ? '启用' : '停用' }}</small></td><td><UiSelectField v-model="c.connection.activeNodeId" :label="`${c.label}当前节点`" :options="c.connection.nodes.map(n => ({ value: n.id, label: n.name || '未命名节点' }))" size="sm" /></td><td>{{ c.selection.mode === 'all' ? c.discovered.length : c.selection.mode === 'whitelist' ? c.selection.models.length : 0 }} / {{ c.discovered.length }}</td><td><div class="model-inline"><UiButton size="sm" @click="editorId = c.id">编辑</UiButton><UiButton size="sm" variant="ghost" :aria-label="`${c.label}上移`" @click="move(c.id,-1)">↑</UiButton><UiButton size="sm" variant="ghost" :aria-label="`${c.label}下移`" @click="move(c.id,1)">↓</UiButton></div></td></tr></tbody></table></div><p v-if="!channels.length" class="model-empty">暂无渠道</p></template>
    <UiDialog :open="saveOpen" title="确认保存渠道配置" :busy="saving" @close="saveOpen = false"><template v-if="savePlan"><p>完整目录条目：{{ savePlan.currentCatalogCount }} → {{ savePlan.catalogCount }} / 100（当前兼容预算）</p><p>范围：{{ savePlan.changedChannels.join('、') }}</p><div v-if="savePlan.protocolGroups?.length"><h3>协议分组</h3><p v-for="group in savePlan.protocolGroups" :key="group" class="mono">{{ group }}</p></div><div v-if="savePlan.routeChanges?.length"><h3>模型运行标识变更</h3><p>协议分组改变以下型号的运行标识，已有会话需重新选择对应型号。</p><p v-for="change in savePlan.routeChanges" :key="change" class="mono">{{ change }}</p></div><p v-if="savePlan.phase === 'prepared'">确认后备份并调用官方接口写入，完成后仍需核对客户端加载状态。</p><p v-else>执行阶段：{{ ({applying:'执行中',providers_written:'渠道已写入',visibility_written:'可见性已写入',order_written:'顺序已写入',runtime_verified:'运行读回已核对',committed:'保存完成，待客户端核对'}[savePlan.phase] ?? '待核对') }}</p><p v-if="compensationConfirm">将核对本次最后读回的运行态，只撤回本次写入；若发现后来修改会拒绝补偿。确认继续？</p><p v-if="savePlan.error || error" class="model-error" role="alert">{{ error || savePlan.error }}</p></template><template #actions><UiButton :disabled="saving" @click="saveOpen = false">{{ savePlan?.phase === 'prepared' ? '取消' : '关闭' }}</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" :disabled="saving" @click="querySave">核对原计划</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" variant="danger" :loading="saving" @click="compensate">{{ compensationConfirm ? '确认撤回本次写入' : '撤回本次未完成写入' }}</UiButton><UiButton v-if="savePlan?.phase === 'prepared'" variant="primary" :loading="saving" @click="applySave">确认保存</UiButton></template></UiDialog>
    <ModelPicker :open="pickerOpen" :rows="preview?.rows ?? []" :channels="models.channelDraft?.channels ?? []" reasoning @close="pickerOpen = false" @confirm="pickerOpen = false" />
  </div>
</template>
