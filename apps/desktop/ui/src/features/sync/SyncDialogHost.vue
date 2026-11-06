<script setup lang="ts">
import {defineAsyncComponent,ref,watch} from 'vue'
import ConfigSyncPreviewV2 from './ConfigSyncPreviewV2.vue'
import ConfigSyncResultV2 from './ConfigSyncResultV2.vue'
import ConfigSyncRecoveryV2 from './ConfigSyncRecoveryV2.vue'
import {useConfigSyncV2Store} from './store-v2'
import {useSyncStore} from './store'
import type {TransferResult} from './modelTransfer'
const ResultDialog=defineAsyncComponent(()=>import('./SyncResultDialog.vue'))
const ConnectionDialog=defineAsyncComponent(()=>import('./ConnectionDialog.vue'))
const ModelTransfer=defineAsyncComponent(()=>import('./ModelTransfer.vue'))
const configV2=useConfigSyncV2Store()
const sync=useSyncStore(),retry=ref<TransferResult|null>(null),refresh=ref<TransferResult|null>(null),newRemote=ref(false)
watch(()=>configV2.fact,value=>{if(!value)return;sync.rememberResult(value.operationId,true);if(configV2.factEndpointGeneration!==configV2.endpointGeneration)return;const token=sync.observationToken();if(!sync.ownsObservation(token))return;const conflicts=['conflict','initialization_ambiguous','generation_changed'].includes(value.relation.remote);const equal=value.relation.local==='observed_equal'&&value.relation.remote==='observed_equal'&&(value.kind==='upload'?value.storageStatus==='stored':value.localApplyStatus==='applied');sync.recordObservation(token,conflicts?'conflict':equal?'synced':value.failure?'failed':'connected',{source:'sync',comparison:equal?'same':null,remoteTime:null})})

function recompare(value:TransferResult){sync.closeResult();if(value.kind==='import')refresh.value=value;else{newRemote.value=true;sync.requestAction('sync')}}
function closeRefresh(){const id=refresh.value?.id;refresh.value=null;if(id)sync.openResult(id)}
function retryImport(value:TransferResult){sync.closeResult();retry.value=value}
function closeRetry(){const id=retry.value?.id;retry.value=null;if(id)sync.openResult(id)}
</script>
<template><ConfigSyncPreviewV2 :open="configV2.previewOpen" :preview="configV2.preview" :entries="configV2.catalog?.entries" :busy="configV2.previewBusy" :error="configV2.error" :stale="configV2.previewStale&&!configV2.preparing" @close="configV2.closePreview()" @direction="configV2.prepare($event)" @backup="configV2.changeBackup" @confirm="configV2.confirm()" @retry="configV2.prepare()" /><ConfigSyncResultV2 :open="configV2.resultOpen" :fact="configV2.fact" :busy="configV2.resultBusy" :error="configV2.resultError" @close="configV2.resultOpen=false" @read="configV2.readResult()" @action="configV2.recoveryAction" @cancel="configV2.stopConfirmed()" /><ConfigSyncRecoveryV2 :open="configV2.recoveryOpen" :state="configV2.catalog?.cloudState??configV2.fact?.observation?.state??'incomplete'" :heads="configV2.catalog?.heads" :actions="configV2.failure?.recoveryActions??(configV2.catalog&&['conflict','initialization_ambiguous'].includes(configV2.catalog.cloudState)?configV2.catalog.recoveryActions:configV2.fact?.recoveryActions??configV2.catalog?.recoveryActions??[])" :busy="configV2.catalogBusy" :status="configV2.recoveryStatus" :error="configV2.error" @close="configV2.recoveryOpen=false" @action="configV2.recoveryAction" @head="configV2.chooseHead" /><ResultDialog v-if="sync.resultOpen" :operation-id="sync.resultId" @close="sync.closeResult()" @retry="retryImport" @refresh="recompare"/><ConnectionDialog v-if="sync.connectionOpen" @close="sync.connectionOpen=false"/><ModelTransfer v-if="retry" action-only :remote="retry.kind==='download'" :retry-receipt="retry" @close="closeRetry" /><ModelTransfer v-if="refresh" action-only :refresh-receipt="refresh" @close="closeRefresh"/><ModelTransfer v-if="newRemote" action-only remote @close="newRemote=false"/></template>
