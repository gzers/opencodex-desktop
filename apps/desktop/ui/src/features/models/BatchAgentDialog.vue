<script setup lang="ts">
import {computed,ref,watch} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import AgentPanel from './AgentPanel.vue'
import {useModelsV2Store} from './store-v2'
import type {LibraryViewV2,RefV2} from './api-v2'
const props=defineProps<{open:boolean;snapshot:LibraryViewV2;selected:string[];versions:Record<string,string>}>()
const emit=defineEmits<{close:[]}>()
const models=useModelsV2Store(),scope=ref<'all'|'selected'>(props.selected.length?'selected':'all')
const sessionKey='batch:v2',session=computed(()=>models.agentSession(sessionKey))
const busy=computed(()=>session.value.preparing||session.value.generating||session.value.accepting||session.value.stopping)
const selectedTemplates=computed(()=>props.snapshot.library.templates.filter(t=>scope.value==='all'||props.selected.includes(t.id)))
const missing=computed(()=>selectedTemplates.value.filter(t=>!t.revisions.some(r=>r.id===(props.versions[t.id]??t.defaultRevision))))
const targets=computed<RefV2[]>(()=>selectedTemplates.value.flatMap(t=>{const revision=t.revisions.find(r=>r.id===(props.versions[t.id]??t.defaultRevision));return revision?[{templateId:t.id,revisionId:revision.id,digest:revision.digest}]:[]}))
watch(()=>props.open,value=>{if(value&&!busy.value)scope.value=props.selected.length?'selected':'all'})
function close(){if(!session.value.accepting)emit('close')}
</script>
<template>
  <UiDialog :open="open" title="检查模版更新" wide :busy="session.accepting" @close="close">
    <div class="batch-agent-form">
      <div class="model-inline"><label><input v-model="scope" type="radio" value="all" :disabled="busy" />全部已保存模版</label><label><input v-model="scope" type="radio" value="selected" :disabled="busy||!selected.length" />已勾选 {{selected.length}} 项</label><span>{{targets.length}} 个确切版本</span></div>
      <p v-if="missing.length" class="model-error" role="alert">以下模版缺少明确保存版本，不会自动取 latest：{{missing.map(t=>t.label).join('、')}}。</p>
      <ul><li v-for="target in targets" :key="target.templateId">{{snapshot.library.templates.find(t=>t.id===target.templateId)?.label}} · {{target.revisionId}}</li></ul>
      <AgentPanel v-if="open" embedded :snapshot="snapshot" :targets="targets" :session-key="sessionKey" :disabled="!!missing.length||!targets.length" />
    </div>
    <template #actions="{close:requestClose}"><UiButton :disabled="session.accepting" @click="requestClose">{{session.generating?'关闭并保留原任务':'关闭'}}</UiButton></template>
  </UiDialog>
</template>
<style scoped>
.batch-agent-form{display:grid;gap:16px;min-width:0}.batch-agent-form li{overflow-wrap:anywhere}.batch-agent-form .model-inline{display:flex;gap:12px;flex-wrap:wrap}.model-error{color:var(--red)}
</style>
