<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import { useSyncStore } from './store'
import { errorText } from '@/features/models/api'
import { testSyncCandidate, type SaveSyncEndpointRequest } from './api'
import { formSignature, sameEndpointForm, type ObservationToken } from './observation'
import { webdavStates } from './states'
const ImportedConnections = defineAsyncComponent(() => import('./ImportedConnections.vue'))
const emit = defineEmits<{ close: [] }>()
const sync = useSyncStore()
const busy = ref(false), testing = ref(false), message = ref(''), error = ref('')
const value = ref<SaveSyncEndpointRequest>({ baseUrl: '', remotePath: '', username: '', password: '', conflictPolicy: 'ask' })
const loading = ref(true), ready = ref(false)
const formEdited = computed(() => sync.config?.endpoint ? !sameEndpointForm(value.value, sync.config.endpoint) || !!value.value.password : [value.value.baseUrl, value.value.remotePath, value.value.username, value.value.password].some(Boolean))
const savedObservationText = computed(() => {
  if (formEdited.value) return '未保存的编辑 · 当前连接观察保持'
  const observation = sync.observation
  const time = observation ? new Date(observation.observedAt).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false }) : ''
  return webdavStates[sync.webdavState].conn + (time ? ' · ' + time : '')
})
let disposed = false, generation = 0
let proof: { signature: string; token: ObservationToken; succeeded: boolean; observedAt: string } | null = null
let testToken: ObservationToken | null = null
async function load() {
  const request = ++generation
  loading.value = true; error.value = ''; proof = null
  try {
    const config = await sync.loadConfig()
    if (disposed || request !== generation) return
    if (!config) throw new Error('无法读取已保存连接；请重新读取后再保存。')
    const endpoint = config.endpoint
    if (endpoint) value.value = { baseUrl: endpoint.url, remotePath: endpoint.remotePath, username: endpoint.username, password: '', conflictPolicy: endpoint.conflictPolicy }
    ready.value = true
  } catch (cause) { if (!disposed && request === generation) { error.value = errorText(cause); ready.value = false } }
  finally { if (!disposed && request === generation) loading.value = false }
}
onMounted(load)
watch(value, () => { proof = null; message.value = ''; if (!testing.value) error.value = '' }, { deep: true })
onBeforeUnmount(() => { disposed = true; generation++; value.value.password = ''; proof = null; if (testToken) sync.endObservation(testToken) })
function valid() {
  try {
    const url = new URL(value.value.baseUrl.trim())
    if (url.protocol !== 'https:' || url.username || url.password || url.search || url.hash) throw new Error()
  } catch { error.value = '请填写不含内嵌凭据或参数的 HTTPS 服务地址。'; return false }
  if (!value.value.remotePath.trim() || !value.value.username.trim()) { error.value = '请填写独立目录和用户名。'; return false }
  const endpoint = sync.config?.endpoint
  if (!value.value.password && (!endpoint || endpoint.url.replace(/\/+$/, '') !== value.value.baseUrl.trim().replace(/\/+$/, '') || endpoint.username !== value.value.username.trim())) {
    error.value = '新连接或更换服务地址、用户名后，请填写连接密码。'; return false
  }
  return true
}
async function test() {
  if (busy.value || testing.value || !ready.value || !valid()) return
  const request = ++generation, snapshot = { ...value.value }, signature = formSignature(snapshot)
  const token = sync.observationToken(), saved = !snapshot.password && sameEndpointForm(snapshot, sync.config?.endpoint)
  testToken = token; testing.value = true; message.value = ''; error.value = ''; proof = null
  if (saved) sync.beginObservation(token, 'connecting')
  try {
    const revision = sync.config?.endpoint?.revision
    const result = await testSyncCandidate({ ...snapshot, ...(revision ? { expectedRevision: revision } : {}) })
    if (disposed || request !== generation || formSignature(value.value) !== signature || !sync.matchesConfig(token)) return
    const succeeded = result.operationState === 'succeeded', observedAt = new Date().toISOString()
    proof = { signature, token, succeeded, observedAt }
    if (saved) sync.recordObservation(token, succeeded ? 'connected' : 'failed', { observedAt, source: 'test' })
    if (succeeded) message.value = result.message || '连接测试成功。'
    else error.value = result.message || '连接测试失败。'
  } catch (cause) {
    if (!disposed && request === generation && sync.matchesConfig(token)) {
      error.value = errorText(cause)
      if (saved) sync.recordObservation(token, 'failed', { source: 'test' })
    }
  } finally {
    snapshot.password = ''
    sync.endObservation(token)
    if (!disposed && request === generation) testing.value = false
    testToken = null
  }
}
async function save() {
  if (busy.value || testing.value || !ready.value || !valid()) return
  const snapshot = { ...value.value }, signature = formSignature(snapshot), candidateProof = proof
  const request = ++generation
  busy.value = true; error.value = ''
  try {
    const proofStillCurrent = !!candidateProof && candidateProof.signature === signature && sync.matchesConfig(candidateProof.token)
    await sync.commitEndpoint(snapshot)
    if (proofStillCurrent && candidateProof) sync.recordObservation(sync.observationToken(), candidateProof.succeeded ? 'connected' : 'failed', { source: 'test', observedAt: candidateProof.observedAt })
    if (!disposed && request === generation) emit('close')
  } catch (cause) { if (!disposed && request === generation) error.value = errorText(cause) }
  finally { snapshot.password = ''; if (!disposed && request === generation) busy.value = false }
}
</script>
<template>
  <UiDialog :open="true" title="WebDAV 连接" :busy="busy || testing" panel-class="sync-connection-dialog" @close="emit('close')">
    <div class="sx-connection-fields">
      <label><span>服务地址</span><input :disabled="busy || testing || loading" v-model="value.baseUrl" class="input" autocapitalize="off" autocorrect="off" spellcheck="false" /></label>
      <label><span>独立配置同步目录</span><input :disabled="busy || testing || loading" v-model="value.remotePath" class="input" autocapitalize="off" autocorrect="off" spellcheck="false" /></label>
      <div class="sx-grid">
        <label><span>用户名</span><input :disabled="busy || testing || loading" v-model="value.username" class="input" autocapitalize="off" autocorrect="off" spellcheck="false" /></label>
        <label><span>连接密码</span><input :disabled="busy || testing || loading" v-model="value.password" type="password" class="input" autocomplete="new-password" :placeholder="sync.config?.endpoint ? '本机已保存 · 留空保留' : '填写连接密码'" /></label>
      </div>
      <div><UiButton :loading="testing" :disabled="busy || testing || !ready" @click="test">测试连接</UiButton></div>
      <UiButton v-if="!ready && !loading" :disabled="busy || testing" @click="load">重新读取</UiButton>
      <p v-if="message" class="model-muted" role="status">{{ message }}</p>
      <p v-else-if="ready && !error" class="model-muted" role="status">{{ savedObservationText }}</p>
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <ImportedConnections @activated="load" />
    </div>
    <template #actions="{ close }">
      <UiButton :disabled="busy || testing" @click="close">取消</UiButton>
      <UiButton variant="primary" :loading="busy" :disabled="busy || testing || !ready" @click="save">保存连接</UiButton>
    </template>
  </UiDialog>
</template>
<style scoped>
:global(.sync-connection-dialog.ui-dialog){width:min(720px,calc(86vw / var(--ui-zoom)));max-height:calc(78vh / var(--ui-zoom))}
.sx-connection-fields,.sx-connection-fields label{display:grid;gap:8px}.sx-connection-fields{gap:16px}.sx-grid{display:grid;grid-template-columns:1fr 1fr;gap:16px}.model-error{color:var(--red)}
@media(max-width:600px){.sx-grid{grid-template-columns:1fr}}
</style>
