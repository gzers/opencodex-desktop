<script setup lang="ts">
import {computed,onMounted,watch} from 'vue'
import ConfigSyncPreviewV2 from './ConfigSyncPreviewV2.vue'
import ConfigFileTransferV2 from './ConfigFileTransferV2.vue'
import {useConfigSyncV2Store} from './store-v2'
import {useConfigFileV2Store} from './file-store-v2'
import {useSyncStore} from './store'
import type {TransferResult} from './modelTransfer'
const props=defineProps<{retryReceipt?:TransferResult;refreshReceipt?:TransferResult;source?:'saved'|'draft';remote?:boolean;importOnly?:boolean;importOpen?:boolean;actionOnly?:boolean}>()
const emit=defineEmits<{'update:importOpen':[value:boolean];close:[]}>()
const configV2=useConfigSyncV2Store(),files=useConfigFileV2Store(),syncStore=useSyncStore()
const configured=computed(()=>!!syncStore.config?.endpoint)
async function configCatalog(read:string|null=null,write:string|null=null){configV2.secrets(read,write);await configV2.loadCatalog()}
async function configPrepare(read:string|null,write:string|null){configV2.secrets(read,write);await configV2.prepare()}
async function pendingAction(){if(props.remote&&configured.value&&syncStore.takeAction('sync')){await configV2.loadCatalog();if(!configV2.error)await configV2.prepare();if(props.actionOnly)emit('close')}else if(!props.remote&&props.actionOnly&&syncStore.takeAction('export')){if(props.source)await files.setSource(props.source);files.scopeOpen=true;emit('close')}}
watch(()=>configV2.scopeEditRevision,()=>{if(props.remote&&configured.value&&configV2.catalog&&!configV2.catalogBusy)void configV2.persistScope()})
watch(()=>syncStore.pendingAction,()=>void pendingAction(),{flush:'post'})
watch(configured,async value=>{if(props.remote&&value){await configV2.loadCatalog();await pendingAction()}})
onMounted(async()=>{if(props.remote&&configured.value){await configV2.loadCatalog();await pendingAction()}else if(!props.remote)await pendingAction()})
</script>
<template><article class="model-transfer"><template v-if="remote"><slot v-if="!configured" name="connection" /><ConfigSyncPreviewV2 v-if="configured" :workspace="!actionOnly" :open="false" :preview="null" :catalog="configV2.catalog" :scope="configV2.scope" :loading="configV2.catalogBusy" :busy="configV2.previewBusy" :error="configV2.error" :scope-error="configV2.scopeError" :has-result="!!configV2.fact" @catalog="configCatalog" @prepare="configPrepare" @toggle="configV2.toggleEntry" @mcp="configV2.setMcpClient" @subset="configV2.channelSubset" @option="configV2.channelOption" @credential="configV2.credentialEntry" @revision="configV2.templateRevision" @default="configV2.adoptDefault" @global="configV2.setScope({...configV2.scope,selection:{...configV2.scope.selection,includeGlobalTemplateRefs:$event}})" @result="configV2.resultOpen=true" @recovery="configV2.recoveryOpen=true" @connection="syncStore.openConnection()"><template #connection><slot name="connection" /></template></ConfigSyncPreviewV2></template><ConfigFileTransferV2 v-else :source="source" :import-only="importOnly" :import-open="importOpen" :action-only="actionOnly" @update:import-open="emit('update:importOpen',$event)" @close="emit('close')" /></article></template>
<style scoped>.model-transfer{display:grid;align-content:start;gap:var(--space-4);min-width:0;width:100%}</style>
