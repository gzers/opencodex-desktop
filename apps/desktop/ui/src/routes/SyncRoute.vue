<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue'
import {layoutObserver} from '@/lib/layoutObserver'
import UiButton from '@/components/ui/UiButton.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import {useRouteStore} from '@/stores/routes'
import {useAppStore} from '@/stores/app'
import {useSyncStore} from '@/features/sync/store'
import {connectionAddress} from '@/features/sync/observation'
import {webdavStates} from '@/features/sync/states'
import {clockText} from '@/features/sync/activity'

// Desktop 只同步自己的独立内容：桌面偏好与扩展设置。
// 渠道、模型、密钥和 OpenCodex 原生配置由 OpenCodex 自己管理。
const routes=useRouteStore(),app=useAppStore(),sync=useSyncStore()
const page=ref<HTMLElement|null>(null),shortHeight=ref(false)
let heightObserver:ReturnType<typeof layoutObserver>
function measureHeight(){const main=page.value?.closest<HTMLElement>('.main');if(!main)return;const style=getComputedStyle(main);shortHeight.value=main.clientHeight-(parseFloat(style.paddingTop)||0)-(parseFloat(style.paddingBottom)||0)<=600}
onMounted(()=>{const main=page.value?.closest<HTMLElement>('.main');measureHeight();heightObserver=layoutObserver(measureHeight);if(main)heightObserver?.observe(main);window.addEventListener('resize',measureHeight)})
onBeforeUnmount(()=>{heightObserver?.disconnect();window.removeEventListener('resize',measureHeight)})
watch(()=>sync.pendingAction,()=>{if(sync.takeAction('connection'))sync.openConnection()},{immediate:true})
const state=computed(()=>webdavStates[app.webdavState])
const observation=computed(()=>{const value=sync.observation;if(!value)return '尚未检查连接';const time=clockText(value.observedAt);return state.value.conn+(time?' · '+time:'')})
const busy=computed(()=>sync.running||['connecting','syncing'].includes(app.webdavState))
async function run(){const token=sync.observationToken();sync.beginObservation(token,'syncing');await app.runSyncNow()}
</script>
<template>
 <section ref="page" class="sync-route" :class="{'sync-short-height':shortHeight}"><AppTopbar/><StickyTabs><div class="sync-tabs" role="tablist" aria-label="同步方式"><button type="button" role="tab" aria-controls="sync-file-panel" :aria-selected="routes.syncTab==='file'" :class="{active:routes.syncTab==='file'}" @click="routes.go('sync',{tab:'file'})">文件同步</button><button type="button" role="tab" aria-controls="sync-webdav-panel" :aria-selected="routes.syncTab==='webdav'" :class="{active:routes.syncTab==='webdav'}" @click="routes.go('sync',{tab:'webdav'})">WebDAV 同步</button></div></StickyTabs>
 <section v-show="routes.syncTab==='file'" id="sync-file-panel" role="tabpanel" aria-label="文件同步">
  <article class="card"><div class="card-head"><div><h2>文件同步</h2><p>只同步本管理器自己的桌面偏好与扩展设置。</p></div></div>
   <dl class="sync-facts"><dt>同步内容</dt><dd>桌面偏好、Skills 与 MCP 扩展设置</dd><dt>不包含</dt><dd>渠道、模型、协议、密钥和 OpenCodex 原生配置；这些由 OpenCodex 自己管理。</dd></dl>
   <p class="sync-note">跨设备搬移模型与渠道请在 OpenCodex 官方界面中完成；Desktop 不再读取、导入、回退或迁移它的配置文件。</p>
  </article>
 </section>
 <section v-show="routes.syncTab==='webdav'" id="sync-webdav-panel" role="tabpanel" aria-label="WebDAV同步">
  <article class="card"><div class="card-head"><div><h2>WebDAV 同步</h2><p>上传或取回本管理器的独立内容；不触碰 OpenCodex 的渠道与模型配置。</p></div></div>
   <div v-if="app.syncConfig?.endpoint" class="sync-connection-summary"><div><strong>{{ app.syncConfig.endpoint.username }}</strong><small>{{ connectionAddress(app.syncConfig.endpoint.url, app.syncConfig.endpoint.remotePath) }}</small></div><UiButton @click="sync.openConnection()">连接设置</UiButton></div>
   <div v-else class="sync-connection-summary"><div><strong>未配置 WebDAV 连接</strong><small>配置后才能上传或取回桌面偏好与扩展设置。</small></div><UiButton @click="sync.openConnection()">连接设置</UiButton></div>
   <dl class="sync-facts"><dt>连接状态</dt><dd>{{ observation }}</dd><dt>最近结果</dt><dd>{{ app.syncStatus?.message || '尚无同步结果' }}</dd></dl>
   <div class="sync-actions"><UiButton variant="primary" :disabled="busy||!app.syncConfig?.endpoint" :loading="sync.running" @click="run">{{ sync.running ? '同步中…' : '同步' }}</UiButton><UiButton :disabled="busy" @click="sync.testConnection()">测试连接</UiButton></div>
  </article>
 </section>
 </section>
</template>
<style scoped>
.sync-connection-summary{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}.sync-connection-summary small{display:block;color:var(--muted);overflow-wrap:anywhere}.sync-route{min-width:0;min-height:0;height:100%;display:flex;flex-direction:column}.sync-route>[role=tabpanel]{display:flex;flex:1;min-height:0}.sync-route :deep(.sticky-page-tabs){flex:none}.sync-tabs{display:flex;gap:var(--space-3);flex-wrap:wrap;border-bottom:1px solid var(--border);padding-block:var(--space-2)}.sync-tabs button{border:0;border-bottom:2px solid transparent;background:transparent;color:var(--muted);font-weight:var(--weight-semibold);min-height:var(--control-md);padding:var(--space-2) var(--space-3);cursor:pointer}.sync-tabs button.active{border-color:var(--text);color:var(--text)}.sync-route [role=tabpanel]{padding-top:12px}.sync-facts{display:grid;grid-template-columns:7em minmax(0,1fr);gap:var(--space-2) var(--space-4);margin:var(--space-4) 0}.sync-facts dt{color:var(--muted)}.sync-facts dd{margin:0;overflow-wrap:anywhere}.sync-note{color:var(--muted);line-height:1.7;overflow-wrap:anywhere}.sync-actions{display:flex;gap:var(--space-2);flex-wrap:wrap;margin-top:var(--space-4)}
:global(.app-window[data-compact="true"] .sync-route){overflow:auto}:global(.app-window[data-compact="true"] .sync-route>[role="tabpanel"]){flex-shrink:0;min-height:400px}
.sync-short-height :deep(.topbar){margin-bottom:12px}
.sync-short-height :deep(.page-title p){display:none}
</style>
