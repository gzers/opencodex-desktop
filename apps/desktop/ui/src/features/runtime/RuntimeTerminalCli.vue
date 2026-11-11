<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useTerminalCliStore } from './terminal-cli'
const props=defineProps<{disabled?:boolean}>()
const terminal=useTerminalCliStore()
const {open,error,result,preview}=storeToRefs(terminal)
const busy=computed(()=>terminal.reading||terminal.executing)
const inspect=()=>terminal.inspect()
const close=()=>terminal.close()
const apply=()=>{if(!props.disabled)void terminal.apply()}
</script>
<template>
 <article class="card">
  <div class="card-head"><div><h2>终端命令</h2><p>终端与管理器使用同一个 OpenCodex 和配置目录。</p></div><UiButton :disabled="disabled&&!terminal.executing" @click="inspect">{{terminal.executing?'查看终端配置进度':'检查 ocx 命令'}}</UiButton></div>
 </article>
 <UiDialog :open="open" title="终端 ocx 命令" @close="close">
  <p v-if="busy" role="status">正在核对终端入口…</p>
  <p v-if="result" role="status">{{result}}</p>
  <template v-if="preview">
   <dl class="terminal-facts"><dt>状态</dt><dd>{{preview.pending?'原操作尚未完成':preview.enabled?'已启用':'未启用'}}</dd><dt>命令位置</dt><dd><code>{{preview.commandPath}}</code></dd><dt>转交入口</dt><dd><code>{{preview.runtimeEntry}}</code></dd><dt v-if="preview.version">版本</dt><dd v-if="preview.version">{{preview.version}}</dd><dt>配置目录</dt><dd><code>{{preview.opencodexHome}}</code></dd><dt>PATH 配置</dt><dd><code v-for="path in preview.profilePaths" :key="path">{{path}}</code></dd></dl>
   <template v-if="preview.conflicts.length"><p class="model-error" role="alert">已有其它来源的 ocx 命令，未覆盖。请处理原命令后重新核对。</p><ul><li v-for="path in preview.conflicts" :key="path"><code>{{path}}</code></li></ul></template>
   <p v-else-if="!result">{{preview.action==='enable'?'仅更新当前用户的命令入口与必要的 PATH，完成后打开新终端生效。':'仅移除本应用登记的入口和 PATH 配置，其它终端内容保留。'}}</p>
  </template>
  <p v-if="error" role="alert" class="model-error">{{error}}</p>
  <template #actions>
   <UiButton @click="close">关闭</UiButton>
   <UiButton v-if="!result&&preview&&!preview.conflicts.length" :disabled="busy||disabled" variant="primary" @click="apply">{{preview.pending?'继续原操作':preview.action==='enable'?'确认启用':'确认停用'}}</UiButton>
   <UiButton v-else-if="error||preview?.conflicts.length" :disabled="busy||disabled" @click="inspect">重新核对</UiButton>
  </template>
 </UiDialog>
</template>
<style scoped>
.terminal-facts{display:grid;grid-template-columns:7em minmax(0,1fr);gap:var(--space-2) var(--space-3)}.terminal-facts dt{color:var(--muted)}.terminal-facts dd{margin:0;overflow-wrap:anywhere}.terminal-facts code{display:block}
</style>
