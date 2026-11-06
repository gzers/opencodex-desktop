<script setup lang="ts">
import {computed} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import {originLabelV2,type ConditionPreviewView,type MatchIssueV2} from './api-v2'
import type {Field} from './api'
const props=defineProps<{result:ConditionPreviewView|null;busy?:boolean;error?:string;issues?:readonly MatchIssueV2[];disabled?:boolean}>()
const emit=defineEmits<{retry:[]}>()
const fields=[['protocol','请求协议'],['contextWindow','上下文窗口'],['inputModalities','输入模态'],['reasoningEfforts','推理档位'],['defaultEffort','默认档位'],['fast','Fast'],['pinnedEffort','强制档位'],['effortMap','档位映射'],['noReasoning','不传推理参数'],['reasoningAuthoritative','推理列表权威'],['fastWire','Fast wire']] as const
const configuration=computed(()=>props.result?.configuration??props.result?.effective??null)
const allIssues=computed(()=>props.issues??props.result?.issues??[])
function value(field:Field<unknown>|undefined){if(!field||field.mode==='inherit')return '无声明';if(field.value===null)return '明确禁用';if(typeof field.value==='boolean')return field.value?'是':'否';if(Array.isArray(field.value))return field.value.length?field.value.join(' / '):'明确空集合';return typeof field.value==='object'?JSON.stringify(field.value):String(field.value)}
</script>
<template>
  <section class="condition-preview" aria-label="条件匹配预览" aria-live="polite" :aria-busy="busy">
    <h2>匹配结果与字段来源</h2>
    <p v-if="busy" role="status">正在核对条件与配置来源…</p>
    <p v-else-if="error" role="alert" class="model-error">{{error}}</p>
    <p v-else-if="!result" class="model-muted">尚无当前输入的预览结果。修改条件后需要重新核对。</p>
    <template v-if="result&&!busy&&!error">
      <p role="status">{{result.reason}}</p>
      <dl><dt>原始模型 ID</dt><dd>{{result.rawModelId}}</dd><dt>匹配视图</dt><dd>{{result.matchingView??'未返回独立匹配视图'}}</dd></dl>
      <article v-for="condition in result.conditions" :key="condition.conditionId">
        <h3>{{condition.matched?'命中覆盖':'未命中覆盖'}}</h3><p>{{condition.reason}}</p>
        <p v-if="!condition.atoms.length" class="model-muted">当前回执未提供逐原子解释，不推断缺失的条件结果。</p><ol><li v-for="atom in condition.atoms" :key="atom.index">条件 {{atom.index+1}} · {{atom.matched?'满足':'不满足'}} · {{atom.reason}}</li></ol>
      </article>
      <template v-if="configuration">
        <p v-if="configuration.protocol.kind==='missing'" class="model-error" role="alert">该模型请求协议待配置；不会使用渠道默认协议。</p>
        <p v-else-if="configuration.protocol.kind==='unsupported'" class="model-error" role="alert">暂不支持的协议：{{configuration.protocol.value}}；保留实际来源。</p>
        <p v-else>最终请求协议：{{configuration.protocol.value}} · {{originLabelV2(configuration.protocol.origin)}}</p>
        <div class="model-table-scroll"><table class="model-table"><caption>本次有效字段与来源</caption><thead><tr><th>字段</th><th>有效值</th><th>来源</th></tr></thead><tbody><tr v-for="[key,label] in fields" :key="key"><th>{{label}}</th><td>{{key==='protocol'&&configuration.protocol.kind==='resolved'?configuration.protocol.value:value(configuration.rules[key])}}</td><td>{{originLabelV2(configuration.fieldOrigins[key])}}</td></tr></tbody></table></div>
        <details v-if="result.effective?.binding"><summary>精确模版版本与来源</summary><dl><dt>引用来源</dt><dd>{{result.effective.binding.kind==='manual'?'人工指定':'读取自动匹配'}}</dd><dt>模版</dt><dd>{{result.effective.binding.reference.templateId}}</dd><dt>版本</dt><dd>{{result.effective.binding.reference.revisionId}}</dd><dt>摘要</dt><dd>{{result.effective.binding.reference.digest}}</dd></dl></details>
      </template>
    </template>
    <ul v-if="allIssues.length" class="model-error"><li v-for="(issue,index) in allIssues" :key="index" role="alert">{{issue.message}} · {{issue.code}}<span v-if="issue.fieldPath"> · {{issue.fieldPath}}</span></li></ul>
    <UiButton v-if="error" size="sm" :disabled="disabled||busy" @click="emit('retry')">重新核对</UiButton>
  </section>
</template>
<style scoped>
.condition-preview{min-width:0;display:grid;gap:var(--space-3)}.condition-preview h2,.condition-preview h3{font-size:var(--text-body);margin:0}.condition-preview dl{display:grid;grid-template-columns:minmax(80px,auto) minmax(0,1fr);gap:var(--space-2)}.condition-preview dt{color:var(--muted)}.condition-preview dd{margin:0;overflow-wrap:anywhere;white-space:pre-wrap}.condition-preview p,.condition-preview li,.condition-preview td{overflow-wrap:anywhere}.condition-preview table{min-width:420px}.condition-preview .model-table-scroll{overflow:auto;min-width:0}
</style>
