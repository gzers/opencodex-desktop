<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import { useNodeMeasurements, nodeMeasurementError } from './useNodeMeasurements'
import type { ChannelPolicy } from './api'
import type { ChannelV2, DiscoveryKindV2, ReadBaseV2 } from './api-v2'

const props = defineProps<{ channel: ChannelPolicy|ChannelV2; expected?: ReadBaseV2|null; open: boolean; disabled?: boolean; readonly?: boolean; measurementDisabled?: boolean; credentialProviderId?: string }>()
const emit = defineEmits<{ close: []; select: [id:string] }>()
// 生命周期归渠道所有，表格筛选、详情导航和关闭观察都不销毁测量。
// 只交稳定渠道/节点ID和可信草稿基准；临时方式不从旧协议推断。
const discoveryKind=ref<DiscoveryKindV2|''>('')
const measurementDisabled=computed(()=>props.measurementDisabled||!props.expected||!('models' in props.channel)||!discoveryKind.value)
const measurement = reactive(useNodeMeasurements(() => props.channel, () => '', {expected:()=>props.expected??null,discoveryKind:()=>discoveryKind.value,enabled:()=>!props.measurementDisabled}))
function text(id: string) {
  if (measurement.pending.includes(id)) return '检测中…'
  const result = measurement.results[id]
  if (result?.error) return nodeMeasurementError(result.error)
  if (result?.elapsedMs != null) return result.elapsedMs + ' ms' + (result.modelCount != null ? ' · ' + result.modelCount + ' 个模型' : '')
  return measurement.stopped ? '已停止' : '未检测'
}
function choose(id: string) {
  if (!props.open || props.disabled || props.readonly || !props.channel.connection.nodes.some(node => node.id === id)) return
  if('models' in props.channel)emit('select',id)
  else props.channel.connection.activeNodeId = id
  emit('close')
}
function measure(ids: string[]) {
  if (props.open && !props.disabled && !measurementDisabled.value) void measurement.measure(ids)
}
</script>

<template>
  <UiDialog :open="open" :title="channel.label + ' · 节点检测'" panel-class="channel-node-dialog" @close="emit('close')">
    <div class="channel-node-detection">
      <UiSelectField v-if="'models' in channel" v-model="discoveryKind" label="本次节点清单获取方式" :disabled="!!measurement.run||disabled||props.measurementDisabled" :options="[{value:'',label:'请选择本次获取方式'},{value:'openai_list',label:'兼容模型清单'},{value:'anthropic_list',label:'Anthropic 模型清单'},{value:'google_list',label:'Google 模型清单'},{value:'ollama_list',label:'Ollama 模型清单'}]" />
      <div class="channel-node-toolbar">
        <UiButton size="sm" :disabled="disabled || measurementDisabled || !!measurement.run || !channel.connection.nodes.length" @click="measure(channel.connection.nodes.map(node => node.id))">一键检测</UiButton>
        <UiButton v-if="measurement.run" size="sm" @click="measurement.stop">停止</UiButton>
        <span v-if="measurement.run" role="status">正在检测 {{ measurement.pending.length }} 个节点…</span>
        <span v-else-if="measurement.stopped" role="status">检测已停止，已完成结果保留。</span>
      </div>
      <p v-if="measurement.error" role="alert" class="model-error">{{ measurement.error }}</p>
      <div class="channel-node-list" role="group" aria-label="选择节点与单节点检测">
        <div v-for="node in channel.connection.nodes" :key="node.id" class="channel-node-option">
          <button type="button" class="channel-node-choice" :aria-label="'选择节点 ' + node.name" :aria-pressed="node.id === channel.connection.activeNodeId" :disabled="disabled || readonly" @click="choose(node.id)">
            <span class="channel-node-check" aria-hidden="true">{{ node.id === channel.connection.activeNodeId ? '✓' : '' }}</span>
            <span class="channel-node-info"><span>{{ node.name }}</span><span class="channel-node-url">{{ node.url }}</span></span>
          </button>
          <span class="channel-node-result" role="status">{{ text(node.id) }}</span>
          <UiButton size="sm" :disabled="disabled || measurementDisabled || !!measurement.run" :aria-label="'检测节点 ' + node.name" @click="measure([node.id])">检测</UiButton>
        </div>
      </div>
      <p class="channel-node-help">选择节点只修改草稿；检测不会自动选择或保存。关闭窗口后检测继续，重新打开可查看结果。所有节点共用渠道认证，耗时仅描述模型清单请求。</p>
    </div>
  </UiDialog>
</template>

<style scoped>
.channel-node-detection{min-width:0}
.channel-node-toolbar{display:flex;align-items:center;gap:var(--space-2);flex-wrap:wrap;margin-bottom:var(--space-3)}
.channel-node-toolbar>span,.channel-node-result,.channel-node-help{font-size:var(--text-caption);color:var(--muted)}
.channel-node-list{display:grid;gap:var(--space-2);min-width:0}
.channel-node-option{display:grid;grid-template-columns:minmax(0,1fr) auto auto;align-items:center;gap:var(--space-2);border-bottom:1px solid var(--border);padding-block:var(--space-2);min-width:0}
.channel-node-choice{display:flex;align-items:center;gap:var(--space-2);min-width:0;min-height:40px;padding:var(--space-2);border:0;border-radius:var(--radius);background:transparent;color:var(--text);font:inherit;font-weight:400;text-align:left;cursor:pointer}
.channel-node-choice:hover:not(:disabled){background:var(--raised)}
.channel-node-choice:focus-visible{background:var(--raised);outline:2px solid var(--accent);outline-offset:1px}
.channel-node-choice:disabled{cursor:default;opacity:.6}
.channel-node-check{width:14px;flex:none;color:var(--accent)}
.channel-node-info{display:grid;gap:var(--space-1);min-width:0;overflow-wrap:anywhere}
.channel-node-url{color:var(--muted);font-size:var(--text-caption);overflow-wrap:anywhere}
.channel-node-result{max-width:150px;overflow-wrap:anywhere}
.channel-node-help{margin-bottom:0;overflow-wrap:anywhere}
@media(max-width:600px){.channel-node-option{grid-template-columns:minmax(0,1fr) auto}.channel-node-choice{grid-column:1/-1}.channel-node-result{max-width:none}}
</style>
