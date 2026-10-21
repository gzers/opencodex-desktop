<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import { modelApi, errorText, type ExternalScene, type ModelSnapshot, type TaskBundle, type TemplateDraft } from './api'
const props = defineProps<{ open: boolean; snapshot: ModelSnapshot; draft?: TemplateDraft | null; dirty?: boolean; selected?: string[] }>()
const emit = defineEmits<{ close: [] }>()
const scope = ref('all'); const scene = ref<ExternalScene>('from_sources'); const allowNew = ref(false); const requirements = ref('')
const bundle = ref<TaskBundle | null>(null); const preparing = ref(false); const copying = ref(false); const error = ref(''); const copied = ref(false)
const output = ref<HTMLTextAreaElement | null>(null)
let timer: ReturnType<typeof setTimeout> | undefined; let sequence = 0
const scenes = [{ value:'from_sources',label:'从资料制模' },{value:'update_models',label:'更新型号资料'},{value:'audit_and_revise',label:'批量检查与修订'},{value:'private_models',label:'私有模型配置'},{value:'preferences',label:'按偏好调整'},{value:'check_only',label:'仅检查，不保存'}]
const targetTemplates = computed(() => scope.value === 'new' ? [] : props.snapshot.library.templates.filter(t => scope.value === 'current' ? t.id === props.draft?.templateId : scope.value === 'selected' ? props.selected?.includes(t.id) : true))
const request = computed(() => ({ scene: scene.value, targets: targetTemplates.value.map(t => ({ templateId: t.id, revisionId: scope.value === 'current' && props.draft?.baseRevision ? props.draft.baseRevision : t.revisions[t.revisions.length-1]!.id })), allowNew: scope.value === 'new' || allowNew.value, requirements: scope.value === 'new' && props.draft ? requirements.value+'\n新建表单：'+JSON.stringify(props.draft) : requirements.value, expectedLibrary: props.snapshot.libraryDigest, buffers: props.dirty && targetTemplates.value.some(t => t.id === props.draft?.templateId) && props.draft?.templateId && props.draft.baseRevision ? [{ templateId: props.draft.templateId, baseRevision: props.draft.baseRevision, label: props.draft.label, content: props.draft.content }] : [] }))
watch(() => props.open, open => { if (open) { scope.value = props.draft ? props.draft.templateId ? 'current' : 'new' : props.selected?.length ? 'selected' : 'all'; copied.value = false } })
watch([() => props.open, request], () => {
  if (timer) clearTimeout(timer); const current = ++sequence; bundle.value = null; copied.value = false
  if (!props.open) return
  preparing.value = true; error.value = ''
  timer = setTimeout(async () => { try { const value = await modelApi.external(JSON.parse(JSON.stringify(request.value))); if (sequence === current) bundle.value = value } catch (cause) { if (sequence === current) error.value = errorText(cause) } finally { if (sequence === current) preparing.value = false } }, 200)
}, { deep: true })
onBeforeUnmount(() => { if (timer) clearTimeout(timer); sequence++ })
async function copy() {
  if (!bundle.value || copying.value) return
  const current = bundle.value; copying.value = true; error.value = ''
  try {
    await modelApi.activate(current.grant)
    await navigator.clipboard.writeText(current.content)
    copied.value = true
  } catch (cause) { error.value = errorText(cause); output.value?.focus(); output.value?.select() } finally { copying.value = false }
}
</script>
<template>
  <UiDialog :open="open" title="外部任务" wide :busy="copying" @close="emit('close')">
    <div class="model-form"><div class="model-columns"><label>范围<UiSelectField v-model="scope" label="任务范围" :options="[{value:'current',label:'当前模版',disabled:!draft?.templateId},{value:'selected',label:'勾选模版',disabled:!selected?.length},{value:'all',label:'全部模版'},{value:'new',label:'仅新建'}]" /></label><label>场景<UiSelectField :model-value="scene" label="任务场景" :options="scenes" @update:model-value="scene = $event as ExternalScene" /></label></div>
      <label v-if="scope !== 'new' && scene !== 'check_only'" class="model-inline"><input v-model="allowNew" type="checkbox" />同时允许新增模版</label>
      <label>补充要求（可选）<textarea class="input" v-model="requirements" rows="3" placeholder="填写资料、型号或需要保留的偏好" /></label>
      <label>完整任务内容<textarea ref="output" readonly :value="bundle?.content ?? ''" rows="15" class="input mono" :aria-busy="preparing" /></label>
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
    </div>
    <template #actions><UiButton @click="emit('close')">关闭</UiButton><UiButton variant="primary" :disabled="!bundle || preparing" :loading="copying" @click="copy">{{ copied ? '再次复制完整任务' : '复制完整任务' }}</UiButton></template>
  </UiDialog>
</template>
