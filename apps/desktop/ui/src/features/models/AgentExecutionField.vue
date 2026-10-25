<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ModelPicker,{type ModelChoice} from './ModelPicker.vue'
import {modelApi,errorText,type AgentSettings,type ExecutionDirectory,type ModelProjection} from './api'
import {useModelsStore} from './store'
const settings=defineModel<AgentSettings|null>({default:null})
defineProps<{disabled?:boolean;compact?:boolean}>()
const error=ref('');const busy=ref(false);const open=ref(false);const configuring=ref(false)
const loading=ref(false)
const candidate=ref<AgentSettings|null>(null);const directory=ref<ExecutionDirectory|null>(null)
const directoryLoaded=ref(false)
const models=useModelsStore()
/**
 * 打开选择器先用模型配置里已保存的渠道/型号即时呈现，避免每次等 Tauri 目录往返。
 * 这里只做显示与初选；真正写入时后端仍按官方目录与渠道身份重新绑定校验。
 */
const localDirectory=computed<ExecutionDirectory>(()=>{
 const policy=models.snapshot?.policy
 if(!policy)return {channels:[],rows:[]}
 const channels=policy.channels.filter(c=>c.connection.enabled)
 return {
  channels:channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId})),
  rows:channels.flatMap(c=>c.discovered.map((model):ModelProjection=>({channelId:c.id,providerId:c.connection.providerId,modelId:model.id,label:model.label,enabled:true,selected:true,discovered:true,templateRef:null,rules:model.rules,source:{}}))),
 }
})
const allRows=computed(()=>directory.value?.rows??localDirectory.value.rows)
const channels=computed(()=>directory.value?.channels??localDirectory.value.channels)
let disposed=false
onBeforeUnmount(()=>{disposed=true})
onMounted(async()=>{busy.value=true;try{const value=await modelApi.agentSettings();if(!disposed)settings.value=value}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}})
const current=computed<ModelChoice[]>(()=>{const value=candidate.value;const channel=channels.value.find(c=>c.providerId===value?.providerId);return value?[{channelId:value.channelId??channel?.id??value.providerId,modelId:value.modelId,...(value.reasoningEffort===null?{}:{reasoningEffort:value.reasoningEffort}),...(value.fast===null?{}:{fast:value.fast})}]:[]})
function configure(){if(busy.value)return;candidate.value=settings.value?{...settings.value}:null;error.value='';configuring.value=true}
let directoryRequest:Promise<void>|null=null
function refreshDirectory(){
 if(directoryLoaded.value)return Promise.resolve()
 if(directoryRequest)return directoryRequest
 loading.value=true
 directoryRequest=modelApi.executionDirectory().then(actual=>{
  if(!disposed){directory.value=actual;directoryLoaded.value=true}
 }).catch(cause=>{if(!disposed&&!localDirectory.value.rows.length)error.value=errorText(cause)}).finally(()=>{directoryRequest=null;if(!disposed)loading.value=false})
 return directoryRequest
}
function browse(){if(busy.value)return;error.value='';open.value=true;void refreshDirectory()}
function choose(picks:ModelChoice[]){const pick=picks[0];if(!pick||pick.nativeModelId)return;const row=allRows.value.find(r=>r.channelId===pick.channelId&&r.modelId===pick.modelId);if(!row||row.unavailableReason||!row.enabled)return;candidate.value={channelId:pick.channelId,connectionDigest:row.connectionDigest,providerId:row.providerId,modelId:pick.modelId!,reasoningEffort:pick.reasoningEffort??null,fast:pick.fast??null};open.value=false}
async function save(){if(!candidate.value||busy.value)return;busy.value=true;error.value='';try{const value=await modelApi.saveAgentSettings({...candidate.value});if(!disposed){settings.value=value;configuring.value=false}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
/** 档位中文名；与原型 levels 表一致，未知值原样显示。 */
const EFFORTS:Record<string,string>={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'}
const committedRow=computed(()=>{const value=settings.value;if(!value)return null;return allRows.value.find(r=>r.modelId===value.modelId&&(!value.channelId||r.channelId===value.channelId))??null})
const channelName=computed(()=>{const value=settings.value;if(!value)return '';return channels.value.find(c=>c.id===value.channelId||c.providerId===value.providerId)?.label??value.providerId})
/** 继承时取模型默认档位，并标记「（模型默认）」。 */
const effort=computed(()=>{const value=settings.value;if(!value)return{text:'',inherited:false};const inherited=value.reasoningEffort===null;const actual=inherited?(committedRow.value?.rules.defaultEffort.mode==='custom'?committedRow.value.rules.defaultEffort.value:null):value.reasoningEffort;return{text:actual?EFFORTS[actual]??actual:'未知',inherited}})
/** 快速模式的能力与选择是两回事：摘要说选择，事实卡说能力。 */
const fastCapability=computed<'supported'|'unsupported'|'unknown'>(()=>{const rules=committedRow.value?.rules.fast;if(!rules)return 'unknown';if(rules.mode!=='custom')return 'unknown';return rules.value?'supported':'unsupported'})
const fastCapabilityText=computed(()=>fastCapability.value==='supported'?'可用':fastCapability.value==='unsupported'?'不可用':'未知')
const fastSummary=computed(()=>{const value=settings.value;if(!value)return '';if(fastCapability.value==='supported')return value.fast?'快速已开启':'快速已关闭';if(fastCapability.value==='unsupported')return value.fast?'已选快速 · 当前模型不支持':'不支持快速模式';return value.fast?'已选快速 · 能力待核对':'快速能力待核对'})
const summary=computed(()=>{if(!settings.value)return '尚未选择执行模型 · 请配置执行模型';return channelName.value+' / '+settings.value.modelId+' · '+effort.value.text+(effort.value.inherited?'（模型默认）':'')+' · '+fastSummary.value})
</script>
<template>
 <div class="model-execution" :class="{'is-compact':compact}">
  <!-- MO-9（原型 model-config.js:425）：模版列表操作条内只有「Agent 配置」按钮，摘要不另起一行；
       已选模型由按钮 title 提供，完整事实在 Agent 配置面板里。 -->
  <template v-if="compact"><UiButton :disabled="disabled" :loading="busy" @click="configure" :title="summary" :aria-label="settings?'Agent 配置 · '+summary:'Agent 配置'">Agent 配置</UiButton></template>
  <template v-else>
   <!-- WP-55：头部只占一行。完整事实与边界说明收进可展开的说明，不再常驻占位。 -->
   <div class="model-execution-head">
    <div class="model-execution-title"><strong>模型模版助手 · 本机共用</strong><button v-if="settings" type="button" class="model-execution-entry" :title="summary" :aria-label="summary" :disabled="disabled || busy" @click="configure"><span>{{ channelName }} / <span class="model-execution-model">{{ settings.modelId }}</span></span><span class="model-execution-effort">{{ effort.text }}</span></button><small v-else class="model-execution-empty">{{ summary }}</small></div>
    <UiButton :variant="settings?'default':'primary'" :disabled="disabled" :loading="busy" @click="configure">{{ settings?'更换执行模型':'Agent 配置' }}</UiButton>
   </div>
   <details class="model-execution-more"><summary>{{ settings ? '执行模型与适用范围' : '为什么需要执行模型' }}</summary>
    <p v-if="!settings" class="model-execution-empty">尚未选择执行模型。自动填充与模版检查需要先配置一个可用的执行模型。</p>
    <div v-else class="model-execution-body"><dl class="model-execution-facts"><div><dt>执行渠道</dt><dd>{{ channelName }}</dd></div><div><dt>型号</dt><dd>{{ settings.modelId }}</dd></div><div><dt>推理档位</dt><dd>{{ effort.text }}<template v-if="effort.inherited">（模型默认）</template></dd></div><div><dt>快速模式</dt><dd>{{ fastCapabilityText }} · {{ fastSummary }}</dd></div></dl><div class="model-execution-usage"><h4>适用范围</h4><ul><li>模版自动填充、单个更新、所选更新与批量检查共用此设置。</li><li>不改变 Codex 的命名、搜索与子 Agent 配置。</li><li>修改只影响之后启动的任务；已启动任务继续使用启动时的设置。</li></ul></div><div class="model-execution-note"><strong>与运行配置的边界</strong><p>保存的是本机执行偏好，写入 manager-state/preferences.json 的 modelTemplateAgent 分区；不进入默认同步或普通导出。</p></div></div>
    <p v-if="error&&!configuring" role="alert" class="model-error">{{ error }}</p>
   </details>
  </template>
  <UiDialog :open="configuring" title="Agent 配置" :busy="busy" @close="configuring=false"><p>模型模版助手 · 本机共用</p><UiButton :loading="busy" @click="browse">{{ candidate?candidate.providerId+' / '+candidate.modelId:'选择执行模型' }}</UiButton><p v-if="candidate">{{ summary }}</p><p>此设置用于下一次自动填充、持续纠偏和批量模版检查。</p><p v-if="error" role="alert" class="model-error">{{ error }}</p><template #actions><UiButton @click="configuring=false">取消</UiButton><UiButton variant="primary" :disabled="!candidate" :loading="busy" @click="save">保存共用设置</UiButton></template></UiDialog>
  <ModelPicker :open="open" :loading="loading" :error="open?error:undefined" @retry="browse" :rows="allRows" :channels="channels" :current="current" reasoning title="执行模型" @close="open=false" @confirm="choose" />
 </div>
</template>
<style scoped>
.model-execution{display:grid;gap:16px;min-width:0}.model-execution.is-compact{gap:6px;width:max-content;max-width:100%}.model-execution.is-compact small{color:var(--muted);overflow-wrap:anywhere}
.model-execution-head{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);min-width:0}
.model-execution-title{display:flex;align-items:center;gap:var(--space-2);min-width:0;font-size:var(--text-caption)}
.model-execution-title>strong{flex:none;font-weight:var(--weight-semibold);font-size:var(--text-label)}
.model-execution-entry{display:inline-flex;align-items:center;gap:var(--space-1);min-width:0;max-width:100%;padding:2px var(--space-2);border:1px solid var(--border);border-radius:var(--radius-2xs);background:var(--raised);color:var(--text);font:inherit;font-size:var(--text-caption);cursor:pointer}
.model-execution-entry:hover:not(:disabled){border-color:var(--accent);background:var(--hover)}
.model-execution-entry:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.model-execution-entry span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-execution-model{font-family:var(--font-code);color:var(--text)}
.model-execution-effort{flex:none;color:var(--muted)}
.model-execution-more>summary{cursor:pointer;color:var(--muted);font-size:var(--text-caption)}
.model-execution-more[open]{display:grid;gap:var(--space-2)}
.model-execution-empty{margin:0;color:var(--muted);font-size:var(--text-caption)}
.model-execution-body{display:grid;gap:16px;min-width:0}
.model-execution-facts{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px;margin:0}
.model-execution-facts>div{display:grid;gap:4px;padding:12px 14px;border:1px solid var(--border);border-radius:var(--radius-sm);background:var(--rail)}
.model-execution-facts dt{color:var(--muted);font-size:var(--text-caption)}.model-execution-facts dd{margin:0;font-size:var(--text-control);font-weight:var(--weight-semibold);overflow-wrap:anywhere}
.model-execution-usage h4{margin:0 0 6px;font-size:var(--text-caption);color:var(--muted);font-weight:var(--weight-semibold)}
.model-execution-usage ul{margin:0;padding-left:18px;display:grid;gap:4px;font-size:var(--text-caption);color:var(--muted);line-height:1.6}
.model-execution-note{padding:12px 14px;border-radius:var(--radius-sm);background:var(--rail);font-size:var(--text-caption);line-height:1.6}
.model-execution-note strong{display:block;margin-bottom:4px}.model-execution-note p{margin:0;color:var(--muted)}
.model-error{color:var(--red);overflow-wrap:anywhere}
</style>

