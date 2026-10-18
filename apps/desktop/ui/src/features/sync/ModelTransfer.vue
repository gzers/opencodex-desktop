<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import { useModelsStore } from '@/features/models/store'
import { errorText, versionLabel } from '@/features/models/api'
import { modelTransferApi, scopeFor, type FilePlan, type IncomingPlan, type UploadPlan } from './modelTransfer'
const props = defineProps<{ remote?: boolean }>()
const models = useModelsStore()
const busy = ref(false)
const error = ref('')
const templateIds = ref<string[]>([])
const channelIds = ref<string[]>([])
const versions = ref<Record<string,string>>({})
const includeGlobal = ref(false)
const selectionOpen = ref(false)
const filePlan = ref<FilePlan | null>(null)
const upload = ref<UploadPlan | null>(null)
const incoming = ref<IncomingPlan | null>(null)
const incomingSource=ref<IncomingPlan|null>(null)
const receiveRefs=ref<string[]>([]);const receiveChannels=ref<string[]>([]);const receiveGlobal=ref(false);const receiveDirty=ref(false)
const exportedPath = ref('')
const exportPassword=ref('');const importPassword=ref('');const passwordOpen=ref(false);const importPath=ref('')
onBeforeUnmount(()=>{exportPassword.value='';importPassword.value=''})
const hasScope = computed(() => templateIds.value.length > 0 || channelIds.value.length > 0 || includeGlobal.value)
const preview = computed(() => incoming.value?.package ?? upload.value?.package ?? filePlan.value?.package)
onMounted(() => { if (!models.snapshot) void models.load() })
async function perform(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try { await action() } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
}
function begin() { selectionOpen.value = true; exportedPath.value = ''; error.value = '' }
async function prepare() {
  if (!models.snapshot || !hasScope.value) return
  await perform(async () => {
    const scope = scopeFor(models.snapshot!, templateIds.value, channelIds.value, includeGlobal.value, versions.value)
    if (props.remote) upload.value = await modelTransferApi.prepareUpload(scope)
    else filePlan.value = exportPassword.value?await modelTransferApi.prepare(scope,exportPassword.value):await modelTransferApi.prepare(scope)
    exportPassword.value=''
    selectionOpen.value = false
  })
}
async function receive() {
  await perform(async () => {
    if (models.channelDirty) { await models.persistDraft(); if (models.channelError) throw { message: models.channelError } }
    if (props.remote) incoming.value = await modelTransferApi.download()
    else {
      const selected = await open({ multiple: false, directory: false, filters: [{ name: '配置文件', extensions: ['ocxdconf','json'] }] })
      if (!selected || Array.isArray(selected)) return
      importPath.value=selected
      try{incoming.value = await modelTransferApi.receive(selected)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){passwordOpen.value=true;importPassword.value='';return}throw cause}
    }
    filePlan.value = null; upload.value = null
    if(incoming.value){incomingSource.value=incoming.value;receiveRefs.value=incoming.value.package.payload.templates.flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=incoming.value.package.payload.policy?.channels.map(c=>c.id)??[];receiveGlobal.value=!!(incoming.value.package.payload.policy?.templateRefs||incoming.value.package.payload.policy?.channelOrder);receiveDirty.value=false}
  })
}
async function unlockFile(){await perform(async()=>{incoming.value=await modelTransferApi.receive(importPath.value,importPassword.value);passwordOpen.value=false;importPassword.value='';filePlan.value=null;upload.value=null;if(incoming.value){incomingSource.value=incoming.value;receiveRefs.value=incoming.value.package.payload.templates.flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=incoming.value.package.payload.policy?.channels.map(c=>c.id)??[];receiveGlobal.value=!!(incoming.value.package.payload.policy?.templateRefs||incoming.value.package.payload.policy?.channelOrder);receiveDirty.value=false}})}
async function confirm() {
  await perform(async () => {
    if (incoming.value) {
      if(receiveDirty.value||incoming.value.previewError)return
      if (models.channelDirty) { await models.persistDraft(); if (models.channelError) throw { message: models.channelError } }
      const saved = await modelTransferApi.accept(incoming.value.id)
      models.acceptTransfer(saved); incoming.value = null;incomingSource.value=null
    } else if (upload.value) {
      upload.value = await modelTransferApi.publish(upload.value.id)
    } else if (filePlan.value) {
      const result = await modelTransferApi.export(filePlan.value.id)
      exportedPath.value = result.path; filePlan.value = null
    }
  })
}
async function compareReceived(){if(!incomingSource.value)return;await perform(async()=>{incoming.value=await modelTransferApi.select(incomingSource.value!.id,{templateRefs:receiveRefs.value.map(value=>{const [templateId,revisionId]=JSON.parse(value) as [string,string];return {templateId,revisionId}}),channelIds:receiveChannels.value,includeGlobal:receiveGlobal.value});receiveDirty.value=false})}
function closePreview() { if (!busy.value) { incoming.value = null;incomingSource.value=null; filePlan.value = null; upload.value = null } }
</script>
<template>
  <article class="card model-transfer">
    <div class="card-head"><div><h2>{{ remote ? '模型配置 · WebDAV' : '模型配置 · 文件同步' }}</h2><p>{{ remote ? '使用已保存的活动端点，下载后比较并采用。' : '选择已保存版本和已提交渠道；采用后进入本机草稿。' }}</p></div></div>
    <p v-if="models.error && !models.snapshot" class="model-error" role="alert">{{ models.error }}</p>
    <p v-if="error && !selectionOpen && !preview" class="model-error" role="alert">{{ error }}</p>
    <div class="controls"><UiButton :disabled="busy || !models.snapshot" @click="begin">{{ remote ? '选择上传范围' : '导出模型配置' }}</UiButton><UiButton :loading="busy" @click="receive">{{ remote ? '下载并比较' : '导入模型配置' }}</UiButton><UiButton variant="ghost" @click="models.load">重新读取</UiButton></div>
    <p v-if="exportedPath" class="model-transfer-path"><span>导出文件</span><code>{{ exportedPath }}</code></p>
    <UiDialog :open="selectionOpen" :title="remote ? '选择模型上传范围' : '选择模型导出范围'" :busy="busy" wide @close="selectionOpen = false">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <fieldset><legend>模版保存版本</legend><div v-for="template in models.snapshot?.library.templates ?? []" :key="template.id" class="model-transfer-choice"><label><input v-model="templateIds" type="checkbox" :value="template.id" :disabled="busy" />{{ template.label }}</label><UiSelectField :model-value="versions[template.id] ?? template.revisions.at(-1)!.id" :options="[...template.revisions].reverse().map(r => ({ value: r.id, label: versionLabel(r, template.source === 'builtin' && r === template.revisions[0]) }))" :label="template.label + '导出版本'" :disabled="busy || !templateIds.includes(template.id)" @update:model-value="versions[template.id] = $event" /></div></fieldset>
      <fieldset><legend>已提交渠道</legend><label v-for="channel in models.snapshot?.policy.channels ?? []" :key="channel.id"><input v-model="channelIds" type="checkbox" :value="channel.id" :disabled="busy" />{{ channel.label }}</label><p v-if="!models.snapshot?.policy.channels.length">暂无已提交渠道</p></fieldset>
      <label><input v-model="includeGlobal" type="checkbox" :disabled="busy" />全局采用及渠道组顺序</label>
      <label v-if="!remote">文件保护口令（可选）<input v-model="exportPassword" class="input" type="password" autocomplete="new-password" :disabled="busy" placeholder="留空使用完整性校验" /></label>
      <template #actions><UiButton variant="primary" :disabled="!hasScope" :loading="busy" @click="prepare">预览清单</UiButton></template>
    </UiDialog>
    <UiDialog :open="!!preview" :title="incoming ? '比较模型配置' : remote ? '确认模型上传' : '确认模型导出'" :busy="busy" wide @close="closePreview">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <p v-if="filePlan">文件保护：{{ filePlan.protected?'独立口令加密':'未加密，完整性校验' }}</p>
      <details v-if="incomingSource" open class="model-transfer-scope"><summary>选择接收范围</summary><div v-for="template in incomingSource.package.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><label v-for="revision in template.revisions" :key="revision.id"><input v-model="receiveRefs" type="checkbox" :value="JSON.stringify([template.id,revision.id])" :disabled="busy" @change="receiveDirty=true" />{{ versionLabel(revision) }}</label></div><label v-for="channel in incomingSource.package.payload.policy?.channels??[]" :key="channel.id"><input v-model="receiveChannels" type="checkbox" :value="channel.id" :disabled="busy" @change="receiveDirty=true" />{{ channel.label }}</label><label v-if="incomingSource.package.payload.policy?.templateRefs||incomingSource.package.payload.policy?.channelOrder"><input v-model="receiveGlobal" type="checkbox" :disabled="busy" @change="receiveDirty=true" />全局采用及渠道顺序</label><p>所选渠道需要的保存版本会一并接收。</p><UiButton :loading="busy" @click="compareReceived">比较所选范围</UiButton></details>
      <template v-if="preview"><p>模版 {{ preview.payload.templates.length }} · 渠道 {{ preview.payload.policy?.channels.length ?? 0 }}</p>
        <ul class="model-transfer-manifest"><li v-for="template in preview.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><span>{{ template.revisions.length }} 个保存版本</span><code>{{ template.id }}</code><small v-if="incoming">{{ models.snapshot?.library.templates.some(t => t.id === template.id) ? '合并保存版本，保持本机默认' : '新增模版，不自动采用' }}</small></li><li v-for="channel in preview.payload.policy?.channels ?? []" :key="channel.id"><strong>{{ channel.label }}</strong><span>{{ channel.selection.mode === 'all' ? '全部模型' : channel.selection.mode === 'none' ? '不选择模型' : '指定模型' }}</span><code>{{ channel.id }}</code><small v-if="incoming">{{ models.snapshot?.policy.channels.some(c => c.id === channel.id) ? '保留本机连接，策略进入草稿' : '新增待绑定渠道，保持停用' }}</small></li></ul>
        <p v-for="notice in incoming?.notices ?? []" :key="notice">{{ notice }}</p><code class="model-transfer-digest">{{ preview.digest }}</code>
      </template>
      <template #actions><UiButton v-if="upload?.phase === 'published'" variant="primary" @click="closePreview">完成</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="!!incoming&&(receiveDirty||!!incoming.previewError)" @click="confirm">{{ incoming ? '采用到本机' : remote ? (upload?.phase === 'prepared' ? '确认上传' : '核对原上传') : '生成导出文件' }}</UiButton></template>
    </UiDialog>
    <UiDialog :open="passwordOpen" title="打开受保护配置文件" :busy="busy" @close="passwordOpen=false;importPassword=''">
      <label>文件保护口令<input v-model="importPassword" class="input" type="password" autocomplete="off" :disabled="busy" /></label><p v-if="error" class="model-error" role="alert">{{ error }}</p><template #actions><UiButton variant="primary" :disabled="!importPassword" :loading="busy" @click="unlockFile">解密并比较</UiButton></template>
    </UiDialog>
  </article>
