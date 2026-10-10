<script setup lang="ts">
import { onMounted, ref } from 'vue'
import UiTreeTable from '@/components/ui/UiTreeTable.vue'
import { initialTreeExpansion, type TreeTableNode } from '@/components/ui/treeTable'
import { getBackupFiles, type BackupFileNode } from './api'

const emit = defineEmits<{ open: [node: BackupFileNode] }>()
const nodes = ref<BackupFileNode[]>([])
const expanded = ref<string[]>([])
const rootPath = ref('')
const loading = ref(false)
const error = ref('')
let initialized = false
async function refresh() {
  if (loading.value) return
  loading.value = true
  error.value = ''
  try {
    const result = await getBackupFiles()
    nodes.value = result.nodes
    rootPath.value = result.rootPath
    if (!initialized && result.nodes.length) {
      expanded.value = initialTreeExpansion(result.nodes, 2)
      initialized = true
    }
  } catch {
    error.value = '无法读取备份文件。请检查目录权限、文件类型与清单，修正后重试。'
  } finally { loading.value = false }
}
function file(node: TreeTableNode): BackupFileNode { return node as BackupFileNode }
defineExpose({ refresh })
onMounted(() => { void refresh() })
</script>

<template>
  <section class="backup-files" aria-labelledby="backup-files-title">
    <div class="backup-files-head">
      <h3 id="backup-files-title">备份文件与目录</h3>
      <button class="btn ghost" type="button" :disabled="loading" @click="refresh">刷新</button>
    </div>
    <p v-if="rootPath" class="backup-root">{{ rootPath }}</p>
    <UiTreeTable v-model:expanded-ids="expanded" :nodes="nodes" :loading="loading" :error="error" label="备份文件与目录" empty-label="尚无备份文件" :columns="[{ key: 'purpose', label: '用途' }, { key: 'action', label: '操作' }]" @retry="refresh">
      <template #icon="{ node }">
        <svg v-if="file(node).directory" viewBox="0 0 20 20" aria-hidden="true"><path d="M2 5h6l2 2h8v10H2Z" /></svg>
        <svg v-else viewBox="0 0 20 20" aria-hidden="true"><path d="M5 2h7l4 4v12H5Z M12 2v5h4" /></svg>
      </template>
      <template #cell-purpose="{ node }"><span class="backup-purpose">{{ file(node).purpose }}</span></template>
      <template #cell-action="{ node }">
        <button class="btn ghost backup-open" type="button" :disabled="!file(node).canOpen" :aria-label="(file(node).directory ? '打开目录 ' : '打开文件 ') + node.label" @click="emit('open', file(node))">{{ file(node).directory ? '打开目录' : '打开文件' }}</button>
      </template>
    </UiTreeTable>
  </section>
</template>

<style scoped>
.backup-files { margin-top: 20px; }
.backup-files-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 10px; }
h3 { font-size: 13px; margin: 0; }
.backup-root { color: var(--muted); font-size: 11px; overflow-wrap: anywhere; margin: 0 0 12px; }
svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-width: 1.3; }
.backup-purpose { color: var(--muted); line-height: 22px; }
.backup-open { white-space: nowrap; }
</style>
