<script setup lang="ts">
import {defineAsyncComponent} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import {useSyncStore} from './store'
const ConnectionDialog=defineAsyncComponent(()=>import('./ConnectionDialog.vue'))
const sync=useSyncStore()
</script>
<template><UiDialog :open="sync.resultOpen" title="原操作结果" @close="sync.closeResult()"><p v-if="sync.resultReadError" role="alert">{{sync.resultReadError}}</p><p>{{sync.resultId}}</p><template #actions><UiButton :disabled="sync.resultResolving" @click="sync.openResult()">重新读取原结果</UiButton></template></UiDialog><ConnectionDialog v-if="sync.connectionOpen" @close="sync.connectionOpen=false"/></template>
