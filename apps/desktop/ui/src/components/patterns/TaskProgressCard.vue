<script setup lang="ts">
import UiIconButton from '@/components/ui/UiIconButton.vue'
withDefaults(defineProps<{title:string;message:string;collapseLabel?:string;phase?:'running'|'done'|'error';showTrack?:boolean;steps?:{label:string;state:string}[]}>(),{collapseLabel:'收起任务卡到铃铛',phase:'running',showTrack:true,steps:()=>[]})
defineEmits<{collapse:[]}>()
</script>
<template>
 <section class="process-progress" :class="{done:phase==='done',error:phase==='error'}" role="status" aria-live="polite" aria-atomic="true">
  <div class="process-progress-head"><div><h3>{{ title }}</h3><p>{{ message }}</p></div><UiIconButton class="notify-min" size="notification" :label="collapseLabel" @click="$emit('collapse')"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg></UiIconButton></div>
  <div v-if="showTrack" class="process-progress-track" aria-hidden="true"></div>
  <ol v-if="steps.length" class="process-progress-steps"><li v-for="(step,index) in steps" :key="index" class="process-progress-step" :class="step.state">{{ step.label }}</li></ol>
  <div v-if="$slots.actions" class="process-progress-actions"><slot name="actions" /></div>
 </section>
</template>
