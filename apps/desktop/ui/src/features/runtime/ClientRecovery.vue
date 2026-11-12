<script setup lang="ts">
import { computed } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useClientRecoveryStore } from './client-recovery-store'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { ClientRecoveryEvidence, ClientRecoveryPhase } from './client-recovery'
const recovery = useClientRecoveryStore(), lifecycle = useLifecycleStore()
const busy = computed(() => recovery.reading || recovery.executing || recovery.refreshing)
const phases: Record<ClientRecoveryPhase, string> = { preparing: '正在登记原会话', captured: '原件已保存', ready: '服务会话已就绪', stop_prepared: '已准备停止', stopped: '服务已停止', restore_prepared: '配置恢复待核对', restore_rolled_back: '本次恢复已撤销', restored: '恢复已完成', unchanged: '原件保持，原会话已结束', cancelled_before_capture: '启动准备已取消', failed: '原会话需要核对' }
const evidence = (value: ClientRecoveryEvidence) => ({ verified: '已核对', not_recorded: '尚未登记', unregistered: '待确认', unknown: '未核实' })[value]
const progress = computed(() => ({ finalize:'正在完成恢复收尾…',continue:'正在继续恢复客户端配置…',rollback:'正在撤销本次恢复…',rearm:'正在重新准备原件恢复…',finishUnchanged:'正在核对原件并结束原会话…',cancelPreparation:'正在核对并取消启动准备…' })[recovery.action])
const restoredOutcome = computed(() => recovery.outcome?.status === 'restored' || recovery.outcome?.status === 'unchanged' || recovery.outcome?.status === 'cancelled_before_capture')
const confirmLabel = computed(() => recovery.choice === 'rollback' ? '确认撤销恢复' : recovery.choice === 'finishUnchanged' ? '确认结束会话' : recovery.choice === 'cancelPreparation' ? '确认取消启动准备' : '确认重新准备')
</script>
<template>
  <section class="client-recovery-entry" aria-labelledby="clientRecoveryTitle">
    <div class="card-head">
      <div><h3 id="clientRecoveryTitle">客户端配置恢复</h3><p>检查未完成的配置恢复，核对后处理原会话。</p></div>
      <div class="toolbar"><UiButton :disabled="lifecycle.processActionBusy && !recovery.executing" @click="recovery.start()">{{ recovery.executing ? '查看恢复进度' : '检查客户端恢复' }}</UiButton></div>
    </div>
  </section>
  <UiDialog :open="recovery.open" title="客户端配置恢复" initial-focus="dialog" @close="recovery.close()">
    <p v-if="recovery.reading" role="status">正在核对原会话和恢复记录…</p>
    <template v-else-if="recovery.executing"><p role="status">{{ progress }}</p><p>关闭窗口后仍会继续，可重新打开查看。</p></template>
    <template v-else-if="recovery.choice === 'rollback'">
      <p class="client-recovery-lead">撤销本次恢复？</p>
      <p>将本次恢复的文件退回恢复前的停止状态。启动前原件仍保留，服务保持停止。</p>
      <p>之后可以重新准备恢复，再确认恢复启动前的原件。</p>
    </template>
    <template v-else-if="recovery.choice === 'rearm'">
      <p class="client-recovery-lead">重新准备原件恢复？</p>
      <p>核对保留的原件和当前文件，准备新的恢复计划。准备完成后，还需点击“确认继续恢复”。</p>
      <p>此次准备不会更改客户端配置，服务保持停止。</p>
    </template>
    <template v-else-if="recovery.choice === 'cancelPreparation'">
      <p class="client-recovery-lead">取消这次启动准备？</p>
      <p>尚未创建原件保管目录。确认后重新核对服务已停止，再取消这次未完成的启动准备。</p>
      <p>保持当前配置，不恢复或删除文件，也不会自动启动。</p>
    </template>
    <template v-else-if="recovery.choice === 'finishUnchanged'">
      <p class="client-recovery-lead">核对并结束原会话？</p>
      <p>确认后重新核对服务已停止、原件完整且未改变，再结束这次未完成会话。</p>
      <p>客户端配置保持不变，不会自动启动服务。</p>
    </template>
    <template v-else-if="restoredOutcome">
      <p role="status">{{ recovery.outcome?.status === 'cancelled_before_capture' ? '启动准备已取消，当前配置保持不变。' : recovery.outcome?.status === 'unchanged' ? '原件保持完整，原会话已结束。' : recovery.continuing ? '客户端配置恢复已完成。' : '配置恢复收尾已完成。' }}</p>
      <p v-if="recovery.ready">服务保持停止，现在可以启动。</p>
      <p v-else-if="recovery.refreshing">正在核对当前运行状态，核对通过后才能启动。</p>
      <p v-else>当前运行状态尚未核实，核对通过后才能启动。</p>
    </template>
    <template v-else-if="recovery.outcome && !recovery.reconfirmed">
      <p role="status">{{ recovery.outcome.status === 'rolled_back' ? '本次恢复已撤销。' : '原件恢复计划已重新准备。' }}</p>
      <p>当前恢复状态尚未核实，服务保持停止。</p>
    </template>
    <template v-else-if="recovery.preview">
      <template v-if="recovery.canCancelPreparation">
        <p class="client-recovery-lead">启动准备已中断，尚未开始捕获原件。</p>
        <p>可核对后取消本次准备，保持当前配置。</p>
      </template>
      <template v-else-if="recovery.canFinishUnchanged">
        <p class="client-recovery-lead">启动前原件保持完整，原会话尚未结束。</p>
        <p>可以重新核对服务已停止后结束原会话。客户端配置保持不变。</p>
      </template>
      <template v-else-if="recovery.canFinalize">
        <p class="client-recovery-lead">客户端配置已恢复，尚有一项收尾未完成。</p>
        <p>确认后重新核对原会话并完成收尾。客户端配置保持不变。</p>
      </template>
      <template v-else-if="recovery.canContinue">
        <p class="client-recovery-lead">客户端配置恢复中断，可以继续原恢复计划。</p>
        <p>确认后恢复到本次启动前的配置，保留当前已归档的文件。服务保持停止。</p>
      </template>
      <p v-else-if="recovery.preview.status === 'not_required' || recovery.preview.status === 'restored'" role="status">没有待处理的客户端配置恢复。</p>
      <p v-else-if="recovery.preview.status === 'active'" role="status">当前会话正在使用。请先正常停止 OpenCodex，再检查恢复状态。</p>
      <template v-else-if="recovery.preview.status === 'pending' && recovery.preview.phase === 'restore_rolled_back'"><p role="status">本次恢复已撤销，文件已回到恢复前的停止状态。</p><p>启动前原件仍保留，尚未恢复；服务保持停止。</p></template>
      <template v-else-if="recovery.preview.status === 'pending'"><p role="status">配置恢复尚未完成，原会话已保留。</p><p>当前现场不能直接收尾，请查看诊断详情和应用日志。</p></template>
      <p v-else-if="recovery.preview.status === 'unsupported'" role="status">当前平台暂不支持此恢复操作。</p>
      <template v-else><p role="status">客户端恢复状态尚未核实。</p><p>请重新检查；若仍无法核对，请保留原文件并查看应用日志。</p></template>
      <details v-if="recovery.preview.phase" class="client-recovery-details">
        <summary>诊断详情</summary>
        <dl class="client-recovery-facts">
          <dt>会话阶段</dt><dd>{{ phases[recovery.preview.phase] }}</dd>
          <dt>客户端原件</dt><dd>{{ evidence(recovery.preview.originalEvidence) }}</dd>
          <dt>恢复计划</dt><dd>{{ evidence(recovery.preview.planEvidence) }}</dd>
          <dt>完成记录</dt><dd>{{ evidence(recovery.preview.completionEvidence) }}</dd>
          <dt v-if="recovery.preview.attemptId">会话编号</dt><dd v-if="recovery.preview.attemptId"><code>{{ recovery.preview.attemptId }}</code></dd>
        </dl>
      </details>
    </template>
    <p v-if="recovery.error" role="alert" class="model-error">{{ recovery.error }}</p>
    <p v-if="recovery.refreshError" role="alert" class="model-error">{{ recovery.refreshError }}</p>
    <template #actions>
      <UiButton v-if="recovery.choice && !recovery.executing" @click="recovery.cancelChoice()">返回</UiButton>
      <UiButton v-else @click="recovery.close()">{{ busy || recovery.outcome ? '关闭' : '取消' }}</UiButton>
      <UiButton v-if="recovery.choice && !recovery.executing" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.apply(recovery.choice!)">{{ confirmLabel }}</UiButton>
      <UiButton v-else-if="restoredOutcome && !recovery.ready" :disabled="busy" @click="recovery.refresh()">刷新运行状态</UiButton>
      <UiButton v-else-if="recovery.outcome && !restoredOutcome && !recovery.reconfirmed" :disabled="busy" @click="recovery.refresh()">刷新恢复状态</UiButton>
      <template v-else-if="!recovery.executing">
        <UiButton v-if="recovery.canRollback" :disabled="busy || lifecycle.processActionBusy" @click="recovery.choose('rollback')">撤销本次恢复</UiButton>
        <UiButton v-if="recovery.canFinalize" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.apply('finalize')">确认完成收尾</UiButton>
        <UiButton v-else-if="recovery.canContinue" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.apply('continue')">确认继续恢复</UiButton>
        <UiButton v-else-if="recovery.canRearm" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.choose('rearm')">重新准备恢复</UiButton>
        <UiButton v-else-if="recovery.canCancelPreparation" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.choose('cancelPreparation')">取消启动准备</UiButton>
        <UiButton v-else-if="recovery.canFinishUnchanged" variant="primary" :disabled="busy || lifecycle.processActionBusy" @click="recovery.choose('finishUnchanged')">核对并结束会话</UiButton>
        <UiButton v-else-if="!recovery.outcome" :disabled="busy" @click="recovery.inspect()">重新检查</UiButton>
      </template>
    </template>
  </UiDialog>
</template>
<style scoped>
.client-recovery-entry{margin-top:var(--space-5);padding-top:var(--space-4);border-top:1px solid var(--border)}
.client-recovery-entry .card-head{align-items:center;gap:var(--space-3)}
.client-recovery-entry h3{margin:0}.client-recovery-entry p{margin:var(--space-2) 0 0}
.client-recovery-lead{font-weight:600}.client-recovery-details{margin-top:var(--space-4)}
.client-recovery-details summary{cursor:pointer;color:var(--muted)}
.client-recovery-facts{display:grid;grid-template-columns:6em minmax(0,1fr);gap:var(--space-2) var(--space-3);margin-top:var(--space-3)}
.client-recovery-facts dt{color:var(--muted)}.client-recovery-facts dd{margin:0;overflow-wrap:anywhere}
@media(max-width:600px){.client-recovery-entry .card-head{align-items:flex-start;flex-direction:column}.client-recovery-facts{grid-template-columns:5.5em minmax(0,1fr)}}
</style>
