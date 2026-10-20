<script setup lang="ts">
import {computed,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {versionLabel,type Template,type TemplateDraft} from './api'
import {templateDifferences} from './templateChanges'
const props=defineProps<{open:boolean;template:Template|null;draft:TemplateDraft|null;libraryDigest:string}>()
defineEmits<{close:[]}>()
const frozen=ref<{template:Template;draft:TemplateDraft;libraryDigest:string}|null>(null),selected=ref('')
watch(()=>props.open,open=>{if(!open){frozen.value=null;return}if(!props.template||!props.draft||props.draft.templateId!==props.template.id){frozen.value=null;return}frozen.value=JSON.parse(JSON.stringify({template:props.template,draft:props.draft,libraryDigest:props.libraryDigest}));selected.value=props.template.defaultRevision??props.draft.baseRevision??props.template.revisions.at(-1)?.id??''},{immediate:true})
const revision=computed(()=>frozen.value?.template.revisions.find(r=>r.id===selected.value))
const differences=computed(()=>revision.value&&frozen.value?templateDifferences(revision.value,frozen.value.draft):[])
const options=computed(()=>[...(frozen.value?.template.revisions??[])].reverse().map(r=>({value:r.id,label:versionLabel(r)+(r.id===frozen.value?.template.defaultRevision?' · 默认':'')})))
const text=(value:unknown)=>value===undefined?'未设置':JSON.stringify(value,null,2)
</script>
<template>
 <UiDialog :open="open" title="与版本比较" wide @close="$emit('close')">
  <template v-if="frozen&&revision"><p>{{ frozen.draft.label }} · 当前编辑缓冲</p><label class="model-comparison-version">比较基准<UiSelectField v-model="selected" :options="options" label="比较基准版本" /></label><p v-if="libraryDigest!==frozen.libraryDigest" role="status">资料库已变化；此处保留打开时的比较快照，请关闭后重新查看。</p><p>共 {{ differences.length }} 项差异</p>
   <p v-if="!differences.length">当前缓冲与所选保存版本一致。</p><article v-for="row in differences" :key="row.field" class="model-comparison-row"><h3>{{ row.field }}</h3><div class="model-comparison-values"><div><strong>保存版本</strong><pre>{{ text(row.before) }}</pre></div><div><strong>当前缓冲</strong><pre>{{ text(row.after) }}</pre></div></div></article>
  </template><p v-else role="alert">当前没有可比较的保存版本。</p>
  <template #actions><UiButton @click="$emit('close')">关闭</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.model-comparison-version{display:grid;gap:6px}.model-comparison-row{padding-block:12px;border-bottom:1px solid var(--border)}.model-comparison-row h3{font-size:var(--text-control);margin:0 0 8px;overflow-wrap:anywhere}.model-comparison-values{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(220px,100%),1fr));gap:12px}.model-comparison-values>div{min-width:0}.model-comparison-values strong{font-size:var(--text-caption);color:var(--muted)}.model-comparison-values pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:200px;overflow:auto;font-size:var(--text-caption);margin:6px 0}
</style>
