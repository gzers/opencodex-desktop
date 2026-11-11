<script setup lang="ts">
import {ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {useAppStore} from '@/stores/app'
import {useDataRootStore} from '@/features/data-root/store'
import {useFeedbackStore} from '@/app/feedback/store'
import {useProgramReinstallStore} from './program-reinstall'
const props=defineProps<{disabled?:boolean}>()
const reinstall=useProgramReinstallStore(),app=useAppStore(),paths=useDataRootStore(),refreshError=ref('')
async function apply(){
 if(props.disabled||!await reinstall.apply())return
 useFeedbackStore().showToast('内置程序已重新安装，配置和备份已保留。')
 try{await Promise.all([app.loadRuntimeSource(),paths.loadPaths()])}
 catch{refreshError.value='重装已完成，但页面信息刷新失败。请刷新位置后核对。'}
}
</script>
<template>
 <UiButton data-testid="runtime-builtin-reinstall" :disabled="disabled&&!reinstall.executing" @click="reinstall.inspect()">{{reinstall.executing?'查看重装进度':'重装内置程序'}}</UiButton>
 <UiDialog :open="reinstall.open" title="重新安装内置 OpenCodex" @close="reinstall.close()">
  <p v-if="reinstall.reading" role="status">正在核对内置资源与原卸载状态…</p>
  <p v-else-if="reinstall.executing" role="status">正在验证并重新安装内置程序，关闭窗口后仍会继续。</p>
  <p v-else-if="reinstall.outcome" role="status">OpenCodex {{reinstall.outcome.version}} 已重新安装，配置和备份已保留。</p>
  <template v-else-if="reinstall.review">
   <p>{{reinstall.review.resume?'继续原内置程序重装。':'使用当前应用携带的内置资源重新安装程序。'}}配置和备份保留。</p>
   <dl class="reinstall-facts"><dt>版本</dt><dd>{{reinstall.review.version}}</dd><dt>程序位置</dt><dd><code>{{reinstall.review.programRoot}}</code></dd><dt>保留配置</dt><dd><code>{{reinstall.review.configPath}}</code></dd></dl>
   <p>请先停止 OpenCodex。重装会核对程序未被使用，不会自动启用终端命令。</p>
  </template>
  <p v-if="reinstall.error" role="alert" class="model-error">{{reinstall.error}}</p>
  <p v-if="refreshError" role="alert">{{refreshError}}</p>
  <template #actions><UiButton @click="reinstall.close()">{{reinstall.executing||reinstall.outcome?'关闭':'取消'}}</UiButton>
   <UiButton v-if="reinstall.review&&!reinstall.executing" variant="primary" :disabled="disabled||reinstall.reading" @click="apply">{{reinstall.review.resume?'确认继续原重装':'确认重新安装'}}</UiButton>
   <UiButton v-else-if="reinstall.error&&!reinstall.executing" :disabled="disabled||reinstall.reading" @click="reinstall.inspect()">重新核对</UiButton>
  </template>
 </UiDialog>
</template>
<style scoped>
.reinstall-facts{display:grid;grid-template-columns:6em minmax(0,1fr);gap:var(--space-2) var(--space-3)}.reinstall-facts dt{color:var(--muted)}.reinstall-facts dd{margin:0;overflow-wrap:anywhere}
</style>
