<script setup lang="ts">
import { computed } from 'vue'
import UiButton from './UiButton.vue'
const props=defineProps<{modelValue?:string;defaultValue?:string;values:string[];label:string;disabled?:boolean}>()
const emit=defineEmits<{'update:modelValue':[value:string|undefined]}>()
const effective=computed(()=>props.modelValue??props.defaultValue)
const index=computed(()=>props.values.indexOf(effective.value??''))
function choose(value:string|undefined){if(!props.disabled)emit('update:modelValue',value)}
function step(direction:number){if(!props.values.length||props.disabled)return;const next=index.value<0?(direction>0?0:props.values.length-1):Math.max(0,Math.min(props.values.length-1,index.value+direction));choose(props.values[next])}
function key(event:KeyboardEvent){if(index.value>=0)return;if(['ArrowLeft','ArrowDown','ArrowRight','ArrowUp','Home','End'].includes(event.key)){event.preventDefault();const last=event.key==='End'||event.key==='ArrowLeft'||event.key==='ArrowDown';choose(props.values[last?props.values.length-1:0])}}
</script>
<template>
 <div class="ui-discrete-slider" role="group" :aria-label="label">
  <div class="ui-discrete-head"><span>{{ label }}</span><strong>{{ index>=0?effective:effective?'原值已不可用':'未知' }}</strong><UiButton size="sm" variant="ghost" :disabled="disabled||modelValue===undefined" aria-label="恢复默认推理档位" @click="choose(undefined)">↺</UiButton></div>
  <template v-if="values.length"><input class="ui-discrete-track" :class="{'is-unknown':index<0}" type="range" min="0" :max="values.length-1" step="1" :value="Math.max(0,index)" :disabled="disabled" :aria-label="label" :aria-valuetext="index>=0?effective:'未知'" @input="choose(values[Number(($event.target as HTMLInputElement).value)])" @keydown="key" @wheel.prevent="step($event.deltaY>0?-1:1)" /><div class="ui-discrete-labels"><button v-for="value in values" :key="value" type="button" :aria-pressed="value===effective" :disabled="disabled" @click="choose(value)">{{ value }}</button></div></template>
  <span v-else class="ui-discrete-unknown">未提供可选档位</span>
 </div>
</template>
<style scoped>
.ui-discrete-slider{display:grid;gap:8px;min-width:0}.ui-discrete-head{display:flex;align-items:center;gap:8px}.ui-discrete-head strong{flex:1;font-size:var(--text-caption);font-weight:var(--weight-medium)}.ui-discrete-track{width:100%;margin:8px 0;accent-color:var(--accent);cursor:pointer}.ui-discrete-labels{display:flex;justify-content:space-between;gap:4px;flex-wrap:wrap}.ui-discrete-labels button{min-height:var(--control-sm);border:0;border-radius:var(--radius-sm);padding:2px 4px;background:transparent;color:var(--muted);font:inherit;font-size:var(--text-caption);cursor:pointer}.ui-discrete-labels button[aria-pressed=true]{color:var(--text);background:var(--hover);font-weight:var(--weight-semibold)}.ui-discrete-labels button:focus-visible{outline:2px solid var(--accent);outline-offset:2px}.ui-discrete-unknown{color:var(--muted);font-size:var(--text-caption)}.ui-discrete-track.is-unknown::-webkit-slider-thumb{opacity:.25}.ui-discrete-track.is-unknown::-moz-range-thumb{opacity:.25}
</style>
