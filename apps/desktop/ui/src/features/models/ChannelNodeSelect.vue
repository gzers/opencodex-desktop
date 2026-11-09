<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import ChannelNodeDialog from './ChannelNodeDialog.vue'
import type { ChannelPolicy } from './api'
import type { ChannelV2, ReadBaseV2 } from './api-v2'

const props = withDefaults(defineProps<{ channel: ChannelPolicy|ChannelV2; expected?: ReadBaseV2|null; disabled?: boolean; readonly?: boolean; active?: boolean; measurementDisabled?: boolean; credentialProviderId?: string; open?: boolean; showTrigger?: boolean }>(), { active: true, measurementDisabled: undefined, open: undefined, showTrigger: true })
const emit = defineEmits<{ close: []; select: [id:string] }>()
const internalOpen = ref(false)
const open = computed(() => props.open ?? internalOpen.value)
function close() { internalOpen.value = false; emit('close') }
watch(() => [props.disabled, props.active], () => { if (open.value && (props.disabled || !props.active)) close() })
</script>
<template>
  <UiButton v-if="showTrigger" size="sm" class="channel-node-detect" :aria-label="'检测 ' + channel.label + ' 服务节点'" aria-haspopup="dialog" :aria-expanded="open" :disabled="disabled || !active || !channel.connection.nodes.length" @click="internalOpen = true">检测</UiButton>
  <!-- 独立展示沿用原只读示例的无IPC合同；允许检测须明确 measurementDisabled=false。 -->
  <ChannelNodeDialog :channel="channel" :expected="expected" :open="open" :disabled="disabled" :readonly="readonly" :measurement-disabled="measurementDisabled ?? readonly" :credential-provider-id="credentialProviderId" @close="close" @select="emit('select',$event)" />
</template>
