<script setup lang="ts">
import {defineAsyncComponent,ref} from 'vue'
import {useSyncStore} from './store'
import type {TransferResult} from './modelTransfer'
const ResultDialog=defineAsyncComponent(()=>import('./SyncResultDialog.vue'))
const ConnectionDialog=defineAsyncComponent(()=>import('./ConnectionDialog.vue'))
const ModelTransfer=defineAsyncComponent(()=>import('./ModelTransfer.vue'))
const sync=useSyncStore(),retry=ref<TransferResult|null>(null),refresh=ref<TransferResult|null>(null),newRemote=ref(false)
function recompare(value:TransferResult){sync.closeResult();if(value.kind==='import')refresh.value=value;else{newRemote.value=true;sync.requestAction('sync')}}
function closeRefresh(){const id=refresh.value?.id;refresh.value=null;if(id)sync.openResult(id)}
function retryImport(value:TransferResult){sync.closeResult();retry.value=value}
function closeRetry(){const id=retry.value?.id;retry.value=null;if(id)sync.openResult(id)}
</script>
<template><ResultDialog v-if="sync.resultOpen" :operation-id="sync.resultId" @close="sync.closeResult()" @retry="retryImport" @refresh="recompare"/><ConnectionDialog v-if="sync.connectionOpen" @close="sync.connectionOpen=false"/><ModelTransfer v-if="retry" action-only :remote="retry.kind==='download'" :retry-receipt="retry" @close="closeRetry" /><ModelTransfer v-if="refresh" action-only :refresh-receipt="refresh" @close="closeRefresh"/><ModelTransfer v-if="newRemote" action-only remote @close="newRemote=false"/></template>
