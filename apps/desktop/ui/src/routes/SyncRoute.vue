<script setup lang="ts">
import {defineAsyncComponent, ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import Stack from '@/components/layout/Stack.vue'
import {useRouteStore} from '@/stores/routes'
import {useAppStore} from '@/stores/app'
const ModelTransfer=defineAsyncComponent(()=>import('@/features/sync/ModelTransfer.vue'))
const ImportedConnections=defineAsyncComponent(()=>import('@/features/sync/ImportedConnections.vue'))
const WebDavConnection=defineAsyncComponent(()=>import('@/features/sync/WebDavConnection.vue'))
const routes=useRouteStore();const app=useAppStore();const connectionOpen=ref(false);const remoteMounted=ref(routes.syncTab==='webdav');watch(()=>routes.syncTab,tab=>{if(tab==='webdav')remoteMounted.value=true})
</script>
<template>
 <section class="sync-route"><AppTopbar/><StickyTabs><div class="sync-tabs" role="tablist" aria-label="同步方式"><button type="button" role="tab" aria-controls="sync-file-panel" :aria-selected="routes.syncTab==='file'" :class="{active:routes.syncTab==='file'}" @click="routes.go('sync',{tab:'file'})">文件同步</button><button type="button" role="tab" aria-controls="sync-webdav-panel" :aria-selected="routes.syncTab==='webdav'" :class="{active:routes.syncTab==='webdav'}" @click="routes.go('sync',{tab:'webdav'})">WebDAV 同步</button></div></StickyTabs>
 <section v-show="routes.syncTab==='file'" id="sync-file-panel" role="tabpanel" aria-label="文件同步"><ModelTransfer/></section>
 <section v-show="routes.syncTab==='webdav'" id="sync-webdav-panel" role="tabpanel" aria-label="WebDAV同步"><Stack v-if="remoteMounted"><div class="sync-connection-summary"><div><strong>{{ app.syncConfig?.endpoint?'当前WebDAV连接':'未配置WebDAV连接' }}</strong><small v-if="app.syncConfig?.endpoint">{{ app.syncConfig.endpoint.url }}</small></div><UiButton @click="connectionOpen=true">连接设置</UiButton></div><ModelTransfer remote/><ImportedConnections @activated="app.loadSyncConfig"/></Stack><UiDialog :open="connectionOpen" title="WebDAV连接设置" wide @close="connectionOpen=false"><WebDavConnection/><template #actions><UiButton @click="connectionOpen=false">关闭</UiButton></template></UiDialog></section>
 </section>
</template>
<style scoped>
.sync-connection-summary{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}.sync-connection-summary small{display:block;color:var(--muted);overflow-wrap:anywhere}.sync-route{min-width:0}.sync-tabs{display:flex;gap:var(--space-3);flex-wrap:wrap;border-bottom:1px solid var(--border);padding-block:var(--space-2)}.sync-tabs button{border:0;border-bottom:2px solid transparent;background:transparent;color:var(--muted);font-weight:var(--weight-semibold);min-height:var(--control-md);padding:var(--space-2) var(--space-3);cursor:pointer}.sync-tabs button.active{border-color:var(--text);color:var(--text)}.sync-route [role=tabpanel]{padding-top:var(--space-4)}
</style>
