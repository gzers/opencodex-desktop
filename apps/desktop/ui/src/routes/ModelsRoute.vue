<script setup lang="ts">
import {computed,nextTick,onBeforeUnmount,onMounted,ref,toRef,watch} from 'vue'
import AppTopbar from '@/components/AppTopbar.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import DetailWorkspace from '@/components/layout/DetailWorkspace.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ChannelsView from '@/features/models/ChannelsView.vue'
import ConditionEditor from '@/features/models/ConditionEditor.vue'
import ConditionPreview from '@/features/models/ConditionPreview.vue'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import AgentPanel from '@/features/models/AgentPanel.vue'
import BatchAgentDialog from '@/features/models/BatchAgentDialog.vue'
import AgentExecutionField from '@/features/models/AgentExecutionField.vue'
import ExternalTaskDialog from '@/features/models/ExternalTaskDialog.vue'
import ModelSaveHistory from '@/features/backup/ModelSaveHistory.vue'
import {useModelsV2Store} from '@/features/models/store-v2'
import {useModelViewPosition} from '@/features/models/useViewPosition'
import {useModalStore} from '@/app/modal/store'
import {modelV2Api,type LibraryChangeV2,type UsageReportV2,type UsageKindV2,type UsageStateV2} from '@/features/models/api-v2'
import type {TemplateV2,RevisionV2,TemplateActionIntentV2,TemplateActionViewV2,RefV2} from '@/features/models/api-v2'
const models=useModelsV2Store(),modal=useModalStore(),page=ref<HTMLElement|null>(null)
const tab=toRef(models.view,'tab'),search=toRef(models.view,'templateSearch')
const editing=computed(()=>models.view.templateEditor&&!!models.draft)
const templates=computed(()=>models.snapshot?.library.templates.filter(t=>t.label.toLowerCase().includes(search.value.toLowerCase()))??[])
const sections=[{id:'model-basic',label:'基本信息'},{id:'model-matching',label:'匹配范围与条件'},{id:'model-rules',label:'族通用配置'},{id:'model-preview',label:'匹配预览'},{id:'model-source',label:'来源与版本'}]
useModelViewPosition(page,()=>tab.value==='channels'?(models.channelEditorId?'channel:'+models.channelEditorId:'channels'):(editing.value?'template:'+models.draft?.templateId:'templates'))
const historyId=ref(''),historySelection=ref<string[]>([]),historySearch=ref('')
const historyTemplate=computed(()=>models.snapshot?.library.templates.find(t=>t.id===historyId.value))
const compareOpen=ref(false),compareRevision=ref('')
const editingTemplate=computed(()=>models.snapshot?.library.templates.find(t=>t.id===models.draft?.templateId))
const comparison=computed(()=>editingTemplate.value?.revisions.find(r=>r.id===compareRevision.value))
const externalOpen=ref(false),agentOpen=ref(false),batchOpen=ref(false),agentTrigger=ref<HTMLElement|null>(null)
async function closeAgent(){agentOpen.value=false;await nextTick();const trigger=agentTrigger.value?.isConnected?agentTrigger.value:page.value?.querySelector<HTMLElement>('[aria-label="Agent自动填充"]');trigger?.focus({preventScroll:true})}
const toolOpen=ref(false),toolBusy=ref(false),toolError=ref(''),toolResult=ref<TemplateActionViewV2|null>(null),toolIntent=ref<TemplateActionIntentV2|null>(null)
let toolSequence=0,toolDraftGeneration=0,disposed=false,stopChanges:(()=>void)|undefined
const libraryOpen=ref(false),libraryBusy=ref(false),libraryWriting=ref(false),libraryError=ref(''),libraryReports=ref<UsageReportV2[]>([]),libraryRequest=ref<LibraryChangeV2|null>(null),libraryLabel=ref('')
const usageOpen=ref(false),usageBusy=ref(false),usageError=ref(''),usageReport=ref<UsageReportV2|null>(null),usageReference=ref<RefV2|null>(null)
let librarySequence=0,usageSequence=0
const libraryTitle=computed(()=>libraryRequest.value?.intent.kind==='set_default'?'设置已保存默认版本':libraryRequest.value?.intent.kind==='clear_default'?'清除默认版本':'删除所选版本')
const usageStates:Record<UsageStateV2,string>={saved:'已保存配置',draft:'服务端草稿',additional:'保留事务',library:'模版库'}
const usageKinds:Record<UsageKindV2,string>={global_reference:'全局精确候选',channel_reference:'渠道精确候选',manual:'人工指定',auto_read:'读取自动匹配',additional_exact:'事务保留精确引用',base_revision:'后继版本基准'}
const errors:Record<string,string>={model_save_stale:'库基准已变化，请重新读取后核对；当前选择和编辑保持。',model_template_default_changed:'当前默认已变化，不能清除另一个版本；请重新核对。',model_revision_builtin_protected:'内置初始版本受保护，不能删除。',model_revision_is_default:'版本仍是当前默认，请先明确清除或更换默认。',model_revision_in_use:'版本仍有精确依赖，请先核对保存、草稿、后继版本或保留事务。',model_revision_selection_invalid:'版本选择无效，请重新选择明确版本。',model_dependency_missing:'精确版本依赖不可用，当前编辑保持。',model_revision_digest_mismatch:'精确版本摘要不一致，未继续操作。'}
function operationError(cause:unknown){if(cause&&typeof cause==='object'&&'code'in cause)return (errors[String(cause.code)]??'模版操作未完成，当前版本与编辑保持。')+' · '+String(cause.code);return cause instanceof Error?cause.message:'模版操作未完成，当前选择保持。'}
function deleteReasons(report:UsageReportV2,ids:readonly string[]){const reasons:string[]=[];if(report.builtinInitial)reasons.push('内置初始版本');if(report.isDefault)reasons.push('当前默认版本');for(const use of report.references){if(use.kind==='base_revision'&&use.dependentRevision&&ids.includes(use.dependentRevision.revisionId))continue;reasons.push(usageStates[use.state]+' · '+usageKinds[use.kind]+(use.rawModelId?' · '+use.rawModelId:''))}if(models.protectsRevision(report.reference.templateId,report.reference.revisionId))reasons.push('当前模版编辑基准');return reasons}
const deletionBlocked=computed(()=>{const request=libraryRequest.value;if(request?.intent.kind!=='delete_revisions')return false;const ids=request.intent.revisionIds;return libraryReports.value.some(r=>deleteReasons(r,ids).length>0)})
function closeLibrary(){if(libraryWriting.value)return;libraryOpen.value=false;libraryBusy.value=false;librarySequence++}
function closeUsage(){usageOpen.value=false;usageSequence++;usageBusy.value=false}
async function readUsage(){const target=usageReference.value;if(!target)return;const request=++usageSequence;usageBusy.value=true;usageError.value='';usageReport.value=null;try{const report=await modelV2Api.templateUsage(target.templateId,target.revisionId);if(request!==usageSequence||!usageOpen.value)return;if(report.dtoVersion!==2||report.reference.templateId!==target.templateId||report.reference.revisionId!==target.revisionId||report.reference.digest!==target.digest||report.libraryDigest!==models.snapshot?.libraryDigest)throw {code:'model_save_stale'};usageReport.value=report}catch(cause){if(request===usageSequence)usageError.value=operationError(cause)}finally{if(request===usageSequence)usageBusy.value=false}}
function showUsage(template:TemplateV2,revision:RevisionV2){usageReference.value=reference(template,revision);usageOpen.value=true;void readUsage()}
async function prepareLibrary(kind:'set_default'|'clear_default'|'delete_revisions',template:TemplateV2,revision?:RevisionV2){
 if(models.saving||libraryBusy.value||!models.snapshot)return
 const refs=kind==='delete_revisions'?template.revisions.filter(r=>historySelection.value.includes(r.id)).map(r=>reference(template,r)):revision?[reference(template,revision)]:[]
 if(kind==='delete_revisions'&&(refs.length!==historySelection.value.length||new Set(historySelection.value).size!==historySelection.value.length)){libraryRequest.value={expectedLibrary:models.snapshot.libraryDigest,intent:{kind,templateId:template.id,revisionIds:[...historySelection.value]}};libraryLabel.value=template.label;libraryReports.value=[];libraryError.value=operationError({code:'model_revision_selection_invalid'});libraryOpen.value=true;return}
 if(!refs.length)return
 libraryRequest.value={expectedLibrary:models.snapshot.libraryDigest,intent:kind==='delete_revisions'?{kind,templateId:template.id,revisionIds:refs.map(r=>r.revisionId)}:{kind,reference:refs[0]!}};libraryLabel.value=template.label;libraryError.value='';libraryReports.value=[];libraryOpen.value=true;const request=++librarySequence
 if(kind!=='delete_revisions')return
 libraryBusy.value=true
 try{const reports=await Promise.all(refs.map(r=>modelV2Api.templateUsage(r.templateId,r.revisionId)));if(request!==librarySequence||!libraryOpen.value)return;for(let i=0;i<reports.length;i++){const report=reports[i]!,target=refs[i]!;if(report.dtoVersion!==2||report.reference.templateId!==target.templateId||report.reference.revisionId!==target.revisionId||report.reference.digest!==target.digest||report.libraryDigest!==libraryRequest.value?.expectedLibrary)throw {code:'model_save_stale'}}libraryReports.value=reports}
 catch(cause){if(request===librarySequence)libraryError.value=operationError(cause)}finally{if(request===librarySequence)libraryBusy.value=false}
}
async function confirmLibrary(){const request=libraryRequest.value;if(!request||libraryBusy.value||deletionBlocked.value)return;if(request.expectedLibrary!==models.snapshot?.libraryDigest){libraryError.value=operationError({code:'model_save_stale'});return}if(request.intent.kind==='delete_revisions'&&libraryReports.value.length!==request.intent.revisionIds.length)return;libraryWriting.value=true;libraryBusy.value=true;libraryError.value='';try{await models.changeLibrary(request);libraryOpen.value=false;librarySequence++;const intent=request.intent;if(intent.kind==='delete_revisions')historySelection.value=historySelection.value.filter(id=>!intent.revisionIds.includes(id))}catch(cause){libraryError.value=operationError(cause)}finally{libraryWriting.value=false;libraryBusy.value=false}}
function selected(t:TemplateV2){const id=models.view.selectedVersions[t.id]??t.defaultRevision;return t.revisions.find(r=>r.id===id)}
function chooseSavedVersion(t:TemplateV2,id:string){if(models.saving||models.dirty)return;if(id===''||t.revisions.some(r=>r.id===id))models.view.selectedVersions[t.id]=id}
function choices(t:TemplateV2){return [{value:'',label:'请选择明确保存版本'},...t.revisions.slice().reverse().map(r=>({value:r.id,label:r.createdAt+' · '+r.id.slice(0,8)+(t.defaultRevision===r.id?' · 默认':'')}))]}
function reference(t:TemplateV2,r:RevisionV2):RefV2{return {templateId:t.id,revisionId:r.id,digest:r.digest}}
function start(t?:TemplateV2,revision?:RevisionV2){const generation=models.editGeneration;const begin=()=>{if(generation!==models.editGeneration){models.error='编辑基准已变化，当前输入保持。';return}models.start(t,revision?.id??(t?selected(t)?.id:undefined));if(models.draft)models.view.templateEditor=true};if(models.dirty){modal.open({title:'切换模版',body:'放弃当前未保存的修改并打开所选版本？',confirmLabel:'放弃并打开',onConfirm:begin})}else begin()}
function create(){const generation=models.editGeneration;const begin=()=>{if(generation!==models.editGeneration)return;models.start();models.view.templateEditor=true};if(models.dirty)modal.open({title:'新建模版',body:'放弃当前未保存修改并创建新模版？',confirmLabel:'放弃并新建',onConfirm:begin});else begin()}
function back(){models.dismissPreview();models.view.templateEditor=false}
function cancelNew(){const generation=models.editGeneration;modal.open({title:'取消新建模版',body:'放弃当前未保存的新建内容？其他模版与渠道草稿保持。',confirmLabel:'放弃新建',onConfirm:()=>{if(generation!==models.editGeneration){models.error='确认已过期，输入保持。';return}models.discardNew();models.view.templateEditor=false}})}
function restore(){const generation=models.editGeneration;modal.open({title:'还原模版修改',body:'还原当前模版到本次编辑起点？其它缓冲与渠道草稿保持。',confirmLabel:'还原修改',onConfirm:()=>{if(generation!==models.editGeneration){models.error='还原确认已过期，当前编辑保持。';return}models.reset()}})}
function save(){const generation=models.editGeneration;modal.open({title:'保存模版新版本',body:'确认后保存为独立版本，历史版本保持，不后台重匹配渠道。',confirmLabel:'确认保存',onConfirm:()=>{if(generation!==models.editGeneration){models.error='保存确认已过期，请重新核对。';return}return models.save()}})}
function history(t:TemplateV2){historyId.value=t.id;historySelection.value=[];historySearch.value=''}
function compare(){compareRevision.value=models.draft?.baseRevision??'';compareOpen.value=true}
function openAction(kind:TemplateActionIntentV2['kind'],t?:TemplateV2,r?:RevisionV2){if(kind==='external_task'){externalOpen.value=true;return}if(kind==='agent_fill'){agentTrigger.value=document.activeElement instanceof HTMLElement?document.activeElement:null;agentOpen.value=true;return}if(kind==='agent_audit'){batchOpen.value=true;return}if(kind==='usage'&&t&&r){showUsage(t,r);return}if(kind==='set_default'&&t&&r){void prepareLibrary('set_default',t,r);return}if(kind==='delete_versions'&&t){void prepareLibrary('delete_revisions',t);return}const target=t??editingTemplate.value,buffer=models.draft;toolIntent.value={kind,expectedLibrary:models.snapshot?.libraryDigest??'',templateId:target?.id??buffer?.templateId??'',references:t?(r?[reference(t,r)]:t.revisions.filter(v=>historySelection.value.includes(v.id)).map(v=>reference(t,v))):target&&buffer?.baseRevision?target.revisions.filter(v=>v.id===buffer.baseRevision).map(v=>reference(target,v)):[],content:buffer?JSON.parse(JSON.stringify(buffer.content)):null,label:target?.label??buffer?.label??''};toolDraftGeneration=models.editGeneration;toolResult.value=null;toolError.value='';toolOpen.value=true;toolSequence++}
function closeTool(){toolOpen.value=false;toolSequence++;toolBusy.value=false}
async function runAction(){if(!toolIntent.value||toolBusy.value)return;const request=++toolSequence;toolBusy.value=true;toolError.value='';try{const value=await models.templateAction(toolIntent.value);if(request===toolSequence&&toolOpen.value)toolResult.value=value}catch(cause){if(request===toolSequence&&toolOpen.value)toolError.value=cause instanceof Error?cause.message:'操作尚未就绪，当前版本与编辑保持。'}finally{if(request===toolSequence)toolBusy.value=false}}
function applyProposal(){const proposal=toolResult.value?.proposal;if(!proposal||!models.draft)return;if(toolDraftGeneration!==models.editGeneration){toolError.value='建议形成后编辑已变化，未覆盖当前草稿；请重新核对。';return}models.setLabel(proposal.label);models.setContent(proposal.content);closeTool()}
const actionTitle=computed(()=>({set_default:'设置已保存默认版本',delete_versions:'删除所选版本',agent_fill:'Agent 自动填充',agent_audit:'模版检查',external_task:'外部任务',usage:'匹配与使用'})[toolIntent.value?.kind??'usage'])
watch(()=>models.externalNotificationTarget,value=>{if(value){externalOpen.value=true;tab.value='templates'}})
watch(tab,value=>{if(value==='templates')void models.load()})
onMounted(async()=>{if(tab.value==='templates')void models.load();try{const {listen}=await import('@tauri-apps/api/event');const stop=await listen('model-config-changed',()=>{if(tab.value==='channels')void models.loadChannels();else void models.load()});if(disposed)stop();else stopChanges=stop}catch{/* Consumer displays actual service failures. */}})
onBeforeUnmount(()=>{disposed=true;toolSequence++;usageSequence++;librarySequence++;stopChanges?.()})
</script>
<template>
  <section ref="page" class="models-page">
    <AppTopbar />
    <StickyTabs><div class="model-tabs-line"><div class="model-tabs ext-tabs" role="tablist" aria-label="模型配置"><button class="tab" :class="{active:tab==='channels'}" role="tab" :aria-selected="tab==='channels'" @click="tab='channels'">渠道模型</button><button class="tab" :class="{active:tab==='templates'}" role="tab" :aria-selected="tab==='templates'" @click="tab='templates'">模型模版</button></div><div id="model-tabs-actions"></div><AgentExecutionField v-if="tab==='templates'&&!editing" compact /></div></StickyTabs>
    <ChannelsView v-if="tab==='channels'" actions-target="#model-tabs-actions" />
    <template v-else>
      <p v-if="models.error" class="model-error" role="alert">{{models.error}}</p><p v-if="models.loading" role="status">正在读取保存模版…</p>
      <template v-if="!editing">
        <div class="model-toolbar"><input v-model="search" class="input" type="search" aria-label="搜索模版" placeholder="搜索模版" /><div class="model-inline"><UiButton v-if="models.draft" @click="models.view.templateEditor=true">继续编辑</UiButton><UiButton :disabled="models.loading" @click="models.load">重新读取</UiButton><UiButton variant="primary" :disabled="models.saving" @click="create">新建模版</UiButton><UiButton :disabled="!models.snapshot" @click="openAction('agent_audit')">{{models.view.selectedTemplates.length?'检查勾选更新':'检查全部更新'}}</UiButton><UiButton @click="openAction('external_task')">外部任务</UiButton></div></div>
        <div class="model-template-list"><article v-for="t in templates" :key="t.id" class="model-template"><div class="model-inline"><label><input v-model="models.view.selectedTemplates" type="checkbox" :value="t.id" />{{t.label}}</label><span class="tag">{{t.source==='builtin'?'内置':'自建'}}</span><UiSelectField :model-value="models.view.selectedVersions[t.id]??t.defaultRevision??''" :options="choices(t)" :label="t.label+'保存版本'" :disabled="models.saving||models.dirty" @update:model-value="chooseSavedVersion(t,$event)" /></div><p class="model-muted">{{selected(t)?.content.description??'请选择明确保存版本；不取任意最新候选。'}}</p><div class="model-inline"><UiButton :disabled="!selected(t)||models.saving" @click="start(t,selected(t))">编辑</UiButton><UiButton @click="history(t)">版本管理</UiButton><UiButton :disabled="!selected(t)" @click="openAction('usage',t,selected(t))">匹配与使用</UiButton><UiButton :disabled="models.saving||!selected(t)||t.defaultRevision===selected(t)?.id" @click="openAction('set_default',t,selected(t))">{{t.defaultRevision===selected(t)?.id?'当前默认':'设为默认'}}</UiButton><UiButton v-if="t.defaultRevision&&t.revisions.some(r=>r.id===t.defaultRevision)" :disabled="models.saving" @click="prepareLibrary('clear_default',t,t.revisions.find(r=>r.id===t.defaultRevision))">清除默认</UiButton></div></article></div><p v-if="models.snapshot&&!templates.length" class="model-muted">暂无符合条件的模版；可新建，不转换旧格式库。</p>
      </template>
      <div v-else-if="models.draft" class="model-v2-workspace" :class="{'with-agent':agentOpen}"><DetailWorkspace label="模版" :sections="sections" v-model:collapsed="models.view.templateNavCollapsed" v-model:active-id="models.view.templateSection">
        <template #header><UiButton size="sm" variant="ghost" @click="back">返回模版列表</UiButton><input class="input model-name" aria-label="模版名称" :value="models.draft.label" :disabled="models.saving" @input="models.setLabel(($event.target as HTMLInputElement).value)" /><UiSelectField v-if="editingTemplate" :model-value="models.draft.baseRevision??''" :options="choices(editingTemplate)" label="模版版本" :disabled="models.dirty||models.saving" @update:model-value="start(editingTemplate,editingTemplate!.revisions.find(r=>r.id===$event))" /><UiButton :disabled="!editingTemplate" @click="compare">与版本比较</UiButton><UiButton v-if="editingTemplate" @click="history(editingTemplate)">版本管理</UiButton><span class="tag">{{models.dirty?'未保存修改':'未修改'}}</span></template>
        <template #actions><UiButton size="sm" aria-label="Agent自动填充" :aria-expanded="agentOpen" aria-controls="model-agent-panel" @click="openAction('agent_fill')">Agent 自动填充</UiButton><UiButton size="sm" @click="openAction('external_task')">外部任务</UiButton><UiButton size="sm" :disabled="models.saving" @click="models.copyTemplate()">复制模版</UiButton><UiButton v-if="!models.draft.baseRevision" size="sm" @click="cancelNew">取消新建</UiButton></template>
        <template #primary><UiButton size="sm" :disabled="models.saving||!models.dirty" @click="restore">还原修改</UiButton><UiButton size="sm" variant="primary" :disabled="models.saving||!models.canSave||!models.dirty||!models.draft.label.trim()" @click="save">保存</UiButton></template>
        <div class="model-form"><section id="model-basic" tabindex="-1"><h2>基本信息</h2><label>说明<textarea class="input" :value="models.draft.content.description" :disabled="models.saving" @input="models.setContent({...models.draft!.content,description:($event.target as HTMLTextAreaElement).value})" /></label></section>
          <section id="model-matching" tabindex="-1"><ConditionEditor :model-value="models.draft.content" :issues="models.issues" :disabled="models.saving" @update:model-value="models.setContent" @preview="models.requestPreview" /></section>
          <section id="model-rules" tabindex="-1"><h2>族通用配置</h2><ModelRuleFields :model-value="models.draft.content.commonRules" :locked-fields="models.draft.content.lockedFields" :disabled="models.saving" @update:model-value="models.setContent({...models.draft!.content,commonRules:$event})" @update:locked-fields="models.setContent({...models.draft!.content,lockedFields:$event})" /></section>
          <section id="model-preview" tabindex="-1"><h2>匹配预览</h2><div class="model-columns"><label>原始模型 ID<input class="input" aria-label="原始模型ID" :value="models.previewModelId" @input="models.setPreviewTarget(($event.target as HTMLInputElement).value,models.previewChannelId)" /></label><label>渠道 ID<input class="input" aria-label="渠道ID" :value="models.previewChannelId" @input="models.setPreviewTarget(models.previewModelId,($event.target as HTMLInputElement).value)" /></label></div><UiButton :disabled="models.previewBusy||models.saving" @click="models.requestPreview">检查匹配</UiButton><ConditionPreview :result="models.preview" :busy="models.previewBusy" :error="models.previewError" :issues="models.issues" @retry="models.requestPreview" /></section>
          <section id="model-source" tabindex="-1"><h2>来源与版本</h2><p>{{models.draft.baseRevision?'基于确切已保存版本；保存生成新版本。':'新建未保存模版。'}}</p><p>{{models.draft.baseRevision}}</p><details v-for="(source,index) in models.draft.content.sources" :key="index"><summary>{{source.field}} · {{source.recordedAt}}</summary><p>{{source.url}}</p><p>{{source.note}}</p></details><label class="model-inline"><input type="checkbox" :checked="models.draft.makeDefault" :disabled="models.saving" @change="models.setDefault(($event.target as HTMLInputElement).checked)" />将本次新版本设为默认保存版本</label></section></div>
      </DetailWorkspace><AgentPanel v-if="models.snapshot" v-show="agentOpen" :draft="models.draft" :snapshot="models.snapshot" :generation="models.editGeneration" :session-key="'template:'+models.draft.templateId" :active="agentOpen" @close="closeAgent" @external="openAction('external_task')" /></div>
    </template>
    <UiDialog :open="!!models.policyNotificationTarget" title="模型配置原保存记录" wide @close="models.policyNotificationTarget=null"><ModelSaveHistory v-if="models.policyNotificationTarget" :initial-operation-id="models.policyNotificationTarget.operationId" @go="models.policyNotificationTarget=null" /></UiDialog>
    <ExternalTaskDialog v-if="models.snapshot" :open="externalOpen" :snapshot="models.snapshot" :draft="editing?models.draft:null" :generation="models.editGeneration" :selected="models.view.selectedTemplates" :versions="models.view.selectedVersions" @close="externalOpen=false" />
    <BatchAgentDialog v-if="models.snapshot" :open="batchOpen" :snapshot="models.snapshot" :selected="models.view.selectedTemplates" :versions="models.view.selectedVersions" @close="batchOpen=false" />
    <UiDialog panel-class="model-version-manager" :open="!!historyTemplate" :title="'版本管理 · '+(historyTemplate?.label??'')" wide :busy="models.saving||libraryWriting" @close="historyId=''">
      <template v-if="historyTemplate"><input v-model="historySearch" class="input" aria-label="搜索版本" placeholder="搜索版本时间或标识" /><article v-for="r in historyTemplate.revisions.slice().reverse().filter(r=>(r.createdAt+' '+r.id).includes(historySearch))" :key="r.id" class="model-version"><label><input v-model="historySelection" type="checkbox" :value="r.id" :disabled="models.saving||historyTemplate.defaultRevision===r.id" />{{r.createdAt}} · {{r.id.slice(0,8)}}<span v-if="historyTemplate.defaultRevision===r.id"> · 当前默认</span></label><div class="model-inline"><UiButton @click="start(historyTemplate,r);historyId=''">编辑</UiButton><UiButton :disabled="historyTemplate.defaultRevision===r.id" @click="openAction('set_default',historyTemplate,r)">设为默认</UiButton><UiButton v-if="historyTemplate.defaultRevision===r.id" :disabled="models.saving" @click="prepareLibrary('clear_default',historyTemplate,r)">清除默认</UiButton><UiButton @click="showUsage(historyTemplate,r)">使用关系</UiButton></div></article></template><template #actions><UiButton v-if="historyTemplate" variant="danger" :disabled="models.saving||!historySelection.length" @click="openAction('delete_versions',historyTemplate)">删除所选版本</UiButton></template>
    </UiDialog>
    <UiDialog :open="libraryOpen" :title="libraryTitle" :panel-class="libraryRequest?.intent.kind==='delete_revisions'?'model-version-delete':'model-version-default'" wide :busy="libraryWriting" @close="closeLibrary">
      <p>{{libraryLabel}} · {{libraryRequest?.intent.kind==='delete_revisions'?'只删除本次明确选择的保存版本。':'更改默认只影响下一次明确读取的版本选择，不后台重配渠道。'}}</p>
      <p v-if="libraryBusy" role="status">正在核对精确版本依赖…</p><p v-if="libraryError" role="alert" class="model-error">{{libraryError}}</p>
      <template v-if="libraryRequest?.intent.kind==='delete_revisions'"><p>内置初始、当前默认、保存/草稿精确引用和保留事务受到保护；后继版本同时删除时按完整选择核对。</p><article v-for="report in libraryReports" :key="report.reference.revisionId"><h3>{{report.reference.revisionId}}</h3><p v-if="deleteReasons(report,libraryRequest.intent.revisionIds).length" role="alert" class="model-error">{{deleteReasons(report,libraryRequest.intent.revisionIds).join('；')}}</p><p v-else>当前报告未发现阻断依赖，确认后仍由事务重新核对。</p></article></template>
      <dl v-else-if="libraryRequest"><dt>确切版本</dt><dd>{{libraryRequest.intent.reference.revisionId}}</dd><dt>摘要</dt><dd class="model-v2-reference">{{libraryRequest.intent.reference.digest}}</dd></dl>
      <p v-if="libraryRequest?.intent.kind==='clear_default'">将清除当前默认保存版本；不会改成任意最新版本，现有精确采用引用保持。</p>
      <template #actions="{close}"><UiButton :disabled="libraryWriting" @click="close">取消</UiButton><UiButton variant="primary" :disabled="libraryBusy||!!libraryError||deletionBlocked" :loading="libraryBusy" @click="confirmLibrary">确认操作</UiButton></template>
    </UiDialog>
    <UiDialog :open="usageOpen" title="精确版本使用关系" wide @close="closeUsage">
      <p v-if="usageBusy" role="status">正在读取精确引用与命中报告…</p><p v-if="usageError" role="alert" class="model-error">{{usageError}}</p>
      <template v-if="usageReport"><p>规则命中与实际采用分别记录；命中不代表配置已采用。</p><dl><dt>确切版本</dt><dd>{{usageReport.reference.revisionId}}</dd><dt>当前默认</dt><dd>{{usageReport.isDefault?'是':'否'}}</dd><dt>内置初始</dt><dd>{{usageReport.builtinInitial?'受保护':'否'}}</dd></dl><h3>精确引用与依赖</h3><p v-if="!usageReport.references.length">当前未报告精确引用。</p><ul><li v-for="(use,index) in usageReport.references" :key="index">{{usageStates[use.state]}} · {{usageKinds[use.kind]}}<span v-if="use.channelId"> · {{models.channels.find(c=>c.id===use.channelId)?.label??use.channelId}}</span><span v-if="use.rawModelId"> · {{use.rawModelId}}</span><span v-if="use.dependentRevision"> · 后继 {{use.dependentRevision.revisionId}}</span></li></ul><h3>模型命中与实际采用</h3><div class="model-table-scroll"><table class="model-table"><thead><tr><th>位置 / 模型</th><th>规则命中</th><th>实际采用本版本</th><th>引用来源</th></tr></thead><tbody><tr v-for="(row,index) in usageReport.models" :key="index"><td>{{usageStates[row.state]}} · {{row.channelLabel}}<code class="model-v2-usage-model">{{row.rawModelId}}</code></td><td>{{row.matched?'命中':'未命中'}}</td><td>{{row.usingQueriedRevision?'已采用':'未采用本版本'}}</td><td>{{row.bindingKind?usageKinds[row.bindingKind]:'无精确绑定'}}<p v-for="(issue,index) in row.issues" :key="index" class="model-error">{{issue.message}} · {{issue.code}}</p></td></tr></tbody></table></div></template>
      <template #actions><UiButton :disabled="usageBusy" @click="readUsage">重新读取报告</UiButton></template>
    </UiDialog>
    <UiDialog :open="compareOpen" title="与保存版本比较" wide @close="compareOpen=false"><template v-if="models.draft"><UiSelectField :model-value="compareRevision" :options="editingTemplate?choices(editingTemplate):[]" label="比较来源版本" @update:model-value="compareRevision=$event" /><p v-if="!comparison" role="alert">请选择可核验的精确保存版本；不猜测已缺失的内容。</p><div v-else class="model-columns"><section><h3>保存版本 · {{comparison.id.slice(0,8)}}</h3><pre class="model-v2-comparison">{{JSON.stringify(comparison.content,null,2)}}</pre></section><section><h3>当前未保存编辑</h3><pre class="model-v2-comparison">{{JSON.stringify(models.draft.content,null,2)}}</pre></section></div></template></UiDialog>
    <UiDialog :open="toolOpen&&toolIntent?.kind!=='agent_fill'" :title="actionTitle" wide :busy="toolBusy&&(toolIntent?.kind==='set_default'||toolIntent?.kind==='delete_versions')" @close="closeTool"><p>{{toolIntent?.label}} · 基于确切版本与当前编辑，旧版本保持。</p><ul v-if="toolIntent?.targets"><li v-for="target in toolIntent.targets" :key="target.templateId">{{target.label}} · {{target.reference?.revisionId??'缺少明确保存版本，需先核对'}}</li></ul><p v-if="toolIntent?.kind==='set_default'">将所选已保存版本设为默认；不后台重配渠道。</p><p v-if="toolIntent?.kind==='delete_versions'">确认后由后端核对精确依赖；有引用的版本不会被静默移除。</p><p v-if="toolError" role="alert" class="model-error">{{toolError}}</p><p v-if="toolResult?.report">{{toolResult.report}}</p><template v-if="toolResult?.proposal"><h3>待比较的建议</h3><pre class="model-v2-comparison">{{JSON.stringify(toolResult.proposal.content,null,2)}}</pre><UiButton @click="applyProposal">接受建议到编辑草稿</UiButton></template><template #actions="{close}"><UiButton :disabled="toolBusy&&(toolIntent?.kind==='set_default'||toolIntent?.kind==='delete_versions')" @click="close">关闭</UiButton><UiButton variant="primary" :disabled="toolBusy" :loading="toolBusy" @click="runAction">{{toolIntent?.kind==='set_default'||toolIntent?.kind==='delete_versions'?'确认操作':'开始核对'}}</UiButton></template></UiDialog>
  </section>
