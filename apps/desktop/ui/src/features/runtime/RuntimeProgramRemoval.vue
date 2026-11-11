<script setup lang="ts">
import {ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {useProgramRemovalStore} from './program-removal'
import {useAppStore} from '@/stores/app'
import {useDataRootStore} from '@/features/data-root/store'
import {useFeedbackStore} from '@/app/feedback/store'
const props=defineProps<{disabled?:boolean}>()
const removal=useProgramRemovalStore(),app=useAppStore(),paths=useDataRootStore(),refreshError=ref('')
async function apply(){
 if(props.disabled||!await removal.apply())return
 useFeedbackStore().showToast('已移除运行程序与自有终端入口，配置和备份已保留。')
 try{await Promise.all([app.loadRuntimeSource(),paths.loadPaths()])}
 catch{refreshError.value='卸载已完成，但页面信息刷新失败。请刷新位置后核对。'}
}
</script>
<template>
 <UiButton variant="danger" data-testid="runtime-program-removal" :disabled="disabled&&!removal.executing" @click="removal.inspect()">{{removal.executing?'查看卸载进度':'卸载程序'}}</UiButton>
 <UiDialog :open="removal.open" title="卸载 OpenCodex 程序" @close="removal.close()">
  <p v-if="removal.reading" role="status">正在核对已登记的运行版本与终端入口…</p>
  <template v-else-if="removal.executing"><p role="status">正在移除已核对的程序文件…</p><p>关闭窗口后仍会继续，可重新打开查看。</p></template>
  <template v-else-if="removal.outcome"><p role="status">程序卸载已完成。</p><p>运行程序与自有终端入口已移除，用户配置和备份已保留。</p></template>
  <template v-else-if="removal.review">
   <p>{{removal.review.resume?'发现尚未完成的卸载，以下为原操作范围。':'将移除本应用已登记的 OpenCodex 运行版本。'}}</p>
   <dl class="removal-facts"><dt>程序位置</dt><dd><code>{{removal.review.removal.programRoot}}</code></dd><dt>运行版本</dt><dd>{{removal.review.removal.versions.join('、')}}</dd><dt>配置与备份</dt><dd>保留 <code>{{removal.review.removal.preservedHome}}</code></dd><template v-if="removal.review.commandPath"><dt>终端入口</dt><dd>移除 <code>{{removal.review.commandPath}}</code></dd><dt>PATH 配置</dt><dd>仅撤销本应用登记的区域 <code v-for="path in removal.review.shellPaths" :key="path">{{path}}</code></dd></template></dl>
   <p>请先停止 OpenCodex。执行前会重新核对程序未被使用并完整备份当前配置；文件变化或终端入口不属于本应用时不会继续移除。</p>
  </template>
  <p v-else-if="removal.checked">本安装的运行程序已卸载，没有待继续的卸载操作。</p>
  <p v-if="removal.error" role="alert" class="model-error">{{removal.error}}</p>
  <p v-if="refreshError" role="alert">{{refreshError}}</p>
  <template #actions>
   <UiButton @click="removal.close()">{{removal.executing?'关闭':'取消'}}</UiButton>
   <UiButton v-if="removal.review&&!removal.executing" variant="danger" :disabled="disabled||removal.reading" @click="apply">{{removal.review.resume?'确认继续原卸载':'确认卸载程序'}}</UiButton>
   <UiButton v-else-if="removal.error&&!removal.executing" :disabled="disabled||removal.reading" @click="removal.inspect()">重新核对原操作</UiButton>
  </template>
 </UiDialog>
</template>
<style scoped>
.removal-facts{display:grid;grid-template-columns:6em minmax(0,1fr);gap:var(--space-2) var(--space-3)}.removal-facts dt{color:var(--muted)}.removal-facts dd{margin:0;overflow-wrap:anywhere}.removal-facts code{display:block}
</style>
