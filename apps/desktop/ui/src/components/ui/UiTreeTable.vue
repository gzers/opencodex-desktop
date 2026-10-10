<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue"
import { initialTreeExpansion, treeTableRows, type TreeTableColumn, type TreeTableNode, type TreeTableRow } from "./treeTable"

const props = withDefaults(defineProps<{
  nodes: readonly TreeTableNode[]
  columns?: readonly TreeTableColumn[]
  label: string
  nameLabel?: string
  expandedIds?: readonly string[]
  initialExpandedDepth?: number
  loading?: boolean
  error?: string
  emptyLabel?: string
}>(), { columns: () => [], nameLabel: "名称", initialExpandedDepth: 2, loading: false, emptyLabel: "暂无内容" })
const emit = defineEmits<{
  "update:expandedIds": [ids: string[]]
  select: [id: string, selected: boolean]
  retry: []
}>()
const localExpanded = ref(initialTreeExpansion(props.nodes, props.initialExpandedDepth))
const expanded = computed(() => new Set(props.expandedIds ?? localExpanded.value))
const rows = computed(() => treeTableRows(props.nodes, expanded.value))
const controls = new Map<string, HTMLButtonElement>()
const focusedId = ref<string>()
let refreshFocusId: string | undefined
const retryControl = ref<HTMLButtonElement>()
let retryHadFocus = false
function setControl(id: string, element: unknown) {
  if (element instanceof HTMLButtonElement) controls.set(id, element)
  else controls.delete(id)
}
function toggle(node: TreeTableNode) {
  if (!node.children?.length) return
  const next = new Set(expanded.value)
  if (next.has(node.id)) next.delete(node.id)
  else next.add(node.id)
  localExpanded.value = [...next]
  emit("update:expandedIds", [...next])
}
async function focus(id?: string) {
  await nextTick()
  if (id) controls.get(id)?.focus()
}
function onKey(event: KeyboardEvent, row: TreeTableRow) {
  const index = rows.value.findIndex(item => item.node.id === row.node.id)
  let target: string | undefined
  switch (event.key) {
    case "ArrowDown": target = rows.value[index + 1]?.node.id; break
    case "ArrowUp": target = rows.value[index - 1]?.node.id; break
    case "Home": target = rows.value[0]?.node.id; break
    case "End": target = rows.value.at(-1)?.node.id; break
    case "ArrowRight":
      if (row.node.children?.length && !expanded.value.has(row.node.id)) toggle(row.node)
      else target = row.node.children?.[0]?.id
      break
    case "ArrowLeft":
      if (expanded.value.has(row.node.id)) toggle(row.node)
      else target = row.parentId
      break
    default: return
  }
  event.preventDefault()
  void focus(target)
}
watch(rows, async (next, previous) => {
  const id = focusedId.value
  const active = document.activeElement
  if (!id || controls.get(id) !== active || next.some(row => row.node.id === id)) return
  let old = previous.find(row => row.node.id === id)
  while (old?.parentId && !next.some(row => row.node.id === old?.parentId)) old = previous.find(row => row.node.id === old?.parentId)
  await nextTick()
  if (document.activeElement === document.body || document.activeElement === active) {
    controls.get(old?.parentId ?? next[0]?.node.id ?? "")?.focus()
  }
})
watch(() => props.loading, async loading => {
  if (loading) {
    refreshFocusId = [...controls].find(([, element]) => element === document.activeElement)?.[0]
    retryHadFocus = retryControl.value === document.activeElement
    return
  }
  const id = refreshFocusId
  const fromRetry = retryHadFocus
  refreshFocusId = undefined
  retryHadFocus = false
  await nextTick()
  // Restore only focus lost when our rows were removed; never steal it from another control.
  if ((id || fromRetry) && document.activeElement === document.body) {
    if (props.error) {
      retryControl.value?.focus()
      return
    }
    const target = id && controls.has(id) ? id : rows.value[0]?.node.id
    if (target) controls.get(target)?.focus()
  }
}, { flush: "pre" })
</script>

