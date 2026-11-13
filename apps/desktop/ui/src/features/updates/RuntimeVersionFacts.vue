<script setup lang="ts">
import {useUpdatesStore} from './store'
import UiButton from '@/components/ui/UiButton.vue'
const versions=useUpdatesStore()
function time(value:string|null){if(!value)return '';const date=new Date(value);return Number.isNaN(date.getTime())?'时间未核实':date.toLocaleString()}
const labels={consistent:'入口与版本清单一致',changed:'检测到程序来源变化',missing:'没有当前程序选择',pending:'程序操作尚未完成',unavailable:'当前程序版本尚未核实'}
</script>
<template>
 <section class="runtime-version-facts" aria-label="OpenCodex 实际版本">
  <div class="version-head"><h4>当前版本事实</h4><UiButton size="sm" :disabled="versions.versionFactsLoading" @click="versions.loadVersions()">{{versions.versionFactsLoading?'核对中':'重新核对版本'}}</UiButton></div>
  <p v-if="versions.versionFactsError" role="alert">{{versions.versionFactsError}}下列信息为上次观察。</p>
  <dl v-if="versions.versionFacts"><dt>磁盘包版本</dt><dd>{{versions.versionFacts.disk.diskVersion??'未读取'}}</dd><dt>运行实例版本</dt><dd>{{versions.versionFacts.runningState==='attested'?versions.versionFacts.running?.version:'未核实'}}<small v-if="versions.versionFacts.running">PID {{versions.versionFacts.running.pid}} · 端口 {{versions.versionFacts.running.port}}</small></dd><dt>上次安装记录</dt><dd>{{versions.versionFacts.disk.recordedVersion??'没有记录'}}<small v-if="versions.versionFacts.disk.recordedAt">{{time(versions.versionFacts.disk.recordedAt)}}</small></dd><dt>来源核对</dt><dd>{{labels[versions.versionFacts.disk.integrity]}}</dd></dl><p v-if="versions.versionFacts">核对时间 {{time(versions.versionFacts.disk.checkedAt)}}</p>
  <p v-else role="status">{{versions.versionFactsLoading?'正在核对实际程序版本…':'实际程序版本尚未核实。'}}</p>
  <p v-if="versions.versionFacts?.running&&versions.versionFacts.disk.diskVersion!==versions.versionFacts.running.version" role="status">磁盘与运行实例版本不同，当前进程尚未采用磁盘版本。</p>
 </section>
</template>
<style scoped>
.runtime-version-facts{display:grid;gap:var(--space-3);padding:var(--space-4);border:1px solid var(--border-soft);border-radius:var(--radius-sm)}.version-head{display:flex;justify-content:space-between;align-items:center;gap:var(--space-3)}h4,p{margin:0}dl{display:grid;grid-template-columns:8em minmax(0,1fr);gap:var(--space-2) var(--space-3);margin:0}dt,small,p{color:var(--muted)}dd{margin:0;overflow-wrap:anywhere}small{display:block}
</style>
