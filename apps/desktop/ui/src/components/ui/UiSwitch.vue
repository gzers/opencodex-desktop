<script setup lang="ts">
withDefaults(defineProps<{modelValue?:boolean;disabled?:boolean;label:string;busy?:boolean}>(),{modelValue:false,disabled:false,busy:false})
const emit=defineEmits<{'update:modelValue':[value:boolean]}>()
</script>
<template>
 <button class="toggle" type="button" role="switch" :aria-checked="modelValue" :aria-label="label" :aria-busy="busy?'true':undefined" :disabled="disabled||busy" @click="emit('update:modelValue',!modelValue)"></button>
</template>
