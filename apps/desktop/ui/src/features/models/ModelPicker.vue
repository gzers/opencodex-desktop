<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import type { ModelProjection } from './api'
export interface ModelChoice { channelId: string; modelId: string; reasoningEffort?: string; fast?: boolean }
const props = defineProps<{ open: boolean; rows: ModelProjection[]; channels: { id: string; label: string }[]; multiple?: boolean; reasoning?: boolean; current?: ModelChoice[]; title?: string }>()
const emit = defineEmits<{ close: []; confirm: [choices: ModelChoice[]] }>()
const search = ref(''); const channel = ref(''); const picks = ref<ModelChoice[]>([]); const focused = ref('')
const key = (row: { channelId: string; modelId: string }) => JSON.stringify([row.channelId, row.modelId])
watch(() => props.open, value => { if (value) { picks.value = props.current?.map(item => ({ ...item })) ?? []; search.value = ''; channel.value = ''; focused.value = '' } }, { immediate: true })
const rows = computed(() => props.rows.filter(row => (!channel.value || row.channelId === channel.value) && `${row.label} ${row.modelId}`.toLowerCase().includes(search.value.toLowerCase())))
const chosen = (row: ModelProjection) => picks.value.some(item => key(item) === key(row))
function toggle(row: ModelProjection) {
  if (!row.enabled || !row.discovered) return
  if (chosen(row)) { picks.value = picks.value.filter(item => key(item) !== key(row)); return }
  if (!props.multiple) picks.value = []
  picks.value.push({ channelId: row.channelId, modelId: row.modelId })
  focused.value = key(row)
}
function option(row: ModelProjection, field: 'reasoningEffort' | 'fast', value?: string | boolean) {
  if (!chosen(row)) toggle(row)
  const choice = picks.value.find(item => key(item) === key(row)); if (!choice) return
  if (value === undefined) { delete choice[field]; return }
  if (field === 'fast' && typeof value === 'boolean') choice.fast = value
  if (field === 'reasoningEffort' && typeof value === 'string') choice.reasoningEffort = value
}
function closeOptions(event: FocusEvent) {
  const target = event.currentTarget as HTMLElement
  if (!target.contains(event.relatedTarget as Node | null)) focused.value = ''
}
</script>
<template>
  <UiDialog :open="open" :title="title ?? '选择渠道模型'" wide @close="emit('close')">
    <div class="model-picker"><nav aria-label="渠道"><UiButton :variant="!channel ? 'primary' : 'ghost'" @click="channel = ''">全部渠道</UiButton><UiButton v-for="c in channels" :key="c.id" :variant="channel === c.id ? 'primary' : 'ghost'" @click="channel = c.id">{{ c.label }}</UiButton></nav><div><input class="input" v-model="search" type="search" aria-label="搜索模型" placeholder="搜索型号或原始ID" /><div class="model-picker-list" role="group" aria-label="可选模型"><article v-for="row in rows" :key="key(row)" class="model-picker-row">
      <label><input :type="multiple ? 'checkbox' : 'radio'" name="model-picker-selection" :checked="chosen(row)" :disabled="!row.enabled || !row.discovered" @change="toggle(row)" /><span><strong>{{ row.label }}</strong><small>{{ row.modelId }}{{ !row.enabled ? ' · 渠道停用' : !row.discovered ? ' · 未发现' : '' }}</small></span></label>
      <div v-if="reasoning" class="select"><UiButton class="select-trigger" size="sm" :disabled="!row.enabled || !row.discovered" @click="focused = focused === key(row) ? '' : key(row)">推理 / Fast</UiButton>
      <UiSelectMenu :open="focused === key(row)" :close-on-select="false" role="group" aria-label="推理与Fast" @close="focused = ''"><div class="model-picker-options" tabindex="-1" @focusout="closeOptions($event)" @keydown.esc.stop.prevent="focused = ''">
        <div class="model-inline"><span>推理</span><UiButton size="sm" @click="option(row, 'reasoningEffort')">默认</UiButton><template v-if="row.rules.reasoningEfforts.mode === 'custom'"><UiButton v-for="effort in row.rules.reasoningEfforts.value" :key="effort" size="sm" :variant="picks.find(p => key(p) === key(row))?.reasoningEffort === effort ? 'primary' : 'ghost'" @click="option(row, 'reasoningEffort', effort)">{{ effort }}</UiButton></template><span v-else class="model-muted">可选档位未知</span></div>
        <div class="model-inline"><span>Fast</span><UiButton size="sm" @click="option(row, 'fast')">默认</UiButton><UiButton size="sm" :variant="picks.find(p => key(p) === key(row))?.fast === false ? 'primary' : 'ghost'" @click="option(row, 'fast', false)">关闭</UiButton><UiButton size="sm" :disabled="row.rules.fast.mode !== 'custom' || !row.rules.fast.value" :variant="picks.find(p => key(p) === key(row))?.fast === true ? 'primary' : 'ghost'" @click="option(row, 'fast', true)">开启</UiButton></div>
      </div></UiSelectMenu></div>
    </article><p v-if="!rows.length">没有可选模型</p></div></div></div>
    <div class="model-picker-summary"><span v-for="pick in picks" :key="key(pick)" class="tag">{{ pick.modelId }}{{ pick.reasoningEffort ? ` · ${pick.reasoningEffort}` : '' }}{{ pick.fast === undefined ? '' : pick.fast ? ' · Fast开' : ' · Fast关' }}</span></div>
    <template #actions><span>已选 {{ picks.length }} 项</span><UiButton @click="emit('close')">取消</UiButton><UiButton variant="primary" :disabled="!picks.length" @click="emit('confirm', picks.map(item => ({ ...item })))">确定选择</UiButton></template>
  </UiDialog>
</template>
<style scoped>
.model-picker{display:grid;grid-template-columns:150px minmax(0,1fr);gap:20px}.model-picker nav{display:grid;gap:6px;align-content:start}.model-picker input[type=search]{width:100%;margin-bottom:12px}.model-picker-list{max-height:440px;overflow:auto}.model-picker-row{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:12px;padding:12px 0;border-bottom:1px solid var(--border)}.model-picker-row>label{display:flex;align-items:center;gap:10px;min-width:0}.model-picker-row small{display:block;color:var(--muted);overflow-wrap:anywhere}.model-picker-options{display:grid;gap:12px;width:100%;background:var(--surface-panel-bg,var(--panel));padding:12px;border-radius:var(--radius)}.model-picker-summary{display:flex;flex-wrap:wrap;gap:6px;margin-top:16px}@media(max-width:700px){.model-picker{grid-template-columns:minmax(0,1fr)}.model-picker nav{display:flex;flex-wrap:wrap}}
</style>
