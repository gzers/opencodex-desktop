<script setup lang="ts">
import {ref,watch,onMounted,onBeforeUnmount} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import {useAppStore} from '@/stores/app'
import type {SyncConflictPolicy} from './api'
const app=useAppStore()
const SYNC_CONFLICT_POLICY_ASK: SyncConflictPolicy = 'ask'
const syncEndpointInput = ref({
  baseUrl: '',
  remotePath: '',
  username: '',
  password: '',
  // 冲突策略只有「每次询问」一种真实行为（引擎不读四档），界面固定只读；见下方 watch 注释。
  conflictPolicy: SYNC_CONFLICT_POLICY_ASK,
})
watch(() => app.syncConfig?.endpoint, endpoint => {
  if (!endpoint) {
    // 未配置端点：空表单。冲突策略固定「每次询问」，不再由「同步冲突策略」偏好驱动——
    // 该偏好四档从未在同步引擎落码，让它左右表单等于把无效果的承诺摆到界面上（§26.1）。
    syncEndpointInput.value.conflictPolicy = SYNC_CONFLICT_POLICY_ASK
    return
  }
  syncEndpointInput.value = {
    baseUrl: endpoint.url,
    remotePath: endpoint.remotePath,
    username: endpoint.username,
    password: '',
    // 只读固定值：无论历史端点里存过哪一档，界面与保存都回到真实行为「每次询问」。
    conflictPolicy: SYNC_CONFLICT_POLICY_ASK,
  }
}, { immediate: true })

async function saveSyncEndpoint() {
  await app.saveSyncEndpoint(syncEndpointInput.value)
}


onMounted(()=>void app.loadSyncConfig())
onBeforeUnmount(()=>{syncEndpointInput.value.password=''})
</script>
<template>
      <article class="card"><div class="card-head"><div><h2>连接设置</h2><p>维护当前活动服务器；配置传输与接收范围在上方选择。</p></div></div>
        <div class="field-grid">
          <div class="field"><label for="sync-baseUrl">服务地址</label><input id="sync-baseUrl" v-model="syncEndpointInput.baseUrl" class="input" placeholder="https://dav.example.com/opencodex/" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label for="sync-remotePath">远端目录</label><input id="sync-remotePath" v-model="syncEndpointInput.remotePath" class="input" placeholder="/desktop-sync/current" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label for="sync-username">账号</label><input id="sync-username" v-model="syncEndpointInput.username" class="input" placeholder="账号" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
          <div class="field"><label for="sync-password">应用密码</label><input id="sync-password" v-model="syncEndpointInput.password" class="input" type="password" autocomplete="new-password" autocapitalize="off" autocorrect="off" spellcheck="false" :placeholder="app.syncConfig?.endpoint?'留空沿用已保存的密码':'输入应用密码'"></div><p v-if="app.syncConfig?.endpoint" class="sync-password-note">只改地址或目录时留空即可，不会覆盖已保存的密码。</p>
          <!-- 只读展示：引擎只实现「每次询问」，四档静默策略从未落码，不能让用户以为可改（§26.1）。
               仍按技术型输入保留关自动大写三属性，避免日后误改成可编辑时被 WKWebView 首字母大写。 -->
          <div class="field"><label>冲突策略</label><input class="input" readonly aria-readonly="true" aria-label="冲突策略" value="每次询问" autocapitalize="off" autocorrect="off" spellcheck="false"></div>
        </div>
        <div class="setting-list"><div class="setting-row"><div><div class="setting-title">配置保护与采用</div><div class="setting-desc">含凭据的快照使用独立口令保护，下载后比较并确认采用；采用前执行保护备份。</div></div></div></div>
        <div v-if="app.syncConfig?.endpoint" class="setting-row"><div><div class="setting-title">已配置端点</div><div class="setting-desc">{{ app.syncConfig.endpoint.url }} · {{ app.syncConfig.endpoint.remotePath }} · 凭据引用 {{ app.syncConfig.endpoint.credentialRefId.slice(0, 12) }}</div></div><div class="controls"><UiButton class="btn danger" :disabled="app.syncSaving" @click="app.deleteSyncEndpoint(true)">删除端点</UiButton></div></div>
        <div v-if="app.syncStatus" class="setting-row"><div><div class="setting-title">最近连接结果</div><div class="setting-desc">{{ app.syncStatus.message || '连接状态已更新。' }}</div></div><div class="controls"><span class="tag">{{ app.syncStatus.connectionState === 'synced' ? '成功' : app.syncStatus.connectionState === 'failed' ? '失败' : app.syncStatus.connectionState }}</span></div></div>
        <div class="controls"><UiButton class="btn" :disabled="app.syncSaving || app.syncTesting" @click="saveSyncEndpoint()">{{ app.syncSaving ? '保存中…' : '保存端点' }}</UiButton><UiButton class="btn ghost" :disabled="app.syncTesting || app.syncSaving" @click="app.testSyncConnection()">{{ app.syncTesting ? '测试中…' : '测试连接' }}</UiButton></div>
      </article>
</template>
<style scoped>
/* 口令留空的含义就地说明，避免用户以为漏填。 */
.sync-password-note{margin:-4px 0 0;color:var(--muted);font-size:var(--text-caption);grid-column:1/-1}
</style>