<template>
  <div class="ui-tree-table" :aria-busy="loading || undefined">
    <table :aria-label="label">
      <thead><tr><th scope="col">{{ nameLabel }}</th><th v-for="column in columns" :key="column.key" scope="col">{{ column.label }}</th></tr></thead>
      <tbody>
        <tr v-if="error || loading || !rows.length"><td :colspan="columns.length + 1" class="tree-message">
          <span :role="error ? 'alert' : 'status'">{{ error || (loading ? "正在读取…" : emptyLabel) }}</span>
          <button v-if="error && !loading" ref="retryControl" type="button" class="btn ghost" @click="emit('retry')">重试</button>
        </td></tr>
        <template v-else>
          <tr v-for="row in rows" :key="row.node.id" :data-node-id="row.node.id">
            <th scope="row">
              <div class="tree-name" :style="{ '--tree-depth': row.depth }">
                <input v-if="row.node.selectable" type="checkbox" class="tree-check" :aria-label="`选择 ${row.node.label}`" :checked="row.node.selected === true" :indeterminate="row.node.selected === 'mixed'" :aria-checked="row.node.selected === 'mixed' ? 'mixed' : row.node.selected === true" :disabled="row.node.disabled" @change="emit('select', row.node.id, ($event.target as HTMLInputElement).checked)" />
                <span v-else class="tree-check-placeholder" aria-hidden="true"></span>
                <button v-if="row.node.children?.length" :ref="el => setControl(row.node.id, el)" class="tree-toggle" type="button" :aria-label="row.node.children?.length ? `${expanded.has(row.node.id) ? '收起' : '展开'} ${row.node.label}` : row.node.label" :aria-expanded="row.node.children?.length ? expanded.has(row.node.id) : undefined" @focus="focusedId = row.node.id" @keydown="onKey($event, row)" @click="toggle(row.node)">
                  <svg v-if="row.node.children?.length" aria-hidden="true" viewBox="0 0 16 16" :class="{ expanded: expanded.has(row.node.id) }"><path d="m6 4 4 4-4 4" /></svg>
                </button>
                <span v-else class="tree-toggle-placeholder" aria-hidden="true"></span>
                <span class="tree-icon" aria-hidden="true"><slot name="icon" :node="row.node" :depth="row.depth"></slot></span>
                <button v-if="!row.node.children?.length" :ref="el => setControl(row.node.id, el)" class="tree-label tree-leaf" type="button" :aria-label="row.node.label" @focus="focusedId = row.node.id" @keydown="onKey($event, row)"><slot name="name" :node="row.node" :depth="row.depth">{{ row.node.label }}</slot></button>
                <span v-else class="tree-label"><slot name="name" :node="row.node" :depth="row.depth">{{ row.node.label }}</slot></span>
              </div>
            </th>
            <td v-for="column in columns" :key="column.key"><slot :name="`cell-${column.key}`" :node="row.node" :depth="row.depth"></slot></td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.ui-tree-table { overflow-x: auto; border: 1px solid var(--border); border-radius: 10px; }
table { width: 100%; border-collapse: collapse; text-align: left; font-size: var(--text-label); }
th, td { padding: 10px 12px; vertical-align: top; border-bottom: 1px solid var(--border); }
thead th { color: var(--muted); font-weight: inherit; }
tbody th { font-weight: inherit; }
tbody tr:last-child > * { border-bottom: 0; }
.tree-name { display: flex; align-items: center; gap: 6px; min-height: 22px; padding-inline-start: calc(var(--tree-depth) * 16px); }
.tree-check, .tree-check-placeholder { flex: 0 0 16px; width: 16px; height: 16px; margin: 0; }
.tree-toggle { flex: 0 0 22px; width: 22px; height: 22px; padding: 3px; border: 0; background: transparent; color: inherit; border-radius: 4px; display: grid; place-items: center; cursor: pointer; }
.tree-toggle:focus-visible, .tree-leaf:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
.tree-toggle-placeholder { flex: 0 0 22px; width: 22px; height: 22px; }
.tree-leaf { text-align: left; border: 0; padding: 0; background: transparent; color: inherit; font: inherit; line-height: 22px; cursor: pointer; border-radius: 4px; }
.tree-toggle svg { width: 16px; height: 16px; fill: none; stroke: currentColor; stroke-width: 1.5; }
.tree-toggle svg.expanded { transform: rotate(90deg); }
.tree-icon { flex: 0 0 18px; height: 22px; display: grid; place-items: center; }
.tree-label { min-width: 0; line-height: 22px; overflow-wrap: anywhere; }
.tree-message { color: var(--muted); padding: 18px 12px; }
.tree-message button { margin-inline-start: 12px; }
</style>
