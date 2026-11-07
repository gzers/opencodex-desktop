<script setup lang="ts">
import {computed,ref} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import {recoveryLabelsV2,type OperationFactV2,type RecoveryActionV2,type ResumePreflightV2} from './api-v2'
const props=defineProps<{open:boolean;fact:OperationFactV2|null;busy?:boolean;error?:string;preflight?:ResumePreflightV2|null}>()
const emit=defineEmits<{close:[];read:[];action:[value:RecoveryActionV2];cancel:[];resume:[];dismissResume:[];receive:[]}>()
const resumeConfirm=ref<RecoveryActionV2|null>(null)
const receiveConfirm=ref(false)
const localLabels={not_compared:'未比较当前本机',observed_equal:'本次观察一致',changed:'本机另有变化',unknown:'当前关系未知'}
const remoteLabels={unknown:'当前关系未知',observed_equal:'本次观察一致',descendant:'远端已有后续版本',conflict:'同代次存在并发配置',initialization_ambiguous:'多个初始化记录，普通写入停止',generation_changed:'原代次已变化'}
const storageLabels={not_attempted:'未尝试写配置提交',unknown:'尚未确认原提交已存储',stored:'原提交与载荷已核验存储',invalid:'身份或内容未通过校验'}
const applyLabels={not_applicable:'不适用',not_attempted:'尚未应用本机',unknown:'本机应用结果待核对',applied:'本机已核验应用',invalid:'本机应用未通过校验'}
const phaseLabels={prepared:'准备完成',initialization_may_be_written:'初始化可能已发送',generation_verified:'初始化代次已核对',payload_may_be_written:'载荷可能已发送',payload_verified:'载荷已核验',commit_may_be_written:'提交可能已发送',settled:'本次处理已停止',cancelled_before_write:'未发送前已取消'}
const mayHaveWritten=computed(()=>props.fact?.lastWriteStep!==undefined&&props.fact.lastWriteStep!=='none')
const canStop=computed(()=>!!props.fact&&!['settled','cancelled_before_write'].includes(props.fact.phase))
function act(value:RecoveryActionV2){if(props.busy)return;if(value.kind==='resume_original_upload'){resumeConfirm.value=value;return}emit('action',value)}
</script>
<template>
  <UiDialog :open="open" title="同步操作结果" wide :busy="busy" @close="emit('close')">
    <p v-if="busy" role="status">正在核对本次操作…</p><p v-if="error" class="model-error" role="alert">{{error}}</p>
    <p v-if="!fact&&!busy&&!error">尚无可读取的原操作记录。</p>
    <template v-if="fact">
      <dl class="sync-facts"><dt>处理阶段</dt><dd>{{phaseLabels[fact.phase]}}</dd><dt>远端存储</dt><dd>{{storageLabels[fact.storageStatus]}}</dd><dt>本机应用</dt><dd>{{applyLabels[fact.localApplyStatus]}}</dd><dt>当前本机关系</dt><dd>{{localLabels[fact.relation.local]}}</dd><dt>当前远端关系</dt><dd>{{remoteLabels[fact.relation.remote]}}</dd></dl>
      <p v-if="fact.storageStatus==='stored'&&fact.relation.local==='changed'" role="status">原上传已存储；本机仍有后续变化，未记为当前已同步。</p>
      <p v-if="fact.kind==='download'&&fact.storageStatus==='stored'&&fact.localApplyStatus!=='applied'" role="status">源提交已核验存储，不代表本机下载应用已完成。</p>
      <p v-if="mayHaveWritten">可能已产生远端写入；停止后续动作不表示撤回。原操作身份与材料保持。</p>
      <p v-else>尚未发送配置写入。读取操作记录只访问本机，不触发上传或远端核对。</p>
      <p v-if="fact.failure" class="model-error" role="alert">{{fact.failure.code}} · {{fact.failure.step}}<span v-if="fact.failure.httpStatus"> · HTTP {{fact.failure.httpStatus}}</span></p>
      <details><summary>原操作标识与核验证据</summary><dl class="sync-facts"><dt>原操作</dt><dd>{{fact.operationId}}</dd><dt>代次</dt><dd>{{fact.generation?.generationId??'尚未确认'}}</dd><dt>载荷摘要</dt><dd>{{fact.payloadDigest}}</dd><dt>存储核验时间</dt><dd>{{fact.storedAt??'未知'}}</dd></dl><p>存储核验时间不是业务保存时间，也不保证所有设备永远最新。</p></details>
    </template>
    <template #actions>
      <UiButton :disabled="busy" @click="emit('read')">读取本机操作记录</UiButton>
      <UiButton v-if="canStop" :disabled="busy" @click="emit('cancel')">{{mayHaveWritten?'停止后续动作':'取消本次操作'}}</UiButton>
      <UiButton v-for="(action,index) in fact?.recoveryActions??[]" :key="index" :disabled="busy" :variant="action.kind==='verify_original_upload'?'primary':'default'" @click="act(action)">{{recoveryLabelsV2[action.kind]}}</UiButton>
      <UiButton v-if="fact?.kind==='download'&&fact.localApplyStatus!=='applied'" :disabled="busy" @click="receiveConfirm=true">核对记录并继续本机更新</UiButton>
    </template>
  </UiDialog>
  <UiDialog :open="!!preflight" title="原上传续传准入" :busy="busy" @close="emit('dismissResume')"><template v-if="preflight"><p>{{preflight.reason}}</p><p>原操作 {{preflight.request.operationId}}</p><p>本次只补发原冻结字节；不重新选取今天的配置或创建上传。</p></template><template #actions="{close}"><UiButton :disabled="busy" @click="close">关闭</UiButton><UiButton variant="primary" :disabled="busy||!preflight?.canResume" @click="emit('resume')">确认补发原操作</UiButton></template></UiDialog>
  <UiDialog :open="receiveConfirm" title="继续原下载的本机更新" :busy="busy" @close="receiveConfirm=false"><p>后端将核对原下载身份和模型已采用的实际证明，再继续剩余本机参与项。证明缺失时保留错误，需重新预览。</p><p>即使原操作结清，也不表示今天本机与云端一致。</p><template #actions="{close}"><UiButton :disabled="busy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="busy" @click="emit('receive');receiveConfirm=false">确认核对原下载记录</UiButton></template></UiDialog>
  <UiDialog :open="!!resumeConfirm" title="确认续传原上传" :busy="busy" @close="resumeConfirm=null">
    <p>先只读核对原提交；仅在后端确认原冻结字节、端点、代次及来源/范围基准仍有效后补发同一操作。不会新建上传或重新初始化。</p>
    <p>{{resumeConfirm?.operationId}}</p>
    <template #actions="{close}"><UiButton :disabled="busy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="busy||!resumeConfirm" @click="resumeConfirm&&(emit('action',resumeConfirm),resumeConfirm=null)">确认核对并续传原操作</UiButton></template>
  </UiDialog>
</template>
<style scoped>
.sync-facts{display:grid;grid-template-columns:minmax(100px,auto) minmax(0,1fr);gap:10px 16px}.sync-facts dt{color:var(--muted)}.sync-facts dd{margin:0;overflow-wrap:anywhere}.model-error{color:var(--red)}p{line-height:1.6;overflow-wrap:anywhere}@media(max-width:560px){.sync-facts{grid-template-columns:minmax(0,1fr)}}
</style>
