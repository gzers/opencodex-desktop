<script setup lang="ts">
import UiIconButton from '@/components/ui/UiIconButton.vue'
withDefaults(defineProps<{title:string;message:string;collapseLabel?:string;showCollapse?:boolean;phase?:'running'|'done'|'error';showTrack?:boolean;steps?:{label:string;state:string}[]}>(),{collapseLabel:'收起任务卡到铃铛',showCollapse:true,phase:'running',showTrack:true,steps:()=>[]})
defineEmits<{collapse:[]}>()
</script>
<template>
 <section class="process-progress" :class="{done:phase==='done',error:phase==='error'}" role="status" aria-live="polite" aria-atomic="true">
  <div class="process-progress-head"><span v-if="phase!=='running'" class="process-progress-result" :class="phase" aria-hidden="true">{{ phase==='error'?'!':'✓' }}</span><div><h3>{{ title }}</h3><p v-if="message">{{ message }}</p></div><UiIconButton v-if="showCollapse" class="notify-min" size="notification" :label="collapseLabel" @click="$emit('collapse')"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 14h6v6"/><path d="M20 10h-6V4"/><path d="M14 10 20 4"/><path d="M10 14 4 20"/></svg></UiIconButton></div>
  <div v-if="showTrack" class="process-progress-track" aria-hidden="true"></div>
  <ol v-if="steps.length" class="process-progress-steps"><li v-for="(step,index) in steps" :key="index" class="process-progress-step" :class="step.state">{{ step.label }}</li></ol>
  <div v-if="$slots.actions" class="process-progress-actions"><slot name="actions" /></div>
 </section>
</template>
