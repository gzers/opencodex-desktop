<script setup lang="ts">
import {computed,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {useModelsV2Store} from './store-v2'
import {originLabelV2,type ChannelV2,type EffectiveModelV2,type ModelEditV2,type ModelSelectionV2,type RulesV2} from './api-v2'
import type {ChannelPolicy,ModelPolicy,ModelProjection} from './api'
import type {ModelRowsState} from './modelRows'
const props=defineProps<{channel:ChannelV2|ChannelPolicy;effective?:readonly EffectiveModelV2[];disabled?:boolean;query?:string;selectedOnly?:boolean;models?:readonly unknown[];templates?:readonly unknown[];policy?:ModelPolicy;projection?:ModelProjection[];projectionBusy?:boolean;filtered?:boolean;state?:ModelRowsState}>()
const emit=defineEmits<{edit:[value:ModelEditV2];selection:[value:ModelSelectionV2];order:[ids:string[]]}>()
const store=useModelsV2Store(),expanded=ref<string|null>(null)
const channel=computed(()=>('models'in props.channel?props.channel:null) as ChannelV2|null)
const selected=(id:string)=>channel.value?.selection.mode==='all'||channel.value?.selection.mode==='whitelist'&&channel.value.selection.models.includes(id)
const ordered=computed(()=>{const c=channel.value;if(!c)return [];const rank=new Map(c.modelOrder.map((id,index)=>[id,index]));return [...c.models].sort((a,b)=>(rank.get(a.rawModelId)??c.models.length)-(rank.get(b.rawModelId)??c.models.length))})
const rows=computed(()=>ordered.value.filter(m=>(!props.selectedOnly||selected(m.rawModelId))&&(!props.query||(m.rawModelId+' '+m.label).toLowerCase().includes(props.query.toLowerCase()))))
const effective=(id:string)=>props.effective?.find(row=>row.channelId===props.channel.id&&row.rawModelId===id)
function toggle(id:string,checked:boolean){if(props.disabled||!channel.value)return;const ids=ordered.value.filter(m=>selected(m.rawModelId)).map(m=>m.rawModelId);emit('selection',{mode:'whitelist',models:checked?[...new Set([...ids,id])]:ids.filter(value=>value!==id)})}
function updateRules(id:string,next:RulesV2){if(props.disabled)return;const before=store.explicitRules(props.channel.id,id);for(const field of Object.keys(next) as (keyof RulesV2)[]){const value=next[field];if(JSON.stringify(value??{mode:'inherit'})===JSON.stringify(before[field]??{mode:'inherit'}))continue;if(!value||value.mode==='inherit')emit('edit',{kind:'clear_override',channelId:props.channel.id,rawModelId:id,field});else emit('edit',{kind:'set_override',channelId:props.channel.id,rawModelId:id,field,value:value.value})}}
function move(id:string,delta:number){if(props.disabled)return;const ids=ordered.value.map(m=>m.rawModelId),at=ids.indexOf(id),to=at+delta;if(at<0||to<0||to>=ids.length)return;[ids[at],ids[to]]=[ids[to]!,ids[at]!];emit('order',ids)}
function protocolText(id:string){const p=effective(id)?.protocol;return !p?'尚未核对':p.kind==='resolved'?p.value:p.kind==='missing'?'待配置':p.value+' · 暂不支持'}
</script>
<template>
  <div class="channel-model-table-v2">
    <p v-if="!channel" role="alert" class="model-error">渠道格式暂不支持，请重新读取新版渠道；未转换旧配置。</p>
    <template v-else>
      <p class="model-muted">勾选控制公开范围；配置只记录明确人工编辑。当前有效值来自服务端快照，未保存编辑不冒充已生效。</p>
      <div class="model-table-scroll"><table class="model-table"><thead><tr><th>公开</th><th>模型</th><th>请求协议与来源</th><th>配置与顺序</th></tr></thead><tbody>
        <template v-for="(model,index) in rows" :key="model.rawModelId">
          <tr><td><input type="checkbox" :aria-label="'公开模型 '+model.rawModelId" :checked="selected(model.rawModelId)" :disabled="disabled" @change="toggle(model.rawModelId,($event.target as HTMLInputElement).checked)" /></td><td><strong>{{model.label||model.rawModelId}}</strong><code>{{model.rawModelId}}</code><small>{{model.observed?'已观察到':'手工/已保存 ID，未要求先发现'}}</small></td><td><span>{{protocolText(model.rawModelId)}}</span><small>{{originLabelV2(effective(model.rawModelId)?.fieldOrigins.protocol)}}</small><small v-if="model.binding">{{model.binding.kind==='manual'?'人工指定模版':'读取自动匹配'}} · {{model.binding.reference.revisionId}}</small></td><td><div class="model-inline"><UiButton size="sm" :aria-expanded="expanded===model.rawModelId" @click="expanded=expanded===model.rawModelId?null:model.rawModelId">配置</UiButton><UiButton size="sm" :disabled="disabled||index===0" @click="move(model.rawModelId,-1)">上移</UiButton><UiButton size="sm" :disabled="disabled||index===rows.length-1" @click="move(model.rawModelId,1)">下移</UiButton></div></td></tr>
          <tr v-if="expanded===model.rawModelId"><td colspan="4"><h3>单模型人工设置 · {{model.rawModelId}}</h3><ModelRuleFields :model-value="store.explicitRules(channel.id,model.rawModelId)" :effective-v2="effective(model.rawModelId)" :disabled="disabled" @update:model-value="updateRules(model.rawModelId,$event)" /><details><summary>实际来源证据与精确模版引用</summary><dl><template v-for="(field,key) in model.source.fields" :key="key"><dt>{{key}}</dt><dd>{{field.evidence.kind}} · {{field.evidence.pointer}} · {{field.evidence.sourceDigest}}</dd></template></dl><p v-if="model.binding">{{model.binding.reference.templateId}} / {{model.binding.reference.revisionId}} / {{model.binding.reference.digest}}</p><p v-else>未引用保存模版。</p></details></td></tr>
        </template>
      </tbody></table></div>
      <p v-if="!rows.length" class="model-muted">当前筛选下没有模型；不会清空原始 ID 或配置。</p>
    </template>
  </div>
</template>
<style scoped>
.channel-model-table-v2{min-width:0}.channel-model-table-v2 .model-table-scroll{overflow:auto}.channel-model-table-v2 table{width:100%;min-width:580px}.channel-model-table-v2 th,.channel-model-table-v2 td{padding:10px;text-align:left;vertical-align:top;border-bottom:1px solid var(--border)}.channel-model-table-v2 code,.channel-model-table-v2 small{display:block;overflow-wrap:anywhere;color:var(--muted)}.channel-model-table-v2 dd{margin:0;overflow-wrap:anywhere}.channel-model-table-v2 h3{font-size:var(--text-body)}
</style>
