<script setup lang="ts">
import { computed, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useActivationRecoveryStore } from './activation-recovery'
import { useAppStore } from '@/stores/app'
import { useDataRootStore } from '@/features/data-root/store'
import { useFeedbackStore } from '@/app/feedback/store'

const props = defineProps<{ disabled?: boolean }>()
const recovery = useActivationRecoveryStore(), app = useAppStore(), paths = useDataRootStore()
const busy = computed(() => recovery.reading || recovery.executing)
const refreshError = ref('')
function open() { refreshError.value = ''; recovery.start() }
async function apply() {
  if (props.disabled || !await recovery.apply()) return
  useFeedbackStore().showToast(recovery.outcome?.decision === 'rollback' ? '已回退安装操作。' : '安装恢复已完成。')
  try { await Promise.all([app.loadRuntimeSource(), paths.loadPaths()]) }
  catch { refreshError.value = '恢复已完成，页面信息刷新失败，请刷新位置后核对。' }
}
</script>

<template>
  <div class="activation-recovery-entry">
    <UiButton data-testid="runtime-activation-recovery" :disabled="disabled && !recovery.executing" @click="open">{{ recovery.executing ? '查看安装恢复进度' : '检查安装恢复' }}</UiButton>
  </div>
  <UiDialog :open="recovery.open" title="安装恢复" @close="recovery.close()">
    <p v-if="recovery.reading" role="status">正在核对原安装操作…</p>
    <template v-else-if="recovery.executing"><p role="status">正在恢复原安装操作…</p><p>关闭窗口后仍会继续，可重新打开查看。</p></template>
    <template v-else-if="recovery.outcome">
      <p role="status">{{ recovery.outcome.decision === 'rollback' ? '原安装操作已回退。' : '安装恢复已完成。' }}</p>
      <p>{{ recovery.outcome.selectedVersion ? '当前选定版本：' + recovery.outcome.selectedVersion : '已恢复为未选择运行版本。' }}</p>
      <p>服务保持停止，请核对后启动。</p>
    </template>
    <template v-else-if="recovery.preview">
      <p>发现未完成的版本切换，请选择处理方式。</p>
      <dl class="activation-facts">
        <dt>原版本</dt><dd>{{ recovery.preview.from ?? '尚未安装' }}</dd>
        <dt>目标版本</dt><dd>{{ recovery.preview.to }}</dd>
        <dt>程序位置</dt><dd><code>{{ recovery.preview.programRoot }}</code></dd>
        <dt>配置保护</dt><dd><code>{{ recovery.preview.configPath }}</code></dd>
      </dl>
      <fieldset class="activation-choices"><legend>恢复方式</legend>
        <label><input v-model="recovery.decision" type="radio" value="complete" :disabled="disabled" />继续完成安装</label>
        <label><input v-model="recovery.decision" type="radio" value="rollback" :disabled="disabled" />{{ recovery.preview.from ? '回退到原版本 ' + recovery.preview.from : '撤销本次版本选择' }}</label>
      </fieldset>
      <p>执行前会确认服务已停止并备份当前配置。程序恢复完成后需要手动启动服务。</p>
    </template>
    <p v-else-if="recovery.checked">没有发现待恢复的版本切换操作。</p>
    <p v-if="recovery.error" role="alert" class="model-error">{{ recovery.error }}</p>
    <p v-if="refreshError" role="alert">{{ refreshError }}</p>
    <template #actions>
      <UiButton @click="recovery.close()">{{ busy ? '关闭' : '取消' }}</UiButton>
      <UiButton v-if="recovery.preview && !recovery.executing" variant="primary" :disabled="disabled || busy" @click="apply">{{ recovery.decision === 'complete' ? '确认继续安装' : '确认回退' }}</UiButton>
      <UiButton v-else-if="recovery.error && !busy" :disabled="disabled" @click="recovery.inspect()">重新核对原操作</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.activation-facts{display:grid;grid-template-columns:6em minmax(0,1fr);gap:var(--space-2) var(--space-3)}
.activation-facts dt{color:var(--muted)}.activation-facts dd{margin:0;overflow-wrap:anywhere}
.activation-choices{display:grid;gap:var(--space-2);border:0;padding:0}.activation-choices label{display:flex;gap:var(--space-2);align-items:center}
</style>