</template>
<style scoped>
fieldset{border:1px solid var(--border);border-radius:var(--radius);margin:0 0 18px;padding:14px;display:grid;gap:12px;min-width:0}
label{display:flex;align-items:center;gap:8px}
.model-error{color:var(--red);overflow-wrap:anywhere}
.model-transfer-choice{display:grid;grid-template-columns:minmax(120px,1fr) minmax(0,2fr);align-items:center;gap:12px}
.model-transfer-path{display:grid;gap:8px;overflow-wrap:anywhere}
.model-transfer-manifest{list-style:none;padding:0;display:grid;gap:12px}
.model-transfer-manifest li{display:grid;grid-template-columns:1fr auto;gap:6px;border-bottom:1px solid var(--border);padding:12px 0}
.model-transfer-manifest code,.model-transfer-manifest small{grid-column:1/-1;overflow-wrap:anywhere;color:var(--muted)}
.model-transfer-digest{display:block;overflow-wrap:anywhere;font-size:var(--text-small)}
.model-transfer-scope{border-bottom:1px solid var(--border);padding-bottom:16px;margin-bottom:16px}.model-transfer-scope>div{display:grid;gap:8px;margin:14px 0}.model-transfer-scope>label{margin:10px 0}
@media(max-width:600px){.model-transfer-choice{grid-template-columns:1fr}}
</style>