</template>
<style>
.model-tabs{display:flex;align-items:center;gap:var(--space-2);padding:10px 0}
.model-tabs{position:static;min-height:48px;background:transparent}
.model-tabs-line{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);flex-wrap:wrap}
/* MO-2（对齐原型 model-config.css:583/593）：这组控件不换行；状态框取与按钮相同的 34px 控件高度变量，
   按钮本身的高度由 .btn 公共配方保证（消费方不得覆盖控件度量）。 */
.model-policy-actions{--mp-policy-control-height:var(--control-md);justify-content:flex-end;padding:6px 0;flex-wrap:nowrap;gap:var(--space-2)}
/* 列表页脚说明（原型 model-config.js:865/867/967 .mp-footer-note）：紧贴列表下方，弱化字号，不进表格。 */
.model-footer-note{margin:var(--space-2) 0 0;color:var(--muted);font-size:var(--text-caption);line-height:1.7}
.model-source-record{padding:10px 0}.model-source-record p{white-space:pre-wrap;overflow-wrap:anywhere}
.model-editor-action{width:100%}.model-editor-action svg{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;flex:none}
.model-name{flex:1;min-width:120px;font-weight:var(--weight-semibold)}.model-identity{flex:1;min-width:180px}
/* 只有文本搜索框吃剩余宽度；复选框/单选不能套这条，否则 13px 勾选框被撑成 180px
   （0.2.7 实屏问题：模型列表「仅启用」被推远、控件高低不一）。 */
