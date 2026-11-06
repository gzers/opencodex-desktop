<script setup lang="ts">
import {computed,onMounted,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ChannelEditor from './ChannelEditor.vue'
import ModelPicker from './ModelPicker.vue'
import NewChannelDialog from './NewChannelDialog.vue'
import {useModelsV2Store} from './store-v2'
import {channelNameErrorV2} from './channelRename'
const props=withDefaults(defineProps<{actionsTarget?:string;active?:boolean}>(),{active:true})
const models=useModelsV2Store(),newChannelOpen=ref(false),saveOpen=ref(false),pickerOpen=ref(false)
const editor=computed(()=>models.channels.find(c=>c.id===models.channelEditorId))
const filtered=computed(()=>models.channels.filter(c=>(c.label+' '+c.id).toLowerCase().includes(models.channelSearch.toLowerCase())))
const nameErrors=computed(()=>models.channels.map(c=>channelNameErrorV2(c.label,c.id,models.channels)).filter(Boolean))
const counts=computed(()=>models.readResults.reduce((total,result)=>{for(const key of Object.keys(total) as (keyof typeof total)[])total[key]+=result.counts[key];return total},{total:0,matched:0,manualPreserved:0,unmatched:0,missingDefault:0,missingDependency:0,conflict:0,changed:0,protocolPending:0}))
let reviewBase=''
function currentReview(){return JSON.stringify({base:models.file?.draft?.base,source:models.file?.sourceDigest,library:models.file?.libraryDigest,channelIntents:models.channelIntents,edits:models.modelEdits})}
function prepareSave(){if(!models.canSaveFile||nameErrors.value.length)return;reviewBase=currentReview();saveOpen.value=true}
async function confirmSave(){if(reviewBase!==currentReview()){models.fileError='保存预览已过期，当前输入已保留，请重新核对。';return}await models.saveChannels(editor.value?.id);if(!models.fileError)saveOpen.value=false}
async function discardLocal(){await models.loadChannels(true)}
onMounted(()=>{if(props.active!==false)void models.loadChannels()})
</script>
<template>
  <section class="channels-v2" aria-label="渠道模型配置">
    <p v-if="models.fileError" role="alert" class="model-error">{{models.fileError}}</p>
    <p v-if="models.fileBusy" role="status">正在核对当前渠道与草稿…</p>
    <p v-if="models.fileNeedsReconcile" role="alert" class="model-error">读取期间有新的明确编辑，当前输入保持。请核对来源后重新读取。</p>
    <div class="model-toolbar"><div class="model-inline"><UiButton :disabled="models.fileBusy||!models.file||!!models.channelIntents.length||!!models.modelEdits.length" @click="pickerOpen=true">预览当前模型目录</UiButton><UiButton :loading="models.fileBusy" :disabled="models.fileBusy" @click="models.loadChannels()">刷新当前快照</UiButton><UiButton :disabled="models.fileBusy||!models.file" @click="models.readExisting">读取现有渠道</UiButton><UiButton :disabled="models.fileBusy||!models.file" @click="newChannelOpen=true">新增渠道</UiButton></div><div class="model-inline"><UiButton :disabled="models.fileBusy||!models.channelIntents.length&&!models.modelEdits.length" @click="discardLocal">还原本地明确编辑</UiButton><UiButton variant="primary" :disabled="!models.canSaveFile||!models.fileDirty||!!nameErrors.length" @click="prepareSave">保存配置</UiButton></div></div>
    <p v-if="models.readResults.length" role="status">上次成功读取 · 自动匹配 {{counts.matched}} · 更新草稿 {{counts.changed}} · 保留人工 {{counts.manualPreserved}} · 未匹配 {{counts.unmatched}} · 缺默认版本 {{counts.missingDefault}} · 缺依赖 {{counts.missingDependency}} · 冲突 {{counts.conflict}} · 协议待配置 {{counts.protocolPending}}。保存配置后才生效。</p>
    <p v-if="models.file?.draft" class="model-muted">当前为服务端草稿；普通快照/开始编辑不重新选择模版。只有“读取现有渠道”才自动匹配已保存模版。</p>
    <ChannelEditor v-if="editor" :channel="editor" :channels="models.channels" :effective="models.file?.effective" :busy="models.fileBusy" :dirty="models.fileDirty" :save-disabled="!models.canSaveFile||!!nameErrors.length" @close="models.channelEditorId=''" @save="prepareSave" @intent="models.queueChannel" @edit="models.queueModel" />
    <template v-else>
      <input v-model="models.channelSearch" class="input" type="search" aria-label="搜索渠道" placeholder="搜索渠道名称或标识" />
      <div class="model-table-scroll"><table class="model-table"><thead><tr><th>渠道</th><th>当前服务节点</th><th>状态与数量</th><th>操作</th></tr></thead><tbody><tr v-for="channel in filtered" :key="channel.id"><td><strong>{{channel.label}}</strong><small>{{channel.id}}</small></td><td><span>{{channel.connection.nodes.find(n=>n.id===channel.connection.activeNodeId)?.name??'无当前节点'}}</span><small>{{channel.connection.nodes.find(n=>n.id===channel.connection.activeNodeId)?.url??'未配置'}}</small></td><td><label><input type="checkbox" :aria-label="'启用渠道 '+channel.label" :checked="channel.connection.enabled" :disabled="models.fileBusy" @change="models.queueChannel({kind:'set_enabled',channelId:channel.id,enabled:($event.target as HTMLInputElement).checked})" />{{channel.connection.enabled?'已启用':'已停用'}}</label><span>已选 {{channel.selection.mode==='all'?channel.models.length:channel.selection.mode==='whitelist'?channel.selection.models.length:0}} / {{channel.models.length}}</span></td><td><div class="model-inline"><UiButton size="sm" :disabled="models.fileBusy" @click="models.channelEditorId=channel.id">编辑</UiButton><UiButton size="sm" disabled title="复制暂不可用；当前配置保持">复制</UiButton><UiButton size="sm" disabled title="删除暂不可用；当前配置保持">删除</UiButton></div></td></tr></tbody></table></div>
      <p v-if="models.file&&!models.channels.length&&!models.fileBusy" class="model-muted">当前没有可编辑渠道。可明确读取新 HOME 的配置，或新增渠道；读取失败不会显示为成功空列表。</p>
      <p v-if="!models.file&&!models.fileBusy" class="model-muted">渠道快照尚未就绪，请重新读取。</p>
    </template>
    <ModelPicker :open="pickerOpen" :rows="[]" :v2-rows="models.file?.effective" :channels="models.channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId}))" title="当前服务端渠道模型目录" @close="pickerOpen=false" @confirm="pickerOpen=false" />
    <NewChannelDialog v-if="newChannelOpen" @close="newChannelOpen=false" @added="models.channelEditorId=$event" />
    <UiDialog :open="saveOpen" title="保存当前渠道草稿" wide :busy="models.fileBusy" @close="saveOpen=false">
      <p>本次提交当前服务端草稿及以下明确编辑。渠道名、运行键与模型配置由同一事务核对；未确认前不写正式配置。</p>
      <p v-if="models.fileError" role="alert" class="model-error">{{models.fileError}}</p>
      <ul><li v-for="channel in models.channels.filter(c=>JSON.stringify(c)!==JSON.stringify(models.file?.savedPolicy.channels.find(saved=>saved.id===c.id)))" :key="channel.id">{{channel.label}} · 有草稿变化</li><li v-for="(intent,index) in models.channelIntents" :key="'c'+index">{{models.channels.find(c=>c.id===intent.channelId)?.label??intent.channelId}} · {{intent.kind==='rename'?'名称 / 实际运行键':intent.kind==='set_enabled'?'启停':intent.kind==='set_selection'?'模型公开范围':'模型顺序'}}</li><li v-for="(edit,index) in models.modelEdits" :key="'m'+index">{{edit.rawModelId}} · {{'field'in edit?edit.field:'明确模版引用'}} · {{edit.kind==='clear_override'?'恢复继承':'明确人工设置'}}</li></ul>
      <p v-if="!models.fileDirty" class="model-muted">没有待保存修改；明确详情保存仍会核对名称与运行键。</p>
      <template #actions="{close}"><UiButton :disabled="models.fileBusy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="!models.canSaveFile||!!nameErrors.length" :loading="models.fileBusy" @click="confirmSave">确认保存</UiButton></template>
    </UiDialog>
  </section>
</template>
<style scoped>
.channels-v2{display:grid;gap:var(--space-4);min-width:0}.channels-v2 .model-table-scroll{overflow:auto}.channels-v2 table{width:100%;min-width:660px}.channels-v2 th,.channels-v2 td{padding:12px;text-align:left;vertical-align:top;border-bottom:1px solid var(--border)}.channels-v2 small{display:block;overflow-wrap:anywhere;color:var(--muted)}.channels-v2 label{display:flex;gap:var(--space-2);align-items:center}.channels-v2 li{overflow-wrap:anywhere}
</style>
