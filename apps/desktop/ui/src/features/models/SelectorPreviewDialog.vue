<script setup lang="ts">
/**
 * 选择器预览：展示“计划发布的目录长什么样”，不提供勾选。
 *
 * 原型里这个入口只读：切换草稿计划 / 已保存配置、看激活预算、按渠道看完整顺序。
 * 早先误接成公共模型选择器，用户点“预览”却得到又一个可选项列表，与原型不符。
 */
import { computed } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import type { DraftProjection } from './api'
const props = defineProps<{
  open: boolean
  source: 'draft' | 'saved'
  projection: DraftProjection | null
  channels: { id: string; label: string; enabled?: boolean }[]
  busy?: boolean
  error?: string
}>()
const emit = defineEmits<{ close: []; 'update:source': [value: 'draft' | 'saved'] }>()
const rows = computed(() => props.projection?.rows ?? [])
const fastCount = computed(() => rows.value.filter(row => row.rules.fast.mode === 'custom' && row.rules.fast.value).length)
const active = computed(() => props.projection?.knownActiveModels ?? rows.value.length)
const over = computed(() => active.value > 100)
const catalog = computed(() => props.projection?.exactCatalogCount)
// 按策略里的渠道顺序分组；组内保持投影顺序，即选择器实际顺序。
// 停用渠道保留已选型号，但不发布目录项，因此不出现在预览里（与原型一致）。
const groups = computed(() => {
  const published = props.channels.filter(channel => channel.enabled !== false)
  const ordered = published.map(channel => channel.id)
  const ids = [...new Set(ordered)]
  let index = 0
  return ids
    .map(id => ({
      id,
      label: published.find(channel => channel.id === id)?.label ?? id,
      items: rows.value
        .filter(row => row.channelId === id)
        .map(row => ({ key: row.channelId + '/' + row.modelId, label: row.label || row.modelId, fast: row.rules.fast.mode === 'custom' && row.rules.fast.value === true, order: ++index })),
    }))
    .filter(group => group.items.length)
})
</script>
<template>
 <UiDialog :open="open" title="预览选择" wide @close="emit('close')">
  <div class="selector-preview" aria-label="预览选择">
   <div class="selector-preview-mode"><UiSelectField :model-value="source" label="预览来源" :options="[{value:'draft',label:'草稿计划'},{value:'saved',label:'已保存配置'}]" :disabled="busy" @update:model-value="emit('update:source',$event as 'draft'|'saved')" /></div>
   <p v-if="busy" role="status">正在解析当前计划…</p>
   <p v-if="error" class="model-error" role="alert">{{ error }}</p>
   <div class="selector-preview-budget" :class="{over}" role="status">
    <strong>模型激活数 {{ active }}/100</strong>
    <span>基础模型 {{ rows.length }} · 支持 Fast {{ fastCount }} · 目录占用 {{ catalog ?? '未提供' }}</span>
    <span v-if="over" class="model-error">超出激活预算，请减少发布项后再保存。</span>
   </div>
   <details v-if="groups.length" class="selector-preview-counts"><summary>按渠道查看数量</summary>
    <div class="selector-preview-count-row" v-for="group in groups" :key="group.id"><strong>{{ group.label }}</strong><span>{{ group.items.length }} 项</span></div>
   </details>
   <div class="selector-preview-order" aria-label="完整选择器顺序">
    <section v-for="group in groups" :key="group.id" class="selector-preview-group"><h3>{{ group.label }}</h3>
     <div class="selector-preview-items"><span v-for="item in group.items" :key="item.key" class="selector-preview-item" :class="{fast:item.fast}" :title="item.key"><em>{{ String(item.order).padStart(2,'0') }}</em><span>{{ item.label }}</span></span></div>
    </section>
    <p v-if="!groups.length" class="model-muted">没有计划发布的目录条目。</p>
   </div>
   <p v-for="notice in projection?.notices ?? []" :key="notice" class="model-muted">{{ notice }}</p>
   <p class="model-muted">渠道顺序 × 内部模型顺序；停用渠道不发布目录项。预览不改变配置或实际选择。</p>
  </div>
  <template #actions><UiButton variant="primary" @click="emit('close')">关闭</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.selector-preview{display:grid;gap:var(--space-4);min-width:0}
.selector-preview-mode{display:flex;gap:var(--space-2);align-items:center}
.selector-preview-budget{display:grid;gap:var(--space-1);padding:var(--space-3) var(--space-4);border:1px solid var(--border);border-radius:var(--radius-sm);font-size:var(--text-label);color:var(--muted)}
.selector-preview-budget strong{color:var(--text);font-size:var(--text-control)}
.selector-preview-budget.over{border-color:color-mix(in srgb,var(--red) 40%,transparent)}
.selector-preview-counts{border:1px solid var(--border);border-radius:var(--radius-sm);padding:var(--space-2) var(--space-3);font-size:var(--text-label)}
.selector-preview-counts summary{cursor:pointer;color:var(--muted)}
.selector-preview-count-row{display:flex;justify-content:space-between;gap:var(--space-3);padding:var(--space-1) 0;border-bottom:1px solid var(--border-soft)}
.selector-preview-count-row:last-child{border-bottom:0}
.selector-preview-order{display:grid;gap:var(--space-3);min-width:0}
.selector-preview-group h3{margin:0 0 var(--space-2);font-size:var(--text-label);color:var(--muted)}
.selector-preview-items{display:flex;flex-wrap:wrap;gap:var(--space-1-5)}
.selector-preview-item{display:inline-flex;align-items:center;gap:var(--space-1-5);max-width:100%;padding:var(--space-1) var(--space-2);border-radius:var(--radius-xs);background:var(--raised);font-size:var(--text-caption)}
.selector-preview-item em{color:var(--muted);font-style:normal;font-variant-numeric:tabular-nums}
.selector-preview-item span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.selector-preview-item.fast{background:var(--accent-soft);color:var(--text)}
.model-error{color:var(--red);overflow-wrap:anywhere}
</style>
