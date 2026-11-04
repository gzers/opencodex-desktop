<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { errorText } from '@/features/models/api'
import { listImportedConnections, activateImportedConnection, type ImportedConnection } from './api'
const emit = defineEmits<{ activated: [] }>()
const items = ref<ImportedConnection[]>([]), baseline = ref(''), selected = ref<ImportedConnection | null>(null)
const loading = ref(true), busy = ref(false), error = ref(''), loaded = ref(false)
let disposed = false, generation = 0
async function load() {
  if (busy.value) return
  const request = ++generation
  loading.value = true; error.value = ''
  try {
    const result = await listImportedConnections()
    if (disposed || request !== generation) return
    if (!result || !Array.isArray(result.items) || typeof result.baseline !== 'string') throw new Error('导入连接列表返回无效，请重试。')
    items.value = result.items; baseline.value = result.baseline; loaded.value = true
  } catch (cause) { if (!disposed && request === generation) error.value = errorText(cause) }
  finally { if (!disposed && request === generation) loading.value = false }
}
onMounted(() => { void load() })
onBeforeUnmount(() => { disposed = true; generation++ })
async function activate() {
  if (!selected.value || !selected.value.hasPassword || busy.value || loading.value) return
  const id = selected.value.id, expected = baseline.value, request = ++generation
  busy.value = true; error.value = ''
  try {
    await activateImportedConnection(id, expected)
    if (disposed || request !== generation) return
    selected.value = null; emit('activated')
  } catch (cause) { if (!disposed && request === generation) error.value = errorText(cause) }
  finally { if (!disposed && request === generation) busy.value = false }
  if (!disposed && request === generation && !error.value) await load()
}
</script>
<template>
  <section v-if="loading || error || items.length" class="imported-connections">
    <p v-if="loading" role="status" class="model-muted">正在读取导入的连接…</p>
    <template v-else>
      <p v-if="error && !selected" class="imported-error" role="alert">{{ error }} <UiButton size="sm" :disabled="busy" @click="load">重新读取</UiButton></p>
      <details v-if="loaded && items.length" class="sx-imported-connections">
        <summary>导入的连接（{{ items.length }}）</summary>
        <p class="model-muted">来自配置包，尚未启用。导入不会切换当前连接；明确启用后用于后续同步。</p>
        <div v-for="item in items" :key="item.id" class="imported-row">
          <div><strong>{{ item.endpoint.username }} · {{ item.endpoint.url }}</strong><small>{{ item.endpoint.remotePath }} · {{ item.hasPassword ? '已携带密码' : '未携带密码，请在连接表单补全' }}</small></div>
          <UiButton :disabled="busy || !item.hasPassword" @click="selected = item">启用此连接</UiButton>
        </div>
      </details>
    </template>
    <UiDialog :open="!!selected" title="启用导入的WebDAV连接" :busy="busy" @close="selected = null">
      <template v-if="selected"><p>后续同步将使用以下服务器；本设备同步范围保持。</p><p>{{ selected.endpoint.url }} / {{ selected.endpoint.remotePath }}</p><p>账号：{{ selected.endpoint.username }}</p></template>
      <p v-if="error" role="alert" class="imported-error">{{ error }}</p>
      <template #actions="{ close }"><UiButton :disabled="busy" @click="close">取消</UiButton><UiButton variant="primary" :loading="busy" @click="activate">确认启用</UiButton></template>
    </UiDialog>
  </section>
</template>
<style scoped>
.imported-row{display:flex;gap:16px;align-items:center;justify-content:space-between;border-top:1px solid var(--border);padding:14px 0}.imported-row>div{min-width:0;overflow-wrap:anywhere}.imported-row small{display:block;color:var(--muted);margin-top:6px}.imported-error{color:var(--red);overflow-wrap:anywhere}.imported-connections{min-width:0}.model-muted{color:var(--muted)}
</style>
