<script setup lang="ts">
import { computed, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ChannelEditor from './ChannelEditor.vue'
import ModelPicker from './ModelPicker.vue'
import { mergeUpstream } from './upstream'
import { useModelsStore } from './store'
import { emptyRules, errorText, modelApi, type ChannelPolicy, type DraftProjection, type SavePlan } from './api'
import { useModalStore } from '@/app/modal/store'
const models = useModelsStore(); const modal = useModalStore()
const search = ref(''); const editorId = ref(''); const previewOpen = ref(false); const preview = ref<DraftProjection | null>(null); const error = ref(''); const previewBusy = ref(false)
const editor = computed(() => models.channelDraft?.channels.find(c => c.id === editorId.value))
const pickerOpen = ref(false)
const savePlan=ref<SavePlan|null>(null);const saveOpen=ref(false);const saving=ref(false)
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
    if(savePlan.value.phase==='committed'){const snapshot=await modelApi.load();models.snapshot=snapshot;models.restoreChannels()}
  }catch(cause){error.value=errorText(cause);try{savePlan.value=await modelApi.policyResult(savePlan.value.id)}catch{/* The original plan id remains visible for explicit readback. */}}
  finally{saving.value=false;models.busy=false}
}

const compensationConfirm=ref(false)
async function querySave(){
 if(!savePlan.value)return
 saving.value=true;error.value=''
 try{savePlan.value=await modelApi.policyResult(savePlan.value.id);if(savePlan.value.phase==='committed'){models.snapshot=await modelApi.load();models.restoreChannels()}}
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
  try { preview.value = await modelApi.project(JSON.parse(JSON.stringify(models.channelDraft))); previewOpen.value = true }
  catch (cause) { error.value = errorText(cause) } finally { previewBusy.value = false }
}
</script>
<template>
  <div class="channels-view">
    <div class="model-toolbar"><input class="input" v-model="search" type="search" placeholder="搜索渠道" aria-label="搜索渠道" /><div class="model-inline"><UiButton :loading="previewBusy" @click="showPreview">预览选择器</UiButton><span class="tag">{{ savePlan?.phase === 'committed' ? '已写入 · 待客户端核对' : models.channelDirty ? '未保存修改' : '未修改' }}</span><UiButton :disabled="!models.channelDirty" @click="restore">还原修改</UiButton><UiButton :loading="upstreamBusy" @click="readUpstream">读取现有渠道</UiButton><UiButton variant="primary" :disabled="!models.channelDirty || models.staging" :loading="models.busy" @click="prepareSave">保存</UiButton><UiButton :disabled="models.busy" @click="create">新增渠道</UiButton></div></div>
    <p v-if="upstreamVersion" class="model-muted">来源：{{ upstreamVersion }} · 已读取到草稿</p>
    <p v-if="error || models.channelError" class="model-error" role="alert">{{ error || models.channelError }}</p><p v-if="models.staging" class="model-muted" role="status">正在暂存渠道草稿…</p>
    <ChannelEditor v-if="editor" :channel="editor" :templates="models.snapshot?.library.templates ?? []" :busy="models.busy" @close="editorId = ''" />
    <template v-else><div class="model-table-scroll"><table class="model-table"><thead><tr><th>渠道</th><th>服务节点</th><th>模型</th><th>操作</th></tr></thead><tbody><tr v-for="c in channels" :key="c.id"><td><strong>{{ c.label || '未命名渠道' }}</strong><small>{{ c.connection.nodes.length }} 节点 · {{ c.connection.enabled ? '启用' : '停用' }}</small></td><td><UiSelectField v-model="c.connection.activeNodeId" :label="`${c.label}当前节点`" :options="c.connection.nodes.map(n => ({ value: n.id, label: n.name || '未命名节点' }))" size="sm" /></td><td>{{ c.selection.mode === 'all' ? c.discovered.length : c.selection.mode === 'whitelist' ? c.selection.models.length : 0 }} / {{ c.discovered.length }}</td><td><div class="model-inline"><UiButton size="sm" @click="editorId = c.id">编辑</UiButton><UiButton size="sm" variant="ghost" :aria-label="`${c.label}上移`" @click="move(c.id,-1)">↑</UiButton><UiButton size="sm" variant="ghost" :aria-label="`${c.label}下移`" @click="move(c.id,1)">↓</UiButton></div></td></tr></tbody></table></div><p v-if="!channels.length" class="model-empty">暂无渠道</p></template>
    <UiDialog :open="previewOpen" title="选择器预览" wide @close="previewOpen = false"><template v-if="preview"><UiButton @click="pickerOpen = true">试用选择器</UiButton><p>草稿中已知激活模型 {{ preview.knownActiveModels }} · 客户端条目数待核对</p><p v-for="notice in preview.notices" :key="notice" class="model-muted">{{ notice }}</p><table class="model-table"><thead><tr><th>渠道 / 模型</th><th>公开</th><th>最终协议</th><th>来源</th></tr></thead><tbody><tr v-for="row in preview.rows" :key="`${row.channelId}/${row.modelId}`"><td>{{ row.providerId }} / {{ row.label }}</td><td>{{ row.enabled && row.selected ? '已选' : '未公开' }}</td><td>{{ row.rules.protocol.mode === 'custom' ? row.rules.protocol.value : '继承' }}</td><td>{{ row.source.protocol }}</td></tr></tbody></table></template></UiDialog>
    <UiDialog :open="saveOpen" title="确认保存渠道配置" :busy="saving" @close="saveOpen = false"><template v-if="savePlan"><p>完整目录条目：{{ savePlan.currentCatalogCount }} → {{ savePlan.catalogCount }} / 100（当前兼容预算）</p><p>范围：{{ savePlan.changedChannels.join('、') }}</p><p v-if="savePlan.phase === 'prepared'">确认后备份并调用官方接口写入，完成后仍需核对客户端加载状态。</p><p v-else>执行阶段：{{ ({applying:'执行中',providers_written:'渠道已写入',visibility_written:'可见性已写入',order_written:'顺序已写入',runtime_verified:'运行读回已核对',committed:'保存完成，待客户端核对'}[savePlan.phase] ?? '待核对') }}</p><p v-if="compensationConfirm">将核对本次最后读回的运行态，只撤回本次写入；若发现后来修改会拒绝补偿。确认继续？</p><p v-if="savePlan.error || error" class="model-error" role="alert">{{ error || savePlan.error }}</p></template><template #actions><UiButton :disabled="saving" @click="saveOpen = false">{{ savePlan?.phase === 'prepared' ? '取消' : '关闭' }}</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" :disabled="saving" @click="querySave">核对原计划</UiButton><UiButton v-if="savePlan && !['prepared','committed','compensated'].includes(savePlan.phase)" variant="danger" :loading="saving" @click="compensate">{{ compensationConfirm ? '确认撤回本次写入' : '撤回本次未完成写入' }}</UiButton><UiButton v-if="savePlan?.phase === 'prepared'" variant="primary" :loading="saving" @click="applySave">确认保存</UiButton></template></UiDialog>
    <ModelPicker :open="pickerOpen" :rows="preview?.rows ?? []" :channels="models.channelDraft?.channels ?? []" reasoning @close="pickerOpen = false" @confirm="pickerOpen = false" />
  </div>
</template>
