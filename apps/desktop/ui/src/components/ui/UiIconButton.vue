<script setup lang="ts">
withDefaults(defineProps<{label:string;size?:'default'|'notification';disabled?:boolean;busy?:boolean;variant?:'default'|'danger'}>(),{size:'default',disabled:false,busy:false,variant:'default'})
</script>
<template>
 <button type="button" class="icon-btn ui-icon-control" :class="{'icon-btn-notification':size==='notification',danger:variant==='danger'}" :aria-label="label" :aria-busy="busy?'true':'false'" :disabled="disabled||busy"><slot /></button>
</template>
