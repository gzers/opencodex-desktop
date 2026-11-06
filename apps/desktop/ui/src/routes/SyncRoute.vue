<script setup lang="ts">
import {defineAsyncComponent,onBeforeUnmount,onMounted,ref,watch} from 'vue'
import {layoutObserver} from '@/lib/layoutObserver'
import UiButton from '@/components/ui/UiButton.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import {useRouteStore} from '@/stores/routes'
import {useAppStore} from '@/stores/app'
import {useSyncStore} from '@/features/sync/store'
import {connectionAddress} from '@/features/sync/observation'
const ModelTransfer=defineAsyncComponent(()=>import('@/features/sync/ModelTransfer.vue'))

const routes=useRouteStore();const app=useAppStore();const remoteMounted=ref(routes.syncTab==='webdav');watch(()=>routes.syncTab,tab=>{if(tab==='webdav')remoteMounted.value=true})
const page=ref<HTMLElement|null>(null),shortHeight=ref(false)
let heightObserver:ReturnType<typeof layoutObserver>
function measureHeight(){const main=page.value?.closest<HTMLElement>('.main');if(!main)return;const style=getComputedStyle(main);shortHeight.value=main.clientHeight-(parseFloat(style.paddingTop)||0)-(parseFloat(style.paddingBottom)||0)<=600}
onMounted(()=>{const main=page.value?.closest<HTMLElement>('.main');measureHeight();heightObserver=layoutObserver(measureHeight);if(main)heightObserver?.observe(main);window.addEventListener('resize',measureHeight)})
onBeforeUnmount(()=>{heightObserver?.disconnect();window.removeEventListener('resize',measureHeight)})
// S18-01：概览 WebDAV 卡片「配置 WebDAV / 重试连接」进入同步页后直接打开连接设置。
const syncStore=useSyncStore();watch(()=>syncStore.pendingAction,()=>{if(syncStore.takeAction('connection')){remoteMounted.value=true;syncStore.openConnection()}},{immediate:true})
</script>
<template>
 <section ref="page" class="sync-route" :class="{'sync-short-height':shortHeight}"><AppTopbar/><StickyTabs><div class="sync-tabs" role="tablist" aria-label="同步方式"><button type="button" role="tab" aria-controls="sync-file-panel" :aria-selected="routes.syncTab==='file'" :class="{active:routes.syncTab==='file'}" @click="routes.go('sync',{tab:'file'})">文件同步</button><button type="button" role="tab" aria-controls="sync-webdav-panel" :aria-selected="routes.syncTab==='webdav'" :class="{active:routes.syncTab==='webdav'}" @click="routes.go('sync',{tab:'webdav'})">WebDAV 同步</button></div></StickyTabs>
 <section v-show="routes.syncTab==='file'" id="sync-file-panel" role="tabpanel" aria-label="文件同步"><ModelTransfer/></section>
 <section v-show="routes.syncTab==='webdav'" id="sync-webdav-panel" role="tabpanel" aria-label="WebDAV同步"><ModelTransfer v-if="remoteMounted" remote><template #connection><div class="sync-connection-summary"><div><strong>{{ app.syncConfig?.endpoint?'当前WebDAV连接':'未配置WebDAV连接' }}</strong><small v-if="app.syncConfig?.endpoint">{{ connectionAddress(app.syncConfig.endpoint.url, app.syncConfig.endpoint.remotePath) }}</small></div><UiButton @click="syncStore.openConnection()">连接设置</UiButton></div></template></ModelTransfer></section>
 </section>
</template>
<style scoped>
.sync-connection-summary{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}.sync-connection-summary small{display:block;color:var(--muted);overflow-wrap:anywhere}.sync-route{min-width:0;min-height:0;height:100%;display:flex;flex-direction:column}.sync-route>[role=tabpanel]{display:flex;flex:1;min-height:0}.sync-route :deep(.sticky-page-tabs){flex:none}.sync-tabs{display:flex;gap:var(--space-3);flex-wrap:wrap;border-bottom:1px solid var(--border);padding-block:var(--space-2)}.sync-tabs button{border:0;border-bottom:2px solid transparent;background:transparent;color:var(--muted);font-weight:var(--weight-semibold);min-height:var(--control-md);padding:var(--space-2) var(--space-3);cursor:pointer}.sync-tabs button.active{border-color:var(--text);color:var(--text)}.sync-route [role=tabpanel]{padding-top:12px}
:global(.app-window[data-compact="true"] .sync-route){overflow:auto}:global(.app-window[data-compact="true"] .sync-route>[role="tabpanel"]){flex-shrink:0;min-height:400px}
/* Frozen sync-config.css:100 uses logical content height. Native WKWebView's
   zoomed container-query units differ, so the layout observer supplies it. */
.sync-short-height :deep(.topbar){margin-bottom:12px}
.sync-short-height :deep(.page-title p),.sync-short-height :deep(.model-transfer>.card-head>div>p){display:none}
.sync-short-height :deep(.model-transfer){gap:8px}
.sync-short-height :deep(.sync-credential-block){padding-block:10px}
</style>