.model-toolbar{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);margin:20px 0;flex-wrap:wrap}
.model-toolbar>input[type="search"],.model-toolbar>input.input[type="text"],.model-toolbar>input.input:not([type]){flex:1 1 180px;max-width:360px;min-width:180px}
.model-table-scroll{overflow-x:auto}
.model-table{width:100%;border-collapse:collapse}
.model-table th{color:var(--muted);font-weight:500;text-align:left}
.model-table th,.model-table td{padding:14px 8px;border-bottom:1px solid var(--border);vertical-align:middle}
.template-title{display:flex;align-items:center;gap:var(--space-2);flex-wrap:wrap}/* 模版表选择列独立成列；名称列吃剩余宽度，版本与操作固定。 */.template-table{min-width:760px}.template-name-col{width:auto}.template-version-col{width:260px}.template-actions-col{width:270px}
.model-table .tag{margin-left:0}
.model-table small,.model-version small{display:block;margin-top:5px;color:var(--muted)}
.model-inline{display:flex;flex-wrap:wrap;align-items:center;gap:var(--space-2)}
.model-form{display:grid;gap:20px}
.model-form section{scroll-margin-top:calc(var(--page-tabs-height,48px) + var(--detail-header-height,48px) + 12px);padding-bottom:24px;border-bottom:1px solid var(--border)}
.model-form h2{font-size:var(--text-title);margin:0 0 16px}
.model-form label,.model-rule-fields>label,.model-columns>label{display:grid;gap:7px}
.model-form label.model-inline,.model-inline label{display:flex;align-items:center;gap:7px}
.model-rule-fields{margin:0;padding:0;border:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px;min-width:0}
.model-field-wide{grid-column:1/-1}
.model-field-title{display:flex;justify-content:space-between;align-items:center;gap:12px;margin:12px 0}
.model-field-title h2{margin:0}
.model-field-title label{display:flex;gap:6px;align-items:center}
.model-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin:14px 0}
.model-pattern{display:grid;grid-template-columns:110px minmax(0,1fr) auto;gap:8px;margin:8px 0}
.model-variant{border:1px solid var(--border);border-radius:var(--radius);padding:16px;margin-top:16px}
.model-variant>button{margin-top:16px}
.model-error{color:var(--red);overflow-wrap:anywhere}
.model-empty{padding:32px 0;text-align:center;color:var(--muted)}
.model-muted{color:var(--muted)}
.model-version{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:14px;padding:16px 0;border-bottom:1px solid var(--border)}
.model-version>label{display:flex;align-items:center;gap:10px}@media(max-width:800px){
.model-rule-fields,.model-columns{grid-template-columns:minmax(0,1fr)}
.model-pattern{grid-template-columns:90px minmax(0,1fr)}
.model-pattern>button{justify-self:end;grid-column:2}}
</style>

<style>
@container detail-content (max-width:520px){.model-rule-fields,.model-columns{grid-template-columns:minmax(0,1fr)}.model-pattern{grid-template-columns:minmax(0,1fr)}.model-pattern>button{justify-self:end}}
</style>
<style scoped>
.model-v2-workspace{display:grid;grid-template-columns:minmax(0,1fr);gap:16px;min-width:0}.model-v2-workspace.with-agent{grid-template-columns:minmax(0,1fr) minmax(260px,340px)}.model-v2-agent{min-width:0;position:sticky;top:16px;align-self:start;max-height:75vh;overflow:auto;padding:16px;border:1px solid var(--border);border-radius:12px;background:var(--panel);display:grid;gap:12px}@media(max-width:1100px){.model-v2-workspace.with-agent{grid-template-columns:minmax(0,1fr)}.model-v2-agent{position:static;max-height:none}}.model-v2-reference{overflow-wrap:anywhere}.model-v2-usage-model{display:block;overflow-wrap:anywhere}.model-v2-comparison{white-space:pre-wrap;overflow-wrap:anywhere;max-height:52vh;overflow:auto;background:var(--panel);border:1px solid var(--border);padding:12px;border-radius:8px;font-size:var(--text-caption)}
</style>
