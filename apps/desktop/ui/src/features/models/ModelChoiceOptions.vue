<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import UiDiscreteSlider from '@/components/ui/UiDiscreteSlider.vue'
import type { ModelProjection } from './api'
import type { ModelChoice } from './ModelPicker.vue'
const props=defineProps<{row:ModelProjection;choice?:ModelChoice;open:boolean;anotherOpen?:boolean;label:string;disabled?:boolean}>()
const emit=defineEmits<{open:[];close:[];select:[];revert:[];option:[field:'reasoningEffort'|'fast',value?:string|boolean]}>()
const panel=ref<HTMLElement|null>(null)
const trigger=ref<HTMLElement|null>(null)
const levels=computed(()=>props.row.rules.reasoningEfforts.mode==='custom'?[...props.row.rules.reasoningEfforts.value].sort((a,b)=>['none','minimal','low','medium','high','xhigh','max','ultra'].indexOf(a)-['none','minimal','low','medium','high','xhigh','max','ultra'].indexOf(b)):[])
let enter:ReturnType<typeof setTimeout>|undefined
let leave:ReturnType<typeof setTimeout>|undefined
function clear(){if(enter)clearTimeout(enter);if(leave)clearTimeout(leave)}
function show(immediate=false){clear();if(props.disabled)return;if(immediate||props.anotherOpen)emit('open');else enter=setTimeout(()=>emit('open'),70)}
function exit(){clear();leave=setTimeout(()=>{const focused=document.activeElement;if(!panel.value?.contains(focused)&&!trigger.value?.contains(focused))emit('close')},220)}
function focusOut(event:FocusEvent){const next=event.relatedTarget as Node|null;if(!panel.value?.contains(next)&&!trigger.value?.contains(next))emit('close')}
onBeforeUnmount(clear)
</script>
<template>
 <span ref="trigger" class="select model-choice-trigger" @pointerenter="show()" @pointerleave="exit" @focusout="focusOut">
  <UiButton class="select-trigger" size="sm" :disabled="disabled" :aria-expanded="open" @click="emit('select');show(true)" @focus="show(true)">{{ label }}</UiButton>
  <UiSelectMenu :open="open" :close-on-select="false" :manage-focus="false" role="group" aria-label="推理与Fast" @close="emit('close')"><div ref="panel" class="model-choice-options" tabindex="-1" @pointerenter="clear" @pointerleave="exit" @focusin="clear" @focusout="focusOut" @keydown.esc.stop.prevent="emit('revert')">
   <UiDiscreteSlider :model-value="choice?.reasoningEffort" :default-value="row.rules.defaultEffort.mode==='custom'?row.rules.defaultEffort.value:undefined" :values="levels" label="推理档位" :disabled="disabled" @update:model-value="emit('option','reasoningEffort',$event)" />
   <div class="model-choice-fast"><label><input type="checkbox" :checked="choice?.fast===true" :disabled="disabled||row.rules.fast.mode!=='custom'||!row.rules.fast.value" @change="emit('option','fast',($event.target as HTMLInputElement).checked)" />快速模式</label><span>{{ row.rules.fast.mode!=='custom'?'能力未知':!row.rules.fast.value?'不支持':choice?.fast===undefined?'未指定':choice.fast?'开启':'关闭' }}</span><UiButton size="sm" variant="ghost" aria-label="恢复默认快速模式" :disabled="disabled||choice?.fast===undefined" @click="emit('option','fast')">↺</UiButton></div>
  </div></UiSelectMenu>
 </span>
</template>
<style scoped>
.model-choice-trigger{display:inline-block;max-width:100%}.model-choice-options{width:268px;max-width:100%;padding:12px;display:grid;gap:14px;background:var(--surface-panel-bg,var(--panel));font-size:var(--text-caption);border-radius:var(--radius)}.model-choice-fast{display:flex;align-items:center;gap:8px}.model-choice-fast label{display:flex;align-items:center;gap:6px;flex:1}.model-choice-fast>span{color:var(--muted)}
</style>
