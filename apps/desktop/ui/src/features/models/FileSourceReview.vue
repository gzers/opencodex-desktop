<script setup lang="ts">
import { ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useModelsV2Store, type FileSourceReview } from './store-v2'
import { useFeedbackStore } from '@/app/feedback/store'

const models = useModelsV2Store()
const open = ref(false)
const error = ref('')
const review = ref<FileSourceReview | null>(null)
const reasons: Record<string, string> = {
  overlap: '磁盘和草稿都修改了此渠道。请先核对两边的修改。',
  removed: '磁盘上已不存在此渠道。当前输入已保留。',
  native_name: '磁盘上的渠道名称已变化，需先核对实际名称。',
  bound_copy: '原来选定的配置来源已变化，无法继续使用原来源。',
  identity: '磁盘新增渠道与草稿中的渠道标识相同。',
}
function describe(cause: unknown) {
  if (cause instanceof Error) return cause.message
  const code = typeof cause === 'object' && cause !== null && 'code' in cause ? String(cause.code) : ''
  if (code === 'model_reconcile_baseline_missing') return '这份草稿缺少可核对的原基准。当前输入仍保留。'
  if (code === 'model_reconcile_stale' || code === 'model_reconcile_overlap') return '磁盘配置或草稿又发生了变化，请重新核对。'
  if (code === 'model_source_provenance_unknown') return '无法核对磁盘上的渠道或协议来源，当前草稿已保留。'
  return '暂时无法核对配置来源，当前输入已保留。'
}
async function inspect() {
  open.value = true; review.value = null; error.value = ''
  try { review.value = await models.reviewFileSource() }
  catch (cause) { error.value = describe(cause) }
}
async function apply() {
  if (!review.value) return
  error.value = ''
  try {
    await models.applyFileSource(review.value)
    open.value = false
    useFeedbackStore().showToast('配置来源已核对，草稿已保留。请检查后重新保存。')
  } catch (cause) { review.value = null; error.value = describe(cause) }
}
</script>

<template>
  <div v-if="models.file?.draft && (models.fileSourceStale || models.fileNeedsReconcile)" class="source-review-entry">
    <span>当前草稿已保留，请先核对配置来源。</span>
    <UiButton :disabled="models.fileBusy" @click="inspect">核对配置来源</UiButton>
  </div>
  <UiDialog :open="open" :busy="models.fileBusy" title="核对配置来源" @close="open = false">
    <p>核对磁盘变化与当前草稿。确认后更新草稿基准，之后仍需手动保存。</p>
    <p v-if="models.fileBusy" role="status">正在核对当前配置…</p>
    <p v-if="error" role="alert" class="model-error">{{ error }}</p>
    <template v-if="review">
      <template v-if="review.preview.conflicts.length">
        <p role="alert" class="model-error">存在冲突，草稿和磁盘配置均已保留。</p>
        <ul><li v-for="(item,index) in review.preview.conflicts" :key="index">{{ item.label }}：{{ reasons[item.reason] ?? '需核对此渠道' }}</li></ul>
      </template>
      <template v-else>
        <p v-if="review.preview.changedChannels.length">磁盘上变化的渠道：{{ review.preview.changedChannels.join('、') }}</p>
        <p v-else-if="review.preview.sourceDigest !== review.request.expected.sourceDigest">磁盘配置已变化，未发现与草稿冲突的渠道。</p>
        <p v-else>磁盘配置与草稿基准一致。</p>
      </template>
    </template>
    <template #actions="{ close }">
      <UiButton :disabled="models.fileBusy" @click="close">取消</UiButton>
      <UiButton v-if="review && !review.preview.conflicts.length" variant="primary" :disabled="models.fileBusy" @click="apply">确认重核</UiButton>
      <UiButton v-else-if="!review && error" :disabled="models.fileBusy" @click="inspect">重新核对</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.source-review-entry { display: flex; gap: var(--space-2); align-items: center; flex-wrap: wrap; }
</style>
