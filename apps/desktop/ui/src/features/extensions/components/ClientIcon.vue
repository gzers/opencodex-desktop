<script setup lang="ts">
import { computed } from 'vue'
import { clientIcons } from '../clientIcons'
import type { ExtensionClientId } from '@/features/extensions/api'

const props = defineProps<{ client: ExtensionClientId }>()
const icon = computed(() => clientIcons[props.client])
</script>

<template>
  <svg class="target-svg" :viewBox="icon.viewBox" aria-hidden="true" focusable="false">
    <path
      v-for="(path, index) in icon.paths"
      :key="index"
      :d="path.d"
      :clip-rule="path.clipRule"
      :fill-rule="path.fillRule"
    />
  </svg>
</template>
