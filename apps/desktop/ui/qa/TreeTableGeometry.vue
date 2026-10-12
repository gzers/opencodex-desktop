<script setup lang="ts">
import UiTreeTable from '../src/components/ui/UiTreeTable.vue'
import type { TreeTableNode } from '../src/components/ui/treeTable'

const nodes: TreeTableNode[] = [{
  id: 'parent', label: 'Root with mixed selection', selectable: true, selected: 'mixed',
  children: [
    { id: 'selected', label: 'Selected leaf', selectable: true, selected: true },
    { id: 'disabled', label: 'Disabled leaf', selectable: true, disabled: true },
    { id: 'long', label: 'Long name across several lines with spaces and enough content to exercise the first line contract', selectable: true },
    { id: 'plain', label: 'Plain leaf without a checkbox' },
  ],
}]
</script>

<template>
  <main class="fixture">
    <UiTreeTable :nodes="nodes" :initial-expanded-depth="8" label="Alignment fixture"
      :columns="[{ key: 'purpose', label: 'Purpose' }, { key: 'action', label: 'Action', kind: 'action' }]">
      <template #icon><svg width="18" height="18" viewBox="0 0 20 20"><path d="M2 5h6l2 2h8v10H2Z" fill="none" stroke="currentColor" /></svg></template>
      <template #cell-purpose="{ node }"><span>{{ node.id === 'long' ? 'A long purpose that wraps to more than one line while keeping its first line aligned' : 'Purpose' }}</span></template>
      <template #cell-action><button type="button" class="btn ghost">Open directory</button></template>
    </UiTreeTable>
  </main>
</template>

<style scoped>
.fixture { width: 700px; padding: 20px; }
:deep(table) { table-layout: fixed; }
</style>
