<script setup lang="ts">
import {defineAsyncComponent,watch} from 'vue'
import ConfigSyncPreviewV2 from './ConfigSyncPreviewV2.vue'
import ConfigFileTransferV2 from './ConfigFileTransferV2.vue'
import ConfigSyncResultV2 from './ConfigSyncResultV2.vue'
import ConfigSyncRecoveryV2 from './ConfigSyncRecoveryV2.vue'
import {useConfigSyncV2Store} from './store-v2'
import {useSyncStore} from './store'
import {useConfigFileV2Store} from './file-store-v2'
const ConnectionDialog=defineAsyncComponent(()=>import('./ConnectionDialog.vue'))
const configV2=useConfigSyncV2Store(),files=useConfigFileV2Store(),sync=useSyncStore()
watch(()=>files.receipt,value=>{if(value)sync.rememberResult(value.operationId,false)})
watch(()=>configV2.fact,value=>{if(!value)return;sync.rememberResult(value.operationId,true);if(configV2.factEndpointGeneration!==configV2.endpointGeneration)return;const token=sync.observationToken();if(!sync.ownsObservation(token))return;const conflicts=['conflict','initialization_ambiguous','generation_changed'].includes(value.relation.remote);const equal=value.relation.local==='observed_equal'&&value.relation.remote==='observed_equal'&&(value.kind==='upload'?value.storageStatus==='stored':value.localApplyStatus==='applied');sync.recordObservation(token,conflicts?'conflict':equal?'synced':value.failure?'failed':'connected',{source:'sync',comparison:equal?'same':null,remoteTime:null})})

</script>
<template><ConfigFileTransferV2 :workspace="false" dialog-owner /><ConfigSyncPreviewV2 :open="configV2.previewOpen" :preview="configV2.preview" :entries="configV2.catalog?.entries" :busy="configV2.previewBusy" :error="configV2.error" :stale="configV2.previewStale&&!configV2.preparing" @close="configV2.closePreview()" @direction="configV2.prepare($event)" @backup="configV2.changeBackup" @confirm="configV2.confirm()" @retry="configV2.prepare()" /><ConfigSyncResultV2 :open="configV2.resultOpen" :fact="configV2.fact" :busy="configV2.resultBusy" :error="configV2.resultError" :preflight="configV2.resumePreflight" @resume="configV2.confirmResume()" @dismiss-resume="configV2.resumePreflight=null" @receive="configV2.resumeReceiveOriginal()" @close="configV2.resultOpen=false" @read="configV2.readResult()" @action="configV2.recoveryAction" @cancel="configV2.stopConfirmed()" /><ConfigSyncRecoveryV2 :open="configV2.recoveryOpen" :state="configV2.catalog?.cloudState??configV2.fact?.observation?.state??'incomplete'" :heads="configV2.catalog?.heads" :actions="configV2.failure?.recoveryActions??(configV2.catalog&&['conflict','initialization_ambiguous'].includes(configV2.catalog.cloudState)?configV2.catalog.recoveryActions:configV2.fact?.recoveryActions??configV2.catalog?.recoveryActions??[])" :busy="configV2.catalogBusy||configV2.recoveryBusy" :preview="configV2.recoveryPreview" :receipt="configV2.recoveryReceipt" :stale="configV2.recoveryPlanStale" @confirm="configV2.confirmRecovery()" @read="configV2.readRecovery()" :status="configV2.recoveryStatus" :error="configV2.error" @close="configV2.closeRecovery()" @action="configV2.recoveryAction" @head="configV2.chooseHead" /><ConnectionDialog v-if="sync.connectionOpen" @close="sync.connectionOpen=false"/></template>
