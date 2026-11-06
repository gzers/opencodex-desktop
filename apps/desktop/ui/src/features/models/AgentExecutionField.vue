<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ModelPicker,{type ModelChoice} from './ModelPicker.vue'
import {modelV2Api,type AgentSettingsV2,type ExecutionDirectoryV2} from './api-v2'
import {errorText} from './api'
import {useModelsV2Store} from './store-v2'
const settings=defineModel<AgentSettingsV2|null>({default:null})
const props=defineProps<{disabled?:boolean;compact?:boolean}>()
const error=ref(''),busy=ref(false),open=ref(false),configuring=ref(false),loading=ref(false)
const candidate=ref<AgentSettingsV2|null>(null),directory=ref<ExecutionDirectoryV2|null>(null)
const models=useModelsV2Store()
const allRows=computed(()=>directory.value?.rows??[]),channels=computed(()=>directory.value?.channels??[])
let disposed=false,directoryRequest=0
onBeforeUnmount(()=>{disposed=true;directoryRequest++})
watch(()=>models.executionPreferenceRevision,()=>{settings.value=models.executionPreference?{...models.executionPreference}:null},{flush:'sync'})
onMounted(async()=>{busy.value=true;try{await models.loadAgentSettings();if(!disposed)settings.value=models.executionPreference?{...models.executionPreference}:null}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}})
const current=computed<ModelChoice[]>(()=>{const value=candidate.value;return value?.channelId?[{channelId:value.channelId,modelId:value.modelId,...(value.reasoningEffort===null?{}:{reasoningEffort:value.reasoningEffort}),...(value.fast===null?{}:{fast:value.fast})}]:[]})
function configure(){if(busy.value||props.disabled)return;candidate.value=settings.value?{...settings.value}:null;error.value='';configuring.value=true}
function closePicker(){open.value=false;directoryRequest++;loading.value=false}
function closeConfiguration(){if(busy.value)return;closePicker();configuring.value=false;candidate.value=null}
watch(()=>[models.file?.sourceDigest,models.file?.rootRevision,models.file?.savedPolicy],()=>{directoryRequest++;directory.value=null;loading.value=false;if(open.value)void refreshDirectory()},{flush:'sync'})
async function refreshDirectory(){const request=++directoryRequest;directory.value=null;error.value='';loading.value=true;try{const actual=await modelV2Api.executionDirectory();if(actual.dtoVersion!==2)throw new Error('执行目录格式不支持，已有偏好保持。');if(!disposed&&request===directoryRequest)directory.value=actual}catch(cause){if(!disposed&&request===directoryRequest)error.value=errorText(cause)}finally{if(!disposed&&request===directoryRequest)loading.value=false}}
function browse(){if(busy.value||props.disabled)return;error.value='';open.value=true;void refreshDirectory()}
function choose(picks:ModelChoice[]){if(loading.value||error.value)return;const pick=picks[0];if(!pick||pick.nativeModelId)return;const matches=directory.value?.rows.filter(row=>row.channelId===pick.channelId&&row.modelId===pick.modelId&&!row.nativeModelId)??[];if(matches.length!==1){error.value='执行目录身份不唯一或已变化，请重新读取。';return}const row=matches[0]!;if(row.unavailableReason||!row.selected||!row.enabled||!row.pickerVisible||!row.connectionDigest||!row.actualSelector||row.protocol.kind!=='resolved'){error.value=row.unavailableReason??'此模型尚无可核验的执行绑定。';return}candidate.value={channelId:row.channelId,connectionDigest:row.connectionDigest,providerId:row.providerId,modelId:row.modelId,reasoningEffort:pick.reasoningEffort??null,fast:pick.fast??null};open.value=false}
async function save(){if(!candidate.value||busy.value||props.disabled)return;busy.value=true;error.value='';try{const value=await models.bindAgent({...candidate.value});if(!disposed){settings.value=value;configuring.value=false}}catch(cause){if(!disposed)error.value=errorText(cause)}finally{if(!disposed)busy.value=false}}
const EFFORTS:Record<string,string>={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'}
const committedRow=computed(()=>{const value=settings.value;return !value?null:allRows.value.find(row=>row.channelId===value.channelId&&row.providerId===value.providerId&&row.modelId===value.modelId)??null})
const channelName=computed(()=>{const value=settings.value;return !value?'':channels.value.find(c=>c.id===value.channelId)?.label??value.providerId})
const effort=computed(()=>{const value=settings.value;if(!value)return{text:'',inherited:false};const inherited=value.reasoningEffort===null,actual=inherited?(committedRow.value?.rules.defaultEffort.mode==='custom'?committedRow.value.rules.defaultEffort.value:null):value.reasoningEffort;return{text:actual?EFFORTS[actual]??actual:'未知',inherited}})
const fastCapability=computed<'supported'|'unsupported'|'unknown'>(()=>{const rule=committedRow.value?.rules.fast;return !rule||rule.mode!=='custom'?'unknown':rule.value?'supported':'unsupported'})
const fastCapabilityText=computed(()=>fastCapability.value==='supported'?'可用':fastCapability.value==='unsupported'?'不可用':'未知')
const fastSummary=computed(()=>!settings.value?'':fastCapability.value==='supported'?(settings.value.fast?'快速已开启':'快速已关闭'):settings.value.fast?'已选快速 · 当前能力待核对':'快速能力待核对')
const summary=computed(()=>!settings.value?'尚未选择执行模型 · 请配置执行模型':channelName.value+' / '+settings.value.modelId+' · '+effort.value.text+(effort.value.inherited?'（模型默认）':'')+' · '+fastSummary.value)
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
    <div v-else class="model-execution-body"><dl class="model-execution-facts"><div><dt>执行渠道</dt><dd>{{ channelName }}</dd></div><div><dt>型号</dt><dd>{{ settings.modelId }}</dd><dt>实际执行绑定</dt><dd>{{committedRow?.actualSelector??'尚未重新读取实际目录'}}</dd></div><div><dt>推理档位</dt><dd>{{ effort.text }}<template v-if="effort.inherited">（模型默认）</template></dd></div><div><dt>快速模式</dt><dd>{{ fastCapabilityText }} · {{ fastSummary }}</dd></div></dl><div class="model-execution-usage"><h4>适用范围</h4><ul><li>模版自动填充、单个更新、所选更新与批量检查共用此设置。</li><li>不改变 Codex 的命名、搜索与子 Agent 配置。</li><li>修改只影响之后启动的任务；已启动任务继续使用启动时的设置。</li></ul></div><div class="model-execution-note"><strong>与运行配置的边界</strong><p>保存的是本机执行偏好，写入 manager-state/preferences.json 的 modelTemplateAgent 分区；不进入默认同步或普通导出。</p></div></div>
    <p v-if="error&&!configuring" role="alert" class="model-error">{{ error }}</p>
   </details>
  </template>
  <UiDialog :open="configuring" title="Agent 配置" panel-class="agent-execution-dialog" :busy="busy" @close="closeConfiguration"><p>模型模版助手 · 本机共用</p><UiButton :loading="busy" @click="browse">{{ candidate?candidate.providerId+' / '+candidate.modelId:'选择执行模型' }}</UiButton><p v-if="candidate">{{ summary }}</p><p>此设置用于下一次自动填充、持续纠偏和批量模版检查。</p><p v-if="error" role="alert" class="model-error">{{ error }}</p><template #actions="{ close }"><UiButton :disabled="busy" @click="close">取消</UiButton><UiButton variant="primary" :disabled="!candidate" :loading="busy" @click="save">保存共用设置</UiButton></template></UiDialog>
  <ModelPicker :open="open" :loading="loading" :error="open?error:undefined" @retry="browse" :rows="[]" :execution-rows="allRows" :channels="channels" :current="current" reasoning title="执行模型" @close="closePicker" @confirm="choose" />
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

