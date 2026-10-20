<script setup lang="ts">
import {computed,nextTick,onBeforeUnmount,reactive,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import {useNodeMeasurements,nodeMeasurementError} from './useNodeMeasurements'
import type {ChannelPolicy} from './api'
const props=defineProps<{channel:ChannelPolicy;disabled?:boolean;readonly?:boolean}>()
const open=ref(false),root=ref<HTMLElement|null>(null),trigger=ref<HTMLButtonElement|null>(null),menu=ref<HTMLElement|null>(null)
const measurement=reactive(useNodeMeasurements(()=>props.channel,()=>''))
const active=computed(()=>props.channel.connection.nodes.find(n=>n.id===props.channel.connection.activeNodeId))
function text(id:string){const result=measurement.results[id];return measurement.pending.includes(id)?'检测中':result?.error?nodeMeasurementError(result.error):result?.elapsedMs!==null&&result?.elapsedMs!==undefined?result.elapsedMs+' ms':'未检测'}
async function choose(id:string){if(props.disabled||props.readonly)return;props.channel.connection.activeNodeId=id;open.value=false;await nextTick();trigger.value?.focus({preventScroll:true})}
watch(()=>props.disabled,value=>{if(value)open.value=false})
function focusout(event:FocusEvent){const target=event.relatedTarget;if(target instanceof Node&&!root.value?.contains(target)&&!menu.value?.contains(target))open.value=false}
onBeforeUnmount(()=>{open.value=false})
</script>
<template>
 <div ref="root" class="select channel-node-select" @focusout="focusout">
  <button ref="trigger" class="select-trigger" type="button" :aria-label="channel.label+'当前节点'" :title="active?active.name+' · '+active.url:undefined" aria-haspopup="menu" :aria-expanded="open" :disabled="disabled||!channel.connection.nodes.length" @click="open=!open"><span>{{ active?.name||'未选择节点' }}<small v-if="active">{{ text(active.id) }}</small></span><span aria-hidden="true">⌄</span></button>
  <UiSelectMenu :open="open" :close-on-select="false" @close="open=false"><div ref="menu" class="channel-node-menu" role="menu" :aria-label="channel.label+'节点选择与检测'" @focusout="focusout">
   <header><strong>服务节点</strong><UiButton size="sm" :disabled="disabled||readonly||!!measurement.run" @click="measurement.measure(channel.connection.nodes.map(n=>n.id))">检测全部节点</UiButton><UiButton v-if="measurement.run" size="sm" @click="measurement.stop">停止</UiButton></header>
   <p v-if="measurement.run" role="status">正在检测 {{ measurement.pending.length }} 个节点…</p><p v-if="measurement.stopped" role="status">检测已停止，已完成结果保留。</p><p v-if="measurement.error" role="alert" class="model-error">{{ measurement.error }}</p>
   <div v-for="node in channel.connection.nodes" :key="node.id" class="channel-node-option"><button class="select-option" role="menuitemradio" :aria-checked="node.id===channel.connection.activeNodeId" :disabled="disabled||readonly" :aria-label="'选择节点 '+node.name" @click="choose(node.id)"><span>{{ node.id===channel.connection.activeNodeId?'✓ ':'' }}{{ node.name }}</span><code>{{ node.url }}</code><small>{{ text(node.id) }}</small></button><UiButton size="sm" variant="ghost" :disabled="disabled||readonly||!!measurement.run" :aria-label="'检测节点 '+node.name" @click="measurement.measure([node.id])">检测</UiButton></div>
   <p>点击节点名称才切换草稿；检测不自动选择。所有节点共用当前渠道认证。</p>
  </div></UiSelectMenu>
 </div>
</template>
<style scoped>
.channel-node-select{min-width:140px;max-width:220px}.select-trigger{width:100%;min-width:0;gap:var(--space-2)}.select-trigger>span:first-child{min-width:0;overflow:hidden;text-overflow:ellipsis}.select-trigger small{display:block;font-size:var(--text-caption);color:var(--muted)}.channel-node-menu{width:360px;max-width:100%;min-width:0}.channel-node-menu header{display:flex;align-items:center;justify-content:space-between;gap:var(--space-2);flex-wrap:wrap;margin-bottom:var(--space-2)}.channel-node-menu p{font-size:var(--text-caption);color:var(--muted);overflow-wrap:anywhere}.channel-node-option{display:flex;gap:var(--space-2);align-items:center;border-top:1px solid var(--border)}.channel-node-option>.select-option{flex:1;min-width:0;display:grid;gap:var(--space-1);white-space:normal}.select-option code{font-size:var(--text-caption);font-family:var(--font-code);overflow-wrap:anywhere;color:var(--muted)}.select-option small{font-size:var(--text-caption);color:var(--muted)}
</style>
