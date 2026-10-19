<script setup lang="ts">
import {computed,onMounted,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {errorText,modelApi,type ModelHistoryEntry,type ModelRecoveryPreview} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
import {useRouteStore} from '@/stores/routes'
const models=useModelsStore();const routes=useRouteStore()
const records=ref<ModelHistoryEntry[]>([]);const query=ref('');const error=ref('');const busy=ref(false)
const selected=ref<ModelHistoryEntry|null>(null);const side=ref<'before'|'after'>('before');const scope=ref<'affected'|'all'>('affected');const restoreDefaults=ref(false)
const preview=ref<ModelRecoveryPreview|null>(null);const restored=ref(false)
const visible=computed(()=>records.value.filter(r=>(r.createdAt+' '+r.id+' '+r.channelLabels.join(' ')).toLowerCase().includes(query.value.toLowerCase())))
async function perform(action:()=>Promise<void>){if(busy.value)return;busy.value=true;error.value='';try{await action()}catch(cause){error.value=errorText(cause)}finally{busy.value=false}}
async function refresh(){await perform(async()=>{records.value=await modelApi.policyHistory()})}
function choose(record:ModelHistoryEntry){selected.value=record;side.value='before';scope.value='affected';restoreDefaults.value=false;preview.value=null;restored.value=false;error.value=''}
function invalidate(){preview.value=null;restored.value=false}
async function compare(){if(!selected.value)return;await perform(async()=>{
 if(!models.snapshot)await models.load()
 if(!models.snapshot)throw {message:'无法读取当前模型配置'}
 if(models.channelDraft){await models.persistDraft();if(models.channelError)throw {message:models.channelError}}
 const snapshot=models.snapshot
 preview.value=await modelApi.prepareRecovery({sourceId:selected.value!.id,side:side.value,scope:scope.value,restoreDefaults:restoreDefaults.value,expectedLibrary:snapshot.libraryDigest,expectedPolicy:snapshot.policyDigest,expectedWorkspace:snapshot.workspaceDigest})
})}
async function accept(){if(!preview.value)return;await perform(async()=>{const saved=await modelApi.acceptRecovery(preview.value!.id);models.acceptTransfer(saved);restored.value=true})}
function close(){if(!busy.value){selected.value=null;preview.value=null;error.value=''}}
function date(value:string){return new Date(value).toLocaleString()}
function field(value:string){const name=value.split('/').at(-1)??value;return ({connection:'连接与节点',templateRefs:'模版采用',modelRefs:'单模型采用',selection:'模型选择',modelOrder:'模型顺序',discovered:'模型定义',defaults:'渠道默认能力',overrides:'单模型覆盖',familyOverrides:'模型族覆盖',label:'渠道名称',channelOrder:'渠道组顺序',globalTemplateRefs:'全局模版采用'} as Record<string,string>)[name]??(value.startsWith('default/')?'模版默认版本':'渠道对象')}
function content(value:unknown):string{
 if(value===null||value===undefined)return '无'
 if(typeof value==='boolean')return value?'是':'否'
 if(Array.isArray(value))return value.length?value.map(content).join('\n'):'无'
 if(typeof value!=='object')return String(value)
 const object=value as Record<string,unknown>
 if(object.mode==='inherit')return '继承'
 if(object.mode==='custom')return content(object.value)
 if(object.mode==='all')return '全部模型'
 if(object.mode==='none')return '不选择模型'
 if(object.mode==='whitelist')return '指定模型\n'+content(object.models)
 if(typeof object.templateId==='string'&&typeof object.revisionId==='string')return (models.snapshot?.library.templates.find(t=>t.id===object.templateId)?.label??object.templateId)+' · '+object.revisionId.slice(0,8)
 const labels:Record<string,string>={id:'标识',name:'名称',url:'服务地址',providerId:'运行渠道',protocol:'默认协议',nodes:'服务节点',activeNodeId:'当前节点',credentialRef:'凭据',enabled:'启用',allowPrivateNetwork:'允许私网',contextWindow:'上下文窗口',inputModalities:'输入类型',reasoningEfforts:'推理等级',defaultEffort:'默认推理',fast:'快速模式',templateRefs:'模版采用',modelRefs:'单模型采用',selection:'模型选择',modelOrder:'模型顺序',discovered:'模型定义',defaults:'渠道默认',overrides:'单模型覆盖',familyOverrides:'模型族覆盖',label:'显示名称',rules:'能力规则'}
 return Object.entries(object).map(([key,item])=>(labels[key]??key)+'：'+content(item)).join('\n')
}
onMounted(()=>void refresh())
</script>
<template>
 <section class="model-save-history" aria-label="模型配置保存记录">
  <div class="model-history-head"><div><h3>模型配置保存记录</h3><p>从成功记录生成恢复草稿，再到模型页确认保存。</p></div><UiButton :loading="busy" @click="refresh">刷新保存记录</UiButton></div>
  <input v-model="query" class="input" type="search" aria-label="搜索模型保存记录" placeholder="搜索时间、渠道或记录标识" />
  <p v-if="error&&!selected" class="model-history-error" role="alert">{{ error }}</p>
  <p v-if="!busy&&!visible.length">暂无符合条件的保存记录。</p>
  <ul class="model-history-list"><li v-for="record in visible" :key="record.id"><div><strong>{{ date(record.createdAt) }}</strong><span>{{ record.channelLabels.join('、')||'全局配置' }}</span><small>{{ record.phase==='committed'?'已成功保存':record.phase==='compensated'?'本次写入已撤回':'待核对的执行记录' }}{{ record.phase==='committed'&&!record.recoverable?' · 缺少组合基线':'' }}</small><code>{{ record.id }}</code></div><UiButton size="sm" :disabled="busy||!record.recoverable" @click="choose(record)">生成恢复草稿</UiButton></li></ul>
  <UiDialog :open="!!selected" title="恢复模型配置到草稿" :busy="busy" wide @close="close">
   <p v-if="selected">保存记录：{{ date(selected.createdAt) }}</p>
   <div class="model-recovery-controls"><UiSelectField v-model="side" label="恢复目标" :disabled="busy||restored" :options="[{value:'before',label:'本次保存前的配置'},{value:'after',label:'本次成功保存的配置'}]" @update:model-value="invalidate" /><UiSelectField v-model="scope" label="恢复范围" :disabled="busy||restored" :options="[{value:'affected',label:'仅本次影响范围'},{value:'all',label:'完整渠道策略'}]" @update:model-value="invalidate" /></div>
   <label><input v-model="restoreDefaults" type="checkbox" :disabled="busy||restored" @change="invalidate" />同时恢复该快照的模版默认指针</label>
   <p v-if="error" class="model-history-error" role="alert">{{ error }}</p>
   <template v-if="preview"><p>影响渠道：{{ preview.channelLabels.join('、')||'全局配置或渠道移除' }}</p><p v-for="notice in preview.notices" :key="notice">{{ notice }}</p><p v-if="!preview.requiresRuntimeReview&&!preview.changes.length&&!preview.defaultChanges.length">所选目标与当前草稿相同。</p><div v-for="(change,index) in [...preview.changes,...preview.defaultChanges]" :key="index" class="model-recovery-change"><h4>{{ field(change.field) }}</h4><div class="model-recovery-values"><div><strong>当前</strong><pre>{{ content(change.before) }}</pre></div><div><strong>恢复后</strong><pre>{{ content(change.after) }}</pre></div></div></div></template>
   <p v-if="restored" role="status">已放入渠道草稿，运行配置尚未改变。</p>
   <template #actions><UiButton @click="close">关闭</UiButton><UiButton v-if="restored" variant="primary" @click="routes.go('models')">前往模型页保存</UiButton><UiButton v-else-if="!preview" variant="primary" :loading="busy" @click="compare">比较恢复差异</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="!preview.requiresRuntimeReview&&!preview.changes.length&&!preview.defaultChanges.length" @click="accept">放入恢复草稿</UiButton></template>
  </UiDialog>
 </section>
</template>
<style scoped>
.model-save-history{border-top:1px solid var(--border);padding-top:20px;margin-top:24px}.model-history-head{display:flex;align-items:center;justify-content:space-between;gap:16px;margin-bottom:12px}.model-history-head p,.model-history-list small{color:var(--muted)}h3{margin:0}.model-history-list{list-style:none;padding:0;display:grid;gap:12px}.model-history-list li{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:12px 0;border-bottom:1px solid var(--border)}.model-history-list li>div{display:grid;gap:4px;min-width:0}.model-history-list code{overflow-wrap:anywhere;font-size:var(--text-small)}.model-recovery-controls,.model-recovery-values{display:grid;grid-template-columns:1fr 1fr;gap:16px}.model-recovery-change{border-top:1px solid var(--border);margin-top:16px}.model-recovery-values>div{min-width:0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:var(--text-small);max-height:260px;overflow:auto;padding:10px;background:var(--panel)}.model-history-error{color:var(--red)}@media(max-width:650px){.model-history-head,.model-history-list li{align-items:flex-start;flex-direction:column}.model-recovery-controls,.model-recovery-values{grid-template-columns:1fr}}
</style>
